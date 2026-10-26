//! The policy seam (B.5) bound to the policy file (C.10):
//! `.config/tailrace.toml` read with git from the tree of the revision
//! asked, never from a checkout (65, rule core.22; design D3). It reads
//! its own tables and the `name` of each `[[component]]`, and leaves the
//! core's other keys (`outputs`, `[version]`, `[projections]`, the rest
//! of `[[component]]`) and every other reader's alone:
//!
//! ```toml
//! [[component]]
//! name = "web"
//! outputs = ["site"]
//! paths = ["site/**"]
//! prefix = "web-v"
//!
//! [[component]]
//! name = "cli"
//! outputs = ["cli"]
//! paths = ["crates/**"]
//! prefix = "cli-v"
//!
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
//! [channels.beta]
//! component = "web"
//! require = [{ kind = "provenance", signers = ["laptop"] }]
//!
//! [channels.stable]
//! component = "web"
//! bound_to = "beta"
//! require = [{ kind = "acceptance", signers = ["gate"] }]
//! ```
//!
//! A signer's table names exactly one key besides `kinds` and `checks`,
//! whose text is what its signer profile verifies it by, handed on whole
//! so each signer profile reads its own (82, 116). Each audience's
//! `require` keeps its order, which fixes the first claim found wanting
//! (58).
//!
//! A channel's table names the component it belongs to with
//! `component`, one a `[[component]]` declares, and may leave it out
//! where one is declared, or none, the one then named `repository` (123,
//! 125, 131); its `bound_to` names a channel of `[channels]` in the same
//! component (37). A file whose channel breaks either is unreadable
//! naming the table, read alike for the channels listed and for one
//! channel's policy, so the two never disagree (83). A channel's keys
//! other than `require`, `bound_to` and `component` are left to the
//! reader that owns them.
//!
//! Every operation begins by resolving the revision in the repository,
//! so a repository that cannot be read answers unreadable with git's
//! words before any absent (58, 59).

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use serde::Deserialize;
use tailrace_seams::{
    Answer, Audience, ChannelName, Check, ClaimKind, ComponentName, DeclaredChannel, NamedSigner,
    Not, Policy, Required, RequiredClaim, Revision, Seam, SignerId, Source, REPOSITORY_COMPONENT,
};

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

/// What this profile reads of the file: the signers, each audience's
/// table, and the components' names.
#[derive(Deserialize)]
struct File {
    #[serde(default)]
    signers: BTreeMap<String, toml::Table>,
    acceptance: Option<AudienceTable>,
    layout: Option<AudienceTable>,
    #[serde(default)]
    channels: BTreeMap<ChannelName, AudienceTable>,
    #[serde(default)]
    component: Vec<ComponentEntry>,
}

/// A `[[component]]` table, of which this profile reads the name alone
/// (123).
#[derive(Deserialize)]
struct ComponentEntry {
    name: ComponentName,
}

/// An audience's table: the claims it requires, in order, and for a
/// channel the channel it is bound to (83) and the component it belongs
/// to (125).
#[derive(Deserialize)]
struct AudienceTable {
    require: Vec<Entry>,
    bound_to: Option<ChannelName>,
    component: Option<ComponentName>,
}

/// The file at a revision, parsed whole, with its signers read.
struct Read {
    file: File,
    signers: BTreeMap<String, NamedSigner>,
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

    /// Unreadable naming the file and the revision, with `said`.
    fn unreadable(&self, revision: &Revision, said: String) -> Not {
        Not::unreadable(
            self.source(),
            format!("{POLICY_FILE} at {revision}"),
            Some(said),
        )
    }

