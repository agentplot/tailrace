//! B.5 The policy: the authors' rule for an audience, and the channels
//! it lists with how each moves, read at a landed revision (82, 83, 131).

use serde::{Deserialize, Serialize};

use crate::{Answer, ChannelId, ChannelName, Check, ClaimKind, NamedSigner, Revision};

/// Whom a policy is for (82): the gate's acceptance, a channel with its
/// component (125), or a consumer layout.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Audience {
    Acceptance,
    Channel(ChannelId),
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
/// for acceptance has nowhere to name a channel, and the policy for a
/// channel names the channel it is bound to where it is bound (37), the
/// signers whose promotion of it counts (32), and the signers that may
/// ask for it to move, none meaning any asker the gate verifies (129,
/// C.10). The policy for a layout says whether it is the given laid
/// beneath a file that writes none (C.10), so a consumer stacks the
/// publisher's policy above the given and never above its own line
/// (113); and it carries the signers the file names, each with what it
/// may claim, so a consumer reads every claim through its file's signers
/// alone where the file names one for it, and through its host's trusted
/// signers only where the file names none (C.10, 118).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Required {
    Acceptance {
        claims: Vec<RequiredClaim>,
    },
    Channel {
        claims: Vec<RequiredClaim>,
        bound_to: Option<ChannelName>,
        promoted_by: Vec<NamedSigner>,
        approvers: Vec<NamedSigner>,
    },
    Layout {
        claims: Vec<RequiredClaim>,
        #[serde(default, skip_serializing_if = "is_false")]
        given: bool,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        signers: Vec<NamedSigner>,
    },
}

fn is_false(given: &bool) -> bool {
    !given
}

impl Required {
    pub fn claims(&self) -> &[RequiredClaim] {
        match self {
            Required::Acceptance { claims }
            | Required::Channel { claims, .. }
            | Required::Layout { claims, .. } => claims,
        }
    }

    /// 83: the channel of the same component this policy's channel is
    /// bound to; never one for acceptance or a layout.
    pub fn bound_to(&self) -> Option<&ChannelName> {
        match self {
            Required::Channel { bound_to, .. } => bound_to.as_ref(),
            Required::Acceptance { .. } | Required::Layout { .. } => None,
        }
    }

    /// 32: the signers whose promotion of this policy's channel counts,
    /// each with what it may claim; none for acceptance or a layout,
    /// which nothing promotes.
    pub fn promoted_by(&self) -> &[NamedSigner] {
        match self {
            Required::Channel { promoted_by, .. } => promoted_by,
            Required::Acceptance { .. } | Required::Layout { .. } => &[],
        }
    }

    /// 32, C.10: the signers that may ask for this policy's channel to
    /// move; none where any asker the gate verifies may (129), and none
    /// for acceptance or a layout, which nothing asks to move. They
    /// narrow who asks and never widen who signs.
    pub fn approvers(&self) -> &[NamedSigner] {
        match self {
            Required::Channel { approvers, .. } => approvers,
            Required::Acceptance { .. } | Required::Layout { .. } => &[],
        }
    }
}

/// 131: how a channel moves: on each accepted publish, as the gate's own
/// act (95), taking the releases the version rule makes and prereleases
/// too only where `prereleases` says so (C.10); by a promotion alone; or
/// bound to a channel of its own component, taking only what that
/// channel held (37, 83). One of the three, so no channel both follows
/// each publish and is bound.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "by", rename_all = "snake_case")]
pub enum Moves {
    EachPublish { prereleases: bool },
    Promotion,
    Bound { to: ChannelName },
}

/// 125, 131: a channel the policy lists, with its component, and how it
/// moves.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclaredChannel {
    pub channel: ChannelId,
    pub moves: Moves,
}

impl DeclaredChannel {
    /// 37, 125: the channel this one is bound to, of the same component;
    /// none where it moves on each publish or by a promotion alone.
    pub fn bound_to(&self) -> Option<ChannelId> {
        match &self.moves {
            Moves::Bound { to } => Some(ChannelId::new(self.channel.component.clone(), to.clone())),
            Moves::EachPublish { .. } | Moves::Promotion => None,
        }
    }
}

/// C.10: a channel the policy gives and holds off until the repository
/// switches it on, with how it would move once switched on and `switch`,
/// the line that switches it on, in the profile's own words, which the
/// core and every surface carry and never parse.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeldChannel {
    #[serde(flatten)]
    pub channel: DeclaredChannel,
    pub switch: String,
}

