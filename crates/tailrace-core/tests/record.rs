//! Spec host-record: the host's record rebuilt from the store and git to
//! the same state, over the fakes and a real history (design D3).

mod common;
mod world;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use tailrace_core::{
    answer_json, rebuild, Ask, Entry, Filter, Noted, Published, RebuildAsk, Rebuilt, Written, BOUND,
};
use tailrace_fakes::{Fault, StoreOp, Times};
use tailrace_seams::{Branch, Digest, Name, Not, Outcome, Report, Revision, Store, Why};
use world::{build_of, output, store, World};

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

/// The record's files by path and bytes, leaving out when each was
/// written.
fn bytes(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    files(dir).into_iter().map(|(p, b, _)| (p, b)).collect()
}

fn ask(world: &World, revision: &Revision) -> RebuildAsk {
    RebuildAsk {
        branch: world.main.clone(),
        revision: revision.clone(),
        store: store(),
    }
}

fn rebuilt(world: &World) -> Rebuilt {
    rebuild(
        &world.fakes.seams(),
        &world.history(),
        &world.kept(),
        &ask(world, &world.head),
    )
    .unwrap_or_else(|not| panic!("the rebuild answered {not:?}"))
}

/// `web` published at the start on `branch`, which has landed it, its
/// site built there; then `lib` published at the head on `main`.
fn published_on(world: &World, branch: &Branch) {
    world.fakes.landing.land(branch, &world.start);
    world.fakes.build.made(
        &world.start,
        &output("site"),
        Report {
            build: build_of("site", &world.start),
            outcome: Outcome::Pass,
            files: vec!["dist/site.tar".into()],
        },
    );
    let web = world.publish_one(&Ask {
        branch: branch.clone(),
        revision: world.start.clone(),
        ..world.ask_for("web")
    });
    assert!(matches!(web, Ok(Published::Version { .. })), "{web:?}");
    let lib = world.publish_one(&world.ask_for("lib"));
    assert!(matches!(lib, Ok(Published::Version { .. })), "{lib:?}");
}

/// Both components published on `main`: `web` 0.1.0 at the start and
/// `lib` 0.2.0 at the head, and `lib-v0.1.0` claimed at the start with
/// no index in the store; every entry the record holds.
fn published(world: &World) -> Vec<Entry> {
    published_on(world, &world.main.clone());
    world.kept().read(&Filter::default()).unwrap()
}

fn versions(rebuilt: &Rebuilt) -> Vec<(&str, &str)> {
    rebuilt
        .entries
        .iter()
        .map(|(e, _)| (e.component.as_str(), e.version.as_str()))
        .collect()
}

fn store_ops_since(world: &World, since: usize) -> Vec<StoreOp> {
    world
        .fakes
        .store
        .calls()
        .split_off(since)
        .into_iter()
        .map(|c| c.op)
        .collect()
}

// Requirement: The record is rebuilt from the store and git

#[test]
fn a_published_versions_entry_deleted_and_rebuilt_is_the_file_publish_wrote() {
    let world = World::new();
    let entries = published(&world);
    let kept = world.kept();
    let before = bytes(kept.dir());
    assert_eq!(before.len(), 2, "{before:?}");
    fs::remove_dir_all(kept.dir()).unwrap();

    let answer = rebuilt(&world);
    assert_eq!(answer.record, kept.dir().display().to_string());
    assert_eq!(versions(&answer), [("lib", "0.2.0"), ("web", "0.1.0")]);
    for (entry, written) in &answer.entries {
        assert_eq!(*written, Ok(Written::Wrote), "{entry:?}");
    }
    let mut rebuilt: Vec<Entry> = answer.entries.iter().map(|(e, _)| e.clone()).collect();
    rebuilt.sort_by(|a, b| a.component.cmp(&b.component));
    assert_eq!(rebuilt, entries);
    assert_eq!(bytes(kept.dir()), before, "the rebuilt record differs");
    let noted: Vec<Option<Noted>> = rebuilt.iter().map(|e| e.notes.clone()).collect();
    assert_eq!(
        noted,
        [
            Some(Noted {
                last: Some("lib-v0.1.0".into()),
                commits: vec![world.head.clone()],
            }),
            Some(Noted {
                last: None,
                commits: vec![world.start.clone()],
            }),
        ]
    );

    let json = answer_json(&Ok::<_, Not>(answer));
    assert_eq!(json["answer"], "ok");
    assert_eq!(json["value"]["record"], kept.dir().display().to_string());
    let entries = &json["value"]["entries"];
    assert_eq!(entries[0]["answer"], "ok");
    assert_eq!(entries[0]["value"], "wrote");
    assert_eq!(entries[0]["entry"]["tag"], "lib-v0.2.0");
    assert_eq!(entries[1]["entry"]["tag"], "web-v0.1.0");
}

