//! The copy group: a version published in the world copied to a second
//! store with every attestation on its digests, and copied again
//! writing nothing (1, 8, 103; design D11).

use serde_json::json;
use tailrace_core::{copy, End, Published};
use tailrace_fakes::FakeStore;
use tailrace_seams::{
    canonical, Claim, ClaimKind, Digest, Envelope, Held, Manifest, Name, Seam, Signer, Source,
    Store,
};

use crate::core::publish::{one, passing};
use crate::world::FakeWorld;
use crate::{ok, Scenario};

pub const GROUP: &str = "copy";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        Scenario {
            name: "a version copied with its claims",
            plays: None,
            cites: &[1, 8, 103],
            run: a_version_copied_with_its_claims,
        },
        Scenario {
            name: "a version copied again",
            plays: None,
            cites: &[1, 103],
            run: a_version_copied_again,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}

/// The second store a scenario copies into, as its answers name it.
fn target() -> Source {
    Source::new(Seam::Store, "a second fake")
}

/// The head published in the world, an acceptance and a promotion
/// attached to its index as a gate would attach them (39); the version's
/// name, its index and its artifacts.
fn published(world: &FakeWorld) -> (Name, Digest, Vec<Digest>) {
    passing(world, &world.head);
    let published = ok(one(world.publish(&world.ask(&world.head))), "the publish");
    let Published::Version {
        tag,
        index,
        artifacts,
        ..
    } = published
    else {
        panic!("the head was not published: {published:?}")
    };
    for kind in [ClaimKind::Acceptance, ClaimKind::Promotion] {
        let claim = Claim {
            subject: index.clone(),
            kind,
            revision: world.head.clone(),
            build: None,
            body: json!({ "channel": "stable" }),
        };
        let envelope = ok(world.fakes.signer.sign(&claim, &world.host), "a sign");
        ok(world.fakes.store.attach(&index, &envelope), "an attach");
    }
    let artifacts = artifacts.into_iter().map(|a| a.digest).collect();
    (Name::new(tag), index, artifacts)
}

fn copied(world: &FakeWorld, to: &FakeStore, name: &Name) -> tailrace_core::Copied {
    let (from, target) = (world.store(), target());
    ok(
        copy(
            End {
                store: &world.fakes.store,
                source: &from,
            },
            End {
                store: to,
                source: &target,
            },
            name,
        ),
        "the copy",
    )
}

fn claims(store: &dyn Store, digest: &Digest, at: &str) -> Vec<Envelope> {
    ok(
        store.claims_of(digest),
        &format!("claims-of {digest} in {at}"),
    )
}

fn a_version_copied_with_its_claims(world: &FakeWorld) {
    let (name, index, artifacts) = published(world);
    let to = FakeStore::new();
    let answer = copied(world, &to, &name);
    assert_eq!(answer.index, index, "the copy answered another index");
    assert_eq!(
        ok(to.resolve(&name), "resolve in the target"),
        index,
        "the target's name names another digest than the source's index"
    );
    let from: &dyn Store = &world.fakes.store;
    for digest in std::iter::once(&index).chain(&artifacts) {
        assert_eq!(
            ok(to.read(digest), "read in the target"),
            ok(from.read(digest), "read in the source"),
            "{digest} holds otherwise in the target than in the source"
        );
        let (held, carried) = (
            claims(from, digest, "the source"),
            claims(&to, digest, "the target"),
        );
        assert!(!held.is_empty(), "the source holds no claim on {digest}");
        assert_eq!(
            carried.iter().map(canonical).collect::<Vec<_>>(),
            held.iter().map(canonical).collect::<Vec<_>>(),
            "the claims on {digest} were not carried byte for byte"
        );
    }
    let kinds: Vec<ClaimKind> = claims(&to, &index, "the target")
        .iter()
        .filter_map(Claim::from_envelope)
        .map(|c| c.kind)
        .collect();
    assert_eq!(
        kinds,
        [ClaimKind::Acceptance, ClaimKind::Promotion],
        "the index's acceptance and promotion were not carried"
    );
    let Held::Manifest(Manifest::Index(_)) = ok(to.read(&index), "read of the index") else {
        panic!("the target holds no index at {index}")
    };
    let names: Vec<Name> = to.contents().names.into_keys().collect();
    assert_eq!(
        names,
        [name],
        "the copy named something in the target beside the version"
    );
}

fn a_version_copied_again(world: &FakeWorld) {
    let (name, index, _) = published(world);
    let to = FakeStore::new();
    let first = copied(world, &to, &name);
    let held = to.contents();
    let again = copied(world, &to, &name);
    assert_eq!(to.contents(), held, "the second copy wrote to the target");
    assert!(again.name_held_already, "the second copy moved the name");
    assert_eq!(again.index, index);
    assert_eq!(
        again
            .written
            .iter()
            .map(|w| (&w.what, &w.digest))
            .collect::<Vec<_>>(),
        first
            .written
            .iter()
            .map(|w| (&w.what, &w.digest))
            .collect::<Vec<_>>(),
        "the second copy answered other digests than the first"
    );
    for written in &again.written {
        assert!(
            written.held_already,
            "the second copy wrote {written:?} again"
        );
    }
}
