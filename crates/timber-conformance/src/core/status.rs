//! The status group: what the host published, answered from its own
//! record with no seam asked and nothing written, each version beside its
//! notes rendered again from git (58, 60 to 62, 134); and, below the marker,
//! status-on-any-machine's channels read from the ledger beside it, each
//! channel's answer its own, as last read where the keeper no longer
//! answers, the same with everything kept deleted, each in a world whose
//! policy declares no environment, so status asks the landing, the policy
//! and the keeper and the store nothing (31, 36, 58 to 61, 63, 101, 105,
//! 125, 131); the channels shown from a landing read as last fetched,
//! saying when (36, 60, 77); and every environment the policy declares,
//! what it should run from its channel's chain and what it runs from the
//! newest counted report the fakes' apply attached, read from the store
//! into the host's copy and never asked of the target, with the notes
//! between where it is behind, as last fetched where the store no longer
//! answers, the same with the copy deleted, counted on the part and the
//! row, and S11 (9, 23, 34, 37, 40, 41, 43, 56 to 61, 63, 91, 97, 98, 132,
//! 134).

use std::fs;

use serde_json::{json, Value};
use timber_core::{
    apply, overview, rebuild, row, status, ApplyAsk, Channel, ChannelStatus, Direction, Entry,
    EnvironmentCounts, EnvironmentStatus, Environments, HeadAt, MovesOn, Noted, Overview,
    OverviewAsk, Published, RebuildAsk, ReportsCopy, Runs, State, StatusAsk,
};
use timber_fakes::{Fault, KeeperOp, SignerOp, StoreOp, Times};
use timber_seams::{
    channel_name, AskedOf, Audience, Branch, ChannelId, ChannelName, ClaimKind, ComponentName,
    DeclaredEnvironment, Digest, Envelope, EnvironmentName, Held, IndexMeta, Manifest, Moves, Name,
    NamedSigner, Not, Read, Required, RequiredClaim, Revision, Seam, Signer, SignerId, Source,
    Store, TargetProfile, TargetTable, Version, Why,
};

