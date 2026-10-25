//! The signer seam (B.4) over a key in AWS KMS that the host uses
//! through its own role and can never export (C.7): an asymmetric
//! `ECC_NIST_P256` signing key named by its key ARN, which is the
//! signer's name. A claim is signed by KMS over the digest of the
//! envelope's DSSE pre-authentication encoding, and told apart by the
//! key's public half, read from KMS at each verify; a disabled key's
//! signatures are told as expired (79 to 81).
//!
//! The profile is the key's ARN and how it reaches KMS, and nothing
//! else: the host's credentials are found as the AWS tools find them at
//! each call and dropped before it answers, so no credential is kept
//! (64, rule core.21).

use aws_sdk_kms::primitives::Blob;
use aws_sdk_kms::types::{
    KeyMetadata, KeySpec, KeyState, KeyUsageType, MessageType, OriginType, SigningAlgorithmSpec,
};
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::DecodePublicKey;
use sha2::{Digest as _, Sha256};
use tailrace_seams::{
    Answer, Claim, Envelope, Made, NamedSigner, Not, Seam, Signature, Signer, SignerId, Source,
    Why, IN_TOTO,
};

use super::host_key::pae;
use reach::Failure;

mod arn;
mod reach;
#[cfg(test)]
mod tests;

pub use arn::{KeyArn, NotKeyArn};
pub use reach::{https, Reach};

/// The codes KMS answers a sign with when the host may not sign with the
/// key now: the role may not use it, no key has the ARN, or the key is
/// disabled or pending deletion. 79 answers each as no identity at hand
/// (design D4).
const STANDING: [&str; 4] = [
    "AccessDeniedException",
    "NotFoundException",
    "DisabledException",
    "KMSInvalidStateException",
];

/// A signer bound to a KMS key, signing as its ARN, or bound to no key on
/// a host that only verifies; and how it reaches KMS.
#[derive(Clone, Debug)]
pub struct Kms {
    signing: Option<KeyArn>,
    reach: Reach,
}

impl Kms {
    /// The signer `key`, signing as its ARN through the host's own AWS
    /// chain. Nothing is reached until a claim is signed.
    pub fn new(key: KeyArn) -> Kms {
        Kms {
            signing: Some(key),
            reach: Reach::Host,
        }
    }

    /// The profile on a host that binds no KMS key: it verifies as a
    /// profile with a key does, by the keys the policy lists, and answers
    /// every sign absent, reaching nothing (79, 80).
    pub fn verifying() -> Kms {
        Kms {
            signing: None,
            reach: Reach::Host,
        }
    }

    /// The same profile, reaching KMS by `reach`.
    pub fn reaching(self, reach: Reach) -> Kms {
        Kms { reach, ..self }
    }

    /// The key this profile signs with, `None` where it only verifies.
    pub fn key(&self) -> Option<&KeyArn> {
        self.signing.as_ref()
    }

    /// The signer seam and this profile, which every answer names.
    pub fn source() -> Source {
        Source::new(Seam::Signer, "KMS key")
    }
}

impl Failure {
    /// The SDK's or KMS's words, whichever failed.
    fn words(self) -> String {
        match self {
            Failure::NoCredentials(words)
            | Failure::Kms { words, .. }
            | Failure::Unreached(words) => words,
        }
    }
}

/// Why the key `key` is not one this profile signs with, from what
/// DescribeKey answers of it: a usage but signing, a spec but P-256, or
/// key material that was ever outside KMS (C.7; design D2, D3).
fn unfit(key: &KeyArn, metadata: &KeyMetadata) -> Option<String> {
    let named = |value: Option<&str>| value.unwrap_or("none").to_string();
    if metadata.key_usage() != Some(&KeyUsageType::SignVerify) {
        return Some(format!(
            "the key {key} has the key usage {}; a KMS key signs claims with the usage \
             SIGN_VERIFY",
            named(metadata.key_usage().map(KeyUsageType::as_str))
        ));
    }
    if metadata.key_spec() != Some(&KeySpec::EccNistP256) {
        return Some(format!(
            "the key {key} has the key spec {}; a KMS key signs claims with the spec \
             ECC_NIST_P256",
            named(metadata.key_spec().map(KeySpec::as_str))
        ));
    }
    match metadata.origin() {
        Some(OriginType::AwsKms | OriginType::AwsCloudhsm) => None,
        Some(OriginType::External) => Some(format!(
            "the key {key} has the origin EXTERNAL: its key material was imported from outside \
             KMS, and so was outside it once; a KMS key signs claims with key material KMS made, \
             the origin AWS_KMS or AWS_CLOUDHSM"
        )),
        other => Some(format!(
            "the key {key} has the origin {}: its key material is held outside KMS; a KMS key \
             signs claims with key material KMS made, the origin AWS_KMS or AWS_CLOUDHSM",
            named(other.map(OriginType::as_str))
        )),
    }
}

