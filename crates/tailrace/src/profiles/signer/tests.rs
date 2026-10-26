//! The host's own signer as the host trusts it for reading (118, C.10):
//! read from its key's public half, over keys made fresh in a temporary
//! directory.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use serde_json::json;
use ssh_key::rand_core::OsRng;
use ssh_key::{Algorithm, LineEnding, PrivateKey};
use tailrace_seams::{Claim, ClaimKind, Digest, Not, Revision, Signer, SignerId};
use tempfile::TempDir;

use super::*;

/// A fresh Ed25519 key written to `signing` in `dir`, mode 0600, and its
/// public half beside it as ssh-keygen writes it; answers the key's path
/// and the public half's line.
fn key_and_public_half(dir: &Path) -> (PathBuf, String) {
    let key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).expect("an Ed25519 key");
    let path = dir.join("signing");
    key.write_openssh_file(&path, LineEnding::LF)
        .expect("the key file written");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("the key file's mode");
    let line = format!(
        "{} laptop@host",
        key.public_key().to_openssh().expect("the public half")
    );
    fs::write(public_half(&path), format!("{line}\n")).expect("the public half written");
    (path, line)
}

fn signing(key: &Path) -> Signing {
    Signing {
        signer: SignerId::new("laptop"),
        key: key.to_path_buf(),
    }
}

/// The words of an unreadable answer naming the public half beside `key`.
fn unreadable(answer: Answer<Option<NamedSigner>>, key: &Path) -> String {
    match answer {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, HostKey::source());
            assert_eq!(
                u.what,
                format!(
                    "the public key file {}, which the host's own signer laptop is trusted by",
                    public_half(key).display()
                )
            );
            u.said.expect("the reader's words")
        }
        other => panic!("expected unreadable, got {other:?}"),
    }
}

#[test]
fn a_host_naming_no_signing_has_no_own_signer_to_trust() {
    assert_eq!(own(None), Ok(None));
}

#[test]
fn the_own_signer_is_named_by_its_public_half_and_may_claim_nothing_of_itself() {
    let dir = TempDir::new().unwrap();
    let (key, line) = key_and_public_half(dir.path());
    assert_eq!(
        own(Some(&signing(&key))),
        Ok(Some(NamedSigner {
            id: SignerId::new("laptop"),
            verified_by: line,
            kinds: vec![],
            checks: vec![],
        }))
    );
}

#[test]
fn a_claim_the_host_signed_verifies_as_its_own_signer_by_its_public_half() {
    let dir = TempDir::new().unwrap();
    let (key, _) = key_and_public_half(dir.path());
    let claim = Claim {
        subject: Digest::of(b"an index"),
        kind: ClaimKind::Acceptance,
        revision: Revision::new("4f2a9c1"),
        build: None,
        body: json!({}),
    };
    let host = HostKey::new(SignerId::new("laptop"), &key);
    let envelope = host.sign(&claim, &SignerId::new("laptop")).unwrap();
    // Read with the private key gone: the public half alone verifies.
    fs::remove_file(&key).unwrap();
    let mut trusted = own(Some(&signing(&key))).unwrap().unwrap();
    trusted.kinds = vec![ClaimKind::Acceptance];
    let made = HostKey::verifying()
        .verify(&envelope, std::slice::from_ref(&trusted))
        .unwrap();
    assert_eq!(made.signer, SignerId::new("laptop"));
}

#[test]
fn a_missing_or_broken_public_half_is_unreadable_naming_it() {
    let dir = TempDir::new().unwrap();
    let (key, _) = key_and_public_half(dir.path());

    fs::remove_file(public_half(&key)).unwrap();
    let said = unreadable(own(Some(&signing(&key))), &key);
    assert!(said.contains("No such file"), "{said}");

    fs::write(public_half(&key), "\n\n").unwrap();
    let said = unreadable(own(Some(&signing(&key))), &key);
    assert_eq!(
        said,
        format!("{} holds no key line", public_half(&key).display())
    );

    fs::write(public_half(&key), "ssh-ed25519 not-a-key laptop@host\n").unwrap();
    let said = unreadable(own(Some(&signing(&key))), &key);
    assert!(!said.is_empty());
}
