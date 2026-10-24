//! Spec host-record: what this host keeps of the versions it published,
//! and status answered from it alone.

mod common;
mod world;

use std::fs;
use std::path::{Path, PathBuf};

use tailrace_core::{status, Entry, Filter, History, Kept, Published, StatusAsk, Written};
use tailrace_seams::{Branch, ComponentName, Digest, Not, Outcome, Revision, Source, Version};
use world::{World, HOST};

fn files(dir: &Path) -> Vec<(PathBuf, Vec<u8>, std::time::SystemTime)> {
    let mut found = Vec::new();
    if !dir.exists() {
        return found;
    }
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            found.extend(files(&path));
        } else {
            let meta = fs::metadata(&path).unwrap();
            found.push((
                path.clone(),
                fs::read(&path).unwrap(),
                meta.modified().unwrap(),
            ));
        }
    }
    found.sort();
    found
}

fn entry(component: &str, version: &str, revision: &str) -> Entry {
    Entry {
        component: ComponentName::new(component),
        version: Version::new(version),
        tag: format!("{component}-v{version}"),
        revision: Revision::new(revision),
        branch: Branch::new("main"),
        index: Digest::of(version.as_bytes()),
        store: world::store(),
    }
}

fn published(world: &World) -> (Version, Digest) {
    let answer = world.publish_one(&world.ask_for("lib")).unwrap();
    let Published::Version { version, index, .. } = answer else {
        panic!("{answer:?}")
    };
    (version, index)
}

// Requirement: The host keeps which versions it published from which revisions

#[test]
fn an_entry_after_a_publish_names_the_version_and_nothing_the_store_holds() {
    let world = World::new();
    let (version, index) = published(&world);
    let kept = world.kept();
    let entries = kept.read(&Filter::default()).unwrap();
    assert_eq!(
        entries,
        [Entry {
            component: ComponentName::new("lib"),
            version,
            tag: "lib-v0.2.0".into(),
            revision: world.head.clone(),
            branch: Branch::new("main"),
            index: index.clone(),
            store: world::store(),
        }]
    );
    let held = files(kept.dir());
    assert_eq!(held.len(), 1);
    assert_eq!(held[0].0, kept.dir().join("lib").join("lib-v0.2.0.json"));
    let text = String::from_utf8(held[0].1.clone()).unwrap();
    let store = world.fakes.store.contents();
    for (digest, _) in store.held.iter().filter(|(d, _)| **d != index) {
        assert!(!text.contains(digest.as_str()), "{text}");
    }
    for envelopes in store.attached.values() {
        for envelope in envelopes {
            assert!(!text.contains(envelope.digest().as_str()), "{text}");
        }
    }
    assert!(
        !text.contains("signature") && !text.contains(HOST),
        "{text}"
    );
}

#[test]
fn nothing_is_kept_of_a_refused_or_stopped_publish() {
    let world = World::new();
    world.made("lib-docs", Outcome::Fail);
    assert!(world.publish_one(&world.ask_for("lib")).is_err());
    assert_eq!(files(world.kept().dir()), []);

    let stopped = World::new();
    let history = stopped.history();
    let ask = tailrace_core::Ask {
        signer: tailrace_seams::SignerId::new("nobody"),
        ..stopped.ask_for("lib")
    };
    let publish =
        tailrace_core::publish(&stopped.fakes.seams(), &history, &Kept::at(&history), &ask)
            .unwrap();
    assert!(publish.components[0].1.is_err());
    assert!(
        !stopped.fakes.store.contents().held.is_empty(),
        "it stopped after a put"
    );
    assert_eq!(files(stopped.kept().dir()), []);
}

// Requirement: The record is a file a reader opens

#[test]
fn an_entry_is_read_as_plain_json_with_a_file_reader() {
    let world = World::new();
    let (version, index) = published(&world);
    let path = world.kept().dir().join("lib").join("lib-v0.2.0.json");
    let text = fs::read_to_string(path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["component"], "lib");
    assert_eq!(json["version"], version.as_str());
    assert_eq!(json["revision"], world.head.as_str());
    assert_eq!(json["index"], index.as_str());
}

#[test]
fn a_second_publish_leaves_the_entry_untouched() {
    let world = World::new();
    published(&world);
    let before = files(world.kept().dir());
    std::thread::sleep(std::time::Duration::from_millis(20));
    published(&world);
    assert_eq!(files(world.kept().dir()), before);
}

#[test]
fn an_entry_held_with_other_content_is_refused_and_left_as_it_is() {
    let world = World::new();
    let kept = world.kept();
    let first = entry("lib", "1.0.0", "4f2a9c1");
    assert_eq!(kept.write(&first), Ok(Written::Wrote));
    assert_eq!(kept.write(&first), Ok(Written::Held));
    let before = files(kept.dir());
    let other = Entry {
        revision: Revision::new("77b0e3d"),
        ..first.clone()
    };
    let Err(Not::Refused(r)) = kept.write(&other) else {
        panic!("another entry was written over the first")
    };
    let path = kept.dir().join("lib").join("lib-v1.0.0.json");
    assert_eq!(
        r.why,
        tailrace_seams::Why::KeptOther {
            path: path.display().to_string()
        }
    );
    assert_eq!(files(kept.dir()), before);
}

