//! Spec after-channels: each channel an `after` makes moved to what the
//! environments it comes after last reported, over the fakes, two
//! versions of `lib` published in the test, each carrying the gate's
//! acceptance, and `stable` promoted between them (design D9).

mod common;
mod world;

use serde_json::{json, Value};
use timber_core::after::{follow, After, FollowAsk, Follows};
use timber_core::{promote, Newest, PromoteAsk, Promoted, Published, Reported, To};
use timber_fakes::{FakeSigner, KeeperOp, SignerOp, StoreOp};
use timber_seams::{
    Answer, AskedOf, Audience, Branch, ChannelId, ChannelName, Claim, ClaimKind, ComponentName,
    DeclaredEnvironment, Digest, EnvironmentName, Keeper, NamedSigner, Not, Promotion, Required,
    RequiredClaim, Signer, SignerId, Store, Version, Why,
};
use world::{version_of, World};

/// The identity the host signs promotions as, which the policy names
/// for acceptance and promotion (108).
const GATE: &str = "gate";

/// The signer each channel an `after` makes lets claim a report.
const DEPLOYER: &str = "deployer";

/// 2026-01-01T00:00:00Z, the moment the first promotion is asked at.
const AT: u64 = 1_767_225_600;

const HOUR: u64 = 3_600;

fn named(id: &str, kinds: &[ClaimKind]) -> NamedSigner {
    NamedSigner {
        id: SignerId::new(id),
        verified_by: format!("{id}'s key"),
        kinds: kinds.to_vec(),
        checks: vec![],
    }
}

fn wanted(kind: ClaimKind, signers: Vec<NamedSigner>) -> RequiredClaim {
    RequiredClaim {
        kind,
        check: None,
        environment: None,
        signers,
        asked_of: AskedOf::Every,
    }
}

fn gate() -> NamedSigner {
    named(GATE, &[ClaimKind::Acceptance, ClaimKind::Promotion])
}

fn deployer() -> NamedSigner {
    named(DEPLOYER, &[ClaimKind::Report])
}

fn lib() -> ComponentName {
    ComponentName::new("lib")
}

fn of(channel: &str) -> ChannelId {
    ChannelId::new(Some(lib()), ChannelName::new(channel))
}

fn env(name: &str) -> EnvironmentName {
    EnvironmentName::new(name)
}

/// A channel's policy: the gate's acceptance then `more`, bound to
/// `stable`, promoted by `promoters`, asked by `approvers`.
fn bound(more: Vec<RequiredClaim>, promoters: &[NamedSigner], approvers: &[&str]) -> Required {
    Required::Channel {
        claims: std::iter::once(wanted(ClaimKind::Acceptance, vec![gate()]))
            .chain(more)
            .collect(),
        bound_to: Some(ChannelName::new("stable")),
        promoted_by: promoters.to_vec(),
        approvers: approvers.iter().map(|a| named(a, &[])).collect(),
    }
}

/// Two published versions of `lib` and their indexes.
struct Two {
    world: World,
    first: Digest,
    second: Digest,
}

/// `lib` published at `start` as 0.1.0 and at the head as 0.2.0, each
/// index carrying the gate's acceptance; `stable` requiring it and
/// promoted by the gate; `srv` declared at the head, following `stable`;
/// and the host's signer holding the gate's identity.
fn two() -> Two {
    let world = World::new();
    world.fakes.landing.land(&world.main, &world.start);
    for o in ["lib-pack", "lib-docs"] {
        world.fakes.build.made(
            &world.start,
            &timber_seams::OutputName::new(o),
            timber_seams::Report {
                build: world::build_of(o, &world.start),
                outcome: timber_seams::Outcome::Pass,
                files: vec![format!("dist/{o}.tar")],
            },
        );
    }
    let at_start = timber_core::Ask {
        revision: world.start.clone(),
        ..world.ask_for("lib")
    };
    let first = index_of(&world.publish_one(&at_start).unwrap(), "0.1.0");
    let second = index_of(&world.publish_one(&world.ask_for("lib")).unwrap(), "0.2.0");
    for index in [&first, &second] {
        attach(&world, index, GATE, ClaimKind::Acceptance, json!({}));
    }
    world.fakes.signer.hold(&SignerId::new(GATE));
    world.fakes.policy.require(
        Audience::Channel(of("stable")),
        Required::Channel {
            claims: vec![wanted(ClaimKind::Acceptance, vec![gate()])],
            bound_to: None,
            promoted_by: vec![gate()],
            approvers: vec![],
        },
    );
    declares(&world, "srv");
    Two {
        world,
        first,
        second,
    }
}

