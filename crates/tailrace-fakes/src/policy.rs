//! The fake policy (B.5): what each audience requires, for every revision
//! or for one, which channels move on each publish, and the givens it
//! holds off, as a test scripts it. A channel carries its component
//! (125), and the fake gives and holds off no channel a test did not
//! script: the channels a profile gives with nothing declared are that
//! profile's (C.10).

use std::collections::BTreeMap;

use serde_json::json;
use tailrace_seams::{
    Answer, Audience, ChannelId, DeclaredChannel, HeldChannel, Moves, Not, Policy, PolicyChannels,
    Required, Revision, Seam,
};

use crate::Fake;

/// The fake policy's operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyOp {
    PolicyFor,
    Channels,
}

/// What each audience requires, at every revision and at one, the
/// channels that move on each publish, at every revision and at one, and
/// the givens held off, each with how it would move and its line.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PolicyState {
    every: BTreeMap<Audience, Required>,
    at: BTreeMap<(Revision, Audience), Required>,
    follows: BTreeMap<ChannelId, bool>,
    follows_at: BTreeMap<(Revision, ChannelId), bool>,
    holds: BTreeMap<ChannelId, (Moves, String)>,
}

/// A policy in memory.
pub type FakePolicy = Fake<PolicyState, PolicyOp>;

impl Default for FakePolicy {
    fn default() -> Self {
        Self::new()
    }
}

fn of(audience: &Audience) -> String {
    match audience {
        Audience::Acceptance => "a policy for acceptance".into(),
        Audience::Channel(channel) => format!("a policy for channel {channel}"),
        Audience::Layout => "a policy for a layout".into(),
    }
}

impl FakePolicy {
    pub fn new() -> Self {
        Fake::with(Seam::Policy, PolicyState::default())
    }

    /// What `audience` requires at every revision.
    pub fn require(&self, audience: Audience, required: Required) {
        self.state().every.insert(audience, required);
    }

    /// What `audience` requires at `revision`, over what it requires at
    /// every revision (107).
    pub fn require_at(&self, revision: &Revision, audience: Audience, required: Required) {
        self.state()
            .at
            .insert((revision.clone(), audience), required);
    }

    /// 95: `channel`, where a policy is held for it, moves on each
    /// accepted publish at every revision, taking prereleases too where
    /// `prereleases` (C.10).
    pub fn follows(&self, channel: &ChannelId, prereleases: bool) {
        self.state().follows.insert(channel.clone(), prereleases);
    }

    /// 95: `channel` moves on each accepted publish at `revision`, taking
    /// prereleases too where `prereleases` (C.10).
    pub fn follows_at(&self, revision: &Revision, channel: &ChannelId, prereleases: bool) {
        self.state()
            .follows_at
            .insert((revision.clone(), channel.clone()), prereleases);
    }

    /// C.10: `channel` held off at every revision, moving as `moves` once
    /// `switch`, the line that switches it on, is written; carried in the
    /// channels answer apart from the channels in force.
    pub fn holds(&self, channel: &ChannelId, moves: Moves, switch: impl Into<String>) {
        self.state()
            .holds
            .insert(channel.clone(), (moves, switch.into()));
    }
}

impl Policy for FakePolicy {
    fn policy_for(&self, revision: &Revision, audience: &Audience) -> Answer<Required> {
        let given = json!({ "revision": revision, "audience": audience });
        self.ask(PolicyOp::PolicyFor, given, of(audience), |s, source| {
            s.at.get(&(revision.clone(), audience.clone()))
                .or_else(|| s.every.get(audience))
                .cloned()
                .ok_or_else(|| Not::absent(source, of(audience)))
        })
    }

