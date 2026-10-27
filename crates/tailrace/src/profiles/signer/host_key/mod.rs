//! The signer seam (B.4) over an SSH key file on the host, held as
//! sluice's signers hold theirs (C.8): an Ed25519 key in OpenSSH's
//! format, with no passphrase, that only its owner may read. A claim is
//! signed as the host's own identity with an SSH signature over the
//! envelope's DSSE pre-authentication encoding, in tailrace's own
//! namespace, and told apart by the public key the policy lists for each
//! signer (79, 80).
//!
//! The profile is a signer's name and a path, and whether the key is the
//! host's given one, and nothing else (design D1): the key is read at
//! the moment it signs and dropped before sign answers, so no credential
//! is kept (64, rule core.21). The given key, which a host whose
//! bindings file names no `[signing]` signs with, is made the first time
//! it signs, with its public half beside it as a policy lists it (118,
//! C.8). A host that only verifies binds it with neither, and signs
//! nothing (79, 80).

use std::ffi::OsString;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ssh_key::rand_core::{OsRng, RngCore};
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

/// A signer bound to the key file at a path, signing as one identity, or
/// bound to no key on a host that only verifies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostKey {
    signing: Option<Signing>,
}

/// The identity a host key signs as, the path of its key file, and
/// whether it is the host's given key, made where sign finds no file.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Signing {
    signer: SignerId,
    key: PathBuf,
    given: bool,
}

/// The file beside a key file that holds its public half, as ssh-keygen
/// names it: the key's path with `.pub` added, `signing.pub` beside
/// `signing`.
pub fn public_half(key: &Path) -> PathBuf {
    let mut name = key.as_os_str().to_os_string();
    name.push(".pub");
    PathBuf::from(name)
}

impl HostKey {
    /// The host's signer `signer`, signing with the private key at `key`.
    /// Nothing is read until a claim is signed.
    pub fn new(signer: SignerId, key: impl Into<PathBuf>) -> HostKey {
        HostKey {
            signing: Some(Signing {
                signer,
                key: key.into(),
                given: false,
            }),
        }
    }

    /// The host's given key at `key`, signing as `signer` (118, C.8;
    /// design D3): a named key but for one thing, that a sign which finds
    /// no file at `key` makes one, and its public half beside it, first.
    /// Nothing is read or made until a claim is signed.
    pub fn given(signer: SignerId, key: impl Into<PathBuf>) -> HostKey {
        HostKey {
            signing: Some(Signing {
                signer,
                key: key.into(),
                given: true,
            }),
        }
    }

    /// The profile on a host that binds no key (design D6): it verifies
    /// as a profile with a key does, by the keys the policy lists, and
    /// answers every sign absent, reading no file.
    pub fn verifying() -> HostKey {
        HostKey { signing: None }
    }

    /// The identity this profile signs as, `None` where it only verifies.
    pub fn signer(&self) -> Option<&SignerId> {
        self.signing.as_ref().map(|s| &s.signer)
    }

    /// The path of the key file it signs with, `None` where it only
    /// verifies.
    pub fn key(&self) -> Option<&Path> {
        self.signing.as_ref().map(|s| s.key.as_path())
    }

    /// The signer seam and this profile, which every answer names.
    pub fn source() -> Source {
        Source::new(Seam::Signer, "host key")
    }
}

impl Signing {
    /// The given key made where no file is at its path (design D3): its
    /// directory made with its parents, mode 0700 where it is made; an
    /// Ed25519 key with no passphrase, named for the signer, written to a
    /// fresh file beside it, mode 0600, and linked into place, so of two
    /// first signs at once one key is kept and both read it; then its
    /// public half the same way, by the sign whose key was kept. A file
    /// that is there, whatever it holds, is never replaced.
    fn make(&self, source: &Source) -> Answer<()> {
        match fs::metadata(&self.key) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            _ => return Ok(()),
        }
        let dir = self
            .key
            .parent()
            .filter(|dir| !dir.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)
            .map_err(|e| self.unreadable(source, e.to_string()))?;
        let mut key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519)
            .map_err(|e| self.unreadable(source, e.to_string()))?;
        key.set_comment(self.signer.as_str());
        let text = key
            .to_openssh(LineEnding::LF)
            .map_err(|e| self.unreadable(source, e.to_string()))?;
        let kept = place(&self.key, text.as_bytes(), 0o600)
            .map_err(|e| self.unreadable(source, e.to_string()))?;
        if !kept {
            return Ok(());
        }
        let public = public_half(&self.key);
        key.public_key()
            .to_openssh()
            .map_err(|e| e.to_string())
            .and_then(|line| {
                place(&public, format!("{line}\n").as_bytes(), 0o644).map_err(|e| e.to_string())
            })
            .map_err(|said| {
                Not::unreadable(
                    source,
                    format!("the public key file {}", public.display()),
                    Some(said),
                )
            })?;
        Ok(())
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

/// `bytes` at `path` with `mode`, written to a fresh file in its
/// directory and linked into place, the fresh file removed either way:
/// `false` where a file was already there, which is kept as it is.
fn place(path: &Path, bytes: &[u8], mode: u32) -> io::Result<bool> {
    let fresh = fresh_beside(path);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(&fresh)?;
    let written = file
        .set_permissions(fs::Permissions::from_mode(mode))
        .and_then(|()| file.write_all(bytes))
        .and_then(|()| file.sync_all())
        .and_then(|()| fs::hard_link(&fresh, path));
    drop(file);
    let _ = fs::remove_file(&fresh);
    match written {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(false),
        Err(e) => Err(e),
    }
}

/// A fresh name beside `path`, for a file written before it is linked
/// into place: `.signing.<random>` beside `signing`.
fn fresh_beside(path: &Path) -> PathBuf {
    let mut name = OsString::from(".");
    name.push(path.file_name().unwrap_or_default());
    name.push(format!(".{:016x}", OsRng.next_u64()));
    path.with_file_name(name)
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
    /// dropped before the envelope is answered (64), the given key made
    /// first where no file is at its path (118); absent for any other
    /// identity, and for every identity where no key is bound, with no
    /// file touched (11, rule core.8).
    fn sign(&self, claim: &Claim, identity: &SignerId) -> Answer<Envelope> {
        let source = HostKey::source();
        let Some(signing) = &self.signing else {
            return Err(Not::absent(
                &source,
                format!(
                    "identity {identity}: no identity is at hand, since this host binds no key"
                ),
            ));
        };
        if identity != &signing.signer {
            return Err(Not::absent(&source, format!("identity {identity}")));
        }
        if signing.given {
            signing.make(&source)?;
        }
        let key = signing.read_key(&source)?;
        let payload = claim.payload();
        let sig = key
            .sign(NAMESPACE, HashAlg::Sha512, &pae(IN_TOTO, &payload))
            .and_then(|sig| sig.to_pem(LineEnding::LF))
            .map_err(|e| signing.unreadable(&source, e.to_string()))?;
        Ok(Envelope::signed(
            IN_TOTO,
            payload,
            Signature {
                signer: signing.signer.clone(),
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
