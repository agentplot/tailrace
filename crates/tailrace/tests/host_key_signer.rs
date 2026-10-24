//! Spec host-key-signer: the conformance suite's signer group over a host
//! key, a fresh key in a temporary directory for each scenario (design
//! D9).

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use ssh_key::rand_core::OsRng;
use ssh_key::{Algorithm, LineEnding, PrivateKey};
use tailrace::profiles::signer::host_key::HostKey;
use tailrace_conformance::seams::signer::{self, SignerRig};
use tailrace_conformance::{not_applicable, run, Rig};
use tailrace_seams::{ClaimKind, NamedSigner, Signer, SignerId};
use tempfile::TempDir;

/// Why the scenario of a signer that cannot be reached is not one a host
/// key plays.
const UNREACHABLE: &str = "a host key's verify reads only the public keys the policy lists and \
                           reaches nothing; an unreadable key file is the profile's own test";

/// The identity the rig's host key signs as.
const HOST: &str = "host";

/// A host key bound as `host` to a fresh key in a temporary directory, and
/// a second key for the stranger.
struct HostKeyRig {
    _dir: TempDir,
    path: PathBuf,
    signer: HostKey,
    key: PrivateKey,
    stranger: PrivateKey,
    expired: RefCell<BTreeSet<SignerId>>,
}

fn new_key() -> PrivateKey {
    PrivateKey::random(&mut OsRng, Algorithm::Ed25519).expect("an Ed25519 key")
}

impl HostKeyRig {
    fn new() -> Self {
        let dir = TempDir::new().expect("a temporary directory");
        let path = dir.path().join("host");
        let key = new_key();
        key.write_openssh_file(&path, LineEnding::LF)
            .expect("the key file written");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
            .expect("the key file's mode set");
        HostKeyRig {
            signer: HostKey::new(SignerId::new(HOST), &path),
            _dir: dir,
            path,
            key,
            stranger: new_key(),
            expired: RefCell::new(BTreeSet::new()),
        }
    }

    /// `id` as a policy lists it, verified by `key`'s public half, valid
    /// before a time long past once `id` has expired.
    fn listed(&self, id: &SignerId, key: &PrivateKey) -> NamedSigner {
        let public = key.public_key().to_openssh().expect("the public key");
        let verified_by = if self.expired.borrow().contains(id) {
            format!("valid-before=\"20000101Z\" {public}")
        } else {
            public
        };
        NamedSigner {
            id: id.clone(),
            verified_by,
            kinds: vec![
                ClaimKind::Provenance,
                ClaimKind::Verdict,
                ClaimKind::Test,
                ClaimKind::Review,
                ClaimKind::Rebuild,
                ClaimKind::Acceptance,
                ClaimKind::Promotion,
                ClaimKind::Carried,
            ],
            checks: vec![],
        }
    }
}

impl Rig for HostKeyRig {
    fn profile(&self) -> String {
        HostKey::source().profile
    }
}

impl SignerRig for HostKeyRig {
    fn signer(&self) -> &dyn Signer {
        &self.signer
    }

    fn identity(&self) -> SignerId {
        SignerId::new(HOST)
    }

    fn named(&self, identity: &SignerId) -> NamedSigner {
        self.listed(identity, &self.key)
    }

    fn stranger(&self) -> NamedSigner {
        self.listed(&SignerId::new("a stranger"), &self.stranger)
    }

    fn no_identity(&self) {
        fs::remove_file(&self.path).expect("the key file removed");
    }

    fn expire(&self, identity: &SignerId) {
        self.expired.borrow_mut().insert(identity.clone());
    }

    fn unreachable(&self) -> String {
        not_applicable(UNREACHABLE)
    }
}

#[test]
fn the_host_key_keeps_the_signers_contract() {
    let report = run(signer::GROUP, &signer::scenarios(), &|| {
        Box::new(HostKeyRig::new())
    });
    report.assert();
    assert_eq!(report.profile, "host key");
    assert_eq!(report.ran, signer::scenarios().len());
    let not_applicable: Vec<(&str, &str)> = report
        .not_applicable
        .iter()
        .map(|n| (n.scenario, n.why.as_str()))
        .collect();
    assert_eq!(
        not_applicable,
        [("a signer that cannot be reached", UNREACHABLE)],
        "{report}"
    );
}
