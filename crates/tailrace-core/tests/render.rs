//! Consumer layouts rendered under a policy, over the fakes (design D2
//! to D7).

use tailrace_core::{artifact_name, locate, Located, ARTIFACT};
use tailrace_fakes::{Fakes, Fault, StoreOp, Times};
use tailrace_seams::{
    Artifact, ArtifactMeta, ComponentName, Digest, Ecosystem, EcosystemName, IndexMeta, Manifest,
    Name, Not, Pinned, Seam, Store, Version,
};

const ECO: &str = "eco";

fn eco() -> Ecosystem {
    Ecosystem::new(ECO)
}

fn pin(name: &str, version: &str) -> Pinned {
    Pinned {
        name: EcosystemName::new(name),
        version: Version::new(version),
    }
}

fn meta(ecosystem: &str, name: &str, version: &str) -> ArtifactMeta {
    ArtifactMeta::new(
        Ecosystem::new(ecosystem),
        EcosystemName::new(name),
        Version::new(version),
    )
    .unwrap()
}

/// An artifact of `meta` put in `store`, its bytes and its manifest.
fn artifact(store: &dyn Store, meta: ArtifactMeta) -> Artifact {
    let bytes = format!("{} {} {}", meta.ecosystem(), meta.name(), meta.version());
    let layer = store.put_bytes(bytes.as_bytes()).unwrap().digest;
    let digest = store
        .put_artifact(&Manifest::Artifact {
            meta: meta.clone(),
            layers: vec![layer],
        })
        .unwrap()
        .digest;
    Artifact { digest, meta }
}

/// The index of `component` at `version` naming `artifacts`, put in
/// `store`.
fn index(store: &dyn Store, component: &str, version: &str, artifacts: Vec<Artifact>) -> Digest {
    let meta = IndexMeta::new(
        ComponentName::new(component),
        Version::new(version),
        artifacts,
    )
    .unwrap();
    store.put_artifact(&Manifest::Index(meta)).unwrap().digest
}

/// `version` of `name` in the ecosystem published to `store`, with a
/// second artifact of another ecosystem beside it in its index, and the
/// name a pin resolves through moved to the index.
fn published(store: &dyn Store, name: &str, version: &str) -> Located {
    let ours = artifact(store, meta(ECO, name, version));
    let other = artifact(store, meta("other", name, version));
    let index = index(store, name, version, vec![other, ours.clone()]);
    store
        .name(
            &artifact_name(&eco(), &EcosystemName::new(name), &Version::new(version)),
            &index,
            None,
        )
        .unwrap();
    Located {
        index,
        artifact: ours,
    }
}

#[test]
fn a_pins_name_is_its_ecosystem_name_and_version_under_artifact() {
    assert_eq!(ARTIFACT, "artifact/");
    assert_eq!(
        artifact_name(
            &eco(),
            &EcosystemName::new("@scope/pkg"),
            &Version::new("1.2.0-rc.1")
        ),
        Name::new("artifact/eco/@scope/pkg@1.2.0-rc.1")
    );
}

#[test]
fn a_pin_resolves_through_its_index_to_the_artifact_of_its_ecosystem_and_name() {
    let fakes = Fakes::new();
    let found = published(&fakes.store, "pkg", "1.1.0");
    assert_eq!(
        locate(&fakes.store, &eco(), &pin("pkg", "1.1.0")),
        Ok(Ok(found.clone()))
    );
    assert_eq!(found.artifact.meta, meta(ECO, "pkg", "1.1.0"));
    let ops: Vec<StoreOp> = fakes.store.calls().into_iter().map(|c| c.op).collect();
    assert_eq!(
        ops[ops.len() - 2..],
        [StoreOp::Resolve, StoreOp::Read],
        "a pin was found by other means"
    );
}

#[test]
fn a_name_the_store_does_not_hold_is_absent_naming_it() {
    let fakes = Fakes::new();
    published(&fakes.store, "pkg", "1.1.0");
    match locate(&fakes.store, &eco(), &pin("left-pad", "1.3.0")) {
        Ok(Err(Not::Absent(a))) => {
            assert_eq!(a.source.seam, Seam::Store);
            assert_eq!(a.what, "name artifact/eco/left-pad@1.3.0");
        }
        other => panic!("a third-party pin answered {other:?}"),
    }
}

#[test]
fn a_name_at_no_such_index_is_unreadable_naming_the_name_and_the_digest() {
    let fakes = Fakes::new();
    let store = &fakes.store;
    let unreadable = |pin: &Pinned, at: &Digest, says: &str| match locate(store, &eco(), pin) {
        Ok(Err(Not::Unreadable(u))) => {
            let name = artifact_name(&eco(), &pin.name, &pin.version);
            assert_eq!(u.what, format!("{name} at {at}"));
            let said = u.said.unwrap();
            assert!(said.contains(says), "{said}");
        }
        other => panic!("{pin:?} answered {other:?}"),
    };

    let bytes = store.put_bytes(b"no index").unwrap().digest;
    let at_bytes = pin("bytes", "1.0.0");
    let name = artifact_name(&eco(), &at_bytes.name, &at_bytes.version);
    store.name(&name, &bytes, None).unwrap();
    unreadable(&at_bytes, &bytes, "holds no index");

    let elsewhere = index(
        store,
        "lib",
        "2.0.0",
        vec![artifact(store, meta("other", "lib", "2.0.0"))],
    );
    let other_ecosystem = pin("lib", "2.0.0");
    let name = artifact_name(&eco(), &other_ecosystem.name, &other_ecosystem.version);
    store.name(&name, &elsewhere, None).unwrap();
    unreadable(
        &other_ecosystem,
        &elsewhere,
        "names 0 artifacts of eco lib 2.0.0",
    );

    let gone = Digest::of(b"never held");
    let at_nothing = pin("gone", "1.0.0");
    let name = artifact_name(&eco(), &at_nothing.name, &at_nothing.version);
    store.name(&name, &gone, None).unwrap();
    unreadable(&at_nothing, &gone, "is not held");
}

#[test]
fn a_store_that_cannot_be_read_is_the_outer_answer() {
    let fakes = Fakes::new();
    published(&fakes.store, "pkg", "1.1.0");
    fakes.store.fault(
        StoreOp::Resolve,
        Fault::Unreadable("connection refused".into()),
        Times::Once,
    );
    match locate(&fakes.store, &eco(), &pin("pkg", "1.1.0")) {
        Err(Not::Unreadable(u)) => assert_eq!(u.said.as_deref(), Some("connection refused")),
        other => panic!("an unreachable store answered {other:?}"),
    }
    fakes
        .store
        .fault(StoreOp::Read, Fault::Refused("denied".into()), Times::Once);
    assert!(matches!(
        locate(&fakes.store, &eco(), &pin("pkg", "1.1.0")),
        Err(Not::Refused(r)) if r.said.as_deref() == Some("denied")
    ));
}
