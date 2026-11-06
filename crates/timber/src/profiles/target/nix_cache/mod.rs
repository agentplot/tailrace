//! The Nix cache target (C.12's Nix binary cache row), bound as
//! `nix_cache`: a version's closures pushed to a Nix binary cache. It is
//! bound and converges nothing yet: every operation answers absent naming
//! itself.

use timber_seams::{
    Answer, Digest, Environment, Head, Not, Running, Seam, Source, Store, Target, Verified,
};

/// The profile name an environment's `target` table names this target by.
pub const PROFILE: &str = "nix_cache";

/// The Nix cache target, reading through a store binding of its own,
/// opened and never made.
pub struct NixCache {
    #[expect(dead_code, reason = "apply reads the version's index through it")]
    store: Box<dyn Store>,
    source: Source,
}

impl NixCache {
    /// The target reading through `store`.
    pub fn new(store: Box<dyn Store>) -> NixCache {
        NixCache {
            store,
            source: Source::new(Seam::Target, PROFILE),
        }
    }

    /// What an operation of this target answers until it is built.
    fn not_built(&self, operation: &str) -> Not {
        Not::absent(&self.source, format!("{operation} of the {PROFILE} target"))
    }
}

impl Target for NixCache {
    fn desired(&self, _: &Environment, _: Head<'_>) -> Answer<Digest> {
        Err(self.not_built("desired"))
    }

    fn current(&self, _: &Environment) -> Answer<Digest> {
        Err(self.not_built("current"))
    }

    fn apply(&self, _: &Environment, _: &Verified) -> Answer<()> {
        Err(self.not_built("apply"))
    }

    fn report(&self, _: &Environment) -> Answer<Running> {
        Err(self.not_built("report"))
    }
}
