//! The projection for Cargo (C.11; ADR 0006): a crate packed by Cargo's
//! own packager, and a local registry rendered from the store, which a
//! consumer's Cargo reads through source replacement.
//!
//! It packs no cache entries: a pack with no version and a render of
//! cache entries answer absent naming them, and read and write nothing
//! (84, 85). It reaches the store only through the store seam it is
//! handed (rule core.2), and runs the cargo it is bound to.

mod crate_file;
#[cfg(test)]
mod fixture;
mod pack;

use std::path::{Path, PathBuf};

use tailrace_seams::{
    no_cache_entries, Answer, ArtifactMeta, BuildId, Contents, Ecosystem, EcosystemName, Layout,
    Not, Output, Packed, Projection, Rendered, Seam, Serving, Source, Store, Version, Why,
};

use super::not_built;
use crate::profiles::run::Tool;

/// The ecosystem this projection packs in, as the bindings bind it.
pub const ECOSYSTEM: &str = "cargo";

/// The Cargo projection of one checkout, reading crates from the store it
/// is handed and running `cargo`.
// The store is read by render, which its own task builds.
#[allow(dead_code)]
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

    fn render(&self, contents: Contents<'_>, _: &str) -> Answer<Rendered> {
        let source = CargoProjection::source();
        let Contents::Artifacts(_) = contents else {
            return Err(no_cache_entries(&source));
        };
        Err(not_built("render", ECOSYSTEM, &source))
    }

    fn serve(&self, _: &Layout) -> Answer<Serving> {
        Err(not_built("serve", ECOSYSTEM, &CargoProjection::source()))
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
}