/// `name` declared at the head, following `stable`, with no target.
fn declares(world: &World, name: &str) {
    world.fakes.policy.declares_at(
        &world.head,
        DeclaredEnvironment {
            name: env(name),
            channel: of("stable"),
            waits: false,
            approvers: vec![],
            target: None,
        },
    );
}

/// `channel` made by an `after` naming `environments`, each report by
/// the deployer, its policy `required`.
fn after(world: &World, channel: &str, environments: &[&str], required: Required) {
    let environments: Vec<EnvironmentName> = environments.iter().map(|e| env(e)).collect();
    world
        .fakes
        .policy
        .after(&of(channel), &environments, &[deployer()], required);
}

fn index_of(published: &Published, version: &str) -> Digest {
    let (v, index) = version_of(published);
    assert_eq!(v.as_str(), version);
    index.clone()
}

/// A claim of `kind` about `index` by `signer`, attached to it.
fn attach(world: &World, index: &Digest, signer: &str, kind: ClaimKind, body: Value) {
    let theirs = FakeSigner::new();
    let id = SignerId::new(signer);
    theirs.hold(&id);
    let claim = Claim {
        subject: index.clone(),
        kind,
        revision: world.head.clone(),
        build: None,
        body,
    };
    let envelope = theirs.sign(&claim, &id).unwrap();
    world.fakes.store.attach(index, &envelope).unwrap();
}

/// `stable` moved to `to` at `at`.
fn move_to(world: &World, to: To, at: u64) {
    let promoted = promote(
        &world.fakes.seams(),
        &world.history(),
        &PromoteAsk {
            branch: Branch::new("main"),
            component: Some(lib()),
            channel: ChannelName::new("stable"),
            to,
            from: None,
            signer: SignerId::new(GATE),
            at,
        },
    );
    assert!(promoted.is_ok(), "{promoted:?}");
}

fn to(version: &str) -> To {
    To::Version(Version::new(version))
}

/// A report of `environment` of `lib` running `version` at `index`, from
/// an apply at `at`, signed by `signer` and attached to the index.
fn report(
    world: &World,
    signer: &str,
    environment: &str,
    (version, index): (&str, &Digest),
    at: u64,
) -> Newest {
    let theirs = FakeSigner::new();
    let id = SignerId::new(signer);
    theirs.hold(&id);
    let report = Reported {
        environment: env(environment),
        component: lib(),
        channel: ChannelName::new("stable"),
        version: Version::new(version),
        index: index.clone(),
        runs: index.clone(),
        converged_at: "2025-12-31T00:00:00Z".into(),
        at,
    };
    let envelope = theirs.sign(&report.claim(world.head.clone()), &id).unwrap();
    world.fakes.store.attach(index, &envelope).unwrap();
    Newest {
        claim: envelope.digest(),
        report,
    }
}

/// The gate's move on `environment`'s reports, asked at `at`.
fn followed(world: &World, environment: &str, at: u64) -> Answer<Follows> {
    follow(
        &world.fakes.seams(),
        &world.history(),
        &FollowAsk {
            branch: Branch::new("main"),
            component: Some(lib()),
            environment: env(environment),
            signer: SignerId::new(GATE),
            at,
        },
    )
}

/// The one channel's answer of `follows`, which must name `channel`.
fn only(follows: Answer<Follows>, channel: &str) -> Answer<After> {
    let follows = follows.expect("follow answered");
    assert_eq!(follows.channels.len(), 1, "{follows:?}");
    let (id, answer) = follows.channels.into_iter().next().unwrap();
    assert_eq!(id, of(channel));
    answer
}