#[test]
fn two_worktrees_of_one_repository_share_one_record() {
    let world = World::new();
    let other = world.fixture.path("other");
    world.repo.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "elsewhere",
        other.to_str().unwrap(),
    ]);
    let (version, _) = published(&world);
    let there = Kept::at(&History::open(&other).unwrap());
    assert_eq!(there, world.kept());
    let answer = status(&there, &StatusAsk::default()).unwrap();
    assert_eq!(answer.components[0].versions[0].version, version);
}

// Requirement: Status answers from the record alone

#[test]
fn status_after_a_publish_answers_its_version_and_asks_no_seam() {
    let world = World::new();
    let (version, index) = published(&world);
    let calls = (
        world.fakes.store.calls().len(),
        world.fakes.signer.calls().len(),
        world.fakes.landing.calls().len(),
        world.fakes.build.calls().len(),
    );
    let answer = status(&world.kept(), &StatusAsk::default()).unwrap();
    assert_eq!(answer.components.len(), 1);
    assert_eq!(answer.components[0].component, ComponentName::new("lib"));
    let kept = &answer.components[0].versions[0];
    assert_eq!(
        (&kept.version, &kept.revision, &kept.index),
        (&version, &world.head, &index)
    );
    assert_eq!(answer.record, world.kept().dir().display().to_string());
    assert_eq!(
        calls,
        (
            world.fakes.store.calls().len(),
            world.fakes.signer.calls().len(),
            world.fakes.landing.calls().len(),
            world.fakes.build.calls().len(),
        )
    );
}

#[test]
fn status_writes_nothing() {
    let world = World::new();
    published(&world);
    let kept = world.kept();
    kept.write(&entry("web", "0.1.0", world.start.as_str()))
        .unwrap();
    let before = files(kept.dir());
    let parent = files(kept.dir().parent().unwrap());
    status(&kept, &StatusAsk::default()).unwrap();
    status(
        &kept,
        &StatusAsk {
            component: Some(ComponentName::new("web")),
            revision: None,
        },
    )
    .unwrap();
    assert_eq!(files(kept.dir()), before);
    assert_eq!(files(kept.dir().parent().unwrap()), parent);
}

#[test]
fn status_for_one_revision_answers_the_versions_it_became_and_no_other() {
    let world = World::new();
    let kept = world.kept();
    kept.write(&entry("lib", "0.1.0", world.start.as_str()))
        .unwrap();
    kept.write(&entry("web", "0.1.0", world.start.as_str()))
        .unwrap();
    published(&world);
    let at_start = status(
        &kept,
        &StatusAsk {
            component: None,
            revision: Some(world.start.clone()),
        },
    )
    .unwrap();
    let versions: Vec<(&str, &str)> = at_start
        .components
        .iter()
        .flat_map(|c| c.versions.iter())
        .map(|e| (e.component.as_str(), e.version.as_str()))
        .collect();
    assert_eq!(versions, [("lib", "0.1.0"), ("web", "0.1.0")]);
    let at_head = status(
        &kept,
        &StatusAsk {
            component: None,
            revision: Some(world.head.clone()),
        },
    )
    .unwrap();
    assert_eq!(at_head.components.len(), 1);
    assert_eq!(at_head.components[0].versions.len(), 1);
    assert_eq!(at_head.components[0].versions[0].version.as_str(), "0.2.0");
}

#[test]
fn versions_are_ordered_by_the_version_rules() {
    let world = World::new();
    let kept = world.kept();
    for version in ["1.10.0", "1.9.0", "1.10.0-pre.2", "1.2.0"] {
        kept.write(&entry("lib", version, "4f2a9c1")).unwrap();
    }
    let answer = status(&kept, &StatusAsk::default()).unwrap();
    let order: Vec<&str> = answer.components[0]
        .versions
        .iter()
        .map(|e| e.version.as_str())
        .collect();
    assert_eq!(order, ["1.2.0", "1.9.0", "1.10.0-pre.2", "1.10.0"]);
}

// Requirement: Status keeps absent and unreadable apart

#[test]
fn nothing_kept_is_absent_naming_the_record() {
    let world = World::new();
    let kept = world.kept();
    let Err(Not::Absent(a)) = status(&kept, &StatusAsk::default()) else {
        panic!("a record holding nothing answered")
    };
    assert_eq!(a.source, Source::host(kept.dir()));
    assert!(!kept.dir().exists(), "status made the record");
}

#[test]
fn an_entry_that_cannot_be_read_is_unreadable_with_the_parsers_words() {
    let world = World::new();
    published(&world);
    let kept = world.kept();
    let bad = kept.dir().join("lib").join("lib-v9.0.0.json");
    let text = "{\"component\": \"lib\", \"version\": 9";
    fs::write(&bad, text).unwrap();
    let Err(Not::Unreadable(u)) = status(&kept, &StatusAsk::default()) else {
        panic!("an entry that cannot be read was answered")
    };
    assert_eq!(u.what, bad.display().to_string());
    let words = serde_json::from_str::<Entry>(text).unwrap_err().to_string();
    assert_eq!(u.said.as_deref(), Some(words.as_str()));
    assert_eq!(u.source, Source::host(kept.dir()));
}