    /// 131: a channel for each channel audience held at `revision` or at
    /// every revision, the revision's policy over every revision's, by
    /// name and then component, each in the component its identity
    /// carries (125): bound where its policy names a binding, moving on
    /// each publish where the test scripted it so, and by a promotion
    /// alone otherwise, taking prereleases as scripted. A binding is
    /// answered as scripted, never checked. Apart from them, each given
    /// scripted held off, by name and then component, with how it would
    /// move and its line; an empty list where none is (C.10).
    fn channels(&self, revision: &Revision) -> Answer<PolicyChannels> {
        let given = json!({ "revision": revision });
        self.ask(PolicyOp::Channels, given, "the channels", |s, _| {
            let mut scripted: BTreeMap<&ChannelId, &Required> = BTreeMap::new();
            for (audience, required) in &s.every {
                if let Audience::Channel(channel) = audience {
                    scripted.insert(channel, required);
                }
            }
            for ((at, audience), required) in &s.at {
                if let (true, Audience::Channel(channel)) = (at == revision, audience) {
                    scripted.insert(channel, required);
                }
            }
            let in_force = scripted
                .into_iter()
                .map(|(channel, required)| {
                    let follows = s
                        .follows_at
                        .get(&(revision.clone(), channel.clone()))
                        .or_else(|| s.follows.get(channel));
                    let moves = match (required.bound_to(), follows) {
                        (Some(to), _) => Moves::Bound { to: to.clone() },
                        (None, Some(prereleases)) => Moves::EachPublish {
                            prereleases: *prereleases,
                        },
                        (None, None) => Moves::Promotion,
                    };
                    DeclaredChannel {
                        channel: channel.clone(),
                        moves,
                    }
                })
                .collect();
            let held = s
                .holds
                .iter()
                .map(|(channel, (moves, switch))| HeldChannel {
                    channel: DeclaredChannel {
                        channel: channel.clone(),
                        moves: moves.clone(),
                    },
                    switch: switch.clone(),
                })
                .collect();
            Ok(PolicyChannels { in_force, held })
        })
    }
}

#[cfg(test)]
mod tests {
    use tailrace_seams::{ChannelName, Check, ClaimKind, ComponentName, RequiredClaim, Source};

    use super::*;
    use crate::{Fault, Times};

    fn claim(check: &str) -> RequiredClaim {
        RequiredClaim {
            kind: ClaimKind::Verdict,
            check: Some(Check::new(check)),
            signers: vec![],
        }
    }

    /// A channel of `component`, or of none where it names none.
    fn id(component: Option<&str>, name: &str) -> ChannelId {
        ChannelId::new(component.map(ComponentName::new), ChannelName::new(name))
    }

    #[test]
    fn acceptance_names_no_channel() {
        let policy = FakePolicy::new();
        policy.require(
            Audience::Acceptance,
            Required::Acceptance {
                claims: vec![claim("test")],
            },
        );
        let required = policy
            .policy_for(&Revision::new("4f2a9c1"), &Audience::Acceptance)
            .unwrap();
        assert_eq!(required.bound_to(), None);
        assert_eq!(required.claims(), &[claim("test")]);
    }

    #[test]
    fn a_bound_channel_names_its_channel() {
        let policy = FakePolicy::new();
        let stable = Audience::Channel(id(None, "stable"));
        policy.require(stable.clone(), channel(Some("beta")));
        let required = policy
            .policy_for(&Revision::new("4f2a9c1"), &stable)
            .unwrap();
        assert_eq!(required.bound_to(), Some(&ChannelName::new("beta")));
    }

    #[test]
    fn a_policy_at_one_revision_is_read_at_that_revision() {
        let policy = FakePolicy::new();
        let (old, new) = (Revision::new("old"), Revision::new("new"));
        policy.require(
            Audience::Layout,
            Required::Layout {
                claims: vec![],
                given: false,
                signers: vec![],
            },
        );
        policy.require_at(
            &new,
            Audience::Layout,
            Required::Layout {
                claims: vec![claim("test")],
                given: false,
                signers: vec![],
            },
        );
        assert_eq!(
            policy.policy_for(&old, &Audience::Layout).unwrap().claims(),
            &[]
        );
        assert_eq!(
            policy.policy_for(&new, &Audience::Layout).unwrap().claims(),
            &[claim("test")]
        );
    }

    #[test]
    fn no_policy_declared_is_absent() {
        let policy = FakePolicy::new();
        assert_eq!(
            policy.policy_for(&Revision::new("4f2a9c1"), &Audience::Acceptance),
            Err(Not::absent(
                &Source::new(Seam::Policy, "fake"),
                "a policy for acceptance"
            ))
        );
    }

