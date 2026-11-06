//! The after group: what an apply reports and what a channel an `after`
//! makes does with it, over the fakes: a converged apply's report signed
//! by the host and attached beside the index, an unread report signing
//! nothing, an environment's newest report across a rollback, and S16
//! (11, 37, 41, 43, 83, 91, 97; design D6, D8 to D10).

use serde_json::json;
use timber_core::after::{follow, After, FollowAsk};
use timber_core::{
    apply, channel as chain, newest, Applied, ApplyAsk, Channel, Newest, Promoted, Published,
    Reported, To,
};
use timber_fakes::{FakeSigner, FakeTarget, SignerOp};
use timber_seams::{
    Answer, AskedOf, Audience, ChannelId, ChannelName, Claim, ClaimKind, ComponentName,
    DeclaredEnvironment, Digest, EnvironmentName, NamedSigner, Not, Required, RequiredClaim, Seams,
    SignerId, TargetProfile, TargetTable, Version, Why,
};

use crate::core::publish::{one, passing};
use crate::world::{FakeWorld, Write, GATE, HOST, STABLE, TARGET, WHEN};
use crate::{ok, Scenario};

pub const GROUP: &str = "after";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        Scenario {
            name: "a converged apply reports",
            plays: None,
            cites: &[11, 43, 91],
            run: a_converged_apply_reports,
        },
        Scenario {
            name: "an unread report signs nothing",
            plays: None,
            cites: &[43, 91],
            run: an_unread_report_signs_nothing,
        },
        Scenario {
            name: "the newest report across a rollback",
            plays: None,
            cites: &[43, 91],
            run: the_newest_report_across_a_rollback,
        },
        Scenario {
            name: "S16",
            plays: Some("S16"),
            cites: &[37, 41, 43, 83, 91, 97],
            run: s16,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}

/// The one component of the world, `repository`.
const COMPONENT: &str = "repository";

/// The environment the report scenarios apply, following `stable`.
const SRV: &str = "srv";

/// S16's manual environment, on `stable`, its own channel named for it.
const FIRST: &str = "first";

/// S16's environment after `first`, its own channel named for it.
const SECOND: &str = "second";

/// The one approver of `first`'s channel.
const DESK: &str = "desk";

/// When the fake target's apply converges.
const CONVERGED: &str = "2026-11-06T09:00:00Z";

fn environment(name: &str) -> EnvironmentName {
    EnvironmentName::new(name)
}

fn channel(name: &str) -> ChannelId {
    ChannelId::new(None, ChannelName::new(name))
}

fn component() -> ComponentName {
    ComponentName::new(COMPONENT)
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

/// A report of `name` by the host, as a channel's policy requires one.
fn report_of(name: &str) -> RequiredClaim {
    RequiredClaim {
        kind: ClaimKind::Report,
        check: None,
        environment: Some(environment(name)),
        signers: vec![reporter()],
        asked_of: AskedOf::Every,
    }
}

/// A channel's policy: the gate's acceptance, bound to `bound_to`,
/// promoted by `promoters` and asked by `approvers`.
fn bound(bound_to: &str, promoters: &[&str], approvers: &[&str]) -> Required {
    Required::Channel {
        claims: vec![RequiredClaim {
            kind: ClaimKind::Acceptance,
            check: None,
            environment: None,
            signers: vec![named(GATE, &[ClaimKind::Acceptance, ClaimKind::Promotion])],
            asked_of: AskedOf::Every,
        }],
        bound_to: Some(ChannelName::new(bound_to)),
        promoted_by: promoters
            .iter()
            .map(|p| named(p, &[ClaimKind::Promotion]))
            .collect(),
        approvers: approvers.iter().map(|a| named(a, &[])).collect(),
    }
}

/// The world's fake target, bound under [`TARGET`].
fn target(world: &FakeWorld) -> FakeTarget {
    world.fakes.targets[&TargetProfile::new(TARGET)].clone()
}

/// `name` declared at the head, following `channel` on the fake target,
/// moving on its own.
fn declares(world: &FakeWorld, name: &str, channel: ChannelId) {
    declared(world, name, channel, false, &[]);
}

/// `name` declared at the head, following `channel` on the fake target,
/// waiting where `waits` on the promotion `approvers` ask for (41, 97).
fn declared(world: &FakeWorld, name: &str, channel: ChannelId, waits: bool, approvers: &[&str]) {
    world.fakes.policy.declares_at(
        &world.head,
        DeclaredEnvironment {
            name: environment(name),
            channel,
            waits,
            approvers: approvers.iter().map(|a| SignerId::new(*a)).collect(),
            target: Some(TargetTable {
                profile: TargetProfile::new(TARGET),
                settings: json!({}),
            }),
        },
    );
}

/// The world's one component published at `start` as 0.1.0 and at
/// `head` as 0.2.0, each index carrying the gate's acceptance;
/// `stable`'s policy requiring it, and `stable` promoted by the gate to
/// 0.1.0 and then 0.2.0; the fake target converging at [`CONVERGED`].
/// The two indexes.
fn published(world: &FakeWorld) -> (Digest, Digest) {
    let mut indexes = Vec::new();
    for revision in [&world.start, &world.head] {
        passing(world, revision);
        let published = ok(one(world.publish(&world.ask(revision))), "the publish");
        let Published::Version { index, .. } = published else {
            panic!("{revision} was not published: {published:?}")
        };
        world.attach_as(GATE, &index, ClaimKind::Acceptance, revision, json!({}));
        indexes.push(index);
    }
    world.channel_policy(STABLE, false, None);
    for version in ["0.1.0", "0.2.0"] {
        moved(
            world.promote(&world.promote_ask(STABLE, to(version))),
            &format!("stable to {version}"),
        );
    }
    target(world).at(CONVERGED);
    (indexes[0].clone(), indexes[1].clone())
}

fn to(version: &str) -> To {
    To::Version(Version::new(version))
}

fn moved(answer: Answer<Promoted>, asked: &str) {
    assert!(
        matches!(answer, Ok(Promoted::Moved { .. })),
        "{asked} answered {answer:?}"
    );
}

/// `name` applied by the host at `at`.
fn applied(world: &FakeWorld, name: &str, at: u64) -> Answer<Applied> {
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
    )
}

