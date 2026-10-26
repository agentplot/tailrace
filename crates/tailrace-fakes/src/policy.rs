//! The fake policy (B.5): what each audience requires, for every revision
//! or for one, and which channels move on each publish, as a test
//! scripts it. A channel carries its component (125), and the fake gives
//! no channel a test did not script: the channels a profile gives with
//! nothing declared are that profile's (C.10).

use std::collections::{BTreeMap, BTreeSet};

use serde_json::json;
use tailrace_seams::{
    Answer, Audience, ChannelId, DeclaredChannel, Moves, Not, Policy, Required, Revision, Seam,
};

use crate::Fake;

/// The fake policy's operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyOp {
    PolicyFor,
    Channels,
}

/// What each audience requires, at every revision and at one, and the
/// channels that move on each publish, at every revision and at one.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PolicyState {
    every: BTreeMap<Audience, Required>,
    at: BTreeMap<(Revision, Audience), Required>,
    follows: BTreeSet<ChannelId>,
    follows_at: BTreeSet<(Revision, ChannelId)>,
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
    /// accepted publish at every revision.
    pub fn follows(&self, channel: &ChannelId) {
        self.state().follows.insert(channel.clone());
    }

    /// 95: `channel` moves on each accepted publish at `revision`.
    pub fn follows_at(&self, revision: &Revision, channel: &ChannelId) {
        self.state()
            .follows_at
            .insert((revision.clone(), channel.clone()));
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
    /// alone otherwise. A binding is answered as scripted, never checked.
    fn channels(&self, revision: &Revision) -> Answer<Vec<DeclaredChannel>> {
        let given = json!({ "revision": revision });
        self.ask(PolicyOp::Channels, given, "the channels", |s, _| {
            let mut held: BTreeMap<&ChannelId, &Required> = BTreeMap::new();
            for (audience, required) in &s.every {
                if let Audience::Channel(channel) = audience {
                    held.insert(channel, required);
                }
            }
            for ((at, audience), required) in &s.at {
                if let (true, Audience::Channel(channel)) = (at == revision, audience) {
                    held.insert(channel, required);
                }
            }
            Ok(held
                .into_iter()
                .map(|(channel, required)| {
                    let follows = s.follows.contains(channel)
                        || s.follows_at.contains(&(revision.clone(), channel.clone()));
                    let moves = match (required.bound_to(), follows) {
                        (Some(to), _) => Moves::Bound { to: to.clone() },
                        (None, true) => Moves::EachPublish,
                        (None, false) => Moves::Promotion,
                    };
                    DeclaredChannel {
                        channel: channel.clone(),
                        moves,
                    }
                })
                .collect())
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
        policy.require(Audience::Layout, Required::Layout { claims: vec![] });
        policy.require_at(
            &new,
            Audience::Layout,
            Required::Layout {
                claims: vec![claim("test")],
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
        }
    }

    fn declared(channel: ChannelId, moves: Moves) -> DeclaredChannel {
        DeclaredChannel { channel, moves }
    }

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
        policy.follows(&beta);

        assert_eq!(
            policy.channels(&at),
            Ok(vec![
                declared(beta.clone(), Moves::EachPublish),
                declared(edge, Moves::Promotion),
                declared(stable.clone(), bound("beta")),
            ])
        );
        assert_eq!(
            policy.channels(&other),
            Ok(vec![
                declared(beta, Moves::EachPublish),
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
        policy.follows(&beta);
        policy.follows_at(&at, &edge);
        assert_eq!(
            policy.channels(&at),
            Ok(vec![
                declared(beta.clone(), Moves::EachPublish),
                declared(edge.clone(), Moves::EachPublish),
            ])
        );
        assert_eq!(
            policy.channels(&other),
            Ok(vec![
                declared(beta, Moves::EachPublish),
                declared(edge, Moves::Promotion),
            ])
        );
    }

    #[test]
    fn two_components_channels_of_one_name_are_two() {
        let policy = FakePolicy::new();
        let (libs, webs) = (id(Some("lib"), "beta"), id(Some("web"), "beta"));
        policy.require(Audience::Channel(webs.clone()), channel(None));
        policy.require(Audience::Channel(libs.clone()), channel(None));
        policy.follows(&webs);
        assert_eq!(
            policy.channels(&Revision::new("4f2a9c1")),
            Ok(vec![
                declared(libs, Moves::Promotion),
                declared(webs, Moves::EachPublish),
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
            policy.channels(&old),
            Ok(vec![declared(id(None, "stable"), Moves::Promotion)])
        );
        assert_eq!(
            policy.channels(&new),
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
        policy.follows(&id(None, "beta"));
        assert_eq!(policy.channels(&Revision::new("4f2a9c1")), Ok(vec![]));
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
            policy.channels(&rev),
            Err(Not::Unreadable(u)) if u.said.as_deref() == Some("no such revision")
        ));
        assert_eq!(policy.channels(&rev).map(|c| c.len()), Ok(1));
        let calls = policy.calls();
        assert_eq!(calls.len(), 2);
        assert!(calls.iter().all(|c| c.op == PolicyOp::Channels));
        assert!(policy.policy_for(&rev, &beta).is_ok());
    }
}