use crate::core::publish::{one, passing, TAG, VERSION};
use crate::core::surfaces::notes_held;
use crate::world::{FakeWorld, GATE, HOST, PROMOTED, STABLE, TARGET, WHEN};
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
            name: "the landing read as last fetched",
            plays: None,
            cites: &[36, 60, 77],
            run: the_landing_read_as_last_fetched,
        },
        Scenario {
            name: "an environment running what it should",
            plays: None,
            cites: &[40, 43, 60, 91, 98],
            run: an_environment_running_what_it_should,
        },
        Scenario {
            name: "an environment behind, with the notes between",
            plays: None,
            cites: &[23, 60, 98, 134],
            run: an_environment_behind_with_the_notes_between,
        },
        Scenario {
            name: "an environment its channel moved back",
            plays: None,
            cites: &[34, 98, 134],
            run: an_environment_its_channel_moved_back,
        },
        Scenario {
            name: "an environment waiting on a person",
            plays: None,
            cites: &[41, 97, 98, 132],
            run: an_environment_waiting_on_a_person,
        },
        Scenario {
            name: "an environment after another",
            plays: None,
            cites: &[37, 41, 98],
            run: an_environment_after_another,
        },
        Scenario {
            name: "an environment no counted report names",
            plays: None,
            cites: &[58, 98],
            run: an_environment_no_counted_report_names,
        },
        Scenario {
            name: "a report by a signer the policy lets claim none",
            plays: None,
            cites: &[9, 61, 98],
            run: a_report_by_a_signer_the_policy_lets_claim_none,
        },
        Scenario {
            name: "an environment whose chain cannot be read, beside another",
            plays: None,
            cites: &[58, 59, 98],
            run: an_environment_whose_chain_cannot_be_read_beside_another,
        },
        Scenario {
            name: "an environment's reports read across a chain as last read",
            plays: None,
            cites: &[60, 98],
            run: an_environments_reports_read_across_a_chain_as_last_read,
        },
        Scenario {
            name: "S10 the environments as last fetched, the store unreachable",
            plays: Some("S10"),
            cites: &[60, 61],
            run: s10_the_environments_as_last_fetched_the_store_unreachable,
        },
        Scenario {
            name: "S12 the reports' copy deleted, the environments the same",
            plays: Some("S12"),
            cites: &[61, 63],
            run: s12_the_reports_copy_deleted_the_environments_the_same,
        },
        Scenario {
            name: "one environment of each state, counted on the part and the row",
            plays: None,
            cites: &[98, 132],
            run: one_environment_of_each_state_counted_on_the_part_and_the_row,
        },
        Scenario {
            name: "S11 each environment with what it should run and what it runs",
            plays: Some("S11"),
            cites: &[56, 57, 98, 132],
            run: s11_each_environment_with_what_it_should_run_and_what_it_runs,
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
            version: timber_seams::Version::new(VERSION),
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

/// 60, 98: the environments' part of `overview` in a world whose policy
/// declares no environment: an empty list with four counts of none, so
/// status had no environment's reports to ask the store for.
fn declares_no_environment(overview: &Overview) {
    assert_eq!(
        overview.environments,
        Ok(Environments {
            counts: EnvironmentCounts::default(),
            environments: vec![],
        }),
        "the world's policy declares an environment"
    );
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
    declares_no_environment(&answer);
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
            to: ChannelName::new(BETA),
            after: vec![],
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
    declares_no_environment(&again);
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
    let gate = timber_fakes::FakeSigner::new();
    gate.hold(&SignerId::new(GATE));
    let sealed = |version: &str| {
        let claim = timber_seams::Claim {
            subject: ok(
                world.fakes.store.resolve(&Name::new(format!("v{version}"))),
                "the version's name",
            ),
            kind: timber_seams::ClaimKind::Promotion,
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
    declares_no_environment(&answer);
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
        "the channels answered otherwise with what timber keeps deleted"
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

// The environments: what each should run, read from its channel's chain,
// and what it runs, the newest counted report the fakes' apply attached
// beside an index that chain moved to, fetched from the store into the
// host's copy and never asked of the target (43, 60, 91, 98).

/// The environments the scenarios declare: two on `stable`, and one
/// that waits on [`OPS`] with a channel of its own.
const SRV: &str = "srv";
const EU: &str = "eu";
const DESK: &str = "desk";

/// The one approver of [`DESK`]'s channel.
const OPS: &str = "ops";

/// The version a feature on the site after the head takes.
const GROWN: &str = "0.3.0";

/// When the fake target's applies converge.
const CONVERGED: &str = "2026-11-06T09:00:00Z";

fn environment(name: &str) -> EnvironmentName {
    EnvironmentName::new(name)
}

fn channel(name: &str) -> ChannelId {
    ChannelId::new(None, ChannelName::new(name))
}

fn named(id: &str, kinds: &[ClaimKind]) -> NamedSigner {
    NamedSigner {
        id: SignerId::new(id),
        verified_by: format!("{id}'s key"),
        kinds: kinds.to_vec(),
        checks: vec![],
    }
}

/// The host's signer as the policy names it for reports (116, C.10).
fn reporter() -> NamedSigner {
    named(HOST, &[ClaimKind::Report])
}

/// 116: the policy for a layout lists `signers`, the signers it lets
/// claim a report among them (rule core.7).
fn lets_report(world: &FakeWorld, signers: Vec<NamedSigner>) {
    world.fakes.policy.require(
        Audience::Layout,
        Required::Layout {
            claims: vec![],
            given: false,
            signers,
        },
    );
}

/// One version the world published: its version, its revision and its
/// index.
pub(crate) struct Released {
    pub version: Version,
    pub revision: Revision,
    pub index: Digest,
}

/// 98: the world's one component published at `start` as 0.1.0, at
/// `head` as 0.2.0 and at a feature grown on the site after it as 0.3.0,
/// each named `v<version>` in the fake store with its notes and carrying
/// the gate's acceptance; `stable` declared moving by a promotion alone,
/// requiring that acceptance, so an apply judges it (42); the policy for
/// a layout letting the host's signer claim a report; and the fake
/// target's applies converging at [`CONVERGED`]. The three, oldest
/// first.
pub(crate) fn three_versions(world: &FakeWorld) -> [Released; 3] {
    let grown = {
        for revision in [&world.start, &world.head] {
            passing(world, revision);
            ok(one(world.publish(&world.ask(revision))), "the publish");
        }
        world.grow()
    };
    passing(world, &grown);
    ok(one(world.publish(&world.ask(&grown))), "the third publish");
    accepting(world, &[STABLE], None);
    lets_report(world, vec![reporter()]);
    world.fakes.targets[&TargetProfile::new(TARGET)].at(CONVERGED);
    [
        ("0.1.0", world.start.clone()),
        (VERSION, world.head.clone()),
        (GROWN, grown),
    ]
    .map(|(version, revision)| {
        let index = ok(
            world.fakes.store.resolve(&Name::new(format!("v{version}"))),
            "the version's name",
        );
        world.attach_as(GATE, &index, ClaimKind::Acceptance, &revision, json!({}));
        Released {
            index,
            version: Version::new(version),
            revision,
        }
    })
}

/// 82, 83: each of `channels` declared requiring the gate's acceptance,
/// bound to `bound_to` where it names a channel.
fn accepting(world: &FakeWorld, channels: &[&str], bound_to: Option<&str>) {
    for name in channels {
        world.channel_policy(name, false, bound_to);
    }
}

/// 136: `name` declared at the branch's landed head, following
/// `follows` on the fake target, waiting where `waits` on `approvers`.
pub(crate) fn declares(
    world: &FakeWorld,
    name: &str,
    follows: &str,
    waits: bool,
    approvers: &[&str],
) {
    let head = ok(world.history().head(&world.main), "the branch's head");
    world.fakes.policy.declares_at(
        &head,
        DeclaredEnvironment {
            name: environment(name),
            channel: channel(follows),
            waits,
            approvers: approvers.iter().map(|a| SignerId::new(*a)).collect(),
            target: Some(TargetTable {
                profile: TargetProfile::new(TARGET),
                settings: json!({}),
            }),
        },
    );
}

/// 43, 91: `name` applied by the host at `at`, converged on its
/// channel's head: the report it attached beside that head's index.
pub(crate) fn reported(world: &FakeWorld, name: &str, at: u64) -> Digest {
    let applied = ok(
        apply(
            &world.seams(),
            &world.history(),
            &ApplyAsk {
                branch: world.main.clone(),
                component: None,
                environment: environment(name),
                signer: SignerId::new(HOST),
                at,
            },
        ),
        &format!("the apply of {name}"),
    );
    ok(applied.report, &format!("{name}'s report"))
}

/// What an environment runs where the host's apply at `at` reported
/// `released`, its claim `claim`, read as `read`.
fn runs(released: &Released, at: u64, claim: &Digest, read: Read) -> Runs {
    Runs {
        version: released.version.clone(),
        index: released.index.clone(),
        converged_at: CONVERGED.into(),
        at,
        claim: claim.clone(),
        signer: SignerId::new(HOST),
        read,
    }
}

/// What a channel's head at `released` answers.
fn at(released: &Released) -> HeadAt {
    HeadAt::At {
        version: released.version.clone(),
        index: released.index.clone(),
    }
}

/// The environments' part of `overview`, answered.
fn environments_of(overview: &Overview) -> &Environments {
    match &overview.environments {
        Ok(environments) => environments,
        Err(not) => panic!("the environments' part answered {not:?}"),
    }
}

/// The environment `name` of `overview`'s environments.
fn of<'o>(overview: &'o Overview, name: &str) -> &'o EnvironmentStatus {
    environments_of(overview)
        .environments
        .iter()
        .find(|e| e.environment == environment(name))
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

/// How many signs the host's signer has been asked.
fn signs(world: &FakeWorld) -> usize {
    world
        .fakes
        .signer
        .calls()
        .iter()
        .filter(|c| c.op == SignerOp::Sign)
        .count()
}

/// 60: what status may not do while it reads the environments: ask a
/// target, sign, or write to the store, all told.
fn untouched(world: &FakeWorld) -> (usize, usize, usize) {
    (
        world.target_calls().len(),
        signs(world),
        world.writes().len(),
    )
}

/// The whole of status asked at `moment`.
fn asked_at(world: &FakeWorld, moment: u64) -> Overview {
    let history = world.history();
    overview(
        &world.seams(),
        &history,
        &world.kept(),
        &OverviewAsk {
            branch: world.main.clone(),
            component: None,
            revision: None,
            at: moment,
        },
    )
}

/// The absent answer `answer` carries, naming what is not there.
fn absent_what<T: std::fmt::Debug>(answer: &timber_seams::Answer<T>) -> String {
    match answer {
        Err(Not::Absent(a)) => a.what.clone(),
        other => panic!("answered {other:?}, not absent"),
    }
}

fn an_environment_running_what_it_should(world: &FakeWorld) {
    let [first, second, _] = three_versions(world);
    declares(world, SRV, STABLE, false, &[]);
    world.promote_fake(STABLE, "0.1.0");
    reported(world, SRV, WHEN);
    world.promote_fake(STABLE, VERSION);
    let newest = reported(world, SRV, WHEN + 60);
    let before = untouched(world);

    let answer = asked(world);
    let srv = of(&answer, SRV);
    assert_eq!(
        (&srv.component, &srv.channel, srv.waits, &srv.moves_on),
        (
            &ComponentName::new(COMPONENT),
            &ChannelName::new(STABLE),
            false,
            &None
        ),
        "srv is named with another component, channel or wait"
    );
    let Ok(Channel::At { head, .. }) = &channels_of(&answer)[0].at else {
        panic!("stable answered {:?}", channels_of(&answer)[0].at)
    };
    assert_eq!(
        srv.should,
        Ok(HeadAt::At {
            version: head.to.version.clone(),
            index: head.to.index.clone(),
        }),
        "what srv should run is not its channel's head"
    );
    assert_eq!(srv.should, Ok(at(&second)));
    assert_eq!(
        srv.runs,
        Ok(runs(&second, WHEN + 60, &newest, Read::Source)),
        "what srv runs is not its newest counted report, with when it converged"
    );
    assert_ne!(first.index, second.index);
    assert_eq!(srv.state, Ok(State::Converged));
    assert_eq!(
        srv.between, None,
        "an environment running what it should has versions between"
    );
    assert_eq!(environments_of(&answer).counts, counts(1, 0, 0, 0));
    assert_eq!(
        untouched(world),
        before,
        "status asked the target, signed, or wrote to the store"
    );
}

fn an_environment_behind_with_the_notes_between(world: &FakeWorld) {
    let [first, second, third] = three_versions(world);
    declares(world, SRV, STABLE, false, &[]);
    world.promote_fake(STABLE, "0.1.0");
    let report = reported(world, SRV, WHEN);
    world.promote_fake(STABLE, VERSION);
    world.promote_fake(STABLE, GROWN);
    let notes = [
        notes_held(world, &second.index),
        notes_held(world, &third.index),
    ];
    let before = untouched(world);

    let answer = asked(world);
    let srv = of(&answer, SRV);
    assert_eq!(srv.should, Ok(at(&third)), "srv should run stable's head");
    assert_eq!(
        srv.runs,
        Ok(runs(&first, WHEN, &report, Read::Source)),
        "srv runs other than its one report names"
    );
    assert_eq!(srv.state, Ok(State::Behind));
    let Some(Ok(between)) = &srv.between else {
        panic!("srv's versions between answered {:?}", srv.between)
    };
    assert_eq!(between.direction, Direction::Ahead);
    let versions: Vec<(&str, &Revision, &timber_seams::Answer<String>)> = between
        .versions
        .iter()
        .map(|v| (v.version.as_str(), &v.revision, &v.notes))
        .collect();
    assert_eq!(
        versions,
        [
            (VERSION, &second.revision, &Ok(notes[0].clone())),
            (GROWN, &third.revision, &Ok(notes[1].clone())),
        ],
        "the versions between are not those above what srv runs, ascending, \
         each at its tag's revision with the notes its index names"
    );
    assert_eq!(environments_of(&answer).counts, counts(0, 1, 0, 0));
    assert_eq!(
        untouched(world),
        before,
        "status asked the target, signed, or wrote to the store"
    );
}

fn an_environment_its_channel_moved_back(world: &FakeWorld) {
    let [first, second, _] = three_versions(world);
    declares(world, SRV, STABLE, false, &[]);
    world.promote_fake(STABLE, "0.1.0");
    world.promote_fake(STABLE, VERSION);
    let report = reported(world, SRV, WHEN);
    // 34: a wrong move undone by a promotion back.
    world.promote_fake(STABLE, "0.1.0");
    let notes = notes_held(world, &second.index);

    let answer = asked(world);
    let srv = of(&answer, SRV);
    assert_eq!(srv.should, Ok(at(&first)));
    assert_eq!(srv.runs, Ok(runs(&second, WHEN, &report, Read::Source)));
    assert_eq!(srv.state, Ok(State::Behind));
    let Some(Ok(between)) = &srv.between else {
        panic!("srv's versions between answered {:?}", srv.between)
    };
    assert_eq!(
        between.direction,
        Direction::Back,
        "srv's channel moved back, and it moves ahead"
    );
    let versions: Vec<(&str, &timber_seams::Answer<String>)> = between
        .versions
        .iter()
        .map(|v| (v.version.as_str(), &v.notes))
        .collect();
    assert_eq!(
        versions,
        [(VERSION, &Ok(notes))],
        "the versions whose changes srv leaves are not 0.2.0's alone"
    );
}

fn an_environment_waiting_on_a_person(world: &FakeWorld) {
    let [first, second, _] = three_versions(world);
    accepting(world, &[DESK], Some(STABLE));
    declares(world, DESK, DESK, true, &[OPS]);
    world.promote_fake(STABLE, "0.1.0");
    world.promote_fake(STABLE, VERSION);
    world.promote_fake(DESK, "0.1.0");
    let report = reported(world, DESK, WHEN);

    let answer = asked(world);
    let desk = of(&answer, DESK);
    assert_eq!(
        (&desk.channel, desk.waits, &desk.approvers),
        (&ChannelName::new(DESK), true, &vec![SignerId::new(OPS)]),
        "desk is named with another channel, or waits on someone else"
    );
    assert_eq!(
        desk.should,
        Ok(at(&first)),
        "desk should run its own channel's head"
    );
    assert_eq!(desk.runs, Ok(runs(&first, WHEN, &report, Read::Source)));
    assert_eq!(
        desk.moves_on,
        Some(MovesOn::Asking {
            bound_to: ChannelName::new(STABLE),
            at: Ok(at(&second)),
        }),
        "desk's channel does not move on an approver's asking toward stable's head"
    );
    assert_eq!(desk.state, Ok(State::Waiting));
    assert_eq!(desk.between, None);
    assert_eq!(environments_of(&answer).counts, counts(0, 0, 1, 0));
    let json = serde_json::to_value(&answer).expect("status serializes");
    let desk = &json["environments"]["value"]["environments"][0];
    assert_eq!(desk["approvers"], json!([OPS]), "{json}");
    assert_eq!(desk["moves_on"]["by"], "asking", "{json}");
    assert_eq!(desk["state"]["value"], "waiting", "{json}");
}

const FIRST: &str = "first";
const SECOND: &str = "second";

fn an_environment_after_another(world: &FakeWorld) {
    let [first, second, _] = three_versions(world);
    accepting(world, &[FIRST], Some(STABLE));
    world.fakes.policy.after(
        &channel(SECOND),
        &[environment(FIRST)],
        &[reporter()],
        Required::Channel {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Acceptance,
                check: None,
                environment: None,
                signers: vec![named(GATE, &[ClaimKind::Acceptance])],
                asked_of: AskedOf::Every,
            }],
            bound_to: Some(ChannelName::new(FIRST)),
            promoted_by: vec![],
            approvers: vec![],
        },
    );
    declares(world, FIRST, FIRST, true, &[OPS]);
    declares(world, SECOND, SECOND, false, &[]);
    world.promote_fake(STABLE, "0.1.0");
    world.promote_fake(STABLE, VERSION);
    world.promote_fake(FIRST, "0.1.0");
    reported(world, FIRST, WHEN);
    world.promote_fake(SECOND, "0.1.0");
    let report = reported(world, SECOND, WHEN + 60);

    let answer = asked(world);
    let after = of(&answer, SECOND);
    assert_eq!(
        after.moves_on,
        Some(MovesOn::Reports {
            after: vec![environment(FIRST)],
        }),
        "second's channel does not move on first's reports"
    );
    assert_eq!(
        after.runs,
        Ok(runs(&first, WHEN + 60, &report, Read::Source))
    );
    assert_eq!(
        after.state,
        Ok(State::Converged),
        "an environment after another waited on a person"
    );
    assert_eq!(
        of(&answer, FIRST).state,
        Ok(State::Waiting),
        "first, behind stable's {} on its own channel, is not waiting",
        second.version
    );
    let json = serde_json::to_value(after).expect("an environment serializes");
    assert_eq!(
        json["moves_on"],
        json!({ "by": "reports", "after": [FIRST] }),
        "{json}"
    );
}

fn an_environment_no_counted_report_names(world: &FakeWorld) {
    three_versions(world);
    declares(world, SRV, STABLE, false, &[]);
    world.promote_fake(STABLE, "0.1.0");

    let answer = asked(world);
    let srv = of(&answer, SRV);
    let what = absent_what(&srv.runs);
    assert!(
        what.contains("a counted report of environment srv of repository"),
        "srv's absence names another thing: {what}"
    );
    assert_eq!(
        srv.state.as_ref().err(),
        srv.runs.as_ref().err(),
        "srv's state is not unread with the reason what it runs is"
    );
    assert_eq!(environments_of(&answer).counts, counts(0, 0, 0, 1));
}

/// A signer the policy lets claim a report, which is not the host's.
const DEPLOYER: &str = "deployer";

fn a_report_by_a_signer_the_policy_lets_claim_none(world: &FakeWorld) {
    let [first, ..] = three_versions(world);
    lets_report(world, vec![named(DEPLOYER, &[ClaimKind::Report])]);
    declares(world, SRV, STABLE, false, &[]);
    world.promote_fake(STABLE, "0.1.0");
    let report = reported(world, SRV, WHEN);

    let answer = asked(world);
    let srv = of(&answer, SRV);
    let what = absent_what(&srv.runs);
    assert!(
        what.contains("a counted report of environment srv of repository"),
        "a report by a signer the policy lets claim none was counted: {what}"
    );
    assert_eq!(environments_of(&answer).counts, counts(0, 0, 0, 1));
    // 9: kept, and never counted.
    let path = ReportsCopy::at(&world.history()).path(&environment(SRV), &channel(STABLE));
    let copy: Value = serde_json::from_str(
        &fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} cannot be read: {e}", path.display())),
    )
    .unwrap_or_else(|e| panic!("{} is not JSON: {e}", path.display()));
    let kept: Vec<Digest> = copy["reports"]
        .as_array()
        .expect("the copy's reports")
        .iter()
        .map(|held| {
            assert_eq!(held["index"], first.index.as_str(), "{copy}");
            serde_json::from_value::<Envelope>(held["envelope"].clone())
                .expect("an envelope")
                .digest()
        })
        .collect();
    assert_eq!(
        kept,
        [report],
        "the copy does not keep the report uncounted"
    );
}