/// `name`'s newest report, read across the chain of `channel`, counted
/// through the host's signer.
fn newest_of(world: &FakeWorld, name: &str, channel: &ChannelId) -> Answer<Newest> {
    newest(
        &world.seams(),
        &environment(name),
        &component(),
        channel,
        &report_of(name),
    )
}

/// The index `name`'s chain has its head at, none where it is empty.
fn head_of(world: &FakeWorld, name: &str) -> Option<Digest> {
    match chain(&world.fakes.keeper, &channel(name)) {
        Ok(Channel::Empty { .. }) => None,
        Ok(Channel::At { head, .. }) => Some(head.to.index),
        Err(not) => panic!("the chain of {name} answered {not:?}"),
    }
}

fn a_converged_apply_reports(world: &FakeWorld) {
    let (_, second) = published(world);
    declares(world, SRV, channel(STABLE));
    let signs = sign_calls(world);
    let writes = world.writes().len();
    let applied = ok(applied(world, SRV, WHEN), "the apply of srv");
    let written = world.writes()[writes..].to_vec();
    let envelope = match written.as_slice() {
        [Write::Attach { subject, envelope }] if *subject == second => envelope.clone(),
        other => panic!("the apply wrote other than one claim beside the index: {other:?}"),
    };
    assert_eq!(
        applied.report,
        Ok(envelope.digest()),
        "the apply answered another report than the one it attached"
    );
    assert_eq!(
        sign_calls(world),
        signs + 1,
        "the apply signed other than once"
    );
    let signers: Vec<&SignerId> = envelope.signatures.iter().map(|s| &s.signer).collect();
    assert_eq!(
        signers,
        [&SignerId::new(HOST)],
        "the report was not signed by the host's signer alone"
    );
    let claim = Claim::from_envelope(&envelope).expect("the report is a claim");
    assert_eq!(claim.kind, ClaimKind::Report);
    assert_eq!(
        claim.revision, world.head,
        "the report names another revision than the index's"
    );
    assert_eq!(
        Reported::of(&claim),
        Some(Reported {
            environment: environment(SRV),
            component: component(),
            channel: ChannelName::new(STABLE),
            version: Version::new("0.2.0"),
            index: second.clone(),
            runs: second.clone(),
            converged_at: CONVERGED.into(),
            at: WHEN,
        }),
        "the report says other than what the host observed"
    );
}

fn sign_calls(world: &FakeWorld) -> usize {
    world
        .fakes
        .signer
        .calls()
        .iter()
        .filter(|c| c.op == SignerOp::Sign)
        .count()
}

fn an_unread_report_signs_nothing(world: &FakeWorld) {
    published(world);
    declares(world, SRV, channel(STABLE));
    target(world).holds_no_marker();
    let signs = sign_calls(world);
    let writes = world.writes().len();
    let applied = ok(applied(world, SRV, WHEN), "the apply of srv");
    assert!(
        matches!(&applied.report, Err(Not::Unreadable(_))),
        "an unread report answered {:?}",
        applied.report
    );
    assert_eq!(
        applied.report.as_ref().err(),
        applied.running.as_ref().err(),
        "the report's answer is not the target's own"
    );
    assert_eq!(sign_calls(world), signs, "an unread report was signed");
    assert_eq!(
        world.writes()[writes..].to_vec(),
        Vec::<Write>::new(),
        "an unread report wrote to the store"
    );
}

