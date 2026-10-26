//! The policy seam (B.5) bound to the policy file (C.10):
//! `.config/tailrace.toml` read with git from the tree of the revision
//! asked, never from a checkout (65, rule core.22; design D3). It reads
//! its own tables and leaves the core's (`outputs`, `[version]`,
//! `[projections]`, `[[component]]`) and every other reader's alone:
//!
//! ```toml
//! [signers.laptop]
//! ssh = "ssh-ed25519 AAAA… host@laptop"
//! kinds = ["provenance"]
//!
//! [signers.box]
//! ssh = "ssh-ed25519 BBBB…"
//! kinds = ["verdict"]
//! checks = ["cli:test"]
//!
//! [acceptance]
//! require = [
//!   { kind = "provenance", signers = ["laptop"] },
//!   { kind = "verdict", check = "cli:test", signers = ["box"] },
//! ]
//!
//! [layout]
//! require = [{ kind = "provenance", signers = ["laptop"] }]
//!
//! [channels.stable]
//! bound_to = "beta"
//! require = [{ kind = "acceptance", signers = ["gate"] }]
//! ```
//!
//! A signer's table names exactly one key besides `kinds` and `checks`,
//! whose text is what its signer profile verifies it by, handed on whole
//! so each signer profile reads its own (82, 116). Each audience's
//! `require` keeps its order, which fixes the first claim found wanting
//! (58). A channel's keys other than `require` and `bound_to` are left
//! to the reader that owns them.
//!
//! A file with no `[layout]` answers for a layout the given policy for
//! one, a version the gate accepted and no other (C.10): acceptance by
//! every signer `[signers]` lets claim acceptance, in the file's order;
//! where it names none and exactly one that may claim provenance, by that
//! one, as the one host 108 admits; otherwise by no signer, so the given
//! admits no version. A `[layout]` written is answered as written, with
//! nothing of the given beneath it. Only a revision holding no policy
//! file is absent for a layout (design D11).
//!
//! Every operation begins by resolving the revision in the repository,
//! so a repository that cannot be read answers unreadable with git's
//! words before any absent (58, 59).

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use serde::Deserialize;
use tailrace_seams::{
    Answer, Audience, ChannelName, Check, ClaimKind, NamedSigner, Not, Policy, Required,
    RequiredClaim, Revision, Seam, SignerId, Source,
};
use toml::Spanned;

use crate::profiles::run::{Ran, Tool, GIT_REPOSITORY_ENV};

#[cfg(test)]
mod tests;

/// The policy file, at the root of the revision's tree.
pub const POLICY_FILE: &str = ".config/tailrace.toml";

/// The policy seam bound to the policy file of the repository at a path.
#[derive(Clone, Debug)]
pub struct PolicyFile {
    repository: PathBuf,
    git: Tool,
}

/// What this profile reads of the file: the signers, each name with
/// where it is written so they keep the file's order, and each audience's
/// table.
#[derive(Deserialize)]
struct File {
    #[serde(default)]
    signers: BTreeMap<Spanned<String>, toml::Table>,
    acceptance: Option<AudienceTable>,
    layout: Option<AudienceTable>,
    #[serde(default)]
    channels: BTreeMap<ChannelName, AudienceTable>,
}

/// An audience's table: the claims it requires, in order, and for a
/// channel the channel it is bound to (83).
#[derive(Deserialize)]
struct AudienceTable {
    require: Vec<Entry>,
    bound_to: Option<ChannelName>,
}

/// One claim an audience requires, its signers by their names in
/// `[signers]`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    kind: ClaimKind,
    check: Option<Check>,
    signers: Vec<SignerId>,
}

fn source() -> Source {
    Source::new(Seam::Policy, "policy file")
}

impl PolicyFile {
    /// The policy file of the repository at `repository`, read with
    /// `git` as found on `PATH`.
    pub fn new(repository: impl Into<PathBuf>) -> PolicyFile {
        PolicyFile {
            repository: repository.into(),
            git: Tool::new("git", source()),
        }
    }