/// 131, C.10: the channels answer: the channels in force, and apart from
/// them the givens held off, so a surface shows both from one answer and
/// no reader takes a held given for a channel in force.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyChannels {
    /// 131: every channel in force, once each, ordered by name and then
    /// component, each with how it moves.
    pub in_force: Vec<DeclaredChannel>,
    /// C.10: every given the repository has not switched on, once each,
    /// ordered by name and then component, each with how it would move
    /// and the line that switches it on; never a channel in force.
    pub held: Vec<HeldChannel>,
}

pub trait Policy {
    /// 82: the claims `audience` requires, read from the repository at
    /// `revision`, a landed revision of the branch the policy governs
    /// (65, 107); absent where no policy is declared for it. For a
    /// channel, the signers whose promotion of it counts (32).
    fn policy_for(&self, revision: &Revision, audience: &Audience) -> Answer<Required>;

    /// 131: every channel the policy at `revision` declares or gives,
    /// once each, ordered by name and then component, each with its
    /// component (125) and how it moves (37, 83, 95); in the one
    /// component of a repository that declares one or none (123), and an
    /// empty list, never absent, where it lists no channel. Apart from
    /// them, every given it holds off that the repository has not
    /// switched on, with the line that switches it on, an empty list
    /// where it holds none (C.10); one answer, so the two never disagree.
    fn channels(&self, revision: &Revision) -> Answer<PolicyChannels>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{round_trip, ComponentName, SignerId};
    use serde_json::json;

    fn gate() -> NamedSigner {
        NamedSigner {
            id: SignerId::new("gate"),
            verified_by: "a public key".into(),
            kinds: vec![ClaimKind::Acceptance],
            checks: vec![],
        }
    }

    fn release() -> NamedSigner {
        NamedSigner {
            id: SignerId::new("release"),
            verified_by: "another public key".into(),
            kinds: vec![],
            checks: vec![],
        }
    }

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

    fn lib(name: &str) -> ChannelId {
        ChannelId::new(Some(ComponentName::new("lib")), ChannelName::new(name))
    }

    #[test]
    fn acceptance_names_no_channel_and_no_promoter() {
        let acceptance = Required::Acceptance { claims: claims() };
        assert_eq!(acceptance.bound_to(), None);
        assert_eq!(acceptance.promoted_by(), &[]);
        assert_eq!(acceptance.approvers(), &[]);
        assert_eq!(acceptance.claims(), claims().as_slice());
        let json = serde_json::to_value(&acceptance).unwrap();
        assert!(json["acceptance"].get("bound_to").is_none(), "{json}");
        assert!(json["acceptance"].get("promoted_by").is_none(), "{json}");
        assert!(json["acceptance"].get("approvers").is_none(), "{json}");
    }

    #[test]
    fn a_channels_policy_names_its_binding_and_its_promoters() {
        let stable = Required::Channel {
            claims: claims(),
            bound_to: Some(ChannelName::new("beta")),
            promoted_by: vec![gate()],
            approvers: vec![],
        };
        assert_eq!(stable.bound_to(), Some(&ChannelName::new("beta")));
        assert_eq!(stable.promoted_by(), &[gate()]);
        assert_eq!(stable.approvers(), &[]);
    }

    #[test]
    fn a_channels_approvers_round_trip_and_never_join_its_promoters() {
        let stable = Required::Channel {
            claims: claims(),
            bound_to: None,
            promoted_by: vec![gate()],
            approvers: vec![release(), gate()],
        };
        assert_eq!(stable.approvers(), &[release(), gate()]);
        assert_eq!(stable.promoted_by(), &[gate()]);
        round_trip(stable.clone());
        let json = serde_json::to_value(&stable).unwrap();
        assert_eq!(json["channel"]["approvers"][0]["id"], "release", "{json}");
    }

    #[test]
    fn every_type_round_trips_through_json() {
        round_trip(Audience::Acceptance);
        round_trip(Audience::Channel(lib("beta")));
        round_trip(Audience::Channel(ChannelId::new(
            None,
            ChannelName::new("beta"),
        )));
        round_trip(Audience::Layout);
        round_trip(claims().remove(0));
        round_trip(Required::Acceptance { claims: claims() });
        round_trip(Required::Channel {
            claims: claims(),
            bound_to: None,
            promoted_by: vec![gate()],
            approvers: vec![],
        });
        round_trip(Required::Layout {
            claims: vec![],
            given: false,
            signers: vec![],
        });
    }

