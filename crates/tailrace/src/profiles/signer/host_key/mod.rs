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

use std::path::{Path, PathBuf};

use tailrace_seams::{Seam, SignerId, Source};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_encoding_is_the_dsse_specifications() {
        assert_eq!(
            pae("http://example.com/HelloWorld", b"hello world"),
            b"DSSEv1 29 http://example.com/HelloWorld 11 hello world"
        );
        assert_eq!(pae("", b""), b"DSSEv1 0  0 ");
    }

    #[test]
    fn the_payload_is_carried_byte_for_byte() {
        let payload = [0xff, 0x00, b' ', 0x80];
        let bytes = pae("t", &payload);
        assert_eq!(&bytes[..11], b"DSSEv1 1 t ");
        assert_eq!(&bytes[11..], b"4 \xff\x00 \x80");
    }

    #[test]
    fn every_answer_names_the_signer_seam_and_the_host_key() {
        let source = HostKey::source();
        assert_eq!(source.seam, Seam::Signer);
        assert_eq!(source.to_string(), "the signer (host key)");
        let host = HostKey::new(SignerId::new("host"), "/keys/host");
        assert_eq!(host.signer(), &SignerId::new("host"));
        assert_eq!(host.key(), Path::new("/keys/host"));
    }
}
