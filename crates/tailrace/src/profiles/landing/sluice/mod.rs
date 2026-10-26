//! The landing seam (B.3) bound to sluice's records (C.5): landed from
//! the branch's log, and verdicts-of from the signed run records of the
//! combination that landed the revision, each envelope carried whole as
//! sluice wrote it (77, 78; ADR 0005). The landing reads what sluice
//! keeps and asks sluice to do nothing: it verifies no signature and
//! signs nothing (rules core.7, core.8).
//!
//! With a remote bound, every operation opens with the remote's listing
//! of sluice's refs, fetches each branch's log and each signer's records
//! it names in one fetch, and remakes the host's copy of them whole, so
//! the copy is the listing as of one fetch and says when (60, 77; design
//! D2, D3). The fetch writes the copy and nothing else. Where the remote
//! does not answer, the listing is read from the copy, nothing is
//! fetched, and both operations answer over it as over the remote,
//! saying when it was fetched and carrying git's words; where the copy
//! holds nothing an operation needs, the remote's failure is its answer
//! (58, 77; design D4). With no remote bound, the host's own records
//! are the landing.

mod dsse;
mod log;
mod records;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use tailrace_seams::{
    Answer, Branch, Envelope, IsLanded, Landing, Not, Read, Revision, Seam, Source, Verdict,
    Verdicts,
};

use super::copy::Refs;
use crate::profiles::run::{Ran, Tool, GIT_REPOSITORY_ENV};

/// Where sluice keeps its refs, and the copy keeps them.
const SLUICE: &str = "refs/sluice/";

/// Where sluice keeps each branch's log.
const STATE: &str = "refs/sluice/state/";

/// The refs of sluice's that the landing reads: each branch's log and
/// each signer's run records.
fn read_by_the_landing(name: &str) -> bool {
    name.starts_with(STATE) || name.starts_with(records::RUNS)
}

/// The opening read of an operation: the listing of sluice's refs the
/// landing reads, each by the object it names, and how it was read. As
/// last fetched, `failed` is the remote's answer, the answer of an
/// operation that needs what the copy does not hold.
struct Opened {
    listed: BTreeMap<String, String>,
    read: Read,
    failed: Option<Not>,
}

impl Opened {
    /// The remote's failure, where the listing was read from the copy;
    /// `None` where the remote answered.
    fn failed(&self) -> Option<Not> {
        self.failed.clone()
    }
}

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

    /// The host's copy of what is fetched from the remote.
    fn copy(&self) -> Refs<'_> {
        Refs {
            git: &self.git,
            repository: &self.repository,
        }
    }

    /// The opening read of every operation. With no remote bound, the
    /// repository's common directory read, and no refs. With one, the
    /// remote's listing of sluice's refs, each the landing reads fetched
    /// in one fetch, `branch`'s head among them where its log is listed,
    /// and the copy remade whole from the listing; where the listing or
    /// the fetch does not answer, the listing as the copy holds it, and
    /// nothing fetched (design D2 to D4).
    fn open(&self, branch: Option<&Branch>) -> Answer<Opened> {
        let Some(remote) = &self.remote else {
            self.common()?;
            return Ok(Opened {
                listed: BTreeMap::new(),
                read: Read::Source,
                failed: None,
            });
        };
        let ran = match self.git(&["ls-remote", remote, &format!("{SLUICE}*")], &[0]) {
            Ok(ran) => ran,
            Err(failed) => return self.as_fetched(failed),
        };
        let listed: BTreeMap<String, String> = String::from_utf8_lossy(&ran.stdout)
            .lines()
            .filter_map(|line| line.split_once('\t'))
            .filter(|(_, name)| read_by_the_landing(name))
            .map(|(object, name)| (name.to_string(), object.to_string()))
            .collect();
        let mut fetched: Vec<String> = listed.keys().cloned().collect();
        if let Some(branch) = branch {
            if listed.contains_key(&format!("{STATE}{branch}")) {
                fetched.push(format!("refs/heads/{branch}"));
            }
        }
        if !fetched.is_empty() {
            let refs: Vec<&str> = fetched.iter().map(String::as_str).collect();
            if let Err(failed) = self.fetch(remote, &refs) {
                return self.as_fetched(failed);
            }
        }
        let copy = self.copy();
        let old: Vec<String> = copy.held(SLUICE)?.into_keys().collect();
        copy.remake(&old.iter().map(String::as_str).collect::<Vec<_>>(), &listed)?;
        Ok(Opened {
            listed,
            read: Read::Source,
            failed: None,
        })
    }

    /// The opening read as the copy was last fetched, for `failed`, the
    /// remote's unreadable answer, at the oldest time among the copy's
    /// refs, all of one fetch; `failed` itself where it is not unreadable
    /// or the copy holds nothing.
    fn as_fetched(&self, failed: Not) -> Answer<Opened> {
        let said = match &failed {
            Not::Unreadable(u) => u.said.clone().unwrap_or_default(),
            _ => return Err(failed),
        };
        let copy = self.copy();
        let listed = copy.held(SLUICE)?;
        let mut oldest: Option<u64> = None;
        for name in listed.keys() {
            let Some(at) = copy.when(name)? else {
                return Err(failed);
            };
            oldest = Some(oldest.map_or(at, |o| o.min(at)));
        }
        let Some(at) = oldest else {
            return Err(failed);
        };
        Ok(Opened {
            listed,
            read: Read::Kept { at, said },
            failed: Some(failed),
        })
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
    /// no ref written but the copy, not the remote-tracking one a
    /// configured refspec would move nor `FETCH_HEAD`: each is read after
    /// by the object the remote's listing named (design D2).
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
    fn landed(&self, branch: &Branch, revision: &Revision) -> Answer<IsLanded> {
        let opened = self.open(Some(branch))?;
        let landed = self.landed_over(&opened, branch, revision)?;
        Ok(IsLanded {
            landed,
            read: opened.read,
        })
    }

    /// 78: the verdicts of the signed run records, under every signer's
    /// ref on the remote, of the combination the revision's
    /// `Sluice-Tested` trailer names (design D6). With no remote bound,
    /// sluice signed nothing, and a revision no trailer names was landed
    /// on no verdict: both answer an empty set (sluice ADR 0032).
    fn verdicts_of(&self, revision: &Revision) -> Answer<Verdicts> {
        let opened = self.open(None)?;
        let verdicts = self.verdicts_over(&opened, revision)?;
        Ok(Verdicts {
            verdicts,
            read: opened.read,
        })
    }
}

impl Sluice {
    fn landed_over(&self, opened: &Opened, branch: &Branch, revision: &Revision) -> Answer<bool> {
        let log = self.log(opened, branch)?;
        let done = log::landings(&log.bytes)
            .map_err(|why| Not::unreadable(self.source(), &log.at, Some(why)))?;
        if done.is_empty() {
            return Ok(false);
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

    fn verdicts_over(&self, opened: &Opened, revision: &Revision) -> Answer<Vec<Verdict>> {
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
        let runs: Vec<(String, String)> = opened
            .listed
            .iter()
            .filter_map(|(name, object)| {
                name.strip_prefix(records::RUNS)
                    .map(|signer| (signer.to_string(), object.clone()))
            })
            .collect();
        if runs.is_empty() {
            return Ok(vec![]);
        }
        let under = self.held(remote, &runs, Some(&format!("{combination}/")))?;
        records::verdicts(remote, combination, &under, || {
            self.held(remote, &runs, None)
        })
    }
}
