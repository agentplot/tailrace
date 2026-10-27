//! The fake landing (B.3): the revisions each branch landed and the
//! verdicts recorded for each, as a test scripts them, and what it held
//! when it last answered, which it answers as last fetched once made
//! unreachable (77, C.5, C.6).

use std::collections::{BTreeMap, BTreeSet};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;
use tailrace_seams::{
    Answer, Branch, IsLanded, Landing, Not, Read, Revision, Seam, Source, Verdict, Verdicts,
};

use crate::Fake;

/// The fake landing's operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandingOp {
    Landed,
    VerdictsOf,
}

/// What each branch landed and the verdicts recorded for each revision.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Held {
    landed: BTreeMap<Branch, BTreeSet<Revision>>,
    verdicts: BTreeMap<Revision, Vec<Verdict>>,
}

impl Held {
    fn landed(&self, branch: &Branch, revision: &Revision) -> bool {
        self.landed
            .get(branch)
            .is_some_and(|l| l.contains(revision))
    }

    fn verdicts(&self, revision: &Revision) -> Vec<Verdict> {
        self.verdicts.get(revision).cloned().unwrap_or_default()
    }
}

/// What the landing holds; its copy, what it held when it last answered
/// and when, in seconds since the Unix epoch; and the words it answers
/// with once unreachable.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LandingState {
    held: Held,
    copy: Option<(Held, u64)>,
    unreachable: Option<String>,
}

impl LandingState {
    /// `answer` over what the landing holds, its copy remade from it,
    /// while it answers; once unreachable, over the copy, [`Read::Kept`]
    /// with its time and the words, or unreadable with them where it
    /// never answered.
    fn read<T>(
        &mut self,
        source: &Source,
        what: String,
        answer: impl Fn(&Held) -> T,
    ) -> Answer<(T, Read)> {
        match (&self.unreachable, &self.copy) {
            (None, _) => {
                self.copy = Some((self.held.clone(), now()));
                Ok((answer(&self.held), Read::Source))
            }
            (Some(words), Some((copy, at))) => Ok((
                answer(copy),
                Read::Kept {
                    at: *at,
                    said: words.clone(),
                },
            )),
            (Some(words), None) => Err(Not::unreadable(source, what, Some(words.clone()))),
        }
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
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
            .held
            .landed
            .entry(branch.clone())
            .or_default()
            .insert(revision.clone());
    }

    /// `verdict` recorded for `revision`, after those recorded before.
    pub fn verdict(&self, revision: &Revision, verdict: Verdict) {
        self.state()
            .held
            .verdicts
            .entry(revision.clone())
            .or_default()
            .push(verdict);
    }

    /// The landing's remote no longer answers, for `words`, its own:
    /// every question is answered from what the landing held when it
    /// last answered, [`Read::Kept`] with the time of that answer and the
    /// words; where it never answered, unreadable with them (77, C.5,
    /// C.6).
    pub fn unreachable(&self, words: impl Into<String>) {
        self.state().unreachable = Some(words.into());
    }
}

impl Landing for FakeLanding {
    fn landed(&self, branch: &Branch, revision: &Revision) -> Answer<IsLanded> {
        let given = json!({ "branch": branch, "revision": revision });
        let what = format!("branch {branch}");
        self.ask(LandingOp::Landed, given, what.clone(), |s, source| {
            let (landed, read) = s.read(source, what, |h| h.landed(branch, revision))?;
            Ok(IsLanded { landed, read })
        })
    }

    fn verdicts_of(&self, revision: &Revision) -> Answer<Verdicts> {
        let given = json!({ "revision": revision });
        let what = format!("the verdicts of {revision}");
        self.ask(LandingOp::VerdictsOf, given, what.clone(), |s, source| {
            let (verdicts, read) = s.read(source, what, |h| h.verdicts(revision))?;
            Ok(Verdicts { verdicts, read })
        })
    }
}

#[cfg(test)]
mod tests {
    use tailrace_seams::{BuildId, Check, Envelope, Outcome, Signature, SignerId, IN_TOTO};

    use super::*;
    use crate::{Fault, Times};

    fn verdict(check: &str) -> Verdict {
        Verdict {
            check: Check::new(check),
            build: BuildId::new("b7e1"),
            outcome: Outcome::Pass,
            signer: SignerId::new("landing"),
            envelope: Envelope::signed(
                IN_TOTO,
                check.as_bytes().to_vec(),
                Signature {
                    signer: SignerId::new("landing"),
                    keyid: String::new(),
                    sig: vec![3],
                },
            ),
            found: format!("the record of {check}"),
        }
    }

