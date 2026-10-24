//! The landing seam (B.3) bound to sluice's records (C.5): landed from
//! the branch's log, and verdicts-of from the signed run records of the
//! combination that landed the revision, each envelope carried whole as
//! sluice wrote it (77, 78; ADR 0005). The landing reads what sluice
//! keeps and asks sluice to do nothing: it verifies no signature and
//! signs nothing (rules core.7, core.8).

mod dsse;
mod log;
mod records;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use tailrace_seams::{Answer, Branch, Envelope, Landing, Not, Revision, Seam, Source, Verdict};

use crate::profiles::run::{Ran, Tool, GIT_REPOSITORY_ENV};

/// The landing seam bound to sluice over the repository at a path, and
/// the remote that holds sluice's records of its branches, where one is
/// bound.
#[derive(Clone, Debug)]
pub struct Sluice {
    repository: PathBuf,
    remote: Option<String>,
    git: Tool,
    /// Each run record read, by the signer whose ref holds it and its
    /// blob, kept for the life of the handle (design D6).
    records: RefCell<BTreeMap<(String, String), records::Record>>,
}

fn source() -> Source {
    Source::new(Seam::Landing, "sluice")
}

/// `envelope` in DSSE's own form, as sluice writes a run record: for a
/// verdict's envelope, the bytes of the record it was found in (12).
pub fn as_written(envelope: &Envelope) -> Vec<u8> {
    dsse::write(envelope)
}

impl Sluice {
    /// sluice's records of the repository at `repository`, on `remote`
    /// where one is bound and in the host's own files otherwise, read
    /// with `git` as found on `PATH`.
    pub fn new(repository: impl Into<PathBuf>, remote: Option<String>) -> Sluice {
        Sluice {
            repository: repository.into(),
            remote,
            git: Tool::new("git", source()),
            records: RefCell::default(),
        }
    }

    /// git run as `program`.
    pub fn git_program(mut self, program: impl Into<PathBuf>) -> Sluice {
        self.git = Tool::new(program, source());
        self
    }

    /// The landing seam and this profile, as every answer names them.
    pub fn source(&self) -> &Source {
        self.git.source()
    }
}

impl Sluice {
    /// git run in the repository with `args`, answering when it exits
    /// with one of `allowed`.
    fn git<S: AsRef<OsStr>>(&self, args: &[S], allowed: &[i32]) -> Answer<Ran> {
        let mut all = vec![OsStr::new("-C"), self.repository.as_os_str()];
        all.extend(args.iter().map(AsRef::as_ref));
        self.git.output(None, &all, GIT_REPOSITORY_ENV, allowed)
    }

    /// The opening read of every operation: with a remote bound, the
    /// remote's listing of sluice's refs, each by the object it names;
    /// with none, the repository's common directory read, and no refs
    /// (design D2).
    fn open(&self) -> Answer<BTreeMap<String, String>> {
        let Some(remote) = &self.remote else {
            self.common()?;
            return Ok(BTreeMap::new());
        };
        let ran = self.git(&["ls-remote", remote, "refs/sluice/*"], &[0])?;
        Ok(String::from_utf8_lossy(&ran.stdout)
            .lines()
            .filter_map(|line| line.split_once('\t'))
            .map(|(object, name)| (name.to_string(), object.to_string()))
            .collect())
    }

    /// The commit `revision` names in the repository, `None` when it
    /// holds none.
    fn commit(&self, revision: &Revision) -> Answer<Option<String>> {
        let ran = self.git(
            &[
                "rev-parse",
                "--verify",
                "--quiet",
                &format!("{revision}^{{commit}}"),
            ],
            &[0, 1],
        )?;
        Ok((ran.code == Some(0)).then(|| String::from_utf8_lossy(&ran.stdout).trim().to_string()))
    }

    /// The repository's common git directory, where sluice keeps a
    /// branch's records on one host.
    fn common(&self) -> Answer<PathBuf> {
        let ran = self.git(
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
            &[0],
        )?;
        Ok(PathBuf::from(
            String::from_utf8_lossy(&ran.stdout).trim_end_matches('\n'),
        ))
    }

