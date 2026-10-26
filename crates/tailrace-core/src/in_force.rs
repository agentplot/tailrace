//! `tailrace policy`: the policy in force at the governed branch's landed
//! head, as the policy seam answers it: what acceptance and a layout
//! require, and every channel the channels answer lists, the givens
//! beside the repository's own, each with how it moves and its own policy
//! (82, 131, C.10; design D11).
//!
//! It asks the landing and the policy alone, and writes nothing (60, rule
//! core.32). Each part, and each channel's policy, carries its own answer,
//! so an absent layout or one channel whose policy cannot be read leaves
//! the rest standing (58), and every surface renders the one answer made
//! here (57, rule core.31).

use serde::{Deserialize, Serialize, Serializer};
use tailrace_seams::{Answer, Audience, Branch, DeclaredChannel, Required, Revision, Seams};

use crate::{answer_json, channels_at, policy_at, History, Landed};

/// What the policy in force is asked: the branch whose landed head holds
/// the policy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InForceAsk {
    pub branch: Branch,
}

/// 82, 131, C.10: the policy in force at `revision`, the branch's landed
/// head, each part its own answer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InForce {
    pub branch: Branch,
    pub revision: Revision,
    #[serde(serialize_with = "one_answer")]
    pub acceptance: Answer<Required>,
    #[serde(serialize_with = "one_answer")]
    pub layout: Answer<Required>,
    #[serde(serialize_with = "one_answer")]
    pub channels: Answer<Vec<ChannelInForce>>,
}

/// 131: one channel as the channels answer lists it, with its component
/// and how it moves, and its policy, its own answer (58): the claims it
/// requires, the channel it is bound to and its promoters (32).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ChannelInForce {
    #[serde(flatten)]
    pub channel: DeclaredChannel,
    #[serde(serialize_with = "one_answer")]
    pub policy: Answer<Required>,
}

/// 57: an inner answer in the one form every surface renders.
fn one_answer<T: Serialize, S: Serializer>(
    answer: &Answer<T>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    answer_json(answer).serialize(serializer)
}

/// 65, 77, 82, 131: the policy in force at `ask.branch`'s head, once the
/// landing says the branch landed it; refused naming the branch and the
/// revision before the policy is asked where it has not (14), and the
/// history's or the landing's own answer where the head cannot be read.
/// Acceptance, a layout, the channels and each channel's policy are each
/// asked of the policy seam and answered as it answers them.
pub fn in_force(seams: &Seams, history: &History, ask: &InForceAsk) -> Answer<InForce> {
    let head = history.head(&ask.branch)?;
    let landed = Landed::check(seams.landing.as_ref(), &ask.branch, &head)?;
    let policy = seams.policy.as_ref();
    let channels = channels_at(policy, &landed).map(|listed| {
        listed
            .into_iter()
            .map(|channel| ChannelInForce {
                policy: policy_at(policy, &landed, &Audience::Channel(channel.channel.clone())),
                channel,
            })
            .collect()
    });
    Ok(InForce {
        branch: ask.branch.clone(),
        revision: head,
        acceptance: policy_at(policy, &landed, &Audience::Acceptance),
        layout: policy_at(policy, &landed, &Audience::Layout),
        channels,
    })
}