    #[test]
    fn its_operation_can_be_faulted_and_is_recorded() {
        let policy = FakePolicy::new();
        policy.fault(
            PolicyOp::PolicyFor,
            Fault::Unreadable("no such revision".into()),
            Times::Once,
        );
        assert!(matches!(
            policy.policy_for(&Revision::new("4f2a9c1"), &Audience::Acceptance),
            Err(Not::Unreadable(u)) if u.said.as_deref() == Some("no such revision")
        ));
        assert_eq!(policy.calls().len(), 1);
    }

    fn channel(bound_to: Option<&str>) -> Required {
        Required::Channel {
            claims: vec![],
            bound_to: bound_to.map(ChannelName::new),
            promoted_by: vec![],
            approvers: vec![],
        }
    }

    fn declared(channel: ChannelId, moves: Moves) -> DeclaredChannel {
        DeclaredChannel { channel, moves }
    }

    /// The channels in force `policy` answers at `at`.
    fn listed(policy: &FakePolicy, at: &Revision) -> Answer<Vec<DeclaredChannel>> {
        policy.channels(at).map(|channels| channels.in_force)
    }

    /// Moving on each publish with releases alone.
    const RELEASES: Moves = Moves::EachPublish { prereleases: false };

    fn bound(to: &str) -> Moves {
        Moves::Bound {
            to: ChannelName::new(to),
        }
    }

