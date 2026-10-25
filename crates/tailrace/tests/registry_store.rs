//! Spec registry-store: the store seam over an OCI registry, against a
//! zot and a reference distribution each started for the test (design
//! D12): the conformance suite's store group over each, and the
//! scenarios of the spec the group does not play. Needs the registries
//! the devenv shell pins, so it is in the tools layer.

#![cfg(feature = "tools")]

mod registry;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::rc::Rc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use registry::{cache_manifest, Kind, Proxy, RegistryRig, Server, PASSWORD, USER};
use serde_json::{json, Value};
use tailrace::profiles::store::layout::Layout;
use tailrace::profiles::store::registry::Registry;
use tailrace_conformance::run;
use tailrace_conformance::seams::store::{self, BYTES};
use tailrace_core::{copy, follow, End, Followed};
use tailrace_fakes::FakeKeeper;
use tailrace_seams::{
    canonical, Answer, Artifact, ArtifactMeta, ChannelName, Claim, ClaimKind, ComponentName,
    Digest, Ecosystem, EcosystemName, Envelope, Held, IndexMeta, Keeper, Manifest, Name, Not,
    Revision, Signature, SignerId, Store, Version, Why, IN_TOTO,
};
use tempfile::TempDir;

const IMAGE_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
const IMAGE_INDEX: &str = "application/vnd.oci.image.index.v1+json";
const ENVELOPE: &str = "application/vnd.agentplot.tailrace.envelope.v1+json";
const EMPTY: &str = "application/vnd.oci.empty.v1+json";
const COSIGN: &str = "application/vnd.dev.cosign.artifact.sig.v1+json";

/// What changes the process's environment, which only the credential
/// tests do, one at a time.
static ENVIRONMENT: Mutex<()> = Mutex::new(());

fn meta(name: &str, version: &str) -> ArtifactMeta {
    ArtifactMeta::new(
        Ecosystem::new("npm"),
        EcosystemName::new(name),
        Version::new(version),
    )
    .unwrap()
}

/// An artifact of `name` at `version` over `bytes`, its bytes put first.
fn artifact(store: &dyn Store, name: &str, version: &str, bytes: &[u8]) -> Manifest {
    Manifest::Artifact {
        meta: meta(name, version),
        layers: vec![store.put_bytes(bytes).unwrap().digest],
    }
}

/// An artifact put, by its digest.
fn put(store: &dyn Store, version: &str) -> Digest {
    store
        .put_artifact(&artifact(store, "site", version, version.as_bytes()))
        .unwrap()
        .digest
}

/// An attestation about `subject`, as a signer would make it.
fn attestation(subject: &Digest, says: &str) -> Envelope {
    let claim = Claim {
        subject: subject.clone(),
        kind: ClaimKind::Verdict,
        revision: Revision::new("4f2a9c1"),
        build: None,
        body: json!({ "says": says }),
    };
    Envelope::signed(
        IN_TOTO,
        claim.payload(),
        Signature {
            signer: SignerId::new("a tester"),
            keyid: String::new(),
            sig: vec![7, 7],
        },
    )
}

fn refused<T: std::fmt::Debug>(answer: Answer<T>) -> tailrace_seams::Refusal {
    match answer {
        Err(Not::Refused(r)) => r,
        other => panic!("answered {other:?}, not refused"),
    }
}

/// What a registry answers a plain `GET` of `url` accepting `accept`.
fn get(url: &str, accept: &str) -> (u16, Vec<u8>) {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .into();
    let mut answered = agent.get(url).header("accept", accept).call().unwrap();
    let body = answered.body_mut().read_to_vec().unwrap();
    (answered.status().as_u16(), body)
}

/// What a registry answers a plain `PUT` of `bytes` to `url`.
fn put_raw(url: &str, content_type: &str, bytes: &[u8]) -> u16 {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .into();
    agent
        .put(url)
        .header("content-type", content_type)
        .send(bytes)
        .unwrap()
        .status()
        .as_u16()
}

fn group(kind: Kind) {
    let server = Rc::new(Server::start(kind));
    let report = run(store::GROUP, &store::scenarios(), &|| {
        Box::new(RegistryRig::new(&server))
    });
    report.assert();
    assert!(
        report.profile.starts_with("OCI registry 127.0.0.1:"),
        "the report names {}",
        report.profile
    );
    assert_eq!(report.ran, store::scenarios().len());
    assert_eq!(report.not_applicable, [], "{report}");
}

