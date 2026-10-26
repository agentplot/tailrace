//! The host's copy of what a landing profile reads from its remote:
//! tailrace's own refs under `refs/tailrace/landing/`, each the remote's
//! ref name with its `refs/` dropped, remade at every fetch so its
//! reflog's one entry says when it was fetched (60, 77, C.5, C.6; design
//! D2). The fetch writes the copy and nothing else: no remote-tracking
//! ref, no `FETCH_HEAD` and no branch. The copy is the repository's own
//! refs, readable with git and no tailrace process, and rebuilt by the
//! next read that reaches the remote (62, 63).

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::Path;

use tailrace_seams::Answer;

use crate::profiles::run::{Ran, Tool, GIT_REPOSITORY_ENV};

/// Where the copy is kept.
const COPY: &str = "refs/tailrace/landing/";

/// The copy of the remote's ref `name`.
pub(super) fn of(name: &str) -> String {
    format!("{COPY}{}", name.strip_prefix("refs/").unwrap_or(name))
}

/// The copy kept in the repository at `repository`, written and read
/// with `git`.
pub(super) struct Refs<'a> {
    pub git: &'a Tool,
    pub repository: &'a Path,
}

impl Refs<'_> {
    /// git run in the repository with every ref update logged, so a
    /// copy made has a reflog.
    fn args<'a, S: AsRef<OsStr>>(&'a self, args: &'a [S]) -> Vec<&'a OsStr> {
        let mut all = vec![
            OsStr::new("-C"),
            self.repository.as_os_str(),
            OsStr::new("-c"),
            OsStr::new("core.logAllRefUpdates=always"),
        ];
        all.extend(args.iter().map(AsRef::as_ref));
        all
    }

    fn git<S: AsRef<OsStr>>(&self, args: &[S], allowed: &[i32]) -> Answer<Ran> {
        self.git
            .output(None, &self.args(args), GIT_REPOSITORY_ENV, allowed)
    }

    /// One `update-ref --stdin` transaction of `lines`.
    fn transaction(&self, lines: &str) -> Answer<()> {
        let args = [
            "update-ref",
            "--create-reflog",
            "-m",
            "tailrace: fetched from the landing's remote",
            "--stdin",
        ];
        let all = self.args(&args);
        let ran = self
            .git
            .fed(None, &all, GIT_REPOSITORY_ENV, lines.as_bytes())?;
        if ran.code != Some(0) {
            return Err(self.git.failed(&all, &ran));
        }
        Ok(())
    }

    /// The copy of each of `old`, the remote's ref names, deleted in one
    /// transaction, and then the copy of each of `fetched` made at the
    /// object it names in a second, so each's reflog's one entry is now.
    /// A crash between the two leaves no copy, never a wrong one.
    pub fn remake(&self, old: &[&str], fetched: &BTreeMap<String, String>) -> Answer<()> {
        let deleted: String = old.iter().map(|n| format!("delete {}\n", of(n))).collect();
        self.transaction(&deleted)?;
        let made: String = fetched
            .iter()
            .map(|(name, object)| format!("create {} {object}\n", of(name)))
            .collect();
        self.transaction(&made)
    }

    /// Every ref the copy holds under the remote's `under`, by the
    /// remote's name, each by the object it names.
    pub fn held(&self, under: &str) -> Answer<BTreeMap<String, String>> {
        let ran = self.git(
            &[
                "for-each-ref",
                "--format=%(objectname) %(refname)",
                &of(under),
            ],
            &[0],
        )?;
        Ok(String::from_utf8_lossy(&ran.stdout)
            .lines()
            .filter_map(|line| line.split_once(' '))
            .filter_map(|(object, copy)| {
                let name = copy.strip_prefix(COPY)?;
                Some((format!("refs/{name}"), object.to_string()))
            })
            .collect())
    }

    /// When the copy of the remote's ref `name` was fetched, in seconds
    /// since the Unix epoch, from its reflog's one entry; `None` where
    /// the copy holds no such ref, or one with no reflog.
    pub fn when(&self, name: &str) -> Answer<Option<u64>> {
        let copy = of(name);
        let held = self.git(&["rev-parse", "--verify", "--quiet", &copy], &[0, 1])?;
        if held.code != Some(0) {
            return Ok(None);
        }
        let ran = self.git(
            &[
                "reflog",
                "show",
                "--date=unix",
                "--format=%gd",
                "-n",
                "1",
                &copy,
            ],
            &[0],
        )?;
        Ok(String::from_utf8_lossy(&ran.stdout)
            .rsplit_once("@{")
            .and_then(|(_, rest)| rest.trim().strip_suffix('}'))
            .and_then(|n| n.parse::<u64>().ok()))
    }
}