    #[test]
    fn a_layout_policy_reads_back_given_or_written() {
        let given = Required::Layout {
            claims: claims(),
            given: true,
            signers: claims().remove(0).signers,
        };
        round_trip(given.clone());
        let json = serde_json::to_value(&given).unwrap();
        assert_eq!(json["layout"]["given"], true, "{json}");
        assert_eq!(json["layout"]["signers"][0]["id"], "landing", "{json}");

        let written = Required::Layout {
            claims: claims(),
            given: false,
            signers: vec![],
        };
        round_trip(written.clone());
        let json = serde_json::to_value(&written).unwrap();
        assert!(json["layout"].get("given").is_none(), "{json}");
        assert!(json["layout"].get("signers").is_none(), "{json}");
        let read: Required = serde_json::from_value(json!({ "layout": { "claims": [] } })).unwrap();
        assert_eq!(
            read,
            Required::Layout {
                claims: vec![],
                given: false,
                signers: vec![],
            }
        );
    }

    #[test]
    fn each_way_a_channel_moves_round_trips_by_what_moves_it() {
        let cases = [
            (
                Moves::EachPublish { prereleases: false },
                serde_json::json!({ "by": "each_publish", "prereleases": false }),
            ),
            (
                Moves::EachPublish { prereleases: true },
                serde_json::json!({ "by": "each_publish", "prereleases": true }),
            ),
            (Moves::Promotion, serde_json::json!({ "by": "promotion" })),
            (
                Moves::Bound {
                    to: ChannelName::new("beta"),
                },
                serde_json::json!({ "by": "bound", "to": "beta" }),
            ),
        ];
        for (moves, json) in cases {
            assert_eq!(serde_json::to_value(&moves).unwrap(), json);
            round_trip(moves.clone());
            round_trip(DeclaredChannel {
                channel: lib("stable"),
                moves,
            });
        }
    }

    #[test]
    fn a_declared_channel_names_its_component_or_none_and_how_it_moves() {
        let stable = DeclaredChannel {
            channel: lib("stable"),
            moves: Moves::Bound {
                to: ChannelName::new("beta"),
            },
        };
        assert_eq!(
            serde_json::to_value(&stable).unwrap(),
            serde_json::json!({
                "channel": { "component": "lib", "name": "stable" },
                "moves": { "by": "bound", "to": "beta" },
            })
        );
        assert_eq!(stable.bound_to(), Some(lib("beta")));

        let edge = DeclaredChannel {
            channel: ChannelId::new(None, ChannelName::new("edge")),
            moves: Moves::EachPublish { prereleases: false },
        };
        assert_eq!(
            serde_json::to_value(&edge).unwrap(),
            serde_json::json!({
                "channel": { "component": null, "name": "edge" },
                "moves": { "by": "each_publish", "prereleases": false },
            })
        );
        assert_eq!(edge.bound_to(), None);
    }

    #[test]
    fn the_channels_answer_carries_the_held_givens_apart_from_the_channels_in_force() {
        let answer = PolicyChannels {
            in_force: vec![DeclaredChannel {
                channel: ChannelId::new(None, ChannelName::new("beta")),
                moves: Moves::EachPublish { prereleases: false },
            }],
            held: vec![
                HeldChannel {
                    channel: DeclaredChannel {
                        channel: ChannelId::new(None, ChannelName::new("alpha")),
                        moves: Moves::EachPublish { prereleases: true },
                    },
                    switch: "[channels.alpha]".into(),
                },
                HeldChannel {
                    channel: DeclaredChannel {
                        channel: lib("lts"),
                        moves: Moves::Bound {
                            to: ChannelName::new("stable"),
                        },
                    },
                    switch: "[channels.lib.lts]".into(),
                },
            ],
        };
        assert_eq!(
            serde_json::to_value(&answer).unwrap(),
            serde_json::json!({
                "in_force": [{
                    "channel": { "component": null, "name": "beta" },
                    "moves": { "by": "each_publish", "prereleases": false },
                }],
                "held": [
                    {
                        "channel": { "component": null, "name": "alpha" },
                        "moves": { "by": "each_publish", "prereleases": true },
                        "switch": "[channels.alpha]",
                    },
                    {
                        "channel": { "component": "lib", "name": "lts" },
                        "moves": { "by": "bound", "to": "stable" },
                        "switch": "[channels.lib.lts]",
                    },
                ],
            })
        );
        for held in answer.held.clone() {
            round_trip(held);
        }
        round_trip(answer);
        round_trip(PolicyChannels::default());
    }
}
