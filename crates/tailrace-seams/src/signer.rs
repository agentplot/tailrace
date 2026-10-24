//! B.4 The signer: claims signed in the identity of the party running
//! tailrace, and attestations told apart by who made them (79 to 81).

use serde::{Deserialize, Serialize};

use crate::{Answer, Claim, Envelope, NamedSigner, SignerId};

/// 80, 81: which of the signers given made an attestation, and whether
/// its identity has expired since.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Made {
    pub signer: SignerId,
    pub expired: bool,
}

/// 81: a signer's identity is bound to the party that holds it and
/// expires with that party's standing.
pub trait Signer {
    /// 79: `claim` signed as `identity`, the identity of the party running
    /// tailrace (11); absent when that identity is not at hand.
    fn sign(&self, claim: &Claim, identity: &SignerId) -> Answer<Envelope>;

    /// 80: which of `signers` made `attestation`: a named signer one of
    /// whose signatures the envelope holds and verifies, whichever of an
    /// envelope's signatures that is (12, C.5); marked expired where its
    /// identity has expired and never answered absent for it (81);
    /// refused [`Why::NoNamedSigner`](crate::Why::NoNamedSigner) naming
    /// the signers checked when none did.
    fn verify(&self, attestation: &Envelope, signers: &[NamedSigner]) -> Answer<Made>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::round_trip;

    #[test]
    fn made_round_trips_through_json() {
        round_trip(Made {
            signer: SignerId::new("gate"),
            expired: true,
        });
    }
}