#[test]
fn zot_keeps_the_stores_contract() {
    group(Kind::Zot);
}

#[test]
fn the_reference_distribution_keeps_the_stores_contract() {
    group(Kind::Distribution);
}

#[test]
fn both_registries_start_and_answer_a_name_never_made_as_absent() {
    for kind in [Kind::Zot, Kind::Distribution] {
        let server = Server::start(kind);
        let store = server.store();
        assert_eq!(
            store.resolve(&Name::new("never-made")),
            Err(Not::absent(store.source(), "name never-made")),
            "{}",
            kind.name()
        );
    }
}

#[test]
fn the_reference_distribution_answers_no_referrers_api() {
    let server = Server::start(Kind::Distribution);
    let store = server.store();
    let subject = put(&store, "1.0.0");
    let (status, _) = get(&format!("{}referrers/{subject}", store.base()), IMAGE_INDEX);
    assert_eq!(status, 404);
}

/// Each scenario below over zot and over the reference distribution.
macro_rules! on_both {
    ($($scenario:ident),* $(,)?) => {
        mod zot {
            $(#[test] fn $scenario() { super::$scenario(super::Kind::Zot) })*
        }
        mod distribution {
            $(#[test] fn $scenario() { super::$scenario(super::Kind::Distribution) })*
        }
    };
}

on_both!(
    the_same_bytes_put_twice_are_uploaded_once,
    one_artifact_has_one_digest_in_a_layout_and_in_a_registry,
    a_manifest_naming_bytes_not_held_puts_nothing,
    a_blob_answered_with_other_bytes_is_refused_naming_both,
    the_same_attestation_attached_twice_is_held_once,
    another_tools_referrer_is_not_answered_and_is_left,
    a_race_lost_after_the_put_is_refused_naming_the_other_writer,
    a_read_back_at_the_former_place_within_the_bound_is_no_lost_race,
    a_tag_still_at_the_former_place_once_the_bound_passes_is_refused_naming_it,
    two_names_differing_only_in_escaped_characters_resolve_apart,
    bytes_are_named_through_a_cache_manifest,
    a_race_lost_on_a_cache_entrys_name_is_refused_naming_the_other_writer,
    a_registry_behind_a_credential_helper,
    a_version_in_a_layout_copied_into_a_registry,
    a_race_lost_on_a_channels_tag_is_repaired_from_the_ledger,
);

fn the_same_bytes_put_twice_are_uploaded_once(kind: Kind) {
    let server = Server::start(kind);
    let proxy = Proxy::before(&server);
    let store = proxy.store(&server.repository());
    let first = store.put_bytes(BYTES).unwrap();
    assert!(!first.held_already);
    assert!(
        proxy.passed().iter().any(|p| p.method == "PUT"),
        "the first put uploaded nothing"
    );
    let second = store.put_bytes(BYTES).unwrap();
    assert_eq!(second.digest, first.digest);
    assert!(second.held_already);
    let passed = proxy.passed();
    assert_eq!(passed.len(), 1, "{passed:?}");
    assert_eq!(passed[0].method, "HEAD");
    assert!(passed[0].path.ends_with(&format!("blobs/{}", first.digest)));
}

fn one_artifact_has_one_digest_in_a_layout_and_in_a_registry(kind: Kind) {
    let dir = TempDir::new().unwrap();
    let layout = Layout::create(dir.path().join("layout")).unwrap();
    let server = Server::start(kind);
    let registry = server.store();
    let digests: Vec<Digest> = [&layout as &dyn Store, &registry]
        .map(|store| {
            store
                .put_artifact(&artifact(store, "@acme/site", "1.3.0", b"site"))
                .unwrap()
                .digest
        })
        .into();
    assert_eq!(digests[0], digests[1]);
    let held = fs::read(layout.root().join("blobs/sha256").join(digests[0].hex())).unwrap();
    let (status, bytes) = get(
        &format!("{}manifests/{}", registry.base(), digests[1]),
        IMAGE_MANIFEST,
    );
    assert_eq!(status, 200);
    assert_eq!(bytes, held);
}

fn a_manifest_naming_bytes_not_held_puts_nothing(kind: Kind) {
    let server = Server::start(kind);
    let proxy = Proxy::before(&server);
    let store = proxy.store(&server.repository());
    let held = store.put_bytes(b"held").unwrap().digest;
    let missing = Digest::of(b"never put");
    proxy.passed();
    let manifests = [
        Manifest::Artifact {
            meta: meta("site", "1.3.0"),
            layers: vec![held, missing.clone()],
        },
        Manifest::Index(
            IndexMeta::new(
                ComponentName::new("acme"),
                Version::new("1.3.0"),
                vec![Artifact {
                    digest: missing.clone(),
                    meta: meta("site", "1.3.0"),
                }],
            )
            .unwrap(),
        ),
    ];
    for manifest in manifests {
        assert_eq!(
            store.put_artifact(&manifest),
            Err(Not::absent(store.source(), missing.to_string()))
        );
        let passed = proxy.passed();
        assert!(
            passed.iter().all(|p| p.method == "HEAD"),
            "a manifest naming bytes not held wrote: {passed:?}"
        );
    }
}

fn a_blob_answered_with_other_bytes_is_refused_naming_both(kind: Kind) {
    let server = Server::start(kind);
    let proxy = Proxy::before(&server);
    let store = proxy.store(&server.repository());
    let digest = store.put_bytes(b"site").unwrap().digest;
    proxy.alter(&format!("blobs/{digest}"));
    let flipped: Vec<u8> = b"site".iter().map(|b| !b).collect();
    assert_eq!(
        refused(store.read(&digest)).why,
        Why::OtherBytes {
            expected: digest,
            answered: Digest::of(&flipped),
        }
    );
}

fn the_same_attestation_attached_twice_is_held_once(kind: Kind) {
    let server = Server::start(kind);
    let store = server.store();
    let subject = put(&store, "1.0.0");
    let envelope = attestation(&subject, "it passed");
    let first = store.attach(&subject, &envelope).unwrap();
    assert_eq!(first.digest, envelope.digest());
    assert!(!first.held_already);
    let second = store.attach(&subject, &envelope).unwrap();
    assert_eq!(second.digest, envelope.digest());
    assert!(second.held_already);
    assert_eq!(store.claims_of(&subject), Ok(vec![envelope]));
}

/// A signature another tool pushes as a referrer of `subject`, listed
/// in the subject's referrers tag where the registry answers no
/// referrers API, as that tool would list it; its digest.
fn their_referrer(store: &Registry, subject: &Digest, kind: Kind) -> Digest {
    let (_, held) = get(
        &format!("{}manifests/{subject}", store.base()),
        IMAGE_MANIFEST,
    );
    let layer = store.put_bytes(b"their signature").unwrap().digest;
    let empty = store.put_bytes(b"{}").unwrap().digest;
    let manifest = serde_json::to_vec(&json!({
        "schemaVersion": 2,
        "mediaType": IMAGE_MANIFEST,
        "artifactType": COSIGN,
        "config": {"mediaType": EMPTY, "digest": empty, "size": 2},
        "layers": [{"mediaType": "application/octet-stream", "digest": layer, "size": 15}],
        "subject": {"mediaType": IMAGE_MANIFEST, "digest": subject, "size": held.len()},
    }))
    .unwrap();
    let digest = Digest::of(&manifest);
    let url = format!("{}manifests/{digest}", store.base());
    assert_eq!(put_raw(&url, IMAGE_MANIFEST, &manifest), 201);
    if kind == Kind::Distribution {
        let tag = format!("{}manifests/sha256-{}", store.base(), subject.hex());
        let (status, bytes) = get(&tag, IMAGE_INDEX);
        let mut index: Value = if status == 200 {
            serde_json::from_slice(&bytes).unwrap()
        } else {
            json!({"schemaVersion": 2, "mediaType": IMAGE_INDEX, "manifests": []})
        };
        index["manifests"].as_array_mut().unwrap().push(json!({
            "mediaType": IMAGE_MANIFEST,
            "digest": digest,
            "size": manifest.len(),
            "artifactType": COSIGN,
        }));
        assert_eq!(
            put_raw(&tag, IMAGE_INDEX, &serde_json::to_vec(&index).unwrap()),
            201
        );
    }
    digest
}

/// The digests a subject's referrers are listed by: by the referrers
/// API on zot, and by the referrers tag on the distribution.
fn listed(store: &Registry, subject: &Digest, kind: Kind) -> Vec<(String, Option<String>)> {
    let url = match kind {
        Kind::Zot => format!("{}referrers/{subject}", store.base()),
        Kind::Distribution => format!("{}manifests/sha256-{}", store.base(), subject.hex()),
    };
    let (status, bytes) = get(&url, IMAGE_INDEX);
    assert_eq!(status, 200, "{url}: {}", String::from_utf8_lossy(&bytes));
    let index: Value = serde_json::from_slice(&bytes).unwrap();
    index["manifests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            (
                d["digest"].as_str().unwrap().to_string(),
                d["artifactType"].as_str().map(str::to_string),
            )
        })
        .collect()
}

