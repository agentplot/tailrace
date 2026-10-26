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
//! runs. A path is absolute or begins `~/`, expanded against
//! the home directory. This module reads each table generically and names no
//! tool or service (rule core.1): the key of each seam's table is
//! matched to its profile by the `bind` function of that seam's profile
//! family, and `[signing]` is read whole by the signer profiles' family,
//! under `profiles/`.
//!
//! A missing file is absent naming its path, and a broken one unreadable
//! naming the file and the table or key, with the parser's words where
//! it has them (58, 59, rule core.19).
//!
//! The binding made from the file binds the store, the landing and the
//! signers it names, and from the repository the command runs in the
//! build, the policy and every projection built, which the repository's
//! policy file picks among (92, C.10); the moon cache reads cache entries
//! through a second binding of `[store]`, opened and never made (design
//! D6). The ledger is bound where the file names `[ledger]`, and answers
//! absent naming it where it does not; what tells the gate is bound for
//! every command, and is absent naming `[gate]` where the file names
//! none; no target is bound yet. Only a
//! publish and a push make the layout they name, and a push needs no
//! `[signing]`, since it signs nothing (60, rule core.32); a promote
//! needs `[signing]` and `[ledger]` and makes nothing (C.13, C.14).
//!
//! Status asks the landing, the policy and the ledger alone (60): it
//! needs `[landing]` and no `[signing]`, binds `[ledger]` and `[store]`
//! where the file names them, the store opened and never made, and a
//! store answering absent naming `[store]` where it names none. On a
//! host with no bindings file status is bound as 118 binds such a host,
//! as far as status asks: the landing and the ledger the landing and
//! ledger families bind for a repository with no file, each with the
//! remote `origin` where the repository names one and none otherwise,
//! no store and no signer, and nothing made (C.5, C.6, C.13; design D2).

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use tailrace_core::History;
use tailrace_seams::{
    Answer, Chain, ChannelName, Digest, Envelope, Held, Keeper, Manifest, Name, Not, Put, Seam,
    Seams, Source, Store,
};

use crate::profiles::signer::Signing;
use crate::{profiles, unsigned, Binding};

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

/// What the bindings file names: the store's table, the landing's, the
/// ledger's and the gate's, where it holds them, and the host's signer.
#[derive(Clone, Debug, PartialEq)]
pub struct Bindings {
    pub file: PathBuf,
    pub store: Option<Table>,
    pub landing: Option<Table>,
    pub ledger: Option<Table>,
    pub gate: Option<Table>,
    pub signing: Option<Signing>,
}

/// What tells this host's gate of a version a publish offered (95, 130,
/// C.16): the one key of `[gate]`, or nothing where the file holds none.
#[derive(Clone, Debug, PartialEq)]
pub enum Teller {
    /// `[gate] publish = {}`: this host's publish tells its own gate.
    Publish,
    /// `[gate] job = "<words>"`: a job outside tailrace tells it, the
    /// words as written; tailrace runs nothing for it.
    Job(String),
    /// No `[gate]`: absent naming the table in the file.
    Nothing(Not),
}

