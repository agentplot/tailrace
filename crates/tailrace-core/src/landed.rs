//! A revision its branch has landed: the one value every read of the
//! repository at a revision takes, so nothing is read at a revision the
//! branch has not landed (14, 65, 77, rule core.22; design D2), and how
//! the landing read it.

use tailrace_seams::{
    Answer, Audience, Branch, DeclaredChannel, IsLanded, Landing, Not, Policy, Read, Required,
    Revision, Seam, Source, Why,
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
    read: Read,
}

impl Landed {
    /// 77: `revision` as landed on `branch`, asked of the landing seam;
    /// refused [`Why::NotLanded`] naming both when the branch has not
    /// landed it (14), and the landing's own answer when it cannot say.
    /// A yes read from the host's copy as last fetched is a yes, since a
    /// branch only moves forward; a no read from it is refused
    /// [`Why::NotLandedAsFetched`] naming when the copy was fetched, with
    /// the remote's words, so it is asked again once the remote answers
    /// (59, 77).
    pub fn check(landing: &dyn Landing, branch: &Branch, revision: &Revision) -> Answer<Landed> {
        let source = Source::new(Seam::Landing, BOUND);
        match landing.landed(branch, revision)? {
            IsLanded { landed: true, read } => Ok(Landed {
                branch: branch.clone(),
                revision: revision.clone(),
                read,
            }),
            IsLanded {
                landed: false,
                read: Read::Source,
            } => Err(Not::refused(
                &source,
                Why::NotLanded {
                    branch: branch.clone(),
                    revision: revision.clone(),
                },
                None,
            )),
            IsLanded {
                landed: false,
                read: Read::Kept { at, said },
            } => Err(Not::refused(
                &source,
                Why::NotLandedAsFetched {
                    branch: branch.clone(),
                    revision: revision.clone(),
                    fetched: at,
                },
                Some(said),
            )),
        }
    }

    pub fn branch(&self) -> &Branch {
        &self.branch
    }

    pub fn revision(&self) -> &Revision {
        &self.revision
    }

    /// How the landing read the revision as landed: from itself, or from
    /// the host's copy as last fetched (77).
    pub fn read(&self) -> &Read {
        &self.read
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
    use tailrace_seams::{ChannelId, ChannelName, ComponentName, Moves};

    use super::*;

    #[test]
    fn the_channels_are_asked_at_the_landed_revision_and_answered_as_given() {
        let (landing, policy) = (FakeLanding::new(), FakePolicy::new());
        let (main, rev) = (Branch::new("main"), Revision::new("4f2a9c1"));
        landing.land(&main, &rev);
        let beta = ChannelId::new(Some(ComponentName::new("lib")), ChannelName::new("beta"));
        policy.require_at(
            &rev,
            Audience::Channel(beta.clone()),
            Required::Channel {
                claims: vec![],
                bound_to: None,
                promoted_by: vec![],
                approvers: vec![],
            },
        );
        policy.follows(&beta, false);

        let landed = Landed::check(&landing, &main, &rev).unwrap();
        assert!(policy.calls().is_empty(), "asked before the landing");
        assert_eq!(
            channels_at(&policy, &landed),
            Ok(vec![DeclaredChannel {
                channel: beta,
                moves: Moves::EachPublish { prereleases: false },
            }])
        );
        let calls = policy.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].op, PolicyOp::Channels);
        assert_eq!(calls[0].given["revision"], "4f2a9c1");
    }

    fn refused(not: Not) -> (Why, Option<String>) {
        match not {
            Not::Refused(r) => {
                assert_eq!(r.source, Source::new(Seam::Landing, BOUND));
                (r.why, r.said)
            }
            other => panic!("{other} is not a refusal"),
        }
    }

    #[test]
    fn a_yes_from_either_read_is_landed_saying_how() {
        let landing = FakeLanding::new();
        let (main, rev) = (Branch::new("main"), Revision::new("4f2a9c1"));
        landing.land(&main, &rev);
        let landed = Landed::check(&landing, &main, &rev).unwrap();
        assert_eq!(
            (landed.branch(), landed.revision(), landed.read()),
            (&main, &rev, &Read::Source)
        );

        landing.unreachable("remote hung up");
        let landed = Landed::check(&landing, &main, &rev).unwrap();
        assert_eq!(landed.revision(), &rev);
        assert!(
            matches!(landed.read(), Read::Kept { said, .. } if said == "remote hung up"),
            "{:?}",
            landed.read()
        );
    }

    #[test]
    fn a_no_from_the_landing_itself_is_not_landed() {
        let landing = FakeLanding::new();
        let (main, rev) = (Branch::new("main"), Revision::new("4f2a9c1"));
        let (why, said) = refused(Landed::check(&landing, &main, &rev).unwrap_err());
        assert_eq!(
            why,
            Why::NotLanded {
                branch: main,
                revision: rev
            }
        );
        assert_eq!(said, None);
    }

    #[test]
    fn a_no_from_the_copy_is_not_landed_as_fetched_with_the_remotes_words() {
        let landing = FakeLanding::new();
        let (main, first, second) = (
            Branch::new("main"),
            Revision::new("4f2a9c1"),
            Revision::new("77b0e3d"),
        );
        landing.land(&main, &first);
        Landed::check(&landing, &main, &first).unwrap();
        landing.land(&main, &second);
        landing.unreachable("fatal: remote hung up\n");

        let (why, said) = refused(Landed::check(&landing, &main, &second).unwrap_err());
        let Why::NotLandedAsFetched {
            branch,
            revision,
            fetched,
        } = why
        else {
            panic!("{why} is not a refusal as last fetched")
        };
        assert_eq!((branch, revision), (main, second));
        assert!(fetched > 0);
        assert_eq!(said.as_deref(), Some("fatal: remote hung up\n"));
    }
}
