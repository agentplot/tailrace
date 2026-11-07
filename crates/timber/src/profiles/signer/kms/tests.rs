//! Spec key-service-signer: the profile holds its key and its reach and
//! nothing else, and bound to reach no service it answers without a
//! request; its answers through KMS are `tests/kms_signer.rs`'s, over a
//! stand-in for KMS.

use std::fs;
use std::os::unix::fs::PermissionsExt;

use p256::ecdsa::signature::Signer as _;
use p256::ecdsa::SigningKey;
use serde_json::json;
use ssh_key::rand_core::OsRng;
use ssh_key::{Algorithm, LineEnding, PrivateKey};
use tempfile::TempDir;
use timber_seams::{BuildId, ClaimKind, Digest, Revision};

use super::*;
use crate::profiles::signer::host_key::HostKey;

const ARN: &str = "arn:aws:kms:us-west-2:111122223333:key/1234abcd-12ab-34cd-56ef-1234567890ab";

#[test]
fn the_profile_holds_its_key_and_its_reach_and_nothing_else() {
    let key = KeyArn::parse(ARN).unwrap();
    let Kms { signing, reach } = Kms::new(key.clone());
    assert_eq!(signing, Some(key.clone()));
    assert!(matches!(reach, Reach::Host));
    let Kms { signing, reach } = Kms::verifying();
    assert_eq!(signing, None);
    assert!(matches!(reach, Reach::Host));
    assert_eq!(Kms::new(key.clone()).key(), Some(&key));
    let Kms { signing, reach } = Kms::verifying().reaching(Reach::None);
    assert_eq!(signing, None);
    assert!(matches!(reach, Reach::None));
}

/// A report of what an environment runs, the claim status reads.
fn report() -> Claim {
    Claim {
        subject: Digest::of(b"an index"),
        kind: ClaimKind::Report,
        revision: Revision::new("4f2a9c1"),
        build: Some(BuildId::new("b7e1")),
        body: json!({ "environment": "production" }),
    }
}

/// `claim` in an envelope signed with a P-256 key, as KMS signs one, its
/// signature naming `ARN`.
fn p256_signed(claim: &Claim) -> Envelope {
    let key = SigningKey::from_slice(&[7; 32]).unwrap();
    let payload = claim.payload();
    let sig: p256::ecdsa::Signature = key.sign(&pae(IN_TOTO, &payload));
    Envelope::signed(
        IN_TOTO,
        payload,
        Signature {
            signer: SignerId::new(ARN),
            keyid: ARN.into(),
            sig: sig.to_der().as_bytes().to_vec(),
        },
    )
}

/// The signer `id`, listed by `verified_by`.
fn listed(id: &str, verified_by: &str) -> NamedSigner {
    NamedSigner {
        id: SignerId::new(id),
        verified_by: verified_by.into(),
        kinds: Vec::new(),
        checks: Vec::new(),
    }
}

#[test]
fn bound_to_reach_no_service_a_p256_signature_against_a_listed_key_is_unreadable_naming_it() {
    let envelope = p256_signed(&report());
    let signers = [
        listed("laptop", "ssh-ed25519 AAAA laptop"),
        listed("fleet", ARN),
    ];
    // Reach::None holds no HTTP client, no credentials and no endpoint:
    // the answer is made with nothing to reach KMS through.
    for profile in [
        Kms::verifying().reaching(Reach::None),
        Kms::new(KeyArn::parse(ARN).unwrap()).reaching(Reach::None),
    ] {
        match profile.verify(&envelope, &signers) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, Kms::source());
                assert_eq!(
                    u.what,
                    format!("the key {ARN} the policy lists for the signer fleet")
                );
                assert_eq!(
                    u.said.as_deref(),
                    Some(
                        "status reaches no key service, and a key-service signer's public half \
                         is read from the service until the policy carries it"
                    )
                );
            }
            other => panic!("answered {other:?}, not unreadable naming the key"),
        }
    }
}

#[test]
fn bound_to_reach_no_service_a_host_keys_envelope_is_refused_unreached() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("host");
    let key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
    key.write_openssh_file(&path, LineEnding::LF).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let laptop = SignerId::new("laptop");
    let envelope = HostKey::new(laptop.clone(), &path)
        .sign(&report(), &laptop)
        .unwrap();
    let signers = [
        listed("fleet", ARN),
        listed("laptop", &key.public_key().to_openssh().unwrap()),
    ];
    match Kms::verifying()
        .reaching(Reach::None)
        .verify(&envelope, &signers)
    {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source, Kms::source());
            assert_eq!(
                r.why,
                Why::NoNamedSigner {
                    checked: vec![SignerId::new("fleet"), laptop],
                }
            );
            let said = r.said.unwrap_or_default();
            assert!(said.contains("not a DER-encoded ECDSA signature"), "{said}");
        }
        other => panic!("answered {other:?}, not refused"),
    }
}

#[test]
fn bound_to_reach_no_service_every_sign_is_absent() {
    let key = KeyArn::parse(ARN).unwrap();
    for profile in [
        Kms::verifying().reaching(Reach::None),
        Kms::new(key.clone()).reaching(Reach::None),
    ] {
        for identity in [SignerId::new(ARN), SignerId::new("laptop")] {
            match profile.sign(&report(), &identity) {
                Err(Not::Absent(a)) => {
                    assert_eq!(a.source, Kms::source());
                    assert!(a.what.starts_with(&format!("identity {identity}")), "{a:?}");
                }
                other => panic!("signing as {identity} answered {other:?}, not absent"),
            }
        }
    }
    match Kms::new(key)
        .reaching(Reach::None)
        .sign(&report(), &SignerId::new(ARN))
    {
        Err(Not::Absent(a)) => assert!(a.what.contains("status reaches no key service"), "{a:?}"),
        other => panic!("answered {other:?}, not absent"),
    }
}