impl Signer for Kms {
    /// 79: `claim` signed by KMS as the bound key's ARN, through the
    /// host's role, over the SHA-256 of the envelope's pre-authentication
    /// encoding (design D2, D3); absent for any other identity, and for
    /// every identity where no key is bound, with nothing reached (11,
    /// rule core.8). A key the host may not sign with now is no identity
    /// at hand, and a key of the wrong kind is refused (design D4).
    fn sign(&self, claim: &Claim, identity: &SignerId) -> Answer<Envelope> {
        let source = Kms::source();
        let Some(key) = &self.signing else {
            return Err(Not::absent(
                &source,
                format!(
                    "identity {identity}: no identity is at hand, since this host binds no KMS \
                     key"
                ),
            ));
        };
        if identity.as_str() != key.as_str() {
            return Err(Not::absent(&source, format!("identity {identity}")));
        }
        let absent = |words: String| Not::absent(&source, format!("identity {identity}: {words}"));
        let unreadable =
            |words: String| Not::unreadable(&source, format!("the key {key}"), Some(words));
        let failed = |failure: Failure| match failure {
            Failure::NoCredentials(words) => absent(words),
            Failure::Kms { code, words } if STANDING.contains(&code.as_str()) => absent(words),
            other => unreadable(other.words()),
        };
        let session = self.reach.open(key.region()).map_err(failed)?;
        let client = session.client(key.region());
        let described = session
            .run(client.describe_key().key_id(key.as_str()).send())
            .map_err(|e| failed(Failure::of(e)))?;
        let metadata = described
            .key_metadata()
            .ok_or_else(|| unreadable("KMS described the key with no metadata".into()))?;
        if let Some(said) = unfit(key, metadata) {
            return Err(Not::refused(&source, Why::Said, Some(said)));
        }
        let payload = claim.payload();
        let digest = Sha256::digest(pae(IN_TOTO, &payload));
        let signed = session
            .run(
                client
                    .sign()
                    .key_id(key.as_str())
                    .message(Blob::new(digest.to_vec()))
                    .message_type(MessageType::Digest)
                    .signing_algorithm(SigningAlgorithmSpec::EcdsaSha256)
                    .send(),
            )
            .map_err(|e| failed(Failure::of(e)))?;
        if signed.key_id() != Some(key.as_str()) {
            return Err(unreadable(format!(
                "KMS answered the signature as made by the key {}, not by {key}",
                signed.key_id().unwrap_or("none")
            )));
        }
        let sig = signed
            .signature()
            .ok_or_else(|| unreadable("KMS answered no signature".into()))?;
        Ok(Envelope::signed(
            IN_TOTO,
            payload,
            Signature {
                signer: SignerId::new(key.as_str()),
                keyid: key.to_string(),
                sig: sig.as_ref().to_vec(),
            },
        ))
    }

