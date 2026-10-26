//! The policy seam (B.5) bound to the policy file (C.10):
//! `.config/tailrace.toml` read with git from the tree of the revision
//! asked, never from a checkout (65, rule core.22; design D3, D4). It
//! reads its own tables, the `name` of each `[[component]]`, and of each
//! environment its `channel`, `policy`, `after` and `approvers`, and
//! leaves the core's other keys (`outputs`, `[version]`, `[projections]`,
//! the rest of `[[component]]`) and every other reader's alone:
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
//! [signers.gate]
//! ssh = "ssh-ed25519 GGGG… gate@box"
//! kinds = ["acceptance", "promotion"]
//!
//! [signers.laptop]
//! ssh = "ssh-ed25519 AAAA… host@laptop"
//! kinds = ["provenance"]
//!
//! [signers.box]
//! ssh = "ssh-ed25519 BBBB…"
//! kinds = ["verdict", "rebuild"]
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
//! # cli switches alpha on: its keys are written in the givens beneath.
//! [channels.cli.alpha]
//!
//! # A line under web's given stable: a rebuild after the gate's
//! # acceptance, stable still bound to beta, and only box may ask for it
//! # to move.
//! [channels.web.stable]
//! require = [{ kind = "rebuild", signers = ["box"] }]
//! approvers = ["box"]
//!
//! # prod waits on a yes: its own channel, prod, bound to stable, which
//! # only box may ask to move.
//! [environments.web.prod]
//! channel = "stable"
//! policy = "manual"
//! approvers = ["box"]
//! ```
//!
//! A signer's table names exactly one key besides `kinds` and `checks`,
//! whose text is what its signer profile verifies it by, handed on whole
//! so each signer profile reads its own (82, 116). Each audience's
//! `require` keeps its order, which fixes the first claim found wanting
//! (58).
//!
//! The given channels are a fragment in this same form, `givens.toml`
//! beside this module, laid beneath the repository's file as one table in
//! force (C.10): `[channels.<name>]` of the fragment lies beneath the
//! repository's `[channels.<name>]`, or each component's
//! `[channels.<component>.<name>]`, a key the repository writes replacing
//! the fragment's, `on_each_publish` and `bound_to` together and with
//! them the fragment's `prereleases` unless the repository writes its
//! own. `held = true` is the fragment's mark alone: a channel it marks is
//! in force only where the repository writes its table, so an empty
//! `[channels.alpha]` switches it on, and the same line rebinds a given or
//! adds claims to it. Writing the table is the one switch: `held` or `on`
//! written in the repository's file is unreadable, and nothing turns a
//! given the fragment does not hold off. No source of this profile names
//! a channel.
//!
//! The channels are resolved from the table in force in one pass, shared
//! by the channels listed and one channel's policy so the two never
//! disagree (83, 131): each table moves on each accepted publish with
//! `on_each_publish = true`, taking prereleases too where it names
//! `prereleases = true`, bound with `bound_to`, and by a promotion alone
//! otherwise (37, 95, C.10); an environment whose `policy` is `manual`,
//! or that names `after`, is given a channel named for it, bound to the
//! channel it names or to the one the environments it comes after follow
//! (41, 97, C.10), and a table under it is a line that adds claims and
//! never changes how it moves; every binding is within its component and
//! none is a cycle (37, 125). Environments are `[environments.<name>]` or
//! `[environments.<component>.<name>]` alike. A file that breaks any of
//! it is unreadable naming the table.
//!
//! The policy for a given or made channel requires the gate's
//! acceptance, by every signer `[signers]` lets claim acceptance, then
//! its line's claims, and names those signers, the gate, as the ones
//! whose promotion of it counts, whatever the line names (32, C.10); a
//! channel declared by hand is promoted by the signers its `require`
//! names that may claim a promotion. A channel's table may name
//! `approvers`, signers `[signers]` holds, the only ones that may then ask
//! for it to move; where none is named any asker the gate verifies may
//! (32, 129, C.10). They never join who signs. A repository writes them on
//! a manual environment for the channel it makes, which takes its
//! environment's, and never under that channel (C.10).
//!
//! Every operation begins by reading the fragment and resolving the
//! revision in the repository, so a fragment or a repository that cannot
//! be read answers unreadable with the parser's or git's words before any
//! absent (58, 59).

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::path::PathBuf;

use serde::Deserialize;
use tailrace_seams::{
    Answer, Audience, ChannelId, ChannelName, Check, ClaimKind, ComponentName, DeclaredChannel,
    Moves, NamedSigner, Not, Policy, PolicyChannels, Required, RequiredClaim, Revision, Seam,
    SignerId, Source,
};

use crate::profiles::run::{Ran, Tool, GIT_REPOSITORY_ENV};

