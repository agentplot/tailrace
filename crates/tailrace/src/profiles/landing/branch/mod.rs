//! The landing seam (B.3) bound to a plain git branch (C.6), for a
//! repository that runs no landing gate: a revision is landed when the
//! branch's history holds it, and no landing records verdicts (77, 78;
//! design D8).
//!
//! Where a remote is bound, every operation begins by reading the
//! remote's branches, and each branch fetched from it is kept in the
//! host's copy, remade at the fetch so it says when (60, 77; design D2,
//! D3). Where the remote does not answer, landed answers from the copy
//! of the branch and verdicts-of from the repository, each saying when
//! the remote was last fetched and carrying git's words; a branch the
//! copy does not hold is unreadable with those words, never not landed
//! (58, 77; design D4). With no remote bound, the repository's own
//! branches are the landing, and a repository that cannot be read
//! answers every operation unreadable with git's words (58, 59).

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use tailrace_seams::{
    Answer, Branch, IsLanded, Landing, Not, Read, Revision, Seam, Source, Verdicts,
};

use super::copy::Refs;
use crate::profiles::run::{Ran, Tool, GIT_REPOSITORY_ENV};

/// Where the remote's branches are, and the copy keeps them.
const HEADS: &str = "refs/heads/";

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

    /// The host's copy of what is fetched from the remote.
    fn copy(&self) -> Refs<'_> {
        Refs {
            git: &self.git,
            repository: &self.repository,
        }
    }

    /// The remote's listing of its branches, each by the commit it names.
    fn listed(&self, remote: &str) -> Answer<BTreeMap<String, String>> {
        let ran = self.git(&["ls-remote", remote, &format!("{HEADS}*")], &[0])?;
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

    /// The branch `name` fetched from `remote` with no ref written, not
    /// even the remote-tracking one a configured refspec would move, nor
    /// `FETCH_HEAD`.
    fn fetch(&self, remote: &str, name: &str) -> Answer<()> {
        self.git(
            &[
                "fetch",
                "--quiet",
                "--no-tags",
                "--no-write-fetch-head",
                "--refmap=",
                remote,
                name,
            ],
            &[0],
        )
        .map(|_| ())
    }

    /// 77: whether `head` holds `revision`, as its head or one of its
    /// ancestors; a revision the repository does not hold is not landed.
    fn holds(&self, head: &str, revision: &Revision) -> Answer<bool> {
        let Some(commit) = self.commit(revision.as_str())? else {
            return Ok(false);
        };
        let ran = self.git(&["merge-base", "--is-ancestor", &commit, head], &[0, 1])?;
        Ok(ran.code == Some(0))
    }

    /// The remote not answering, for `failed`: `failed`'s own words, and
    /// `failed` itself where it is not unreadable, which is no remote not
    /// answering (design D4).
    fn said(failed: &Not) -> Option<String> {
        match failed {
            Not::Unreadable(u) => Some(u.said.clone().unwrap_or_default()),
            _ => None,
        }
    }

    /// 77: landed as the copy of `branch` was last fetched, for `failed`,
    /// the remote's answer; `failed` itself where the copy holds no such
    /// branch.
    fn landed_as_fetched(
        &self,
        branch: &Branch,
        revision: &Revision,
        failed: Not,
    ) -> Answer<IsLanded> {
        let Some(said) = Self::said(&failed) else {
            return Err(failed);
        };
        let name = format!("{HEADS}{branch}");
        let copy = self.copy();
        let (Some(head), Some(at)) = (copy.held(&name)?.remove(&name), copy.when(&name)?) else {
            return Err(failed);
        };
        Ok(IsLanded {
            landed: self.holds(&head, revision)?,
            read: Read::Kept { at, said },
        })
    }

    /// 78: the empty set of a revision the repository holds, as the
    /// remote was last fetched, the newest of the copy's times, for
    /// `failed`; `failed` itself where the copy holds no branch.
    fn verdicts_as_fetched(&self, revision: &Revision, failed: Not) -> Answer<Verdicts> {
        let Some(said) = Self::said(&failed) else {
            return Err(failed);
        };
        let copy = self.copy();
        let mut newest = None;
        for name in copy.held(HEADS)?.keys() {
            newest = newest.max(copy.when(name)?);
        }
        let Some(at) = newest else {
            return Err(failed);
        };
        self.no_verdicts(revision, Read::Kept { at, said })
    }

    /// 78: no landing records verdicts on a plain branch, so every
    /// revision the repository holds has an empty set, neither absent
    /// nor unreadable; one it does not hold is absent (C.6).
    fn no_verdicts(&self, revision: &Revision, read: Read) -> Answer<Verdicts> {
        match self.commit(revision.as_str())? {
            Some(_) => Ok(Verdicts {
                verdicts: vec![],
                read,
            }),
            None => Err(Not::absent(self.source(), format!("revision {revision}"))),
        }
    }
}

impl Landing for PlainBranch {
    /// 77: landed when the revision is the branch's head or one of its
    /// ancestors; a branch that does not exist, or a revision the
    /// repository does not hold, has landed nothing. With a remote
    /// bound, the branch is the remote's, fetched into the copy, and as
    /// the copy last fetched it where the remote does not answer.
    fn landed(&self, branch: &Branch, revision: &Revision) -> Answer<IsLanded> {
        let Some(remote) = &self.remote else {
            self.git(&["rev-parse", "--git-common-dir"], &[0])?;
            let landed = match self.commit(&format!("{HEADS}{branch}"))? {
                Some(head) => self.holds(&head, revision)?,
                None => false,
            };
            return Ok(IsLanded {
                landed,
                read: Read::Source,
            });
        };
        let name = format!("{HEADS}{branch}");
        let listed = match self.listed(remote) {
            Ok(listed) => listed,
            Err(failed) => return self.landed_as_fetched(branch, revision, failed),
        };
        let source = |landed| IsLanded {
            landed,
            read: Read::Source,
        };
        // A branch the remote does not list leaves the copy, so the copy
        // never holds a branch the remote no longer did (design D3).
        let Some(head) = listed.get(&name) else {
            self.copy().remake(&[&name], &BTreeMap::new())?;
            return Ok(source(false));
        };
        if let Err(failed) = self.fetch(remote, &name) {
            return self.landed_as_fetched(branch, revision, failed);
        }
        self.copy()
            .remake(&[&name], &BTreeMap::from([(name.clone(), head.clone())]))?;
        Ok(source(self.holds(head, revision)?))
    }

    /// 78: an empty set for every revision the repository holds, and
    /// absent for one it does not (C.6), read once the remote's listing
    /// answers; where it does not, as the copy was last fetched.
    fn verdicts_of(&self, revision: &Revision) -> Answer<Verdicts> {
        match &self.remote {
            None => {
                self.git(&["rev-parse", "--git-common-dir"], &[0])?;
            }
            Some(remote) => {
                if let Err(failed) = self.listed(remote) {
                    return self.verdicts_as_fetched(revision, failed);
                }
            }
        }
        self.no_verdicts(revision, Read::Source)
    }
}
