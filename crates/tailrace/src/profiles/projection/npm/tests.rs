//! The npm projection's operations over tarballs a test makes and a fake
//! store, needing no npm (design D12).

use std::fs;

use base64::Engine as _;
use serde_json::json;
use sha1::Digest as _;
use tailrace_fakes::{FakeStore, Fault, StoreOp, Times};
use tailrace_seams::{
    Contents, Digest, EcosystemName, Held, Holds, Manifest, Not, Projection, Range, Requirement,
    Store, Version, Why,
};
use tempfile::TempDir;

use super::fixture::{artifact_of, held, tarball};
use super::NpmProjection;

const ORIGIN: &str = "http://127.0.0.1:7420/";

/// A checkout whose `.npmrc` sends `@family` to the host's serving
/// process, and the npm projection bound in it over a fake store.
struct Rig {
    dir: TempDir,
    store: FakeStore,
}

impl Rig {
    fn new() -> Rig {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join(".npmrc"),
            "@family:registry=http://127.0.0.1:7420/agentplot/consumer/npm/\n",
        )
        .unwrap();
        Rig {
            dir,
            store: FakeStore::new(),
        }
    }

    fn projection(&self) -> NpmProjection {
        NpmProjection::new(self.dir.path(), Box::new(self.store.clone()), ORIGIN)
    }
}

fn requirement(name: &str, range: &str) -> Requirement {
    Requirement {
        name: EcosystemName::new(name),
        range: Range::new(range),
    }
}

fn versions(spelled: &[&str]) -> Vec<Version> {
    spelled.iter().map(|v| Version::new(*v)).collect()
}

#[test]
fn a_held_package_requires_its_family_dependencies_and_installed_peers() {
    let rig = Rig::new();
    let widget = held(
        &rig.store,
        &json!({
            "name": "@family/widget",
            "version": "1.4.0",
            "dependencies": { "@family/base": "^2.1.0", "left-pad": "^1.3.0" },
            "peerDependencies": { "@family/ui": "^3.0.0", "@family/theme": "^1.0.0" },
            "peerDependenciesMeta": { "@family/theme": { "optional": true } },
            "devDependencies": { "@family/test-kit": "^1.0.0" },
        }),
    );
    assert_eq!(
        rig.projection().requires(&widget).unwrap(),
        [
            requirement("@family/base", "^2.1.0"),
            requirement("@family/ui", "^3.0.0")
        ]
    );
}

#[test]
fn a_bundled_name_and_a_scope_sent_elsewhere_are_passed_over() {
    let rig = Rig::new();
    let widget = held(
        &rig.store,
        &json!({
            "name": "@family/widget",
            "version": "1.4.0",
            "dependencies": {
                "@family/inside": "^1.0.0",
                "@family/base": "^2.1.0",
                "@elsewhere/thing": "^1.0.0",
            },
            "bundleDependencies": ["@family/inside"],
        }),
    );
    assert_eq!(
        rig.projection().requires(&widget).unwrap(),
        [requirement("@family/base", "^2.1.0")]
    );
    fs::remove_file(rig.dir.path().join(".npmrc")).unwrap();
    assert_eq!(
        rig.projection().requires(&widget).unwrap(),
        [],
        "a checkout with no .npmrc has a family"
    );
}

#[test]
fn versions_within_a_range_are_npms_and_newest_first() {
    let rig = Rig::new();
    let projection = rig.projection();
    let given = versions(&["2.3.0", "3.0.0", "2.4.0-rc.1", "2.9.0", "2.10.0", "2.0.9"]);
    assert_eq!(
        projection
            .satisfying(&Range::new("^2.1.0"), &given)
            .unwrap(),
        versions(&["2.10.0", "2.9.0", "2.3.0"])
    );
    assert_eq!(
        projection
            .satisfying(&Range::new(">=2.4.0-rc.0 <3"), &given)
            .unwrap(),
        versions(&["2.10.0", "2.9.0", "2.4.0-rc.1"])
    );
    assert_eq!(
        projection.satisfying(&Range::new("*"), &given).unwrap()[0],
        Version::new("3.0.0")
    );
    assert_eq!(
        projection
            .satisfying(&Range::new("file:../local"), &given)
            .unwrap(),
        []
    );
    assert_eq!(
        projection
            .satisfying(&Range::new("^2"), &versions(&["latest", "2.1"]))
            .unwrap(),
        []
    );
}

