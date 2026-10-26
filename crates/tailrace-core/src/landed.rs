//! A revision its branch has landed: the one value every read of the
//! repository at a revision takes, so nothing is read at a revision the
//! branch has not landed (14, 65, 77, rule core.22; design D2).

use tailrace_seams::{
    Answer, Audience, Branch, DeclaredChannel, Landing, Not, Policy, Required, Revision, Seam,
    Source, Why,
};

/// The profile a refusal the core makes from a seam's own answer names:
/// a trait does not name the profile that implements it, so the core
/// names the one bound.
pub const BOUND: &str = "bound";

/// A revision `branch` has landed. Only the landing seam's yes makes one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Landed {
    branch: Branch,
    revision: Revision,
}

impl Landed {
    /// 77: `revision` as landed on `branch`, asked of the landing seam;
    /// refused [`Why::NotLanded`] naming both when the branch has not
    /// landed it (14), and the landing's own answer when it cannot say.
    pub fn check(landing: &dyn Landing, branch: &Branch, revision: &Revision) -> Answer<Landed> {
        if landing.landed(branch, revision)? {
            Ok(Landed {
                branch: branch.clone(),
                revision: revision.clone(),
            })
        } else {
            Err(Not::refused(
                &Source::new(Seam::Landing, BOUND),
                Why::NotLanded {
                    branch: branch.clone(),
                    revision: revision.clone(),
                },
                None,
            ))
        }
    }

    pub fn branch(&self) -> &Branch {
        &self.branch
    }

    pub fn revision(&self) -> &Revision {
        &self.revision
    }
}

/// 65, 82, 83: what `audience` requires at the landed revision, as the
/// policy seam answers it.
pub fn policy_at(policy: &dyn Policy, landed: &Landed, audience: &Audience) -> Answer<Required> {
    policy.policy_for(&landed.revision, audience)
}

/// 65, 131: the channels the policy declares at the landed revision, as
/// the policy seam answers them.
pub fn channels_at(policy: &dyn Policy, landed: &Landed) -> Answer<Vec<DeclaredChannel>> {
    policy.channels(&landed.revision)
}

#[cfg(test)]
mod tests {
    use tailrace_fakes::{FakeLanding, FakePolicy, PolicyOp};
    use tailrace_seams::{ChannelName, ComponentName};

    use super::*;

    #[test]
    fn the_channels_are_asked_at_the_landed_revision_and_answered_as_given() {
        let (landing, policy) = (FakeLanding::new(), FakePolicy::new());
        let (main, rev) = (Branch::new("main"), Revision::new("4f2a9c1"));
        landing.land(&main, &rev);
        let beta = ChannelName::new("beta");
        policy.require_at(
            &rev,
            Audience::Channel(beta.clone()),
            Required::Channel {
                claims: vec![],
                bound_to: None,
            },
        );
        policy.belongs_at(&rev, &beta, &ComponentName::new("lib"));

        let landed = Landed::check(&landing, &main, &rev).unwrap();
        assert!(policy.calls().is_empty(), "asked before the landing");
        assert_eq!(
            channels_at(&policy, &landed),
            Ok(vec![DeclaredChannel {
                name: beta,
                component: ComponentName::new("lib"),
                bound_to: None,
            }])
        );
        let calls = policy.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].op, PolicyOp::Channels);
        assert_eq!(calls[0].given["revision"], "4f2a9c1");
    }
}
