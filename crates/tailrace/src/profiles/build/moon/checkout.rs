//! A revision's checkout: a detached worktree of exactly the revision,
//! under the repository's common git directory, holding nothing of the
//! publisher's checkout but moon's outputs directory, which every
//! checkout on a host shares (74, C.3, rule core.11; design D4, D5).

use std::ffi::OsStr;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use tailrace_seams::Answer;

use super::report::OUTPUTS;
use super::Moon;

/// Where the checkout of `commit` is kept: under the common directory, so
/// every worktree of the repository finds the one checkout of a revision.
fn path(common: &Path, commit: &str) -> PathBuf {
    common.join("tailrace").join("builds").join(commit)
}

impl Moon {
    /// The checkout of `commit`, made by `git worktree add --detach` the
    /// first time and reused after, its outputs directory shared with the
    /// bound repository's at every asking.
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
        self.share_outputs(&at)?;
        Ok(at)
    }

    /// `checkout`'s moon outputs directory made a link to the bound
    /// repository's, resolved through any link that one is and made where
    /// it is missing, so the archives any checkout on the host holds are
    /// the ones moon replays here (C.3). A directory of the checkout's own
    /// there is removed first: what it holds is cache, which may go at any
    /// time (54).
    fn share_outputs(&self, checkout: &Path) -> Answer<()> {
        let bound = self.repository.join(OUTPUTS);
        let target = match (fs::read_link(&bound), bound.parent()) {
            (Ok(to), Some(dir)) => dir.join(to),
            _ => bound,
        };
        let target = fs::create_dir_all(&target)
            .and_then(|()| fs::canonicalize(&target))
            .map_err(|e| self.unreadable_file(&target, e.to_string()))?;
        let own = checkout.join(OUTPUTS);
        linked(&own, &target).map_err(|e| self.unreadable_file(&own, e.to_string()))
    }

    /// The checkout of `commit`, where one has been made.
    pub(super) fn made(&self, common: &Path, commit: &str) -> Option<PathBuf> {
        let at = path(common, commit);
        at.join(".git").is_file().then_some(at)
    }
}

/// `own` a link to `target`: left as it is where it already leads there,
/// else whatever is there removed and the link made.
fn linked(own: &Path, target: &Path) -> io::Result<()> {
    match fs::symlink_metadata(own) {
        Ok(meta) if meta.file_type().is_symlink() => {
            if fs::canonicalize(own).is_ok_and(|at| at == target) {
                return Ok(());
            }
            fs::remove_file(own)?;
        }
        Ok(meta) if meta.is_dir() => fs::remove_dir_all(own)?,
        Ok(_) => fs::remove_file(own)?,
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    if let Some(dir) = own.parent() {
        fs::create_dir_all(dir)?;
    }
    std::os::unix::fs::symlink(target, own)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::super::tests::Repo;
    use super::OUTPUTS;

    #[test]
    fn the_checkouts_outputs_are_the_bound_repositorys_made_where_missing() {
        let repo = Repo::new();
        repo.write("web/index.html", "landed");
        let landed = repo.commit("landed");
        let moon = repo.moon();
        let common = moon.common().unwrap();
        let bound = repo.path().join(OUTPUTS);
        assert!(!bound.exists());

        let at = moon.checkout(&common, &landed).unwrap();
        let own = at.join(OUTPUTS);
        assert!(fs::symlink_metadata(&own).unwrap().file_type().is_symlink());
        assert!(
            bound.is_dir(),
            "the bound repository's outputs were not made"
        );
        fs::write(bound.join("b7e1.tar.gz"), "an archive").unwrap();
        assert_eq!(
            fs::read_to_string(own.join("b7e1.tar.gz")).unwrap(),
            "an archive"
        );
    }

    #[test]
    fn a_checkouts_own_outputs_directory_is_replaced_by_the_link() {
        let repo = Repo::new();
        repo.write("web/index.html", "landed");
        let landed = repo.commit("landed");
        let moon = repo.moon();
        let common = moon.common().unwrap();
        let at = moon.checkout(&common, &landed).unwrap();
        let own = at.join(OUTPUTS);
        fs::remove_file(&own).unwrap();
        fs::create_dir_all(&own).unwrap();
        fs::write(own.join("old.tar.gz"), "left by an earlier version").unwrap();

        moon.checkout(&common, &landed).unwrap();
        assert!(fs::symlink_metadata(&own).unwrap().file_type().is_symlink());
        assert!(!own.join("old.tar.gz").exists());
    }

    #[test]
    fn a_bound_outputs_link_is_followed_to_where_it_points() {
        let repo = Repo::new();
        repo.write("web/index.html", "landed");
        let landed = repo.commit("landed");
        let moon = repo.moon();
        let common = moon.common().unwrap();
        let shared = common.join("moon-outputs");
        let bound = repo.path().join(OUTPUTS);
        fs::create_dir_all(bound.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&shared, &bound).unwrap();

        let at = moon.checkout(&common, &landed).unwrap();
        assert!(shared.is_dir(), "the link's target was not made");
        assert_eq!(
            fs::read_link(at.join(OUTPUTS)).unwrap(),
            shared.canonicalize().unwrap()
        );
    }

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
