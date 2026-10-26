//! The signer seam's profiles (B.4), the binding of the host's, and the
//! host's own signer as the host trusts it for reading.

pub mod host_key;

use std::fs;

use ssh_key::PublicKey;
use tailrace_seams::{Answer, NamedSigner, Not, Signer};

use crate::bindings::Signing;
use host_key::{public_half, HostKey};

#[cfg(test)]
mod tests;

/// The signers bound for the host (92, C.8): the host key `[signing]`
/// names, signing as its signer, or where it names none the host key
/// that only verifies, by the keys the policy lists (design D6).
pub fn bind(signing: Option<&Signing>) -> Vec<Box<dyn Signer>> {
    let host_key = match signing {
        Some(signing) => HostKey::new(signing.signer.clone(), &signing.key),
        None => HostKey::verifying(),
    };
    vec![Box::new(host_key)]
}

/// The host's own signer, which the host trusts for reading, so it
/// consumes what it published itself (118, C.10): the signer `[signing]`
/// names, verified by its key's public half, the one line of the file
/// beside the key ssh-keygen names `<key>.pub`, read now and never the
/// key itself; none where the file names no `[signing]`. What it may
/// claim is the reader's to say, claim by claim. Unreadable naming the
/// public key file, in the operating system's or the parser's words,
/// where it cannot be read or holds no OpenSSH public key line.
pub fn own(signing: Option<&Signing>) -> Answer<Option<NamedSigner>> {
    let Some(signing) = signing else {
        return Ok(None);
    };
    let public = public_half(&signing.key);
    let cannot = |said: String| {
        Not::unreadable(
            &HostKey::source(),
            format!(
                "the public key file {}, which the host's own signer {} is trusted by",
                public.display(),
                signing.signer
            ),
            Some(said),
        )
    };
    let text = fs::read_to_string(&public).map_err(|e| cannot(e.to_string()))?;
    let line = text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .ok_or_else(|| cannot(format!("{} holds no key line", public.display())))?;
    PublicKey::from_openssh(line).map_err(|e| cannot(e.to_string()))?;
    Ok(Some(NamedSigner {
        id: signing.signer.clone(),
        verified_by: line.to_string(),
        kinds: Vec::new(),
        checks: Vec::new(),
    }))
}
