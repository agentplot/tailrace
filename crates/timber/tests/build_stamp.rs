//! The revision the binary stamps (statement 24, C.11): a build runs its
//! script again when a file the script watches is written or missing, so
//! every file a commit writes to move HEAD is one it watches, and no build
//! of a commit stamps the commit before it. Needs git alone, so it is in
//! the fast layer.

#[path = "../build/revision.rs"]
mod revision;
mod scratch;

use std::fs;
use std::path::PathBuf;

use scratch::{Repo, Scratch, MAIN};

/// Each file the build watches in `repo`, and what it holds, `None` where
/// it is missing.
fn watched(repo: &Repo) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    revision::watched(&repo.path)
        .into_iter()
        .map(|path| {
            let held = fs::read(&path).ok();
            (path, held)
        })
        .collect()
}

/// Whether the build runs its script again after `before`: a file it
/// watched is missing, or holds something else now.
fn runs_again(before: &[(PathBuf, Option<Vec<u8>>)]) -> bool {
    before
        .iter()
        .any(|(path, held)| held.is_none() || fs::read(path).ok() != *held)
}

/// A commit in `repo` runs the build's script again, which stamps it.
fn a_commit_runs_the_script_again(repo: &Repo, message: &str) {
    let before = watched(repo);
    let commit = repo.commit(message);
    assert!(
        runs_again(&before),
        "the commit wrote none of the files the build watches: {before:?}"
    );
    assert_eq!(revision::head(&repo.path), Some(commit));
}

#[test]
fn a_commit_on_a_branch_runs_the_build_script_again() {
    let scratch = Scratch::new();
    a_commit_runs_the_script_again(&scratch.repo, "feat: the next commit");
}

#[test]
fn a_commit_on_a_linked_worktrees_branch_runs_the_build_script_again() {
    let scratch = Scratch::new();
    let path = scratch.path().join("unit");
    scratch.repo.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "unit/the-next",
        path.to_str().unwrap(),
        MAIN,
    ]);
    a_commit_runs_the_script_again(&Repo { path }, "feat: the unit's commit");
}

#[test]
fn a_branch_held_only_packed_runs_the_build_script_every_time() {
    let scratch = Scratch::new();
    scratch.repo.git(&["pack-refs", "--all"]);
    let before = watched(&scratch.repo);
    assert!(
        before.iter().any(|(_, held)| held.is_none()),
        "the build watches no missing file: {before:?}"
    );
    a_commit_runs_the_script_again(&scratch.repo, "feat: the next commit");
}