impl Teller {
    /// The teller `gate`, the `[gate]` of the file at `file`, names:
    /// `publish` an empty table, `job` a string with words in it, and
    /// any other key unreadable naming the two the table reads.
    pub fn of(file: &Path, gate: Option<&Table>) -> Answer<Teller> {
        let Some(table) = gate else {
            return Ok(Teller::Nothing(Not::absent(
                &source(file),
                format!(
                    "binding of what tells the gate, [gate] in {}",
                    file.display()
                ),
            )));
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

/// A seam's table: the one key naming its profile, and that key's value.
#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    file: PathBuf,
    home: Option<PathBuf>,
    name: &'static str,
    key: String,
    value: toml::Value,
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
    /// The key naming the seam's profile.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The value of the key, which its profile reads.
    pub fn value(&self) -> &toml::Value {
        &self.value
    }

    /// Where the table is: `[store] layout`.
    fn at(&self) -> String {
        format!("[{}] {}", self.name, self.key)
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

    /// What a profile answers where it needs `table`, which the file
    /// beside this table does not hold: absent naming it in the file.
    pub fn missing(&self, table: &str) -> Not {
        missing(&self.file, table)
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
/// absent naming the path where there is no file, and unreadable where
/// it cannot be read, is not well formed, or names a table this file
/// does not bind or a seam's table naming no profile or several.
pub fn read(file: &Path, home: Option<&Path>) -> Answer<Bindings> {
    let text = match fs::read(file) {
        Ok(bytes) => String::from_utf8(bytes).map_err(|e| unreadable(file, None, e.to_string()))?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Err(Not::absent(
                &source(file),
                format!("bindings file at {}", file.display()),
            ))
        }
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
    };
    for (name, value) in tables {
        let at = format!("[{name}]");
        let Some(table) = value.as_table() else {
            return Err(unreadable(
                file,
                Some(&name),
                format!(
                    "{name} is not a table; the file binds [store], [landing], [ledger], \
                     [signing] and [gate]"
                ),
            ));
        };
        match name.as_str() {
            "store" => bindings.store = Some(seam(file, home, "store", table)?),
            "landing" => bindings.landing = Some(seam(file, home, "landing", table)?),
            "ledger" => bindings.ledger = Some(seam(file, home, "ledger", table)?),
            "gate" => bindings.gate = Some(seam(file, home, "gate", table)?),
            "signing" => bindings.signing = Some(profiles::signer::read(file, home, table)?),
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
                         [landing], [ledger], [signing] and [gate]"
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

/// The command a binding is made for: publish or push, which make the
/// layout the file names the first time; verify, sync or rebuild, which
/// never make or write one; promote, which needs the host's signer and
/// its ledger and never makes the layout; or status, which needs the
/// landing alone and is bound on a host with no file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum For {
    Publish,
    Verify,
    Sync,
    Push,
    Rebuild,
    Promote,
    Status,
}

/// The remote a host with no bindings file binds, where the repository
/// names it: the default name of the remote a clone came from.
pub const ORIGIN: &str = "origin";

/// `table` is not in the file at `file`: absent naming it.
fn missing(file: &Path, table: &str) -> Not {
    Not::absent(
        &source(file),
        format!("binding of the {table}, [{table}] in {}", file.display()),
    )
}

/// The ledger of a host whose file names no `[ledger]`: every operation
/// absent naming the table in the file.
struct NoLedger {
    file: PathBuf,
}

impl NoLedger {
    fn absent(&self) -> Not {
        Not::absent(
            &Source::new(Seam::Keeper, "unbound"),
            format!("binding of the ledger, [ledger] in {}", self.file.display()),
        )
    }
}

impl Keeper for NoLedger {
    fn append(&self, _: &ChannelName, _: &Envelope, _: Option<&Digest>) -> Answer<Digest> {
        Err(self.absent())
    }

    fn chain(&self, _: &ChannelName) -> Answer<Chain> {
        Err(self.absent())
    }
}

/// The store of a binding that names none: every operation absent naming
/// what was looked for (60).
struct NoStore {
    what: String,
}

impl NoStore {
    fn absent(&self) -> Not {
        Not::absent(&Source::new(Seam::Store, "unbound"), &self.what)
    }
}

impl Store for NoStore {
    fn put_bytes(&self, _: &[u8]) -> Answer<Put> {
        Err(self.absent())
    }

    fn put_artifact(&self, _: &Manifest) -> Answer<Put> {
        Err(self.absent())
    }

    fn attach(&self, _: &Digest, _: &Envelope) -> Answer<Put> {
        Err(self.absent())
    }

    fn claims_of(&self, _: &Digest) -> Answer<Vec<Envelope>> {
        Err(self.absent())
    }

    fn name(&self, _: &Name, _: &Digest, _: Option<&Digest>) -> Answer<()> {
        Err(self.absent())
    }

    fn resolve(&self, _: &Name) -> Answer<Digest> {
        Err(self.absent())
    }

    fn read(&self, _: &Digest) -> Answer<Held> {
        Err(self.absent())
    }
}

/// How a binding with no store names it, which status keeps nowhere.
fn unbound_store() -> Source {
    Source::new(Seam::Store, "unbound")
}

/// The binding the bindings file at `file` names for `command`, in the
/// repository at `repository`, its `~/` paths expanded against `home`.
/// A publish or a promote on a host whose file names no `[signing]`, and
/// a promote on one whose file names no `[ledger]`, is absent naming it
/// before anything is read; a file naming no `[store]` or no `[landing]`
/// is absent naming the table. Every table is read before the
/// layout is made, `[gate]` among them, so a file that cannot bind
/// leaves nothing behind. The
/// store is named as the file names it, its key and what it names, a
/// path with `~/` expanded, so the record names a directory a reader
/// finds whoever's home it was written under (61, 118; design D9). The
/// signers are bound in the signer family's order: the one `[signing]`
/// names first, the one publish signs with, and every other bound
/// profile after it, verify-only (92).
pub fn bind(file: &Path, home: Option<&Path>, repository: &Path, command: For) -> Answer<Binding> {
    if command == For::Status {
        return status(file, home, repository);
    }
    let bindings = read(file, home)?;
    let gate = Teller::of(file, bindings.gate.as_ref())?;
    let signing = bindings.signing.as_ref();
    if matches!(command, For::Publish | For::Promote) && signing.is_none() {
        return Err(unsigned());
    }
    if command == For::Promote && bindings.ledger.is_none() {
        return Err(missing(file, "ledger"));
    }
    let landing = bindings
        .landing
        .as_ref()
        .ok_or_else(|| missing(file, "landing"))?;
    let store = bindings
        .store
        .as_ref()
        .ok_or_else(|| missing(file, "store"))?;
    let named = profiles::store::named(store)?;
    let landing = profiles::landing::bind(landing, repository)?;
    let bound = profiles::store::bind(store, matches!(command, For::Publish | For::Push))?;
    // After the seams' own store, so a publish or a push has made the
    // layout.
    let cache = profiles::store::bind(store, false)?;
    let keeper: Box<dyn Keeper> = match &bindings.ledger {
        Some(ledger) => profiles::ledger::bind(ledger, repository, Some(store))?,
        None => Box::new(NoLedger {
            file: file.to_path_buf(),
        }),
    };
    Ok(Binding {
        seams: Seams {
            store: bound,
            build: profiles::build::bind(repository),
            landing,
            policy: profiles::policy::bind(repository),
            keeper,
            signers: profiles::signer::bind(signing),
            projections: profiles::projection::bound(cache, repository),
            targets: BTreeMap::new(),
        },
        store: named,
        signer: signing.map(Signing::signer),
        gate,
    })
}

/// 60, 118: status's binding, of the landing, the policy and the ledger:
/// from the file at `file` where there is one, needing `[landing]`, the
/// ledger and the store bound where it names them, the store never made,
/// and no signer; with no file there, as a host with no file is bound.
fn status(file: &Path, home: Option<&Path>, repository: &Path) -> Answer<Binding> {
    let bindings = match read(file, home) {
        Ok(bindings) => bindings,
        Err(Not::Absent(_)) => return unfiled(file, repository),
        Err(not) => return Err(not),
    };
    let landing = bindings
        .landing
        .as_ref()
        .ok_or_else(|| missing(file, "landing"))?;
    let gate = Teller::of(file, bindings.gate.as_ref())?;
    let landing = profiles::landing::bind(landing, repository)?;
    let (store, named): (Box<dyn Store>, Source) = match &bindings.store {
        Some(store) => (
            profiles::store::bind(store, false)?,
            profiles::store::named(store)?,
        ),
        None => (
            Box::new(NoStore {
                what: format!("binding of the store, [store] in {}", file.display()),
            }),
            unbound_store(),
        ),
    };
    let keeper: Box<dyn Keeper> = match &bindings.ledger {
        Some(ledger) => profiles::ledger::bind(ledger, repository, bindings.store.as_ref())?,
        None => Box::new(NoLedger {
            file: file.to_path_buf(),
        }),
    };
    Ok(Binding {
        seams: Seams {
            store,
            build: profiles::build::bind(repository),
            landing,
            policy: profiles::policy::bind(repository),
            keeper,
            signers: profiles::signer::bind(None),
            projections: BTreeMap::new(),
            targets: BTreeMap::new(),
        },
        store: named,
        signer: None,
        gate,
    })
}

/// 118, as far as status asks: a host with no bindings file at `file`,
/// in the repository at `repository`, bound to the landing and the
/// ledger each family binds for a repository with no file, with the
/// remote `origin` where the repository names one; no store and no
/// signer; and nothing made (C.5, C.6, C.13).
fn unfiled(file: &Path, repository: &Path) -> Answer<Binding> {
    let remote = origin(repository)?;
    Ok(Binding {
        seams: Seams {
            store: Box::new(NoStore {
                what: format!(
                    "binding of the store, which a host with no bindings file at {} binds none",
                    file.display()
                ),
            }),
            build: profiles::build::bind(repository),
            landing: profiles::landing::unfiled(repository, remote.clone()),
            policy: profiles::policy::bind(repository),
            keeper: profiles::ledger::unfiled(repository, remote),
            signers: profiles::signer::bind(None),
            projections: BTreeMap::new(),
            targets: BTreeMap::new(),
        },
        store: unbound_store(),
        signer: None,
        gate: Teller::of(file, None)?,
    })
}

/// `origin`, where the repository at `repository` names that remote.
fn origin(repository: &Path) -> Answer<Option<String>> {
    Ok(History::open(repository)?
        .remote_url(ORIGIN)?
        .map(|_| ORIGIN.to_string()))
}

/// 114: the remote of the landing status binds in the repository at
/// `repository`: the one `[landing]` names where the file at `file` is
/// there, `origin` where it is not and the repository names it, and
/// none otherwise. It reads the file and binds nothing.
pub fn landing_remote(
    file: &Path,
    home: Option<&Path>,
    repository: &Path,
) -> Answer<Option<String>> {
    match read(file, home) {
        Ok(bindings) => {
            let landing = bindings
                .landing
                .as_ref()
                .ok_or_else(|| missing(file, "landing"))?;
            profiles::landing::remote(landing)
        }
        Err(Not::Absent(_)) => origin(repository),
        Err(not) => Err(not),
    }
}

/// The binding for `command` in the repository at `repository`, from the
/// bindings file this process's `XDG_CONFIG_HOME` or `HOME` places;
/// absent naming both variables where neither is set.
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
    bind(&file, home.as_deref(), repository, command)
}
