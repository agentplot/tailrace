//! The policy seam's profiles (B.5), and the binding of the repository's.

pub mod file;

use std::path::Path;

use tailrace_seams::Policy;

use file::PolicyFile;

/// The policy the repository at `repository` binds (C.10): its policy
/// file, read at the revision asked.
pub fn bind(repository: &Path) -> Box<dyn Policy> {
    Box::new(PolicyFile::new(repository))
}
