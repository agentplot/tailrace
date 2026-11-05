//! The projection for Cargo (C.11; ADR 0006): a crate packed alone by
//! Cargo's own packager without its own lockfile, its siblings at the
//! component's version and another component's crates at the requirement
//! their manifests write; a checkout's lockfile pins and its manifests'
//! ranges, answered with or without a lockfile; what a held crate
//! requires of the family, and which versions a range admits, newest
//! first, by Cargo's rules; and a sparse registry rendered from the store
//! and served from the host's one process, in the one shape of request,
//! response and miss every projection serves in (84 to 87, 115, 137,
//! 140; design D3, D5, D8, D9).
//!
//! | module | what |
//! |---|---|
//! | `pack` | the component's version written into the package and its siblings, a crate of another component checked against the store's versions at its requirement, `cargo package` without the crate's lockfile, every manifest written back (84, 87, 110, 123) |
//! | `render` | a sparse registry holding exactly the crates given, each checked before anything is written (85, 48) |
//! | `serve` | Cargo's sparse protocol from the layout: its configuration, its index files, and each crate, one it lacks asked of the core (86, 115, 99) |
//! | `index` | a crate's index line, from its normalized manifest |
//! | `crate_file` | a crate's normalized manifest, read from its bytes |
//! | `pinned` | the versions a checkout's `Cargo.lock` pins from a registry, and the ranges its manifests declare in `timber` for names it pins none of, with or without a lockfile (137) |
//! | `requires` | what a held crate requires of the family, from its normalized manifest (140) |
//! | `satisfying` | the versions within a range, newest first, by Cargo's rules (140, 23) |
//!
//! It packs no cache entries: a pack with no version and a render of
//! cache entries answer absent naming them, and read and write nothing
//! (84, 85). It reaches the store only through the store seam it is
//! handed (rule core.2), and runs the cargo it is bound to.

mod crate_file;
#[cfg(test)]
mod fixture;
mod index;
mod pack;
mod pinned;
mod render;
mod requires;
mod satisfying;
mod serve;

use std::path::{Path, PathBuf};

use timber_seams::{
    no_cache_entries, Answer, Artifact, ArtifactMeta, BuildId, Contents, Ecosystem, EcosystemName,
    Layout, Miss, Not, Output, PackAt, Packed, Pin, Projection, Range, Rendered, Request,
    Requirement, Response, Seam, Source, Store, Version, Why,
};

use crate::profiles::run::Tool;

/// The ecosystem this projection packs in, as the bindings bind it.
pub const ECOSYSTEM: &str = "cargo";

/// The Cargo projection of one checkout, reading crates from the store it
/// is handed and running `cargo`.
pub struct CargoProjection {
    root: PathBuf,
    store: Box<dyn Store>,
    cargo: Tool,
}

impl CargoProjection {
    /// The projection of the checkout at `root`, reading crates from
    /// `store` and running `cargo` as found on `PATH`.
    pub fn new(root: impl Into<PathBuf>, store: Box<dyn Store>) -> CargoProjection {
        CargoProjection {
            root: root.into(),
            store,
            cargo: Tool::new("cargo", CargoProjection::source()),
        }
    }

    /// cargo run as `program`.
    pub fn cargo_program(mut self, program: impl Into<PathBuf>) -> CargoProjection {
        self.cargo = Tool::new(program, CargoProjection::source());
        self
    }

    /// The projection seam and this profile, as every answer names them.
    pub fn source() -> Source {
        Source::new(Seam::Projection, ECOSYSTEM)
    }

    /// `at` as a path: itself where absolute, else under the checkout.
    fn under_root(&self, at: &str) -> PathBuf {
        let at = Path::new(at);
        if at.is_absolute() {
            at.to_path_buf()
        } else {
            self.root.join(at)
        }
    }
}