    /// The policy seam and this profile, as every answer names them.
    pub fn source(&self) -> &Source {
        self.git.source()
    }

    /// git run in the repository with `args`, answering when it exits
    /// with one of `allowed`.
    fn git<S: AsRef<OsStr>>(&self, args: &[S], allowed: &[i32]) -> Answer<Ran> {
        let mut all = vec![OsStr::new("-C"), self.repository.as_os_str()];
        all.extend(args.iter().map(AsRef::as_ref));
        self.git.output(None, &all, GIT_REPOSITORY_ENV, allowed)
    }

    /// The commit `revision` names, resolved before anything is read at
    /// it (rule core.4); absent when the repository holds none.
    fn commit(&self, revision: &Revision) -> Answer<String> {
        let ran = self.git(
            &[
                "rev-parse",
                "--verify",
                "--quiet",
                &format!("{revision}^{{commit}}"),
            ],
            &[0, 1],
        )?;
        if ran.code != Some(0) {
            return Err(Not::absent(self.source(), format!("revision {revision}")));
        }
        Ok(String::from_utf8_lossy(&ran.stdout).trim().to_string())
    }

    /// The policy file's text in `commit`'s tree, `None` where the tree
    /// holds none.
    fn text(&self, commit: &str) -> Answer<Option<Vec<u8>>> {
        let listed = self.git(&["ls-tree", "-z", commit, "--", POLICY_FILE], &[0])?;
        let Some(blob) = String::from_utf8_lossy(&listed.stdout)
            .split('\0')
            .find_map(|entry| entry.split_whitespace().nth(2).map(str::to_string))
        else {
            return Ok(None);
        };
        Ok(Some(self.git(&["cat-file", "blob", &blob], &[0])?.stdout))
    }
}

/// The audience as an answer names it.
fn audience_name(audience: &Audience) -> String {
    match audience {
        Audience::Acceptance => "acceptance".into(),
        Audience::Layout => "layout".into(),
        Audience::Channel(channel) => format!("the channel {channel}"),
    }
}

/// `[signers.<name>]` read as the signer a policy names: its one
/// verifier, and what it may claim.
fn signer(name: &str, table: &toml::Table) -> Result<NamedSigner, String> {
    let verifiers: Vec<(&String, &toml::Value)> = table
        .iter()
        .filter(|(key, _)| !matches!(key.as_str(), "kinds" | "checks"))
        .collect();
    let (key, value) = match verifiers.as_slice() {
        [one] => *one,
        [] => {
            return Err(format!(
                "[signers.{name}] names no key that verifies it besides kinds and checks"
            ))
        }
        several => {
            let mut keys: Vec<&str> = several.iter().map(|(k, _)| k.as_str()).collect();
            keys.sort_unstable();
            return Err(format!(
                "[signers.{name}] names {}, and a signer is verified by one key",
                keys.join(", ")
            ));
        }
    };
    let verified_by = value
        .as_str()
        .ok_or_else(|| format!("[signers.{name}] {key} is not a string"))?;
    Ok(NamedSigner {
        id: SignerId::new(name),
        verified_by: verified_by.to_string(),
        kinds: list(name, table, "kinds")?,
        checks: list(name, table, "checks")?,
    })
}

/// A signer's `field`, a list, empty where the table names none.
fn list<T: serde::de::DeserializeOwned>(
    name: &str,
    table: &toml::Table,
    field: &str,
) -> Result<Vec<T>, String> {
    match table.get(field) {
        Some(value) => value
            .clone()
            .try_into()
            .map_err(|e: toml::de::Error| format!("[signers.{name}] {field}: {e}")),
        None => Ok(Vec::new()),
    }
}

