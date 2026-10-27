//! The host's bindings (92, 118; design D1, D2): one TOML file the
//! operator keeps beside the landing's own, at
//! `$XDG_CONFIG_HOME/tailrace/config.toml`, or under `~/.config` where
//! that variable is unset or empty, readable with any file reader (62).
//! It binds what the operator binds for a host, and nothing the
//! repository's authors bind:
//!
//! ```toml
//! [store]
//! layout = "~/.local/share/tailrace/layout"
//! # registry = "registry.example.com/tailrace"   # a registry in its place
//!
//! [landing]
//! branch = { remote = "origin" }   # the remote optional
//!
//! [ledger]
//! git_ref = { remote = "origin" }  # the remote optional
//! # store = {}                     # the chain held in [store] in its place
//!
//! [signing]
//! signer = "laptop"
//! key = "~/.config/tailrace/signing"
//!
//! [gate]
//! publish = {}                     # this host's publish tells its own gate
//! # job = "the release job a push to main wakes"   # a gate told elsewhere
//!
//! [trust.gate]
//! ssh = "ssh-ed25519 AAAA… gate@box"
//! ```
//!
//! `[store]`, `[landing]` and `[ledger]` each name their profile by their
//! one key, so a later profile is a new key and not a new shape;
//! `[signing]` names the host's signer and its key file, in the
//! landing's own `[signing]` shape, or a key in a key service by the
//! key's own name. `[gate]` names what tells the host's gate of a
//! version a publish offered, by one key in the same form (130, C.16):
//! `publish`, this host's own publish step, or `job`, the operator's
//! words for a job outside tailrace, which tailrace carries and never
//! runs. A file with no `[gate]` binds `publish`, the host's default
//! (118): the publish's own host tells its gate. `[trust.<signer>]` names
//! a signer the host trusts, one table each, by its one key, whose text
//! is what the signer's profile verifies it by, as the policy file's
//! `[signers]` names one but with no `kinds` or `checks`: a public key,
//! never a credential (64, rule core.21). For reading, the host trusts
//! its own signer, the one `[signing]` names, by what its profile
//! verifies it by, a host key's public half beside the key, and the
//! signers `[trust]` names, so a host consumes what it published itself;
//! a consumer reads through them every claim its policy file names no
//! signer for, and every claim where it has no policy file, what each may
//! claim being the reader's to say (118, C.10; design D12). A path is
//! absolute or begins `~/`, expanded against the home directory. This
//! module reads each table generically and names no
//! tool or service (rule core.1): the key of each seam's table is
//! matched to its profile by the `bind` function of that seam's profile
//! family, and `[signing]` is read whole by the signer profiles' family,
//! under `profiles/`.
//!
//! A host with no file is bound as one whose file is empty, and a table
//! the file leaves out is bound to the binding its seam's profile family
//! gives in its place (118, design D1, D2): the store a layout at
//! `tailrace/layout` under the user's data directory (C.2); the landing
//! the repository's `HEAD` commit is set up for (C.5, C.6), over the
//! checkout's `origin` or its one remote; the ledger the git ref on the
//! landing's remote, written or given, or in the repository's own refs
//! where the landing has none (C.13); for a publish or a promote, the
//! host key at `tailrace/host/signing.key` under the same data
//! directory, made on its first sign and signing as the host's name
//! (C.8); and the host's own publish as what tells its gate (130). A
//! file that is there is
//! never guessed past: a broken one is unreadable naming the file and the
//! table or key, with the parser's words where it has them (58, 59, rule
//! core.19), and binds nothing given in its place.
//!
//! The binding binds the store, the landing, the ledger and the signers,
//! written or given, carries the signers it trusts, its own among them
//! for a sync, the one command that reads through them, and from the
//! repository the command runs in the
//! build, the policy and every projection built, which the repository's
//! policy file picks among (92, C.10); the moon cache reads cache entries
//! through a second binding of the store, opened and never made (design
//! D6). What tells the gate is bound for every command; no target is
//! bound yet. Only a publish and a push make the layout, the given one
//! with its parents, at the first operation asked of the store and never
//! in binding it, so a publish refused before it asks the store anything
//! leaves nothing behind; a push signs nothing (60, rules core.11,
//! core.32); every other command opens it, and a promote makes nothing
//! (C.13, C.14).
//!
//! Status asks the landing, the policy and the ledger alone (60; design
//! D8): it is bound by the same path, with no signer but every profile
//! verify-only, and the store opened, never made, only for a ledger held
//! in it, every operation of the store answering absent naming it
//! otherwise; so status on a host with nothing written reads the given
//! landing and the given ledger and makes nothing. The policy command is
//! bound as status is, with no store whether or not the file names one
//! (the channels unit's design D11).

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use tailrace_seams::{
    Answer, Digest, Envelope, Held, Manifest, Name, NamedSigner, Not, Put, Seam, Seams, SignerId,
    Source, Store,
};