impl Projection for CargoProjection {
    /// 84, 87, 110: the crate `cargo package` makes of the package the
    /// output's crate file names, at the component's version with its
    /// siblings at it and another component's crates at the requirements
    /// the manifests write, named by the package's name and the version,
    /// packing the same again (design D1 to D4).
    fn pack(&self, output: &Output, _: &BuildId, version: Option<&PackAt<'_>>) -> Answer<Packed> {
        let source = CargoProjection::source();
        let Some(at) = version else {
            return Err(no_cache_entries(&source));
        };
        let found = self.crate_of(output)?;
        let bytes = self.packed(&found, at)?;
        let meta = ArtifactMeta::new(
            Ecosystem::new(ECOSYSTEM),
            EcosystemName::new(found.name),
            at.version.clone(),
        )
        .map_err(|bad| Not::refused(&source, Why::Said, Some(bad.to_string())))?;
        Ok(Packed {
            bytes,
            meta: Some(meta),
            same_again: true,
        })
    }

    /// 85, 48: a sparse registry at `at` holding exactly `contents`'
    /// crates, each read through the store and checked before anything is
    /// written, and naming no address (design D8).
    fn render(&self, contents: Contents<'_>, at: &str) -> Answer<Rendered> {
        let Contents::Artifacts(artifacts) = contents else {
            return Err(no_cache_entries(&CargoProjection::source()));
        };
        self.render_crates(artifacts, at)
    }

    /// 86, 115, 99: `request`, one of Cargo's own, answered from the
    /// layout in Cargo's sparse registry protocol: the configuration
    /// naming downloads under the request's base, an index file from the
    /// layout alone, and a crate's bytes, one the layout lacks asked of
    /// the core once through `miss` (design D9).
    fn serve(&self, layout: &Layout, request: &Request, miss: Miss<'_>) -> Answer<Response> {
        self.serve_request(layout, request, miss)
    }

    /// 137: the versions the checkout's root `Cargo.lock` pins from a
    /// registry, as it spells them, and the ranges its manifests declare
    /// in `timber` for names it pins no version of, with or without a
    /// lockfile (design D5).
    fn pinned(&self, checkout: &str) -> Answer<Vec<Pin>> {
        self.pins_and_ranges(checkout)
    }

    /// 140: each family crate `held`'s own manifest requires, with the
    /// requirement it writes (design D8).
    fn requires(&self, held: &Artifact) -> Answer<Vec<Requirement>> {
        self.required(held)
    }

    /// 140, 23: those of `versions` within `range`, newest first, by
    /// Cargo's own rules (design D8).
    fn satisfying(&self, range: &Range, versions: &[Version]) -> Answer<Vec<Version>> {
        satisfying::satisfying(range, versions)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;
    use timber_fakes::FakeStore;
    use timber_seams::{CacheEntry, Digest, OutputName};

    use super::*;

    #[test]
    fn a_pack_with_no_version_and_a_render_of_cache_entries_are_absent_and_touch_nothing() {
        let dir = TempDir::new().unwrap();
        let store = FakeStore::new();
        let gone = dir.path().join("cargo");
        let projection =
            CargoProjection::new(dir.path(), Box::new(store.clone())).cargo_program(&gone);
        let absent = Not::absent(&CargoProjection::source(), "cache entries");
        let output = Output {
            name: OutputName::new("fam"),
            files: vec!["fam/Cargo.toml".into()],
        };
        assert_eq!(
            projection.pack(&output, &BuildId::new("b7e1"), None),
            Err(absent.clone())
        );
        let entries = [CacheEntry {
            build: BuildId::new("b7e1"),
            digest: Digest::of(b"archive"),
        }];
        assert_eq!(
            projection.render(Contents::CacheEntries(&entries), "layouts/cargo"),
            Err(absent)
        );
        assert_eq!(store.calls(), [], "the store was read");
        assert_eq!(
            fs::read_dir(dir.path()).unwrap().count(),
            0,
            "the checkout was written"
        );
    }
}
