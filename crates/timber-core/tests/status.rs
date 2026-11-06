//! Spec host-record: what this host keeps of the versions it published,
//! and status answered from it alone; and spec channel-status: every
//! channel the policy declares, read from the ledger alone, beside it.

mod common;
mod world;

use std::fs;
use std::path::{Path, PathBuf};

use timber_core::{
    overview, row, status, Ask, Channel, ChannelStatus, Entry, EnvironmentCounts, Filter, Head,
    HeadAt, History, Kept, Logged, Noted, Overview, OverviewAsk, Published, RepositoryName, Shown,
    StatusAsk, Written, LOCAL, NOTES,
};
use timber_fakes::{FakeSigner, Fault, KeeperOp, LandingOp, StoreOp, Times};
use timber_seams::{
    Audience, Branch, ChannelId, ChannelName, ComponentName, Digest, Envelope, Held, Keeper,
    Manifest, Moved, Moves, Not, Outcome, Promotion, Read, Required, Revision, Satisfied, Seam,
    Signer, SignerId, Source, Store, TargetProfile, Version, Why,
};
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
        notes: None,
    }
}

fn published(world: &World) -> (Version, Digest) {
    let answer = world.publish_one(&world.ask_for("lib")).unwrap();
    let Published::Version { version, index, .. } = answer else {
        panic!("{answer:?}")
    };
    (version, index)
}

