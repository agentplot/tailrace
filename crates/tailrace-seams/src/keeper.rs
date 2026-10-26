//! B.8 The keeper: where the ledger is held, a chain of promotion claims
//! per channel (120 to 122), and what crosses it: the body every
//! promotion claim holds (33, 39, 105), the name a channel takes in the
//! store, and how a chain was read, from the source holding it or as the
//! host last read it where that source does not answer (60, 61, C.13,
//! C.14).

use serde::{Deserialize, Serialize};

use crate::{
    Answer, ChannelId, ChannelName, Claim, ClaimKind, ComponentName, Digest, Envelope, Name,
    Revision, SignerId, Version,
};

/// A version and the index its name resolved to, which a promotion
/// moves a channel to or from (3, 33).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Moved {
    pub version: Version,
    pub index: Digest,
}

/// The revision a channel's policy was read at, and the digests of the
/// envelopes that satisfied it (33, 65).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Satisfied {
    pub revision: Revision,
    pub claims: Vec<Digest>,
}

/// 33, 39, 105: the body of every promotion claim: the channel and the
/// component, the version and index moved to, the version and index
/// moved from or none for the channel's first, the digest of the
/// promotion it follows or none, who asked, the policy it was judged
/// under, and when, in seconds since the Unix epoch. It is defined with
/// the seam because a ledger profile walks by it (C.14).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Promotion {
    pub channel: ChannelName,
    pub component: ComponentName,
    pub to: Moved,
    pub from: Option<Moved>,
    pub predecessor: Option<Digest>,
    pub asked_by: SignerId,
    pub policy: Satisfied,
    pub at: u64,
}

impl Promotion {
    /// The promotion `claim` holds, or `None` for a claim of another
    /// kind, one whose body is not a promotion's, or one about another
    /// digest than the index it moves to.
    pub fn of(claim: &Claim) -> Option<Promotion> {
        if claim.kind != ClaimKind::Promotion {
            return None;
        }
        let promotion: Promotion = serde_json::from_value(claim.body.clone()).ok()?;
        (promotion.to.index == claim.subject).then_some(promotion)
    }

    /// 39, 13: the claim about the index moved to, made for `revision`,
    /// the revision that version's tag names, with no build identity,
    /// which a promotion has none of (rule core.9).
    pub fn claim(&self, revision: Revision) -> Claim {
        Claim {
            subject: self.to.index.clone(),
            kind: ClaimKind::Promotion,
            revision,
            build: None,
            body: serde_json::to_value(self).expect("a promotion is plain data"),
        }
    }
}

/// 3, 125, C.14: the name `channel` takes in the store, `channel/` and
/// the channel as it is spelled, `channel/<name>` where no component is
/// declared and `channel/<component>/<name>` where one is, which no
/// version's name, a component's prefix and a version, nor a cache
/// entry's, `cache/` and a build identity, can be.
pub fn channel_name(channel: &ChannelId) -> Name {
    Name::new(format!("channel/{}", channel.spelled()))
}

/// How a chain was read (60, 61, C.13, C.14).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Read {
    /// From the source holding the ledger itself.
    Source,
    /// As this host last read it, at `at`, in seconds since the Unix
    /// epoch, because the source did not answer, for `said`, its own
    /// words.
    Kept { at: u64, said: String },
}

/// 121: a channel's promotion claims in order from the first, and how
/// they were read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chain {
    pub claims: Vec<Envelope>,
    pub read: Read,
}

/// 122: a keeper never removes or reorders a claim, and answers the same
/// chain to every reader for as long as it holds it. A chain is a
/// channel's with its component, so two components' channels of one
/// name are two chains (125).
pub trait Keeper {
    /// 120: `promotion` held at the head of `channel`'s chain, answering
    /// its envelope's digest, when `predecessor` is the digest of the
    /// claim at the head, or `None` and the chain is empty; refused
    /// [`Why::NotHead`](crate::Why::NotHead) naming the claim at the head
    /// otherwise, and the chain is unchanged. Never held by a copy alone:
    /// where the source holding the ledger does not answer, unreadable
    /// with its words.
    fn append(
        &self,
        channel: &ChannelId,
        promotion: &Envelope,
        predecessor: Option<&Digest>,
    ) -> Answer<Digest>;

