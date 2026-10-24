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
