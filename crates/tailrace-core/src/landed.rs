//! A revision its branch has landed: the one value every read of the
//! repository at a revision takes, so nothing is read at a revision the
//! branch has not landed (14, 65, 77, rule core.22; design D2).

use tailrace_seams::{
    Answer, Audience, Branch, Landing, Not, Policy, Required, Revision, Seam, Source, Why,
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