#[test]
fn a_tarball_the_store_does_not_hold_is_unreadable_naming_the_artifact() {
    let rig = Rig::new();
    let other = FakeStore::new();
    let widget = held(
        &other,
        &json!({ "name": "@family/widget", "version": "1.4.0" }),
    );
    match rig.projection().requires(&widget) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, NpmProjection::source());
            assert!(u.what.contains("@family/widget 1.4.0"), "{u:?}");
            assert!(u.what.contains(&widget.digest.to_string()), "{u:?}");
        }
        other => panic!("a tarball the store does not hold answered {other:?}"),
    }
}

#[test]
fn a_tarball_with_no_package_json_is_unreadable_naming_the_artifact() {
    let rig = Rig::new();
    let empty = {
        let mut archive = tar::Builder::new(flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::default(),
        ));
        let mut header = tar::Header::new_gnu();
        header.set_size(3);
        header.set_mode(0o644);
        header.set_cksum();
        archive
            .append_data(&mut header, "package/README", &b"hi\n"[..])
            .unwrap();
        archive.into_inner().unwrap().finish().unwrap()
    };
    let widget = artifact_of(&rig.store, "@family/widget", "1.4.0", &empty);
    match rig.projection().requires(&widget) {
        Err(Not::Unreadable(u)) => {
            assert!(u.what.contains("@family/widget 1.4.0"), "{u:?}");
            assert!(
                u.said
                    .as_deref()
                    .is_some_and(|s| s.contains("package.json")),
                "{u:?}"
            );
        }
        other => panic!("a tarball with no package.json answered {other:?}"),
    }
}

#[test]
fn a_store_that_cannot_be_read_is_its_own_answer() {
    let rig = Rig::new();
    let widget = artifact_of(
        &rig.store,
        "@family/widget",
        "1.4.0",
        &tarball(&json!({ "name": "@family/widget", "version": "1.4.0" })),
    );
    rig.store.fault(
        StoreOp::Read,
        Fault::Unreadable("connection refused".into()),
        Times::UntilHealed,
    );
    match rig.projection().requires(&widget) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.said.as_deref(), Some("connection refused"));
        }
        other => panic!("an unreachable store answered {other:?}"),
    }
}

#[test]
fn a_tarball_whose_bytes_are_not_its_layers_is_refused_naming_both() {
    let rig = Rig::new();
    let bytes = tarball(&json!({ "name": "@family/widget", "version": "1.4.0" }));
    let widget = artifact_of(&rig.store, "@family/widget", "1.4.0", &bytes);
    let layer = tailrace_seams::Digest::of(&bytes);
    let other = b"other bytes".to_vec();
    rig.store.corrupt(&layer, &other);
    match rig.projection().requires(&widget) {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::OtherBytes {
                expected: layer,
                answered: tailrace_seams::Digest::of(&other),
            }
        ),
        other => panic!("a tarball of other bytes answered {other:?}"),
    }
}

/// `@family/widget` at `version`, packed and put in the rig's store.
fn widget(rig: &Rig, version: &str) -> tailrace_seams::Artifact {
    held(
        &rig.store,
        &json!({
            "name": "@family/widget",
            "version": version,
            "dependencies": { "@family/base": "^2.0.0" },
        }),
    )
}

/// Every file under `at`, by its path within it, and its bytes.
fn files(at: &std::path::Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    let mut all = std::collections::BTreeMap::new();
    let mut dirs = vec![at.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
            } else {
                let within = path.strip_prefix(at).unwrap().display().to_string();
                all.insert(within, fs::read(&path).unwrap());
            }
        }
    }
    all
}

fn packument(at: &std::path::Path, name: &str) -> serde_json::Value {
    serde_json::from_slice(&fs::read(at.join(name).join("index.json")).unwrap()).unwrap()
}

