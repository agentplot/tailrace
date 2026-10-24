//! Spec layout-store: the store seam over an OCI image layout on disk,
//! each scenario on a new layout in a temporary directory: the
//! conformance suite's store group over a [`LayoutRig`], and the
//! scenarios of the spec the group does not play (design D6).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Barrier;

use serde_json::{json, Value};
use tailrace::profiles::store::layout::Layout;
use tailrace_conformance::seams::store::{self, StoreRig};
use tailrace_conformance::{run, Rig};
use tailrace_seams::{
    Answer, Artifact, ArtifactMeta, ComponentName, Digest, Ecosystem, EcosystemName, Envelope,
    Held, IndexMeta, Manifest, Name, Not, Put, Seam, Signature, SignerId, Store, Version, Why,
    IN_TOTO,
};
use tempfile::TempDir;

const IMAGE_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
const IMAGE_INDEX: &str = "application/vnd.oci.image.index.v1+json";
const REF_NAME: &str = "org.opencontainers.image.ref.name";
const SUBJECT: &str = "io.github.agentplot.tailrace.subject";

/// A layout made in a new temporary directory.
fn fresh() -> (TempDir, Layout) {
    let dir = TempDir::new().expect("a temporary directory");
    let layout = Layout::create(dir.path().join("layout")).expect("a layout made");
    (dir, layout)
}

fn meta(name: &str) -> ArtifactMeta {
    ArtifactMeta::new(
        Ecosystem::new("npm"),
        EcosystemName::new(name),
        Version::new("1.3.0"),
    )
    .unwrap()
}

/// An artifact of `name` over `bytes`, its bytes put first.
fn artifact(store: &dyn Store, name: &str, bytes: &[u8]) -> Manifest {
    Manifest::Artifact {
        meta: meta(name),
        layers: vec![store.put_bytes(bytes).unwrap().digest],
    }
}

fn envelope(says: &str) -> Envelope {
    Envelope {
        payload_type: IN_TOTO.into(),
        payload: says.as_bytes().to_vec(),
        signature: Signature {
            signer: SignerId::new("a tester"),
            sig: vec![7, 7],
        },
    }
}

fn blob(layout: &Layout, digest: &Digest) -> PathBuf {
    layout.root().join("blobs/sha256").join(digest.hex())
}

fn index(layout: &Layout) -> Value {
    serde_json::from_slice(&fs::read(layout.root().join("index.json")).unwrap()).unwrap()
}

fn entries(layout: &Layout) -> Vec<Value> {
    index(layout)["manifests"].as_array().unwrap().clone()
}

/// Every file under `dir`, by its path from `dir`, with its bytes.
fn files(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut found = Vec::new();
    let mut dirs = vec![dir.to_path_buf()];
    while let Some(at) = dirs.pop() {
        for entry in fs::read_dir(&at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
            } else {
                let bytes = fs::read(&path).unwrap();
                found.push((path.strip_prefix(dir).unwrap().to_path_buf(), bytes));
            }
        }
    }
    found.sort();
    found
}

fn copy(from: &Path, to: &Path) {
    for (path, bytes) in files(from) {
        let target = to.join(path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, bytes).unwrap();
    }
}

fn unreadable<T: std::fmt::Debug>(answer: Answer<T>) -> tailrace_seams::Unreadable {
    match answer {
        Err(Not::Unreadable(u)) => u,
        other => panic!("answered {other:?}, not unreadable"),
    }
}

fn refused(answer: Answer<impl std::fmt::Debug>) -> Why {
    match answer {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source.seam, Seam::Store);
            r.why
        }
        other => panic!("answered {other:?}, not refused"),
    }
}

/// The store group's rig: a new layout in a new temporary directory.
struct LayoutRig {
    _dir: TempDir,
    layout: Layout,
}

impl LayoutRig {
    fn new() -> Self {
        let (dir, layout) = fresh();
        LayoutRig { _dir: dir, layout }
    }
}

impl Rig for LayoutRig {
    fn profile(&self) -> String {
        self.layout.source().profile.clone()
    }
}

impl StoreRig for LayoutRig {
    fn store(&self) -> &dyn Store {
        &self.layout
    }

    /// The layout's directory removed: every operation answers with what
    /// opening its `oci-layout` answers.
    fn unreachable(&self) -> String {
        fs::remove_dir_all(self.layout.root()).expect("the layout removed");
        fs::File::open(self.layout.root().join("oci-layout"))
            .expect_err("the layout is gone")
            .to_string()
    }
}

#[test]
fn the_layout_keeps_the_stores_contract() {
    let report = run(store::GROUP, &store::scenarios(), &|| {
        Box::new(LayoutRig::new())
    });
    report.assert();
    assert!(
        report.profile.starts_with("OCI layout "),
        "the report names {}",
        report.profile
    );
    assert_eq!(report.ran, store::scenarios().len());
    assert_eq!(report.not_applicable, [], "{report}");
}