    /// 80: the first of `signers`, in order, listed by a KMS key whose
    /// public half, read from KMS now, verifies one of the envelope's
    /// signatures over exactly its payload and payload type, whichever
    /// signature that is (12, C.5); expired where the key is disabled now
    /// (81). The name the envelope gives a signer decides nothing (rule
    /// core.7). An envelope with no signature of this profile's form, or
    /// signers none of whom is listed by a KMS key, is refused reaching
    /// nothing, so the core asks the next signer bound; a key whose public
    /// half or state cannot be read is unreadable, never refused (design
    /// D7).
    fn verify(&self, attestation: &Envelope, signers: &[NamedSigner]) -> Answer<Made> {
        let source = Kms::source();
        let mut keys = Vec::new();
        for named in signers {
            match KeyArn::parse(&named.verified_by) {
                Ok(key) => keys.push((named, key)),
                Err(NotKeyArn::Broken(words)) => {
                    return Err(Not::unreadable(
                        &source,
                        format!("the key the policy lists for the signer {}", named.id),
                        Some(words),
                    ))
                }
                Err(NotKeyArn::Other(_)) => {}
            }
        }
        let refused = |said: String| {
            Not::refused(
                &source,
                Why::NoNamedSigner {
                    checked: signers.iter().map(|n| n.id.clone()).collect(),
                },
                Some(said),
            )
        };
        let sigs: Vec<_> = attestation.signatures.iter().map(read_sig).collect();
        let Some((first, key)) = keys.first() else {
            return Err(refused("no signer checked is listed by a KMS key".into()));
        };
        if !sigs.iter().any(Result::is_ok) {
            return Err(refused(not_made(attestation, &sigs)));
        }
        let listed = |named: &NamedSigner, key: &KeyArn, words: String| {
            Not::unreadable(
                &source,
                format!("the key {key} the policy lists for the signer {}", named.id),
                Some(words),
            )
        };
        let session = self
            .reach
            .open(key.region())
            .map_err(|failure| listed(first, key, failure.words()))?;
        let signed = pae(&attestation.payload_type, &attestation.payload);
        for (named, key) in &keys {
            let unreadable = |words: String| listed(named, key, words);
            let client = session.client(key.region());
            let public = session
                .run(client.get_public_key().key_id(key.as_str()).send())
                .map_err(|e| unreadable(Failure::of(e).words()))?;
            if public.key_usage() != Some(&KeyUsageType::SignVerify) {
                return Err(unreadable(format!(
                    "KMS gives the key's usage as {}; a KMS key's signatures are told with the \
                     usage SIGN_VERIFY",
                    public.key_usage().map_or("none", KeyUsageType::as_str)
                )));
            }
            if public.key_spec() != Some(&KeySpec::EccNistP256) {
                return Err(unreadable(format!(
                    "KMS gives the key's spec as {}; a KMS key's signatures are told with the \
                     spec ECC_NIST_P256",
                    public.key_spec().map_or("none", KeySpec::as_str)
                )));
            }
            let der = public
                .public_key()
                .ok_or_else(|| unreadable("KMS answered no public key".into()))?;
            let verifying = VerifyingKey::from_public_key_der(der.as_ref())
                .map_err(|e| unreadable(e.to_string()))?;
            if !sigs
                .iter()
                .flatten()
                .any(|sig| verifying.verify(&signed, sig).is_ok())
            {
                continue;
            }
            let described = session
                .run(client.describe_key().key_id(key.as_str()).send())
                .map_err(|e| unreadable(Failure::of(e).words()))?;
            let expired = match described.key_metadata().and_then(KeyMetadata::key_state) {
                Some(KeyState::Enabled) => false,
                Some(KeyState::Disabled) => true,
                Some(state) => {
                    return Err(unreadable(format!(
                        "KMS gives the key's state as {}, neither enabled nor disabled",
                        state.as_str()
                    )))
                }
                None => return Err(unreadable("KMS described the key with no state".into())),
            };
            return Ok(Made {
                signer: named.id.clone(),
                expired,
            });
        }
        Err(refused(not_made(attestation, &sigs)))
    }
}

/// One of an envelope's signatures, read as a DER-encoded ECDSA P-256
/// signature with its S made low, as `p256` verifies only those and KMS
/// does not promise one; or the words saying why it is none.
fn read_sig(held: &Signature) -> Result<p256::ecdsa::Signature, String> {
    let sig = p256::ecdsa::Signature::from_der(&held.sig).map_err(|_| {
        format!(
            "the signature the envelope names as made by {} is not a DER-encoded ECDSA \
             signature",
            held.signer
        )
    })?;
    Ok(sig.normalize_s().unwrap_or(sig))
}

/// The words of a refusal no listed key made: each signature's key id
/// and signer as the envelope gives them, so whoever reads the refusal
/// can tell whose claim it says it was (S5, 9).
fn not_made(attestation: &Envelope, sigs: &[Result<p256::ecdsa::Signature, String>]) -> String {
    let said: Vec<String> = attestation
        .signatures
        .iter()
        .zip(sigs)
        .map(|(held, sig)| match sig {
            Ok(_) => format!(
                "the envelope names the signature as made by {} with the key {}; no key \
                 listed for a signer checked made it",
                held.signer,
                if held.keyid.is_empty() {
                    "it does not name"
                } else {
                    held.keyid.as_str()
                }
            ),
            Err(said) => said.clone(),
        })
        .collect();
    if said.is_empty() {
        "the envelope holds no signature".into()
    } else {
        said.join("; ")
    }
}
