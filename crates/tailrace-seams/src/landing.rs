//! B.3 The landing: the repository's own landing gate, read for what it
//! landed and the verdicts it recorded (77, 78), each answer saying how
//! it was read: from the landing's remote, or the repository itself where
//! no remote is bound, or from the host's copy as last fetched where the
//! remote does not answer (59 to 61, 77, C.5, C.6).

use serde::{Deserialize, Serialize};

use crate::{Answer, Branch, BuildId, Check, Envelope, Outcome, Read, Revision, SignerId};

/// 78: a verdict as the landing recorded it: its check, the build
/// identity it was reached on, its outcome and its signer, with the
/// signed envelope as the landing made it, carried and never re-signed,
/// and where the landing found it, in the landing's own words (12).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verdict {
    pub check: Check,
    pub build: BuildId,
    pub outcome: Outcome,
    pub signer: SignerId,
    pub envelope: Envelope,
    pub found: String,
}

/// 77: whether a branch has landed a revision, and how that was read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IsLanded {
    pub landed: bool,
    pub read: Read,
}

/// 78: every verdict the landing recorded for a revision, and how they
/// were read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verdicts {
    pub verdicts: Vec<Verdict>,
    pub read: Read,
}

/// Each answer says how it was read: [`Read::Source`] from the landing's
/// remote, or from the repository itself where no remote is bound; and
/// where the remote does not answer, [`Read::Kept`] from the host's copy
/// as last fetched, with when that copy was fetched and the remote's own
/// words of why it did not answer, the two ways a chain says it was read
/// (59, 60, 61, 77, C.5, C.6). Where the copy holds nothing to answer
/// from, the answer is unreadable with the remote's words, never not
/// landed and never an empty set (58).
pub trait Landing {
    /// 77: whether `branch` has landed `revision`.
    fn landed(&self, branch: &Branch, revision: &Revision) -> Answer<IsLanded>;

    /// 78: every verdict the landing recorded for `revision`, as it
    /// recorded them; empty when it recorded none, which is neither
    /// absent nor unreadable.
    fn verdicts_of(&self, revision: &Revision) -> Answer<Verdicts>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{round_trip, Signature, IN_TOTO};

    fn verdict() -> Verdict {
        Verdict {
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
            found: "the landing's record of run 12".into(),
        }
    }

    fn kept() -> Read {
        Read::Kept {
            at: 1_793_000_000,
            said: "fatal: '/srv/remote.git' does not appear to be a git repository\n".into(),
        }
    }

    #[test]
    fn a_verdict_round_trips_through_json() {
        round_trip(verdict());
    }

    #[test]
    fn each_answer_round_trips_through_json_with_how_it_was_read() {
        for read in [Read::Source, kept()] {
            round_trip(IsLanded {
                landed: true,
                read: read.clone(),
            });
            round_trip(IsLanded {
                landed: false,
                read: read.clone(),
            });
            round_trip(Verdicts {
                verdicts: vec![verdict()],
                read: read.clone(),
            });
            round_trip(Verdicts {
                verdicts: vec![],
                read,
            });
        }
        let json = serde_json::to_value(IsLanded {
            landed: false,
            read: kept(),
        })
        .unwrap();
        assert_eq!(json["landed"], false, "{json}");
        assert_eq!(json["read"]["kept"]["at"], 1_793_000_000, "{json}");
        let json = serde_json::to_value(Verdicts {
            verdicts: vec![],
            read: Read::Source,
        })
        .unwrap();
        assert_eq!(json["read"], "source", "{json}");
    }
}