use crate::profiles::signer::Signing;
use crate::{profiles, Binding};

#[cfg(test)]
mod tests;

/// Where the bindings file is, under the configuration directory.
pub const FILE: &str = "tailrace/config.toml";

/// The bindings file's path: under `xdg_config_home` where it is set and
/// not empty, else under `home`'s `.config`; `None` where neither is set.
pub fn path(xdg_config_home: Option<&OsStr>, home: Option<&Path>) -> Option<PathBuf> {
    let config = match xdg_config_home {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => home?.join(".config"),
    };
    Some(config.join(FILE))
}

/// The user's data directory, where the given layout and the given host
/// key are made (118, C.2, C.8): under `xdg_data_home` where it is set
/// and not empty, else `home`'s `.local/share`; `None` where neither is
/// set.
pub fn data(xdg_data_home: Option<&OsStr>, home: Option<&Path>) -> Option<PathBuf> {
    match xdg_data_home {
        Some(dir) if !dir.is_empty() => Some(PathBuf::from(dir)),
        _ => Some(home?.join(".local/share")),
    }
}

/// What the bindings file names: the store's table, the landing's, the
/// ledger's and the gate's, where it holds them, the host's signer, and
/// the other signers it trusts, each with no claim of its own to make.
#[derive(Clone, Debug, PartialEq)]
pub struct Bindings {
    pub file: PathBuf,
    pub store: Option<Table>,
    pub landing: Option<Table>,
    pub ledger: Option<Table>,
    pub gate: Option<Table>,
    pub signing: Option<Signing>,
    pub trust: Vec<NamedSigner>,
}

/// What tells this host's gate of a version a publish offered (95, 130,
/// C.16): the one key of `[gate]`, or the host's own publish where the
/// file holds none (118).
#[derive(Clone, Debug, PartialEq)]
pub enum Teller {
    /// `[gate] publish = {}`, or no `[gate]` (118): this host's publish
    /// tells its own gate.
    Publish,
    /// `[gate] job = "<words>"`: a job outside tailrace tells it, the
    /// words as written; tailrace runs nothing for it.
    Job(String),
}

impl Teller {
    /// The teller `gate`, the file's `[gate]`, names: `publish` an empty
    /// table, `job` a string with words in it, and any other key
    /// unreadable naming the two the table reads; the host's publish
    /// where the file holds no `[gate]` (118).
    pub fn of(gate: Option<&Table>) -> Answer<Teller> {
        let Some(table) = gate else {
            return Ok(Teller::Publish);
        };
        match table.key() {
            "publish" => match table.value().as_table() {
                Some(options) if options.is_empty() => Ok(Teller::Publish),
                _ => Err(table.unreadable("[gate] publish takes no options: publish = {}")),
            },
            "job" => match table.value().as_str() {
                Some(words) if !words.trim().is_empty() => Ok(Teller::Job(words.to_string())),
                Some(_) => Err(table.unreadable("[gate] job holds no words naming the job")),
                None => Err(table.unreadable("[gate] job is not the job's words in a string")),
            },
            _ => Err(table.unknown(&["publish", "job"])),
        }
    }
}

/// A seam's table: the one key naming its profile, and that key's value,
/// as the file names it or, where the file names no such table, as its
/// seam's profile family gives it (118).
#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    file: PathBuf,
    home: Option<PathBuf>,
    name: &'static str,
    key: String,
    value: toml::Value,
    given: bool,
}

