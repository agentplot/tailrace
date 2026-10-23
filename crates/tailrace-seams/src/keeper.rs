//! B.8 The keeper: where the ledger is held, a chain of promotion claims
//! per channel (120 to 122).

use crate::{Answer, ChannelName, Digest, Envelope};

/// 122: a keeper never removes or reorders a claim, and answers the same
/// chain to every reader for as long as it holds it.
pub trait Keeper {
    /// 120: `promotion` held at the head of `channel`'s chain, answering
    /// its envelope's digest, when `predecessor` is the digest of the
    /// claim at the head, or `None` and the chain is empty; refused
    /// [`Why::NotHead`](crate::Why::NotHead) naming the claim at the head
    /// otherwise, and the chain is unchanged.
    fn append(
        &self,
        channel: &ChannelName,
        promotion: &Envelope,
        predecessor: Option<&Digest>,
    ) -> Answer<Digest>;

    /// 121: every promotion claim of `channel` in order from the first;
    /// absent when none is held, and refused
    /// [`Why::Fork`](crate::Why::Fork) naming the claims when two name
    /// one predecessor, never answered by choosing one (105).
    fn chain(&self, channel: &ChannelName) -> Answer<Vec<Envelope>>;
}
