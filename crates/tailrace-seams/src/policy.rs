//! B.5 The policy: the authors' rule for an audience, read at a landed
//! revision (82, 83).

use serde::{Deserialize, Serialize};

use crate::{Answer, ChannelName, Check, ClaimKind, NamedSigner, Revision};

/// Whom a policy is for (82): the gate's acceptance, a channel, or a
/// consumer layout.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Audience {
    Acceptance,
    Channel(ChannelName),
    Layout,
}

/// 82: one claim a policy requires: its kind, the check it is for where
/// it is a verdict, and the signers that may make it, each with what it
/// may claim.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequiredClaim {
    pub kind: ClaimKind,
    pub check: Option<Check>,
    pub signers: Vec<NamedSigner>,
}

/// 82, 83: the claims an audience requires, by audience, so the policy
/// for acceptance has nowhere to name a channel and the policy for a
/// channel names the channel it is bound to where it is bound (37).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Required {
    Acceptance {
        claims: Vec<RequiredClaim>,
    },
    Channel {
        claims: Vec<RequiredClaim>,
        bound_to: Option<ChannelName>,
    },
    Layout {
        claims: Vec<RequiredClaim>,
    },
}

impl Required {
    pub fn claims(&self) -> &[RequiredClaim] {
        match self {
            Required::Acceptance { claims }
            | Required::Channel { claims, .. }
            | Required::Layout { claims } => claims,
        }
    }

    /// 83: the channel this policy's channel is bound to; never one for
    /// acceptance or a layout.
    pub fn bound_to(&self) -> Option<&ChannelName> {
        match self {
            Required::Channel { bound_to, .. } => bound_to.as_ref(),
            Required::Acceptance { .. } | Required::Layout { .. } => None,
        }
    }
}

pub trait Policy {
    /// 82: the claims `audience` requires, read from the repository at
    /// `revision`, a landed revision of the branch the policy governs
    /// (65, 107); absent where no policy is declared for it.
    fn policy_for(&self, revision: &Revision, audience: &Audience) -> Answer<Required>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{round_trip, SignerId};

    fn claims() -> Vec<RequiredClaim> {
        vec![RequiredClaim {
            kind: ClaimKind::Verdict,
            check: Some(Check::new("test")),
            signers: vec![NamedSigner {
                id: SignerId::new("landing"),
                verified_by: "a public key".into(),
                kinds: vec![ClaimKind::Verdict],
                checks: vec![Check::new("test")],
            }],
        }]
    }

    #[test]
    fn acceptance_names_no_channel() {
        let acceptance = Required::Acceptance { claims: claims() };
        assert_eq!(acceptance.bound_to(), None);
        assert_eq!(acceptance.claims(), claims().as_slice());
        let json = serde_json::to_value(&acceptance).unwrap();
        assert!(json["acceptance"].get("bound_to").is_none(), "{json}");
    }

    #[test]
    fn a_bound_channel_names_its_channel() {
        let stable = Required::Channel {
            claims: claims(),
            bound_to: Some(ChannelName::new("beta")),
        };
        assert_eq!(stable.bound_to(), Some(&ChannelName::new("beta")));
    }

    #[test]
    fn every_type_round_trips_through_json() {
        round_trip(Audience::Acceptance);
        round_trip(Audience::Channel(ChannelName::new("beta")));
        round_trip(Audience::Layout);
        round_trip(claims().remove(0));
        round_trip(Required::Acceptance { claims: claims() });
        round_trip(Required::Channel {
            claims: claims(),
            bound_to: None,
        });
        round_trip(Required::Layout { claims: vec![] });
    }
}