/// The host and the bindings file, as every answer of the file names it.
pub fn source(file: &Path) -> Source {
    Source::host(file)
}

/// `at` of the file, `[landing]` or `[signing] key`, could not be read,
/// for `said`.
pub(crate) fn unreadable(file: &Path, at: Option<&str>, said: impl Into<String>) -> Not {
    let what = match at {
        Some(at) => format!("{}, {at}", file.display()),
        None => file.display().to_string(),
    };
    Not::unreadable(&source(file), what, Some(said.into()))
}

/// `text` as a path: absolute, or beginning `~/` and expanded against
/// `home`; unreadable naming `at` otherwise, since a relative path would
/// mean another file in each working directory.
pub(crate) fn path_of(file: &Path, home: Option<&Path>, at: &str, text: &str) -> Answer<PathBuf> {
    if Path::new(text).is_absolute() {
        return Ok(PathBuf::from(text));
    }
    match (text.strip_prefix("~/"), home) {
        (Some(rest), Some(home)) => Ok(home.join(rest)),
        (Some(_), None) => Err(unreadable(
            file,
            Some(at),
            format!("{text} begins ~/, and no home directory is set"),
        )),
        (None, _) => Err(unreadable(
            file,
            Some(at),
            format!("{text} is a relative path; a path here is absolute or begins ~/"),
        )),
    }
}

impl Table {
    /// The table `[name]` the file at `file` leaves out, made by its
    /// seam's profile family as the binding it gives in its place (118):
    /// `key` naming its profile and `value` that key's value, a path in
    /// it absolute. Its answers name the file and the table as given.
    pub fn given(file: &Path, name: &'static str, key: &str, value: toml::Value) -> Table {
        Table {
            file: file.to_path_buf(),
            home: None,
            name,
            key: key.to_string(),
            value,
            given: true,
        }
    }

    /// The key naming the seam's profile.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The value of the key, which its profile reads.
    pub fn value(&self) -> &toml::Value {
        &self.value
    }

    /// Where the table is: `[store] layout`, or `[store] layout (given)`
    /// where the file leaves it out.
    fn at(&self) -> String {
        let given = if self.given { " (given)" } else { "" };
        format!("[{}] {}{given}", self.name, self.key)
    }

    /// The value as a path, absolute or `~/` expanded against the home
    /// directory; unreadable naming the file, the table and the key
    /// otherwise.
    pub fn path(&self) -> Answer<PathBuf> {
        let text = self.value.as_str().ok_or_else(|| {
            unreadable(
                &self.file,
                Some(&self.at()),
                format!("{} is not a path in a string", self.at()),
            )
        })?;
        path_of(&self.file, self.home.as_deref(), &self.at(), text)
    }

    /// The value read as `T`, its profile's options; unreadable naming
    /// the file, the table and the key, with the parser's words, where
    /// it is not.
    pub fn options<T: DeserializeOwned>(&self) -> Answer<T> {
        self.value
            .clone()
            .try_into()
            .map_err(|e: toml::de::Error| unreadable(&self.file, Some(&self.at()), e.to_string()))
    }

    /// The value could not be read as its profile reads it, for `said`:
    /// unreadable naming the file, the table and the key.
    pub fn unreadable(&self, said: impl Into<String>) -> Not {
        unreadable(&self.file, Some(&self.at()), said)
    }

    /// What a seam's profile family answers for a key that names none of
    /// its profiles, `known` the keys it reads.
    pub fn unknown(&self, known: &[&str]) -> Not {
        unreadable(
            &self.file,
            Some(&format!("[{}]", self.name)),
            format!(
                "[{}] names {}, which no {} profile is bound by; it reads {}",
                self.name,
                self.key,
                self.name,
                known.join(" or ")
            ),
        )
    }
}