fn an_environment_whose_chain_cannot_be_read_beside_another(world: &FakeWorld) {
    let [first, ..] = three_versions(world);
    let edge = "edge";
    accepting(world, &[edge], None);
    declares(world, "web", edge, false, &[]);
    declares(world, SRV, STABLE, false, &[]);
    world.promote_fake(edge, "0.1.0");
    world.promote_fake(STABLE, "0.1.0");
    let report = reported(world, SRV, WHEN);
    world.fakes.keeper.fault(
        KeeperOp::Chain,
        Fault::Unreadable("connection reset".into()),
        Times::Once,
    );

    let answer = asked(world);
    let web = of(&answer, "web");
    assert_eq!(
        web.should.as_ref().err(),
        channels_of(&answer)[0].at.as_ref().err(),
        "what web should run is not its chain's own answer"
    );
    let Err(Not::Unreadable(u)) = &web.state else {
        panic!("web's state answered {:?}", web.state)
    };
    assert_eq!(
        u.said.as_deref(),
        Some("connection reset"),
        "the keeper's words were not carried verbatim"
    );
    let srv = of(&answer, SRV);
    assert_eq!(srv.runs, Ok(runs(&first, WHEN, &report, Read::Source)));
    assert_eq!(
        srv.state,
        Ok(State::Converged),
        "srv was not shown beside web"
    );
    assert_eq!(environments_of(&answer).counts, counts(1, 0, 0, 1));
}

