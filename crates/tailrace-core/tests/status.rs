//! Spec host-record: what this host keeps of the versions it published,
//! and status answered from it alone; and spec channel-status: every
//! channel the policy declares, read from the ledger alone, beside it.

mod common;
mod world;

use std::fs;
use std::path::{Path, PathBuf};

use tailrace_core::{
    overview, row, status, Channel, ChannelStatus, Entry, Filter, Head, HeadAt, History, Kept,
    Overview, OverviewAsk, Published, RepositoryName, StatusAsk, Written,
};
use tailrace_fakes::{FakeSigner, Fault, KeeperOp, LandingOp, Times};
use tailrace_seams::{
    Audience, Branch, ChannelId, ChannelName, ComponentName, Digest, Envelope, EnvironmentName,
    Keeper, Moved, Moves, Not, Outcome, Promotion, Read, Required, Revision, Satisfied, Seam,
    Signer, SignerId, Source, Version, Why,
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
    let promotion = Promotion {
        channel: channel.name.clone(),
        component: channel.component_name(),
        to: moved(to),
        from: from.map(moved),
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
    let mut chain: Vec<(Digest, Promotion)> = Vec::new();
    for (i, version) in versions.iter().enumerate() {
        let from = i.checked_sub(1).map(|j| versions[j]);
        let predecessor = chain.last().map(|(d, _)| d.clone());
        let (promotion, envelope) = sealed(
            world,
            channel,
            version,
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
    assert_eq!(channels[0].at, Ok(Channel::Empty));
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
    assert_eq!(published.components[0].versions[0].version, version);
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
    declare(&world, &stable(), "lib", None);
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
    assert_eq!(answer.environments, before.environments);
    unread(&answer.environments);
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
    declare(&world, &beta(), "lib", None);
    declare(&world, &stable(), "lib", None);
    promoted(&world, &stable(), &["0.1.0", "0.2.0"]);
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
    declare(&world, &beta(), "lib", None);
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
    assert_eq!(published.components[0].versions[0].version, version);
}

// Requirement: Asking reaches no store, signer or target and writes nothing

#[test]
fn status_asks_no_store_signer_target_or_projection_and_writes_nothing() {
    let mut world = World::new();
    let production = world.fakes.target(&EnvironmentName::new("production"));
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

// Requirement: Status names the environments it cannot yet read

/// 98: the environments' part is unreadable from the target seam, saying
/// no target is read.
fn unread(answer: &tailrace_seams::Answer<tailrace_core::EnvironmentCounts>) {
    let Err(Not::Unreadable(u)) = answer else {
        panic!("the environments answered {answer:?}")
    };
    assert_eq!(u.source.seam, Seam::Target);
    assert!(u.what.contains("environments"), "{u:?}");
    assert!(u.what.contains("no target is read"), "{u:?}");
}

#[test]
fn a_target_bound_and_never_asked_leaves_the_environments_unreadable() {
    let mut world = World::new();
    let production = world.fakes.target(&EnvironmentName::new("production"));
    declare(&world, &stable(), None);
    promoted(&world, &stable(), &["0.1.0"]);
    let answer = asked(&world, &ask(&world));
    unread(&answer.environments);
    assert_eq!(channels_of(&answer).len(), 1);
    assert!(matches!(channels_of(&answer)[0].at, Ok(Channel::At { .. })));
    assert!(production.calls().is_empty(), "status asked the target");
    let json = serde_json::to_value(&answer).unwrap();
    assert_eq!(json["environments"]["answer"], "unreadable", "{json}");

    let history = world.history();
    let unbound = tailrace_core::overview_unbound(
        Not::absent(&Source::new(Seam::Host, "bindings"), "the bindings"),
        &Kept::at(&history),
        &ask(&world),
    );
    unread(&unbound.environments);
}

// Requirement: A repository's row is made from its status

fn repository() -> RepositoryName {
    RepositoryName::from_remote("https://example.com/o/r.git").unwrap()
}

#[test]
fn a_row_names_every_channels_head_and_the_environments_answer() {
    let world = World::new();
    declare(&world, &beta(), None);
    declare(&world, &stable(), Some("beta"));
    let chain = promoted(&world, &stable(), &["0.1.0", "0.2.0"]);
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
    assert_eq!(row.environments, answer.environments);
    unread(&row.environments);
    let json = serde_json::to_value(&row).unwrap();
    assert_eq!(json["repository"]["owner"], "o", "{json}");
    assert_eq!(json["heads"]["value"][1]["at"]["value"]["state"], "at");
    assert_eq!(json["environments"]["answer"], "unreadable", "{json}");
}

#[test]
fn a_row_whose_channels_cannot_be_read_carries_that_answer() {
    let world = World::new();
    declare(&world, &stable(), None);
    world.fakes.policy.fault(
        tailrace_fakes::PolicyOp::Channels,
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
    let url = "https://example.com/agentplot/tailrace.git";
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
        tailrace_core::RepositoryName::from_remote(url).map(|r| r.to_string()),
        Some("agentplot/tailrace".to_string())
    );
}