/// The bindings file at `file`, its `~/` paths expanded against `home`:
/// every table `None` where there is no file, as for an empty one (118;
/// design D1), and unreadable where it cannot be read, is not well
/// formed, or names a table this file does not bind or a seam's table
/// naming no profile or several.
pub fn read(file: &Path, home: Option<&Path>) -> Answer<Bindings> {
    let text = match fs::read(file) {
        Ok(bytes) => String::from_utf8(bytes).map_err(|e| unreadable(file, None, e.to_string()))?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(unreadable(file, None, e.to_string())),
    };
    let tables: toml::Table =
        toml::from_str(&text).map_err(|e| unreadable(file, None, e.to_string()))?;
    let mut bindings = Bindings {
        file: file.to_path_buf(),
        store: None,
        landing: None,
        ledger: None,
        gate: None,
        signing: None,
        trust: Vec::new(),
    };
    for (name, value) in tables {
        let at = format!("[{name}]");
        let Some(table) = value.as_table() else {
            return Err(unreadable(
                file,
                Some(&name),
                format!(
                    "{name} is not a table; the file binds [store], [landing], [ledger], \
                     [signing], [gate] and [trust]"
                ),
            ));
        };
        match name.as_str() {
            "store" => bindings.store = Some(seam(file, home, "store", table)?),
            "landing" => bindings.landing = Some(seam(file, home, "landing", table)?),
            "ledger" => bindings.ledger = Some(seam(file, home, "ledger", table)?),
            "gate" => bindings.gate = Some(seam(file, home, "gate", table)?),
            "signing" => bindings.signing = Some(profiles::signer::read(file, home, table)?),
            "trust" => {
                for (id, value) in table {
                    bindings.trust.push(trusted(file, id, value)?);
                }
            }
            "build" | "projections" => {
                return Err(unreadable(
                    file,
                    Some(&at),
                    format!(
                        "{at} is the repository's to bind, in its policy file, \
                         and not the host's"
                    ),
                ))
            }
            _ => {
                return Err(unreadable(
                    file,
                    Some(&at),
                    format!(
                        "{at} is no table of the bindings file, which binds [store], \
                         [landing], [ledger], [signing], [gate] and [trust]"
                    ),
                ))
            }
        }
    }
    Ok(bindings)
}

/// A table naming its profile by one key: a seam's, `[store]`,
/// `[landing]` or `[ledger]`, or `[gate]`, naming what tells the gate.
fn seam(
    file: &Path,
    home: Option<&Path>,
    name: &'static str,
    table: &toml::Table,
) -> Answer<Table> {
    let at = format!("[{name}]");
    let mut keys: Vec<&String> = table.keys().collect();
    keys.sort_unstable();
    match keys.as_slice() {
        [key] => Ok(Table {
            file: file.to_path_buf(),
            home: home.map(Path::to_path_buf),
            name,
            key: key.to_string(),
            value: table[key.as_str()].clone(),
            given: false,
        }),
        [] => Err(unreadable(
            file,
            Some(&at),
            format!("{at} names no profile"),
        )),
        several => {
            let names: Vec<&str> = several.iter().map(|k| k.as_str()).collect();
            Err(unreadable(
                file,
                Some(&at),
                format!(
                    "{at} names {}, and a seam is bound to one profile",
                    names.join(" and ")
                ),
            ))
        }
    }
}

/// `[trust.<id>]`: the signer `id` the host trusts, by its one key,
/// whose text is what the signer's profile verifies it by, with no claim
/// of its own to make, since a reader allows it the one claim it reads
/// through it; unreadable naming the table where it names no key, two, or
/// a key that is not a string (118, C.10).
fn trusted(file: &Path, id: &str, value: &toml::Value) -> Answer<NamedSigner> {
    let at = format!("[trust.{id}]");
    let cannot = |said: String| unreadable(file, Some(&at), said);
    let table = value.as_table().ok_or_else(|| {
        cannot(format!(
            "{at} is not a table naming the key {id} is verified by"
        ))
    })?;
    let mut keys: Vec<&String> = table.keys().collect();
    keys.sort_unstable();
    let key = match keys.as_slice() {
        [key] => key.as_str(),
        [] => return Err(cannot(format!("{at} names no key that verifies it"))),
        several => {
            let names: Vec<&str> = several.iter().map(|k| k.as_str()).collect();
            return Err(cannot(format!(
                "{at} names {}, and a signer is verified by one key",
                names.join(", ")
            )));
        }
    };
    let verified_by = table[key]
        .as_str()
        .ok_or_else(|| cannot(format!("{at} {key} is not a string")))?;
    Ok(NamedSigner {
        id: SignerId::new(id),
        verified_by: verified_by.to_string(),
        kinds: Vec::new(),
        checks: Vec::new(),
    })
}

