//! The fake policy (B.5): what each audience requires, for every revision
//! or for one, and the component each channel belongs to, as a test
//! scripts it.

use std::collections::BTreeMap;

use serde_json::json;
use tailrace_seams::{
    Answer, Audience, ChannelName, ComponentName, DeclaredChannel, Not, Policy, Required, Revision,
    Seam, REPOSITORY_COMPONENT,
};

use crate::Fake;

/// The fake policy's operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyOp {
    PolicyFor,
    Channels,
}

/// What each audience requires, and the component each channel belongs
/// to, at every revision and at one.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PolicyState {
    every: BTreeMap<Audience, Required>,
    at: BTreeMap<(Revision, Audience), Required>,
    belongs: BTreeMap<ChannelName, ComponentName>,
    belongs_at: BTreeMap<(Revision, ChannelName), ComponentName>,
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

    /// `channel` in `component` at every revision; a channel no test puts
    /// in a component belongs to the one named `repository` (123).
    pub fn belongs(&self, channel: &ChannelName, component: &ComponentName) {
        self.state()
            .belongs
            .insert(channel.clone(), component.clone());
    }

    /// `channel` in `component` at `revision`, over the component it is
    /// in at every revision.
    pub fn belongs_at(
        &self,
        revision: &Revision,
        channel: &ChannelName,
        component: &ComponentName,
    ) {
        self.state()
            .belongs_at
            .insert((revision.clone(), channel.clone()), component.clone());
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
    /// every revision, the revision's policy over every revision's, in
    /// the component scripted for it, bound as its policy says. A binding
    /// is answered as scripted, never checked.
    fn channels(&self, revision: &Revision) -> Answer<Vec<DeclaredChannel>> {
        let given = json!({ "revision": revision });
        self.ask(PolicyOp::Channels, given, "the channels", |s, _| {
            let mut held: BTreeMap<&ChannelName, &Required> = BTreeMap::new();
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
                .map(|(channel, required)| DeclaredChannel {
                    name: channel.clone(),
                    component: s
                        .belongs_at
                        .get(&(revision.clone(), channel.clone()))
                        .or_else(|| s.belongs.get(channel))
                        .cloned()
                        .unwrap_or_else(|| ComponentName::new(REPOSITORY_COMPONENT)),
                    bound_to: required.bound_to().cloned(),
                })
                .collect())
        })
    }
}

#[cfg(test)]
mod tests {
    use tailrace_seams::{ChannelName, Check, ClaimKind, RequiredClaim, Source};

    use super::*;
    use crate::{Fault, Times};

    fn claim(check: &str) -> RequiredClaim {
        RequiredClaim {
            kind: ClaimKind::Verdict,
            check: Some(Check::new(check)),
            signers: vec![],
        }
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
        let stable = Audience::Channel(ChannelName::new("stable"));
        policy.require(
            stable.clone(),
            Required::Channel {
                claims: vec![],
                bound_to: Some(ChannelName::new("beta")),
            },
        );
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
        }
    }

    fn declared(name: &str, component: &str, bound_to: Option<&str>) -> DeclaredChannel {
        DeclaredChannel {
            name: ChannelName::new(name),
            component: ComponentName::new(component),
            bound_to: bound_to.map(ChannelName::new),
        }
    }

    #[test]
    fn the_channels_are_in_the_component_scripted_at_a_revision_and_bound_as_their_policy_says() {
        let policy = FakePolicy::new();
        let (beta, stable) = (ChannelName::new("beta"), ChannelName::new("stable"));
        let (at, other) = (Revision::new("4f2a9c1"), Revision::new("77b0e3d"));
        policy.require(Audience::Channel(beta.clone()), channel(None));
        policy.require(Audience::Channel(stable), channel(Some("beta")));
        policy.require(
            Audience::Acceptance,
            Required::Acceptance { claims: vec![] },
        );
        policy.belongs_at(&at, &beta, &ComponentName::new("lib"));

        assert_eq!(
            policy.channels(&at),
            Ok(vec![
                declared("beta", "lib", None),
                declared("stable", "repository", Some("beta")),
            ])
        );
        assert_eq!(
            policy.channels(&other),
            Ok(vec![
                declared("beta", "repository", None),
                declared("stable", "repository", Some("beta")),
            ])
        );
        let calls = policy.calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].op, PolicyOp::Channels);
        assert_eq!(calls[0].given["revision"], "4f2a9c1");
    }

    #[test]
    fn a_channel_in_a_component_at_every_revision_yields_to_the_revisions() {
        let policy = FakePolicy::new();
        let beta = ChannelName::new("beta");
        let (at, other) = (Revision::new("4f2a9c1"), Revision::new("77b0e3d"));
        policy.require(Audience::Channel(beta.clone()), channel(None));
        policy.belongs(&beta, &ComponentName::new("web"));
        policy.belongs_at(&at, &beta, &ComponentName::new("lib"));
        assert_eq!(
            policy.channels(&at),
            Ok(vec![declared("beta", "lib", None)])
        );
        assert_eq!(
            policy.channels(&other),
            Ok(vec![declared("beta", "web", None)])
        );
    }

    #[test]
    fn a_channels_policy_at_a_revision_is_read_over_every_revisions() {
        let policy = FakePolicy::new();
        let stable = Audience::Channel(ChannelName::new("stable"));
        let (old, new) = (Revision::new("old"), Revision::new("new"));
        policy.require(stable.clone(), channel(None));
        policy.require_at(&new, stable, channel(Some("beta")));
        policy.require_at(
            &new,
            Audience::Channel(ChannelName::new("edge")),
            channel(None),
        );
        assert_eq!(
            policy.channels(&old),
            Ok(vec![declared("stable", "repository", None)])
        );
        assert_eq!(
            policy.channels(&new),
            Ok(vec![
                declared("edge", "repository", None),
                declared("stable", "repository", Some("beta")),
            ])
        );
    }

    #[test]
    fn no_channel_declared_answers_an_empty_list() {
        let policy = FakePolicy::new();
        policy.require(
            Audience::Acceptance,
            Required::Acceptance { claims: vec![] },
        );
        assert_eq!(policy.channels(&Revision::new("4f2a9c1")), Ok(vec![]));
    }

    #[test]
    fn the_channels_can_be_faulted_and_are_recorded() {
        let policy = FakePolicy::new();
        policy.require(Audience::Channel(ChannelName::new("beta")), channel(None));
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
        assert!(policy
            .policy_for(&rev, &Audience::Channel(ChannelName::new("beta")))
            .is_ok());
    }
}
