//! Spec cache-push: the step after a landing that writes the cache
//! entries of the build identities its passing verdicts name, over the
//! fakes, offering nothing and keeping nothing (design D1, D2).

mod common;
mod world;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tailrace_core::cache::Written;
use tailrace_core::{answer_json, cache_name, push, Push, PushAsk, BOUND};
use tailrace_fakes::{Fault, LandingOp, ProjectionOp, StoreOp, Times};
use tailrace_seams::{Answer, BuildId, Held, Not, Outcome, Seam, Why};
use world::{eco, World, LANDING};

fn ask(world: &World) -> PushAsk {
    PushAsk {
        branch: world.main.clone(),
        revision: world.head.clone(),
    }
}

fn pushed(world: &World) -> Push {
    push(&world.fakes.seams(), &ask(world))
        .unwrap_or_else(|not| panic!("the push answered {not:?}"))
}

fn written(answer: &Answer<Written>) -> &Written {
    answer
        .as_ref()
        .unwrap_or_else(|not| panic!("the entry answered {not:?}"))
}

fn build(name: &str) -> BuildId {
    BuildId::new(name)
}

/// The world's head with a verdict of `test` recorded for each of
/// `verdicts`.
fn landed_with(verdicts: &[(&str, Outcome)]) -> World {
    let world = World::new();
    for (b, outcome) in verdicts {
        world.verdict("test", &build(b), *outcome, LANDING);
    }
    world
}

fn store_ops(world: &World) -> Vec<StoreOp> {
    world
        .fakes
        .store
        .calls()
        .into_iter()
        .map(|c| c.op)
        .collect()
}

/// The build identities each projection was asked to pack, in order.
fn packed(world: &World, ecosystem: &str) -> Vec<String> {
    world.fakes.projections[&eco(ecosystem)]
        .calls()
        .into_iter()
        .filter(|c| c.op == ProjectionOp::Pack)
        .map(|c| c.given["build"].as_str().unwrap().to_string())
        .collect()
}

fn files(dir: &Path) -> Vec<(PathBuf, Vec<u8>, SystemTime)> {
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return found;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            found.extend(files(&path));
        } else {
            let modified = fs::metadata(&path).unwrap().modified().unwrap();
            found.push((path.clone(), fs::read(&path).unwrap(), modified));
        }
    }
    found.sort();
    found
}

// Requirement: The push asks the landing first

#[test]
fn a_revision_not_landed_is_refused_and_the_store_and_projections_are_asked_nothing() {
    let world = landed_with(&[("b7e1", Outcome::Pass)]);
    let other = world.start.clone();
    let answer = push(
        &world.fakes.seams(),
        &PushAsk {
            branch: world.main.clone(),
            revision: other.clone(),
        },
    );
    match answer {
        Err(Not::Refused(r)) => {
            assert_eq!(
                r.why,
                Why::NotLanded {
                    branch: world.main.clone(),
                    revision: other,
                }
            );
            assert_eq!(r.source.profile, BOUND);
        }
        other => panic!("answered {other:?}"),
    }
    assert_eq!(store_ops(&world), [], "the store was asked something");
    for (ecosystem, projection) in &world.fakes.projections {
        assert_eq!(projection.calls(), [], "{ecosystem} was asked something");
    }
    let landing: Vec<LandingOp> = world
        .fakes
        .landing
        .calls()
        .into_iter()
        .map(|c| c.op)
        .collect();
    assert_eq!(landing, [LandingOp::Landed], "the verdicts were read");
}

// Requirement: The push writes the entries of a landing's passing builds

