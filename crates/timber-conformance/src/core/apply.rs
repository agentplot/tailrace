//! The apply group: one environment converged on its channel's head over
//! the fakes, a scenario for each requirement of environment-apply: the
//! head applied once its claims are judged, a wanting claim, an empty
//! channel, an unbound profile, and an environment already at the head
//! (31, 40, 42, 88 to 90, 92, 136, rule core.29; design D5).

use std::cell::Cell;
use std::rc::Rc;

use serde_json::json;
use timber_core::{apply, Applied, ApplyAsk, Promoted, Published, To};
use timber_fakes::{FakeStore, FakeTarget, StoreOp, TargetOp};
use timber_seams::{
    Answer, AskedOf, Audience, ChannelId, ChannelName, ClaimKind, ComponentName,
    DeclaredEnvironment, Digest, Environment, EnvironmentName, Head, NamedSigner, Not, Required,
    RequiredClaim, Running, Seam, SignerId, Target, TargetProfile, TargetTable, Verified, Version,
    Why,
};

use crate::core::publish::{one, passing};
use crate::world::{FakeWorld, GATE, REBUILDER, STABLE, TARGET};
use crate::{ok, Scenario};

pub const GROUP: &str = "apply";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        Scenario {
            name: "the head applied after its claims are judged",
            plays: None,
            cites: &[40, 42, 88, 90, 136],
            run: the_head_applied_after_its_claims_are_judged,
        },
        Scenario {
            name: "a wanting claim stops the apply",
            plays: None,
            cites: &[42, 58],
            run: a_wanting_claim_stops_the_apply,
        },
        Scenario {
            name: "an empty channel",
            plays: None,
            cites: &[31, 88],
            run: an_empty_channel,
        },
        Scenario {
            name: "an unbound profile",
            plays: None,
            cites: &[92, 136],
            run: an_unbound_profile,
        },
        Scenario {
            name: "an environment already at the head",
            plays: None,
            cites: &[40, 89, 90],
            run: an_environment_already_at_the_head,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}

/// The one component of the world, `repository`.
const COMPONENT: &str = "repository";

/// The environment the group applies.
const CACHE: &str = "cache";

/// When the fake target's apply converges.
const CONVERGED: &str = "2026-11-05T09:00:00Z";

fn cache() -> EnvironmentName {
    EnvironmentName::new(CACHE)
}

fn channel(name: &str) -> ChannelId {
    ChannelId::new(None, ChannelName::new(name))
}

/// The target's own settings, as the environment's table writes them
/// (45, C.10).
fn settings() -> serde_json::Value {
    json!({ "at": "a place of its own" })
}

/// `cache` following `channel`, its target the profile `profile` names
/// with the settings.
fn cache_on(channel: ChannelId, profile: &str) -> DeclaredEnvironment {
    DeclaredEnvironment {
        name: cache(),
        channel,
        waits: false,
        approvers: vec![],
        target: Some(TargetTable {
            profile: TargetProfile::new(profile),
            settings: settings(),
        }),
    }
}

/// The world's fake target, bound under [`TARGET`].
fn target(world: &FakeWorld) -> FakeTarget {
    world.fakes.targets[&TargetProfile::new(TARGET)].clone()
}