fn another_tools_referrer_is_not_answered_and_is_left(kind: Kind) {
    let server = Server::start(kind);
    let store = server.store();
    let subject = put(&store, "1.0.0");
    let ours = attestation(&subject, "it passed");
    store.attach(&subject, &ours).unwrap();
    let theirs = their_referrer(&store, &subject, kind);
    let again = attestation(&subject, "it passed again");
    store.attach(&subject, &again).unwrap();
    let claims = store.claims_of(&subject).unwrap();
    assert_eq!(claims.len(), 2, "{claims:?}");
    assert!(claims.contains(&ours) && claims.contains(&again));
    let entries = listed(&store, &subject, kind);
    assert_eq!(entries.len(), 3, "{entries:?}");
    assert!(entries.contains(&(theirs.to_string(), Some(COSIGN.into()))));
    assert_eq!(
        entries
            .iter()
            .filter(|(_, t)| t.as_deref() == Some(ENVELOPE))
            .count(),
        2
    );
    let (status, _) = get(
        &format!("{}manifests/{theirs}", store.base()),
        IMAGE_MANIFEST,
    );
    assert_eq!(status, 200, "their referrer was removed");
    if kind == Kind::Zot {
        let (status, _) = get(
            &format!("{}manifests/sha256-{}", store.base(), subject.hex()),
            IMAGE_INDEX,
        );
        assert_eq!(
            status, 404,
            "zot answered a subject, yet a referrers tag was pushed"
        );
    }
}

