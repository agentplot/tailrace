//! A revision's checkout: a detached worktree of exactly the revision,
//! under the repository's common git directory, holding nothing of the
//! publisher's checkout (74, rule core.11; design D4).

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use tailrace_seams::Answer;

use super::Moon;

/// Where the checkout of `commit` is kept: under the common directory, so
/// every worktree of the repository finds the one checkout of a revision.
fn path(common: &Path, commit: &str) -> PathBuf {
    common.join("tailrace").join("builds").join(commit)
}

// Ensure and report read through these from the tasks that write them.
#[allow(dead_code)]
impl Moon {
    /// The checkout of `commit`, made by `git worktree add --detach` the
    /// first time and reused after.
    pub(super) fn checkout(&self, common: &Path, commit: &str) -> Answer<PathBuf> {
        let at = path(common, commit);
        if self.made(common, commit).is_none() {
            self.git(
                &[
                    OsStr::new("worktree"),
                    OsStr::new("add"),
                    OsStr::new("--quiet"),
                    OsStr::new("--detach"),
                    OsStr::new("--force"),
                    at.as_os_str(),
                    OsStr::new(commit),
                ],
                &[0],
            )?;
        }
        Ok(at)
    }

    /// The checkout of `commit`, where one has been made.
    pub(super) fn made(&self, common: &Path, commit: &str) -> Option<PathBuf> {
        let at = path(common, commit);
        at.join(".git").is_file().then_some(at)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::super::tests::Repo;

    #[test]
    fn the_checkout_holds_the_revisions_tree_and_is_reused_for_it() {
        let repo = Repo::new();
        repo.write("web/index.html", "first");
        let first = repo.commit("first");
        repo.write("web/index.html", "second");
        repo.write("web/added.html", "added");
        let second = repo.commit("second");
        let moon = repo.moon();
        let common = moon.common().unwrap();
        assert_eq!(moon.made(&common, &first), None);

        let at = moon.checkout(&common, &first).unwrap();
        assert!(at.starts_with(common.join("tailrace/builds")), "{at:?}");
        assert_eq!(
            fs::read_to_string(at.join("web/index.html")).unwrap(),
            "first"
        );
        assert!(!at.join("web/added.html").exists());
        assert_eq!(
            repo.git(&["-C", at.to_str().unwrap(), "rev-parse", "HEAD"]),
            first
        );
        assert_eq!(moon.made(&common, &first), Some(at.clone()));

        fs::write(at.join("web/dist.txt"), "a build's output").unwrap();
        assert_eq!(moon.checkout(&common, &first), Ok(at.clone()));
        assert!(
            at.join("web/dist.txt").exists(),
            "the checkout was made again"
        );

        let other = moon.checkout(&common, &second).unwrap();
        assert_ne!(other, at);
        assert_eq!(
            fs::read_to_string(other.join("web/index.html")).unwrap(),
            "second"
        );
    }

    #[test]
    fn a_checkouts_own_uncommitted_file_is_not_in_it() {
        let repo = Repo::new();
        repo.write("web/index.html", "landed");
        let landed = repo.commit("landed");
        repo.write("web/index.html", "edited, not committed");
        repo.write("web/untracked.html", "never committed");
        let moon = repo.moon();
        let common = moon.common().unwrap();
        let at = moon.checkout(&common, &landed).unwrap();
        assert_eq!(
            fs::read_to_string(at.join("web/index.html")).unwrap(),
            "landed"
        );
        assert!(!at.join("web/untracked.html").exists());
    }
}