/// The world's one component published at `start` as 0.1.0 and at
/// `head` as 0.2.0, each index carrying the gate's acceptance and the
/// rebuild signer's matching rebuild; `stable`'s policy requiring both,
/// and `stable` promoted to 0.2.0 by the gate; `cache` declared at the
/// head following `stable` on the fake target, which runs 0.1.0. The two
/// indexes.
fn applying(world: &FakeWorld) -> (Digest, Digest) {
    let mut indexes = Vec::new();
    for revision in [&world.start, &world.head] {
        passing(world, revision);
        let published = ok(one(world.publish(&world.ask(revision))), "the publish");
        let Published::Version { index, .. } = published else {
            panic!("{revision} was not published: {published:?}")
        };
        world.attach_as(GATE, &index, ClaimKind::Acceptance, revision, json!({}));
        world.attach_as(
            REBUILDER,
            &index,
            ClaimKind::Rebuild,
            revision,
            json!({ "matched": true }),
        );
        indexes.push(index);
    }
    world.channel_policy(STABLE, true, None);
    let promoted = world.promote(&world.promote_ask(STABLE, To::Version(Version::new("0.2.0"))));
    assert!(
        matches!(promoted, Ok(Promoted::Moved { .. })),
        "stable to 0.2.0 answered {promoted:?}"
    );
    world
        .fakes
        .policy
        .declares_at(&world.head, cache_on(channel(STABLE), TARGET));
    let target = target(world);
    target.runs(
        &cache(),
        Running {
            digest: indexes[0].clone(),
            converged_at: "2026-01-01T00:00:00Z".into(),
        },
    );
    target.at(CONVERGED);
    (indexes[0].clone(), indexes[1].clone())
}

fn ask(world: &FakeWorld) -> ApplyAsk {
    ApplyAsk {
        branch: world.main.clone(),
        component: None,
        environment: cache(),
    }
}

fn applied(world: &FakeWorld) -> Answer<Applied> {
    apply(&world.seams(), &world.history(), &ask(world))
}

/// The operations the fake target was asked, in order.
fn ops(target: &FakeTarget) -> Vec<TargetOp> {
    target.calls().into_iter().map(|c| c.op).collect()
}

fn absent(answer: Answer<Applied>, asked: &str) -> timber_seams::Absent {
    match answer {
        Err(Not::Absent(a)) => a,
        other => panic!("{asked} answered {other:?}, not absent"),
    }
}

/// A target that, at an apply, records how many operations the store
/// had been asked, then asks the fake target.
struct Watched {
    inner: FakeTarget,
    store: FakeStore,
    at: Rc<Cell<Option<usize>>>,
}

impl Target for Watched {
    fn desired(&self, environment: &Environment, head: Head<'_>) -> Answer<Digest> {
        self.inner.desired(environment, head)
    }

    fn current(&self, environment: &Environment) -> Answer<Digest> {
        self.inner.current(environment)
    }

    fn apply(&self, environment: &Environment, version: &Verified) -> Answer<()> {
        self.at.set(Some(self.store.calls().len()));
        self.inner.apply(environment, version)
    }

    fn report(&self, environment: &Environment) -> Answer<Running> {
        self.inner.report(environment)
    }
}

fn the_head_applied_after_its_claims_are_judged(world: &FakeWorld) {
    let (first, second) = applying(world);
    let target = target(world);
    let at = Rc::new(Cell::new(None));
    let mut seams = world.seams();
    seams.targets.insert(
        TargetProfile::new(TARGET),
        Box::new(Watched {
            inner: target.clone(),
            store: world.fakes.store.clone(),
            at: at.clone(),
        }),
    );
    let (before, writes) = (world.fakes.store.calls().len(), world.writes().len());
    let answer = apply(&seams, &world.history(), &ask(world));
    assert_eq!(
        answer,
        Ok(Applied {
            environment: cache(),
            component: ComponentName::new(COMPONENT),
            channel: ChannelName::new(STABLE),
            version: Version::new("0.2.0"),
            index: second.clone(),
            applied: true,
            current: Ok(first),
            report: Ok(Running {
                digest: second.clone(),
                converged_at: CONVERGED.into(),
            }),
        }),
        "the apply named another environment, channel, version or report"
    );
    assert_eq!(
        ops(&target),
        [
            TargetOp::Desired,
            TargetOp::Current,
            TargetOp::Apply,
            TargetOp::Report
        ]
    );
    let calls = target.calls();
    assert_eq!(
        calls[0].answer,
        Ok(json!(second)),
        "desired was not handed the chain's head"
    );
    assert_eq!(
        calls[2].given,
        json!({
            "environment": Environment {
                name: cache(),
                channel: channel(STABLE),
                waits: false,
                settings: settings(),
            },
            "version": Verified::new(second.clone(), Version::new("0.2.0")),
        }),
        "the target was handed another environment or version"
    );
    let at = at.get().expect("the target was asked to apply");
    let read = |range: &[timber_fakes::Call<StoreOp>]| {
        range
            .iter()
            .any(|c| c.op == StoreOp::ClaimsOf && c.given["digest"] == json!(second))
    };
    let store = world.fakes.store.calls();
    assert!(
        read(&store[before..at]),
        "the target was handed the version before its claims were read"
    );
    assert!(!read(&store[at..]), "the claims were read after the apply");
    assert_eq!(world.writes().len(), writes, "the apply wrote to the store");
}