#[cfg(test)]
mod tests;

/// The policy file, at the root of the revision's tree.
pub const POLICY_FILE: &str = ".config/tailrace.toml";

/// The given channels shipped with tailrace, a fragment in the policy
/// file's own form (C.10).
const GIVENS: &str = include_str!("givens.toml");

/// The fragment, as an answer that cannot read it names it.
const GIVENS_NAME: &str = "the givens' fragment givens.toml";

/// The keys a table of the givens' fragment may name (37, 83, 95, C.10).
const GIVEN_KEYS: [&str; 6] = [
    "require",
    "bound_to",
    "on_each_publish",
    "prereleases",
    "approvers",
    "held",
];

/// The keys that mark a table as a channel's own: what this profile
/// reads of one, and `on`, read only to refuse it (C.10).
const CHANNEL_KEYS: [&str; 7] = [
    "require",
    "bound_to",
    "on_each_publish",
    "prereleases",
    "approvers",
    "held",
    "on",
];

/// The policy seam bound to the policy file of the repository at a path,
/// with the givens' fragment laid beneath it.
#[derive(Clone, Debug)]
pub struct PolicyFile {
    repository: PathBuf,
    git: Tool,
    givens: Result<Givens, String>,
}

/// C.10: the fragment's channels' tables, by name, each as written there.
#[derive(Clone, Debug)]
struct Givens {
    channels: toml::Table,
}

/// What this profile reads of the file: the signers, each audience's
/// table, the channels' and the environments' tables as written, read
/// once the components are known, and the components' names.
#[derive(Deserialize)]
struct File {
    #[serde(default)]
    signers: BTreeMap<String, toml::Table>,
    acceptance: Option<AudienceTable>,
    layout: Option<AudienceTable>,
    #[serde(default)]
    channels: toml::Table,
    #[serde(default)]
    environments: toml::Table,
    #[serde(default)]
    component: Vec<ComponentEntry>,
}

/// A `[[component]]` table, of which this profile reads the name alone
/// (123).
#[derive(Deserialize)]
struct ComponentEntry {
    name: ComponentName,
}

/// `[acceptance]` or `[layout]`: the claims it requires, in order, and a
/// `bound_to` read only to refuse it (83).
#[derive(Deserialize)]
struct AudienceTable {
    require: Vec<Entry>,
    bound_to: Option<ChannelName>,
}

/// A channel's table in force: the claims it requires, in order, none
/// where it names none, how it moves (37, 95), whether it takes
/// prereleases and who may ask for it (C.10); `held` and `on` are read
/// only to refuse them. Its other keys are its readers'.
#[derive(Clone, Deserialize)]
struct ChannelTable {
    #[serde(default)]
    require: Vec<Entry>,
    bound_to: Option<ChannelName>,
    on_each_publish: Option<bool>,
    prereleases: Option<bool>,
    approvers: Option<Vec<SignerId>>,
    held: Option<bool>,
    on: Option<bool>,
}

/// An environment's table, of which this profile reads the channel it
/// names, its policy, the environments it comes after and, where it is
/// manual, who may say yes to it (C.10); its other keys are its
/// target's.
#[derive(Deserialize)]
struct EnvironmentTable {
    channel: Option<ChannelName>,
    policy: Option<String>,
    #[serde(default)]
    after: Vec<String>,
    approvers: Option<Vec<SignerId>>,
}

/// The file at a revision, parsed whole, with its signers read.
struct Read {
    file: File,
    signers: BTreeMap<String, NamedSigner>,
}

/// One claim an audience requires, its signers by their names in
/// `[signers]`.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    kind: ClaimKind,
    check: Option<Check>,
    signers: Vec<SignerId>,
}

/// D4, C.10: the table in force, the repository's file as written with
/// the fragment's channels laid beneath its own, which [`resolve`] alone
/// reads: the channels' tables, the name of every channel the fragment
/// holds and of those it holds off, and each channel held off that the
/// repository has not written, with its table header and the fragment's
/// table.
struct InForce<'a> {
    file: &'a File,
    channels: toml::Table,
    givens: BTreeSet<String>,
    held: BTreeSet<String>,
    off: Vec<(ChannelId, String, toml::Table)>,
}

/// Where a channel comes from: held by the givens, made by an
/// environment's words, or declared by hand (C.10).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Origin {
    Given,
    Made,
    Declared,
}

/// A channel as the table in force resolves it: how it moves, where it
/// comes from, the table that says so, the claims written for it, by its
/// own table or a line under it, and who may ask for it, none where any
/// asker may.
struct Resolved {
    moves: Moves,
    origin: Origin,
    table: String,
    require: Vec<Entry>,
    approvers: Vec<SignerId>,
}

fn source() -> Source {
    Source::new(Seam::Policy, "policy file")
}