/// The version and index a move answered `Moved` went to.
fn moved_to(answer: &Answer<After>) -> (Version, Digest) {
    match answer {
        Ok(After::Promoted(Promoted::Moved { to, .. })) => (to.version.clone(), to.index.clone()),
        other => panic!("answered {other:?}, not moved"),
    }
}

fn chain_of(world: &World, channel: &ChannelId) -> Vec<Promotion> {
    match world.fakes.keeper.chain(channel) {
        Ok(chain) => chain
            .claims
            .iter()
            .map(|e| Promotion::of(&Claim::from_envelope(e).unwrap()).unwrap())
            .collect(),
        Err(Not::Absent(_)) => vec![],
        Err(not) => panic!("the chain of {channel} answered {not:?}"),
    }
}

fn refused(answer: Answer<After>) -> Why {
    match answer {
        Err(Not::Refused(r)) => r.why,
        other => panic!("answered {other:?}, not refused"),
    }
}

/// The appends and signatures made since `before`, the counts of each
/// fake's calls.
fn written_since(world: &World, before: (usize, usize)) -> (usize, usize) {
    (
        world.fakes.keeper.calls()[before.0..]
            .iter()
            .filter(|c| c.op == KeeperOp::Append)
            .count(),
        world.fakes.signer.calls()[before.1..]
            .iter()
            .filter(|c| c.op == SignerOp::Sign)
            .count(),
    )
}

fn counts(world: &World) -> (usize, usize) {
    (
        world.fakes.keeper.calls().len(),
        world.fakes.signer.calls().len(),
    )
}

// Requirement: A channel an after makes moves to what its environments
// last reported

#[test]
fn second_follows_srv_forward_to_each_version_srv_last_reported() {
    let Two {
        world,
        first,
        second,
    } = two();
    after(&world, "second", &["srv"], bound(vec![], &[gate()], &[]));
    move_to(&world, to("0.1.0"), AT);
    report(&world, DEPLOYER, "srv", ("0.1.0", &first), AT + HOUR);
    let answer = only(followed(&world, "srv", AT + HOUR), "second");
    assert_eq!(moved_to(&answer), (Version::new("0.1.0"), first.clone()));

    move_to(&world, to("0.2.0"), AT + 2 * HOUR);
    report(&world, DEPLOYER, "srv", ("0.2.0", &second), AT + 3 * HOUR);
    let answer = only(followed(&world, "srv", AT + 3 * HOUR), "second");
    let Ok(After::Promoted(Promoted::Moved { to, from, .. })) = &answer else {
        panic!("answered {answer:?}, not moved")
    };
    assert_eq!(to.index, second);
    assert_eq!(from.as_ref().map(|f| &f.index), Some(&first));
    let chain = chain_of(&world, &of("second"));
    let indexes: Vec<&Digest> = chain.iter().map(|p| &p.to.index).collect();
    assert_eq!(indexes, [&first, &second]);
    assert!(chain[1].predecessor.is_some());
}

#[test]
fn a_channel_at_what_its_environments_last_reported_is_there_and_nothing_is_appended() {
    let Two { world, first, .. } = two();
    after(&world, "second", &["srv"], bound(vec![], &[gate()], &[]));
    move_to(&world, to("0.1.0"), AT);
    report(&world, DEPLOYER, "srv", ("0.1.0", &first), AT + HOUR);
    assert_eq!(
        moved_to(&only(followed(&world, "srv", AT + HOUR), "second")).1,
        first
    );
    let before = counts(&world);
    let answer = only(followed(&world, "srv", AT + 2 * HOUR), "second");
    assert!(
        matches!(&answer, Ok(After::Promoted(Promoted::There { at, .. })) if at.index == first),
        "{answer:?}"
    );
    assert_eq!(written_since(&world, before), (0, 0));
}