fn the_newest_report_across_a_rollback(world: &FakeWorld) {
    let (first, second) = published(world);
    declares(world, SRV, channel(STABLE));
    let on_second = ok(applied(world, SRV, WHEN), "the apply of 0.2.0");
    moved(
        world.promote(&world.promote_ask(STABLE, To::Back)),
        "stable back",
    );
    let on_first = ok(applied(world, SRV, WHEN + 60), "the apply of 0.1.0");
    let newest = ok(
        newest_of(world, SRV, &channel(STABLE)),
        "srv's newest report",
    );
    assert_eq!(
        Ok(&newest.claim),
        on_first.report.as_ref(),
        "the newest report is not the one the rollback's apply made"
    );
    assert_eq!(
        (newest.report.version.as_str(), &newest.report.index),
        ("0.1.0", &first),
        "srv's newest report names another version than the one it went back to"
    );
    assert!(
        on_second.report.as_ref().is_ok_and(|d| *d != newest.claim),
        "the report of 0.2.0, on {second}, is answered newest"
    );
}

/// The fakes as the one binding with a signer holding `id` bound first,
/// as on a host that signs as `id`.
fn seams_as(world: &FakeWorld, id: &str) -> Seams {
    let mut seams = world.seams();
    let signer = FakeSigner::new();
    signer.hold(&SignerId::new(id));
    seams.signers.insert(0, Box::new(signer));
    seams
}

/// `first`'s channel moved `to`, asked by `desk`.
fn desk_moves_first(world: &FakeWorld, to: To, step: &str) {
    let ask = timber_core::PromoteAsk {
        signer: SignerId::new(DESK),
        ..world.promote_ask(FIRST, to)
    };
    moved(
        timber_core::promote(&seams_as(world, DESK), &world.history(), &ask),
        step,
    );
}

/// The gate's move on `first`'s report, the host asking at `at`; the
/// one channel's answer, which must name `second`.
fn follows_first(world: &FakeWorld, at: u64, step: &str) -> Answer<After> {
    let follows = ok(
        follow(
            &world.seams(),
            &world.history(),
            &FollowAsk {
                branch: world.main.clone(),
                component: None,
                environment: environment(FIRST),
                signer: SignerId::new(HOST),
                at,
            },
        ),
        step,
    );
    let channels: Vec<&ChannelId> = follows.channels.iter().map(|(c, _)| c).collect();
    assert_eq!(
        channels,
        [&channel(SECOND)],
        "{step}: the channels after first are not second alone"
    );
    follows.channels.into_iter().next().expect("second").1
}

/// 37, C.10: `second`'s channel empty or at the index of `first`'s
/// newest report, failing naming `step` and both indexes otherwise.
pub(crate) fn never_ahead(world: &FakeWorld, step: &str) {
    let Some(at) = head_of(world, SECOND) else {
        return;
    };
    let reported = newest_of(world, FIRST, &channel(FIRST))
        .ok()
        .map(|n| n.report.index);
    assert!(
        reported.as_ref() == Some(&at),
        "{step}: second is at {at}, and first last reported {}",
        reported.map_or_else(|| "nothing".to_string(), |d| d.to_string())
    );
}

/// The S16 world: [`published`], `first` declared manual on `stable` with
/// its own channel, which `desk` alone may ask to move, and `second`
/// declared after `first` with its own channel, bound to `first`'s,
/// promoted by the host and requiring a report of `first` by it.
fn s16_world(world: &FakeWorld) -> (Digest, Digest) {
    let indexes = published(world);
    world.fakes.policy.require(
        Audience::Channel(channel(FIRST)),
        bound(STABLE, &[DESK], &[DESK]),
    );
    world.fakes.policy.after(
        &channel(SECOND),
        &[environment(FIRST)],
        &[reporter()],
        bound(FIRST, &[HOST], &[]),
    );
    declared(world, FIRST, channel(FIRST), true, &[DESK]);
    declares(world, SECOND, channel(SECOND));
    indexes
}

