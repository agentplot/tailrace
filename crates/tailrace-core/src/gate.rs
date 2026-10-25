//! The gate: an offered version judged under the policy for acceptance
//! at the governed branch's landed head, accepted whole with one claim of
//! the gate's own or refused whole naming the claim found wanting, and
//! each channel that follows each publish moved by the gate itself (25
//! to 30, 94 to 96, 107, 108, rules core.4, core.7, core.14; design D1).
//! It imports nothing of publish, and publish nothing of it.

use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use tailrace_seams::{
    Answer, Branch, ChannelName, ComponentName, Digest, RequiredClaim, Revision, SignerId, Version,
};

use crate::{answer_json, Followed};

/// What told the gate of a version (95): a publish's answer, or an asker
/// or a rule the policy names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToldBy {
    Publish,
    Asked,
}

/// What the gate is asked: the version, of the one component named or
/// the only one declared, on the branch; what told it; the identity it
/// signs as, the host's own (79, 95); and the moment of the act, in
/// seconds since the Unix epoch, so the core reads no clock.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AcceptAsk {
    pub branch: Branch,
    pub component: Option<ComponentName>,
    pub version: Version,
    pub told: ToldBy,
    pub gate: SignerId,
    pub when: u64,
}

/// 28, 58: the gate's one verdict on a version, each naming the index
/// its name resolved to (rule core.4), the version's revision, and the
/// branch's landed head the policy was read at (107).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "judged", rename_all = "snake_case")]
pub enum Judged {
    /// Every artifact satisfies every required claim: `acceptance` is the
    /// envelope of the gate's claim on the index, `already` when it was
    /// there before this judgment (94), `one_host` when the gate accepted
    /// what its own identity offered (108), and each channel that follows
    /// each publish answered on its own (95).
    Accepted {
        component: ComponentName,
        version: Version,
        index: Digest,
        revision: Revision,
        policy_at: Revision,
        acceptance: Digest,
        already: bool,
        one_host: bool,
        #[serde(serialize_with = "each_channel")]
        channels: Vec<(ChannelName, Answer<Moved>)>,
    },
    /// `artifact`, the first found wanting in the index's order, lacks
    /// `wanting`, the first required claim in the policy's order it does
    /// not satisfy, for `why`; nothing was written.
    Refused {
        component: ComponentName,
        version: Version,
        index: Digest,
        revision: Revision,
        policy_at: Revision,
        artifact: Digest,
        wanting: RequiredClaim,
        why: WhyWanting,
    },
}

/// What a channel that follows each publish came to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "moved", rename_all = "snake_case")]
pub enum Moved {
    /// The channel moved `from` the version its chain's head was about,
    /// none for an empty chain, `to` the index, by the promotion whose
    /// envelope is `promotion`; `name` is how the store's name followed
    /// (39, 109).
    Promoted {
        from: Option<Digest>,
        to: Digest,
        promotion: Digest,
        name: Followed,
    },
    /// The chain's head was about the index already, and nothing was
    /// written.
    There { at: Digest },
    /// The channel's own policy refused the version: `artifact` lacks
    /// `wanting` for `why`, and the channel did not move.
    Held {
        artifact: Digest,
        wanting: RequiredClaim,
        why: WhyWanting,
    },
}

/// Why a required claim found no claim that counts for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WhyWanting {
    /// No claim attached counts for it (9, 13).
    Missing,
    /// The one claim that would count was made by `signer`, whose
    /// identity has expired (81).
    Expired { signer: SignerId },
    /// The provenance that counts was made by `signer`, the gate's own
    /// identity, which the policy does not let claim acceptance too (27,
    /// 96, 108).
    OfferedByGate { signer: SignerId },
    /// The rebuild that would count was made by `signer`, the maker of
    /// the artifact's provenance, or said the digests did not match (30).
    Unmatched { signer: SignerId },
}

/// 57: each channel's answer in the one form, naming its channel, as a
/// publish names each component's.
fn each_channel<S: Serializer>(
    channels: &[(ChannelName, Answer<Moved>)],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let answers: Vec<Value> = channels
        .iter()
        .map(|(channel, answer)| {
            let mut value = answer_json(answer);
            value["channel"] = json!(channel);
            value
        })
        .collect();
    answers.serialize(serializer)
}