/// The command a binding is made for: publish or push, which make the
/// layout the first time; verify, sync or rebuild, which never make or
/// write one; promote, which signs and never makes the layout; status,
/// which asks the landing, the policy and the ledger alone; or policy,
/// bound as status is with no store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum For {
    Publish,
    Verify,
    Sync,
    Push,
    Rebuild,
    Promote,
    Status,
    Policy,
}

/// The store status binds where no ledger is held in it, and the policy
/// command always, which neither asks: every operation answers why it
/// was not opened (60).
struct Unopened {
    not: Not,
}

impl Unopened {
    /// Every operation absent naming `what`, the binding not opened.
    fn absent(what: String) -> Box<dyn Store> {
        Box::new(Unopened {
            not: Not::absent(&Source::new(Seam::Store, "unbound"), what),
        })
    }
}

impl Store for Unopened {
    fn put_bytes(&self, _: &[u8]) -> Answer<Put> {
        Err(self.not.clone())
    }

    fn put_artifact(&self, _: &Manifest) -> Answer<Put> {
        Err(self.not.clone())
    }

    fn attach(&self, _: &Digest, _: &Envelope) -> Answer<Put> {
        Err(self.not.clone())
    }

    fn claims_of(&self, _: &Digest) -> Answer<Vec<Envelope>> {
        Err(self.not.clone())
    }

    fn name(&self, _: &Name, _: &Digest, _: Option<&Digest>) -> Answer<()> {
        Err(self.not.clone())
    }

    fn resolve(&self, _: &Name) -> Answer<Digest> {
        Err(self.not.clone())
    }

    fn read(&self, _: &Digest) -> Answer<Held> {
        Err(self.not.clone())
    }
}

