//! The status group: what the host published, answered from its own
//! record with no seam asked and nothing written, each version beside its
//! notes rendered again from git (58, 60 to 62, 134); and, below the marker,
//! status-on-any-machine's channels read from the ledger beside it, each
//! channel's answer its own, as last read where the keeper no longer
//! answers, the same with everything kept deleted, and the environments
//! not yet read, each asking the landing, the policy and the keeper alone
//! (31, 36, 58 to 61, 63, 101, 105, 125, 131); and the channels shown from
//! a landing read as last fetched, saying when (36, 60, 77).

use std::fs;

use serde_json::Value;
use tailrace_core::{
    overview, rebuild, status, Channel, ChannelStatus, Entry, Noted, Overview, OverviewAsk,
    Published, RebuildAsk, StatusAsk,
};
use tailrace_fakes::{Fault, KeeperOp, StoreOp, Times};
use tailrace_seams::{
    channel_name, Branch, ChannelId, ChannelName, ComponentName, Held, IndexMeta, Manifest, Moves,
    Name, Not, Read, Seam, Signer, SignerId, Source, Store, Version, Why,
};

use crate::core::publish::{one, passing, TAG, VERSION};
use crate::world::{FakeWorld, GATE, PROMOTED, STABLE};
use crate::{ok, Scenario};

