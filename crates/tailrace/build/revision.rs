//! The revision a checkout is at, and the files whose change moves it:
//! HEAD, and the branch HEAD names, whose file a commit on the branch
//! rewrites while HEAD's stays as it was. A branch git holds only packed
//! has no file of its own, and a build that watches a missing file runs
//! its script every time, so it stamps the right revision there too.
//! The build script and its test both include this file.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The commit `dir`'s checkout is at.
pub fn head(dir: &Path) -> Option<String> {
    git(dir, &["rev-parse", "HEAD"])
}

/// The files a commit, a checkout or a reset in `dir`'s checkout writes:
/// HEAD, and the branch's file where HEAD names a branch.
pub fn watched(dir: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = git_path(dir, "HEAD").into_iter().collect();
    if let Some(branch) = git(dir, &["symbolic-ref", "-q", "HEAD"]) {
        paths.extend(git_path(dir, &branch));
    }
    paths
}

fn git_path(dir: &Path, name: &str) -> Option<PathBuf> {
    git(
        dir,
        &["rev-parse", "--path-format=absolute", "--git-path", name],
    )
    .map(PathBuf::from)
}

fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}
