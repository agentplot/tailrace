//! The landing seam (B.3) bound to sluice's records (C.5): landed from
//! the branch's log, and verdicts-of from the signed run records of the
//! combination that landed the revision, each envelope carried whole as
//! sluice wrote it (77, 78; ADR 0005). The landing reads what sluice
//! keeps and asks sluice to do nothing: it verifies no signature and
//! signs nothing (rules core.7, core.8).

// Their readers are the landing's from the task that writes it.
#[allow(dead_code)]
mod dsse;
#[allow(dead_code)]
mod log;
#[allow(dead_code)]
mod records;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use tailrace_seams::{Answer, Seam, Source};

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

// The log's reader reads through these until the landing's operations
// do.
#[allow(dead_code)]
impl Sluice {
    /// git run in the repository with `args`, answering when it exits
    /// with one of `allowed`.
    fn git<S: AsRef<OsStr>>(&self, args: &[S], allowed: &[i32]) -> Answer<Ran> {
        let mut all = vec![OsStr::new("-C"), self.repository.as_os_str()];
        all.extend(args.iter().map(AsRef::as_ref));
        self.git.output(None, &all, GIT_REPOSITORY_ENV, allowed)
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
    /// no ref written: each is read after by the object the remote's
    /// listing named (design D2).
    fn fetch(&self, remote: &str, refs: &[&str]) -> Answer<()> {
        let mut args = vec![
            "fetch",
            "--quiet",
            "--no-tags",
            "--no-write-fetch-head",
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