fn a_wanting_claim_stops_the_apply(world: &FakeWorld) {
    applying(world);
    let named = |id: &str, kinds: &[ClaimKind]| NamedSigner {
        id: SignerId::new(id),
        verified_by: format!("{id}'s key"),
        kinds: kinds.to_vec(),
        checks: vec![],
    };
    let gate = named(GATE, &[ClaimKind::Acceptance, ClaimKind::Promotion]);
    let wanted = |kind: ClaimKind, signer: NamedSigner| RequiredClaim {
        kind,
        check: None,
        environment: None,
        signers: vec![signer],
        asked_of: AskedOf::Every,
    };
    world.fakes.policy.require(
        Audience::Channel(channel(STABLE)),
        Required::Channel {
            claims: vec![
                wanted(ClaimKind::Acceptance, gate.clone()),
                wanted(ClaimKind::Rebuild, named("box", &[ClaimKind::Rebuild])),
            ],
            bound_to: None,
            promoted_by: vec![gate],
            approvers: vec![],
        },
    );
    let target = target(world);
    match applied(world) {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::Wanting {
                kind: ClaimKind::Rebuild,
                check: None,
                environment: None,
                signers: vec![SignerId::new("box")],
            },
            "the apply was refused for another claim than the rebuild by box"
        ),
        other => panic!("an apply of a head wanting a rebuild by box answered {other:?}"),
    }
    assert!(
        !ops(&target).contains(&TargetOp::Apply),
        "the target was asked to apply: {:?}",
        ops(&target)
    );
}

fn an_empty_channel(world: &FakeWorld) {
    applying(world);
    world
        .fakes
        .policy
        .declares_at(&world.head, cache_on(channel("beta"), TARGET));
    let a = absent(applied(world), "an apply of an environment following beta");
    assert_eq!(a.source.seam, Seam::Keeper, "{a:?}");
    assert!(
        a.what.contains("beta"),
        "the answer did not name beta: {a:?}"
    );
    assert_eq!(world.target_calls(), [], "the target was asked");
}

fn an_unbound_profile(world: &FakeWorld) {
    applying(world);
    world
        .fakes
        .policy
        .declares_at(&world.head, cache_on(channel(STABLE), "bucket"));
    let a = absent(applied(world), "an apply of an environment on bucket");
    assert_eq!(a.source.seam, Seam::Target, "{a:?}");
    assert!(
        a.what.contains("bucket"),
        "the answer did not name bucket: {a:?}"
    );
    assert_eq!(world.target_calls(), [], "a target was asked");
}

fn an_environment_already_at_the_head(world: &FakeWorld) {
    let (_, second) = applying(world);
    let target = target(world);
    target.runs(
        &cache(),
        Running {
            digest: second.clone(),
            converged_at: "2026-02-01T00:00:00Z".into(),
        },
    );
    let answer = ok(applied(world), "an apply of an environment at the head");
    assert!(!answer.applied, "the answer says it applied: {answer:?}");
    assert_eq!(answer.current, Ok(second.clone()), "{answer:?}");
    assert_eq!(
        (answer.version.as_str(), &answer.index),
        ("0.2.0", &second),
        "{answer:?}"
    );
    assert_eq!(
        ops(&target),
        [TargetOp::Desired, TargetOp::Current, TargetOp::Report],
        "the target was asked to apply what it runs"
    );
}
