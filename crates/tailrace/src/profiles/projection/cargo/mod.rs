//! The projection for Cargo (C.11; ADR 0006): a crate packed by Cargo's
//! own packager, and a local registry rendered from the store, which a
//! consumer's Cargo reads through source replacement.
//!
//! | module | what |
//! |---|---|
//! | `pack` | the version written into the manifest that holds it, `cargo package`, the manifest written back (84, 87) |
//! | `render` | a local registry holding exactly the crates given, each checked before anything is written (85, 48) |
//! | `index` | a crate's index line, from its normalized manifest |
//! | `crate_file` | a crate's normalized manifest, read from its bytes |
//! | `pinned` | the versions a checkout's `Cargo.lock` pins from a registry, and the ranges its manifests declare in `tailrace` for names it pins none of (137) |
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

use std::path::{Path, PathBuf};

use tailrace_seams::{
    no_cache_entries, Answer, Artifact, ArtifactMeta, BuildId, Contents, Ecosystem, EcosystemName,
    Layout, Not, Output, Packed, Pin, Projection, Range, Rendered, Requirement, Seam, Serving,
    Source, Store, Version, Why,
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
    /// 84, 87: the crate `cargo package` makes of the output's package at
    /// `version`, named by the package's name and the version, packing
    /// the same again (design D3).
    fn pack(&self, output: &Output, _: &BuildId, version: Option<&Version>) -> Answer<Packed> {
        let source = CargoProjection::source();
        let Some(version) = version else {
            return Err(no_cache_entries(&source));
        };
        let manifest = self.package_manifest(output)?;
        let (name, bytes) = self.packed(&manifest, version)?;
        let meta = ArtifactMeta::new(
            Ecosystem::new(ECOSYSTEM),
            EcosystemName::new(name),
            version.clone(),
        )
        .map_err(|bad| Not::refused(&source, Why::Said, Some(bad.to_string())))?;
        Ok(Packed {
            bytes,
            meta: Some(meta),
            same_again: true,
        })
    }

    /// 85, 48: a local registry at `at` holding exactly `contents`' crates,
    /// each read through the store and checked before anything is written.
    fn render(&self, contents: Contents<'_>, at: &str) -> Answer<Rendered> {
        let Contents::Artifacts(artifacts) = contents else {
            return Err(no_cache_entries(&CargoProjection::source()));
        };
        self.render_crates(artifacts, at)
    }

    /// 86: absent, since Cargo reads a local registry itself through
    /// source replacement and needs no server.
    fn serve(&self, _: &Layout) -> Answer<Serving> {
        Err(Not::absent(
            &CargoProjection::source(),
            "serve of the cargo projection, whose local registry Cargo reads itself through \
             source replacement",
        ))
    }

    /// 137: the versions the checkout's root `Cargo.lock` pins from a
    /// registry, as it spells them, and the ranges its manifests declare
    /// in `tailrace` for names it pins no version of, with or without a
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

    use tailrace_fakes::FakeStore;
    use tailrace_seams::{CacheEntry, Digest, OutputName};
    use tempfile::TempDir;

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

    #[test]
    fn serve_is_absent_naming_serve_of_the_cargo_projection() {
        let dir = TempDir::new().unwrap();
        let projection = CargoProjection::new(dir.path(), Box::new(FakeStore::new()));
        match projection.serve(&Layout {
            at: "layouts/cargo".into(),
        }) {
            Err(Not::Absent(a)) => {
                assert_eq!(a.source, CargoProjection::source());
                assert!(a.what.starts_with("serve of the cargo projection"), "{a:?}");
            }
            other => panic!("answered {other:?}"),
        }
    }
}