#[test]
fn a_layout_made() {
    let dir = TempDir::new().unwrap();
    let layout = Layout::create(dir.path()).unwrap();
    let marker: Value =
        serde_json::from_slice(&fs::read(dir.path().join("oci-layout")).unwrap()).unwrap();
    assert_eq!(marker, json!({"imageLayoutVersion": "1.0.0"}));
    let index = index(&layout);
    assert_eq!(index["schemaVersion"], 2);
    assert_eq!(index["mediaType"], IMAGE_INDEX);
    assert_eq!(index["manifests"], json!([]));
    let blobs = dir.path().join("blobs/sha256");
    assert!(blobs.is_dir());
    assert_eq!(fs::read_dir(blobs).unwrap().count(), 0);
}

#[test]
fn a_layout_is_never_made_by_an_operation() {
    let (dir, layout) = fresh();
    fs::remove_dir_all(layout.root()).unwrap();
    let words = fs::File::open(layout.root().join("oci-layout"))
        .unwrap_err()
        .to_string();
    let u = unreadable(layout.put_bytes(b"site"));
    assert_eq!(u.source, *layout.source());
    assert_eq!(u.said.as_deref(), Some(words.as_str()));
    assert!(!layout.root().exists(), "an operation made the layout");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[test]
fn a_layout_another_tool_wrote() {
    let (_dir, theirs) = fresh();
    let (_other, plain) = fresh();
    let others = json!([
        {
            "mediaType": "application/vnd.docker.distribution.manifest.list.v2+json",
            "digest": Digest::of(b"their list"),
            "size": 10,
            "annotations": {"org.example.kept": "as written"},
        },
        {
            "mediaType": "application/vnd.example.thing+json",
            "digest": "sha512:00ff",
            "size": 2,
            "platform": {"architecture": "arm64", "os": "linux"},
        },
    ]);
    fs::write(
        theirs.root().join("index.json"),
        json!({"schemaVersion": 2, "mediaType": IMAGE_INDEX, "manifests": others}).to_string(),
    )
    .unwrap();
    let their_blob = theirs.root().join("blobs/sha256").join("0".repeat(64));
    fs::write(&their_blob, b"no entry names me").unwrap();

    let ask = |store: &dyn Store| {
        let put = store.put_bytes(b"site").unwrap();
        let manifest = artifact(store, "site", b"site");
        let artifact = store.put_artifact(&manifest).unwrap();
        let attached = store
            .attach(&artifact.digest, &envelope("it passed"))
            .unwrap();
        let name = Name::new("site-v1");
        let named = store.name(&name, &artifact.digest, None);
        let moved = store.name(&name, &put.digest, None);
        (
            put.clone(),
            artifact.clone(),
            attached,
            store.claims_of(&artifact.digest),
            named,
            moved,
            store.resolve(&name),
            store.read(&artifact.digest),
            store.read(&put.digest),
            store.resolve(&Name::new("never-made")),
        )
    };
    // Each answer as it reads with the layout's path, which every answer
    // names, put aside.
    let answers = |layout: &Layout| {
        format!("{:?}", ask(layout)).replace(&layout.root().display().to_string(), "<layout>")
    };
    assert_eq!(answers(&theirs), answers(&plain));

    let held = entries(&theirs);
    assert_eq!(held[..2], others.as_array().unwrap()[..]);
    assert_eq!(held.len(), 2 + entries(&plain).len());
    assert_eq!(fs::read(&their_blob).unwrap(), b"no entry names me");
}

#[test]
fn where_bytes_are_held() {
    let (_dir, layout) = fresh();
    let put = layout.put_bytes(b"the bytes of an artifact").unwrap();
    let path = blob(&layout, &put.digest);
    assert_eq!(put.digest, Digest::of(b"the bytes of an artifact"));
    assert_eq!(fs::read(&path).unwrap(), b"the bytes of an artifact");
    let holding: Vec<PathBuf> = files(&layout.root().join("blobs"))
        .into_iter()
        .filter(|(_, bytes)| bytes == b"the bytes of an artifact")
        .map(|(path, _)| path)
        .collect();
    assert_eq!(holding, [Path::new("sha256").join(put.digest.hex())]);

    let modified = fs::metadata(&path).unwrap().modified().unwrap();
    let again = layout.put_bytes(b"the bytes of an artifact").unwrap();
    assert_eq!(
        again,
        Put {
            digest: put.digest.clone(),
            held_already: true
        }
    );
    assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), modified);
}