#[test]
fn the_reference_distributions_referrers_tag_lists_both_attestations() {
    let server = Server::start(Kind::Distribution);
    let store = server.store();
    let subject = put(&store, "1.0.0");
    let attached = [
        attestation(&subject, "it passed"),
        attestation(&subject, "it passed again"),
    ];
    for envelope in &attached {
        store.attach(&subject, envelope).unwrap();
    }
    let (status, bytes) = get(
        &format!("{}manifests/sha256-{}", store.base(), subject.hex()),
        IMAGE_INDEX,
    );
    assert_eq!(status, 200);
    let index: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(index["mediaType"], IMAGE_INDEX);
    let entries = index["manifests"].as_array().unwrap();
    assert_eq!(entries.len(), 2, "{index}");
    for entry in entries {
        assert_eq!(entry["artifactType"], ENVELOPE);
        assert_eq!(
            entry["annotations"]["in-toto.io/predicate-type"],
            ClaimKind::Verdict.predicate_type()
        );
    }
    let claims = store.claims_of(&subject).unwrap();
    assert_eq!(claims.len(), 2);
    for envelope in &attached {
        let held = claims.iter().find(|c| *c == envelope).unwrap();
        assert_eq!(canonical(held), canonical(envelope));
    }
}

#[test]
fn a_referrers_tag_rewritten_before_the_read_back_is_unreadable() {
    let server = Server::start(Kind::Distribution);
    let proxy = Proxy::before(&server);
    let repository = server.repository();
    let mut store = proxy.store(&repository);
    let subject = put(&store, "1.0.0");
    let direct = Registry::new(&repository).unwrap().base().to_string();
    store.between_put_and_read_back(move |tag| {
        if tag.starts_with("sha256-") {
            let empty = json!({"schemaVersion": 2, "mediaType": IMAGE_INDEX, "manifests": []});
            let url = format!("{direct}manifests/{tag}");
            assert_eq!(
                put_raw(&url, IMAGE_INDEX, &serde_json::to_vec(&empty).unwrap()),
                201
            );
        }
    });
    let envelope = attestation(&subject, "it passed");
    proxy.passed();
    let tag = format!("sha256-{}", subject.hex());
    match store.attach(&subject, &envelope) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(&u.source, store.source());
            assert!(u.what.contains(&tag), "{u:?}");
            let said = u.said.unwrap();
            assert!(said.contains(&tag), "{said}");
            assert!(said.contains("sha256:"), "{said}");
        }
        other => panic!("an entry pushed over answered {other:?}"),
    }
    let pushes: Vec<_> = proxy
        .passed()
        .into_iter()
        .filter(|p| p.method == "PUT" && p.path.ends_with(&format!("manifests/{tag}")))
        .collect();
    assert_eq!(pushes.len(), 1, "{pushes:?}");
}