/// The bytes of the notes the index at `index` names, as the fake store
/// holds them, as text.
fn notes_held(world: &World, index: &Digest) -> String {
    let held = world.fakes.store.contents().held;
    let Held::Manifest(Manifest::Index(meta)) = &held[index] else {
        panic!("no index under {index}")
    };
    let notes = meta
        .artifacts()
        .iter()
        .find(|a| a.meta.ecosystem().as_str() == NOTES)
        .expect("the index names notes");
    let Held::Manifest(Manifest::Artifact { layers, .. }) = &held[&notes.digest] else {
        panic!("no artifact under {}", notes.digest)
    };
    let Held::Bytes(bytes) = &held[&layers[0]] else {
        panic!("no bytes under {}", layers[0])
    };
    String::from_utf8(bytes.clone()).unwrap()
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
            notes: Some(Noted {
                last: Some("lib-v0.1.0".into()),
                commits: vec![world.head.clone()],
            }),
        }]
    );
    let held = files(kept.dir());
    assert_eq!(held.len(), 1);
    assert_eq!(held[0].0, kept.dir().join("lib").join("lib-v0.2.0.json"));
    let text = String::from_utf8(held[0].1.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        json["notes"],
        serde_json::json!({ "last": "lib-v0.1.0", "commits": [world.head.as_str()] })
    );
    assert!(
        !text.contains("lib grows") && !text.contains("##"),
        "{text}"
    );
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
    let ask = timber_core::Ask {
        signer: timber_seams::SignerId::new("nobody"),
        ..stopped.ask_for("lib")
    };
    let publish =
        timber_core::publish(&stopped.fakes.seams(), &history, &Kept::at(&history), &ask).unwrap();
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
fn an_entry_written_before_notes_reads_back_naming_none_and_is_answered_held() {
    let world = World::new();
    world::published_before_notes(&world);
    let path = world.kept().dir().join("lib").join("lib-v0.2.0.json");
    let text = fs::read_to_string(&path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        json.as_object().unwrap().keys().collect::<Vec<_>>(),
        [
            "branch",
            "component",
            "index",
            "revision",
            "store",
            "tag",
            "version"
        ],
        "{text}"
    );
    let entry: Entry = serde_json::from_str(&text).unwrap();
    assert_eq!(entry.notes, None);

    let before = files(world.kept().dir());
    std::thread::sleep(std::time::Duration::from_millis(20));
    assert_eq!(world.kept().write(&entry), Ok(Written::Held));
    let again = world.publish_one(&world.ask_for("lib")).unwrap();
    assert!(
        matches!(&again, Published::Version { notes: None, .. }),
        "{again:?}"
    );
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
        timber_seams::Why::KeptOther {
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
    let answer = status(&world.history(), &there, &StatusAsk::default()).unwrap();
    assert_eq!(answer.components[0].versions[0].entry.version, version);
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
    let answer = status(&world.history(), &world.kept(), &StatusAsk::default()).unwrap();
    assert_eq!(answer.components.len(), 1);
    assert_eq!(answer.components[0].component, ComponentName::new("lib"));
    let shown = &answer.components[0].versions[0];
    let kept = &shown.entry;
    assert_eq!(
        (&kept.version, &kept.revision, &kept.index),
        (&version, &world.head, &index)
    );
    assert_eq!(shown.notes, Some(Ok(notes_held(&world, &index))));
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

// Requirement: Status shows each version's notes from git

#[test]
fn the_notes_status_shows_are_the_notes_the_version_carries() {
    let world = World::new();
    let (_, index) = published(&world);
    let answer = status(&world.history(), &world.kept(), &StatusAsk::default()).unwrap();
    let shown = &answer.components[0].versions[0];
    let held = notes_held(&world, &index);
    assert_eq!(shown.notes, Some(Ok(held.clone())));

    let json = serde_json::to_value(shown).unwrap();
    assert_eq!(json["notes"]["answer"], "ok");
    assert_eq!(json["notes"]["value"], held.as_str());
    assert_eq!(json["version"], "0.2.0");
    assert_eq!(json["index"], index.as_str());
}

#[test]
fn a_version_with_no_notes_shows_none_and_one_whose_commit_is_gone_shows_them_unreadable() {
    let world = World::new();
    let kept = world.kept();
    kept.write(&entry("lib", "0.1.0", world.start.as_str()))
        .unwrap();
    let gone = Revision::new("4f2a9c1e0b7d3a5f6c8e9d0a1b2c3d4e5f6a7b8c");
    let lost = Entry {
        notes: Some(Noted {
            last: Some("web-v0.1.0".into()),
            commits: vec![world.start.clone(), gone.clone()],
        }),
        ..entry("web", "0.2.0", world.start.as_str())
    };
    kept.write(&lost).unwrap();

    let answer = status(&world.history(), &kept, &StatusAsk::default()).unwrap();
    let shown: Vec<&Shown> = answer
        .components
        .iter()
        .flat_map(|c| c.versions.iter())
        .collect();
    assert_eq!(shown.len(), 2, "{answer:?}");
    assert_eq!(shown[0].entry, entry("lib", "0.1.0", world.start.as_str()));
    assert_eq!(shown[0].notes, None);
    assert_eq!(shown[1].entry, lost);
    let Some(Err(Not::Unreadable(u))) = &shown[1].notes else {
        panic!(
            "notes rendered from a commit the history lacks: {:?}",
            shown[1]
        )
    };
    assert!(u.what.contains(gone.as_str()), "{u:?}");
    assert!(u.said.as_deref().unwrap().contains(gone.as_str()), "{u:?}");

    let json = serde_json::to_value(&answer).unwrap();
    let versions: Vec<&serde_json::Value> = json["components"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|c| c["versions"].as_array().unwrap())
        .collect();
    assert!(versions[0].get("notes").is_none(), "{json}");
    assert_eq!(versions[1]["notes"]["answer"], "unreadable", "{json}");
    assert_eq!(versions[1]["revision"], world.start.as_str());
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
    status(&world.history(), &kept, &StatusAsk::default()).unwrap();
    status(
        &world.history(),
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
        &world.history(),
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
        .map(|s| (s.entry.component.as_str(), s.entry.version.as_str()))
        .collect();
    assert_eq!(versions, [("lib", "0.1.0"), ("web", "0.1.0")]);
    let at_head = status(
        &world.history(),
        &kept,
        &StatusAsk {
            component: None,
            revision: Some(world.head.clone()),
        },
    )
    .unwrap();
    assert_eq!(at_head.components.len(), 1);
    assert_eq!(at_head.components[0].versions.len(), 1);
    assert_eq!(
        at_head.components[0].versions[0].entry.version.as_str(),
        "0.2.0"
    );
}

#[test]
fn versions_are_ordered_by_the_version_rules() {
    let world = World::new();
    let kept = world.kept();
    for version in ["1.10.0", "1.9.0", "1.10.0-pre.2", "1.2.0"] {
        kept.write(&entry("lib", version, "4f2a9c1")).unwrap();
    }
    let answer = status(&world.history(), &kept, &StatusAsk::default()).unwrap();
    let order: Vec<&str> = answer.components[0]
        .versions
        .iter()
        .map(|s| s.entry.version.as_str())
        .collect();
    assert_eq!(order, ["1.2.0", "1.9.0", "1.10.0-pre.2", "1.10.0"]);
}

// Requirement: Status keeps absent and unreadable apart

#[test]
fn nothing_kept_is_absent_naming_the_record() {
    let world = World::new();
    let kept = world.kept();
    let Err(Not::Absent(a)) = status(&world.history(), &kept, &StatusAsk::default()) else {
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
    let Err(Not::Unreadable(u)) = status(&world.history(), &kept, &StatusAsk::default()) else {
        panic!("an entry that cannot be read was answered")
    };
    assert_eq!(u.what, bad.display().to_string());
    let words = serde_json::from_str::<Entry>(text).unwrap_err().to_string();
    assert_eq!(u.said.as_deref(), Some(words.as_str()));
    assert_eq!(u.source, Source::host(kept.dir()));
}

// Requirement: Status shows every channel the policy declares

/// The moment every promotion here was made at.
const AT: u64 = 1_793_000_000;

/// The channel `name` of `component`, as the world declares components
/// (125).
fn of(component: &str, name: &str) -> ChannelId {
    ChannelId::new(Some(ComponentName::new(component)), ChannelName::new(name))
}

fn beta() -> ChannelId {
    of("lib", "beta")
}

fn stable() -> ChannelId {
    of("lib", "stable")
}

/// `channel` declared at every revision, bound to `bound_to` where it
/// names one.
fn declare(world: &World, channel: &ChannelId, bound_to: Option<&str>) {
    world.fakes.policy.require(
        Audience::Channel(channel.clone()),
        Required::Channel {
            claims: vec![],
            bound_to: bound_to.map(ChannelName::new),
            promoted_by: vec![],
            approvers: vec![],
        },
    );
}

fn moved(version: &str) -> Moved {
    Moved {
        version: Version::new(version),
        index: Digest::of(format!("{version}'s index").as_bytes()),
    }
}

/// A promotion of `channel` in `lib` to `to`, from `from`, after the
/// claim `predecessor`, signed by the gate as the fake signer signs.
fn sealed(
    world: &World,
    channel: &ChannelId,
    to: &str,
    from: Option<&str>,
    predecessor: Option<&Digest>,
    at: u64,
) -> (Promotion, Envelope) {
    let from = from.map(moved);
    sealed_to(world, channel, &moved(to), from.as_ref(), predecessor, at)
}

/// A promotion of `channel` to the version and index `to`, from `from`,
/// after the claim `predecessor`, signed by the gate as the fake signer
/// signs.
fn sealed_to(
    world: &World,
    channel: &ChannelId,
    to: &Moved,
    from: Option<&Moved>,
    predecessor: Option<&Digest>,
    at: u64,
) -> (Promotion, Envelope) {
    let promotion = Promotion {
        channel: channel.name.clone(),
        component: channel.component_name(),
        to: to.clone(),
        from: from.cloned(),
        predecessor: predecessor.cloned(),
        asked_by: SignerId::new("gate"),
        policy: Satisfied {
            revision: world.head.clone(),
            claims: vec![Digest::of(b"acceptance")],
        },
        at,
    };
    let gate = FakeSigner::new();
    gate.hold(&SignerId::new("gate"));
    let envelope = gate
        .sign(
            &promotion.claim(world.start.clone()),
            &SignerId::new("gate"),
        )
        .unwrap();
    (promotion, envelope)
}

/// `channel` promoted in the fake keeper to each version in turn, each
/// from the one before: every promotion with its envelope's digest.
fn promoted(world: &World, channel: &ChannelId, versions: &[&str]) -> Vec<(Digest, Promotion)> {
    let moves: Vec<Moved> = versions.iter().map(|v| moved(v)).collect();
    promoted_to(world, channel, &moves)
}

/// `channel` promoted in the fake keeper to each version and index in
/// turn, each from the one before: every promotion with its envelope's
/// digest.
fn promoted_to(world: &World, channel: &ChannelId, moves: &[Moved]) -> Vec<(Digest, Promotion)> {
    let mut chain: Vec<(Digest, Promotion)> = Vec::new();
    for (i, to) in moves.iter().enumerate() {
        let from = i.checked_sub(1).map(|j| &moves[j]);
        let predecessor = chain.last().map(|(d, _)| d.clone());
        let (promotion, envelope) = sealed_to(
            world,
            channel,
            to,
            from,
            predecessor.as_ref(),
            AT + i as u64,
        );
        let digest = world
            .fakes
            .keeper
            .append(channel, &envelope, predecessor.as_ref())
            .unwrap();
        chain.push((digest, promotion));
    }
    chain
}

fn ask(world: &World) -> OverviewAsk {
    OverviewAsk {
        branch: world.main.clone(),
        component: None,
        revision: None,
        at: AT + 100,
    }
}

fn asked(world: &World, ask: &OverviewAsk) -> Overview {
    let history = world.history();
    overview(&world.fakes.seams(), &history, &Kept::at(&history), ask)
}

fn channels_of(overview: &Overview) -> &[ChannelStatus] {
    match &overview.channels {
        Ok(channels) => &channels.channels,
        Err(not) => panic!("the channels answered {not:?}"),
    }
}

#[test]
fn every_channel_is_at_its_version_or_empty_with_the_chain_that_put_it_there() {
    let world = World::new();
    declare(&world, &beta(), None);
    declare(&world, &stable(), Some("beta"));
    let chain = promoted(&world, &stable(), &["0.1.0", "0.2.0"]);

    let answer = asked(&world, &ask(&world));
    assert_eq!(answer.branch, world.main);
    assert_eq!(answer.asked_at, AT + 100);
    assert_eq!(
        answer.channels.as_ref().map(|c| &c.revision),
        Ok(&world.head)
    );
    let channels = channels_of(&answer);
    let names: Vec<&str> = channels.iter().map(|c| c.name().as_str()).collect();
    assert_eq!(names, ["beta", "stable"]);
    assert_eq!(
        channels[0].declared.channel.component_name(),
        ComponentName::new("lib")
    );
    assert_eq!(channels[0].declared.moves, Moves::Promotion);
    assert_eq!(channels[0].at, Ok(Channel::Empty { read: Read::Source }));
    assert_eq!(channels[1].declared.bound_to(), Some(beta()));
    assert_eq!(
        channels[1].at,
        Ok(Channel::At {
            head: chain[1].1.clone(),
            chain: chain.clone(),
            read: Read::Source,
        })
    );

    // A host that never published: the record's part is absent, naming
    // the record, apart from the channels.
    let Err(Not::Absent(a)) = &answer.published else {
        panic!("the record answered {:?}", answer.published)
    };
    assert_eq!(a.source, Source::host(world.kept().dir()));

    // Each inner answer is in the one form every surface renders (57).
    let json = serde_json::to_value(&answer).unwrap();
    assert_eq!(json["asked_at"], AT + 100);
    assert_eq!(json["channels"]["answer"], "ok", "{json}");
    let listed = &json["channels"]["value"]["channels"];
    assert_eq!(listed[0]["channel"]["name"], "beta", "{json}");
    assert_eq!(listed[0]["channel"]["component"], "lib", "{json}");
    assert_eq!(listed[0]["moves"]["by"], "promotion", "{json}");
    assert_eq!(listed[0]["at"]["answer"], "ok", "{json}");
    assert_eq!(listed[0]["at"]["value"]["state"], "empty", "{json}");
    assert_eq!(listed[1]["moves"]["by"], "bound", "{json}");
    assert_eq!(listed[1]["moves"]["to"], "beta", "{json}");
    assert_eq!(listed[1]["at"]["value"]["state"], "at", "{json}");
    assert_eq!(listed[1]["at"]["value"]["head"]["to"]["version"], "0.2.0");
    assert_eq!(json["published"]["answer"], "absent", "{json}");
}

#[test]
fn a_channel_of_another_component_is_left_out_when_one_is_named() {
    let world = World::new();
    declare(&world, &beta(), None);
    declare(&world, &of("web", "preview"), None);
    let all = asked(&world, &ask(&world));
    assert_eq!(channels_of(&all).len(), 2);
    let lib = asked(
        &world,
        &OverviewAsk {
            component: Some(ComponentName::new("lib")),
            ..ask(&world)
        },
    );
    let names: Vec<&str> = channels_of(&lib)
        .iter()
        .map(|c| c.name().as_str())
        .collect();
    assert_eq!(names, ["beta"]);
}

#[test]
fn a_policy_declaring_no_channel_answers_an_empty_list() {
    let world = World::new();
    let answer = asked(&world, &ask(&world));
    assert_eq!(channels_of(&answer), []);
}

#[test]
fn a_head_not_landed_refuses_the_channels_and_the_record_still_answers() {
    let world = World::new();
    declare(&world, &beta(), None);
    let (version, _) = published(&world);
    let unlanded = world.repo.touch("feat: lib grows again", &["crates/lib"]);
    let answer = asked(&world, &ask(&world));
    let Err(Not::Refused(r)) = &answer.channels else {
        panic!("the channels answered {:?}", answer.channels)
    };
    assert_eq!(
        r.why,
        Why::NotLanded {
            branch: world.main.clone(),
            revision: Revision::new(unlanded.as_str()),
        }
    );
    let published = answer.published.as_ref().expect("the record answered");
    assert_eq!(published.components[0].versions[0].entry.version, version);
}

// Requirement: Each channel's answer is its own

#[test]
fn one_chain_unreadable_and_one_forked_are_each_that_channels_own_answer() {
    let world = World::new();
    let edge = of("lib", "edge");
    for channel in [beta(), edge.clone(), stable()] {
        declare(&world, &channel, None);
    }
    promoted(&world, &beta(), &["0.1.0"]);
    let chain = promoted(&world, &stable(), &["0.1.0"]);
    let (_, a) = sealed(&world, &edge, "0.1.0", None, None, AT);
    let (_, b) = sealed(&world, &edge, "0.2.0", None, None, AT + 1);
    world.fakes.keeper.fork(&edge, &a, &b);
    world.fakes.keeper.fault(
        KeeperOp::Chain,
        Fault::Unreadable("fatal: unable to access".into()),
        Times::Once,
    );

    let answer = asked(&world, &ask(&world));
    let channels = channels_of(&answer);
    let Err(Not::Unreadable(u)) = &channels[0].at else {
        panic!("beta answered {:?}", channels[0].at)
    };
    assert_eq!(u.source.seam, Seam::Keeper);
    assert_eq!(u.said.as_deref(), Some("fatal: unable to access"));
    let Err(Not::Refused(r)) = &channels[1].at else {
        panic!("edge answered {:?}", channels[1].at)
    };
    assert_eq!(
        r.why,
        Why::Fork {
            predecessor: None,
            claims: vec![a.digest(), b.digest()],
        }
    );
    assert!(
        matches!(&channels[2].at, Ok(Channel::At { head, .. }) if *head == chain[0].1),
        "stable answered {:?}",
        channels[2].at
    );
}

// Requirement: Every reader gets one answer as of when it read

#[test]
fn a_chain_answered_as_last_read_is_shown_with_its_time_and_words() {
    let world = World::new();
    declare(&world, &stable(), None);
    let chain = promoted(&world, &stable(), &["0.1.0", "0.2.0"]);
    let before = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let first = asked(&world, &ask(&world));
    world.fakes.keeper.unreachable("fatal: unable to access");
    let again = asked(&world, &ask(&world));
    let Ok(Channel::At {
        head,
        chain: read,
        read: Read::Kept { at, said },
    }) = &channels_of(&again)[0].at
    else {
        panic!("stable answered {:?}", channels_of(&again)[0].at)
    };
    assert!(*at >= before, "kept at {at}, first read after {before}");
    assert_eq!(said, "fatal: unable to access");
    assert_eq!((head, read), (&chain[1].1, &chain));
    assert!(matches!(
        &channels_of(&first)[0].at,
        Ok(Channel::At {
            read: Read::Source,
            ..
        })
    ));
}

#[test]
fn a_landing_that_cannot_be_read_is_the_channels_unreadable_answer_and_the_rest_still_answers() {
    let world = World::new();
    declare(&world, &stable(), None);
    published(&world);
    promoted(&world, &stable(), &["0.1.0"]);
    let before = asked(&world, &ask(&world));
    assert_eq!(channels_of(&before).len(), 1);
    world.fakes.landing.fault(
        LandingOp::Landed,
        Fault::Unreadable("fatal: unable to access 'https://example.com/o/r.git/'".into()),
        Times::Once,
    );
    let asked_of_keeper = world.fakes.keeper.calls().len();

    let answer = asked(&world, &ask(&world));
    let Err(Not::Unreadable(u)) = &answer.channels else {
        panic!("the channels answered {:?}", answer.channels)
    };
    assert_eq!(u.source.seam, Seam::Landing);
    assert_eq!(
        u.said.as_deref(),
        Some("fatal: unable to access 'https://example.com/o/r.git/'")
    );
    assert_eq!(answer.published, before.published);
    assert!(answer.published.is_ok(), "{:?}", answer.published);
    assert_eq!(
        answer.environments.as_ref().err(),
        Some(&Not::Unreadable(u.clone())),
        "the environments are not the landing's answer"
    );
    assert!(before.environments.is_ok(), "{:?}", before.environments);
    assert_eq!(
        world.fakes.keeper.calls().len(),
        asked_of_keeper,
        "the keeper was asked with the landing unread"
    );
    let json = serde_json::to_value(&answer).unwrap();
    assert_eq!(json["channels"]["answer"], "unreadable", "{json}");
}

// Requirement: Status says when the landing was fetched

const UNREACHABLE: &str = "fatal: unable to access 'https://example.com/o/r.git/': \
                           Could not resolve host: example.com\n";

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[test]
fn a_landing_read_as_last_fetched_shows_every_channel_and_says_when() {
    let world = World::new();
    declare(&world, &beta(), None);
    declare(&world, &stable(), None);
    promoted(&world, &stable(), &["0.1.0", "0.2.0"]);
    // 0.1.0's tag is at the start: landed, its notes are rendered from
    // the landing's copy as from the landing.
    world.fakes.landing.land(&world.main, &world.start);
    let before = now();
    let first = asked(&world, &ask(&world));
    let after = now();
    assert_eq!(
        first.channels.as_ref().map(|c| &c.landing),
        Ok(&Read::Source)
    );
    world.fakes.landing.unreachable(UNREACHABLE);

    let again = asked(&world, &ask(&world));
    let channels = again.channels.as_ref().expect("the channels answered");
    let Read::Kept { at, said } = &channels.landing else {
        panic!("the landing was read {:?}", channels.landing)
    };
    assert!((before..=after).contains(at), "fetched at {at}");
    assert_eq!(said, UNREACHABLE);
    assert_eq!(channels.revision, world.head);
    assert_eq!(channels_of(&again), channels_of(&first));
    assert!(channels_of(&again).iter().all(|c| c.at.is_ok()));
    assert!(
        channels_of(&again)[1].notes[0].notes.is_ok(),
        "{:?}",
        channels_of(&again)[1].notes
    );
    assert_eq!(again.published, first.published);

    let json = serde_json::to_value(&again).unwrap();
    let landing = &json["channels"]["value"]["landing"]["kept"];
    assert_eq!(landing["at"], *at, "{json}");
    assert_eq!(landing["said"], UNREACHABLE, "{json}");
    let json = serde_json::to_value(&first).unwrap();
    assert_eq!(json["channels"]["value"]["landing"], "source", "{json}");
}

#[test]
fn a_head_not_landed_as_last_fetched_refuses_the_channels_and_the_record_still_answers() {
    let world = World::new();
    declare(&world, &beta(), None);
    let before = now();
    let (version, _) = published(&world);
    let after = now();
    let later = world.repo.touch("feat: lib grows again", &["crates/lib"]);
    world.fakes.landing.land(&world.main, &later);
    world.fakes.landing.unreachable(UNREACHABLE);

    let answer = asked(&world, &ask(&world));
    let Err(Not::Refused(r)) = &answer.channels else {
        panic!("the channels answered {:?}", answer.channels)
    };
    let Why::NotLandedAsFetched {
        branch,
        revision,
        fetched,
    } = &r.why
    else {
        panic!("{} is not a refusal as last fetched", r.why)
    };
    assert_eq!((branch, revision), (&world.main, &later));
    assert!((before..=after).contains(fetched), "fetched at {fetched}");
    assert_eq!(r.said.as_deref(), Some(UNREACHABLE));
    assert!(r.why.to_string().contains("ask again"), "{}", r.why);
    let published = answer.published.as_ref().expect("the record answered");
    assert_eq!(published.components[0].versions[0].entry.version, version);
}

// Requirement: Asking reaches no signer or target and writes nothing but
// the host's copy

#[test]
fn status_declaring_no_environment_asks_no_store_signer_target_or_projection_and_writes_nothing() {
    let mut world = World::new();
    let production = world.fakes.target(&TargetProfile::new("fake"));
    declare(&world, &beta(), None);
    declare(&world, &stable(), None);
    published(&world);
    promoted(&world, &stable(), &["0.1.0", "0.2.0"]);
    let kept = world.kept();
    let record = files(kept.dir());
    let calls = || {
        (
            world.fakes.store.calls().len(),
            world.fakes.signer.calls().len(),
            production.calls().len(),
            world
                .fakes
                .projections
                .values()
                .map(|p| p.calls().len())
                .sum::<usize>(),
        )
    };
    let appends = || {
        world
            .fakes
            .keeper
            .calls()
            .iter()
            .filter(|c| c.op == KeeperOp::Append)
            .count()
    };
    let (before, appended) = (calls(), appends());
    let answer = asked(&world, &ask(&world));
    assert_eq!(channels_of(&answer).len(), 2);
    assert!(answer.published.is_ok(), "{:?}", answer.published);
    assert_eq!(
        calls(),
        before,
        "status asked the store, a signer, a target or a projection"
    );
    assert_eq!(appends(), appended, "status appended to the ledger");
    assert_eq!(files(kept.dir()), record, "status wrote to the record");
}

// Requirement: A channel's changelog is its chain read with each
// promotion's notes

/// `lib` published at the head, 0.2.0, and again at `later`, one commit
/// on from it: each version and its index.
fn published_again(world: &World, later: &Revision) -> [Moved; 2] {
    let (version, index) = published(world);
    let answer = world
        .publish_one(&Ask {
            revision: later.clone(),
            ..world.ask_for("lib")
        })
        .unwrap();
    let (again, at) = world::version_of(&answer);
    [
        Moved { version, index },
        Moved {
            version: again.clone(),
            index: at.clone(),
        },
    ]
}

/// `lib` published at the head, 0.2.0, and one feature later, 0.3.0.
fn two_published(world: &World) -> [Moved; 2] {
    let later = world.next("feat: lib grows more");
    published_again(world, &later)
}

/// `moved`'s notes as the fake store holds them, answered.
fn held_notes(world: &World, moved: &Moved) -> Logged {
    Logged {
        version: moved.version.clone(),
        index: moved.index.clone(),
        notes: Ok(notes_held(world, &moved.index)),
    }
}

#[test]
fn a_channel_promoted_to_two_versions_answers_each_versions_notes_as_its_index_holds_them() {
    let world = World::new();
    declare(&world, &stable(), None);
    let [first, second] = two_published(&world);
    let chain = promoted_to(&world, &stable(), &[first.clone(), second.clone()]);

    let answer = asked(&world, &ask(&world));
    let stable = &channels_of(&answer)[0];
    assert_eq!(
        stable.at,
        Ok(Channel::At {
            head: chain[1].1.clone(),
            chain: chain.clone(),
            read: Read::Source,
        })
    );
    for (_, promotion) in &chain {
        assert_eq!(promotion.asked_by, SignerId::new("gate"));
    }
    assert_eq!(
        stable.notes,
        [held_notes(&world, &first), held_notes(&world, &second)]
    );
    assert_ne!(stable.notes[0].notes, stable.notes[1].notes);

    // Beside the channel, each version's notes in the one form (57).
    let json = serde_json::to_value(&answer).unwrap();
    let notes = &json["channels"]["value"]["channels"][0]["notes"];
    assert_eq!(notes[0]["version"], "0.2.0", "{json}");
    assert_eq!(notes[0]["index"], first.index.as_str(), "{json}");
    assert_eq!(notes[0]["notes"]["answer"], "ok", "{json}");
    assert_eq!(
        notes[0]["notes"]["value"],
        notes_held(&world, &first.index).as_str()
    );
    assert_eq!(notes[1]["version"], "0.3.0", "{json}");
    assert_eq!(
        notes[1]["notes"]["value"],
        notes_held(&world, &second.index).as_str()
    );
}

#[test]
fn a_move_back_answers_each_versions_notes_once_and_every_promotion_finds_its_own() {
    let world = World::new();
    declare(&world, &stable(), None);
    let [first, second] = two_published(&world);
    let chain = promoted_to(
        &world,
        &stable(),
        &[first.clone(), second.clone(), first.clone()],
    );

    let answer = asked(&world, &ask(&world));
    let stable = &channels_of(&answer)[0];
    assert_eq!(
        stable.notes,
        [held_notes(&world, &first), held_notes(&world, &second)]
    );
    assert_eq!(chain.len(), 3);
    for (_, promotion) in &chain {
        let logged = stable
            .notes
            .iter()
            .find(|l| l.version == promotion.to.version && l.index == promotion.to.index)
            .expect("each promotion's version has its notes");
        assert_eq!(
            logged.notes,
            Ok(notes_held(&world, &promotion.to.index)),
            "{promotion:?}"
        );
    }
}

#[test]
fn an_empty_channel_and_one_whose_chain_is_not_answered_answer_no_notes() {
    let world = World::new();
    let edge = of("lib", "edge");
    declare(&world, &beta(), None);
    declare(&world, &edge, None);
    published(&world);
    let (_, a) = sealed(&world, &edge, "0.1.0", None, None, AT);
    let (_, b) = sealed(&world, &edge, "0.2.0", None, None, AT + 1);
    world.fakes.keeper.fork(&edge, &a, &b);

    let answer = asked(&world, &ask(&world));
    let channels = channels_of(&answer);
    assert_eq!(channels[0].at, Ok(Channel::Empty { read: Read::Source }));
    assert_eq!(channels[0].notes, []);
    assert!(
        matches!(&channels[1].at, Err(Not::Refused(_))),
        "{:?}",
        channels[1].at
    );
    assert_eq!(channels[1].notes, []);
    let json = serde_json::to_value(&answer).unwrap();
    let listed = &json["channels"]["value"]["channels"];
    assert_eq!(listed[0]["notes"], serde_json::json!([]), "{json}");
    assert_eq!(listed[1]["notes"], serde_json::json!([]), "{json}");
}

// Requirement: The changelog's notes are rendered from git on any machine

#[test]
fn with_the_store_unreachable_the_changelog_answers_the_same_notes_and_asks_no_store_or_signer() {
    let world = World::new();
    declare(&world, &stable(), None);
    let [first, second] = two_published(&world);
    promoted_to(&world, &stable(), &[first.clone(), second.clone()]);
    let before = asked(&world, &ask(&world));
    for op in [
        StoreOp::PutBytes,
        StoreOp::PutArtifact,
        StoreOp::Attach,
        StoreOp::ClaimsOf,
        StoreOp::Name,
        StoreOp::Resolve,
        StoreOp::Read,
    ] {
        world.fakes.store.fault(
            op,
            Fault::Unreadable("connection refused".into()),
            Times::UntilHealed,
        );
    }
    let calls = (
        world.fakes.store.calls().len(),
        world.fakes.signer.calls().len(),
    );

    let answer = asked(&world, &ask(&world));
    assert_eq!(
        channels_of(&answer)[0].notes,
        [held_notes(&world, &first), held_notes(&world, &second)]
    );
    assert_eq!(channels_of(&answer), channels_of(&before));
    assert_eq!(
        (
            world.fakes.store.calls().len(),
            world.fakes.signer.calls().len(),
        ),
        calls,
        "status asked the store or the signer"
    );
}

// Requirement: Each version's notes in the changelog are their own answer

#[test]
fn a_tag_the_history_does_not_hold_leaves_that_versions_notes_unreadable_and_the_rest_shown() {
    let world = World::new();
    declare(&world, &stable(), None);
    let [first, second] = two_published(&world);
    let chain = promoted_to(&world, &stable(), &[first.clone(), second.clone()]);
    world.repo.git(&["tag", "-d", "lib-v0.2.0"]);
    world.shared.git(&["tag", "-d", "lib-v0.2.0"]);

    let answer = asked(&world, &ask(&world));
    let stable = &channels_of(&answer)[0];
    assert_eq!(
        stable.at,
        Ok(Channel::At {
            head: chain[1].1.clone(),
            chain: chain.clone(),
            read: Read::Source,
        })
    );
    assert_eq!(
        stable.notes.len(),
        2,
        "every version keeps its place: {:?}",
        stable.notes
    );
    assert_eq!(
        (&stable.notes[0].version, &stable.notes[0].index),
        (&first.version, &first.index)
    );
    let Err(Not::Unreadable(u)) = &stable.notes[0].notes else {
        panic!("the notes of a tag gone answered {:?}", stable.notes[0])
    };
    assert_eq!(u.source, Source::history(LOCAL));
    assert_eq!(u.what, "the notes of lib 0.2.0");
    let said = u.said.as_deref().unwrap();
    assert!(said.contains("no tag lib-v0.2.0"), "{said}");
    assert!(said.contains(world.repo.head().as_str()), "{said}");
    // The second is rendered still, since the last tag the history holds.
    assert_eq!(
        (&stable.notes[1].version, &stable.notes[1].index),
        (&second.version, &second.index)
    );
    assert!(stable.notes[1].notes.is_ok(), "{:?}", stable.notes[1]);
    let json = serde_json::to_value(&answer).unwrap();
    let notes = &json["channels"]["value"]["channels"][0]["notes"];
    assert_eq!(notes[0]["notes"]["answer"], "unreadable", "{json}");
    assert_eq!(notes[1]["notes"]["answer"], "ok", "{json}");
}

#[test]
fn a_commit_the_history_no_longer_holds_leaves_the_notes_unreadable_naming_it() {
    let world = World::new();
    declare(&world, &stable(), None);
    let mend = world.repo.touch("fix: lib mends", &["crates/lib"]);
    let later = world.next("feat: lib grows more");
    let [first, second] = published_again(&world, &later);
    let chain = promoted_to(&world, &stable(), &[first, second.clone()]);
    let hex = mend.as_str();
    std::fs::remove_file(
        world
            .repo
            .path
            .join(".git/objects")
            .join(&hex[..2])
            .join(&hex[2..]),
    )
    .unwrap();

    let answer = asked(&world, &ask(&world));
    let stable = &channels_of(&answer)[0];
    assert_eq!(
        stable.at,
        Ok(Channel::At {
            head: chain[1].1.clone(),
            chain: chain.clone(),
            read: Read::Source,
        })
    );
    let logged = &stable.notes[1];
    assert_eq!(
        (&logged.version, &logged.index),
        (&second.version, &second.index)
    );
    let Err(Not::Unreadable(u)) = &logged.notes else {
        panic!("notes rendered past a commit gone answered {logged:?}")
    };
    assert!(
        format!("{} {}", u.what, u.said.as_deref().unwrap_or_default()).contains(hex),
        "{u:?}"
    );
}

// Requirement (environment-status): Status shows every environment the
// policy declares

/// The signer whose reports count, which the policy lets claim one.
fn deployer() -> timber_seams::NamedSigner {
    timber_seams::NamedSigner {
        id: SignerId::new(DEPLOYER),
        verified_by: format!("{DEPLOYER}'s key"),
        kinds: vec![timber_seams::ClaimKind::Report],
        checks: vec![],
    }
}

/// The policy for a layout lists the deployer, letting it claim a report
/// (116).
fn lets_report(world: &World) {
    world.fakes.policy.require(
        Audience::Layout,
        Required::Layout {
            claims: vec![],
            given: false,
            signers: vec![deployer()],
        },
    );
}

/// `name` declared at `main`'s head, following `channel`, waiting on
/// `approvers` where `waits`, with no target.
fn declares(world: &World, name: &str, channel: &ChannelId, waits: bool, approvers: &[&str]) {
    world.fakes.policy.declares_at(
        &world.history().head(&world.main).unwrap(),
        timber_seams::DeclaredEnvironment {
            name: env(name),
            channel: channel.clone(),
            waits,
            approvers: approvers.iter().map(|a| SignerId::new(*a)).collect(),
            target: None,
        },
    );
}

/// `channel` promoted to each version in turn, each version's index held
/// in the fake store: every promotion with its envelope's digest.
fn held_chain(world: &World, channel: &ChannelId, versions: &[&str]) -> Vec<(Digest, Promotion)> {
    for version in versions {
        held_index(world, &format!("{version}'s index"));
    }
    promoted(world, channel, versions)
}

/// The environments' part of `overview`, answered.
fn environments_of(overview: &Overview) -> &timber_core::Environments {
    match &overview.environments {
        Ok(environments) => environments,
        Err(not) => panic!("the environments answered {not:?}"),
    }
}

/// The environment `name` of `overview`'s environments.
fn one<'o>(overview: &'o Overview, name: &str) -> &'o timber_core::EnvironmentStatus {
    environments_of(overview)
        .environments
        .iter()
        .find(|e| e.environment == env(name))
        .unwrap_or_else(|| panic!("no environment {name} in {:?}", overview.environments))
}

fn counts(converged: usize, behind: usize, waiting: usize, unreadable: usize) -> EnvironmentCounts {
    EnvironmentCounts {
        converged,
        behind,
        waiting,
        unreadable,
    }
}

/// What a head at `version` of the fake chains answers.
fn at(version: &str) -> HeadAt {
    let moved = moved(version);
    HeadAt::At {
        version: moved.version,
        index: moved.index,
    }
}

/// The absent answer `answer` carries, naming what is not there.
fn absent_what<T: std::fmt::Debug>(answer: &timber_seams::Answer<T>) -> String {
    match answer {
        Err(Not::Absent(a)) => a.what.clone(),
        other => panic!("answered {other:?}, not absent"),
    }
}

#[test]
fn two_environments_of_one_component_are_named_with_their_channels_and_whom_they_wait_on() {
    let world = World::new();
    let desk = of("lib", "desk");
    declare(&world, &stable(), None);
    declare(&world, &desk, Some("stable"));
    declares(&world, "srv", &stable(), false, &[]);
    declares(&world, "desk", &desk, true, &["ops"]);
    let answer = asked(&world, &ask(&world));
    let srv = one(&answer, "srv");
    assert_eq!(
        (&srv.component, &srv.channel, srv.waits, &srv.approvers),
        (
            &ComponentName::new("lib"),
            &ChannelName::new("stable"),
            false,
            &vec![]
        )
    );
    assert_eq!(srv.moves_on, None);
    let desk = one(&answer, "desk");
    assert_eq!(
        (&desk.channel, desk.waits, &desk.approvers),
        (&ChannelName::new("desk"), true, &vec![SignerId::new("ops")])
    );
    assert_eq!(
        desk.moves_on,
        Some(timber_core::MovesOn::Asking {
            bound_to: ChannelName::new("stable"),
            at: Ok(HeadAt::Empty),
        })
    );
    let json = serde_json::to_value(&answer).unwrap();
    let environments = &json["environments"];
    assert_eq!(environments["answer"], "ok", "{json}");
    assert_eq!(environments["value"]["unreadable"], 2, "{json}");
    let srv = &environments["value"]["environments"][1];
    assert_eq!(srv["environment"], "srv", "{json}");
    assert!(srv.get("moves_on").is_none(), "{json}");
    let desk = &environments["value"]["environments"][0];
    assert_eq!(desk["moves_on"]["by"], "asking", "{json}");
    assert_eq!(desk["moves_on"]["at"]["value"]["state"], "empty", "{json}");
}

#[test]
fn an_environment_running_its_channels_head_is_converged_and_no_target_is_asked() {
    let mut world = World::new();
    let production = world.fakes.target(&TargetProfile::new("fake"));
    lets_report(&world);
    declare(&world, &stable(), None);
    declares(&world, "srv", &stable(), false, &[]);
    let chain = held_chain(&world, &stable(), &["0.1.0", "0.2.0"]);
    let (first, second) = (&chain[0].1.to.index, &chain[1].1.to.index);
    reported_by(&world, DEPLOYER, ("srv", &stable()), ("0.1.0", first), AT);
    let newest = reported_by(
        &world,
        DEPLOYER,
        ("srv", &stable()),
        ("0.2.0", second),
        AT + 60,
    );
    let signs = || {
        world
            .fakes
            .signer
            .calls()
            .iter()
            .filter(|c| c.op == timber_fakes::SignerOp::Sign)
            .count()
    };
    let signed = signs();

    let answer = asked(&world, &ask(&world));
    let srv = one(&answer, "srv");
    assert_eq!(srv.should, Ok(at("0.2.0")));
    let Ok(Channel::At { head, .. }) = &channels_of(&answer)[0].at else {
        panic!("stable answered {:?}", channels_of(&answer)[0].at)
    };
    assert_eq!(
        srv.should,
        Ok(HeadAt::At {
            version: head.to.version.clone(),
            index: head.to.index.clone()
        })
    );
    assert_eq!(
        srv.runs,
        Ok(timber_core::Runs {
            version: Version::new("0.2.0"),
            index: second.clone(),
            converged_at: "2026-10-31T09:00:00Z".into(),
            at: AT + 60,
            claim: newest.digest(),
            signer: SignerId::new(DEPLOYER),
            read: Read::Source,
        })
    );
    assert_eq!(srv.state, Ok(timber_core::State::Converged));
    assert_eq!(srv.between, None);
    assert_eq!(environments_of(&answer).counts, counts(1, 0, 0, 0));
    assert!(production.calls().is_empty(), "status asked the target");
    assert_eq!(signs(), signed, "status signed");
    let json = serde_json::to_value(&answer).unwrap();
    let srv = &json["environments"]["value"]["environments"][0];
    assert_eq!(
        srv["state"],
        serde_json::json!({ "answer": "ok", "value": "converged" })
    );
    assert_eq!(srv["runs"]["value"]["read"], "source", "{json}");
    assert!(srv.get("between").is_none(), "{json}");
    assert_eq!(json["environments"]["value"]["converged"], 1, "{json}");
}

#[test]
fn an_environment_whose_newest_counted_report_names_another_index_is_behind() {
    let world = World::new();
    lets_report(&world);
    declare(&world, &stable(), None);
    declares(&world, "srv", &stable(), false, &[]);
    let chain = held_chain(&world, &stable(), &["0.1.0", "0.2.0"]);
    reported_by(
        &world,
        DEPLOYER,
        ("srv", &stable()),
        ("0.1.0", &chain[0].1.to.index),
        AT,
    );
    let answer = asked(&world, &ask(&world));
    let srv = one(&answer, "srv");
    assert_eq!(srv.should, Ok(at("0.2.0")));
    assert_eq!(
        srv.runs.as_ref().map(|r| r.version.clone()),
        Ok(Version::new("0.1.0"))
    );
    assert_eq!(srv.state, Ok(timber_core::State::Behind));
    assert_eq!(environments_of(&answer).counts, counts(0, 1, 0, 0));
}

#[test]
fn a_waiting_environment_whose_own_channel_is_behind_the_one_it_is_bound_to_is_waiting() {
    let world = World::new();
    let desk = of("lib", "desk");
    lets_report(&world);
    declare(&world, &stable(), None);
    declare(&world, &desk, Some("stable"));
    declares(&world, "desk", &desk, true, &["ops"]);
    held_chain(&world, &stable(), &["0.1.0", "0.2.0"]);
    let own = held_chain(&world, &desk, &["0.1.0"]);
    reported_by(
        &world,
        DEPLOYER,
        ("desk", &desk),
        ("0.1.0", &own[0].1.to.index),
        AT,
    );
    let answer = asked(&world, &ask(&world));
    let desk_status = one(&answer, "desk");
    assert_eq!(desk_status.should, Ok(at("0.1.0")));
    assert_eq!(desk_status.state, Ok(timber_core::State::Waiting));
    assert_eq!(
        desk_status.moves_on,
        Some(timber_core::MovesOn::Asking {
            bound_to: ChannelName::new("stable"),
            at: Ok(at("0.2.0")),
        })
    );
    assert_eq!(desk_status.approvers, [SignerId::new("ops")]);
    assert_eq!(environments_of(&answer).counts, counts(0, 0, 1, 0));

    // Its own channel moved on, its converge due, it is behind and waits
    // on no one.
    let (_, envelope) = sealed_to(
        &world,
        &desk,
        &moved("0.2.0"),
        Some(&moved("0.1.0")),
        Some(&own[0].0),
        AT + 1,
    );
    world
        .fakes
        .keeper
        .append(&desk, &envelope, Some(&own[0].0))
        .unwrap();
    let answer = asked(&world, &ask(&world));
    assert_eq!(one(&answer, "desk").state, Ok(timber_core::State::Behind));
}

#[test]
fn an_environment_after_another_runs_what_it_should_and_is_never_waiting() {
    let world = World::new();
    let (first, second) = (of("lib", "first"), of("lib", "second"));
    lets_report(&world);
    declare(&world, &stable(), None);
    declare(&world, &first, Some("stable"));
    world.fakes.policy.after(
        &second,
        &[env("first")],
        &[deployer()],
        Required::Channel {
            claims: vec![],
            bound_to: Some(ChannelName::new("first")),
            promoted_by: vec![],
            approvers: vec![],
        },
    );
    declares(&world, "first", &first, true, &["ops"]);
    declares(&world, "second", &second, false, &[]);
    held_chain(&world, &first, &["0.1.0", "0.2.0"]);
    let own = held_chain(&world, &second, &["0.1.0"]);
    reported_by(
        &world,
        DEPLOYER,
        ("second", &second),
        ("0.1.0", &own[0].1.to.index),
        AT,
    );
    let answer = asked(&world, &ask(&world));
    let after = one(&answer, "second");
    assert_eq!(
        after.moves_on,
        Some(timber_core::MovesOn::Reports {
            after: vec![env("first")]
        })
    );
    assert_eq!(after.state, Ok(timber_core::State::Converged));
    let json = serde_json::to_value(after).unwrap();
    assert_eq!(
        json["moves_on"],
        serde_json::json!({ "by": "reports", "after": ["first"] })
    );
}

#[test]
fn an_environment_whose_channel_is_empty_is_unread_saying_so() {
    let world = World::new();
    lets_report(&world);
    declare(&world, &stable(), None);
    declares(&world, "srv", &stable(), false, &[]);
    let before = world.fakes.store.calls().len();
    let answer = asked(&world, &ask(&world));
    let srv = one(&answer, "srv");
    assert_eq!(srv.should, Ok(HeadAt::Empty));
    let what = absent_what(&srv.runs);
    assert!(what.contains("environment srv of lib"), "{what}");
    assert!(what.contains("its channel lib/stable is empty"), "{what}");
    assert_eq!(srv.state.as_ref().err(), srv.runs.as_ref().err());
    assert!(matches!(&srv.state, Err(Not::Absent(_))), "{:?}", srv.state);
    assert_eq!(environments_of(&answer).counts, counts(0, 0, 0, 1));
    assert_eq!(
        world.fakes.store.calls().len(),
        before,
        "an empty chain's reports were fetched"
    );
}

#[test]
fn an_environment_no_counted_report_names_is_unread_naming_it() {
    let world = World::new();
    lets_report(&world);
    declare(&world, &stable(), None);
    declares(&world, "srv", &stable(), false, &[]);
    let chain = held_chain(&world, &stable(), &["0.1.0"]);
    let theirs = reported_by(
        &world,
        "desk",
        ("srv", &stable()),
        ("0.1.0", &chain[0].1.to.index),
        AT,
    );
    let answer = asked(&world, &ask(&world));
    let srv = one(&answer, "srv");
    let what = absent_what(&srv.runs);
    assert!(
        what.contains("a counted report of environment srv of lib"),
        "{what}"
    );
    assert!(!what.contains("lets no signer"), "{what}");
    assert_eq!(srv.state.as_ref().err(), srv.runs.as_ref().err());
    assert_eq!(environments_of(&answer).counts, counts(0, 0, 0, 1));
    let copy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(copy_of(&world).path(&env("srv"), &stable())).unwrap(),
    )
    .unwrap();
    assert_eq!(
        copy["reports"][0]["envelope"],
        serde_json::json!(theirs),
        "the copy does not keep the report uncounted"
    );
}

#[test]
fn a_policy_letting_no_signer_report_leaves_every_environment_unread_saying_so() {
    let world = World::new();
    declare(&world, &stable(), None);
    declares(&world, "srv", &stable(), false, &[]);
    declares(&world, "eu", &stable(), false, &[]);
    let chain = held_chain(&world, &stable(), &["0.1.0"]);
    reported_by(
        &world,
        DEPLOYER,
        ("srv", &stable()),
        ("0.1.0", &chain[0].1.to.index),
        AT,
    );
    let answer = asked(&world, &ask(&world));
    for name in ["srv", "eu"] {
        let what = absent_what(&one(&answer, name).runs);
        assert!(
            what.contains(&format!("environment {name} of lib")),
            "{what}"
        );
        assert!(
            what.contains("the policy lets no signer claim a report"),
            "{what}"
        );
    }
    assert_eq!(environments_of(&answer).counts, counts(0, 0, 0, 2));
}

#[test]
fn one_component_asked_shows_its_environments_alone_and_counts_them() {
    let world = World::new();
    let site = of("web", "stable");
    lets_report(&world);
    declare(&world, &stable(), None);
    declare(&world, &site, None);
    declares(&world, "srv", &stable(), false, &[]);
    declares(&world, "edge", &site, false, &[]);
    let answer = asked(
        &world,
        &OverviewAsk {
            component: Some(ComponentName::new("web")),
            ..ask(&world)
        },
    );
    let names: Vec<_> = environments_of(&answer)
        .environments
        .iter()
        .map(|e| e.environment.clone())
        .collect();
    assert_eq!(names, [env("edge")]);
    assert_eq!(environments_of(&answer).counts, counts(0, 0, 0, 1));
    let everything = asked(&world, &ask(&world));
    assert_eq!(environments_of(&everything).environments.len(), 2);
}

#[test]
fn a_policy_declaring_no_environment_answers_an_empty_list_with_four_counts_of_none() {
    let world = World::new();
    declare(&world, &stable(), None);
    promoted(&world, &stable(), &["0.1.0"]);
    let answer = asked(&world, &ask(&world));
    assert_eq!(
        answer.environments,
        Ok(timber_core::Environments {
            counts: counts(0, 0, 0, 0),
            environments: vec![],
        })
    );
    let json = serde_json::to_value(&answer).unwrap();
    assert_eq!(
        json["environments"],
        serde_json::json!({
            "answer": "ok",
            "value": {
                "converged": 0, "behind": 0, "waiting": 0, "unreadable": 0,
                "environments": [],
            },
        })
    );
}

#[test]
fn a_chain_answered_as_last_read_has_its_environments_reports_read_across_it_from_the_store() {
    let world = World::new();
    lets_report(&world);
    declare(&world, &stable(), None);
    declares(&world, "srv", &stable(), false, &[]);
    let chain = held_chain(&world, &stable(), &["0.1.0", "0.2.0"]);
    asked(&world, &ask(&world));
    world.fakes.keeper.unreachable("fatal: unable to access");
    reported_by(
        &world,
        DEPLOYER,
        ("srv", &stable()),
        ("0.2.0", &chain[1].1.to.index),
        AT,
    );
    let answer = asked(&world, &ask(&world));
    assert!(
        matches!(
            &channels_of(&answer)[0].at,
            Ok(Channel::At {
                read: Read::Kept { .. },
                ..
            })
        ),
        "{:?}",
        channels_of(&answer)[0].at
    );
    let srv = one(&answer, "srv");
    assert_eq!(srv.should, Ok(at("0.2.0")));
    let runs = srv.runs.as_ref().expect("srv's report read");
    assert_eq!(
        (&runs.version, &runs.read),
        (&Version::new("0.2.0"), &Read::Source)
    );
    assert_eq!(srv.state, Ok(timber_core::State::Converged));
}

#[test]
fn a_chain_that_cannot_be_read_is_what_the_environment_should_run_and_it_is_unread() {
    let world = World::new();
    lets_report(&world);
    declare(&world, &stable(), None);
    declares(&world, "srv", &stable(), false, &[]);
    held_chain(&world, &stable(), &["0.1.0"]);
    world.fakes.keeper.fault(
        KeeperOp::Chain,
        Fault::Unreadable("fatal: unable to access".into()),
        Times::Once,
    );
    let answer = asked(&world, &ask(&world));
    let srv = one(&answer, "srv");
    assert_eq!(
        srv.should.as_ref().err(),
        channels_of(&answer)[0].at.as_ref().err()
    );
    let Err(Not::Unreadable(u)) = &srv.state else {
        panic!("srv's state answered {:?}", srv.state)
    };
    assert_eq!(u.said.as_deref(), Some("fatal: unable to access"));
    assert_eq!(environments_of(&answer).counts, counts(0, 0, 0, 1));
}

#[test]
fn four_environments_one_of_each_are_counted_one_one_one_and_one() {
    let world = World::new();
    let desk = of("lib", "desk");
    let beta = beta();
    lets_report(&world);
    declare(&world, &beta, None);
    declare(&world, &stable(), None);
    declare(&world, &desk, Some("stable"));
    declares(&world, "srv", &stable(), false, &[]);
    declares(&world, "eu", &stable(), false, &[]);
    declares(&world, "desk", &desk, true, &["ops"]);
    declares(&world, "lab", &beta, false, &[]);
    let chain = held_chain(&world, &stable(), &["0.1.0", "0.2.0"]);
    let own = held_chain(&world, &desk, &["0.1.0"]);
    let (first, second) = (&chain[0].1.to.index, &chain[1].1.to.index);
    reported_by(&world, DEPLOYER, ("srv", &stable()), ("0.2.0", second), AT);
    reported_by(&world, DEPLOYER, ("eu", &stable()), ("0.1.0", first), AT);
    reported_by(
        &world,
        DEPLOYER,
        ("desk", &desk),
        ("0.1.0", &own[0].1.to.index),
        AT,
    );
    let answer = asked(&world, &ask(&world));
    assert_eq!(environments_of(&answer).counts, counts(1, 1, 1, 1));
    let json = serde_json::to_value(&answer).unwrap();
    let value = &json["environments"]["value"];
    for (key, n) in [
        ("converged", 1),
        ("behind", 1),
        ("waiting", 1),
        ("unreadable", 1),
    ] {
        assert_eq!(value[key], n, "{json}");
    }
}

// Requirement (environment-status): The versions between what an
// environment runs and what it should

/// `lib` tagged each of `versions` in turn, the first at the head and
/// each after at a commit one feature on, landed on `main`: each one's
/// revision.
fn released(world: &World, versions: &[&str]) -> Vec<Revision> {
    versions
        .iter()
        .enumerate()
        .map(|(i, version)| {
            let revision = match i {
                0 => world.head.clone(),
                _ => world.next(&format!("feat: lib grows to {version}")),
            };
            world.repo.tag(&format!("lib-v{version}"), &revision);
            revision
        })
        .collect()
}

/// `srv` declared at `main`'s head, following `stable`, which is
/// promoted to `chain` in turn, and reporting it runs `runs`, by the
/// deployer the policy lets report: status's answer.
fn behind(world: &World, chain: &[&str], runs: &str) -> Overview {
    lets_report(world);
    declare(world, &stable(), None);
    declares(world, "srv", &stable(), false, &[]);
    held_chain(world, &stable(), chain);
    let index = held_index(world, &format!("{runs}'s index"));
    reported_by(world, DEPLOYER, ("srv", &stable()), (runs, &index), AT);
    asked(world, &ask(world))
}

/// `srv`'s versions between, answered.
fn between_of(overview: &Overview) -> &timber_core::Between {
    let srv = one(overview, "srv");
    assert_eq!(srv.state, Ok(timber_core::State::Behind));
    match &srv.between {
        Some(Ok(between)) => between,
        other => panic!("srv's between answered {other:?}"),
    }
}

/// The versions and revisions `between` names, in its order.
fn named(between: &timber_core::Between) -> Vec<(Version, Revision)> {
    between
        .versions
        .iter()
        .map(|v| (v.version.clone(), v.revision.clone()))
        .collect()
}

#[test]
fn two_versions_ahead_are_named_with_their_tags_revisions_and_notes_as_the_changelogs() {
    let world = World::new();
    let revisions = released(&world, &["0.2.0", "0.3.0", "0.4.0"]);
    let stores = world.fakes.store.calls().len();
    let answer = behind(&world, &["0.2.0", "0.3.0", "0.4.0"], "0.2.0");
    let between = between_of(&answer);
    assert_eq!(between.direction, timber_core::Direction::Ahead);
    assert_eq!(
        named(between),
        [
            (Version::new("0.3.0"), revisions[1].clone()),
            (Version::new("0.4.0"), revisions[2].clone()),
        ]
    );
    let changelog = &channels_of(&answer)[0].notes;
    for version in &between.versions {
        let logged = changelog
            .iter()
            .find(|l| l.version == version.version)
            .expect("the changelog holds the version");
        let notes = version.notes.as_ref().expect("the notes rendered");
        assert_eq!(
            Ok(notes.as_bytes()),
            logged.notes.as_ref().map(|n| n.as_bytes())
        );
        assert!(notes.contains(version.version.as_str()), "{notes}");
    }
    assert!(
        world.fakes.store.calls()[stores..].iter().all(|c| matches!(
            c.op,
            StoreOp::ClaimsOf | StoreOp::PutBytes | StoreOp::Attach
        )),
        "status asked the store for the notes"
    );
    let json = serde_json::to_value(&answer).unwrap();
    let srv = &json["environments"]["value"]["environments"][0];
    assert_eq!(srv["between"]["answer"], "ok", "{json}");
    assert_eq!(srv["between"]["value"]["direction"], "ahead", "{json}");
    assert_eq!(
        srv["between"]["value"]["versions"][1]["version"], "0.4.0",
        "{json}"
    );
    assert_eq!(
        srv["between"]["value"]["versions"][1]["notes"]["answer"], "ok",
        "{json}"
    );
}

#[test]
fn a_channel_moved_back_names_the_versions_whose_changes_the_environment_leaves() {
    let world = World::new();
    released(&world, &["0.2.0", "0.3.0", "0.4.0"]);
    let answer = behind(&world, &["0.2.0", "0.3.0", "0.4.0", "0.2.0"], "0.4.0");
    let between = between_of(&answer);
    assert_eq!(between.direction, timber_core::Direction::Back);
    let versions: Vec<Version> = named(between).into_iter().map(|(v, _)| v).collect();
    assert_eq!(versions, [Version::new("0.3.0"), Version::new("0.4.0")]);
}

#[test]
fn a_version_the_channel_skipped_is_between_with_its_notes() {
    let world = World::new();
    released(&world, &["0.2.0", "0.3.0", "0.4.0"]);
    let answer = behind(&world, &["0.2.0", "0.4.0"], "0.2.0");
    let between = between_of(&answer);
    let versions: Vec<Version> = named(between).into_iter().map(|(v, _)| v).collect();
    assert_eq!(versions, [Version::new("0.3.0"), Version::new("0.4.0")]);
    let skipped = between.versions[0].notes.as_ref().expect("0.3.0's notes");
    assert!(skipped.contains("lib grows to 0.3.0"), "{skipped}");
    assert!(
        channels_of(&answer)[0]
            .notes
            .iter()
            .all(|l| l.version != Version::new("0.3.0")),
        "the changelog holds 0.3.0, which the channel skipped"
    );
}

#[test]
fn a_prerelease_between_is_left_out_and_a_prerelease_at_the_higher_end_is_named() {
    let world = World::new();
    released(&world, &["0.2.0", "0.3.0", "0.4.0-rc.1", "0.4.0"]);
    let answer = behind(&world, &["0.3.0", "0.4.0"], "0.3.0");
    let versions: Vec<Version> = named(between_of(&answer))
        .into_iter()
        .map(|(v, _)| v)
        .collect();
    assert_eq!(versions, [Version::new("0.4.0")]);

    let world = World::new();
    released(&world, &["0.2.0", "0.3.0", "0.4.0-rc.1", "0.4.0"]);
    let answer = behind(&world, &["0.3.0", "0.4.0-rc.1"], "0.3.0");
    let versions: Vec<Version> = named(between_of(&answer))
        .into_iter()
        .map(|(v, _)| v)
        .collect();
    assert_eq!(versions, [Version::new("0.4.0-rc.1")]);
}

/// One version between whose notes cannot be rendered: 0.3.0 tagged at a
/// revision whose template the rule's tool cannot read.
#[test]
fn a_version_whose_notes_cannot_be_rendered_is_its_own_answer_and_the_rest_shown() {
    let world = World::new();
    world.repo.tag("lib-v0.2.0", &world.head);
    let three = world.repo.commit(
        "feat: lib grows to 0.3.0",
        &[
            ("crates/lib/touched", "0.3.0"),
            ("cliff.toml", "[changelog\nbody = "),
        ],
    );
    world.fakes.landing.land(&world.main, &three);
    world.repo.tag("lib-v0.3.0", &three);
    let four = world.repo.commit(
        "feat: lib grows to 0.4.0",
        &[
            ("crates/lib/touched", "0.4.0"),
            ("cliff.toml", common::CLIFF_TOML),
        ],
    );
    world.fakes.landing.land(&world.main, &four);
    world.repo.tag("lib-v0.4.0", &four);
    let answer = behind(&world, &["0.2.0", "0.4.0"], "0.2.0");
    let between = between_of(&answer);
    assert_eq!(
        named(between),
        [
            (Version::new("0.3.0"), three.clone()),
            (Version::new("0.4.0"), four.clone()),
        ]
    );
    let Err(Not::Unreadable(u)) = &between.versions[0].notes else {
        panic!("0.3.0's notes answered {:?}", between.versions[0].notes)
    };
    assert!(u.what.contains("cliff.toml"), "{u:?}");
    assert!(u.said.is_some(), "{u:?}");
    let shown = between.versions[1].notes.as_ref().expect("0.4.0's notes");
    assert!(shown.contains("lib grows to 0.4.0"), "{shown}");
    let srv = one(&answer, "srv");
    assert!(srv.should.is_ok() && srv.runs.is_ok(), "{srv:?}");
}

/// A commit the history lacks, between the two ends: the tags the head
/// reaches cannot be read past it, so the versions between are unreadable
/// naming them with git's words, never empty, and the rest is shown.
#[test]
fn a_commit_the_history_lacks_makes_the_versions_between_unreadable_naming_it() {
    let world = World::new();
    world.repo.tag("lib-v0.2.0", &world.head);
    let mend = world.repo.touch("fix: lib mends", &["crates/lib"]);
    let three = world.next("feat: lib grows to 0.3.0");
    world.repo.tag("lib-v0.3.0", &three);
    let four = world.next("feat: lib grows to 0.4.0");
    world.repo.tag("lib-v0.4.0", &four);
    let hex = mend.as_str();
    fs::remove_file(
        world
            .repo
            .path
            .join(".git/objects")
            .join(&hex[..2])
            .join(&hex[2..]),
    )
    .unwrap();
    let answer = behind(&world, &["0.2.0", "0.4.0"], "0.2.0");
    let srv = one(&answer, "srv");
    let Some(Err(Not::Unreadable(u))) = &srv.between else {
        panic!("srv's between answered {:?}", srv.between)
    };
    assert_eq!(u.source, Source::history(LOCAL));
    assert!(u.what.contains("lib-v"), "{u:?}");
    assert!(u.said.as_deref().unwrap_or_default().contains(hex), "{u:?}");
    assert_eq!(srv.state, Ok(timber_core::State::Behind));
    assert!(srv.should.is_ok() && srv.runs.is_ok(), "{srv:?}");
}

#[test]
fn a_version_that_is_no_semantic_version_makes_the_versions_between_unreadable_naming_it() {
    let world = World::new();
    released(&world, &["0.2.0"]);
    let answer = behind(&world, &["0.2.0", "latest"], "0.2.0");
    let srv = one(&answer, "srv");
    assert_eq!(srv.state, Ok(timber_core::State::Behind));
    let Some(Err(Not::Unreadable(u))) = &srv.between else {
        panic!("srv's between answered {:?}", srv.between)
    };
    assert!(u.what.contains("between 0.2.0 and latest"), "{u:?}");
    assert!(
        u.said
            .as_deref()
            .unwrap_or_default()
            .contains("latest is no semantic version"),
        "{u:?}"
    );
}

#[test]
fn an_environment_behind_on_its_version_at_another_index_names_no_version_between() {
    let world = World::new();
    released(&world, &["0.2.0"]);
    lets_report(&world);
    declare(&world, &stable(), None);
    declares(&world, "srv", &stable(), false, &[]);
    let first = Moved {
        version: Version::new("0.2.0"),
        index: held_index(&world, "0.2.0 packed first"),
    };
    held_index(&world, "0.2.0's index");
    promoted_to(&world, &stable(), &[first.clone(), moved("0.2.0")]);
    reported_by(
        &world,
        DEPLOYER,
        ("srv", &stable()),
        ("0.2.0", &first.index),
        AT,
    );
    let answer = asked(&world, &ask(&world));
    assert_eq!(between_of(&answer).versions, []);
}

// Requirement (channel-status): Asking reaches no signer or target and
// writes nothing but the host's copy

#[test]
fn status_with_environments_asks_the_store_only_their_claims_and_writes_only_the_copy() {
    let mut world = World::new();
    let production = world.fakes.target(&TargetProfile::new("fake"));
    lets_report(&world);
    declare(&world, &stable(), None);
    declares(&world, "srv", &stable(), false, &[]);
    published(&world);
    let chain = held_chain(&world, &stable(), &["0.1.0", "0.2.0"]);
    reported_by(
        &world,
        DEPLOYER,
        ("srv", &stable()),
        ("0.2.0", &chain[1].1.to.index),
        AT,
    );
    let kept = world.kept();
    let record = files(kept.dir());
    let common = world.history().common_dir().join("timber");
    let outside = || -> Vec<_> {
        files(&common)
            .into_iter()
            .filter(|(p, _, _)| !p.starts_with(copy_of(&world).dir()))
            .collect()
    };
    let (untouched, stores, appends, signs) = (
        outside(),
        world.fakes.store.calls().len(),
        world.fakes.keeper.calls().len(),
        world.fakes.signer.calls().len(),
    );
    let answer = asked(&world, &ask(&world));
    assert_eq!(one(&answer, "srv").state, Ok(timber_core::State::Converged));
    let asked_of_store: Vec<StoreOp> = world.fakes.store.calls()[stores..]
        .iter()
        .map(|c| c.op)
        .collect();
    assert_eq!(asked_of_store, [StoreOp::ClaimsOf, StoreOp::ClaimsOf]);
    assert!(
        world.fakes.keeper.calls()[appends..]
            .iter()
            .all(|c| c.op != KeeperOp::Append),
        "status appended to the ledger"
    );
    assert!(production.calls().is_empty(), "status asked the target");
    assert!(
        world.fakes.signer.calls()[signs..]
            .iter()
            .all(|c| c.op == timber_fakes::SignerOp::Verify),
        "status signed"
    );
    assert_eq!(files(kept.dir()), record, "status wrote to the record");
    assert_eq!(outside(), untouched, "status wrote outside the copy");
    assert!(copy_of(&world).path(&env("srv"), &stable()).is_file());
}

// Requirement (channel-status): Everything kept deleted answers the same

#[test]
fn the_reports_copy_deleted_answers_the_same_environments_and_is_written_again() {
    let world = World::new();
    lets_report(&world);
    declare(&world, &stable(), None);
    declares(&world, "srv", &stable(), false, &[]);
    let chain = held_chain(&world, &stable(), &["0.1.0", "0.2.0"]);
    reported_by(
        &world,
        DEPLOYER,
        ("srv", &stable()),
        ("0.1.0", &chain[0].1.to.index),
        AT,
    );
    let before = asked(&world, &ask(&world));
    let path = copy_of(&world).path(&env("srv"), &stable());
    let written = fs::read(&path).unwrap();
    fs::remove_dir_all(copy_of(&world).dir()).unwrap();
    let again = asked(&world, &ask(&world));
    assert_eq!(again.environments, before.environments);
    assert_eq!(fs::read(&path).unwrap(), written);
}

// Requirement: A repository's row is made from its status

fn repository() -> RepositoryName {
    RepositoryName::from_remote("https://example.com/o/r.git").unwrap()
}

#[test]
fn a_row_names_every_channels_head_and_the_environments_counts() {
    let world = World::new();
    lets_report(&world);
    declare(&world, &beta(), None);
    declare(&world, &stable(), Some("beta"));
    declares(&world, "srv", &stable(), false, &[]);
    declares(&world, "eu", &stable(), false, &[]);
    let chain = held_chain(&world, &stable(), &["0.1.0", "0.2.0"]);
    let (first, second) = (&chain[0].1.to.index, &chain[1].1.to.index);
    reported_by(&world, DEPLOYER, ("srv", &stable()), ("0.2.0", second), AT);
    reported_by(&world, DEPLOYER, ("eu", &stable()), ("0.1.0", first), AT);
    let answer = asked(&world, &ask(&world));
    let row = row(&repository(), &answer);
    assert_eq!(row.repository, repository());
    assert_eq!(
        row.heads,
        Ok(vec![
            Head {
                component: ComponentName::new("lib"),
                channel: ChannelName::new("beta"),
                at: Ok(HeadAt::Empty),
            },
            Head {
                component: ComponentName::new("lib"),
                channel: ChannelName::new("stable"),
                at: Ok(HeadAt::At {
                    version: Version::new("0.2.0"),
                    index: chain[1].1.to.index.clone(),
                }),
            },
        ])
    );
    assert_eq!(row.environments, Ok(counts(1, 1, 0, 0)));
    assert_eq!(
        row.environments.as_ref().ok(),
        answer.environments.as_ref().ok().map(|e| &e.counts)
    );
    let json = serde_json::to_value(&row).unwrap();
    assert_eq!(json["repository"]["owner"], "o", "{json}");
    assert_eq!(json["heads"]["value"][1]["at"]["value"]["state"], "at");
    assert_eq!(
        json["environments"],
        serde_json::json!({
            "answer": "ok",
            "value": { "converged": 1, "behind": 1, "waiting": 0, "unreadable": 0 },
        })
    );

    // The environments' part's own answer where it gives no counts.
    world.fakes.policy.fault(
        timber_fakes::PolicyOp::Environments,
        Fault::Unreadable("policy file: expected `]`".into()),
        Times::Once,
    );
    let answer = asked(&world, &ask(&world));
    let row = timber_core::row(&repository(), &answer);
    let Err(Not::Unreadable(u)) = &row.environments else {
        panic!("the row's environments answered {:?}", row.environments)
    };
    assert_eq!(u.said.as_deref(), Some("policy file: expected `]`"));
    assert_eq!(
        row.environments.as_ref().err(),
        answer.environments.as_ref().err()
    );
}

#[test]
fn a_row_whose_channels_cannot_be_read_carries_that_answer() {
    let world = World::new();
    declare(&world, &stable(), None);
    world.fakes.policy.fault(
        timber_fakes::PolicyOp::Channels,
        Fault::Unreadable("policy file: expected `]`".into()),
        Times::Once,
    );
    let answer = asked(&world, &ask(&world));
    let row = row(&repository(), &answer);
    let Err(Not::Unreadable(u)) = &row.heads else {
        panic!("the heads answered {:?}", row.heads)
    };
    assert_eq!(u.said.as_deref(), Some("policy file: expected `]`"));
    assert_eq!(Err(Not::Unreadable(u.clone())), answer.channels);
}

#[test]
fn one_channels_unreadable_chain_is_that_heads_own_answer() {
    let world = World::new();
    declare(&world, &beta(), None);
    declare(&world, &stable(), None);
    promoted(&world, &beta(), &["0.1.0"]);
    promoted(&world, &stable(), &["0.1.0"]);
    world.fakes.keeper.fault(
        KeeperOp::Chain,
        Fault::Unreadable("fatal: unable to access".into()),
        Times::Once,
    );
    let row = row(&repository(), &asked(&world, &ask(&world)));
    let heads = row.heads.expect("the heads answered");
    let Err(Not::Unreadable(u)) = &heads[0].at else {
        panic!("beta answered {:?}", heads[0].at)
    };
    assert_eq!(u.said.as_deref(), Some("fatal: unable to access"));
    assert!(
        matches!(&heads[1].at, Ok(HeadAt::At { version, .. }) if version.as_str() == "0.1.0"),
        "{:?}",
        heads[1].at
    );
}

// Requirement (serving-process): The repository is in the path, by its
// remote's owner and name

#[test]
fn a_remotes_address_is_read_as_written_with_no_rewrite_applied() {
    let world = World::new();
    let url = "https://example.com/agentplot/timber.git";
    world.repo.git(&["remote", "add", "upstream", url]);
    world.repo.git(&[
        "config",
        "url./srv/elsewhere/.insteadOf",
        "https://example.com/",
    ]);
    let history = world.history();
    assert_eq!(history.remote_url("upstream"), Ok(Some(url.to_string())));
    assert_eq!(history.remote_url("nowhere"), Ok(None));
    assert_eq!(
        timber_core::RepositoryName::from_remote(url).map(|r| r.to_string()),
        Some("agentplot/timber".to_string())
    );
}

// Requirement (environment-status): The reports are fetched into the
// host's copy

/// A store's own words, carried verbatim (59).
const WORDS: &str = "referrers of the index: 503 Service Unavailable\n\t\u{fc} ";

/// The signer whose reports the policy lets count.
const DEPLOYER: &str = "deployer";

fn env(name: &str) -> timber_seams::EnvironmentName {
    timber_seams::EnvironmentName::new(name)
}

/// A digest the fake store holds, which a chain may have moved to.
fn held_index(world: &World, name: &str) -> Digest {
    world.fakes.store.put_bytes(name.as_bytes()).unwrap().digest
}

/// A claim of `kind` with `body` about `index`, signed by `signer` as
/// the fake signer signs and attached to the index: its envelope.
fn attach_claim(
    world: &World,
    index: &Digest,
    signer: &str,
    kind: timber_seams::ClaimKind,
    body: serde_json::Value,
) -> Envelope {
    let theirs = FakeSigner::new();
    theirs.hold(&SignerId::new(signer));
    let claim = timber_seams::Claim {
        subject: index.clone(),
        kind,
        revision: world.head.clone(),
        build: None,
        body,
    };
    let envelope = theirs.sign(&claim, &SignerId::new(signer)).unwrap();
    world.fakes.store.attach(index, &envelope).unwrap();
    envelope
}

/// A report of `environment` of `lib`, which follows `channel`, running
/// `version` at `index`, from an apply at `at`, by `signer`, attached to
/// the index: its envelope.
fn reported_by(
    world: &World,
    signer: &str,
    (environment, channel): (&str, &ChannelId),
    (version, index): (&str, &Digest),
    at: u64,
) -> Envelope {
    let report = timber_core::Reported {
        environment: env(environment),
        component: channel.component_name(),
        channel: channel.name.clone(),
        version: Version::new(version),
        index: index.clone(),
        runs: index.clone(),
        converged_at: "2026-10-31T09:00:00Z".into(),
        at,
    };
    attach_claim(
        world,
        index,
        signer,
        timber_seams::ClaimKind::Report,
        serde_json::to_value(&report).unwrap(),
    )
}

fn copy_of(world: &World) -> timber_core::ReportsCopy {
    timber_core::ReportsCopy::at(&world.history())
}

/// The reports of `environment`, which follows `channel`, fetched over
/// `indexes` at `at` in a fetch of its own.
fn fetched(
    world: &World,
    environment: &str,
    channel: &ChannelId,
    indexes: &[Digest],
    at: u64,
) -> timber_seams::Answer<timber_core::Fetched> {
    let copy = copy_of(world);
    timber_core::Fetch::new(&world.fakes.store, &copy, at).reports(
        &env(environment),
        channel,
        indexes,
    )
}

/// The envelopes of `fetched`'s reports, in the order answered.
fn envelopes(fetched: &timber_core::Fetched) -> Vec<Envelope> {
    fetched
        .reports
        .attached
        .iter()
        .map(|r| r.envelope.clone())
        .collect()
}

#[test]
fn the_reports_are_kept_with_their_indexes_and_the_moment_and_nothing_else() {
    let world = World::new();
    let (first, second) = (held_index(&world, "0.1.0"), held_index(&world, "0.2.0"));
    let older = reported_by(&world, DEPLOYER, ("srv", &stable()), ("0.1.0", &first), AT);
    attach_claim(
        &world,
        &first,
        "gate",
        timber_seams::ClaimKind::Acceptance,
        serde_json::json!({}),
    );
    reported_by(&world, DEPLOYER, ("desk", &stable()), ("0.1.0", &first), AT);
    let newer = reported_by(
        &world,
        "desk",
        ("srv", &stable()),
        ("0.2.0", &second),
        AT + 60,
    );

    let answer = fetched(
        &world,
        "srv",
        &stable(),
        &[first.clone(), second.clone()],
        AT + 100,
    )
    .expect("the reports fetched");
    assert_eq!(answer.read, Read::Source);
    assert_eq!(envelopes(&answer), [older.clone(), newer.clone()]);
    assert_eq!(answer.reports.environment, env("srv"));

    let path = copy_of(&world).dir().join("lib").join("srv.json");
    assert_eq!(copy_of(&world).path(&env("srv"), &stable()), path);
    let text = fs::read_to_string(&path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "at": AT + 100,
            "reports": [
                { "index": first, "envelope": older },
                { "index": second, "envelope": newer },
            ],
        })
    );
    let beside: Vec<_> = files(copy_of(&world).dir())
        .into_iter()
        .map(|(p, _, _)| p)
        .collect();
    assert_eq!(beside, [path], "the copy left a file beside");
}

