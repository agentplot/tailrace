//! The landing seam (B.3) bound to a plain git branch (C.6), for a
//! repository that runs no landing gate: a revision is landed when the
//! branch's history holds it, and no landing records verdicts (77, 78;
//! design D8).
//!
//! Every operation begins by reading the bound remote's branches, or the
//! repository's common git directory where no remote is bound, so a
//! repository or a remote that cannot be read answers every operation
//! unreadable with git's words (58, 59).

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use tailrace_seams::{
    Answer, Branch, IsLanded, Landing, Not, Read, Revision, Seam, Source, Verdicts,
};

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

    /// The opening read of every operation: where a remote is bound, the
    /// remote's listing of its branches, each by the commit it names;
    /// with none, the repository's common git directory read, and no
    /// branches.
    fn open(&self) -> Answer<BTreeMap<String, String>> {
        let Some(remote) = &self.remote else {
            self.git(&["rev-parse", "--git-common-dir"], &[0])?;
            return Ok(BTreeMap::new());
        };
        let ran = self.git(&["ls-remote", remote, "refs/heads/*"], &[0])?;
        Ok(String::from_utf8_lossy(&ran.stdout)
            .lines()
            .filter_map(|line| line.split_once('\t'))
            .map(|(object, name)| (name.to_string(), object.to_string()))
            .collect())
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

    /// The commit at the head of `branch`: the remote's, as `listed`
    /// names it and fetched, where a remote is bound, and the
    /// repository's own otherwise; `None` for a branch that does not
    /// exist.
    fn head(&self, branch: &Branch, listed: &BTreeMap<String, String>) -> Answer<Option<String>> {
        let name = format!("refs/heads/{branch}");
        let Some(remote) = &self.remote else {
            return self.commit(&name);
        };
        let Some(head) = listed.get(&name).cloned() else {
            return Ok(None);
        };
        self.git(
            &[
                "fetch",
                "--quiet",
                "--no-tags",
                "--no-write-fetch-head",
                "--refmap=",
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
    fn landed(&self, branch: &Branch, revision: &Revision) -> Answer<IsLanded> {
        let landed = |landed| IsLanded {
            landed,
            read: Read::Source,
        };
        let listed = self.open()?;
        let Some(head) = self.head(branch, &listed)? else {
            return Ok(landed(false));
        };
        let Some(commit) = self.commit(revision.as_str())? else {
            return Ok(landed(false));
        };
        let ran = self.git(&["merge-base", "--is-ancestor", &commit, &head], &[0, 1])?;
        Ok(landed(ran.code == Some(0)))
    }

    /// 78: no landing records verdicts on a plain branch, so every
    /// revision the repository holds has an empty set, neither absent
    /// nor unreadable; one it does not hold is absent (C.6).
    fn verdicts_of(&self, revision: &Revision) -> Answer<Verdicts> {
        self.open()?;
        match self.commit(revision.as_str())? {
            Some(_) => Ok(Verdicts {
                verdicts: vec![],
                read: Read::Source,
            }),
            None => Err(Not::absent(self.source(), format!("revision {revision}"))),
        }
    }
}
