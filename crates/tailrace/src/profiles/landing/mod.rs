//! The landing seam's profiles (B.3), and the binding of the one the
//! host's `[landing]` names.

pub mod branch;
pub mod sluice;

use std::path::Path;

use serde::Deserialize;
use tailrace_seams::{Answer, Landing};

use crate::bindings::Table;
use branch::PlainBranch;
use sluice::Sluice;

/// What `[landing]`'s profile is given: the remote that holds the
/// landing's records, or the branches, where one is bound.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Options {
    remote: Option<String>,
}

/// The landing `[landing]` names (92, C.5, C.6) over the repository at
/// `repository`: `sluice` or `branch`, each with its optional remote;
/// unreadable naming any other key, or an option neither reads.
pub fn bind(table: &Table, repository: &Path) -> Answer<Box<dyn Landing>> {
    match table.key() {
        "sluice" => {
            let options: Options = table.options()?;
            Ok(Box::new(Sluice::new(repository, options.remote)))
        }
        "branch" => {
            let options: Options = table.options()?;
            Ok(Box::new(PlainBranch::new(repository, options.remote)))
        }
        _ => Err(table.unknown(&["sluice", "branch"])),
    }
}