    /// The policy file at `revision` parsed whole, its signers read,
    /// `None` where the revision's tree holds none; unreadable with git's
    /// or the parser's words where it cannot be read, so a file one
    /// operation cannot read no other reads either.
    fn read(&self, revision: &Revision) -> Answer<Option<Read>> {
        let commit = self.commit(revision)?;
        let unreadable = |said: String| self.unreadable(revision, said);
        let Some(bytes) = self.text(&commit)? else {
            return Ok(None);
        };
        let text = String::from_utf8(bytes).map_err(|e| unreadable(e.to_string()))?;
        let file: File = toml::from_str(&text).map_err(|e| unreadable(e.to_string()))?;
        let mut signers = BTreeMap::new();
        for (name, table) in &file.signers {
            signers.insert(name.clone(), signer(name, table).map_err(unreadable)?);
        }
        Ok(Some(Read { file, signers }))
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

/// The components the file declares: each `[[component]]`'s name, or
/// the one named `repository` where it declares none (123).
fn components(file: &File) -> Vec<ComponentName> {
    if file.component.is_empty() {
        vec![ComponentName::new(REPOSITORY_COMPONENT)]
    } else {
        file.component.iter().map(|c| c.name.clone()).collect()
    }
}

/// The component `[channels.<name>]` belongs to: the one it names, which
/// the file must declare, or the one declared where it names none and
/// one is (125), in the gate's words where the gate has words for it.
fn component_of(
    file: &File,
    name: &ChannelName,
    table: &AudienceTable,
) -> Result<ComponentName, String> {
    let declared = components(file);
    let listed = || {
        declared
            .iter()
            .map(ComponentName::as_str)
            .collect::<Vec<_>>()
            .join(", ")
    };
    match (&table.component, declared.as_slice()) {
        (Some(named), _) if declared.contains(named) => Ok(named.clone()),
        (Some(named), _) => Err(format!(
            "[channels.{name}] names the component {named}, which {POLICY_FILE} does not declare; it declares {}",
            listed()
        )),
        (None, [one]) => Ok(one.clone()),
        (None, _) => Err(format!(
            "[channels.{name}] names no component, and several are declared: {}",
            listed()
        )),
    }
}

/// 125, 131: `[channels.<name>]` as the channel the policy declares, its
/// component, and the channel of that component it is bound to (37), or
/// the words that make the file unreadable; one reading for the channels
/// listed and for one channel's policy (83).
fn resolve(file: &File, name: &ChannelName) -> Result<DeclaredChannel, String> {
    let table = &file.channels[name];
    let component = component_of(file, name, table)?;
    if let Some(bound) = &table.bound_to {
        let bound_table = file.channels.get(bound).ok_or_else(|| {
            format!("[channels.{name}] is bound to {bound}, which [channels] does not declare")
        })?;
        let bound_component = component_of(file, bound, bound_table)?;
        if bound_component != component {
            return Err(format!(
                "[channels.{name}] is bound to {bound}, a channel of {bound_component}, and a channel is bound only within its component, {component} (125)"
            ));
        }
    }
    Ok(DeclaredChannel {
        name: name.clone(),
        component,
        bound_to: table.bound_to.clone(),
    })
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

impl Policy for PolicyFile {
    /// 82, 83: the claims `audience` requires in the policy file at
    /// `revision`, each signer with its verifier and its reach; absent
    /// naming the audience where the file or its table is missing, and
    /// unreadable naming the file and the revision, with git's or the
    /// parser's words, where it cannot be read.
    fn policy_for(&self, revision: &Revision, audience: &Audience) -> Answer<Required> {
        let absent = || {
            Not::absent(
                self.source(),
                format!("policy for {} in {POLICY_FILE}", audience_name(audience)),
            )
        };
        let unreadable = |said: String| self.unreadable(revision, said);
        let Some(Read { file, signers }) = self.read(revision)? else {
            return Err(absent());
        };

        let (table, name) = match audience {
            Audience::Acceptance => (file.acceptance.as_ref(), "[acceptance]".to_string()),
            Audience::Layout => (file.layout.as_ref(), "[layout]".to_string()),
            Audience::Channel(channel) => {
                (file.channels.get(channel), format!("[channels.{channel}]"))
            }
        };
        let table = table.ok_or_else(absent)?;
        if table.bound_to.is_some() && !matches!(audience, Audience::Channel(_)) {
            return Err(unreadable(format!(
                "{name} names bound_to, and only a channel is bound to another"
            )));
        }
        let bound_to = match audience {
            Audience::Channel(channel) => resolve(&file, channel).map_err(unreadable)?.bound_to,
            Audience::Acceptance | Audience::Layout => None,
        };
        let mut claims = Vec::new();
        for entry in &table.require {
            let mut named = Vec::new();
            for id in &entry.signers {
                let held = signers.get(id.as_str()).ok_or_else(|| {
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
            Audience::Layout => Required::Layout { claims },
            Audience::Channel(_) => Required::Channel { claims, bound_to },
        })
    }

    /// 131: every `[channels.<name>]` of the policy file at `revision`, by
    /// name, each with its component and the channel it is bound to; an
    /// empty list where the file or its `[channels]` is missing, and
    /// unreadable naming the file and the revision, with the words of
    /// the first channel that breaks 125 or 37, where one does.
    fn channels(&self, revision: &Revision) -> Answer<Vec<DeclaredChannel>> {
        let Some(Read { file, .. }) = self.read(revision)? else {
            return Ok(Vec::new());
        };
        file.channels
            .keys()
            .map(|name| resolve(&file, name).map_err(|said| self.unreadable(revision, said)))
            .collect()
    }
}