/// The binding the bindings file at `file` names for `command`, in the
/// repository at `repository`, its `~/` paths expanded against `home`,
/// and the given layout placed under `xdg_data_home` or `home` (118;
/// design D1, D2). Each table the file names binds what it names, and
/// each it leaves out, or every table where there is no file, binds what
/// its seam's profile family gives: the layout under the data directory,
/// the landing the repository is set up for over the checkout's remote,
/// the git ref on the landing's remote, and for a publish or a promote
/// the given host key under the same data directory, signing as the name
/// its public half gives (C.2, C.5, C.6, C.8, C.13); every other command
/// binds both signers verify-only. A sync trusts the host's own signer,
/// the one `[signing]` names, before the signers `[trust]` names, what
/// verifies it read before the store is bound.
/// Every table is read, `[gate]` among them, and nothing is made: a
/// publish or a push makes the layout at its store's first operation, so
/// path with `~/` expanded, so the record names a directory a reader
/// finds whoever's home it was written under (61, 118; design D9). The
/// signers are bound in the signer family's order: the one the host
/// signs with first, and every other bound profile after it,
/// verify-only (92).
pub fn bind(
    file: &Path,
    home: Option<&Path>,
    xdg_data_home: Option<&OsStr>,
    repository: &Path,
    command: For,
) -> Answer<Binding> {
    let bindings = read(file, home)?;
    let gate = Teller::of(bindings.gate.as_ref())?;
    let given_signing;
    let signing = match (&bindings.signing, command) {
        (_, For::Status | For::Policy) => None,
        (Some(signing), _) => Some(signing),
        (None, For::Publish | For::Promote) => {
            let data = data(xdg_data_home, home).ok_or_else(|| {
                Not::absent(
                    &source(file),
                    format!(
                        "binding of the signer: neither XDG_DATA_HOME nor HOME is set to place \
                         {}",
                        profiles::signer::HOST_KEY
                    ),
                )
            })?;
            given_signing = profiles::signer::given(&data)?;
            Some(&given_signing)
        }
        (None, _) => None,
    };
    let mut trust = Vec::new();
    if command == For::Sync {
        trust.extend(profiles::signer::own(signing)?);
    }
    trust.extend(bindings.trust.iter().cloned());
    let given_store;
    let store = match &bindings.store {
        Some(store) => store,
        None => {
            given_store = profiles::store::given(file, xdg_data_home, home)?;
            &given_store
        }
    };
    let named = match command {
        For::Policy => Source::new(Seam::Store, "unbound"),
        _ => profiles::store::named(store)?,
    };
    // The landing's remote, as the file writes it or as the checkout
    // gives it, is where the given ledger is kept (C.13).
    let (landing, remote) = match &bindings.landing {
        Some(landing) => (
            profiles::landing::bind(landing, repository)?,
            profiles::landing::remote(landing)?,
        ),
        None => profiles::landing::given(file, repository)?,
    };
    // A publish or a push makes the layout at its store's first
    // operation, through whichever binding of it is asked first.
    let make = matches!(command, For::Publish | For::Push);
    let ledger = |store: &Table| match &bindings.ledger {
        Some(ledger) => profiles::ledger::bind(ledger, repository, store, make),
        None => Ok(profiles::ledger::given(repository, remote.clone())),
    };
    let (bound, keeper, projections) = if matches!(command, For::Status | For::Policy) {
        let keeper = ledger(store)?;
        // No part of status reads the store but a ledger held in it, so a
        // store that is not there leaves every other ledger's chains read.
        let bound = match (&bindings.ledger, command) {
            (_, For::Policy) => {
                Unopened::absent("binding of the store, which tailrace policy binds none".into())
            }
            (Some(ledger), _) if profiles::ledger::in_store(ledger) => {
                profiles::store::bind(store, false)?
            }
            _ => Unopened::absent(format!(
                "binding of the store, {} in {}, which status opens only for a ledger held in it",
                store.at(),
                file.display()
            )),
        };
        (bound, keeper, BTreeMap::new())
    } else {
        let bound = profiles::store::bind(store, make)?;
        let cache = profiles::store::bind(store, make)?;
        let keeper = ledger(store)?;
        (
            bound,
            keeper,
            profiles::projection::bound(cache, repository),
        )
    };
    Ok(Binding {
        seams: Seams {
            store: bound,
            build: profiles::build::bind(repository),
            landing,
            policy: profiles::policy::bind(repository),
            keeper,
            signers: profiles::signer::bind(signing),
            projections,
            targets: BTreeMap::new(),
        },
        store: named,
        signer: signing.map(Signing::signer),
        gate,
        trust,
    })
}

/// 114: the remote of the landing the host binds in the repository at
/// `repository`: the one `[landing]` names where the file at `file`
/// names it, and the given landing's remote where it names none or there
/// is no file (118; design D5, D8). It reads the file and binds nothing.
pub fn landing_remote(
    file: &Path,
    home: Option<&Path>,
    repository: &Path,
) -> Answer<Option<String>> {
    match &read(file, home)?.landing {
        Some(landing) => profiles::landing::remote(landing),
        None => profiles::landing::given_remote(file, repository),
    }
}

/// The binding for `command` in the repository at `repository`, from the
/// bindings file this process's `XDG_CONFIG_HOME` or `HOME` places, the
/// given layout under its `XDG_DATA_HOME` or `HOME`; absent naming both
/// variables where neither is set.
pub fn load(repository: &Path, command: For) -> Answer<Binding> {
    let home = std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(PathBuf::from);
    let xdg = std::env::var_os("XDG_CONFIG_HOME");
    let file = path(xdg.as_deref(), home.as_deref()).ok_or_else(|| {
        Not::absent(
            &Source::new(Seam::Host, "bindings"),
            format!("bindings file: neither XDG_CONFIG_HOME nor HOME is set to place {FILE}"),
        )
    })?;
    let data = std::env::var_os("XDG_DATA_HOME");
    bind(&file, home.as_deref(), data.as_deref(), repository, command)
}
