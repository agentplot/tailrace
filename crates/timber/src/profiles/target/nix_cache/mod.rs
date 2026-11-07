//! The Nix cache target (C.12's Nix binary cache row), bound as
//! `nix_cache`: a version's closures pushed to the fleet's Nix binary
//! cache with attic, from the host that holds them, so a closure built on
//! one host is realised on another from the cache alone.
//!
//! An environment names it in its `target` table, which names the cache
//! as attic's own command names it, `<server>:<cache>`, and the outputs
//! whose artifacts name closures, and nothing else (C.10; design D7):
//!
//! ```toml
//! [environments.swancloud-srv.cache]
//! channel = "beta"
//! target.nix_cache = { attic = "fleet:swancloud", closures = ["swancloud-srv"] }
//! ```
//!
//! Any other key, an empty `closures`, or an `attic` that is not
//! `<server>:<cache>` is unreadable naming the environment and the key,
//! at every operation, so no token is written where timber reads (45,
//! rule core.21).
//!
//! A closure is one artifact naming its roots: a machine's build task
//! leaves a file of its closure's root store paths, one per line, which
//! its component declares as an output and the file projection packs.
//! Its form is read by [`nix_closure`], which the Nix
//! profiles share (design D6). The store holds that file and no store
//! path of the closure; the paths are the cache's (section 10).
//!
//! Apply reads the verified version's index and each closure's file
//! through the store by digest (42, rule core.4), then checks every root
//! valid in this host's Nix store with `nix-store --check-validity`,
//! refusing the first it does not hold, naming it and the host: the host
//! that applies is the host that built, and nothing is built, substituted
//! or fetched (15, I3; design D8). Then `attic push <server>:<cache>
//! <roots…>` pushes each root with its closure, skipping what the cache
//! holds already. attic reads its servers and the environment's token
//! from its own configuration on that host,
//! `$XDG_CONFIG_HOME/attic/config.toml`, at the push: timber passes it no
//! token, sets none of its variables and reads nothing of its
//! configuration (45), and attic's words on a failure are the answer,
//! unreadable, verbatim (59).
//!
//! A Nix binary cache holds store paths by their hash and keeps no name a
//! marker could be held under, so current and report are unread, saying
//! so and naming the cache, never the digest last pushed and never
//! converged (119, 89, 98; design D9). desired answers the head the core
//! hands it (88), and apply runs whenever it is asked.

mod table;
#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::io;
use std::path::PathBuf;

use timber_seams::{
    Answer, Digest, Environment, Head, Not, Running, Seam, Source, Store, Target, Verified, Why,
};

use super::nix_closure::{self, StorePath};
use crate::profiles::run::Tool;
use table::{Named, Table};

/// The profile name an environment's `target` table names this target by.
pub const PROFILE: &str = "nix_cache";

/// The Nix cache target, reading through a store binding of its own,
/// opened and never made, and running attic and nix-store.
pub struct NixCache {
    store: Box<dyn Store>,
    attic: Tool,
    nix_store: Tool,
    source: Source,
}

impl NixCache {
    /// The target reading through `store`, running `attic` and
    /// `nix-store` as found on `PATH`.
    pub fn new(store: Box<dyn Store>) -> NixCache {
        NixCache::with_programs(store, "attic", "nix-store")
    }

    /// The target reading through `store`, running attic as `attic` and
    /// nix-store as `nix_store`.
    pub fn with_programs(
        store: Box<dyn Store>,
        attic: impl Into<PathBuf>,
        nix_store: impl Into<PathBuf>,
    ) -> NixCache {
        NixCache {
            store,
            attic: Tool::new(attic, NixCache::source()),
            nix_store: Tool::new(nix_store, NixCache::source()),
            source: NixCache::source(),
        }
    }

    /// The target seam and this profile, as every answer names them.
    pub fn source() -> Source {
        Source::new(Seam::Target, PROFILE)
    }

