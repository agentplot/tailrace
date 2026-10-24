//! The signer's group: 79 to 81, and 7 and 10 as a signer keeps them
//! (B.4).

use tailrace_seams::{NamedSigner, Signer, SignerId};

use crate::{Rig, Scenario};

pub const GROUP: &str = "signer";

/// A signer, made new for each scenario.
pub trait SignerRig: Rig {
    fn signer(&self) -> &dyn Signer;

    /// The identity at hand, of the party running tailrace.
    fn identity(&self) -> SignerId;

    /// `identity` as a policy names it, for every kind of claim.
    fn named(&self, identity: &SignerId) -> NamedSigner;

    /// A signer a policy names that is not the identity at hand.
    fn stranger(&self) -> NamedSigner;

    /// No identity at hand from now.
    fn no_identity(&self);

    /// `identity` expired from now.
    fn expire(&self, identity: &SignerId);

    /// The signer made unreachable, every operation answering unreadable,
    /// and the words it answers with.
    fn unreachable(&self) -> String;
}

pub fn scenarios() -> Vec<Scenario<dyn SignerRig>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
