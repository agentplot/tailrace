//! The status group: what the host published, answered from its own
//! record with no seam asked and nothing written (58, 60 to 62).

use std::fs;

use serde_json::Value;
use tailrace_core::{status, Entry, Published, StatusAsk};
use tailrace_seams::{Branch, ComponentName, Not, Source};

use crate::core::publish::{one, passing, TAG, VERSION};
use crate::world::FakeWorld;
use crate::{ok, Scenario};

pub const GROUP: &str = "status";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        Scenario {
            name: "status after a publish",
            plays: None,
            cites: &[60, 61],
            run: status_after_a_publish,
        },
        Scenario {
            name: "the record read with a file reader",
            plays: None,
            cites: &[62],
            run: the_record_read_with_a_file_reader,
        },
        Scenario {
            name: "nothing kept, and an entry that cannot be read",
            plays: None,
            cites: &[58],
            run: nothing_kept_and_an_entry_that_cannot_be_read,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}

/// The one component of the world, `repository`.
const COMPONENT: &str = "repository";

fn published(world: &FakeWorld) -> Published {
    passing(world, &world.head);
    ok(one(world.publish(&world.ask(&world.head))), "the publish")
}

fn status_after_a_publish(world: &FakeWorld) {
    let published = published(world);
    let Published::Version { index, store, .. } = &published else {
        panic!("the head was not published: {published:?}")
    };
    let asked = world.asked();
    let record = world.record();
    assert!(!record.is_empty(), "the publish recorded nothing");

    let answer = ok(
        status(&world.history(), &world.kept(), &StatusAsk::default()),
        "status",
    );
    assert_eq!(answer.components.len(), 1, "{answer:?}");
    let versions = &answer.components[0];
    assert_eq!(versions.component, ComponentName::new(COMPONENT));
    let entries: Vec<&Entry> = versions.versions.iter().map(|s| &s.entry).collect();
    assert_eq!(
        entries,
        [&Entry {
            component: ComponentName::new(COMPONENT),
            version: tailrace_seams::Version::new(VERSION),
            tag: TAG.into(),
            revision: world.head.clone(),
            branch: Branch::new(crate::fixture::MAIN),
            index: index.clone(),
            store: store.clone(),
            notes: None,
        }],
        "status answered another version, revision or index than the publish's"
    );
    assert_eq!(world.asked(), asked, "status asked a seam");
    assert_eq!(world.record(), record, "status wrote to the record");
}

fn the_record_read_with_a_file_reader(world: &FakeWorld) {
    let published = published(world);
    let Published::Version {
        index, artifacts, ..
    } = &published
    else {
        unreachable!()
    };
    let path = world
        .kept()
        .dir()
        .join(COMPONENT)
        .join(format!("{TAG}.json"));
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} cannot be read as a file: {e}", path.display()));
    let entry: Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("{} is not plain JSON: {e}", path.display()));
    for (key, value) in [
        ("component", COMPONENT),
        ("version", VERSION),
        ("revision", world.head.as_str()),
        ("branch", crate::fixture::MAIN),
        ("index", index.as_str()),
    ] {
        assert_eq!(entry[key], value, "the entry names another {key}: {text}");
    }
    assert_eq!(
        entry["store"]["seam"], "store",
        "the entry names no store: {text}"
    );
    for artifact in artifacts {
        assert!(
            !text.contains(artifact.digest.as_str()) && !text.contains(artifact.bytes.as_str()),
            "the entry keeps what the store holds: {text}"
        );
        for claim in &artifact.claims {
            assert!(
                !text.contains(claim.envelope.as_str()),
                "the entry keeps a claim: {text}"
            );
        }
    }
    assert!(
        !text.contains("payload") && !text.contains("signature"),
        "the entry keeps a claim: {text}"
    );
}

fn nothing_kept_and_an_entry_that_cannot_be_read(world: &FakeWorld) {
    let kept = world.kept();
    match status(&world.history(), &kept, &StatusAsk::default()) {
        Err(Not::Absent(a)) => assert_eq!(
            a.source,
            Source::host(kept.dir()),
            "the absence names another directory"
        ),
        other => panic!("status of an empty record answered {other:?}"),
    }
    assert!(!kept.dir().exists(), "status made the record");

    let bad = kept.dir().join(COMPONENT).join("v9.0.0.json");
    fs::create_dir_all(bad.parent().expect("under the record")).expect("the record is writable");
    fs::write(&bad, "{\"component\": \"repository\", \"version\": 9").expect("writable");
    match status(&world.history(), &kept, &StatusAsk::default()) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.what, bad.display().to_string(), "{u:?}");
            assert_eq!(u.source, Source::host(kept.dir()));
            assert!(
                u.said.is_some(),
                "the parser's words were not carried: {u:?}"
            );
        }
        other => panic!("status of an entry that cannot be read answered {other:?}"),
    }
}