#[test]
fn two_versions_of_one_package_are_its_tarballs_and_one_packument() {
    let rig = Rig::new();
    let at = rig.dir.path().join(".tailrace/layouts/npm");
    let given = [widget(&rig, "1.2.0"), widget(&rig, "1.4.0")];
    let rendered = rig
        .projection()
        .render(Contents::Artifacts(&given), at.to_str().unwrap())
        .unwrap();
    assert_eq!(rendered.layout.at, at.display().to_string());
    assert_eq!(
        rendered.holds,
        Holds::Versions(given.iter().map(|a| a.meta.clone()).collect())
    );
    let packument = packument(&at, "@family/widget");
    assert_eq!(packument["name"], "@family/widget");
    assert_eq!(packument["dist-tags"]["latest"], "1.4.0");
    let listed: Vec<&String> = packument["versions"].as_object().unwrap().keys().collect();
    assert_eq!(listed, ["1.2.0", "1.4.0"]);
    for (artifact, version) in given.iter().zip(["1.2.0", "1.4.0"]) {
        let path = format!("@family/widget/-/widget-{version}.tgz");
        let bytes = fs::read(at.join(&path)).unwrap();
        let held = match rig.store.read(&artifact.digest).unwrap() {
            Held::Manifest(Manifest::Artifact { layers, .. }) => layers[0].clone(),
            other => panic!("the artifact's manifest is {other:?}"),
        };
        assert_eq!(
            Digest::of(&bytes),
            held,
            "the tarball is not the store's bytes"
        );
        let entry = &packument["versions"][version];
        assert_eq!(entry["name"], "@family/widget");
        assert_eq!(entry["version"], version);
        assert_eq!(entry["dependencies"]["@family/base"], "^2.0.0");
        assert_eq!(entry["dist"]["tarball"], path);
        let integrity = format!(
            "sha512-{}",
            base64::engine::general_purpose::STANDARD.encode(sha2::Sha512::digest(&bytes))
        );
        assert_eq!(entry["dist"]["integrity"], integrity);
        let shasum: String = sha1::Sha1::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(entry["dist"]["shasum"], shasum);
    }
}

#[test]
fn a_version_not_given_again_leaves_the_layout_and_its_packument() {
    let rig = Rig::new();
    let at = rig.dir.path().join("layout");
    let (old, new) = (widget(&rig, "1.2.0"), widget(&rig, "1.4.0"));
    let base = held(
        &rig.store,
        &json!({ "name": "@family/base", "version": "2.2.0" }),
    );
    let at_str = at.to_str().unwrap();
    let projection = rig.projection();
    projection
        .render(Contents::Artifacts(&[old, new.clone(), base]), at_str)
        .unwrap();
    assert!(at.join("@family/base/index.json").is_file());
    projection
        .render(Contents::Artifacts(&[new]), at_str)
        .unwrap();
    let listed: Vec<String> = packument(&at, "@family/widget")["versions"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    assert_eq!(listed, ["1.4.0"]);
    assert!(!at.join("@family/widget/-/widget-1.2.0.tgz").exists());
    assert!(at.join("@family/widget/-/widget-1.4.0.tgz").is_file());
    assert!(
        !at.join("@family/base").exists(),
        "a name not given is held"
    );
}

#[test]
fn a_refused_version_is_in_no_packument() {
    let rig = Rig::new();
    let at = rig.dir.path().join("layout");
    widget(&rig, "1.2.0");
    let admitted = widget(&rig, "1.4.0");
    rig.projection()
        .render(Contents::Artifacts(&[admitted]), at.to_str().unwrap())
        .unwrap();
    let packument = packument(&at, "@family/widget");
    assert_eq!(packument["versions"].as_object().unwrap().len(), 1);
    assert_eq!(packument["versions"]["1.2.0"], json!(null));
    assert!(!at.join("@family/widget/-/widget-1.2.0.tgz").exists());
}

#[test]
fn the_same_artifacts_rendered_again_give_the_same_layout_naming_no_place() {
    let rig = Rig::new();
    let at = rig.dir.path().join("layout");
    let given = [widget(&rig, "1.2.0"), widget(&rig, "1.4.0")];
    let projection = rig.projection();
    let first = projection
        .render(Contents::Artifacts(&given), at.to_str().unwrap())
        .unwrap();
    let written = files(&at);
    let again = projection
        .render(Contents::Artifacts(&given), at.to_str().unwrap())
        .unwrap();
    assert_eq!(first, again);
    assert_eq!(files(&at), written, "the layout changed");
    let place = rig.dir.path().display().to_string();
    for (path, bytes) in written.iter().filter(|(p, _)| p.ends_with(".json")) {
        let text = String::from_utf8(bytes.clone()).unwrap();
        for named in [place.as_str(), "127.0.0.1", "http", "sha256:"] {
            assert!(!text.contains(named), "{path} names {named}: {text}");
        }
    }
}

#[test]
fn a_tarball_the_store_cannot_answer_leaves_the_layout_as_it_was() {
    let rig = Rig::new();
    let at = rig.dir.path().join("layout");
    let held_one = widget(&rig, "1.2.0");
    let projection = rig.projection();
    projection
        .render(
            Contents::Artifacts(std::slice::from_ref(&held_one)),
            at.to_str().unwrap(),
        )
        .unwrap();
    let before = files(&at);

    let elsewhere = FakeStore::new();
    let missing = held(
        &elsewhere,
        &json!({ "name": "@family/widget", "version": "1.4.0" }),
    );
    match projection.render(
        Contents::Artifacts(&[held_one.clone(), missing.clone()]),
        at.to_str().unwrap(),
    ) {
        Err(Not::Unreadable(u)) => {
            assert!(u.what.contains(&missing.digest.to_string()), "{u:?}")
        }
        other => panic!("a tarball the store does not hold answered {other:?}"),
    }
    assert_eq!(files(&at), before, "the layout changed");

    rig.store.fault(
        StoreOp::Read,
        Fault::Unreadable("connection refused".into()),
        Times::Once,
    );
    match projection.render(Contents::Artifacts(&[held_one]), at.to_str().unwrap()) {
        Err(Not::Unreadable(u)) => assert_eq!(u.said.as_deref(), Some("connection refused")),
        other => panic!("an unreachable store answered {other:?}"),
    }
    assert_eq!(files(&at), before, "the layout changed");
}

#[test]
fn a_tarball_of_other_bytes_is_refused_and_written_nowhere() {
    let rig = Rig::new();
    let at = rig.dir.path().join("layout");
    let bytes = tarball(&json!({ "name": "@family/widget", "version": "1.4.0" }));
    let widget = artifact_of(&rig.store, "@family/widget", "1.4.0", &bytes);
    let layer = Digest::of(&bytes);
    rig.store.corrupt(&layer, b"other bytes");
    match rig
        .projection()
        .render(Contents::Artifacts(&[widget]), at.to_str().unwrap())
    {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::OtherBytes {
                expected: layer,
                answered: Digest::of(b"other bytes"),
            }
        ),
        other => panic!("a tarball of other bytes answered {other:?}"),
    }
    assert!(
        !at.join("@family").exists(),
        "a refused tarball was written"
    );
}