#[test]
fn a_repository_declaring_no_component_keeps_its_reports_by_environment_alone() {
    let world = World::with_policy(world::UNDECLARED);
    let undeclared = ChannelId::new(None, ChannelName::new("stable"));
    let index = held_index(&world, "0.1.0");
    reported_by(
        &world,
        DEPLOYER,
        ("srv", &undeclared),
        ("0.1.0", &index),
        AT,
    );
    let answer = fetched(&world, "srv", &undeclared, &[index], AT).unwrap();
    assert_eq!(answer.reports.attached.len(), 1);
    assert!(copy_of(&world).dir().join("srv.json").is_file());
}

#[test]
fn with_the_store_unreadable_the_reports_are_answered_from_the_copy_saying_when() {
    let world = World::new();
    let (first, second) = (held_index(&world, "0.1.0"), held_index(&world, "0.2.0"));
    reported_by(&world, DEPLOYER, ("srv", &stable()), ("0.1.0", &first), AT);
    let indexes = [first.clone(), second.clone()];
    let before = fetched(&world, "srv", &stable(), &indexes, AT + 100).unwrap();
    let path = copy_of(&world).path(&env("srv"), &stable());
    let kept = fs::read(&path).unwrap();

    reported_by(
        &world,
        DEPLOYER,
        ("srv", &stable()),
        ("0.2.0", &second),
        AT + 200,
    );
    world.fakes.store.fault(
        StoreOp::ClaimsOf,
        Fault::Unreadable(WORDS.into()),
        Times::Once,
    );
    let after = fetched(&world, "srv", &stable(), &indexes, AT + 300).unwrap();
    assert_eq!(
        after.read,
        Read::Kept {
            at: AT + 100,
            said: WORDS.into()
        }
    );
    assert_eq!(after.reports, before.reports);
    assert_eq!(fs::read(&path).unwrap(), kept, "the copy was written");
}