fn an_environments_reports_read_across_a_chain_as_last_read(world: &FakeWorld) {
    let [_, second, _] = three_versions(world);
    declares(world, SRV, STABLE, false, &[]);
    world.promote_fake(STABLE, "0.1.0");
    world.promote_fake(STABLE, VERSION);
    let unreported = asked(world);
    assert!(
        matches!(of(&unreported, SRV).runs, Err(Not::Absent(_))),
        "srv answered {:?} before any apply",
        of(&unreported, SRV).runs
    );
    let report = reported(world, SRV, WHEN);
    world
        .fakes
        .keeper
        .unreachable("fatal: unable to access the ledger");

    let answer = asked(world);
    assert!(
        matches!(
            &channels_of(&answer)[0].at,
            Ok(Channel::At {
                read: Read::Kept { .. },
                ..
            })
        ),
        "stable answered {:?}, not as last read",
        channels_of(&answer)[0].at
    );
    let srv = of(&answer, SRV);
    assert_eq!(srv.should, Ok(at(&second)));
    assert_eq!(
        srv.runs,
        Ok(runs(&second, WHEN, &report, Read::Source)),
        "srv's reports were not read from the store across the chain as last read"
    );
    assert_eq!(srv.state, Ok(State::Converged));
}

/// `srv` behind, running 0.1.0 while `stable` is at 0.3.0, and `eu`
/// running 0.3.0, each reported by the host's apply; the three versions.
pub(crate) fn one_behind_and_one_not(world: &FakeWorld) -> [Released; 3] {
    let released = three_versions(world);
    declares(world, SRV, STABLE, false, &[]);
    declares(world, EU, STABLE, false, &[]);
    world.promote_fake(STABLE, "0.1.0");
    reported(world, SRV, WHEN);
    world.promote_fake(STABLE, VERSION);
    world.promote_fake(STABLE, GROWN);
    reported(world, EU, WHEN + 60);
    released
}

