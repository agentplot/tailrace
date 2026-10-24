//! The signer seam (B.4) over an SSH key file on the host, held as
//! sluice's signers hold theirs (C.8): an Ed25519 key in OpenSSH's
//! format, with no passphrase, that only its owner may read. A claim is
//! signed as the host's own identity with an SSH signature over the
//! envelope's DSSE pre-authentication encoding, in tailrace's own
//! namespace, and told apart by the public key the policy lists for each
//! signer (79, 80).
//!
//! The profile is a signer's name and a path, and nothing else (design
//! D1): the key is read at the moment it signs and dropped before sign
//! answers, so no credential is kept (64, rule core.21).

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ssh_key::{Algorithm, HashAlg, LineEnding, PrivateKey, SshSig};
use tailrace_seams::{
    Answer, Claim, Envelope, Made, NamedSigner, Not, Seam, Signature, Signer, SignerId, Source,
    Why, IN_TOTO,
};

mod listed;
#[cfg(test)]
mod tests;

/// The SSH signature namespace every claim is signed in, so a signature
/// made for anything else, a run of sluice's or a commit of git's, never
/// verifies as a claim (design D3).
pub const NAMESPACE: &str = "tailrace-claim";

/// A signer bound to the key file at a path, signing as one identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostKey {
    signer: SignerId,
    key: PathBuf,
}

impl HostKey {
    /// The host's signer `signer`, signing with the private key at `key`.
    /// Nothing is read until a claim is signed.
    pub fn new(signer: SignerId, key: impl Into<PathBuf>) -> HostKey {
        HostKey {
            signer,
            key: key.into(),
        }
    }

    /// The identity this profile signs as.
    pub fn signer(&self) -> &SignerId {
        &self.signer
    }

    /// The path of the key file it signs with.
    pub fn key(&self) -> &Path {
        &self.key
    }

    /// The signer seam and this profile, which every answer names.
    pub fn source() -> Source {
        Source::new(Seam::Signer, "host key")
    }

    /// The key from its file, read now and checked in sluice's order
    /// (design D2): there, a regular file, only its owner may read it, an
    /// OpenSSH private key, no passphrase, Ed25519.
    fn read_key(&self, source: &Source) -> Answer<PrivateKey> {
        let path = self.key.display();
        let meta = match fs::metadata(&self.key) {
            Ok(meta) => meta,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(Not::absent(
                    source,
                    format!("identity {}: no key file at {path}", self.signer),
                ));
            }
            Err(e) => return Err(self.unreadable(source, e.to_string())),
        };
        if !meta.is_file() {
            // A directory's read fails at once, in the operating system's
            // words; a fifo's would wait for a writer, so no other kind
            // is read.
            let said = meta
                .is_dir()
                .then(|| fs::read(&self.key).err())
                .flatten()
                .map(|e| e.to_string());
            return Err(self.unreadable(
                source,
                said.unwrap_or_else(|| format!("{path} is not a regular file")),
            ));
        }
        let mode = meta.permissions().mode() & 0o777;
        if mode & 0o077 != 0 {
            return Err(Not::refused(
                source,
                Why::Said,
                Some(format!(
                    "the key file {path} has mode {mode:04o}, which lets others read it: \
                     give it mode 0600 (chmod 600 {path})"
                )),
            ));
        }
        let mut text = fs::read(&self.key).map_err(|e| self.unreadable(source, e.to_string()))?;
        let parsed = PrivateKey::from_openssh(&text);
        wipe(&mut text);
        let key = parsed.map_err(|e| self.unreadable(source, e.to_string()))?;
        if key.is_encrypted() {
            return Err(Not::refused(
                source,
                Why::Said,
                Some(format!(
                    "the key file {path} has a passphrase; tailrace signs with none to type"
                )),
            ));
        }
        if key.algorithm() != Algorithm::Ed25519 {
            return Err(Not::refused(
                source,
                Why::Said,
                Some(format!(
                    "the key file {path} holds an {} key; tailrace signs with {} keys",
                    key.algorithm(),
                    Algorithm::Ed25519
                )),
            ));
        }
        Ok(key)
    }

    /// The key file could not be read, or what it holds could not be
    /// parsed or used: `said` the operating system's or ssh-key's words.
    fn unreadable(&self, source: &Source, said: String) -> Not {
        Not::unreadable(
            source,
            format!("the key file {}", self.key.display()),
            Some(said),
        )
    }
}

