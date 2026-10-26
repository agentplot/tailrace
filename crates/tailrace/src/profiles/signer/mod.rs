//! The signer seam's profiles (B.4), the reading of the host's
//! `[signing]`, and the binding of every signer profile (92; design D8).

pub mod host_key;
pub mod kms;
#[cfg(test)]
mod tests;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ssh_key::PublicKey;
use tailrace_seams::{Answer, Not, Signer, SignerId};

use crate::bindings::{path_of, unreadable};
use host_key::{public_half, HostKey};
use kms::{KeyArn, Kms};

/// What the host signs with: the identity `[signing]` names and its key,
/// or the given host key where the file names no `[signing]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Signing {
    /// A host key: the signer's name and its key file's path, in the
    /// landing's own `[signing]` shape (C.8).
    HostKey { signer: SignerId, key: PathBuf },
    /// A key in AWS KMS by its key ARN, which is the signer's name (C.7).
    Kms(KeyArn),
    /// No `[signing]`: the host's given key, made on its first sign, and
    /// the name it signs as (118, C.8).
    Given { signer: SignerId, key: PathBuf },
}

/// Where the given host key is, beside the bindings file.
pub const GIVEN_KEY: &str = "signing";

impl Signing {
    /// The identity the host signs as.
    pub fn signer(&self) -> SignerId {
        match self {
            Signing::HostKey { signer, .. } | Signing::Given { signer, .. } => signer.clone(),
            Signing::Kms(key) => SignerId::new(key.as_str()),
        }
    }
}

/// What a publish or a promote signs with where the bindings file at
/// `file` names no `[signing]` (118, C.8; design D3, D4): the given host
/// key at [`GIVEN_KEY`] beside the file, signing as [`given_signer`]
/// names it from the key's public half. That public half is the one
/// file read, and nothing is made: the key is made by its first sign.
pub fn given(file: &Path) -> Answer<Signing> {
    let key = file.with_file_name(GIVEN_KEY);
    let signer = given_signer(&public_half(&key))?;
    Ok(Signing::Given { signer, key })
}

/// The given host key's signer, whose public half is at `public` (118,
/// C.8; design D4): the name that file's one line gives, its comment,
/// where the file is there, so the name is fixed once the key is made;
/// the host's own name otherwise, as [`host_signer`] cuts it.
/// Unreadable naming the file where it holds no OpenSSH public key line
/// naming a signer, with the parser's words, or naming the host's name
/// where it cannot be read.
pub fn given_signer(public: &Path) -> Answer<SignerId> {
    let shown = || format!("the public key file {}", public.display());
    let text = match fs::read_to_string(public) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return host_signer(&host_name()?),
        Err(e) => {
            return Err(Not::unreadable(
                &HostKey::source(),
                shown(),
                Some(e.to_string()),
            ))
        }
    };
    let Some(line) = text.lines().map(str::trim).find(|line| !line.is_empty()) else {
        return Err(Not::unreadable(
            &HostKey::source(),
            shown(),
            Some(format!("{} holds no key line", public.display())),
        ));
    };
    let key = PublicKey::from_openssh(line)
        .map_err(|e| Not::unreadable(&HostKey::source(), shown(), Some(e.to_string())))?;
    match key.comment().trim() {
        "" => Err(Not::unreadable(
            &HostKey::source(),
            shown(),
            Some(format!(
                "{}'s key line names no signer after its key",
                public.display()
            )),
        )),
        signer => Ok(SignerId::new(signer)),
    }
}

/// The signer a host named `name` signs as: its first label, cut at its
/// first `.`, in lower case, so `Laptop.local` is `laptop`; unreadable
/// naming the host's name where that leaves nothing.
pub fn host_signer(name: &str) -> Answer<SignerId> {
    let label = name.split('.').next().unwrap_or_default().to_lowercase();
    if label.is_empty() {
        return Err(Not::unreadable(
            &HostKey::source(),
            "the host's name",
            Some(format!(
                "the host's name {name:?} has no first label to sign as"
            )),
        ));
    }
    Ok(SignerId::new(label))
}

/// The host's name as the operating system answers it; unreadable naming
/// it, in the operating system's words, where it cannot be read.
fn host_name() -> Answer<String> {
    let mut name = [0u8; 256];
    // SAFETY: the pointer and the length are the buffer's own, and
    // gethostname writes no more than the length it is given.
    let made = unsafe { libc::gethostname(name.as_mut_ptr().cast(), name.len()) };
    if made != 0 {
        return Err(Not::unreadable(
            &HostKey::source(),
            "the host's name",
            Some(io::Error::last_os_error().to_string()),
        ));
    }
    let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
    String::from_utf8(name[..end].to_vec()).map_err(|e| {
        Not::unreadable(
            &HostKey::source(),
            "the host's name",
            Some(format!("the host's name is not UTF-8: {e}")),
        )
    })
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

/// What to bind, in order (80, 92): the profile `[signing]` names, or
/// the given host key where a publish or a promote has none, signing as
/// its signer, then the other verify-only, so a claim either kind of
/// signer made is told on any host; with no signing, both verify-only,
/// the host key first. Nothing is read or reached.
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
        Some(Signing::Given { signer, key }) => vec![
            Bound::HostKey(HostKey::given(signer.clone(), key)),
            Bound::Kms(None),
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
