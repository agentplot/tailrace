//! The build seam's profiles (B.2), and the binding of the repository's.

pub mod moon;

use std::path::Path;

use tailrace_seams::Build;

use moon::Moon;

/// The build the repository at `repository` binds (92, C.3): moon, the
/// one build profile, each output's task named in its policy file.
pub fn bind(repository: &Path) -> Box<dyn Build> {
    Box::new(Moon::new(repository))
}

/// The name of the build profile [`bind`] binds, which a store's cache
/// repository is named for, so no binding names the tool (C.1, rule
/// core.1; design D9).
pub fn name() -> &'static str {
    "moon"
}