/// The key file's text overwritten before it is dropped, as ssh-key's own
/// read of a key file does, so nothing of the key outlives the signature
/// (64).
fn wipe(text: &mut [u8]) {
    text.fill(0);
    std::hint::black_box(text);
}

impl Signer for HostKey {
    /// 79: `claim` signed as the bound signer, with the key read now and
    /// dropped before the envelope is answered (64); absent for any other
    /// identity, with no file touched (11, rule core.8).
    fn sign(&self, claim: &Claim, identity: &SignerId) -> Answer<Envelope> {
        let source = HostKey::source();
        if identity != &self.signer {
            return Err(Not::absent(&source, format!("identity {identity}")));
        }
        let key = self.read_key(&source)?;
        let payload = claim.payload();
        let sig = key
            .sign(NAMESPACE, HashAlg::Sha512, &pae(IN_TOTO, &payload))
            .and_then(|sig| sig.to_pem(LineEnding::LF))
            .map_err(|e| self.unreadable(&source, e.to_string()))?;
        Ok(Envelope::signed(
            IN_TOTO,
            payload,
            Signature {
                signer: self.signer.clone(),
                keyid: String::new(),
                sig: sig.into_bytes(),
            },
        ))
    }

    /// 80: the first of `signers`, in order, whose listed key made a
    /// valid signature in tailrace's namespace over exactly this envelope,
    /// whichever of the envelope's signatures that is (12, C.5), expired
    /// once its listing's `valid-before` has passed (81). No key file is
    /// read and nothing is reached, and the name the envelope gives a
    /// signer decides nothing (rule core.7). A signature this profile
    /// cannot read is refused, never unreadable, so the core asks the next
    /// signer bound (design D7).
    fn verify(&self, attestation: &Envelope, signers: &[NamedSigner]) -> Answer<Made> {
        let source = HostKey::source();
        let mut keys = Vec::new();
        for named in signers {
            if let Some(key) = listed::read(named)? {
                keys.push((named, key));
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
        let signed = pae(&attestation.payload_type, &attestation.payload);
        let sigs: Vec<_> = attestation.signatures.iter().map(read_sig).collect();
        let made = keys.iter().find(|(_, listed)| {
            sigs.iter().flatten().any(|sig| {
                listed.key.key_data() == sig.public_key()
                    && listed.key.verify(NAMESPACE, &signed, sig).is_ok()
            })
        });
        if let Some((named, listed)) = made {
            return Ok(Made {
                signer: named.id.clone(),
                expired: listed.valid_before.is_some_and(|before| now() >= before),
            });
        }
        let said: Vec<_> = attestation
            .signatures
            .iter()
            .zip(sigs)
            .map(|(held, sig)| match sig {
                Ok(sig) => format!(
                    "signed by the key {}, which the envelope names {}; no signer checked \
                     lists that key with a signature over this envelope",
                    sig.public_key().fingerprint(HashAlg::Sha256),
                    held.signer
                ),
                Err(said) => said,
            })
            .collect();
        Err(refused(if said.is_empty() {
            "the envelope holds no signature".into()
        } else {
            said.join("; ")
        }))
    }
}

/// One of an envelope's signatures, read as an SSH signature in
/// tailrace's namespace, or the words saying why it is none.
fn read_sig(held: &Signature) -> Result<SshSig, String> {
    let sig = std::str::from_utf8(&held.sig)
        .ok()
        .and_then(|pem| SshSig::from_pem(pem).ok())
        .ok_or_else(|| "the signature is not an SSH signature".to_string())?;
    if sig.namespace() != NAMESPACE {
        return Err(format!(
            "the signature was made in the namespace {}, not {NAMESPACE}",
            sig.namespace()
        ));
    }
    Ok(sig)
}

/// Seconds since the epoch, now.
fn now() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(after) => i64::try_from(after.as_secs()).unwrap_or(i64::MAX),
        Err(before) => -i64::try_from(before.duration().as_secs()).unwrap_or(i64::MAX),
    }
}

/// DSSE's pre-authentication encoding of a payload and its type, the
/// bytes a signature is made over (design D4):
/// `DSSEv1 <len(type)> <type> <len(payload)> <payload>`.
pub fn pae(payload_type: &str, payload: &[u8]) -> Vec<u8> {
    let mut bytes = format!(
        "DSSEv1 {} {payload_type} {} ",
        payload_type.len(),
        payload.len()
    )
    .into_bytes();
    bytes.extend_from_slice(payload);
    bytes
}