#[test]
fn with_no_copy_and_the_store_unreadable_the_reports_are_unreadable_naming_them() {
    let world = World::new();
    let index = held_index(&world, "0.1.0");
    reported_by(&world, DEPLOYER, ("srv", &stable()), ("0.1.0", &index), AT);
    world.fakes.store.fault(
        StoreOp::ClaimsOf,
        Fault::Unreadable(WORDS.into()),
        Times::Once,
    );
    match fetched(&world, "srv", &stable(), &[index], AT) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source.seam, Seam::Store);
            assert!(
                u.what.contains("the reports of environment srv of lib"),
                "{}",
                u.what
            );
            assert_eq!(u.said.as_deref(), Some(WORDS));
        }
        other => panic!("answered {other:?}, not unreadable"),
    }
    assert!(
        files(copy_of(&world).dir()).is_empty(),
        "a copy was written"
    );
}

#[test]
fn an_index_the_store_holds_none_of_is_unreadable_naming_it_and_the_copy_is_left() {
    let world = World::new();
    let index = held_index(&world, "0.1.0");
    reported_by(&world, DEPLOYER, ("srv", &stable()), ("0.1.0", &index), AT);
    fetched(&world, "srv", &stable(), std::slice::from_ref(&index), AT).unwrap();
    let kept = files(copy_of(&world).dir());

    let gone = Digest::of(b"an index the store never held");
    match fetched(&world, "srv", &stable(), &[index, gone.clone()], AT + 100) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source.seam, Seam::Store);
            assert!(u.what.contains(gone.as_str()), "{}", u.what);
            assert!(u.what.contains("lib/stable"), "{}", u.what);
        }
        other => panic!("answered {other:?}, not unreadable"),
    }
    assert_eq!(files(copy_of(&world).dir()), kept, "the copy was written");
}

