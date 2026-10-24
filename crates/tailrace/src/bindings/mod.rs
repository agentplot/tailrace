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
//! [signing]
//! signer = "laptop"
//! key = "~/.config/tailrace/signing"
//! ```
//!
//! `[store]` and `[landing]` each name their profile by their one key,
//! so a later profile is a new key and not a new shape; `[signing]`
//! names the host's signer and its key file, in the landing's own
//! `[signing]` shape. A path is absolute or begins `~/`, expanded
//! against the home directory. This module reads each table generically
//! and names no tool (rule core.1): the key of each seam's table is
//! matched to its profile by the `bind` function of that seam's profile
//! family, under `profiles/`.
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
//! D6). No ledger and no target are bound yet. Only a publish makes the
//! layout it names (60, rule core.32).

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use tailrace_seams::{
    Answer, ChannelName, Digest, Envelope, Keeper, Not, Seam, Seams, SignerId, Source,
};

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

/// What the bindings file names: the store's table and the landing's,
/// where it holds them, and the host's signer.
#[derive(Clone, Debug, PartialEq)]
pub struct Bindings {
    pub file: PathBuf,
    pub store: Option<Table>,
    pub landing: Option<Table>,
    pub signing: Option<Signing>,
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

/// `[signing]`: the identity the host signs as, and its key file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signing {
    pub signer: SignerId,
    pub key: PathBuf,
}

/// The host and the bindings file, as every answer of the file names it.
pub fn source(file: &Path) -> Source {
    Source::host(file)
}

/// `at` of the file, `[landing]` or `[signing] key`, could not be read,
/// for `said`.
fn unreadable(file: &Path, at: Option<&str>, said: impl Into<String>) -> Not {
    let what = match at {
        Some(at) => format!("{}, {at}", file.display()),
        None => file.display().to_string(),
    };
    Not::unreadable(&source(file), what, Some(said.into()))
}

/// `text` as a path: absolute, or beginning `~/` and expanded against
/// `home`; unreadable naming `at` otherwise, since a relative path would
/// mean another file in each working directory.
fn path_of(file: &Path, home: Option<&Path>, at: &str, text: &str) -> Answer<PathBuf> {
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
        signing: None,
    };
    for (name, value) in tables {
        let at = format!("[{name}]");
        let Some(table) = value.as_table() else {
            return Err(unreadable(
                file,
                Some(&name),
                format!("{name} is not a table; the file binds [store], [landing] and [signing]"),
            ));
        };
        match name.as_str() {
            "store" => bindings.store = Some(seam(file, home, "store", table)?),
            "landing" => bindings.landing = Some(seam(file, home, "landing", table)?),
            "signing" => bindings.signing = Some(signing(file, home, table)?),
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
            "ledger" => {
                return Err(unreadable(
                    file,
                    Some(&at),
                    format!("{at} names a ledger, and no ledger profile is built yet"),
                ))
            }
            _ => {
                return Err(unreadable(
                    file,
                    Some(&at),
                    format!(
                        "{at} is no table of the bindings file, which binds [store], \
                         [landing] and [signing]"
                    ),
                ))
            }
        }
    }
    Ok(bindings)
}

/// A seam's table, `[store]` or `[landing]`, by its one key.
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

/// `[signing]`: `signer` and `key`, and nothing else.
fn signing(file: &Path, home: Option<&Path>, table: &toml::Table) -> Answer<Signing> {
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
    Ok(Signing {
        signer: SignerId::new(signer),
        key,
    })
}

/// The command a binding is made for: publish, which makes the layout
/// the file names the first time, or verify or sync, which never make or
/// write one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum For {
    Publish,
    Verify,
    Sync,
}

/// The ledger, which the host's file names no profile of until bolt 4's
/// ledger unit: every operation absent naming its binding.
struct NoLedger;

impl NoLedger {
    fn absent() -> Not {
        Not::absent(
            &Source::new(Seam::Keeper, "unbound"),
            "binding of the ledger",
        )
    }
}

impl Keeper for NoLedger {
    fn append(&self, _: &ChannelName, _: &Envelope, _: Option<&Digest>) -> Answer<Digest> {
        Err(NoLedger::absent())
    }

    fn chain(&self, _: &ChannelName) -> Answer<Vec<Envelope>> {
        Err(NoLedger::absent())
    }
}

/// The binding the bindings file at `file` names for `command`, in the
/// repository at `repository`, its `~/` paths expanded against `home`.
/// A publish on a host whose file names no `[signing]` is absent naming
/// it before anything is read; a file naming no `[store]` or no
/// `[landing]` is absent naming the table. Every table is read before the
/// layout is made, so a file that cannot bind leaves nothing behind. The
/// store is named as the file names it, its key and what it names, a
/// path with `~/` expanded, so the record names a directory a reader
/// finds whoever's home it was written under (61, 118; design D9).
pub fn bind(file: &Path, home: Option<&Path>, repository: &Path, command: For) -> Answer<Binding> {
    let bindings = read(file, home)?;
    let signing = bindings.signing.as_ref();
    if command == For::Publish && signing.is_none() {
        return Err(unsigned());
    }
    let missing = |table: &str| {
        Not::absent(
            &source(file),
            format!("binding of the {table}, [{table}] in {}", file.display()),
        )
    };
    let landing = bindings
        .landing
        .as_ref()
        .ok_or_else(|| missing("landing"))?;
    let store = bindings.store.as_ref().ok_or_else(|| missing("store"))?;
    let named = profiles::store::named(store)?;
    let landing = profiles::landing::bind(landing, repository)?;
    let bound = profiles::store::bind(store, command == For::Publish)?;
    // After the seams' own store, so a publish has made the layout.
    let cache = profiles::store::bind(store, false)?;
    Ok(Binding {
        seams: Seams {
            store: bound,
            build: profiles::build::bind(repository),
            landing,
            policy: profiles::policy::bind(repository),
            keeper: Box::new(NoLedger),
            signers: profiles::signer::bind(signing),
            projections: profiles::projection::bound(cache, repository),
            targets: BTreeMap::new(),
        },
        store: named,
        signer: signing.map(|s| s.signer.clone()),
    })
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

/// What a command that reads no bindings file is handed in place of a
/// binding: `status`, which needs no seam (60).
pub fn not_read() -> Not {
    Not::absent(
        &Source::new(Seam::Host, "bindings"),
        "binding, since the command reads no bindings file",
    )
}