#[test]
fn an_unscoped_package_and_a_prerelease_alone_render_as_npm_reads_them() {
    let rig = Rig::new();
    let at = rig.dir.path().join("layout");
    let given = [
        held(
            &rig.store,
            &json!({ "name": "pad", "version": "2.0.0-rc.1" }),
        ),
        held(
            &rig.store,
            &json!({ "name": "pad", "version": "2.0.0-rc.2" }),
        ),
    ];
    rig.projection()
        .render(Contents::Artifacts(&given), at.to_str().unwrap())
        .unwrap();
    assert!(at.join("pad/-/pad-2.0.0-rc.2.tgz").is_file());
    assert_eq!(packument(&at, "pad")["dist-tags"]["latest"], "2.0.0-rc.2");
}

#[test]
fn cache_entries_are_absent_and_another_ecosystem_is_refused() {
    let rig = Rig::new();
    let at = rig.dir.path().join("layout");
    let entry = tailrace_seams::CacheEntry {
        build: tailrace_seams::BuildId::new("b7e1"),
        digest: Digest::of(b"archive"),
    };
    match rig
        .projection()
        .render(Contents::CacheEntries(&[entry]), at.to_str().unwrap())
    {
        Err(Not::Absent(a)) => assert_eq!(a.what, "cache entries"),
        other => panic!("cache entries answered {other:?}"),
    }
    let file = tailrace_seams::Artifact {
        digest: Digest::of(b"notes"),
        meta: tailrace_seams::ArtifactMeta::new(
            tailrace_seams::Ecosystem::new("file"),
            EcosystemName::new("notes"),
            Version::new("1.0.0"),
        )
        .unwrap(),
    };
    assert!(matches!(
        rig.projection()
            .render(Contents::Artifacts(&[file]), at.to_str().unwrap()),
        Err(Not::Refused(_))
    ));
    assert!(!at.exists(), "a refused render made the layout");
}
