//! The signer seam's profiles (B.4), the reading of the host's
//! `[signing]`, and the binding of every signer profile (92; design D8).

pub mod host_key;
pub mod kms;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use tailrace_seams::{Answer, Signer, SignerId};

use crate::bindings::{path_of, unreadable};
use host_key::HostKey;
use kms::{KeyArn, Kms};

/// `[signing]`: the identity the host signs as, and the key it signs
/// with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Signing {
    /// A host key: the signer's name and its key file's path, in the
    /// landing's own `[signing]` shape (C.8).
    HostKey { signer: SignerId, key: PathBuf },
    /// A key in AWS KMS by its key ARN, which is the signer's name (C.7).
    Kms(KeyArn),
}

impl Signing {
    /// The identity the host signs as.
    pub fn signer(&self) -> SignerId {
        match self {
            Signing::HostKey { signer, .. } => signer.clone(),
            Signing::Kms(key) => SignerId::new(key.as_str()),
        }
    }
}

/// One signer profile to bind, and the key it signs with, `None` or a
/// verify-only host key where it only verifies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Bound {
    HostKey(HostKey),
    Kms(Option<KeyArn>),
}

/// The bindings file's `[signing]` table at `file`, its `~/` paths
/// expanded against `home`: `signer` and `key`, a host key, or `kms`
/// alone, a KMS key by its key ARN; unreadable naming the file and the
/// table or the key otherwise (58, 59).
pub fn read(file: &Path, home: Option<&Path>, table: &toml::Table) -> Answer<Signing> {
    if table.contains_key("kms") {
        let others: Vec<&str> = table
            .keys()
            .map(String::as_str)
            .filter(|k| *k != "kms")
            .collect();
        if !others.is_empty() {
            return Err(unreadable(
                file,
                Some("[signing]"),
                format!(
                    "[signing] names kms with {}, and a key service signer's name is its \
                     key's: the table names kms alone",
                    others.join(" and ")
                ),
            ));
        }
        let at = "[signing] kms";
        let text = table["kms"]
            .as_str()
            .ok_or_else(|| unreadable(file, Some(at), format!("{at} is not a string")))?;
        return KeyArn::parse(text)
            .map(Signing::Kms)
            .map_err(|not| unreadable(file, Some(at), not.words()));
    }
    if let Some(other) = table
        .keys()
        .find(|k| !matches!(k.as_str(), "signer" | "key"))
    {
        return Err(unreadable(
            file,
            Some(&format!("[signing] {other}")),
            format!("[signing] names {other}, and reads signer and key alone"),
        ));
    }
    let text = |key: &str| -> Answer<&str> {
        let at = format!("[signing] {key}");
        match table.get(key) {
            Some(value) => value
                .as_str()
                .ok_or_else(|| unreadable(file, Some(&at), format!("{at} is not a string"))),
            None => Err(unreadable(
                file,
                Some(&at),
                format!("[signing] names no {key}"),
            )),
        }
    };
    let signer = text("signer")?;
    let key = path_of(file, home, "[signing] key", text("key")?)?;
    Ok(Signing::HostKey {
        signer: SignerId::new(signer),
        key,
    })
}

/// What to bind, in order (80, 92): the profile `[signing]` names,
/// signing as its signer, then the other verify-only, so a claim either
/// kind of signer made is told on any host; with no `[signing]`, both
/// verify-only, the host key first. Nothing is read or reached.
pub fn plan(signing: Option<&Signing>) -> Vec<Bound> {
    match signing {
        Some(Signing::HostKey { signer, key }) => vec![
            Bound::HostKey(HostKey::new(signer.clone(), key)),
            Bound::Kms(None),
        ],
        Some(Signing::Kms(key)) => vec![
            Bound::Kms(Some(key.clone())),
            Bound::HostKey(HostKey::verifying()),
        ],
        None => vec![Bound::HostKey(HostKey::verifying()), Bound::Kms(None)],
    }
}

/// The signers bound for the host, as [`plan`] orders them; publish signs
/// with the first.
pub fn bind(signing: Option<&Signing>) -> Vec<Box<dyn Signer>> {
    plan(signing)
        .into_iter()
        .map(|bound| -> Box<dyn Signer> {
            match bound {
                Bound::HostKey(host_key) => Box::new(host_key),
                Bound::Kms(Some(key)) => Box::new(Kms::new(key)),
                Bound::Kms(None) => Box::new(Kms::verifying()),
            }
        })
        .collect()
}
