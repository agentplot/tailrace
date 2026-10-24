//! The record's group: the host's record deleted and rebuilt from the
//! store and git to the same state, S12 among them, a rebuild writing
//! nothing but the record and naming what it could not read (58, 63).

use std::fs;
use std::path::PathBuf;

use tailrace_core::{rebuild, status, RebuildAsk, Rebuilt, Status, StatusAsk};
use tailrace_fakes::{Fault, StoreOp, Times};
use tailrace_seams::{Answer, Name, Not, Outcome, Revision};

use crate::world::FakeWorld;
use crate::{ok, Scenario};

pub const GROUP: &str = "record";

/// The group runs over [`FakeWorld::two_components`].
pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
        Scenario {
            name: "everything kept deleted and rebuilt",
            plays: Some("S12"),
            cites: &[63],
            run: everything_kept_deleted_and_rebuilt,
        },
        Scenario {
            name: "a claimed version with no index",
            plays: None,
            cites: &[63],
            run: a_claimed_version_with_no_index,
        },
        Scenario {
            name: "a rebuild writes only the record",
            plays: None,
            cites: &[63],
            run: a_rebuild_writes_only_the_record,
        },
        Scenario {
            name: "a rebuild that cannot read the store",
            plays: None,
            cites: &[58, 63],
            run: a_rebuild_that_cannot_read_the_store,
        },
    ]
}

/// `revision` landed with no verdicts, both components' outputs built
/// and passing there.
fn landed(world: &FakeWorld, revision: &Revision) {
    world.land(revision, &[]);
    world.outputs(
        revision,
        &[("library", Outcome::Pass), ("server", Outcome::Pass)],
    );
}

/// Both components published at the start, `lib-v0.1.0` and
/// `app-v0.1.0`, and `lib` again at the head, `lib-v0.2.0`.
fn published(world: &FakeWorld) {
    for revision in [&world.start, &world.head] {
        landed(world, revision);
        let answer = ok(world.publish(&world.ask(revision)), "the publish");
        for (component, published) in answer.components {
            ok(published, &format!("the publish of {component}"));
        }
    }
}

fn rebuilt(world: &FakeWorld) -> Answer<Rebuilt> {
    let history = world.history();
    let ask = RebuildAsk {
        branch: world.main.clone(),
        revision: world.head.clone(),
        store: world.store(),
    };
    rebuild(&world.seams(), &history, &world.kept(), &ask)
}

/// The record's files by path and bytes.
fn record(world: &FakeWorld) -> Vec<(PathBuf, Vec<u8>)> {
    world
        .record()
        .into_iter()
        .map(|(path, bytes, _)| (path, bytes))
        .collect()
}

fn status_of(world: &FakeWorld) -> Status {
    ok(status(&world.kept(), &StatusAsk::default()), "status")
}

/// The record's file of `tag`, of `component`.
fn file(world: &FakeWorld, component: &str, tag: &str) -> PathBuf {
    world
        .kept()
        .dir()
        .join(component)
        .join(format!("{tag}.json"))
}

fn everything_kept_deleted_and_rebuilt(world: &FakeWorld) {
    published(world);
    let before = record(world);
    let tags: Vec<String> = before
        .iter()
        .map(|(p, _)| p.file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        tags,
        ["app-v0.1.0", "lib-v0.1.0", "lib-v0.2.0"],
        "the publishes kept other versions"
    );
    let answered = status_of(world);
    world.delete_record();
    assert_eq!(world.record(), [], "the record was not deleted");

    let answer = ok(rebuilt(world), "the rebuild");
    for (entry, written) in &answer.entries {
        assert!(written.is_ok(), "{} answered {written:?}", entry.tag);
    }
    assert_eq!(
        record(world),
        before,
        "the rebuilt record holds other files or other bytes"
    );
    assert_eq!(
        status_of(world),
        answered,
        "status answers otherwise after the rebuild"
    );
}

fn a_claimed_version_with_no_index(world: &FakeWorld) {
    published(world);
    world.delete_record();
    world.fakes.store.forget(&Name::new("app-v0.1.0"));
    ok(rebuilt(world), "the rebuild");
    assert!(
        !file(world, "app", "app-v0.1.0").exists(),
        "a version whose name the store does not hold was rebuilt"
    );
    for tag in ["lib-v0.1.0", "lib-v0.2.0"] {
        assert!(file(world, "lib", tag).exists(), "{tag} was not rebuilt");
    }
}

fn a_rebuild_writes_only_the_record(world: &FakeWorld) {
    published(world);
    world.delete_record();
    let (writes, store) = (world.writes(), world.store_calls().len());
    let (signer, build, policy, keeper, targets) = (
        world.fakes.signer.calls().len(),
        world.build_calls().len(),
        world.fakes.policy.calls().len(),
        world.keeper_calls().len(),
        world.target_calls().len(),
    );
    let projections: Vec<usize> = world
        .fakes
        .projections
        .values()
        .map(|p| p.calls().len())
        .collect();

    ok(rebuilt(world), "the rebuild");
    assert_eq!(world.writes(), writes, "the rebuild wrote to the store");
    let asked: Vec<StoreOp> = world.store_calls().split_off(store);
    assert!(
        !asked.is_empty()
            && asked
                .iter()
                .all(|op| matches!(op, StoreOp::Resolve | StoreOp::Read)),
        "the store was asked {asked:?}, not only to resolve and read"
    );
    assert_eq!(
        world.fakes.signer.calls().len(),
        signer,
        "a signer was asked"
    );
    assert_eq!(world.build_calls().len(), build, "the build was asked");
    assert_eq!(
        world.fakes.policy.calls().len(),
        policy,
        "the policy was asked"
    );
    assert_eq!(world.keeper_calls().len(), keeper, "the keeper was asked");
    assert_eq!(world.target_calls().len(), targets, "a target was asked");
    let after: Vec<usize> = world
        .fakes
        .projections
        .values()
        .map(|p| p.calls().len())
        .collect();
    assert_eq!(after, projections, "a projection was asked");
    assert_eq!(world.record().len(), 3, "the record was not rebuilt");
}

fn a_rebuild_that_cannot_read_the_store(world: &FakeWorld) {
    published(world);
    world.delete_record();
    world.fakes.store.fault(
        StoreOp::Resolve,
        Fault::Unreadable("connection refused".into()),
        Times::UntilHealed,
    );
    match rebuilt(world) {
        Err(Not::Unreadable(u)) => assert_eq!(
            u.said.as_deref(),
            Some("connection refused"),
            "the store's words were not carried verbatim"
        ),
        other => panic!("a rebuild over a store it cannot read answered {other:?}"),
    }
    assert_eq!(world.record(), [], "a file of the record was written");
    assert!(
        fs::read_dir(world.kept().dir()).is_err(),
        "the record's directory was made"
    );
}