    /// 121: every promotion claim of `channel` in order from the first;
    /// absent when none is held, and refused
    /// [`Why::Fork`](crate::Why::Fork) naming the claims when two name
    /// one predecessor, never answered by choosing one (105). Where the
    /// source does not answer, the chain as this host last read it,
    /// [`Read::Kept`] saying when and with the source's words (60, 61).
    fn chain(&self, channel: &ChannelId) -> Answer<Chain>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{round_trip, Signature, IN_TOTO};

    fn promotion() -> Promotion {
        Promotion {
            channel: ChannelName::new("stable"),
            component: ComponentName::new("lib"),
            to: Moved {
                version: Version::new("0.2.0"),
                index: Digest::of(b"0.2.0's index"),
            },
            from: Some(Moved {
                version: Version::new("0.1.0"),
                index: Digest::of(b"0.1.0's index"),
            }),
            predecessor: Some(Digest::of(b"the claim before")),
            asked_by: SignerId::new("gate"),
            policy: Satisfied {
                revision: Revision::new("4f2a9c1"),
                claims: vec![Digest::of(b"acceptance"), Digest::of(b"rebuild")],
            },
            at: 1_793_000_000,
        }
    }

    fn sealed(claim: &Claim) -> Envelope {
        Envelope::signed(
            IN_TOTO,
            claim.payload(),
            Signature {
                signer: SignerId::new("gate"),
                keyid: String::new(),
                sig: vec![1, 2, 3],
            },
        )
    }

    #[test]
    fn a_promotions_body_reads_back_from_its_envelope() {
        let made = promotion();
        let claim = made.claim(Revision::new("77b0e3d"));
        assert_eq!(claim.subject, made.to.index);
        assert_eq!(claim.kind, ClaimKind::Promotion);
        assert_eq!(claim.build, None);
        let back = Claim::from_envelope(&sealed(&claim)).unwrap();
        assert_eq!(back.revision, Revision::new("77b0e3d"));
        assert_eq!(Promotion::of(&back), Some(made));

        let first = Promotion {
            from: None,
            predecessor: None,
            ..promotion()
        };
        let back = Claim::from_envelope(&sealed(&first.claim(Revision::new("77b0e3d")))).unwrap();
        assert_eq!(Promotion::of(&back), Some(first));
    }

    #[test]
    fn a_claim_that_is_no_promotion_reads_as_none() {
        let acceptance = Claim {
            kind: ClaimKind::Acceptance,
            ..promotion().claim(Revision::new("77b0e3d"))
        };
        let back = Claim::from_envelope(&sealed(&acceptance)).unwrap();
        assert_eq!(Promotion::of(&back), None);

        let other_body = Claim {
            body: serde_json::json!({ "channel": "stable" }),
            ..promotion().claim(Revision::new("77b0e3d"))
        };
        assert_eq!(Promotion::of(&other_body), None);

        let other_subject = Claim {
            subject: Digest::of(b"another index"),
            ..promotion().claim(Revision::new("77b0e3d"))
        };
        assert_eq!(Promotion::of(&other_subject), None);
    }

    #[test]
    fn stables_name_in_the_store_carries_its_component_where_one_is_declared() {
        assert_eq!(
            channel_name(&ChannelId::new(None, ChannelName::new("stable"))),
            Name::new("channel/stable")
        );
        assert_eq!(
            channel_name(&ChannelId::new(
                Some(ComponentName::new("lib")),
                ChannelName::new("stable")
            )),
            Name::new("channel/lib/stable")
        );
    }

    #[test]
    fn every_type_round_trips_through_json() {
        round_trip(promotion());
        round_trip(Moved {
            version: Version::new("0.2.0"),
            index: Digest::of(b"index"),
        });
        round_trip(Satisfied {
            revision: Revision::new("4f2a9c1"),
            claims: vec![],
        });
        round_trip(Read::Source);
        round_trip(Read::Kept {
            at: 1_793_000_000,
            said: "fatal: '/srv/remote.git' does not appear to be a git repository\n".into(),
        });
        let claim = promotion().claim(Revision::new("77b0e3d"));
        round_trip(Chain {
            claims: vec![sealed(&claim)],
            read: Read::Source,
        });
        round_trip(Chain {
            claims: vec![],
            read: Read::Kept {
                at: 0,
                said: "connection refused".into(),
            },
        });
    }
}