#[test]
fn no_channel_after_the_environment_answers_none_and_reads_no_report() {
    let Two { world, first, .. } = two();
    after(&world, "second", &["eu"], bound(vec![], &[gate()], &[]));
    move_to(&world, to("0.1.0"), AT);
    report(&world, DEPLOYER, "srv", ("0.1.0", &first), AT + HOUR);
    let before = world.fakes.store.calls().len();
    assert_eq!(
        followed(&world, "srv", AT + HOUR),
        Ok(Follows { channels: vec![] })
    );
    assert!(
        world.fakes.store.calls()[before..]
            .iter()
            .all(|c| c.op != StoreOp::ClaimsOf),
        "a report was read"
    );
}

// Requirement: The channel follows its environments back

#[test]
fn second_follows_srv_back_with_a_promotion_naming_its_predecessor() {
    let Two {
        world,
        first,
        second,
    } = two();
    after(&world, "second", &["srv"], bound(vec![], &[gate()], &[]));
    move_to(&world, to("0.1.0"), AT);
    move_to(&world, to("0.2.0"), AT + HOUR);
    report(&world, DEPLOYER, "srv", ("0.2.0", &second), AT + 2 * HOUR);
    moved_to(&only(followed(&world, "srv", AT + 2 * HOUR), "second"));

    move_to(&world, To::Back, AT + 3 * HOUR);
    report(&world, DEPLOYER, "srv", ("0.1.0", &first), AT + 4 * HOUR);
    let answer = only(followed(&world, "srv", AT + 4 * HOUR), "second");
    let Ok(After::Promoted(Promoted::Moved {
        to,
        from,
        predecessor,
        ..
    })) = &answer
    else {
        panic!("answered {answer:?}, not moved")
    };
    assert_eq!(to.index, first);
    assert_eq!(from.as_ref().map(|f| &f.index), Some(&second));
    let chain = world.fakes.keeper.chain(&of("second")).unwrap().claims;
    assert_eq!(chain.len(), 2, "second's chain holds other than both moves");
    assert_eq!(predecessor.as_ref(), Some(&chain[0].digest()));
}

// Requirement: The channel is never moved ahead of what was last
// reported

#[test]
fn an_older_report_of_a_newer_version_never_carries_the_channel_ahead() {
    let Two {
        world,
        first,
        second,
    } = two();
    after(&world, "second", &["srv"], bound(vec![], &[gate()], &[]));
    move_to(&world, to("0.1.0"), AT);
    move_to(&world, to("0.2.0"), AT + HOUR);
    report(&world, DEPLOYER, "srv", ("0.2.0", &second), AT + 2 * HOUR);
    move_to(&world, To::Back, AT + 3 * HOUR);
    report(&world, DEPLOYER, "srv", ("0.1.0", &first), AT + 4 * HOUR);
    let answer = only(followed(&world, "srv", AT + 4 * HOUR), "second");
    assert_eq!(moved_to(&answer), (Version::new("0.1.0"), first.clone()));
    let chain = chain_of(&world, &of("second"));
    assert!(
        chain.iter().all(|p| p.to.index != second),
        "second was moved to 0.2.0, which srv last reported before 0.1.0"
    );
}

// Requirement: A channel whose environments disagree is held

#[test]
fn two_environments_that_disagree_hold_the_channel_naming_each_and_nothing_is_written() {
    let Two {
        world,
        first,
        second,
    } = two();
    declares(&world, "us");
    declares(&world, "asia");
    after(&world, "eu", &["us", "asia"], bound(vec![], &[gate()], &[]));
    move_to(&world, to("0.1.0"), AT);
    move_to(&world, to("0.2.0"), AT + HOUR);
    let us = report(&world, DEPLOYER, "us", ("0.2.0", &second), AT + 2 * HOUR);
    let asia = report(&world, DEPLOYER, "asia", ("0.1.0", &first), AT + 2 * HOUR);
    let before = counts(&world);
    let answer = only(followed(&world, "us", AT + 3 * HOUR), "eu");
    assert_eq!(
        answer,
        Ok(After::Held {
            reports: vec![(env("us"), Ok(us)), (env("asia"), Ok(asia))],
        })
    );
    assert_eq!(written_since(&world, before), (0, 0));
    assert!(chain_of(&world, &of("eu")).is_empty());
}