pub const GROUP: &str = "status";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        Scenario {
            name: "status after a publish",
            plays: None,
            cites: &[60, 61, 134],
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
        Scenario {
            name: "every channel at its version or empty",
            plays: None,
            cites: &[31, 101, 125, 131],
            run: every_channel_at_its_version_or_empty,
        },
        Scenario {
            name: "S10 a channel read as last read, the store unreachable",
            plays: Some("S10"),
            cites: &[36, 60, 61],
            run: s10_a_channel_read_as_last_read_the_store_unreachable,
        },
        Scenario {
            name: "one channel's answer beside another's",
            plays: None,
            cites: &[58, 59, 105],
            run: one_channels_answer_beside_anothers,
        },
        Scenario {
            name: "S12 everything kept deleted, the channels the same",
            plays: Some("S12"),
            cites: &[63],
            run: s12_everything_kept_deleted_the_channels_the_same,
        },
        Scenario {
            name: "the environments not yet read, a fake target bound",
            plays: None,
            cites: &[58, 60],
            run: the_environments_not_yet_read_a_fake_target_bound,
        },
        Scenario {
            name: "the landing read as last fetched",
            plays: None,
            cites: &[36, 60, 77],
            run: the_landing_read_as_last_fetched,
        },
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
    let Published::Version {
        index,
        store,
        notes,
        ..
    } = &published
    else {
        panic!("the head was not published: {published:?}")
    };
    let notes = notes.as_ref().expect("the publish answered no notes");
    let Ok(Held::Bytes(carried)) = world.fakes.store.read(&notes.bytes) else {
        panic!("{} holds no bytes", notes.bytes)
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
            notes: Some(Noted {
                last: Some("v0.1.0".into()),
                commits: vec![world.head.clone()],
            }),
        }],
        "status answered another version, revision or index than the publish's, \
         or another entry than the commits the notes were rendered from"
    );
    assert_eq!(
        versions.versions[0].notes,
        Some(Ok(String::from_utf8(carried).expect("the notes are text"))),
        "status's notes are not the bytes the version carries"
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
    assert_eq!(
        entry["notes"]["commits"],
        serde_json::json!([world.head.as_str()]),
        "the entry names other revisions than the notes were rendered from: {text}"
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

const BETA: &str = "beta";

/// The moment status is asked at in the status group's scenarios.
const ASKED: u64 = PROMOTED + 3_600;

/// The head published as 0.2.0, and an index of 0.3.0 put and named
/// `v0.3.0` beside it, so a channel has two versions to move to.
fn two_versions(world: &FakeWorld) {
    published(world);
    let meta = IndexMeta::new(ComponentName::new(COMPONENT), Version::new("0.3.0"), vec![])
        .expect("an index of no artifact");
    let index = ok(
        world.fakes.store.put_artifact(&Manifest::Index(meta)),
        "the other index's put",
    )
    .digest;
    ok(
        world.fakes.store.name(&Name::new("v0.3.0"), &index, None),
        "the other index's name",
    );
}

/// The whole of status over the world's binding, asked at [`ASKED`].
fn asked(world: &FakeWorld) -> Overview {
    let history = world.history();
    overview(
        &world.seams(),
        &history,
        &world.kept(),
        &OverviewAsk {
            branch: world.main.clone(),
            component: None,
            revision: None,
            at: ASKED,
        },
    )
}

fn channels_of(overview: &Overview) -> &[ChannelStatus] {
    match &overview.channels {
        Ok(channels) => &channels.channels,
        Err(not) => panic!("the channels' part answered {not:?}"),
    }
}

/// How many operations the store, the signers, the targets and the
/// projections have been asked, all told: what status asks none of.
fn reached(world: &FakeWorld) -> usize {
    let fakes = &world.fakes;
    fakes.store.calls().len()
        + fakes.signer.calls().len()
        + world.target_calls().len()
        + fakes
            .projections
            .values()
            .map(|p| p.calls().len())
            .sum::<usize>()
}

/// How many appends the keeper has been asked.
fn appends(world: &FakeWorld) -> usize {
    world
        .keeper_calls()
        .iter()
        .filter(|op| **op == KeeperOp::Append)
        .count()
}

fn every_channel_at_its_version_or_empty(world: &FakeWorld) {
    two_versions(world);
    world.declare_channels(&[(BETA, None), (STABLE, Some(BETA))]);
    let first = world.promote_fake(STABLE, VERSION);
    let second = world.promote_fake(STABLE, "0.3.0");
    // A name in the store that says otherwise is not where the channel is
    // (101).
    let elsewhere = ok(
        world.fakes.store.resolve(&Name::new(TAG)),
        "the published version's name",
    );
    ok(
        world.fakes.store.name(
            &channel_name(&ChannelId::new(None, ChannelName::new(STABLE))),
            &elsewhere,
            None,
        ),
        "another name for stable",
    );
    let (before, appended, record) = (reached(world), appends(world), world.record());

    let answer = asked(world);
    assert_eq!(
        answer.channels.as_ref().map(|c| &c.revision),
        Ok(&world.head),
        "the channels were read at another revision than the landed head"
    );
    let channels = channels_of(&answer);
    let names: Vec<&str> = channels.iter().map(|c| c.name().as_str()).collect();
    assert_eq!(names, [BETA, STABLE], "the policy's channels, in its order");
    for channel in channels {
        assert_eq!(
            channel.declared.channel.component_name(),
            ComponentName::new(COMPONENT),
            "{} names another component",
            channel.name()
        );
    }
    assert_eq!(
        channels[0].at,
        Ok(Channel::Empty { read: Read::Source }),
        "beta was never promoted"
    );
    assert_eq!(channels[0].declared.moves, Moves::Promotion);
    assert_eq!(
        channels[1].declared.moves,
        Moves::Bound {
            to: ChannelName::new(BETA)
        }
    );
    let Ok(Channel::At { head, chain, read }) = &channels[1].at else {
        panic!("stable answered {:?}", channels[1].at)
    };
    assert_eq!(
        head.to.version.as_str(),
        "0.3.0",
        "stable is at its head's version"
    );
    let newest_first: Vec<_> = chain.iter().rev().map(|(d, _)| d.clone()).collect();
    assert_eq!(
        newest_first,
        [second, first],
        "both promotions, newest first"
    );
    assert_eq!(
        chain[1].1.from.as_ref().map(|f| f.version.as_str()),
        Some(VERSION),
        "the second promotion moved from the first's version"
    );
    assert_eq!(*read, Read::Source);
    assert_eq!(
        reached(world),
        before,
        "status asked the store, a signer, a target or a projection"
    );
    assert_eq!(appends(world), appended, "status appended to the ledger");
    assert_eq!(world.record(), record, "status wrote to the record");
}

fn s10_a_channel_read_as_last_read_the_store_unreachable(world: &FakeWorld) {
    two_versions(world);
    world.declare_channels(&[(BETA, None), (STABLE, None)]);
    world.promote_fake(BETA, "0.3.0");
    world.promote_fake(STABLE, VERSION);
    let first = asked(world);
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
    world
        .fakes
        .keeper
        .unreachable("fatal: unable to access the ledger");
    let store = world.store_calls().len();

    let again = asked(world);
    let (before, after) = (channels_of(&first), channels_of(&again));
    assert_eq!(after.len(), before.len());
    for (was, is) in before.iter().zip(after) {
        let (
            Ok(Channel::At {
                head,
                chain,
                read: Read::Source,
            }),
            Ok(Channel::At {
                head: kept_head,
                chain: kept_chain,
                read: Read::Kept { at, said },
            }),
        ) = (&was.at, &is.at)
        else {
            panic!(
                "{} answered {:?} and then {:?}, not as last read",
                is.name(),
                was.at,
                is.at
            )
        };
        assert_eq!(
            (kept_head, kept_chain),
            (head, chain),
            "{} moved",
            is.name()
        );
        assert!(*at > 0, "{} was last read at no time", is.name());
        assert_eq!(
            said, "fatal: unable to access the ledger",
            "the keeper's words"
        );
    }
    assert!(
        !matches!(again.published, Err(Not::Refused(_))),
        "the record's part was refused: {:?}",
        again.published
    );
    assert_eq!(
        first.published, again.published,
        "the record answered otherwise"
    );
    assert_eq!(world.store_calls().len(), store, "status asked the store");
}

fn one_channels_answer_beside_anothers(world: &FakeWorld) {
    two_versions(world);
    let edge = "edge";
    world.declare_channels(&[(BETA, None), (edge, None), (STABLE, None)]);
    world.promote_fake(BETA, VERSION);
    let held = world.promote_fake(STABLE, VERSION);
    let gate = tailrace_fakes::FakeSigner::new();
    gate.hold(&SignerId::new(GATE));
    let sealed = |version: &str| {
        let claim = tailrace_seams::Claim {
            subject: ok(
                world.fakes.store.resolve(&Name::new(format!("v{version}"))),
                "the version's name",
            ),
            kind: tailrace_seams::ClaimKind::Promotion,
            revision: world.head.clone(),
            build: None,
            body: serde_json::json!({ "channel": edge, "version": version }),
        };
        ok(gate.sign(&claim, &SignerId::new(GATE)), "the gate's seal")
    };
    let (a, b) = (sealed(VERSION), sealed("0.3.0"));
    world
        .fakes
        .keeper
        .fork(&ChannelId::new(None, ChannelName::new(edge)), &a, &b);
    world.fakes.keeper.fault(
        KeeperOp::Chain,
        Fault::Unreadable("connection reset".into()),
        Times::Once,
    );
    let before = reached(world);

    let answer = asked(world);
    let channels = channels_of(&answer);
    match &channels[0].at {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source.seam, Seam::Keeper, "{u:?}");
            assert_eq!(
                u.said.as_deref(),
                Some("connection reset"),
                "the keeper's words were not carried verbatim"
            );
        }
        other => panic!("the first channel asked answered {other:?}, not unreadable"),
    }
    match &channels[1].at {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::Fork {
                predecessor: None,
                claims: vec![a.digest(), b.digest()],
            },
            "the fork names other claims"
        ),
        other => panic!("the forked channel answered {other:?}, not refused"),
    }
    match &channels[2].at {
        Ok(Channel::At { chain, .. }) => assert_eq!(chain[0].0, held),
        other => panic!("stable answered {other:?} beside the others"),
    }
    assert_eq!(
        reached(world),
        before,
        "status asked the store, a signer, a target or a projection"
    );
}

