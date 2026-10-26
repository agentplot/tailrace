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

/// The file a repository set up for sluice holds in its tree.
pub const SLUICE_FILE: &str = ".config/sluice.toml";

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

/// The remote `[landing]`'s profile is given, none where it names none;
/// unreadable as [`bind`] answers it.
pub fn remote(table: &Table) -> Answer<Option<String>> {
    match table.key() {
        "sluice" | "branch" => Ok(table.options::<Options>()?.remote),
        _ => Err(table.unknown(&["sluice", "branch"])),
    }
}

/// 118: the landing of a host with no bindings file, over the repository
/// at `repository` and `remote`: sluice where the repository's tree holds
/// [`SLUICE_FILE`], so it is set up for it (C.5), and the plain branch
/// otherwise (C.6).
pub fn unfiled(repository: &Path, remote: Option<String>) -> Box<dyn Landing> {
    if repository.join(SLUICE_FILE).is_file() {
        Box::new(Sluice::new(repository, remote))
    } else {
        Box::new(PlainBranch::new(repository, remote))
    }
}