#[test]
fn an_environment_with_no_report_holds_the_channel_naming_its_absent_answer() {
    let Two { world, second, .. } = two();
    declares(&world, "us");
    declares(&world, "asia");
    after(&world, "eu", &["us", "asia"], bound(vec![], &[gate()], &[]));
    move_to(&world, to("0.2.0"), AT);
    let us = report(&world, DEPLOYER, "us", ("0.2.0", &second), AT + HOUR);
    let answer = only(followed(&world, "us", AT + 2 * HOUR), "eu");
    let Ok(After::Held { reports }) = answer else {
        panic!("answered {answer:?}, not held")
    };
    assert_eq!(reports[0], (env("us"), Ok(us)));
    assert_eq!(reports[1].0, env("asia"));
    assert!(
        matches!(&reports[1].1, Err(Not::Absent(a)) if a.what.contains("environment asia of lib")),
        "{:?}",
        reports[1].1
    );
    assert!(chain_of(&world, &of("eu")).is_empty());
}

// Requirement: The move is judged under the channel's own policy

#[test]
fn a_report_by_a_signer_that_may_not_report_counts_for_nothing_and_holds_the_channel() {
    let Two { world, first, .. } = two();
    after(&world, "second", &["srv"], bound(vec![], &[gate()], &[]));
    move_to(&world, to("0.1.0"), AT);
    report(&world, "desk", "srv", ("0.1.0", &first), AT + HOUR);
    let answer = only(followed(&world, "srv", AT + HOUR), "second");
    let Ok(After::Held { reports }) = answer else {
        panic!("answered {answer:?}, not held")
    };
    assert!(matches!(&reports[..], [(e, Err(Not::Absent(_)))] if *e == env("srv")));
    assert!(chain_of(&world, &of("second")).is_empty());
}

#[test]
fn a_claim_the_channels_line_wants_refuses_it_naming_the_claim_and_nothing_is_written() {
    let Two { world, first, .. } = two();
    let review = wanted(ClaimKind::Review, vec![named("desk", &[ClaimKind::Review])]);
    after(
        &world,
        "second",
        &["srv"],
        bound(vec![review], &[gate()], &[]),
    );
    move_to(&world, to("0.1.0"), AT);
    report(&world, DEPLOYER, "srv", ("0.1.0", &first), AT + HOUR);
    let before = counts(&world);
    assert_eq!(
        refused(only(followed(&world, "srv", AT + HOUR), "second")),
        Why::Wanting {
            kind: ClaimKind::Review,
            check: None,
            environment: None,
            signers: vec![SignerId::new("desk")],
        }
    );
    assert_eq!(written_since(&world, before), (0, 0));
    assert!(chain_of(&world, &of("second")).is_empty());
}

// Requirement: The move is signed by one of the channel's promoters

#[test]
fn a_host_that_is_not_a_promoter_is_refused_naming_both_and_reads_no_report() {
    let Two { world, first, .. } = two();
    after(
        &world,
        "second",
        &["srv"],
        bound(vec![], &[named(DEPLOYER, &[ClaimKind::Promotion])], &[]),
    );
    move_to(&world, to("0.1.0"), AT);
    report(&world, DEPLOYER, "srv", ("0.1.0", &first), AT + HOUR);
    let before = (counts(&world), world.fakes.store.calls().len());
    let why = refused(only(followed(&world, "srv", AT + HOUR), "second"));
    assert_eq!(
        why,
        Why::NotPromoter {
            channel: of("second"),
            signer: SignerId::new(GATE),
            promoters: vec![SignerId::new(DEPLOYER)],
        }
    );
    assert_eq!(
        why.to_string(),
        "gate may not promote lib/second; its promoters are deployer"
    );
    assert_eq!(written_since(&world, before.0), (0, 0));
    assert!(
        world.fakes.store.calls()[before.1..]
            .iter()
            .all(|c| c.op != StoreOp::ClaimsOf),
        "a report was read"
    );
}

// Requirement: The move is the gate's act, not an asking