impl PolicyFile {
    /// The policy file of the repository at `repository`, read with
    /// `git` as found on `PATH`.
    pub fn new(repository: impl Into<PathBuf>) -> PolicyFile {
        PolicyFile::with_givens(repository, GIVENS)
    }

    /// The policy file of the repository at `repository` with the
    /// fragment `givens` laid beneath it in place of the one shipped, so
    /// nothing but a fragment names a given channel (C.10).
    pub fn with_givens(repository: impl Into<PathBuf>, givens: &str) -> PolicyFile {
        PolicyFile {
            repository: repository.into(),
            git: Tool::new("git", source()),
            givens: Givens::read(givens),
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

    /// The fragment, or unreadable naming it with the words it could not
    /// be read with, so a fragment one operation cannot read no other
    /// reads either.
    fn givens(&self) -> Answer<&Givens> {
        self.givens
            .as_ref()
            .map_err(|said| Not::unreadable(self.source(), GIVENS_NAME, Some(said.clone())))
    }

    /// The policy file at `revision` parsed whole, its signers read,
    /// `None` where the revision's tree holds none; unreadable with git's
    /// or the parser's words where it or the fragment cannot be read, so
    /// a file one operation cannot read no other reads either.
    fn read(&self, revision: &Revision) -> Answer<Option<Read>> {
        self.givens()?;
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

/// The file's declared components, joined for an answer's words.
fn listed(components: &[ComponentName]) -> String {
    components
        .iter()
        .map(ComponentName::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}

/// The name a table of `section` takes for `name` in `component`:
/// `[<section>.<name>]` where no component is declared, and
/// `[<section>.<component>.<name>]` where one is.
fn table_name(section: &str, component: &Option<ComponentName>, name: &str) -> String {
    match component {
        None => format!("[{section}.{name}]"),
        Some(component) => format!("[{section}.{component}.{name}]"),
    }
}

/// The words that a channel `name` is no channel in `component`, for the
/// table `table` naming it as `what`: held off by the givens where `off`.
fn unknown(
    table: &str,
    what: &str,
    name: &ChannelName,
    component: &Option<ComponentName>,
    off: bool,
) -> String {
    match (component, off) {
        (None, true) => format!(
            "{table} {what} {name}, which the givens hold off until [channels.{name}] is written, and a channel held off is no channel (C.10)"
        ),
        (Some(component), true) => format!(
            "{table} {what} {name}, which the givens hold off in {component} until [channels.{component}.{name}] is written, and a channel held off is no channel (C.10)"
        ),
        (None, false) => {
            format!("{table} {what} {name}, which the file neither declares, gives nor makes")
        }
        (Some(component), false) => format!(
            "{table} {what} {name}, which is no channel of {component}, and a channel is bound only within its component (125)"
        ),
    }
}

/// The article `noun` takes.
fn article(noun: &str) -> &'static str {
    if noun.starts_with(['a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    }
}

/// 125, C.10: the tables of `section` the file writes, each under the
/// component it is of: `[<section>.<name>]` where no component is
/// declared, `[<section>.<component>.<name>]` where some are, the
/// component one they declare. A table of the one form where the other
/// is the file's, or under a component not declared, is unreadable naming
/// it and the form to use; `keys` are the keys that mark a table as one
/// of `section`'s own, `noun` what one is called.
fn tables<T: serde::de::DeserializeOwned>(
    raw: &toml::Table,
    section: &str,
    noun: &str,
    keys: &[&str],
    components: &[ComponentName],
) -> Result<Vec<(Option<ComponentName>, String, String, T)>, String> {
    let read = |name: String, value: &toml::Value| -> Result<T, String> {
        let table = value
            .as_table()
            .ok_or_else(|| format!("{name} is not a table"))?;
        toml::Value::Table(table.clone())
            .try_into()
            .map_err(|e: toml::de::Error| format!("{name}: {}", e.message()))
    };
    let mut found = Vec::new();
    if components.is_empty() {
        for (key, value) in raw {
            if let Some(table) = value.as_table() {
                let nested = !table.is_empty()
                    && !keys.iter().any(|k| table.contains_key(*k))
                    && table.values().all(toml::Value::is_table);
                if let (true, Some(first)) = (nested, table.keys().next()) {
                    return Err(format!(
                        "[{section}.{key}.{first}] names the component {key}, and {POLICY_FILE} declares no component: {} {noun} is declared as [{section}.<name>]",
                        article(noun)
                    ));
                }
            }
            let name = table_name(section, &None, key);
            found.push((None, key.clone(), name.clone(), read(name, value)?));
        }
        return Ok(found);
    }
    for (key, value) in raw {
        let named = ComponentName::new(key);
        let table = value
            .as_table()
            .ok_or_else(|| format!("[{section}.{key}] is not a table"))?;
        if components.contains(&named) {
            for (name, value) in table {
                let full = table_name(section, &Some(named.clone()), name);
                if !value.is_table() {
                    return Err(format!(
                        "[{section}.{key}] names {name}, which is no {noun}'s table: {} {noun} is declared as [{section}.<component>.<name>] where components are declared",
                        article(noun)
                    ));
                }
                found.push((
                    Some(named.clone()),
                    name.clone(),
                    full.clone(),
                    read(full, value)?,
                ));
            }
            continue;
        }
        if let Some(old) = table.get("component").and_then(toml::Value::as_str) {
            return Err(format!(
                "[{section}.{key}] names its component with component = \"{old}\", and {} {noun} of a component is declared as [{section}.{old}.{key}]",
                article(noun)
            ));
        }
        if table.is_empty() || keys.iter().any(|k| table.contains_key(*k)) {
            return Err(format!(
                "[{section}.{key}] names no component, and {POLICY_FILE} declares {}: {} {noun} is declared as [{section}.<component>.<name>]",
                listed(components),
                article(noun)
            ));
        }
        let first = table.keys().next().map(String::as_str).unwrap_or_default();
        return Err(format!(
            "[{section}.{key}.{first}] names the component {key}, which {POLICY_FILE} does not declare; it declares {}",
            listed(components)
        ));
    }
    Ok(found)
}

impl Givens {
    /// C.10: `text` read as the givens' fragment: only channels' tables,
    /// `[channels.<name>]`, each naming only the keys a given's table
    /// names, `held` among them; otherwise the parser's words or the
    /// table that breaks it.
    fn read(text: &str) -> Result<Givens, String> {
        let mut top: toml::Table = toml::from_str(text).map_err(|e| e.to_string())?;
        if let Some(key) = top.keys().find(|key| *key != "channels") {
            return Err(format!(
                "the givens hold {key}, and they hold only channels' tables, [channels.<name>]"
            ));
        }
        let channels = match top.remove("channels") {
            None => toml::Table::new(),
            Some(toml::Value::Table(channels)) => channels,
            Some(_) => return Err("the givens' channels are not a table".into()),
        };
        for (name, value) in &channels {
            let table = value
                .as_table()
                .ok_or_else(|| format!("the givens' [channels.{name}] is not a table"))?;
            if let Some(key) = table.keys().find(|k| !GIVEN_KEYS.contains(&k.as_str())) {
                return Err(format!(
                    "the givens' [channels.{name}] names {key}, and a given's table names only {}",
                    GIVEN_KEYS.join(", ")
                ));
            }
            toml::Value::Table(table.clone())
                .try_into::<ChannelTable>()
                .map_err(|e| format!("the givens' [channels.{name}]: {}", e.message()))?;
        }
        Ok(Givens { channels })
    }

    /// Whether the fragment marks `name` held, so it is in force only
    /// where the repository writes its table (C.10).
    fn holds_off(&self, name: &str) -> bool {
        self.channels
            .get(name)
            .and_then(|t| t.get("held"))
            .and_then(toml::Value::as_bool)
            == Some(true)
    }
}

/// 123: the components `file` declares, each by its name.
fn components(file: &File) -> Vec<ComponentName> {
    file.component.iter().map(|c| c.name.clone()).collect()
}

/// The components a channel may be of: each declared, or the one of the
/// repository where none is (123).
fn scopes(components: &[ComponentName]) -> Vec<Option<ComponentName>> {
    if components.is_empty() {
        vec![None]
    } else {
        components.iter().cloned().map(Some).collect()
    }
}

/// C.10: the fragment's table `given` laid beneath the table `written`
/// the repository writes for the same channel and component: the
/// fragment's keys where the repository writes none, `on_each_publish`
/// and `bound_to` replaced together where it writes either, and with them
/// the fragment's `prereleases` unless it writes its own (37); the
/// fragment's `held` mark never in force.
fn laid(given: &toml::Table, written: Option<&toml::Table>) -> toml::Table {
    let mut table = given.clone();
    table.remove("held");
    let Some(written) = written else {
        return table;
    };
    if written.contains_key("on_each_publish") || written.contains_key("bound_to") {
        table.remove("on_each_publish");
        table.remove("bound_to");
        table.remove("prereleases");
    }
    table.extend(written.clone());
    table
}

/// D4, C.10: the table in force: `file` as written, with the fragment's
/// table of each channel `givens` holds laid beneath the repository's
/// table of that channel, `[channels.<name>]` where no component is
/// declared and `[channels.<component>.<name>]` in every component where
/// some are; a channel the fragment holds off laid only where the
/// repository writes its table, and kept aside as held otherwise. A table
/// not of a channel's form is left as written, for the resolution to
/// name.
fn in_force<'a>(file: &'a File, givens: &Givens) -> InForce<'a> {
    let mut channels = file.channels.clone();
    let mut off = Vec::new();
    for scope in scopes(&components(file)) {
        let under = match &scope {
            None => Some(&mut channels),
            Some(component) => channels
                .entry(component.as_str())
                .or_insert_with(|| toml::Value::Table(toml::Table::new()))
                .as_table_mut(),
        };
        let Some(under) = under else {
            continue;
        };
        for (name, given) in &givens.channels {
            let Some(given) = given.as_table() else {
                continue;
            };
            let written = match under.get(name) {
                None => None,
                Some(toml::Value::Table(written)) => Some(written),
                Some(_) => continue,
            };
            let table = laid(given, written);
            if written.is_none() && givens.holds_off(name) {
                let id = ChannelId::new(scope.clone(), ChannelName::new(name));
                off.push((id, table_name("channels", &scope, name), table));
                continue;
            }
            under.insert(name.clone(), toml::Value::Table(table));
        }
    }
    InForce {
        file,
        channels,
        givens: givens.channels.keys().cloned().collect(),
        held: givens
            .channels
            .keys()
            .filter(|n| givens.holds_off(n))
            .cloned()
            .collect(),
        off,
    }
}

/// 37, C.10: how the table `full` in force says its channel moves: on
/// each accepted publish where it names `on_each_publish = true`, taking
/// prereleases too where it names `prereleases = true`; bound where it
/// names `bound_to`; and by a promotion alone otherwise. Both, or
/// `prereleases` on a channel that does not move on each publish, are the
/// words that make the file unreadable.
fn moves_of(full: &str, keys: &ChannelTable) -> Result<Moves, String> {
    let moves = match (keys.on_each_publish, &keys.bound_to) {
        (Some(true), Some(bound)) => {
            return Err(format!(
                "{full} follows each publish and is bound to {bound}, and a bound channel takes no version from a publish (37)"
            ))
        }
        (Some(true), None) => Moves::EachPublish {
            prereleases: keys.prereleases == Some(true),
        },
        (_, Some(bound)) => Moves::Bound { to: bound.clone() },
        (_, None) => Moves::Promotion,
    };
    if keys.prereleases.is_some() && !matches!(moves, Moves::EachPublish { .. }) {
        return Err(format!(
            "{full} names prereleases, and only a channel that moves on each publish takes prereleases: this one takes no version from a publish (37, C.10)"
        ));
    }
    Ok(moves)
}

/// 32, C.10: the approvers the table `full` names, each one `[signers]`
/// holds and the list never empty, since an empty one would read as
/// naming no one; none where it names none.
fn approvers_of(
    full: &str,
    approvers: &Option<Vec<SignerId>>,
    signers: &BTreeMap<String, toml::Table>,
) -> Result<Vec<SignerId>, String> {
    match approvers {
        None => Ok(Vec::new()),
        Some(approvers) if approvers.is_empty() => Err(format!(
            "{full} names no approver, and an empty approvers would read as naming no one; leave it out for any asker the gate verifies (C.10)"
        )),
        Some(approvers) => match approvers
            .iter()
            .find(|a| !signers.contains_key(a.as_str()))
        {
            Some(unknown) => Err(format!(
                "{full} names the approver {unknown}, which [signers] does not name"
            )),
            None => Ok(approvers.clone()),
        },
    }
}

/// 125, 131, C.10: every channel of the table in force, by its identity,
/// in one pass shared by the channels listed and one channel's policy
/// (83), in D4's six steps: the components; the channels' tables; how
/// each table moves and who may ask for it; the environments and the
/// channels their words make; a table under a made channel read as a
/// line under it; and every binding, and every environment's channel,
/// within its component and in no cycle. A channel held off is no
/// channel. The first that breaks is the words that make the file
/// unreadable, naming its table.
fn resolve(table: &InForce) -> Result<BTreeMap<ChannelId, Resolved>, String> {
    // 1: the components, or none.
    let components = components(table.file);
    let scopes = scopes(&components);

    // 2: the channels' tables.
    let written: Vec<(Option<ComponentName>, String, String, ChannelTable)> = tables(
        &table.channels,
        "channels",
        "channel",
        &CHANNEL_KEYS,
        &components,
    )?;

    // 3: each table in force moves as it says (37, 95), names no key
    // that would switch a channel on or off, and names as approvers only
    // signers the file holds (C.10).
    let stays: Vec<&str> = table
        .givens
        .iter()
        .filter(|n| !table.held.contains(*n))
        .map(String::as_str)
        .collect();
    let mut tables_of: Vec<(ChannelId, String, ChannelTable, Moves)> = Vec::new();
    for (component, name, full, keys) in written {
        let key = match (keys.held, keys.on) {
            (Some(_), _) => Some("held"),
            (None, Some(_)) => Some("on"),
            (None, None) => None,
        };
        if let Some(key) = key {
            return Err(format!(
                "{full} names {key}, and writing a channel's table is the one switch that turns it on: no key turns a channel on or off, and nothing turns {} off (C.10)",
                stays.join(" or ")
            ));
        }
        let moves = moves_of(&full, &keys)?;
        approvers_of(&full, &keys.approvers, &table.file.signers)?;
        tables_of.push((
            ChannelId::new(component, ChannelName::new(&name)),
            full,
            keys,
            moves,
        ));
    }

    // 4: the environments, and the channels their words make (41, 97).
    let environments: Vec<(Option<ComponentName>, String, String, EnvironmentTable)> = tables(
        &table.file.environments,
        "environments",
        "environment",
        &["channel", "policy", "after"],
        &components,
    )?;
    let mut follows: Vec<(String, Option<ComponentName>, ChannelName)> = Vec::new();
    let mut made: BTreeMap<ChannelId, (String, ChannelName, Vec<SignerId>)> = BTreeMap::new();
    for scope in &scopes {
        let of: BTreeMap<&str, (&String, &EnvironmentTable)> = environments
            .iter()
            .filter(|(c, ..)| c == scope)
            .map(|(_, name, table, env)| (name.as_str(), (table, env)))
            .collect();
        for (name, (env_table, env)) in &of {
            let channel = env
                .channel
                .clone()
                .ok_or_else(|| format!("{env_table} names no channel"))?;
            let manual = match env.policy.as_deref() {
                None | Some("auto") => false,
                Some("manual") => true,
                Some(other) => {
                    return Err(format!(
                        "{env_table} names the policy {other}, and an environment's policy is auto or manual"
                    ))
                }
            };
            follows.push(((*env_table).clone(), scope.clone(), channel.clone()));
            let approvers = if manual {
                approvers_of(env_table, &env.approvers, &table.file.signers)?
            } else {
                Vec::new()
            };
            let bound = if !env.after.is_empty() {
                after(&of, name, env_table, env, scope)?
            } else if manual {
                channel
            } else {
                continue;
            };
            if table.givens.contains(*name) {
                return Err(format!(
                    "{env_table} makes a channel named {name}, which the givens hold, held off or not (C.10)"
                ));
            }
            made.insert(
                ChannelId::new(scope.clone(), ChannelName::new(*name)),
                ((*env_table).clone(), bound, approvers),
            );
        }
    }

    // 5: a table under a made channel is a line under it, which never
    // changes how it moves and names no approvers, its environment's
    // words saying both, so it takes its environment's approvers; every
    // other is the channel it says.
    let mut resolved: BTreeMap<ChannelId, Resolved> = BTreeMap::new();
    let off: BTreeSet<ChannelId> = table.off.iter().map(|(id, ..)| id.clone()).collect();
    let mut lines: BTreeMap<ChannelId, (String, Vec<Entry>)> = BTreeMap::new();
    for (id, full, keys, moves) in tables_of {
        if let Some((env_table, to, _)) = made.get(&id) {
            let restates = keys.bound_to.as_ref().is_none_or(|b| b == to)
                && keys.on_each_publish != Some(true);
            if !restates {
                return Err(format!(
                    "{full} would change how {} moves, and {} is bound to {to} by its environment's words: a line under a channel an environment makes never changes how it moves (C.10)",
                    id.name, id.name
                ));
            }
            if keys.approvers.is_some() {
                return Err(format!(
                    "{full} names approvers, and who may ask for {}, a channel its environment makes, is written on {env_table} and never on the channel it makes (C.10)",
                    id.name
                ));
            }
            lines.insert(id, (full, keys.require));
            continue;
        }
        let origin = if table.givens.contains(id.name.as_str()) {
            Origin::Given
        } else {
            Origin::Declared
        };
        resolved.insert(
            id,
            Resolved {
                moves,
                origin,
                table: full,
                require: keys.require,
                approvers: keys.approvers.unwrap_or_default(),
            },
        );
    }
    for (id, (env_table, to, approvers)) in made {
        let (table, require) = lines.remove(&id).unwrap_or((env_table, Vec::new()));
        resolved.insert(
            id,
            Resolved {
                moves: Moves::Bound { to },
                origin: Origin::Made,
                table,
                require,
                approvers,
            },
        );
    }

    // 6: every binding, and every environment's channel, names a channel
    // of its component in force (37, 125), and no binding is a cycle.
    let known = |id: &ChannelId| resolved.contains_key(id);
    for (env_table, scope, channel) in &follows {
        let id = ChannelId::new(scope.clone(), channel.clone());
        if !known(&id) {
            return Err(unknown(
                env_table,
                "follows",
                channel,
                scope,
                off.contains(&id),
            ));
        }
    }
    for (id, entry) in &resolved {
        let Moves::Bound { to } = &entry.moves else {
            continue;
        };
        let target = ChannelId::new(id.component.clone(), to.clone());
        if !known(&target) {
            return Err(unknown(
                &entry.table,
                "is bound to",
                to,
                &id.component,
                off.contains(&target),
            ));
        }
    }
    for (id, entry) in &resolved {
        let mut path = vec![id.name.to_string()];
        let mut at = id.clone();
        while let Some(Resolved {
            moves: Moves::Bound { to },
            ..
        }) = resolved.get(&at)
        {
            path.push(to.to_string());
            at = ChannelId::new(id.component.clone(), to.clone());
            if at == *id {
                return Err(format!(
                    "{} is bound in a cycle, {}, and no version could ever reach it (37)",
                    entry.table,
                    path.join(" to ")
                ));
            }
            if path.len() > resolved.len() {
                break;
            }
        }
    }
    Ok(resolved)
}

/// 37, C.10: the channel an environment `name`, of the environments
/// `of` of its component, is bound to by its `after`: the one every
/// environment it names follows, their own where they have one and the
/// one they name otherwise. A name no environment of the component
/// bears, a cycle back to `name`, or environments that follow two
/// channels is unreadable naming `table`.
fn after(
    of: &BTreeMap<&str, (&String, &EnvironmentTable)>,
    name: &str,
    table: &str,
    env: &EnvironmentTable,
    scope: &Option<ComponentName>,
) -> Result<ChannelName, String> {
    let named = |other: &str| match scope {
        None => format!("{table} is after {other}, which [environments] does not declare"),
        Some(c) => format!("{table} is after {other}, which is no environment of {c}"),
    };
    // A cycle through `after` back to `name` (C.10).
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut walk: Vec<&str> = env.after.iter().map(String::as_str).collect();
    while let Some(next) = walk.pop() {
        if next == name {
            return Err(format!("{table} comes after itself through after"));
        }
        // A name no environment bears is its own table's to answer.
        let (true, Some((_, other))) = (seen.insert(next), of.get(next)) else {
            continue;
        };
        walk.extend(other.after.iter().map(String::as_str));
    }
    let mut channels: Vec<(String, ChannelName)> = Vec::new();
    for other in &env.after {
        let (_, them) = of.get(other.as_str()).ok_or_else(|| named(other))?;
        let own = !them.after.is_empty() || them.policy.as_deref() == Some("manual");
        let followed = match (&them.channel, own) {
            (_, true) => ChannelName::new(other),
            (Some(channel), false) => channel.clone(),
            (None, false) => return Err(format!("{} names no channel", of[other.as_str()].0)),
        };
        channels.push((other.clone(), followed));
    }
    let first = channels[0].1.clone();
    if channels.iter().any(|(_, c)| *c != first) {
        let (names, followed): (Vec<String>, Vec<String>) = channels
            .iter()
            .map(|(n, c)| (n.clone(), c.to_string()))
            .unzip();
        return Err(format!(
            "{table} is after {}, which follow {}, and the environments one comes after follow one channel",
            names.join(" and "),
            followed.join(" and ")
        ));
    }
    Ok(first)
}

/// The claims `entries` require, each signer as `[signers]` names it,
/// for `table`; the words that make the file unreadable where one names
/// a signer it does not hold.
fn claims_of(
    table: &str,
    entries: &[Entry],
    signers: &BTreeMap<String, NamedSigner>,
) -> Result<Vec<RequiredClaim>, String> {
    let mut claims = Vec::new();
    for entry in entries {
        let mut named = Vec::new();
        for id in &entry.signers {
            let held = signers.get(id.as_str()).ok_or_else(|| {
                format!(
                    "{table} requires a claim of the signer {id}, which [signers] does not name"
                )
            })?;
            named.push(held.clone());
        }
        claims.push(RequiredClaim {
            kind: entry.kind,
            check: entry.check.clone(),
            signers: named,
        });
    }
    Ok(claims)
}

/// 32: the signers that may claim `kind` among `signers`, once each, in
/// the order first named.
fn may_claim<'a>(
    signers: impl Iterator<Item = &'a NamedSigner>,
    kind: ClaimKind,
) -> Vec<NamedSigner> {
    let mut found: Vec<NamedSigner> = Vec::new();
    for signer in signers {
        if signer.kinds.contains(&kind) && !found.iter().any(|s| s.id == signer.id) {
            found.push(signer.clone());
        }
    }
    found
}

/// 32, 82, 83, C.10: the policy for the channel `resolved`: for one
/// given or made, the gate's acceptance by every signer that may claim
/// acceptance, then its line's claims, promoted by those same signers;
/// for one declared by hand, its table's claims, promoted by the signers
/// they name that may claim a promotion; and for any, the approvers its
/// table names, or for one made its environment's, each as `[signers]`
/// names it.
fn channel_policy(
    resolved: &Resolved,
    signers: &BTreeMap<String, NamedSigner>,
) -> Result<Required, String> {
    let claims = claims_of(&resolved.table, &resolved.require, signers)?;
    let (claims, promoted_by) = match resolved.origin {
        Origin::Given | Origin::Made => {
            let gate = may_claim(signers.values(), ClaimKind::Acceptance);
            let mut all = vec![RequiredClaim {
                kind: ClaimKind::Acceptance,
                check: None,
                signers: gate.clone(),
            }];
            all.extend(claims);
            (all, gate)
        }
        Origin::Declared => {
            let promoters = may_claim(claims.iter().flat_map(|c| &c.signers), ClaimKind::Promotion);
            (claims, promoters)
        }
    };
    let mut approvers = Vec::new();
    for id in &resolved.approvers {
        let named = signers.get(id.as_str()).ok_or_else(|| {
            format!(
                "{} names the approver {id}, which [signers] does not name",
                resolved.table
            )
        })?;
        approvers.push(named.clone());
    }
    Ok(Required::Channel {
        claims,
        bound_to: match &resolved.moves {
            Moves::Bound { to } => Some(to.clone()),
            Moves::EachPublish { .. } | Moves::Promotion => None,
        },
        promoted_by,
        approvers,
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
    /// `revision`, each signer with its verifier and its reach; for a
    /// channel, the channel it is bound to and the signers whose
    /// promotion of it counts (32). Absent naming the audience where the
    /// file or its table is missing and the channel is neither on in the
    /// table in force nor made (C.10), and unreadable naming the file and
    /// the revision, with git's or the parser's words, where it cannot be
    /// read.
    fn policy_for(&self, revision: &Revision, audience: &Audience) -> Answer<Required> {
        let absent = || {
            Not::absent(
                self.source(),
                format!("policy for {} in {POLICY_FILE}", audience_name(audience)),
            )
        };
        let unreadable = |said: String| self.unreadable(revision, said);
        let read = self.read(revision)?;
        if let Audience::Channel(channel) = audience {
            let (file, signers) = match &read {
                Some(Read { file, signers }) => (file, signers),
                None => (&File::none(), &BTreeMap::new()),
            };
            let resolved = resolve(&in_force(file, self.givens()?)).map_err(unreadable)?;
            let entry = resolved.get(channel).ok_or_else(absent)?;
            return channel_policy(entry, signers).map_err(unreadable);
        }
        let Some(Read { file, signers }) = read else {
            return Err(absent());
        };
        let (table, name) = match audience {
            Audience::Acceptance => (file.acceptance.as_ref(), "[acceptance]"),
            Audience::Layout => (file.layout.as_ref(), "[layout]"),
            Audience::Channel(_) => unreachable!("a channel is answered above"),
        };
        let table = table.ok_or_else(absent)?;
        if table.bound_to.is_some() {
            return Err(unreadable(format!(
                "{name} names bound_to, and only a channel is bound to another"
            )));
        }
        let claims = claims_of(name, &table.require, &signers).map_err(unreadable)?;
        Ok(match audience {
            Audience::Layout => Required::Layout { claims },
            _ => Required::Acceptance { claims },
        })
    }

    /// 131, C.10: every channel the table in force at `revision` has on,
    /// given, made or declared, once each by name and then component,
    /// each with its component and how it moves; the givens that are on,
    /// of the one component, where the revision holds no file; and
    /// unreadable naming the file and the revision, with the words of the
    /// first table that breaks 125, 37 or C.10, where one does.
    fn channels(&self, revision: &Revision) -> Answer<PolicyChannels> {
        let read = self.read(revision)?;
        let file = match &read {
            Some(Read { file, .. }) => file,
            None => &File::none(),
        };
        let resolved = resolve(&in_force(file, self.givens()?))
            .map_err(|said| self.unreadable(revision, said))?;
        Ok(PolicyChannels {
            in_force: resolved
                .into_iter()
                .map(|(channel, entry)| DeclaredChannel {
                    channel,
                    moves: entry.moves,
                })
                .collect(),
            held: vec![],
        })
    }
}

impl File {
    /// What a revision with no policy file declares: nothing.
    fn none() -> File {
        File {
            signers: BTreeMap::new(),
            acceptance: None,
            layout: None,
            channels: toml::Table::new(),
            environments: toml::Table::new(),
            component: Vec::new(),
        }
    }
}
