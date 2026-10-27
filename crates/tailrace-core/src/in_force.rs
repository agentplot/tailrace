//! `tailrace policy`: the policy in force at the governed branch's landed
//! head, as the policy seam answers it: what acceptance and a layout
//! require, and every channel the channels answer lists, the givens
//! beside the repository's own, each with how it moves and its own policy,
//! and apart from them the givens held off, each with the line that
//! switches it on, from the same one answer (82, 131, C.10; design D11,
//! D13).
//!
//! It asks the landing and the policy alone, and writes nothing (60, rule
//! core.32). Each part, and each channel's policy, carries its own answer,
//! so an absent layout, one channel whose policy cannot be read, or a
//! channels answer that cannot be read leaves the rest standing (58), and
//! every surface renders the one answer made here (57, rule core.31).

use serde::{Deserialize, Serialize, Serializer};
use tailrace_seams::{
    Answer, Audience, Branch, DeclaredChannel, HeldChannel, Required, Revision, Seams,
};

use crate::{answer_json, policy_at, policy_channels_at, History, Landed};

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
    pub channels: Answer<ChannelsInForce>,
}

/// 131, C.10: the one channels answer, each channel in force given its
/// policy, and the givens held off as the answer carries them, apart.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ChannelsInForce {
    pub in_force: Vec<ChannelInForce>,
    pub held: Vec<HeldChannel>,
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
/// Acceptance, a layout, the channels with the givens held off, and each
/// channel's policy are each asked of the policy seam and answered as it
/// answers them.
pub fn in_force(seams: &Seams, history: &History, ask: &InForceAsk) -> Answer<InForce> {
    let head = history.head(&ask.branch)?;
    let landed = Landed::check(seams.landing.as_ref(), &ask.branch, &head)?;
    let policy = seams.policy.as_ref();
    let channels = policy_channels_at(policy, &landed).map(|answer| ChannelsInForce {
        in_force: answer
            .in_force
            .into_iter()
            .map(|channel| ChannelInForce {
                policy: policy_at(policy, &landed, &Audience::Channel(channel.channel.clone())),
                channel,
            })
            .collect(),
        held: answer.held,
    });
    Ok(InForce {
        branch: ask.branch.clone(),
        revision: head,
        acceptance: policy_at(policy, &landed, &Audience::Acceptance),
        layout: policy_at(policy, &landed, &Audience::Layout),
        channels,
    })
}