#[test]
fn each_passing_build_identity_is_written_and_a_failed_ones_never_packed() {
    let world = landed_with(&[
        ("b7e1", Outcome::Pass),
        ("c3d4", Outcome::Fail),
        ("d5f6", Outcome::Pass),
    ]);
    let answer = pushed(&world);

    let builds: Vec<&str> = answer.entries.iter().map(|(b, _)| b.as_str()).collect();
    assert_eq!(builds, ["b7e1", "d5f6"]);
    assert_eq!(answer.branch, world.main);
    assert_eq!(answer.revision, world.head);
    let contents = world.fakes.store.contents();
    for (build, entry) in &answer.entries {
        let entry = written(entry);
        assert_eq!(entry.held_at, None, "{build} was found held");
        assert_eq!(entry.entry.build, *build);
        assert_eq!(entry.projection, eco("eco-a"));
        assert_eq!(
            contents.names.get(&cache_name(build)),
            Some(&entry.entry.digest)
        );
        assert!(matches!(
            contents.held.get(&entry.entry.digest),
            Some(Held::Bytes(_))
        ));
    }
    assert_eq!(packed(&world, "eco-a"), ["b7e1", "d5f6"]);
    assert_eq!(packed(&world, "eco-b"), Vec::<String>::new());
    assert!(
        !contents.names.contains_key(&cache_name(&build("c3d4"))),
        "the failed build was written"
    );

    let asked = &world.fakes.projections[&eco("eco-a")].calls()[0].given;
    assert_eq!(asked["version"], serde_json::Value::Null);
    assert_eq!(asked["output"]["name"], "test");
    assert_eq!(asked["output"]["files"], serde_json::json!([]));
}

#[test]
fn two_passing_verdicts_naming_one_build_identity_write_it_once() {
    let world = landed_with(&[("b7e1", Outcome::Pass), ("b7e1", Outcome::Pass)]);
    let answer = pushed(&world);
    assert_eq!(answer.entries.len(), 1);
    assert_eq!(packed(&world, "eco-a"), ["b7e1"], "packed twice");
    assert_eq!(store_ops(&world), [StoreOp::PutBytes, StoreOp::Name]);
}

#[test]
fn a_build_identity_passing_once_and_failing_once_is_written() {
    let world = landed_with(&[("b7e1", Outcome::Fail), ("b7e1", Outcome::Pass)]);
    let answer = pushed(&world);
    assert_eq!(answer.entries.len(), 1);
    written(&answer.entries[0].1);
}

// Requirement: A push repeated writes nothing

#[test]
fn a_second_push_answers_each_held_at_the_firsts_digest_and_writes_nothing() {
    let world = landed_with(&[("b7e1", Outcome::Pass), ("d5f6", Outcome::Pass)]);
    let first = pushed(&world);
    let held = world.fakes.store.contents();
    let since = world.fakes.store.calls().len();

    let second = pushed(&world);
    for ((build, first), (again, second)) in first.entries.iter().zip(&second.entries) {
        assert_eq!(build, again);
        let (first, second) = (written(first), written(second));
        assert_eq!(second.held_at.as_ref(), Some(&first.entry.digest));
        assert!(second.bytes_held_already);
    }
    assert_eq!(world.fakes.store.contents(), held, "the second push wrote");
    let ops: Vec<StoreOp> = world
        .fakes
        .store
        .calls()
        .split_off(since)
        .into_iter()
        .map(|c| c.op)
        .collect();
    assert_eq!(
        ops,
        [
            StoreOp::PutBytes,
            StoreOp::Name,
            StoreOp::PutBytes,
            StoreOp::Name
        ],
        "a held name was retried"
    );
}

// Requirement: Each build identity is answered on its own

#[test]
fn an_archive_not_on_this_host_is_absent_and_the_other_is_written() {
    let world = landed_with(&[("b7e1", Outcome::Pass), ("d5f6", Outcome::Pass)]);
    for projection in world.fakes.projections.values() {
        projection.no_archive(&build("b7e1"));
    }
    let answer = pushed(&world);
    match &answer.entries[0] {
        (b, Err(Not::Absent(a))) => {
            assert_eq!(*b, build("b7e1"));
            assert_eq!(a.source.seam, Seam::Projection);
            assert!(a.what.contains("b7e1"), "{}", a.what);
        }
        other => panic!("the missing archive answered {other:?}"),
    }
    let kept = written(&answer.entries[1].1).clone();
    assert_eq!(kept.entry.build, build("d5f6"));
    let names: Vec<String> = world
        .fakes
        .store
        .contents()
        .names
        .into_keys()
        .map(|n| n.to_string())
        .collect();
    assert_eq!(names, ["cache/d5f6"]);

    let json = answer_json(&Ok::<_, Not>(answer));
    assert_eq!(json["answer"], "ok");
    let entries = &json["value"]["entries"];
    assert_eq!(entries[0]["answer"], "absent");
    assert_eq!(entries[0]["build"], "b7e1");
    assert_eq!(entries[1]["answer"], "ok");
    assert_eq!(entries[1]["build"], "d5f6");
    assert_eq!(
        entries[1]["value"]["entry"]["digest"],
        serde_json::json!(kept.entry.digest)
    );
    assert_eq!(json["value"]["branch"], "main");
}