#[test]
fn a_version_published_before_notes_is_rebuilt_naming_no_notes() {
    let world = World::new();
    world::published_before_notes(&world);
    let kept = world.kept();
    let before = bytes(kept.dir());
    fs::remove_dir_all(kept.dir()).unwrap();

    let answer = rebuilt(&world);
    assert_eq!(versions(&answer), [("lib", "0.2.0")]);
    assert_eq!(answer.entries[0].0.notes, None);
    assert_eq!(bytes(kept.dir()), before, "the rebuilt record differs");
}

#[test]
fn a_tag_claimed_with_no_name_in_the_store_gives_no_entry() {
    let world = World::new();
    published(&world);
    let kept = world.kept();
    fs::remove_dir_all(kept.dir()).unwrap();
    world.fakes.store.forget(&Name::new("web-v0.1.0"));

    let answer = rebuilt(&world);
    assert_eq!(
        versions(&answer),
        [("lib", "0.2.0")],
        "a version whose name the store does not hold was rebuilt"
    );
    assert!(!kept.dir().join("lib").join("lib-v0.1.0.json").exists());
    assert!(!kept.dir().join("web").exists());
}

#[test]
fn a_tag_on_a_revision_the_branch_did_not_land_gives_no_entry() {
    let world = World::new();
    let release = Branch::new("release");
    published_on(&world, &release);
    let kept = world.kept();
    fs::remove_dir_all(kept.dir()).unwrap();
    assert!(
        world
            .fakes
            .store
            .contents()
            .names
            .contains_key(&Name::new("web-v0.1.0")),
        "the store names no index for the other branch's version"
    );

    let answer = rebuilt(&world);
    assert_eq!(
        versions(&answer),
        [("lib", "0.2.0")],
        "a version another branch landed was rebuilt for main"
    );
    assert!(!kept.dir().join("web").exists());
}

// Requirement: A rebuild writes only the record

#[test]
fn a_rebuild_of_a_revision_not_landed_is_refused_and_the_store_asked_nothing() {
    let world = World::new();
    let answer = rebuild(
        &world.fakes.seams(),
        &world.history(),
        &world.kept(),
        &ask(&world, &world.start),
    );
    match answer {
        Err(Not::Refused(r)) => {
            assert_eq!(
                r.why,
                Why::NotLanded {
                    branch: world.main.clone(),
                    revision: world.start.clone(),
                }
            );
            assert_eq!(r.source.profile, BOUND);
        }
        other => panic!("answered {other:?}"),
    }
    assert_eq!(world.fakes.store.calls(), [], "the store was asked");
    assert!(!world.kept().dir().exists(), "the record was made");
}

#[test]
fn the_store_is_asked_only_to_resolve_and_read_and_the_signer_nothing() {
    let world = World::new();
    published(&world);
    fs::remove_dir_all(world.kept().dir()).unwrap();
    let (store, signer, build, policy) = (
        world.fakes.store.calls().len(),
        world.fakes.signer.calls().len(),
        world.fakes.build.calls().len(),
        world.fakes.policy.calls().len(),
    );
    let packs: Vec<usize> = world
        .fakes
        .projections
        .values()
        .map(|p| p.calls().len())
        .collect();

    rebuilt(&world);
    let ops = store_ops_since(&world, store);
    assert!(!ops.is_empty());
    assert!(
        ops.iter()
            .all(|op| matches!(op, StoreOp::Resolve | StoreOp::Read)),
        "the store was asked {ops:?}"
    );
    assert_eq!(
        world.fakes.signer.calls().len(),
        signer,
        "the signer was asked"
    );
    assert_eq!(
        world.fakes.build.calls().len(),
        build,
        "the build was asked"
    );
    assert_eq!(
        world.fakes.policy.calls().len(),
        policy,
        "the policy was asked"
    );
    let after: Vec<usize> = world
        .fakes
        .projections
        .values()
        .map(|p| p.calls().len())
        .collect();
    assert_eq!(after, packs, "a projection was asked");
}