#[test]
fn a_copy_that_cannot_be_read_is_unreadable_naming_its_file() {
    let world = World::new();
    let index = held_index(&world, "0.1.0");
    let path = copy_of(&world).path(&env("srv"), &stable());
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "{ \"at\": ").unwrap();
    world.fakes.store.fault(
        StoreOp::ClaimsOf,
        Fault::Unreadable(WORDS.into()),
        Times::Once,
    );
    match fetched(&world, "srv", &stable(), &[index], AT) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, Source::host(&path));
            assert_eq!(u.what, path.display().to_string());
            assert!(u.said.is_some(), "{u:?}");
        }
        other => panic!("answered {other:?}, not unreadable"),
    }
}

#[test]
fn a_copy_that_cannot_be_written_is_unreadable_naming_its_file() {
    let world = World::new();
    let index = held_index(&world, "0.1.0");
    reported_by(&world, DEPLOYER, ("srv", &stable()), ("0.1.0", &index), AT);
    let path = copy_of(&world).path(&env("srv"), &stable());
    fs::create_dir_all(path.join("in the way")).unwrap();
    match fetched(&world, "srv", &stable(), &[index], AT) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, Source::host(&path));
            assert_eq!(u.what, path.display().to_string());
            assert!(u.said.is_some(), "{u:?}");
        }
        other => panic!("answered {other:?}, not unreadable"),
    }
    let left: Vec<_> = files(path.parent().unwrap())
        .into_iter()
        .filter(|(p, _, _)| !p.starts_with(&path))
        .collect();
    assert!(left.is_empty(), "a file was left beside: {left:?}");
}

