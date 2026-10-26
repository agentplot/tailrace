//! Spec command-line, the policy in force: what acceptance and a layout
//! require, every channel the policy lists and the givens it holds off,
//! asked of the core over the fakes at the branch's landed head, each part
//! and each channel its own answer.

mod common;
mod world;

use tailrace_core::{answer_json, in_force, ChannelInForce, ChannelsInForce, InForce, InForceAsk};
use tailrace_fakes::{FakePolicy, LandingOp, PolicyOp, PROFILE};
use tailrace_seams::{
    Answer, Audience, ChannelId, ChannelName, ClaimKind, ComponentName, DeclaredChannel,
    EnvironmentName, HeldChannel, Moves, Not, Policy, PolicyChannels, Required, RequiredClaim,
    Revision, Seam, Seams, Source, Why,
};
use world::World;

/// The channel `name` of `component`, as the world declares components
/// (125).
fn of(component: &str, name: &str) -> ChannelId {
    ChannelId::new(Some(ComponentName::new(component)), ChannelName::new(name))
}

fn acceptance() -> Required {
    Required::Acceptance {
        claims: vec![RequiredClaim {
            kind: ClaimKind::Provenance,
            check: None,
            signers: vec![],
        }],
    }
}

fn channel_policy(bound_to: Option<&str>) -> Required {
    Required::Channel {
        claims: vec![],
        bound_to: bound_to.map(ChannelName::new),
        promoted_by: vec![],
        approvers: vec![],
    }
}

/// The fake policy requiring an acceptance, holding `beta` of `lib`
/// moving on each publish, `edge` of `lib` by a promotion alone and
/// `stable` of `lib` bound to `beta`, and no layout.
fn declare(policy: &FakePolicy) {
    policy.require(Audience::Acceptance, acceptance());
    policy.require(Audience::Channel(of("lib", "beta")), channel_policy(None));
    policy.follows(&of("lib", "beta"), false);
    policy.require(Audience::Channel(of("lib", "edge")), channel_policy(None));
    policy.require(
        Audience::Channel(of("lib", "stable")),
        channel_policy(Some("beta")),
    );
}

fn ask(world: &World) -> InForceAsk {
    InForceAsk {
        branch: world.main.clone(),
    }
}

fn asked(world: &World, seams: &Seams) -> Answer<InForce> {
    in_force(seams, &world.history(), &ask(world))
}

fn listed(channel: ChannelId, moves: Moves, policy: Answer<Required>) -> ChannelInForce {
    ChannelInForce {
        channel: DeclaredChannel { channel, moves },
        policy,
    }
}

// Requirement: Policy shows the policy in force

#[test]
fn a_head_not_landed_is_refused_before_the_policy_is_asked() {
    let world = World::new();
    declare(&world.fakes.policy);
    let unlanded = world.repo.touch("feat: lib grows again", &["crates/lib"]);
    let Err(Not::Refused(r)) = asked(&world, &world.fakes.seams()) else {
        panic!("a head not landed was answered")
    };
    assert_eq!(
        r.why,
        Why::NotLanded {
            branch: world.main.clone(),
            revision: Revision::new(unlanded.as_str()),
        }
    );
    assert!(
        world.fakes.policy.calls().is_empty(),
        "the policy was asked before the landing said yes"
    );
}

#[test]
fn an_absent_layout_is_shown_beside_the_channels() {
    let world = World::new();
    declare(&world.fakes.policy);
    let answer = asked(&world, &world.fakes.seams()).unwrap();
    assert_eq!(
        (&answer.branch, &answer.revision),
        (&world.main, &world.head)
    );
    assert_eq!(answer.acceptance, Ok(acceptance()));
    let Err(Not::Absent(a)) = &answer.layout else {
        panic!("the layout answered {:?}", answer.layout)
    };
    assert!(a.what.contains("layout"), "{a:?}");
    assert_eq!(
        answer.channels.clone().map(|c| c.in_force),
        Ok(vec![
            listed(
                of("lib", "beta"),
                Moves::EachPublish { prereleases: false },
                Ok(channel_policy(None))
            ),
            listed(
                of("lib", "edge"),
                Moves::Promotion,
                Ok(channel_policy(None))
            ),
            listed(
                of("lib", "stable"),
                Moves::Bound {
                    to: ChannelName::new("beta")
                },
                Ok(channel_policy(Some("beta")))
            ),
        ])
    );

    // Each part carries its own answer in the one form (57).
    let json = answer_json(&Ok::<_, Not>(answer));
    let value = &json["value"];
    assert_eq!(value["acceptance"]["answer"], "ok", "{json}");
    assert_eq!(value["layout"]["answer"], "absent", "{json}");
    let first = &value["channels"]["value"]["in_force"][0];
    assert_eq!(first["channel"]["name"], "beta", "{json}");
    assert_eq!(first["channel"]["component"], "lib", "{json}");
    assert_eq!(first["moves"]["by"], "each_publish", "{json}");
    assert_eq!(first["policy"]["answer"], "ok", "{json}");
}

/// The fake policy, answering the policy for `channel` unreadable with
/// `said`, as a profile whose one table cannot be read would.
struct OneUnreadable {
    inner: FakePolicy,
    channel: ChannelId,
    said: &'static str,
}

