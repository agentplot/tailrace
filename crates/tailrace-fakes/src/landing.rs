//! The fake landing (B.3): the revisions each branch landed and the
//! verdicts recorded for each, as a test scripts them.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::json;
use tailrace_seams::{Answer, Branch, Landing, Revision, Seam, Verdict};

use crate::Fake;

/// The fake landing's operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandingOp {
    Landed,
    VerdictsOf,
}

/// What each branch landed and the verdicts recorded for each revision.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LandingState {
    landed: BTreeMap<Branch, BTreeSet<Revision>>,
    verdicts: BTreeMap<Revision, Vec<Verdict>>,
}

/// A landing in memory.
pub type FakeLanding = Fake<LandingState, LandingOp>;

impl Default for FakeLanding {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeLanding {
    pub fn new() -> Self {
        Fake::with(Seam::Landing, LandingState::default())
    }

    /// `branch` has landed `revision`.
    pub fn land(&self, branch: &Branch, revision: &Revision) {
        self.state()
            .landed
            .entry(branch.clone())
            .or_default()
            .insert(revision.clone());
    }

    /// `verdict` recorded for `revision`, after those recorded before.
    pub fn verdict(&self, revision: &Revision, verdict: Verdict) {
        self.state()
            .verdicts
            .entry(revision.clone())
            .or_default()
            .push(verdict);
    }
}

impl Landing for FakeLanding {
    fn landed(&self, branch: &Branch, revision: &Revision) -> Answer<bool> {
        let given = json!({ "branch": branch, "revision": revision });
        self.ask(
            LandingOp::Landed,
            given,
            format!("branch {branch}"),
            |s, _| Ok(s.landed.get(branch).is_some_and(|l| l.contains(revision))),
        )
    }

    fn verdicts_of(&self, revision: &Revision) -> Answer<Vec<Verdict>> {
        let given = json!({ "revision": revision });
        self.ask(
            LandingOp::VerdictsOf,
            given,
            format!("the verdicts of {revision}"),
            |s, _| Ok(s.verdicts.get(revision).cloned().unwrap_or_default()),
        )
    }
}

#[cfg(test)]
mod tests {
    use tailrace_seams::{BuildId, Check, Envelope, Not, Outcome, Signature, SignerId, IN_TOTO};

    use super::*;
    use crate::{Fault, Times};

    fn verdict(check: &str) -> Verdict {
        Verdict {
            check: Check::new(check),
            build: BuildId::new("b7e1"),
            outcome: Outcome::Pass,
            signer: SignerId::new("landing"),
            envelope: Envelope {
                payload_type: IN_TOTO.into(),
                payload: check.as_bytes().to_vec(),
                signature: Signature {
                    signer: SignerId::new("landing"),
                    sig: vec![3],
                },
            },
        }
    }

    #[test]
    fn a_landing_scripted_for_one_test() {
        let landing = FakeLanding::new();
        let (main, rev) = (Branch::new("main"), Revision::new("4f2a9c1"));
        landing.land(&main, &rev);
        landing.verdict(&rev, verdict("test"));
        landing.verdict(&rev, verdict("lint"));
        assert_eq!(landing.landed(&main, &rev), Ok(true));
        assert_eq!(landing.landed(&main, &Revision::new("other")), Ok(false));
        assert_eq!(landing.landed(&Branch::new("next"), &rev), Ok(false));
        assert_eq!(
            landing.verdicts_of(&rev),
            Ok(vec![verdict("test"), verdict("lint")])
        );
    }

    #[test]
    fn a_revision_with_no_verdicts_answers_empty() {
        let landing = FakeLanding::new();
        assert_eq!(landing.verdicts_of(&Revision::new("4f2a9c1")), Ok(vec![]));
    }

    #[test]
    fn every_operation_can_be_faulted_and_is_recorded() {
        let landing = FakeLanding::new();
        let rev = Revision::new("4f2a9c1");
        landing.fault(
            LandingOp::Landed,
            Fault::Refused("denied".into()),
            Times::Once,
        );
        landing.fault(
            LandingOp::VerdictsOf,
            Fault::Unreadable("timed out".into()),
            Times::Once,
        );
        assert!(matches!(
            landing.landed(&Branch::new("main"), &rev),
            Err(Not::Refused(_))
        ));
        assert!(
            matches!(landing.verdicts_of(&rev), Err(Not::Unreadable(u)) if u.said.as_deref() == Some("timed out"))
        );
        assert_eq!(
            landing.calls().iter().map(|c| c.op).collect::<Vec<_>>(),
            [LandingOp::Landed, LandingOp::VerdictsOf]
        );
    }
}