fn s12_everything_kept_deleted_the_channels_the_same(world: &FakeWorld) {
    two_versions(world);
    world.declare_channels(&[(BETA, None), (STABLE, Some(BETA))]);
    world.promote_fake(BETA, VERSION);
    world.promote_fake(STABLE, VERSION);
    let before = asked(world);
    assert!(before.published.is_ok(), "{:?}", before.published);

    world.delete_record();
    let deleted = asked(world);
    assert_eq!(
        deleted.channels, before.channels,
        "the channels answered otherwise with what tailrace keeps deleted"
    );
    assert!(
        matches!(deleted.published, Err(Not::Absent(_))),
        "the record's part answered {:?} once deleted",
        deleted.published
    );

    let history = world.history();
    ok(
        rebuild(
            &world.seams(),
            &history,
            &world.kept(),
            &RebuildAsk {
                branch: world.main.clone(),
                revision: world.head.clone(),
                store: world.store(),
            },
        ),
        "the rebuild",
    );
    let rebuilt = asked(world);
    assert_eq!(rebuilt.channels, before.channels);
    assert_eq!(
        rebuilt.published, before.published,
        "the record's part answered otherwise once rebuilt"
    );
}

fn the_environments_not_yet_read_a_fake_target_bound(world: &FakeWorld) {
    assert!(
        world.fakes.targets.contains_key(&world.environment.name),
        "the world binds no target"
    );
    published(world);
    world.declare_channels(&[(STABLE, None)]);
    world.promote_fake(STABLE, VERSION);
    let targets = world.target_calls();

    let answer = asked(world);
    match &answer.environments {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source.seam, Seam::Target, "{u:?}");
            assert!(u.what.contains("no target is read"), "{u:?}");
        }
        other => panic!("the environments answered {other:?}, not unreadable"),
    }
    assert!(
        matches!(&channels_of(&answer)[0].at, Ok(Channel::At { .. })),
        "the channels were not shown beside the environments"
    );
    assert_eq!(world.target_calls(), targets, "status asked the target");
}