    fn landed(landed: bool) -> Answer<IsLanded> {
        Ok(IsLanded {
            landed,
            read: Read::Source,
        })
    }

    fn source() -> Source {
        Source::new(Seam::Landing, "fake")
    }

    #[test]
    fn a_landing_scripted_for_one_test() {
        let landing = FakeLanding::new();
        let (main, rev) = (Branch::new("main"), Revision::new("4f2a9c1"));
        landing.land(&main, &rev);
        landing.verdict(&rev, verdict("test"));
        landing.verdict(&rev, verdict("lint"));
        assert_eq!(landing.landed(&main, &rev), landed(true));
        assert_eq!(
            landing.landed(&main, &Revision::new("other")),
            landed(false)
        );
        assert_eq!(landing.landed(&Branch::new("next"), &rev), landed(false));
        assert_eq!(
            landing.verdicts_of(&rev),
            Ok(Verdicts {
                verdicts: vec![verdict("test"), verdict("lint")],
                read: Read::Source
            })
        );
    }

    #[test]
    fn a_revision_with_no_verdicts_answers_empty() {
        let landing = FakeLanding::new();
        assert_eq!(
            landing.verdicts_of(&Revision::new("4f2a9c1")),
            Ok(Verdicts {
                verdicts: vec![],
                read: Read::Source
            })
        );
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

    #[test]
    fn unreachable_after_a_read_answers_from_the_copy() {
        let landing = FakeLanding::new();
        let (main, rev) = (Branch::new("main"), Revision::new("4f2a9c1"));
        landing.land(&main, &rev);
        landing.verdict(&rev, verdict("test"));
        let before = now();
        assert_eq!(landing.landed(&main, &rev), landed(true));
        let after = now();

        landing.unreachable("remote hung up");
        let Ok(IsLanded {
            landed: true,
            read: Read::Kept { at, said },
        }) = landing.landed(&main, &rev)
        else {
            panic!("an unreachable landing answered no landed revision from its copy")
        };
        assert!(
            (before..=after).contains(&at),
            "{at} is not the first answer's time"
        );
        assert_eq!(said, "remote hung up");
        assert_eq!(
            landing.verdicts_of(&rev),
            Ok(Verdicts {
                verdicts: vec![verdict("test")],
                read: Read::Kept {
                    at,
                    said: "remote hung up".into()
                }
            })
        );
    }

    #[test]
    fn a_landing_after_the_last_answer_is_not_landed_from_the_copy() {
        let landing = FakeLanding::new();
        let (main, first, second) = (
            Branch::new("main"),
            Revision::new("4f2a9c1"),
            Revision::new("77b0e3d"),
        );
        landing.land(&main, &first);
        landing.landed(&main, &first).unwrap();
        let before = now();
        assert_eq!(landing.landed(&main, &second), landed(false));
        let after = now();
        landing.land(&main, &second);

        landing.unreachable("remote hung up");
        let Ok(IsLanded {
            landed: false,
            read: Read::Kept { at, .. },
        }) = landing.landed(&main, &second)
        else {
            panic!(
                "a revision landed after the last answer was not answered not landed from the copy"
            )
        };
        assert!(
            (before..=after).contains(&at),
            "{at} is not the last answer's time"
        );
    }

    #[test]
    fn unreachable_before_any_answer_is_unreadable_with_its_words() {
        let landing = FakeLanding::new();
        let (main, rev) = (Branch::new("main"), Revision::new("4f2a9c1"));
        landing.land(&main, &rev);
        landing.unreachable("remote hung up");
        let words = Some("remote hung up".to_string());
        assert_eq!(
            landing.landed(&main, &rev),
            Err(Not::unreadable(&source(), "branch main", words.clone()))
        );
        assert_eq!(
            landing.verdicts_of(&rev),
            Err(Not::unreadable(&source(), "the verdicts of 4f2a9c1", words))
        );
    }

    #[test]
    fn a_fault_is_answered_before_the_copy() {
        let landing = FakeLanding::new();
        let (main, rev) = (Branch::new("main"), Revision::new("4f2a9c1"));
        landing.land(&main, &rev);
        landing.landed(&main, &rev).unwrap();
        landing.unreachable("remote hung up");
        landing.fault(
            LandingOp::Landed,
            Fault::Refused("denied".into()),
            Times::Once,
        );
        assert!(matches!(
            landing.landed(&main, &rev),
            Err(Not::Refused(r)) if r.said.as_deref() == Some("denied")
        ));
        assert!(matches!(
            landing.landed(&main, &rev),
            Ok(IsLanded {
                landed: true,
                read: Read::Kept { .. }
            })
        ));
    }
}