impl Policy for OneUnreadable {
    fn policy_for(&self, revision: &Revision, audience: &Audience) -> Answer<Required> {
        if *audience == Audience::Channel(self.channel.clone()) {
            return Err(Not::unreadable(
                &Source::new(Seam::Policy, PROFILE),
                format!("the policy for {}", self.channel),
                Some(self.said.into()),
            ));
        }
        self.inner.policy_for(revision, audience)
    }

    fn channels(&self, revision: &Revision) -> Answer<PolicyChannels> {
        self.inner.channels(revision)
    }
}

#[test]
fn one_channels_policy_unreadable_leaves_the_rest_standing() {
    let world = World::new();
    declare(&world.fakes.policy);
    let mut seams = world.fakes.seams();
    seams.policy = Box::new(OneUnreadable {
        inner: world.fakes.policy.clone(),
        channel: of("lib", "edge"),
        said: "edge's table cannot be read",
    });
    let answer = asked(&world, &seams).unwrap();
    assert_eq!(answer.acceptance, Ok(acceptance()));
    let channels = answer.channels.expect("the channels answered").in_force;
    let policies: Vec<(String, bool)> = channels
        .iter()
        .map(|c| (c.channel.channel.spelled(), c.policy.is_ok()))
        .collect();
    assert_eq!(
        policies,
        [
            ("lib/beta".to_string(), true),
            ("lib/edge".to_string(), false),
            ("lib/stable".to_string(), true),
        ]
    );
    let Err(Not::Unreadable(u)) = &channels[1].policy else {
        panic!("edge answered {:?}", channels[1].policy)
    };
    assert_eq!(u.said.as_deref(), Some("edge's table cannot be read"));
}

#[test]
fn the_policy_in_force_asks_the_landing_and_the_policy_alone_and_writes_nothing() {
    let mut world = World::new();
    let production = world.fakes.target(&EnvironmentName::new("production"));
    declare(&world.fakes.policy);
    let others = || {
        (
            world.fakes.store.calls().len(),
            world.fakes.build.calls().len(),
            world.fakes.signer.calls().len(),
            world.fakes.keeper.calls().len(),
            production.calls().len(),
            world
                .fakes
                .projections
                .values()
                .map(|p| p.calls().len())
                .sum::<usize>(),
        )
    };
    let (before, landing, policy) = (
        others(),
        world.fakes.landing.calls().len(),
        world.fakes.policy.calls().len(),
    );
    assert!(asked(&world, &world.fakes.seams()).is_ok());
    assert_eq!(
        others(),
        before,
        "the policy in force asked the store, the build, a signer, the ledger, a target or a projection"
    );
    let landing: Vec<LandingOp> = world.fakes.landing.calls()[landing..]
        .iter()
        .map(|c| c.op)
        .collect();
    assert_eq!(landing, [LandingOp::Landed]);
    let asked_of_policy = &world.fakes.policy.calls()[policy..];
    assert!(
        asked_of_policy
            .iter()
            .all(|c| matches!(c.op, PolicyOp::PolicyFor | PolicyOp::Channels)),
        "the policy was asked to do other than answer"
    );
    // Acceptance, a layout, the channels and each of the three channels.
    assert_eq!(asked_of_policy.len(), 6);
}

#[test]
fn the_givens_held_off_are_shown_beside_the_channels_from_one_answer() {
    let world = World::new();
    declare(&world.fakes.policy);
    let canary = of("lib", "canary");
    world.fakes.policy.holds(
        &canary,
        Moves::EachPublish { prereleases: true },
        "[channels.lib.canary]",
    );
    let before = world.fakes.policy.calls().len();
    let answer = asked(&world, &world.fakes.seams()).unwrap();
    let Ok(ChannelsInForce { in_force, held }) = &answer.channels else {
        panic!("the channels answered {:?}", answer.channels)
    };
    assert_eq!(
        held,
        &[HeldChannel {
            channel: DeclaredChannel {
                channel: canary.clone(),
                moves: Moves::EachPublish { prereleases: true },
            },
            switch: "[channels.lib.canary]".into(),
        }]
    );
    assert!(
        !in_force.iter().any(|c| c.channel.channel == canary),
        "a held given is among the channels in force"
    );
    let channels_asked = world.fakes.policy.calls()[before..]
        .iter()
        .filter(|c| c.op == PolicyOp::Channels)
        .count();
    assert_eq!(channels_asked, 1, "the channels were asked more than once");

    let json = answer_json(&Ok::<_, Not>(answer));
    let held = &json["value"]["channels"]["value"]["held"][0];
    assert_eq!(held["channel"]["name"], "canary", "{json}");
    assert_eq!(held["switch"], "[channels.lib.canary]", "{json}");
}

#[test]
fn an_unreadable_channels_answer_leaves_the_acceptance_and_the_layout_shown() {
    let world = World::new();
    declare(&world.fakes.policy);
    world
        .fakes
        .policy
        .require(Audience::Layout, Required::Layout { claims: vec![] });
    world.fakes.policy.fault(
        PolicyOp::Channels,
        tailrace_fakes::Fault::Unreadable("the channels cannot be read".into()),
        tailrace_fakes::Times::Once,
    );
    let answer = asked(&world, &world.fakes.seams()).unwrap();
    assert_eq!(answer.acceptance, Ok(acceptance()));
    assert_eq!(answer.layout, Ok(Required::Layout { claims: vec![] }));
    let Err(Not::Unreadable(u)) = &answer.channels else {
        panic!("the channels answered {:?}", answer.channels)
    };
    assert_eq!(u.said.as_deref(), Some("the channels cannot be read"));
}