#[test]
fn one_index_is_asked_its_claims_once_for_two_environments() {
    let world = World::new();
    let desk = of("lib", "desk");
    let (first, second) = (held_index(&world, "0.1.0"), held_index(&world, "0.2.0"));
    reported_by(&world, DEPLOYER, ("srv", &stable()), ("0.1.0", &first), AT);
    reported_by(&world, DEPLOYER, ("desk", &desk), ("0.1.0", &first), AT);
    let copy = copy_of(&world);
    let before = world.fakes.store.calls().len();
    let mut fetch = timber_core::Fetch::new(&world.fakes.store, &copy, AT);
    let srv = fetch
        .reports(&env("srv"), &stable(), &[first.clone(), second.clone()])
        .unwrap();
    let desk = fetch
        .reports(&env("desk"), &desk, std::slice::from_ref(&first))
        .unwrap();
    assert_eq!(srv.reports.attached.len(), 1);
    assert_eq!(desk.reports.attached.len(), 1);
    let asked: Vec<serde_json::Value> = world.fakes.store.calls()[before..]
        .iter()
        .map(|c| {
            assert_eq!(c.op, StoreOp::ClaimsOf, "status asked the store {:?}", c.op);
            c.given["digest"].clone()
        })
        .collect();
    assert_eq!(asked, [serde_json::json!(first), serde_json::json!(second)]);
}