#[test]
fn a_blob_altered_on_disk_is_read_and_put_again() {
    let (_dir, layout) = fresh();
    let digest = layout.put_bytes(b"site").unwrap().digest;
    let path = blob(&layout, &digest);
    fs::write(&path, b"altered").unwrap();
    let both = Why::OtherBytes {
        expected: digest.clone(),
        answered: Digest::of(b"altered"),
    };
    assert_eq!(refused(layout.read(&digest)), both);
    assert_eq!(fs::read(&path).unwrap(), b"altered");
    assert_eq!(refused(layout.put_bytes(b"site")), both);
    assert_eq!(fs::read(&path).unwrap(), b"altered");
}

#[test]
fn an_index_read_back() {
    let (_dir, layout) = fresh();
    let artifacts: Vec<Artifact> = ["web", "cli"]
        .into_iter()
        .map(|name| Artifact {
            digest: layout
                .put_artifact(&artifact(&layout, name, name.as_bytes()))
                .unwrap()
                .digest,
            meta: meta(name),
        })
        .collect();
    let index = Manifest::Index(
        IndexMeta::new(
            ComponentName::new("acme"),
            Version::new("1.3.0"),
            artifacts.clone(),
        )
        .unwrap(),
    );
    let put = layout.put_artifact(&index).unwrap();
    match layout.read(&put.digest).unwrap() {
        Held::Manifest(Manifest::Index(read)) => {
            assert_eq!(read.component(), &ComponentName::new("acme"));
            assert_eq!(read.version(), &Version::new("1.3.0"));
            assert_eq!(read.artifacts(), &artifacts[..]);
        }
        held => panic!("an index read back as {held:?}"),
    }
}

#[test]
fn the_same_artifact_on_two_hosts() {
    let (_a, first) = fresh();
    let (_b, second) = fresh();
    let puts: Vec<Digest> = [&first, &second]
        .map(|layout| {
            let manifest = artifact(layout, "site", b"site");
            layout.put_artifact(&manifest).unwrap().digest
        })
        .into();
    assert_eq!(puts[0], puts[1]);
    assert_eq!(
        fs::read(blob(&first, &puts[0])).unwrap(),
        fs::read(blob(&second, &puts[1])).unwrap()
    );
}

#[test]
fn an_artifact_a_registry_can_read() {
    let (_dir, layout) = fresh();
    let manifest = artifact(&layout, "site", b"site");
    let put = layout.put_artifact(&manifest).unwrap();
    let bytes = fs::read(blob(&layout, &put.digest)).unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["schemaVersion"], 2);
    assert_eq!(json["mediaType"], IMAGE_MANIFEST);
    assert_eq!(
        json["artifactType"],
        "application/vnd.agentplot.tailrace.artifact.npm"
    );
    let named: Vec<&Value> = std::iter::once(&json["config"])
        .chain(json["layers"].as_array().unwrap())
        .collect();
    assert_eq!(named.len(), 2);
    for descriptor in named {
        let digest = Digest::parse(descriptor["digest"].as_str().unwrap()).unwrap();
        let held = fs::read(blob(&layout, &digest)).unwrap();
        assert_eq!(Digest::of(&held), digest);
        assert_eq!(descriptor["size"], held.len());
    }
    assert_eq!(
        json["config"]["mediaType"],
        "application/vnd.agentplot.tailrace.meta.v1+json"
    );
    assert_eq!(json["layers"][0]["digest"], Digest::of(b"site").as_str());
    assert_eq!(
        entries(&layout),
        [json!({
            "mediaType": IMAGE_MANIFEST,
            "digest": put.digest,
            "size": bytes.len(),
            "artifactType": "application/vnd.agentplot.tailrace.artifact.npm",
        })]
    );
}

#[test]
fn an_artifact_naming_bytes_not_held() {
    let (_dir, layout) = fresh();
    let missing = Digest::of(b"never put");
    let before = files(layout.root());
    let manifests = [
        Manifest::Artifact {
            meta: meta("site"),
            layers: vec![missing.clone()],
        },
        Manifest::Index(
            IndexMeta::new(
                ComponentName::new("acme"),
                Version::new("1.3.0"),
                vec![Artifact {
                    digest: missing.clone(),
                    meta: meta("site"),
                }],
            )
            .unwrap(),
        ),
    ];
    for manifest in manifests {
        assert_eq!(
            layout.put_artifact(&manifest),
            Err(Not::absent(layout.source(), missing.to_string()))
        );
        assert_eq!(files(layout.root()), before);
    }
}

#[test]
fn the_same_attestation_attached_twice() {
    let (_dir, layout) = fresh();
    let subject = layout.put_bytes(b"site").unwrap().digest;
    let attestation = envelope("it passed");
    let first = layout.attach(&subject, &attestation).unwrap();
    let second = layout.attach(&subject, &attestation).unwrap();
    assert_eq!(second.digest, first.digest);
    assert_eq!(second.digest, attestation.digest());
    assert!(second.held_already);
    assert_eq!(layout.claims_of(&subject), Ok(vec![attestation]));
    let attached: Vec<Value> = entries(&layout)
        .into_iter()
        .filter(|e| e["annotations"][SUBJECT] == subject.as_str())
        .collect();
    assert_eq!(attached.len(), 1);
}