#[test]
fn a_refused_write_is_that_build_identitys_answer_and_the_next_is_written() {
    let world = landed_with(&[("b7e1", Outcome::Pass), ("d5f6", Outcome::Pass)]);
    world.fakes.store.fault(
        StoreOp::PutBytes,
        Fault::Refused("quota exceeded".into()),
        Times::Once,
    );
    let answer = pushed(&world);
    match &answer.entries[0].1 {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source.seam, Seam::Store);
            assert_eq!(r.said.as_deref(), Some("quota exceeded"));
        }
        other => panic!("the refused write answered {other:?}"),
    }
    let next = written(&answer.entries[1].1);
    assert_eq!(next.entry.build, build("d5f6"));
    assert_eq!(
        world
            .fakes
            .store
            .contents()
            .names
            .get(&cache_name(&build("d5f6"))),
        Some(&next.entry.digest)
    );
}

#[test]
fn verdicts_that_cannot_be_read_are_the_whole_answer() {
    let world = landed_with(&[("b7e1", Outcome::Pass)]);
    world.fakes.landing.fault(
        LandingOp::VerdictsOf,
        Fault::Unreadable("the run log is gone".into()),
        Times::Once,
    );
    match push(&world.fakes.seams(), &ask(&world)) {
        Err(Not::Unreadable(u)) => assert_eq!(u.said.as_deref(), Some("the run log is gone")),
        other => panic!("answered {other:?}"),
    }
    assert_eq!(store_ops(&world), []);
}

// Requirement: The push offers nothing and keeps nothing

#[test]
fn the_push_asks_the_store_only_to_put_bytes_and_name_and_keeps_nothing() {
    let world = World::new();
    world.publish_one(&world.ask_for("lib")).unwrap();
    let record = files(world.kept().dir());
    assert!(!record.is_empty(), "the publish kept nothing");
    let (store, signer, policy, builds) = (
        world.fakes.store.calls().len(),
        world.fakes.signer.calls().len(),
        world.fakes.policy.calls().len(),
        world.fakes.build.calls().len(),
    );
    world.verdict("test", &build("b7e1"), Outcome::Pass, LANDING);
    world.verdict("lint", &build("d5f6"), Outcome::Pass, LANDING);

    let answer = pushed(&world);
    for (_, entry) in &answer.entries {
        written(entry);
    }
    let ops: Vec<StoreOp> = world
        .fakes
        .store
        .calls()
        .split_off(store)
        .into_iter()
        .map(|c| c.op)
        .collect();
    assert_eq!(
        ops,
        [
            StoreOp::PutBytes,
            StoreOp::Name,
            StoreOp::PutBytes,
            StoreOp::Name
        ]
    );
    assert_eq!(
        world.fakes.signer.calls().len(),
        signer,
        "the signer was asked"
    );
    assert_eq!(
        world.fakes.policy.calls().len(),
        policy,
        "the policy was asked"
    );
    assert_eq!(
        world.fakes.build.calls().len(),
        builds,
        "the build was asked"
    );
    assert_eq!(world.fakes.keeper.calls(), [], "the keeper was asked");
    assert_eq!(
        files(world.kept().dir()),
        record,
        "the host's record changed"
    );
    let named: Vec<String> = world
        .fakes
        .store
        .contents()
        .names
        .into_keys()
        .map(|n| n.to_string())
        .filter(|n| n.starts_with("cache/"))
        .collect();
    assert_eq!(named, ["cache/b7e1", "cache/d5f6"]);
}

#[test]
fn with_no_projection_that_packs_cache_entries_each_is_absent_and_nothing_written() {
    let world = landed_with(&[("b7e1", Outcome::Pass)]);
    for projection in world.fakes.projections.values() {
        projection.packs_no_cache_entries();
    }
    let answer = pushed(&world);
    match &answer.entries[0].1 {
        Err(Not::Absent(a)) => assert_eq!(a.what, "cache entries"),
        other => panic!("answered {other:?}"),
    }
    assert_eq!(store_ops(&world), []);
}
