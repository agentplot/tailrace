//! The landing seam (B.3) bound to a plain git branch (C.6), for a
//! repository that runs no landing gate: a revision is landed when the
//! branch's history holds it, and no landing records verdicts (77, 78;
//! design D8).
//!
//! Every operation begins by reading the repository's common git
//! directory, so a repository that cannot be read answers every
//! operation unreadable with git's words (58, 59).

use std::ffi::OsStr;
use std::path::PathBuf;

use tailrace_seams::{Answer, Branch, Landing, Not, Revision, Seam, Source, Verdict};

use crate::profiles::run::{Ran, Tool, GIT_REPOSITORY_ENV};

/// The landing seam bound to a plain branch of the repository at a path,
/// read from the remote where one is bound.
#[derive(Clone, Debug)]
pub struct PlainBranch {
    repository: PathBuf,
    remote: Option<String>,
    git: Tool,
}

fn source() -> Source {
    Source::new(Seam::Landing, "branch")
}

impl PlainBranch {
    /// The branches of the repository at `repository`, as `remote` holds
    /// them where one is bound and as the repository does otherwise, read
    /// with `git` as found on `PATH`.
    pub fn new(repository: impl Into<PathBuf>, remote: Option<String>) -> PlainBranch {
        PlainBranch {
            repository: repository.into(),
            remote,
            git: Tool::new("git", source()),
        }
    }

    /// git run as `program`.
    pub fn git_program(mut self, program: impl Into<PathBuf>) -> PlainBranch {
        self.git = Tool::new(program, source());
        self
    }

    /// The landing seam and this profile, as every answer names them.
    pub fn source(&self) -> &Source {
        self.git.source()
    }

    /// git run in the repository with `args`, answering when it exits
    /// with one of `allowed`.
    fn git<S: AsRef<OsStr>>(&self, args: &[S], allowed: &[i32]) -> Answer<Ran> {
        let mut all = vec![OsStr::new("-C"), self.repository.as_os_str()];
        all.extend(args.iter().map(AsRef::as_ref));
        self.git.output(None, &all, GIT_REPOSITORY_ENV, allowed)
    }

    /// The opening read of every operation.
    fn open(&self) -> Answer<()> {
        self.git(&["rev-parse", "--git-common-dir"], &[0])
            .map(|_| ())
    }

    /// The commit `name` names in the repository, `None` when it names
    /// none.
    fn commit(&self, name: &str) -> Answer<Option<String>> {
        let ran = self.git(
            &[
                "rev-parse",
                "--verify",
                "--quiet",
                &format!("{name}^{{commit}}"),
            ],
            &[0, 1],
        )?;
        Ok((ran.code == Some(0)).then(|| String::from_utf8_lossy(&ran.stdout).trim().to_string()))
    }

    /// The commit at the head of `branch`: the remote's, fetched, where a
    /// remote is bound, and the repository's own otherwise; `None` for a
    /// branch that does not exist.
    fn head(&self, branch: &Branch) -> Answer<Option<String>> {
        let name = format!("refs/heads/{branch}");
        let Some(remote) = &self.remote else {
            return self.commit(&name);
        };
        let listed = self.git(&["ls-remote", remote, &name], &[0])?;
        let Some(head) = String::from_utf8_lossy(&listed.stdout)
            .lines()
            .find_map(|line| {
                line.split_once('\t')
                    .filter(|(_, listed)| *listed == name)
                    .map(|(object, _)| object.to_string())
            })
        else {
            return Ok(None);
        };
        self.git(
            &[
                "fetch",
                "--quiet",
                "--no-tags",
                "--no-write-fetch-head",
                remote,
                &name,
            ],
            &[0],
        )?;
        Ok(Some(head))
    }
}

impl Landing for PlainBranch {
    /// 77: landed when the revision is the branch's head or one of its
    /// ancestors; a branch that does not exist, or a revision the
    /// repository does not hold, has landed nothing.
    fn landed(&self, branch: &Branch, revision: &Revision) -> Answer<bool> {
        self.open()?;
        let Some(head) = self.head(branch)? else {
            return Ok(false);
        };
        let Some(commit) = self.commit(revision.as_str())? else {
            return Ok(false);
        };
        let ran = self.git(&["merge-base", "--is-ancestor", &commit, &head], &[0, 1])?;
        Ok(ran.code == Some(0))
    }

    /// 78: no landing records verdicts on a plain branch, so every
    /// revision the repository holds has an empty set, neither absent
    /// nor unreadable; one it does not hold is absent (C.6).
    fn verdicts_of(&self, revision: &Revision) -> Answer<Vec<Verdict>> {
        self.open()?;
        match self.commit(revision.as_str())? {
            Some(_) => Ok(vec![]),
            None => Err(Not::absent(self.source(), format!("revision {revision}"))),
        }
    }
}