fn a_race_lost_after_the_put_is_refused_naming_the_other_writer(kind: Kind) {
    let server = Server::start(kind);
    let proxy = Proxy::before(&server);
    let repository = server.repository();
    let direct = Registry::new(&repository).unwrap();
    let [at, to, other] = ["1.0.0", "1.1.0", "1.2.0"].map(|v| put(&direct, v));
    let name = Name::new("site-v1");
    direct.name(&name, &at, None).unwrap();
    let mut store = proxy.store(&repository);
    let (moved, to_) = (other.clone(), to.clone());
    let writer = Registry::new(&repository).unwrap();
    store.between_put_and_read_back(move |_| {
        writer
            .name(&Name::new("site-v1"), &moved, Some(&to_))
            .expect("the other writer moves the name");
    });
    proxy.passed();
    assert_eq!(
        refused(store.name(&name, &to, Some(&at))).why,
        Why::NameAt {
            at: Some(other.clone())
        }
    );
    let puts: Vec<_> = proxy
        .passed()
        .into_iter()
        .filter(|p| p.method == "PUT")
        .collect();
    assert_eq!(puts.len(), 1, "{puts:?}");
    assert!(puts[0].path.ends_with("manifests/site-v1"));
    assert_eq!(direct.resolve(&name), Ok(other));
}

/// A name `site-v1` made at the first of two artifacts, a store through
/// the proxy whose put of the tag is followed by the tag put back at the
/// first, as a registry not yet showing a put answers it, and `shown`
/// run with the repository after; and the two artifacts.
fn put_back_after_the_put(
    kind: Kind,
    shown: impl Fn(String, Digest, Digest) + 'static,
) -> (Server, Proxy, Registry, [Digest; 2]) {
    let server = Server::start(kind);
    let proxy = Proxy::before(&server);
    let repository = server.repository();
    let direct = Registry::new(&repository).unwrap();
    let [at, to] = ["1.0.0", "1.1.0"].map(|v| put(&direct, v));
    direct.name(&Name::new("site-v1"), &at, None).unwrap();
    let mut store = proxy.store(&repository);
    let (former, moved) = (at.clone(), to.clone());
    store.between_put_and_read_back(move |tag| {
        Registry::new(&repository)
            .unwrap()
            .name(&Name::new(tag), &former, Some(&moved))
            .expect("the tag is put back at the former place");
        shown(repository.clone(), former.clone(), moved.clone());
    });
    (server, proxy, store, [at, to])
}

/// The requests `passed` names: the puts, and the reads of `site-v1`.
fn puts_and_reads(passed: &[registry::Passed]) -> (usize, usize) {
    let puts = passed.iter().filter(|p| p.method == "PUT").count();
    let reads = passed
        .iter()
        .filter(|p| p.method == "GET" && p.path.ends_with("manifests/site-v1"))
        .count();
    (puts, reads)
}

fn a_read_back_at_the_former_place_within_the_bound_is_no_lost_race(kind: Kind) {
    let (_server, proxy, store, [at, to]) =
        put_back_after_the_put(kind, |repository, former, moved| {
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(1));
                Registry::new(&repository)
                    .unwrap()
                    .name(&Name::new("site-v1"), &moved, Some(&former))
                    .expect("the registry shows the put");
            });
        });
    let name = Name::new("site-v1");
    proxy.passed();
    assert_eq!(store.name(&name, &to, Some(&at)), Ok(()));
    let (puts, reads) = puts_and_reads(&proxy.passed());
    assert_eq!(puts, 1, "the tag was put again");
    assert!(reads > 2, "the tag was read {reads} times, never again");
    assert_eq!(store.resolve(&name), Ok(to));
}