fn s16(world: &FakeWorld) {
    let (first, second) = s16_world(world);
    desk_moves_first(
        world,
        to("0.1.0"),
        "first to 0.1.0, which it holds before S16",
    );
    never_ahead(world, "before step 1");

    // 1: desk asks for first to move to 0.2.0, and second stays empty.
    desk_moves_first(world, to("0.2.0"), "step 1, desk's asking");
    assert_eq!(
        head_of(world, FIRST),
        Some(second.clone()),
        "step 1: first did not move"
    );
    assert_eq!(
        head_of(world, SECOND),
        None,
        "step 1: desk's asking moved second"
    );
    never_ahead(world, "step 1");

    // 2: first is applied and reports 0.2.0, and second follows.
    let reported = ok(
        applied(world, FIRST, WHEN + 60),
        "step 2, the apply of first",
    );
    let report = ok(reported.report, "step 2, first's report");
    let answer = follows_first(world, WHEN + 60, "step 2, the gate's move");
    let Ok(After::Promoted(Promoted::Moved { to, .. })) = &answer else {
        panic!("step 2: second answered {answer:?}, not moved")
    };
    assert_eq!(to.index, second, "step 2: second moved to another index");
    assert_eq!(head_of(world, SECOND), Some(second.clone()));
    let promotion = chain(&world.fakes.keeper, &channel(SECOND));
    let Ok(Channel::At { head, .. }) = promotion else {
        panic!("step 2: second's chain answered {promotion:?}")
    };
    assert_eq!(
        head.asked_by,
        SignerId::new(HOST),
        "step 2: second's move names another asker than the gate's own signer"
    );
    assert!(
        head.policy.claims.contains(&report),
        "step 2: first's report is not among the claims second's move counted"
    );
    never_ahead(world, "step 2");

    // 3: first is moved back, applied and reports 0.1.0, and second
    // follows back.
    desk_moves_first(world, To::Back, "step 3, desk's asking back");
    never_ahead(world, "step 3, first moved back");
    ok(
        applied(world, FIRST, WHEN + 120),
        "step 3, the apply of first",
    );
    let answer = follows_first(world, WHEN + 120, "step 3, the gate's move");
    let Ok(After::Promoted(Promoted::Moved { to, from, .. })) = &answer else {
        panic!("step 3: second answered {answer:?}, not moved")
    };
    assert_eq!(to.index, first, "step 3: second moved to another index");
    assert_eq!(
        from.as_ref().map(|f| &f.index),
        Some(&second),
        "step 3: second moved from another index"
    );
    never_ahead(world, "step 3");
    let moves = match chain(&world.fakes.keeper, &channel(SECOND)) {
        Ok(Channel::At { chain, .. }) => chain.len(),
        other => panic!("step 3: second's chain answered {other:?}"),
    };
    assert_eq!(
        moves, 2,
        "step 3: second's chain holds other than its two moves"
    );

    // 4: the host, second's own promoter, asks for second to move to
    // 0.2.0, whose index carries first's older report, and is refused:
    // the gate alone moves second, on first's reports (rule core.72).
    let writes = world.writes().len();
    let signs = sign_calls(world);
    let ask = timber_core::PromoteAsk {
        signer: SignerId::new(HOST),
        ..world.promote_ask(SECOND, To::Version(Version::new("0.2.0")))
    };
    let answer = timber_core::promote(&world.seams(), &world.history(), &ask);
    let Err(Not::Refused(refusal)) = &answer else {
        panic!("step 4: asking second to 0.2.0 answered {answer:?}, not refused")
    };
    assert_eq!(
        refusal.why,
        Why::MovesOnReports {
            channel: channel(SECOND),
            after: vec![environment(FIRST)],
        },
        "step 4: the refusal names other than second and first"
    );
    assert_eq!(
        (world.writes().len(), sign_calls(world)),
        (writes, signs),
        "step 4: an asking refused signed or wrote"
    );
    assert_eq!(
        head_of(world, SECOND),
        Some(first.clone()),
        "step 4: second moved at an asking"
    );
    never_ahead(world, "step 4");
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use super::*;

    /// S16 fails naming the step and both indexes where second is moved
    /// to a version first has not reported.
    #[test]
    fn a_move_ahead_of_firsts_report_fails_naming_the_step_and_the_two_indexes() {
        let world = FakeWorld::new();
        let (first, second) = s16_world(&world);
        desk_moves_first(&world, to("0.1.0"), "first to 0.1.0");
        ok(applied(&world, FIRST, WHEN), "the apply of first");
        world.promote_fake(SECOND, "0.2.0");
        let failed = catch_unwind(AssertUnwindSafe(|| never_ahead(&world, "step 2")))
            .expect_err("second ahead of first's report passed");
        let why = failed
            .downcast_ref::<String>()
            .cloned()
            .expect("the failure's words");
        assert!(why.starts_with("step 2: "), "{why}");
        assert!(why.contains(second.as_str()), "{why}");
        assert!(why.contains(first.as_str()), "{why}");
    }
}