// Requirement: A rebuild never rewrites what the record holds

#[test]
fn a_whole_record_rebuilt_answers_every_entry_held_and_writes_no_file() {
    let world = World::new();
    published(&world);
    let kept = world.kept();
    let before = files(kept.dir());
    std::thread::sleep(Duration::from_millis(20));

    let answer = rebuilt(&world);
    assert_eq!(answer.entries.len(), 2);
    for (entry, written) in &answer.entries {
        assert_eq!(*written, Ok(Written::Held), "{entry:?}");
    }
    assert_eq!(
        files(kept.dir()),
        before,
        "a file of the record was written"
    );
}

#[test]
fn an_entry_naming_another_index_is_refused_left_unchanged_and_the_others_written() {
    let world = World::new();
    let entries = published(&world);
    let kept = world.kept();
    let lib = kept.dir().join("lib").join("lib-v0.2.0.json");
    let other = Entry {
        index: Digest::of(b"another index"),
        ..entries[0].clone()
    };
    fs::remove_file(&lib).unwrap();
    kept.write(&other).unwrap();
    let held = fs::read(&lib).unwrap();
    fs::remove_dir_all(kept.dir().join("web")).unwrap();

    let answer = rebuilt(&world);
    match &answer.entries[0] {
        (entry, Err(Not::Refused(r))) => {
            assert_eq!(entry.tag, "lib-v0.2.0");
            assert_eq!(
                r.why,
                Why::KeptOther {
                    path: lib.display().to_string()
                }
            );
        }
        other => panic!("the entry that disagrees answered {other:?}"),
    }
    assert_eq!(fs::read(&lib).unwrap(), held, "the entry was rewritten");
    assert_eq!(answer.entries[1].1, Ok(Written::Wrote));
    let web = kept.dir().join("web").join("web-v0.1.0.json");
    let read: Entry = serde_json::from_slice(&fs::read(web).unwrap()).unwrap();
    assert_eq!(read, entries[1]);
}

// Requirement: A rebuild that cannot read is unreadable, and writes nothing

#[test]
fn a_store_that_cannot_be_reached_is_unreadable_with_its_words_and_writes_no_file() {
    let world = World::new();
    published(&world);
    let kept = world.kept();
    fs::remove_dir_all(kept.dir()).unwrap();
    world.fakes.store.fault(
        StoreOp::Resolve,
        Fault::Unreadable("connection refused".into()),
        Times::UntilHealed,
    );

    match rebuild(
        &world.fakes.seams(),
        &world.history(),
        &kept,
        &ask(&world, &world.head),
    ) {
        Err(Not::Unreadable(u)) => assert_eq!(u.said.as_deref(), Some("connection refused")),
        other => panic!("answered {other:?}"),
    }
    assert!(!kept.dir().exists(), "a file of the record was written");
}

#[test]
fn a_name_at_bytes_is_unreadable_naming_the_tag_and_the_digest_and_writes_no_file() {
    let world = World::new();
    let entries = published(&world);
    let kept = world.kept();
    fs::remove_dir_all(kept.dir()).unwrap();
    let store_handle = &world.fakes.store;
    let bytes = store_handle.put_bytes(b"not an index").unwrap().digest;
    let name = Name::new("web-v0.1.0");
    store_handle
        .name(&name, &bytes, Some(&entries[1].index))
        .unwrap();

    match rebuild(
        &world.fakes.seams(),
        &world.history(),
        &kept,
        &ask(&world, &world.head),
    ) {
        Err(Not::Unreadable(u)) => {
            assert!(u.what.contains("web-v0.1.0"), "{}", u.what);
            assert!(u.what.contains(bytes.as_str()), "{}", u.what);
            assert_eq!(u.source, store());
        }
        other => panic!("answered {other:?}"),
    }
    assert!(
        !kept.dir().exists(),
        "the entry read before the bad one was written"
    );
}
