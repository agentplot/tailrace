//! The fake signer (B.4): the identity at hand and the identities that
//! have expired. A signature is the digest of the signer's id, the
//! payload's type and the payload, so a changed claim fails verify and no
//! key is used (design D7).

use std::collections::BTreeSet;

use serde_json::json;
use tailrace_seams::{
    canonical, Answer, Claim, Digest, Envelope, Made, NamedSigner, Not, Seam, Signature, Signer,
    SignerId, Why, IN_TOTO,
};

use crate::Fake;

/// The fake signer's operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignerOp {
    Sign,
    Verify,
}

/// The identity at hand and the identities that have expired.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SignerState {
    identity: Option<SignerId>,
    expired: BTreeSet<SignerId>,
}

/// A signer in memory, with no cryptography.
pub type FakeSigner = Fake<SignerState, SignerOp>;

impl Default for FakeSigner {
    fn default() -> Self {
        Self::new()
    }
}

fn signature(signer: &SignerId, payload_type: &str, payload: &[u8]) -> Vec<u8> {
    Digest::of(&canonical(&(signer, payload_type, payload)))
        .as_str()
        .as_bytes()
        .to_vec()
}

impl FakeSigner {
    /// A signer with no identity at hand.
    pub fn new() -> Self {
        Fake::with(Seam::Signer, SignerState::default())
    }

    /// `identity` at hand, to sign as.
    pub fn hold(&self, identity: &SignerId) {
        self.state().identity = Some(identity.clone());
    }

    /// 81: `identity` has expired, and verify says so of what it signed.
    pub fn expire(&self, identity: &SignerId) {
        self.state().expired.insert(identity.clone());
    }
}

impl Signer for FakeSigner {
    fn sign(&self, claim: &Claim, identity: &SignerId) -> Answer<Envelope> {
        let given = json!({ "claim": claim, "identity": identity });
        self.ask(
            SignerOp::Sign,
            given,
            format!("the identity {identity}"),
            |s, source| {
                if s.identity.as_ref() != Some(identity) {
                    return Err(Not::absent(source, format!("identity {identity}")));
                }
                let payload = claim.payload();
                Ok(Envelope {
                    payload_type: IN_TOTO.into(),
                    signature: Signature {
                        signer: identity.clone(),
                        sig: signature(identity, IN_TOTO, &payload),
                    },
                    payload,
                })
            },
        )
    }

    fn verify(&self, attestation: &Envelope, signers: &[NamedSigner]) -> Answer<Made> {
        let given = json!({ "attestation": attestation, "signers": signers });
        let what = format!("the signature of {}", attestation.digest());
        self.ask(SignerOp::Verify, given, what, |s, source| {
            let made = signers.iter().find(|named| {
                named.id == attestation.signature.signer
                    && attestation.signature.sig
                        == signature(&named.id, &attestation.payload_type, &attestation.payload)
            });
            match made {
                Some(named) => Ok(Made {
                    signer: named.id.clone(),
                    expired: s.expired.contains(&named.id),
                }),
                None => Err(Not::refused(
                    source,
                    Why::NoNamedSigner {
                        checked: signers.iter().map(|n| n.id.clone()).collect(),
                    },
                    None,
                )),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use tailrace_seams::{ClaimKind, Revision, Source};

    use super::*;
    use crate::{Fault, Times};

    fn claim() -> Claim {
        Claim {
            subject: Digest::of(b"index"),
            kind: ClaimKind::Acceptance,
            revision: Revision::new("4f2a9c1"),
            build: None,
            body: Value::Null,
        }
    }

    fn named(id: &str) -> NamedSigner {
        NamedSigner {
            id: SignerId::new(id),
            verified_by: format!("{id}'s key"),
            kinds: vec![ClaimKind::Acceptance],
            checks: vec![],
        }
    }

    fn source() -> Source {
        Source::new(Seam::Signer, "fake")
    }

    #[test]
    fn a_claim_signed_is_verified_as_its_signers() {
        let signer = FakeSigner::new();
        let gate = SignerId::new("gate");
        signer.hold(&gate);
        let envelope = signer.sign(&claim(), &gate).unwrap();
        assert_eq!(Claim::from_envelope(&envelope), Some(claim()));
        assert_eq!(envelope.signature.signer, gate);
        assert_eq!(
            signer.verify(&envelope, &[named("host"), named("gate")]),
            Ok(Made {
                signer: gate,
                expired: false
            })
        );
    }

    #[test]
    fn no_identity_at_hand_is_absent() {
        let signer = FakeSigner::new();
        assert_eq!(
            signer.sign(&claim(), &SignerId::new("gate")),
            Err(Not::absent(&source(), "identity gate"))
        );
        signer.hold(&SignerId::new("host"));
        assert!(matches!(
            signer.sign(&claim(), &SignerId::new("gate")),
            Err(Not::Absent(_))
        ));
    }

    #[test]
    fn an_expired_signer_is_answered_expired() {
        let signer = FakeSigner::new();
        let gate = SignerId::new("gate");
        signer.hold(&gate);
        let envelope = signer.sign(&claim(), &gate).unwrap();
        signer.expire(&gate);
        assert_eq!(
            signer.verify(&envelope, &[named("gate")]),
            Ok(Made {
                signer: gate,
                expired: true
            })
        );
    }

    #[test]
    fn an_attestation_changed_after_signing_is_refused() {
        let signer = FakeSigner::new();
        let gate = SignerId::new("gate");
        signer.hold(&gate);
        let mut envelope = signer.sign(&claim(), &gate).unwrap();
        envelope.payload = Claim {
            subject: Digest::of(b"another index"),
            ..claim()
        }
        .payload();
        assert_eq!(
            signer.verify(&envelope, &[named("gate"), named("host")]),
            Err(Not::refused(
                &source(),
                Why::NoNamedSigner {
                    checked: vec![SignerId::new("gate"), SignerId::new("host")]
                },
                None
            ))
        );
    }

    #[test]
    fn a_signer_the_policy_does_not_name_made_nothing() {
        let signer = FakeSigner::new();
        let gate = SignerId::new("gate");
        signer.hold(&gate);
        let envelope = signer.sign(&claim(), &gate).unwrap();
        assert!(matches!(
            signer.verify(&envelope, &[named("host")]),
            Err(Not::Refused(r)) if r.why == Why::NoNamedSigner { checked: vec![SignerId::new("host")] }
        ));
    }

    #[test]
    fn every_operation_can_be_faulted_and_is_recorded() {
        let signer = FakeSigner::new();
        signer.hold(&SignerId::new("gate"));
        signer.fault(
            SignerOp::Sign,
            Fault::Unreadable("key service down".into()),
            Times::Once,
        );
        signer.fault(
            SignerOp::Verify,
            Fault::Refused("revoked".into()),
            Times::Once,
        );
        assert!(matches!(
            signer.sign(&claim(), &SignerId::new("gate")),
            Err(Not::Unreadable(_))
        ));
        let envelope = signer.sign(&claim(), &SignerId::new("gate")).unwrap();
        assert!(
            matches!(signer.verify(&envelope, &[named("gate")]), Err(Not::Refused(r)) if r.why == Why::Said)
        );
        assert_eq!(
            signer.calls().iter().map(|c| c.op).collect::<Vec<_>>(),
            [SignerOp::Sign, SignerOp::Sign, SignerOp::Verify]
        );
    }
}