    /// `refs` fetched from `remote` into the repository's objects, with
    /// no ref written, not even the remote-tracking one a configured
    /// refspec would move: each is read after by the object the remote's
    /// listing named (design D2).
    fn fetch(&self, remote: &str, refs: &[&str]) -> Answer<()> {
        let mut args = vec![
            "fetch",
            "--quiet",
            "--no-tags",
            "--no-write-fetch-head",
            "--refmap=",
            remote,
        ];
        args.extend(refs);
        self.git(&args, &[0]).map(|_| ())
    }

    /// The file at `path` in the tree of `commit`, `None` when the tree
    /// holds none.
    fn file_at(&self, commit: &str, path: &str) -> Answer<Option<Vec<u8>>> {
        let listed = self.git(&["ls-tree", "-z", commit, "--", path], &[0])?;
        let Some(blob) = String::from_utf8_lossy(&listed.stdout)
            .split('\0')
            .find_map(|entry| entry.split_whitespace().nth(2).map(str::to_string))
        else {
            return Ok(None);
        };
        Ok(Some(self.git(&["cat-file", "blob", &blob], &[0])?.stdout))
    }
}

impl Landing for Sluice {
    /// 77: landed only when the branch's log records a landing done whose
    /// commits, the first-parent line from the revision it moved the
    /// branch to back to the one it moved it from, include the revision
    /// (design D5).
    fn landed(&self, branch: &Branch, revision: &Revision) -> Answer<bool> {
        let listed = self.open()?;
        let log = self.log(&listed, branch)?;
        let done = log::landings(&log.bytes)
            .map_err(|why| Not::unreadable(self.source(), &log.at, Some(why)))?;
        if done.is_empty() {
            return Ok(false);
        }
        let state = format!("refs/sluice/state/{branch}");
        if let (Some(remote), true) = (&self.remote, listed.contains_key(&state)) {
            self.fetch(remote, &[&format!("refs/heads/{branch}")])?;
        }
        let Some(commit) = self.commit(revision)? else {
            return Ok(false);
        };
        for landing in done {
            let range = format!("{}..{}", landing.from, landing.to);
            let ran = self.git(&["rev-list", "--first-parent", &range], &[0])?;
            if String::from_utf8_lossy(&ran.stdout)
                .lines()
                .any(|line| line == commit)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// 78: the verdicts of the signed run records, under every signer's
    /// ref on the remote, of the combination the revision's
    /// `Sluice-Tested` trailer names (design D6). With no remote bound,
    /// sluice signed nothing, and a revision no trailer names was landed
    /// on no verdict: both answer an empty set (sluice ADR 0032).
    fn verdicts_of(&self, revision: &Revision) -> Answer<Vec<Verdict>> {
        let listed = self.open()?;
        let Some(remote) = &self.remote else {
            return Ok(vec![]);
        };
        let Some(commit) = self.commit(revision)? else {
            return Err(Not::absent(self.source(), format!("revision {revision}")));
        };
        let ran = self.git(
            &[
                "log",
                "-1",
                "--format=%(trailers:key=Sluice-Tested,valueonly)",
                &commit,
            ],
            &[0],
        )?;
        let trailer = String::from_utf8_lossy(&ran.stdout);
        let Some(combination) = trailer.lines().map(str::trim).find(|l| !l.is_empty()) else {
            return Ok(vec![]);
        };
        let runs: Vec<(String, String)> = listed
            .iter()
            .filter_map(|(name, object)| {
                name.strip_prefix(records::RUNS)
                    .map(|signer| (signer.to_string(), object.clone()))
            })
            .collect();
        if runs.is_empty() {
            return Ok(vec![]);
        }
        let names: Vec<String> = runs
            .iter()
            .map(|(signer, _)| format!("{}{signer}", records::RUNS))
            .collect();
        self.fetch(
            remote,
            &names.iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
        let under = self.held(remote, &runs, Some(&format!("{combination}/")))?;
        records::verdicts(remote, combination, &under, || {
            self.held(remote, &runs, None)
        })
    }
}