/// 77: the branch's head answered landed from the landing's copy shows
/// every channel as a landing answering from itself shows it, the
/// channels' part saying the landing was read as last fetched, when, and
/// with the remote's words (36, 60).
fn the_landing_read_as_last_fetched(world: &FakeWorld) {
    const WORDS: &str = "fatal: unable to access 'https://forge.example/o/r.git/': \
                         Could not resolve host: forge.example\n";
    two_versions(world);
    world.declare_channels(&[(BETA, None), (STABLE, None)]);
    world.promote_fake(STABLE, VERSION);
    let since = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let first = asked(world);
    assert_eq!(
        first.channels.as_ref().map(|c| &c.landing),
        Ok(&Read::Source),
        "a landing answering from itself said it read a copy"
    );
    world.fakes.landing.unreachable(WORDS);

    let again = asked(world);
    let channels = match &again.channels {
        Ok(channels) => channels,
        Err(not) => panic!("the channels' part answered {not:?} from a landed head's copy"),
    };
    let Read::Kept { at, said } = &channels.landing else {
        panic!(
            "the channels' part says the landing was read {:?}",
            channels.landing
        )
    };
    assert!(
        *at >= since,
        "the landing was fetched at {at}, before it was read"
    );
    assert_eq!(said, WORDS, "the remote's words were not carried unchanged");
    assert_eq!(
        channels_of(&again),
        channels_of(&first),
        "the channels were shown otherwise from the landing's copy"
    );
    assert_eq!(channels.revision, world.head);
    let json = serde_json::to_value(&again).expect("status serializes");
    let landing = &json["channels"]["value"]["landing"]["kept"];
    assert_eq!(landing["at"], *at, "{json}");
    assert_eq!(landing["said"], WORDS, "{json}");
}
