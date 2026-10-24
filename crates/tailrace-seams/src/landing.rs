//! B.3 The landing: the repository's own landing gate, read for what it
//! landed and the verdicts it recorded (77, 78).

use serde::{Deserialize, Serialize};

use crate::{Answer, Branch, BuildId, Check, Envelope, Outcome, Revision, SignerId};

/// 78: a verdict as the landing recorded it: its check, the build
/// identity it was reached on, its outcome and its signer, with the
/// signed envelope as the landing made it, carried and never re-signed
/// (12).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verdict {
    pub check: Check,
    pub build: BuildId,
    pub outcome: Outcome,
    pub signer: SignerId,
    pub envelope: Envelope,
}

pub trait Landing {
    /// 77: whether `branch` has landed `revision`.
    fn landed(&self, branch: &Branch, revision: &Revision) -> Answer<bool>;

    /// 78: every verdict the landing recorded for `revision`, as it
    /// recorded them; empty when it recorded none, which is neither
    /// absent nor unreadable.
    fn verdicts_of(&self, revision: &Revision) -> Answer<Vec<Verdict>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{round_trip, Signature, IN_TOTO};

    #[test]
    fn a_verdict_round_trips_through_json() {
        round_trip(Verdict {
            check: Check::new("test"),
            build: BuildId::new("b7e1"),
            outcome: Outcome::Pass,
            signer: SignerId::new("landing"),
            envelope: crate::Envelope::signed(
                IN_TOTO,
                b"{}".to_vec(),
                Signature {
                    signer: SignerId::new("landing"),
                    keyid: "SHA256:l4nd".into(),
                    sig: vec![7],
                },
            ),
        });
    }
}
