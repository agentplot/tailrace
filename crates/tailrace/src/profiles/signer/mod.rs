//! The signer seam's profiles (B.4), and the binding of the host's.

pub mod host_key;
pub mod kms;

use tailrace_seams::Signer;

use crate::bindings::Signing;
use host_key::HostKey;

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
