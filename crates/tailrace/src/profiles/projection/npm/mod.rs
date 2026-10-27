//! The projection for npm (C.11; design D6 to D9, D13, D14): a package
//! packed by npm's own packager, a lockfile's pins and a manifest's
//! family ranges read as npm spells them, what a held package requires of
//! the family, a registry read layout of tarballs and packuments, and
//! npm's registry protocol served from that layout (84 to 87, 115, 137,
//! 140). It names npm and nothing else, and reaches the store only
//! through the store seam it is handed (rules core.1, core.2).

use std::path::PathBuf;

use tailrace_seams::{
    Answer, Artifact, BuildId, Contents, Layout, Miss, Output, Packed, Pinned, Projection, Range,
    Rendered, Request, Requirement, Response, Seam, Source, Store, Version,
};

use super::not_built;

/// The ecosystem this projection packs in, as the bindings bind it.
pub const ECOSYSTEM: &str = "npm";

/// The npm projection of one checkout: the checkout whose `.npmrc` names
/// the family, the store it reads tarballs through, and the host's
/// serving process's address, which a family scope's registry begins
/// with.
pub struct NpmProjection {
    // Read by requires and render, which the projection's later
    // operations write.
    #[allow(dead_code)]
    root: PathBuf,
    #[allow(dead_code)]
    store: Box<dyn Store>,
    #[allow(dead_code)]
    origin: String,
}

impl NpmProjection {
    /// The projection of the checkout at `root`, reading tarballs through
    /// `store`, its family the scopes the checkout's `.npmrc` sends to
    /// `origin`, the host's serving process (design D13).
    pub fn new(
        root: impl Into<PathBuf>,
        store: Box<dyn Store>,
        origin: impl Into<String>,
    ) -> NpmProjection {
        NpmProjection {
            root: root.into(),
            store,
            origin: origin.into(),
        }
    }

    /// The projection seam and this profile, as every answer names them.
    pub fn source() -> Source {
        Source::new(Seam::Projection, ECOSYSTEM)
    }
}

impl Projection for NpmProjection {
    fn pack(&self, _: &Output, _: &BuildId, _: Option<&Version>) -> Answer<Packed> {
        Err(not_built("pack", ECOSYSTEM, &NpmProjection::source()))
    }

    fn render(&self, _: Contents<'_>, _: &str) -> Answer<Rendered> {
        Err(not_built("render", ECOSYSTEM, &NpmProjection::source()))
    }

    fn serve(&self, _: &Layout, _: &Request, _: Miss<'_>) -> Answer<Response> {
        Err(not_built("serve", ECOSYSTEM, &NpmProjection::source()))
    }

    fn pinned(&self, _: &str) -> Answer<Vec<Pinned>> {
        Err(not_built("pinned", ECOSYSTEM, &NpmProjection::source()))
    }

    fn requires(&self, _: &Artifact) -> Answer<Vec<Requirement>> {
        Err(not_built("requires", ECOSYSTEM, &NpmProjection::source()))
    }

    fn satisfying(&self, _: &Range, _: &[Version]) -> Answer<Vec<Version>> {
        Err(not_built("satisfying", ECOSYSTEM, &NpmProjection::source()))
    }
}
