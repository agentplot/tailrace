//! The landing seam's profiles (B.3), and the binding of the one the
//! host's `[landing]` names.

pub mod branch;
pub mod sluice;

use std::path::Path;

use serde::Deserialize;
use tailrace_seams::{Answer, Landing, Not, Seam, Source};

use crate::bindings::Table;
use crate::profiles::run::{Tool, GIT_REPOSITORY_ENV};
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

/// The remote a checkout is cloned from, by git's own default name.
pub const ORIGIN: &str = "origin";

/// The landing a host binds where its file at `file` names no
/// `[landing]` (118), over the repository at `repository`: sluice where
/// the repository is set up for it, its `HEAD` commit holding
/// [`SLUICE_FILE`], read with git from that commit and never from the
/// working tree (C.5, rule core.22); the plain branch otherwise, a `HEAD`
/// naming no commit yet among them (C.6); either over [`given_remote`]'s
/// remote. Unreadable with git's words where git cannot open the
/// repository (58, 59), and as [`given_remote`] answers.
pub fn given(file: &Path, repository: &Path) -> Answer<Box<dyn Landing>> {
    let sluice = set_up_for_sluice(repository)?;
    let remote = remote_of(file, repository, &source(sluice))?;
    Ok(if sluice {
        Box::new(Sluice::new(repository, remote))
    } else {
        Box::new(PlainBranch::new(repository, remote))
    })
}

/// The remote of the landing a host binds where its file at `file` names
/// no `[landing]`, in the checkout at `repository` (118; design D5):
/// [`ORIGIN`] where the checkout has it, its one remote where it has one
/// of another name, and none where it has none, the repository's own
/// refs then read. Unreadable naming the checkout, every remote in git's
/// order and `[landing]` in the file where it has several and none is
/// [`ORIGIN`] (58), and with git's words where git cannot list them.
pub fn given_remote(file: &Path, repository: &Path) -> Answer<Option<String>> {
    remote_of(file, repository, &source(set_up_for_sluice(repository)?))
}

/// Whether the repository at `repository` is set up for sluice: its
/// `HEAD` commit holds [`SLUICE_FILE`].
fn set_up_for_sluice(repository: &Path) -> Answer<bool> {
    let git = Tool::new("git", source(true));
    let object = format!("HEAD:{SLUICE_FILE}");
    let held = git.output(
        None,
        &[
            "-C",
            &repository.display().to_string(),
            "rev-parse",
            "--verify",
            "--quiet",
            &object,
        ],
        GIT_REPOSITORY_ENV,
        &[0, 1],
    )?;
    Ok(held.code == Some(0))
}

/// The source of the given landing's answers: sluice's where `sluice`,
/// the plain branch's otherwise.
fn source(sluice: bool) -> Source {
    Source::new(Seam::Landing, if sluice { "sluice" } else { "branch" })
}

/// [`given_remote`]'s answer, as the landing of `source` answers it.
fn remote_of(file: &Path, repository: &Path, source: &Source) -> Answer<Option<String>> {
    let git = Tool::new("git", source.clone());
    let listed = git.output(
        None,
        &["-C", &repository.display().to_string(), "remote"],
        GIT_REPOSITORY_ENV,
        &[0],
    )?;
    let remotes: Vec<String> = String::from_utf8_lossy(&listed.stdout)
        .lines()
        .map(str::trim)
        .filter(|remote| !remote.is_empty())
        .map(str::to_string)
        .collect();
    if remotes.iter().any(|remote| remote == ORIGIN) {
        return Ok(Some(ORIGIN.to_string()));
    }
    match remotes.as_slice() {
        [] => Ok(None),
        [one] => Ok(Some(one.clone())),
        several => Err(Not::unreadable(
            source,
            format!(
                "the landing remote of the checkout at {}",
                repository.display()
            ),
            Some(format!(
                "the checkout has the remotes {}, none of them {ORIGIN}; name one in \
                 [landing] remote of {}",
                several.join(", "),
                file.display()
            )),
        )),
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