/// `environment` as status showed it from the source, where it shows it
/// read from the copy.
fn read_from_source(environment: &EnvironmentStatus) -> EnvironmentStatus {
    let mut shown = environment.clone();
    if let Ok(runs) = &mut shown.runs {
        runs.read = Read::Source;
    }
    shown
}

fn s10_the_environments_as_last_fetched_the_store_unreachable(world: &FakeWorld) {
    const WORDS: &str = "connection refused";
    one_behind_and_one_not(world);
    let first = asked_at(world, ASKED);
    assert_eq!(environments_of(&first).counts, counts(1, 1, 0, 0));
    for op in [
        StoreOp::PutBytes,
        StoreOp::PutArtifact,
        StoreOp::Attach,
        StoreOp::ClaimsOf,
        StoreOp::Name,
        StoreOp::Resolve,
        StoreOp::Read,
        StoreOp::Versions,
    ] {
        world
            .fakes
            .store
            .fault(op, Fault::Unreadable(WORDS.into()), Times::UntilHealed);
    }
    let writes = world.writes().len();

    let again = asked_at(world, ASKED + 60);
    let (before, after) = (
        &environments_of(&first).environments,
        &environments_of(&again).environments,
    );
    assert_eq!(after.len(), before.len());
    for (was, is) in before.iter().zip(after) {
        assert!(
            matches!(
                &was.runs,
                Ok(Runs {
                    read: Read::Source,
                    ..
                })
            ),
            "{} was first read as {:?}",
            was.environment,
            was.runs
        );
        let Ok(Runs {
            read: Read::Kept { at, said },
            ..
        }) = &is.runs
        else {
            panic!(
                "{} answered {:?} with the store unreachable, not as last fetched",
                is.environment, is.runs
            )
        };
        assert_eq!(
            *at, ASKED,
            "{} was last fetched at another moment",
            is.environment
        );
        assert_eq!(said, WORDS, "the store's words were not carried");
        assert_eq!(
            &read_from_source(is),
            was,
            "{} was shown otherwise from the copy",
            is.environment
        );
    }
    assert_eq!(
        environments_of(&again).counts,
        environments_of(&first).counts
    );
    for (part, refused) in [
        ("channels", matches!(again.channels, Err(Not::Refused(_)))),
        ("published", matches!(again.published, Err(Not::Refused(_)))),
    ] {
        assert!(
            !refused,
            "the {part} part was refused with the store unreachable"
        );
    }
    assert_eq!(world.writes().len(), writes, "status wrote to the store");
}