#[test]
fn an_attestation_and_claims_of_for_a_digest_not_held() {
    let (_dir, layout) = fresh();
    let missing = Digest::of(b"never put");
    let before = files(layout.root());
    let absent = Not::absent(layout.source(), missing.to_string());
    assert_eq!(
        layout.attach(&missing, &envelope("nothing")),
        Err(absent.clone())
    );
    assert_eq!(files(layout.root()), before);
    assert_eq!(layout.claims_of(&missing), Err(absent));
}

#[test]
fn attestations_travel_with_a_copy_of_the_layout() {
    let (dir, layout) = fresh();
    let subject = layout
        .put_artifact(&artifact(&layout, "site", b"site"))
        .unwrap()
        .digest;
    let attached = [envelope("it passed"), envelope("it was reviewed")];
    for attestation in &attached {
        layout.attach(&subject, attestation).unwrap();
    }
    let elsewhere = dir.path().join("elsewhere");
    copy(layout.root(), &elsewhere);
    let copied = Layout::open(&elsewhere).unwrap();
    assert_eq!(copied.claims_of(&subject), layout.claims_of(&subject));
    assert_eq!(copied.claims_of(&subject).unwrap(), attached);
}

#[test]
fn a_name_made_and_moved() {
    let (_dir, layout) = fresh();
    let name = Name::new("site-v1");
    let first = layout
        .put_artifact(&artifact(&layout, "site", b"one"))
        .unwrap()
        .digest;
    let second = layout
        .put_artifact(&artifact(&layout, "site", b"two"))
        .unwrap()
        .digest;
    layout.name(&name, &first, None).unwrap();
    layout.name(&name, &second, Some(&first)).unwrap();
    assert_eq!(layout.resolve(&name), Ok(second.clone()));
    let named: Vec<Value> = entries(&layout)
        .into_iter()
        .filter(|e| e["annotations"][REF_NAME] == "site-v1")
        .collect();
    assert_eq!(named.len(), 1);
    assert_eq!(named[0]["digest"], second.as_str());
}

#[test]
fn two_writers_move_one_name() {
    let (_dir, layout) = fresh();
    let [at, a, b] =
        [b"at" as &[u8], b"a", b"b"].map(|bytes| layout.put_bytes(bytes).unwrap().digest);
    for round in 0..16 {
        let name = Name::new(format!("site-v{round}"));
        layout.name(&name, &at, None).unwrap();
        let start = Barrier::new(2);
        let answers: Vec<(Digest, Answer<()>)> = std::thread::scope(|s| {
            let moves: Vec<_> = [&a, &b]
                .map(|to| {
                    let (name, at, start) = (&name, &at, &start);
                    let handle = Layout::open(layout.root()).unwrap();
                    s.spawn(move || {
                        start.wait();
                        (to.clone(), handle.name(name, to, Some(at)))
                    })
                })
                .into();
            moves.into_iter().map(|m| m.join().unwrap()).collect()
        });
        let won: Vec<&Digest> = answers
            .iter()
            .filter(|(_, answer)| answer.is_ok())
            .map(|(to, _)| to)
            .collect();
        assert_eq!(won.len(), 1, "round {round}: {answers:?}");
        let winner = won[0].clone();
        let (_, lost) = answers.into_iter().find(|(to, _)| *to != winner).unwrap();
        assert_eq!(
            refused(lost),
            Why::NameAt {
                at: Some(winner.clone())
            }
        );
        assert_eq!(layout.resolve(&name), Ok(winner));
    }
}

#[test]
fn a_name_moved_to_a_digest_not_held() {
    let (_dir, layout) = fresh();
    let name = Name::new("site-v1");
    let at = layout.put_bytes(b"at").unwrap().digest;
    layout.name(&name, &at, None).unwrap();
    let missing = Digest::of(b"never put");
    assert_eq!(
        layout.name(&name, &missing, Some(&at)),
        Err(Not::absent(layout.source(), missing.to_string()))
    );
    assert_eq!(layout.resolve(&name), Ok(at));
}

#[test]
fn a_malformed_index() {
    let (_dir, layout) = fresh();
    let path = layout.root().join("index.json");
    fs::write(&path, "[\"not an image index\"]").unwrap();
    let u = unreadable(layout.resolve(&Name::new("site-v1")));
    assert_eq!(u.source, *layout.source());
    assert_eq!(u.what, path.display().to_string());
    let words = u.said.unwrap();
    assert!(words.contains("invalid type"), "{words}");
    assert_eq!(fs::read(&path).unwrap(), b"[\"not an image index\"]");
}