/// C.10, 108: the given policy for a layout, standing in for a
/// `[layout]` the file does not write: acceptance by the gate, every
/// signer of `signers`, in the file's order, that may claim acceptance;
/// where none may and exactly one may claim provenance, that one, the
/// one host 108 admits as the gate, which may then claim acceptance too;
/// otherwise no signer, since 108 admits no gate, and no version counts
/// (design D11).
fn given_layout(signers: &[NamedSigner]) -> Vec<RequiredClaim> {
    let may = |kind: ClaimKind| signers.iter().filter(move |s| s.kinds.contains(&kind));
    let mut gate: Vec<NamedSigner> = may(ClaimKind::Acceptance).cloned().collect();
    if gate.is_empty() {
        if let [one] = may(ClaimKind::Provenance).collect::<Vec<_>>().as_slice() {
            let mut host = (*one).clone();
            host.kinds.push(ClaimKind::Acceptance);
            gate.push(host);
        }
    }
    vec![RequiredClaim {
        kind: ClaimKind::Acceptance,
        check: None,
        signers: gate,
    }]
}

impl Policy for PolicyFile {
    /// 82, 83: the claims `audience` requires in the policy file at
    /// `revision`, each signer with its verifier and its reach; for a
    /// layout with no `[layout]`, the given (C.10); absent naming the
    /// audience where the file, or the table of acceptance or a channel,
    /// is missing, and unreadable naming the file and the revision, with
    /// git's or the parser's words, where it cannot be read.
    fn policy_for(&self, revision: &Revision, audience: &Audience) -> Answer<Required> {
        let commit = self.commit(revision)?;
        let absent = || {
            Not::absent(
                self.source(),
                format!("policy for {} in {POLICY_FILE}", audience_name(audience)),
            )
        };
        let what = format!("{POLICY_FILE} at {revision}");
        let unreadable = |said: String| Not::unreadable(self.source(), &what, Some(said));
        let Some(bytes) = self.text(&commit)? else {
            return Err(absent());
        };
        let text = String::from_utf8(bytes).map_err(|e| unreadable(e.to_string()))?;
        let file: File = toml::from_str(&text).map_err(|e| unreadable(e.to_string()))?;

        let mut written = Vec::new();
        for (name, table) in &file.signers {
            let named = signer(name.as_ref(), table).map_err(&unreadable)?;
            written.push((name.span().start, named));
        }
        written.sort_by_key(|(at, _)| *at);
        let signers: Vec<NamedSigner> = written.into_iter().map(|(_, s)| s).collect();

        let (table, name) = match audience {
            Audience::Acceptance => (file.acceptance.as_ref(), "[acceptance]".to_string()),
            Audience::Layout => (file.layout.as_ref(), "[layout]".to_string()),
            Audience::Channel(channel) => {
                (file.channels.get(channel), format!("[channels.{channel}]"))
            }
        };
        let table = match (table, audience) {
            (Some(table), _) => table,
            (None, Audience::Layout) => {
                return Ok(Required::Layout {
                    claims: given_layout(&signers),
                    given: false,
                })
            }
            (None, _) => return Err(absent()),
        };
        if table.bound_to.is_some() && !matches!(audience, Audience::Channel(_)) {
            return Err(unreadable(format!(
                "{name} names bound_to, and only a channel is bound to another"
            )));
        }
        let mut claims = Vec::new();
        for entry in &table.require {
            let mut named = Vec::new();
            for id in &entry.signers {
                let held = signers.iter().find(|s| s.id == *id).ok_or_else(|| {
                    unreadable(format!(
                        "{name} requires a claim of the signer {id}, which [signers] does not name"
                    ))
                })?;
                named.push(held.clone());
            }
            claims.push(RequiredClaim {
                kind: entry.kind,
                check: entry.check.clone(),
                signers: named,
            });
        }
        Ok(match audience {
            Audience::Acceptance => Required::Acceptance { claims },
            Audience::Layout => Required::Layout {
                claims,
                given: false,
            },
            Audience::Channel(_) => Required::Channel {
                claims,
                bound_to: table.bound_to.clone(),
            },
        })
    }
}