#[test]
fn the_move_names_the_hosts_signer_as_who_asked_whatever_approvers_and_counts_the_report() {
    let Two { world, first, .. } = two();
    after(
        &world,
        "second",
        &["srv"],
        bound(vec![], &[gate()], &["desk"]),
    );
    move_to(&world, to("0.1.0"), AT);
    let srv = report(&world, DEPLOYER, "srv", ("0.1.0", &first), AT + HOUR);
    moved_to(&only(followed(&world, "srv", AT + 2 * HOUR), "second"));
    let chain = chain_of(&world, &of("second"));
    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0].asked_by, SignerId::new(GATE));
    assert_eq!(chain[0].at, AT + 2 * HOUR);
    assert!(
        chain[0].policy.claims.contains(&srv.claim),
        "srv's report is not among the claims counted: {:?}",
        chain[0].policy.claims
    );
}

// Requirement: Each channel is answered on its own

#[test]
fn two_channels_after_srv_one_moves_and_the_other_is_refused_naming_the_review() {
    let Two { world, first, .. } = two();
    let review = wanted(ClaimKind::Review, vec![named("desk", &[ClaimKind::Review])]);
    after(&world, "second", &["srv"], bound(vec![], &[gate()], &[]));
    after(
        &world,
        "third",
        &["srv"],
        bound(vec![review], &[gate()], &[]),
    );
    move_to(&world, to("0.1.0"), AT);
    report(&world, DEPLOYER, "srv", ("0.1.0", &first), AT + HOUR);
    let follows = followed(&world, "srv", AT + HOUR).unwrap();
    let channels: Vec<&ChannelId> = follows.channels.iter().map(|(c, _)| c).collect();
    assert_eq!(channels, [&of("second"), &of("third")]);
    assert_eq!(moved_to(&follows.channels[0].1).1, first);
    assert!(
        matches!(
            &follows.channels[1].1,
            Err(Not::Refused(r)) if matches!(r.why, Why::Wanting { kind: ClaimKind::Review, .. })
        ),
        "{:?}",
        follows.channels[1].1
    );
    assert_eq!(chain_of(&world, &of("second")).len(), 1);
    assert!(chain_of(&world, &of("third")).is_empty());
}

// Requirement: Apply's answer carries the report and the moves after it

#[test]
fn each_channel_and_each_held_report_is_in_the_one_form_naming_it() {
    let Two { world, second, .. } = two();
    declares(&world, "us");
    declares(&world, "asia");
    after(&world, "eu", &["us", "asia"], bound(vec![], &[gate()], &[]));
    move_to(&world, to("0.2.0"), AT);
    let us = report(&world, DEPLOYER, "us", ("0.2.0", &second), AT + HOUR);
    let follows = followed(&world, "us", AT + HOUR).unwrap();
    let value = serde_json::to_value(&follows).unwrap();
    let channel = &value["channels"][0];
    assert_eq!(channel["channel"], json!(of("eu")));
    assert_eq!(channel["answer"], "ok");
    assert_eq!(channel["value"]["after"], "held");
    let reports = &channel["value"]["reports"];
    assert_eq!(reports[0]["environment"], "us");
    assert_eq!(reports[0]["answer"], "ok");
    assert_eq!(reports[0]["value"]["claim"], json!(us.claim));
    assert_eq!(reports[1]["environment"], "asia");
    assert_eq!(reports[1]["answer"], "absent");
}

#[test]
fn a_report_whose_version_resolves_to_another_index_is_unreadable_naming_both() {
    let Two {
        world,
        first,
        second,
    } = two();
    after(&world, "second", &["srv"], bound(vec![], &[gate()], &[]));
    move_to(&world, to("0.1.0"), AT);
    move_to(&world, to("0.2.0"), AT + HOUR);
    report(&world, DEPLOYER, "srv", ("0.1.0", &second), AT + 2 * HOUR);
    let before = counts(&world);
    match only(followed(&world, "srv", AT + 2 * HOUR), "second") {
        Err(Not::Unreadable(u)) => {
            let said = u.said.expect("the store's resolve named");
            assert!(said.contains(first.as_str()), "{said}");
            assert!(said.contains(second.as_str()), "{said}");
        }
        other => panic!("answered {other:?}, not unreadable"),
    }
    assert_eq!(written_since(&world, before), (0, 0));
}
