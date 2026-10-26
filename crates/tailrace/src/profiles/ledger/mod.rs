//! The ledger seam's profiles (B.8), the git ref (C.13) and the store
//! (C.14), and the binding of the one the host's `[ledger]` names.

pub mod git_ref;
pub mod store;

use std::path::{Path, PathBuf};

use serde::Deserialize;
use tailrace_seams::{Answer, Keeper, Seam, Source};

use crate::bindings::Table;
use crate::profiles::run::{Tool, GIT_REPOSITORY_ENV};
use git_ref::GitRef;
use store::InStore;

/// What `[ledger] git_ref` is given: the remote holding the refs, where
/// one does (C.13).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GitRefOptions {
    remote: Option<String>,
}

/// What `[ledger] store` is given: nothing, the store being `[store]`'s.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoreOptions {}

/// The ledger `[ledger]` names (92, C.13, C.14) for the repository at
/// `repository`: `git_ref`, with its optional remote, the repository
/// holding the refs where it names none; or `store`, the chain held in
/// the store `store` binds, written or given, through a binding of its
/// own that never makes it, and kept under the repository's common git
/// directory (61, 62). Unreadable naming any other key, or an option its
/// profile does not read.
pub fn bind(table: &Table, repository: &Path, store: &Table) -> Answer<Box<dyn Keeper>> {
    match table.key() {
        "git_ref" => {
            let options: GitRefOptions = table.options()?;
            Ok(Box::new(GitRef::new(repository, options.remote)))
        }
        "store" => {
            let StoreOptions {} = table.options()?;
            let kept = common_dir(repository)?.join("tailrace").join("ledger");
            let store = crate::profiles::store::bind(store, false)?;
            Ok(Box::new(InStore::new(store, kept)))
        }
        _ => Err(table.unknown(&["git_ref", "store"])),
    }
}

/// The ledger a host binds where its file names no `[ledger]` (118,
/// C.13): the git ref in the repository at `repository`, read and
/// written with no remote.
pub fn given(repository: &Path) -> Box<dyn Keeper> {
    Box::new(GitRef::new(repository, None))
}

/// The common git directory of the repository at `repository`, which
/// every worktree of it on the host shares and no push carries (62).
fn common_dir(repository: &Path) -> Answer<PathBuf> {
    let git = Tool::new("git", Source::new(Seam::Keeper, "store"));
    let ran = git.output(
        None,
        &[
            "-C",
            &repository.display().to_string(),
            "rev-parse",
            "--path-format=absolute",
            "--git-common-dir",
        ],
        GIT_REPOSITORY_ENV,
        &[0],
    )?;
    Ok(PathBuf::from(
        String::from_utf8_lossy(&ran.stdout).trim().to_string(),
    ))
}