    #[test]
    fn the_channels_move_as_scripted_each_in_its_own_component() {
        let policy = FakePolicy::new();
        let (beta, stable, edge) = (
            id(Some("lib"), "beta"),
            id(Some("lib"), "stable"),
            id(None, "edge"),
        );
        let (at, other) = (Revision::new("4f2a9c1"), Revision::new("77b0e3d"));
        policy.require(Audience::Channel(beta.clone()), channel(None));
        policy.require(Audience::Channel(stable.clone()), channel(Some("beta")));
        policy.require_at(&at, Audience::Channel(edge.clone()), channel(None));
        policy.require(
            Audience::Acceptance,
            Required::Acceptance { claims: vec![] },
        );
        policy.follows(&beta, false);

        assert_eq!(
            listed(&policy, &at),
            Ok(vec![
                declared(beta.clone(), RELEASES),
                declared(edge, Moves::Promotion),
                declared(stable.clone(), bound("beta")),
            ])
        );
        assert_eq!(
            listed(&policy, &other),
            Ok(vec![
                declared(beta, RELEASES),
                declared(stable, bound("beta")),
            ])
        );
        let calls = policy.calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].op, PolicyOp::Channels);
        assert_eq!(calls[0].given["revision"], "4f2a9c1");
    }

    #[test]
    fn a_channel_follows_each_publish_at_one_revision_or_at_every_one() {
        let policy = FakePolicy::new();
        let (beta, edge) = (id(None, "beta"), id(None, "edge"));
        let (at, other) = (Revision::new("4f2a9c1"), Revision::new("77b0e3d"));
        policy.require(Audience::Channel(beta.clone()), channel(None));
        policy.require(Audience::Channel(edge.clone()), channel(None));
        policy.follows(&beta, false);
        policy.follows_at(&at, &edge, false);
        assert_eq!(
            listed(&policy, &at),
            Ok(vec![
                declared(beta.clone(), RELEASES),
                declared(edge.clone(), RELEASES),
            ])
        );
        assert_eq!(
            listed(&policy, &other),
            Ok(vec![
                declared(beta, RELEASES),
                declared(edge, Moves::Promotion),
            ])
        );
    }

    #[test]
    fn a_following_channel_scripted_to_take_prereleases_is_listed_so() {
        let policy = FakePolicy::new();
        let (beta, nightly) = (id(None, "beta"), id(None, "nightly"));
        policy.require(Audience::Channel(beta.clone()), channel(None));
        policy.require(Audience::Channel(nightly.clone()), channel(None));
        policy.follows(&beta, false);
        policy.follows(&nightly, true);
        assert_eq!(
            listed(&policy, &Revision::new("4f2a9c1")),
            Ok(vec![
                declared(beta, RELEASES),
                declared(nightly, Moves::EachPublish { prereleases: true }),
            ])
        );
    }

    #[test]
    fn two_components_channels_of_one_name_are_two() {
        let policy = FakePolicy::new();
        let (libs, webs) = (id(Some("lib"), "beta"), id(Some("web"), "beta"));
        policy.require(Audience::Channel(webs.clone()), channel(None));
        policy.require(Audience::Channel(libs.clone()), channel(None));
        policy.follows(&webs, false);
        assert_eq!(
            listed(&policy, &Revision::new("4f2a9c1")),
            Ok(vec![
                declared(libs, Moves::Promotion),
                declared(webs, RELEASES),
            ])
        );
    }

    #[test]
    fn a_channels_policy_at_a_revision_is_read_over_every_revisions() {
        let policy = FakePolicy::new();
        let stable = Audience::Channel(id(None, "stable"));
        let (old, new) = (Revision::new("old"), Revision::new("new"));
        policy.require(stable.clone(), channel(None));
        policy.require_at(&new, stable, channel(Some("beta")));
        policy.require_at(&new, Audience::Channel(id(None, "edge")), channel(None));
        assert_eq!(
            listed(&policy, &old),
            Ok(vec![declared(id(None, "stable"), Moves::Promotion)])
        );
        assert_eq!(
            listed(&policy, &new),
            Ok(vec![
                declared(id(None, "edge"), Moves::Promotion),
                declared(id(None, "stable"), bound("beta")),
            ])
        );
    }

    #[test]
    fn no_channel_scripted_answers_an_empty_list() {
        let policy = FakePolicy::new();
        policy.require(
            Audience::Acceptance,
            Required::Acceptance { claims: vec![] },
        );
        policy.follows(&id(None, "beta"), false);
        assert_eq!(listed(&policy, &Revision::new("4f2a9c1")), Ok(vec![]));
    }

    #[test]
    fn a_given_held_off_is_carried_apart_from_the_channels_in_force() {
        let policy = FakePolicy::new();
        let rev = Revision::new("4f2a9c1");
        assert_eq!(
            policy.channels(&rev),
            Ok(PolicyChannels {
                in_force: vec![],
                held: vec![],
            })
        );
        let canary = id(None, "canary");
        policy.holds(&canary, RELEASES, "[channels.canary]");
        let answer = policy.channels(&rev).unwrap();
        assert_eq!(
            answer.held,
            [HeldChannel {
                channel: declared(canary.clone(), RELEASES),
                switch: "[channels.canary]".into(),
            }]
        );
        assert!(
            !answer.in_force.iter().any(|c| c.channel == canary),
            "{answer:?}"
        );
        policy.fault(
            PolicyOp::Channels,
            Fault::Unreadable("no such revision".into()),
            Times::Once,
        );
        assert!(matches!(
            policy.channels(&rev),
            Err(Not::Unreadable(u)) if u.said.as_deref() == Some("no such revision")
        ));
        let calls = policy.calls();
        assert_eq!(calls.len(), 3);
        assert!(calls.iter().all(|c| c.op == PolicyOp::Channels));
    }

    #[test]
    fn the_channels_can_be_faulted_and_are_recorded() {
        let policy = FakePolicy::new();
        let beta = Audience::Channel(id(None, "beta"));
        policy.require(beta.clone(), channel(None));
        policy.fault(
            PolicyOp::Channels,
            Fault::Unreadable("no such revision".into()),
            Times::Once,
        );
        let rev = Revision::new("4f2a9c1");
        assert!(matches!(
            listed(&policy, &rev),
            Err(Not::Unreadable(u)) if u.said.as_deref() == Some("no such revision")
        ));
        assert_eq!(listed(&policy, &rev).map(|c| c.len()), Ok(1));
        let calls = policy.calls();
        assert_eq!(calls.len(), 2);
        assert!(calls.iter().all(|c| c.op == PolicyOp::Channels));
        assert!(policy.policy_for(&rev, &beta).is_ok());
    }
}