fn s12_the_reports_copy_deleted_the_environments_the_same(world: &FakeWorld) {
    one_behind_and_one_not(world);
    let before = asked(world);
    assert_eq!(environments_of(&before).counts, counts(1, 1, 0, 0));
    let copy = ReportsCopy::at(&world.history());
    assert!(copy.dir().exists(), "status kept no copy of the reports");
    fs::remove_dir_all(copy.dir()).expect("the copy is deleted");

    let deleted = asked(world);
    assert_eq!(
        deleted.environments, before.environments,
        "the environments answered otherwise with the reports' copy deleted"
    );
    assert!(
        copy.path(&environment(SRV), &channel(STABLE)).exists(),
        "the reports were not fetched into the copy again"
    );
}

fn one_environment_of_each_state_counted_on_the_part_and_the_row(world: &FakeWorld) {
    one_behind_and_one_not(world);
    accepting(world, &[BETA], None);
    accepting(world, &[DESK], Some(STABLE));
    declares(world, DESK, DESK, true, &[OPS]);
    declares(world, "lab", BETA, false, &[]);
    world.promote_fake(DESK, "0.1.0");
    reported(world, DESK, WHEN + 120);

    let answer = asked(world);
    let states: Vec<(&str, Result<State, &'static str>)> = environments_of(&answer)
        .environments
        .iter()
        .map(|e| {
            (
                e.environment.as_str(),
                e.state.as_ref().copied().map_err(|_| "unread"),
            )
        })
        .collect();
    assert_eq!(
        states,
        [
            (DESK, Ok(State::Waiting)),
            (EU, Ok(State::Converged)),
            ("lab", Err("unread")),
            (SRV, Ok(State::Behind)),
        ],
        "the environments are not one of each state, in the policy's order"
    );
    assert_eq!(environments_of(&answer).counts, counts(1, 1, 1, 1));
    let row = row(&world.repository(), &answer);
    assert_eq!(
        row.environments,
        Ok(counts(1, 1, 1, 1)),
        "the row's counts are not the part's"
    );
    let json = serde_json::to_value(&row).expect("a row serializes");
    for (key, n) in [
        ("converged", 1),
        ("behind", 1),
        ("waiting", 1),
        ("unreadable", 1),
    ] {
        assert_eq!(json["environments"]["value"][key], n, "{json}");
        assert_eq!(
            serde_json::to_value(&answer).expect("status serializes")["environments"]["value"][key],
            n
        );
    }
}

fn s11_each_environment_with_what_it_should_run_and_what_it_runs(world: &FakeWorld) {
    let [first, _, third] = one_behind_and_one_not(world);
    let answer = asked(world);
    let shown: Vec<(
        &str,
        &timber_seams::Answer<HeadAt>,
        Option<&Digest>,
        Option<State>,
    )> = environments_of(&answer)
        .environments
        .iter()
        .map(|e| {
            (
                e.environment.as_str(),
                &e.should,
                e.runs.as_ref().ok().map(|r| &r.index),
                e.state.as_ref().ok().copied(),
            )
        })
        .collect();
    assert_eq!(
        shown,
        [
            (
                EU,
                &Ok(at(&third)),
                Some(&third.index),
                Some(State::Converged)
            ),
            (
                SRV,
                &Ok(at(&third)),
                Some(&first.index),
                Some(State::Behind)
            ),
        ],
        "status does not name each environment with what it should run, what it runs \
         and whether they differ"
    );
    // 57: as a program reads it.
    let json = serde_json::to_value(&answer).expect("status serializes");
    for (n, (name, runs, state)) in [(EU, &third, "converged"), (SRV, &first, "behind")]
        .into_iter()
        .enumerate()
    {
        let shown = &json["environments"]["value"]["environments"][n];
        assert_eq!(shown["environment"], name, "{json}");
        assert_eq!(shown["should"]["value"]["version"], GROWN, "{json}");
        assert_eq!(
            shown["should"]["value"]["index"],
            third.index.as_str(),
            "{json}"
        );
        assert_eq!(
            shown["runs"]["value"]["version"],
            runs.version.as_str(),
            "{json}"
        );
        assert_eq!(
            shown["runs"]["value"]["index"],
            runs.index.as_str(),
            "{json}"
        );
        assert_eq!(shown["runs"]["value"]["converged_at"], CONVERGED, "{json}");
        assert_eq!(shown["state"]["value"], state, "{json}");
    }
    // 132: the row made from it.
    assert_eq!(
        row(&world.repository(), &answer).environments,
        Ok(counts(1, 1, 0, 0)),
        "the row's counts are not one running what it should and one behind"
    );
}
