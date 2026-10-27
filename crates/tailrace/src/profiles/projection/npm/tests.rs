//! The npm projection's operations over tarballs a test makes and a fake
//! store, needing no npm (design D12).

use std::fs;

use serde_json::json;
use tailrace_fakes::{FakeStore, Fault, StoreOp, Times};
use tailrace_seams::{EcosystemName, Not, Projection, Range, Requirement, Version, Why};
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