    /// 119: what the environment runs, unread, since the cache keeps no
    /// marker of what was pushed to it (design D9).
    fn no_marker(&self, environment: &Environment, table: &Table) -> Not {
        Not::unreadable(
            &self.source,
            format!("what {} runs", Named(environment)),
            Some(format!(
                "the Nix binary cache {} holds store paths by their hash and keeps no marker of \
                 what was pushed to it",
                table.attic
            )),
        )
    }

    /// Refused, by this target, for `said`.
    fn refused(&self, said: String) -> Not {
        Not::refused(&self.source, Why::Said, Some(said))
    }

    /// Every root of `roots` valid in this host's Nix store, or refused
    /// naming the first it does not hold and the host; nothing is built,
    /// substituted or fetched (15, I3; design D8).
    fn held(&self, roots: &[StorePath]) -> Answer<()> {
        let mut args = vec!["--check-validity", "--print-invalid"];
        args.extend(roots.iter().map(StorePath::as_str));
        let ran = self.nix_store.output(None, &args, &[], &[0])?;
        let printed = String::from_utf8_lossy(&ran.stdout);
        let invalid: BTreeSet<&str> = printed.lines().collect();
        match roots.iter().find(|root| invalid.contains(root.as_str())) {
            None => Ok(()),
            Some(root) => Err(self.refused(format!(
                "{root} is not valid in the Nix store of {}, and the {PROFILE} target pushes \
                 only what the host that applies holds, building and fetching nothing",
                host()
            ))),
        }
    }

    /// `attic push <server>:<cache> <roots…>`: attic reads the cache's
    /// server and token from its own configuration and pushes each root
    /// with its closure; timber passes it no token and sets none of its
    /// variables (45, rule core.21; design D7). attic's words on a
    /// failure are the answer, unreadable.
    fn push(&self, attic: &str, roots: &[StorePath]) -> Answer<()> {
        let mut args = vec!["push", attic];
        args.extend(roots.iter().map(StorePath::as_str));
        self.attic.output(None, &args, &[], &[0]).map(|_| ())
    }
}

/// The host this target runs on, by the name the operating system
/// answers, or saying it cannot be read.
fn host() -> String {
    let mut name = [0u8; 256];
    // SAFETY: the pointer and the length are the buffer's own, and
    // gethostname writes no more than the length it is given.
    let made = unsafe { libc::gethostname(name.as_mut_ptr().cast(), name.len()) };
    if made != 0 {
        return format!(
            "this host, whose name cannot be read: {}",
            io::Error::last_os_error()
        );
    }
    let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
    format!("the host {}", String::from_utf8_lossy(&name[..end]))
}

impl Target for NixCache {
    /// 88: the head the core handed, as a target that is pushed to
    /// answers it, once the table is read.
    fn desired(&self, environment: &Environment, head: Head<'_>) -> Answer<Digest> {
        Table::read(environment, &self.source)?;
        head()
    }

    fn current(&self, environment: &Environment) -> Answer<Digest> {
        let table = Table::read(environment, &self.source)?;
        Err(self.no_marker(environment, &table))
    }

    /// 90: the roots of every closure the table names, read from the
    /// version's index through the store, checked held in this host's Nix
    /// store, and pushed with attic; nothing runs before every closure is
    /// read, and attic does not run before every root is held.
    fn apply(&self, environment: &Environment, version: &Verified) -> Answer<()> {
        let table = Table::read(environment, &self.source)?;
        let index = nix_closure::index(self.store.as_ref(), version, &self.source)?;
        let store_dir = nix_closure::store_dir();
        let mut roots: Vec<StorePath> = Vec::new();
        for output in &table.closures {
            let artifact = nix_closure::artifact_of(&index, output, &self.source)?;
            let bytes = nix_closure::bytes(self.store.as_ref(), artifact, &self.source)?;
            for root in nix_closure::roots(artifact, &bytes, &store_dir, &self.source)? {
                if !roots.contains(&root) {
                    roots.push(root);
                }
            }
        }
        self.held(&roots)?;
        self.push(&table.attic, &roots)
    }

    fn report(&self, environment: &Environment) -> Answer<Running> {
        let table = Table::read(environment, &self.source)?;
        Err(self.no_marker(environment, &table))
    }
}
