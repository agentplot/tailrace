//! The fake policy (B.5): what each audience requires, for every revision
//! or for one, as a test scripts it.

use std::collections::BTreeMap;

use serde_json::json;
use tailrace_seams::{Answer, Audience, Not, Policy, Required, Revision, Seam};

use crate::Fake;

/// The fake policy's operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyOp {
    PolicyFor,
}

/// What each audience requires, at every revision and at one.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PolicyState {
    every: BTreeMap<Audience, Required>,
    at: BTreeMap<(Revision, Audience), Required>,
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
}
