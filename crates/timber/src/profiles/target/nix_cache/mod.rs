//! The Nix cache target (C.12's Nix binary cache row), bound as
//! `nix_cache`: a version's closures pushed to a Nix binary cache with
//! attic, from the host that holds them.

mod table;
#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::io;
use std::path::PathBuf;

use timber_seams::{
    Answer, Artifact, Digest, Environment, Head, Held, IndexMeta, Manifest, Not, Running, Seam,
    Source, Store, Target, Verified, Why,
};

use super::nix_closure::{self, ClosureFile, StorePath};
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

    /// The index `version` names, read through the store by its digest,
    /// naming the version verified (42, rule core.4).
    fn index(&self, version: &Verified) -> Answer<IndexMeta> {
        let digest = version.index();
        let meta = match self.store.read(digest) {
            Ok(Held::Manifest(Manifest::Index(meta))) => meta,
            Ok(_) => {
                return Err(self.refused(format!(
                    "{digest}, the index of version {} as verified, holds no index",
                    version.version()
                )))
            }
            Err(Not::Absent(_)) => {
                return Err(Not::unreadable(
                    &self.source,
                    format!("the index of version {}", version.version()),
                    Some(format!("the store holds no {digest}")),
                ))
            }
            Err(not) => return Err(not),
        };
        if meta.version() != version.version() {
            return Err(self.refused(format!(
                "the index {digest} names version {}, and the version verified is {}",
                meta.version(),
                version.version()
            )));
        }
        Ok(meta)
    }

    /// The bytes of the closure file `artifact`, read through the store
    /// by the one layer its manifest names and checked against that
    /// layer's digest (42, rule core.5).
    fn bytes(&self, artifact: &Artifact) -> Answer<Vec<u8>> {
        let (digest, file) = (&artifact.digest, ClosureFile(artifact));
        let unheld = |what: &Digest| {
            Not::unreadable(
                &self.source,
                file.to_string(),
                Some(format!("the store holds no {what}")),
            )
        };
        let layers = match self.store.read(digest) {
            Ok(Held::Manifest(Manifest::Artifact { meta, layers })) if meta == artifact.meta => {
                layers
            }
            Ok(Held::Manifest(Manifest::Artifact { meta, .. })) => {
                return Err(self.refused(format!(
                    "the store's manifest at {digest} names {} {} {}, and {file}",
                    meta.ecosystem(),
                    meta.name(),
                    meta.version()
                )))
            }
            Ok(_) => {
                return Err(
                    self.refused(format!("{digest} holds no artifact's manifest, and {file}"))
                )
            }
            Err(Not::Absent(_)) => return Err(unheld(digest)),
            Err(not) => return Err(not),
        };
        let [layer] = layers.as_slice() else {
            return Err(self.refused(format!(
                "{file} names {} layers, and a closure file is one",
                layers.len()
            )));
        };
        let bytes = match self.store.read(layer) {
            Ok(Held::Bytes(bytes)) => bytes,
            Ok(Held::Manifest(_)) => {
                return Err(self.refused(format!(
                    "the layer {layer} of {file} holds a manifest and not a file"
                )))
            }
            Err(Not::Absent(_)) => return Err(unheld(layer)),
            Err(not) => return Err(not),
        };
        let answered = Digest::of(&bytes);
        if &answered != layer {
            return Err(Not::refused(
                &self.source,
                Why::OtherBytes {
                    expected: layer.clone(),
                    answered,
                },
                None,
            ));
        }
        Ok(bytes)
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
        let index = self.index(version)?;
        let store_dir = nix_closure::store_dir();
        let mut roots: Vec<StorePath> = Vec::new();
        for output in &table.closures {
            let artifact = nix_closure::artifact_of(&index, output, &self.source)?;
            let bytes = self.bytes(artifact)?;
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
