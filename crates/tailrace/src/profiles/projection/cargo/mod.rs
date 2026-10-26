//! The projection for Cargo (C.11; ADR 0006): a crate packed by Cargo's
//! own packager, and a local registry rendered from the store, which a
//! consumer's Cargo reads through source replacement.
//!
//! It reaches the store only through the store seam it is handed (rule
//! core.2), and runs the cargo it is bound to.

use std::path::PathBuf;

use tailrace_seams::{
    Answer, BuildId, Contents, Layout, Output, Packed, Projection, Rendered, Seam, Serving, Source,
    Store, Version,
};

use super::not_built;
use crate::profiles::run::Tool;

/// The ecosystem this projection packs in, as the bindings bind it.
pub const ECOSYSTEM: &str = "cargo";

/// The Cargo projection of one checkout, reading crates from the store it
/// is handed and running `cargo`.
// The checkout, the store and cargo are read by pack and render, which
// their own tasks build.
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
}

impl Projection for CargoProjection {
    fn pack(&self, _: &Output, _: &BuildId, _: Option<&Version>) -> Answer<Packed> {
        Err(not_built("pack", ECOSYSTEM, &CargoProjection::source()))
    }

    fn render(&self, _: Contents<'_>, _: &str) -> Answer<Rendered> {
        Err(not_built("render", ECOSYSTEM, &CargoProjection::source()))
    }

    fn serve(&self, _: &Layout) -> Answer<Serving> {
        Err(not_built("serve", ECOSYSTEM, &CargoProjection::source()))
    }
}