fn a_tag_still_at_the_former_place_once_the_bound_passes_is_refused_naming_it(kind: Kind) {
    let (_server, proxy, mut store, [at, to]) = put_back_after_the_put(kind, |_, _, _| {});
    let bound = Duration::from_secs(1);
    store.read_again_within(bound);
    let name = Name::new("site-v1");
    proxy.passed();
    let started = Instant::now();
    let refusal = refused(store.name(&name, &to, Some(&at)));
    assert!(
        started.elapsed() >= bound,
        "refused before the bound passed"
    );
    assert_eq!(&refusal.source, store.source());
    assert_eq!(
        refusal.why,
        Why::NameAt {
            at: Some(at.clone())
        }
    );
    let said = refusal.said.unwrap();
    assert!(
        said.contains(&at.to_string()) && said.contains(&to.to_string()),
        "{said}"
    );
    let (puts, reads) = puts_and_reads(&proxy.passed());
    assert_eq!(puts, 1, "the tag was put again");
    assert!(reads > 2, "the tag was read {reads} times, never again");
    assert_eq!(store.resolve(&name), Ok(at));
}

fn two_names_differing_only_in_escaped_characters_resolve_apart(kind: Kind) {
    let server = Server::start(kind);
    let store = server.store();
    let names = [Name::new("a/b+c"), Name::new("a_2fb+c")];
    let digests = [put(&store, "1.0.0"), put(&store, "1.1.0")];
    for (name, digest) in names.iter().zip(&digests) {
        store.name(name, digest, None).unwrap();
    }
    for (name, digest) in names.iter().zip(&digests) {
        assert_eq!(store.resolve(name).as_ref(), Ok(digest));
    }
    let (status, bytes) = get(&format!("{}tags/list", store.base()), "application/json");
    assert_eq!(status, 200);
    let tags: Value = serde_json::from_slice(&bytes).unwrap();
    let mut tags: Vec<&str> = tags["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap())
        .collect();
    tags.sort_unstable();
    assert_eq!(tags, ["a_2fb_2bc", "a_5f2fb_2bc"]);
}

fn bytes_are_named_through_a_cache_manifest(kind: Kind) {
    let server = Server::start(kind);
    let proxy = Proxy::before(&server);
    let store = proxy.store(&server.repository());
    registry::cache_entries(&store, store.bound());

    let entry = b"a cache entry under two names";
    let bytes = store.put_bytes(entry).unwrap().digest;
    let manifest = cache_manifest(entry);
    for (name, tag, writes) in [
        (
            "cache/site",
            "cache_2fsite",
            vec![Digest::of(&manifest).to_string(), "cache_2fsite".into()],
        ),
        (
            "cache/server",
            "cache_2fserver",
            vec!["cache_2fserver".into()],
        ),
    ] {
        proxy.passed();
        store.name(&Name::new(name), &bytes, None).unwrap();
        let wrote: Vec<String> = proxy
            .passed()
            .into_iter()
            .filter(|p| p.method != "HEAD" && p.method != "GET")
            .map(|p| format!("{} {}", p.method, p.path))
            .collect();
        let expected: Vec<String> = writes
            .iter()
            .map(|r| {
                format!(
                    "PUT /v2/{}/manifests/{r}",
                    store.repository().split_once('/').unwrap().1
                )
            })
            .collect();
        assert_eq!(wrote, expected, "naming {name}");
        let (status, held) = get(&format!("{}manifests/{tag}", store.base()), IMAGE_MANIFEST);
        assert_eq!(status, 200);
        assert_eq!(
            held,
            manifest,
            "{tag} is at {}",
            String::from_utf8_lossy(&held)
        );
    }

    let missing = Digest::of(b"never put");
    proxy.passed();
    assert_eq!(
        store.name(&Name::new("cache/none"), &missing, None),
        Err(Not::absent(store.source(), missing.to_string()))
    );
    let passed = proxy.passed();
    assert!(
        passed
            .iter()
            .all(|p| p.method == "HEAD" || p.method == "GET"),
        "a name at a digest not held wrote: {passed:?}"
    );
}

fn a_race_lost_on_a_cache_entrys_name_is_refused_naming_the_other_writer(kind: Kind) {
    let server = Server::start(kind);
    let proxy = Proxy::before(&server);
    let repository = server.repository();
    let direct = Registry::new(&repository).unwrap();
    let [at, to, other] =
        [b"one".as_slice(), b"two", b"three"].map(|bytes| direct.put_bytes(bytes).unwrap().digest);
    let name = Name::new("cache/site");
    direct.name(&name, &at, None).unwrap();
    let mut store = proxy.store(&repository);
    let (moved, to_) = (other.clone(), to.clone());
    let writer = Registry::new(&repository).unwrap();
    store.between_put_and_read_back(move |_| {
        writer
            .name(&Name::new("cache/site"), &moved, Some(&to_))
            .expect("the other writer moves the name");
    });
    proxy.passed();
    assert_eq!(
        refused(store.name(&name, &to, Some(&at))).why,
        Why::NameAt {
            at: Some(other.clone())
        }
    );
    let tagged: Vec<_> = proxy
        .passed()
        .into_iter()
        .filter(|p| p.method == "PUT" && p.path.ends_with("manifests/cache_2fsite"))
        .collect();
    assert_eq!(tagged.len(), 1, "{tagged:?}");
    assert_eq!(direct.resolve(&name), Ok(other));
}

/// Every file under `dir`, by its path from `dir`, with its bytes.
fn files(dir: &std::path::Path) -> Vec<(std::path::PathBuf, Vec<u8>)> {
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

fn a_registry_behind_a_credential_helper(kind: Kind) {
    let _environment = ENVIRONMENT.lock().unwrap_or_else(|e| e.into_inner());
    let server = Server::asking(kind);
    let proxy = Proxy::before(&server);
    let repository = server.repository();
    let path = std::env::var_os("PATH").unwrap_or_default();

    let none = TempDir::new().unwrap();
    fs::write(none.path().join("config.json"), "{}").unwrap();
    std::env::set_var("DOCKER_CONFIG", none.path());
    let store = proxy.store(&repository);
    let r = refused(store.put_bytes(BYTES));
    assert_eq!(&r.source, store.source());
    let words = r.said.unwrap();
    assert!(
        words.to_lowercase().contains("unauthorized"),
        "the registry's words were not carried: {words}"
    );

    let helpers = TempDir::new().unwrap();
    let asked = helpers.path().join("asked");
    let helper = helpers.path().join("docker-credential-tailrace-tools");
    fs::write(
        &helper,
        format!(
            "#!/bin/sh\nread -r server\necho \"$1 $server\" >> {}\n\
             printf '{{\"Username\":\"{USER}\",\"Secret\":\"{PASSWORD}\"}}'\n",
            asked.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&helper, fs::Permissions::from_mode(0o755)).unwrap();
    let config = TempDir::new().unwrap();
    let written = format!(
        r#"{{"credHelpers":{{"127.0.0.1:{}":"tailrace-tools"}}}}"#,
        proxy.port
    );
    fs::write(config.path().join("config.json"), &written).unwrap();
    std::env::set_var("DOCKER_CONFIG", config.path());
    let mut paths = vec![helpers.path().to_path_buf()];
    paths.extend(std::env::split_paths(&path));
    std::env::set_var("PATH", std::env::join_paths(paths).unwrap());

    let store = proxy.store(&repository);
    proxy.passed();
    let answers = (
        store.put_artifact(&artifact(&store, "site", "1.0.0", BYTES)),
        store.resolve(&Name::new("never-made")),
    );
    std::env::set_var("PATH", &path);
    std::env::remove_var("DOCKER_CONFIG");

    let digest = answers
        .0
        .expect("an artifact put behind a credential")
        .digest;
    assert!(matches!(answers.1, Err(Not::Absent(_))), "{:?}", answers.1);
    assert_eq!(
        store.read(&digest).map(|h| matches!(h, Held::Manifest(_))),
        Ok(true)
    );
    let challenged = proxy
        .passed()
        .into_iter()
        .filter(|p| p.status == 401)
        .count();
    assert_eq!(challenged, 1, "the registry challenged more than once");
    let asked = fs::read_to_string(&asked).unwrap();
    assert_eq!(asked.lines().count(), 1, "{asked}");
    assert_eq!(asked.trim(), format!("get 127.0.0.1:{}", proxy.port));
    assert_eq!(
        files(config.path()),
        [("config.json".into(), written.into_bytes())]
    );
    for (_, bytes) in files(config.path()).iter().chain(&files(none.path())) {
        assert!(!String::from_utf8_lossy(bytes).contains(PASSWORD));
    }
}

/// A version of two artifacts published into `store` as `layout_store.rs`
/// builds one, a provenance on each artifact and an acceptance on the
/// index, and named `web-v1.3.0`; its name, its index and its artifacts.
fn published(store: &dyn Store) -> (Name, Digest, Vec<Digest>) {
    let mut artifacts = Vec::new();
    for name in ["site", "server"] {
        let digest = store
            .put_artifact(&artifact(store, name, "1.3.0", name.as_bytes()))
            .unwrap()
            .digest;
        store
            .attach(&digest, &attestation(&digest, "it was built here"))
            .unwrap();
        artifacts.push(Artifact {
            digest,
            meta: meta(name, "1.3.0"),
        });
    }
    let digests = artifacts.iter().map(|a| a.digest.clone()).collect();
    let index = store
        .put_artifact(&Manifest::Index(
            IndexMeta::new(ComponentName::new("web"), Version::new("1.3.0"), artifacts).unwrap(),
        ))
        .unwrap()
        .digest;
    store
        .attach(&index, &attestation(&index, "it was accepted"))
        .unwrap();
    let name = Name::new("web-v1.3.0");
    store.name(&name, &index, None).unwrap();
    (name, index, digests)
}

fn a_version_in_a_layout_copied_into_a_registry(kind: Kind) {
    let dir = TempDir::new().unwrap();
    let layout = Layout::create(dir.path().join("layout")).unwrap();
    let (name, index, artifacts) = published(&layout);
    let server = Server::start(kind);
    let proxy = Proxy::before(&server);
    let registry = proxy.store(&server.repository());
    let ends = || {
        (
            End {
                store: &layout,
                source: layout.source(),
            },
            End {
                store: &registry,
                source: registry.source(),
            },
        )
    };
    let (from, to) = ends();
    let first = copy(from, to, &name).unwrap();
    assert_eq!(first.index, index);
    assert_eq!(registry.resolve(&name), Ok(index.clone()));
    for digest in std::iter::once(&index).chain(&artifacts) {
        assert_eq!(registry.read(digest), layout.read(digest), "{digest}");
        let held: Vec<Vec<u8>> = layout
            .claims_of(digest)
            .unwrap()
            .iter()
            .map(canonical)
            .collect();
        let carried: Vec<Vec<u8>> = registry
            .claims_of(digest)
            .unwrap()
            .iter()
            .map(canonical)
            .collect();
        assert_eq!(held.len(), 1);
        assert_eq!(
            carried, held,
            "the claims on {digest} were not carried byte for byte"
        );
    }
    proxy.passed();
    let (from, to) = ends();
    let again = copy(from, to, &name).unwrap();
    assert!(again.name_held_already);
    assert!(again.written.iter().all(|w| w.held_already), "{again:?}");
    let wrote: Vec<_> = proxy
        .passed()
        .into_iter()
        .filter(|p| p.method != "HEAD" && p.method != "GET")
        .collect();
    assert_eq!(wrote, [], "the second copy wrote");
}

fn a_race_lost_on_a_channels_tag_is_repaired_from_the_ledger(kind: Kind) {
    let server = Server::start(kind);
    let repository = server.repository();
    let direct = Registry::new(&repository).unwrap();
    let [first, second] = ["1.3.0", "1.4.0"].map(|version| {
        let digest = put(&direct, version);
        direct
            .put_artifact(&Manifest::Index(
                IndexMeta::new(
                    ComponentName::new("web"),
                    Version::new(version),
                    vec![Artifact {
                        digest,
                        meta: meta("site", version),
                    }],
                )
                .unwrap(),
            ))
            .unwrap()
            .digest
    });
    let stable = ChannelName::new("stable");
    let keeper = FakeKeeper::new();
    let promotion = Envelope::signed(
        IN_TOTO,
        Claim {
            subject: first.clone(),
            kind: ClaimKind::Promotion,
            revision: Revision::new("4f2a9c1"),
            build: None,
            body: json!({ "channel": "stable" }),
        }
        .payload(),
        Signature {
            signer: SignerId::new("gate"),
            keyid: String::new(),
            sig: vec![9],
        },
    );
    keeper.append(&stable, &promotion, None).unwrap();

    let mut store = Registry::new(&repository).unwrap();
    let writer = Registry::new(&repository).unwrap();
    let (moved, raced) = (second.clone(), std::cell::Cell::new(false));
    store.between_put_and_read_back(move |tag| {
        if !raced.replace(true) {
            let at = writer.resolve(&Name::new(tag)).ok();
            writer
                .name(&Name::new(tag), &moved, at.as_ref())
                .expect("the other writer moves the tag");
        }
    });
    let name = Name::new("stable");
    assert_eq!(
        follow(&store, &keeper, &stable, &name, None),
        Ok(Followed::LostRace {
            found: Some(second),
            to: first.clone()
        })
    );
    assert_eq!(direct.resolve(&name), Ok(first));
}
