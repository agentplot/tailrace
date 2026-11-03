//! The host's copy of what a landing profile reads from its remote:
//! tailrace's own refs under `refs/tailrace/landing/`, each the remote's
//! ref name with its `refs/` dropped, remade at every fetch so its
//! reflog's newest entry says when it was fetched and none older is kept
//! (60, 77, C.5, C.6; design D2). A copy the remote still lists is never
//! deleted, so two reads of one checkout at once both answer. The fetch
//! writes the copy and nothing else: no remote-tracking ref, no
//! `FETCH_HEAD` and no branch. The copy is the repository's own refs,
//! readable with git and no tailrace process, and rebuilt by the next
//! read that reaches the remote (62, 63).

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use tailrace_seams::Answer;

use crate::profiles::run::{Ran, Tool, GIT_REPOSITORY_ENV};

/// Where the copy is kept.
const COPY: &str = "refs/tailrace/landing/";

/// The message of each entry the copy's reflogs hold.
const FETCHED: &str = "tailrace: fetched from the landing's remote";

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
    /// copy made has a reflog, and waiting seconds rather than git's
    /// tenth of one for a copy's lock another read of the checkout holds.
    fn args<'a, S: AsRef<OsStr>>(&'a self, args: &'a [S]) -> Vec<&'a OsStr> {
        let mut all = vec![
            OsStr::new("-C"),
            self.repository.as_os_str(),
            OsStr::new("-c"),
            OsStr::new("core.logAllRefUpdates=always"),
            OsStr::new("-c"),
            OsStr::new("core.filesRefLockTimeout=5000"),
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
        let args = ["update-ref", "--create-reflog", "-m", FETCHED, "--stdin"];
        let all = self.args(&args);
        let ran = self
            .git
            .fed(None, &all, GIT_REPOSITORY_ENV, lines.as_bytes())?;
        if ran.code != Some(0) {
            return Err(self.git.failed(&all, &ran));
        }
        Ok(())
    }

    /// The copy of what the remote's `under` names, one ref or, where it
    /// ends in `/`, every ref under it, remade to `fetched`, the remote's
    /// listing of them by the object each names. One transaction deletes
    /// each copy held and not listed and makes or moves each listed one,
    /// so the copy is the listing whole, and a crash leaves it as it was,
    /// never a wrong one. A listed copy is never deleted: another read of
    /// the checkout at once, as a serving process's two panes make, would
    /// find the directory it locks the copy in pruned with the last ref
    /// under it. Each listed copy's reflog then has an entry from now, one
    /// written where the copy did not move, since git logs no update that
    /// leaves a ref where it was, so a crash before it leaves the earlier
    /// fetch's time, never a later one; every entry from before this
    /// remake is then pruned. What is held is read after the remake's
    /// time is taken, so where another read moved a copy meanwhile its
    /// entry is kept.
    pub fn remake(&self, under: &str, fetched: &BTreeMap<String, String>) -> Answer<()> {
        let at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_secs());
        let held: BTreeMap<String, String> = self
            .held(under)?
            .into_iter()
            .filter(|(name, _)| under.ends_with('/') || name == under)
            .collect();
        let unmoved = |name: &String, object: &String| held.get(name) == Some(object);
        let deleted = held
            .keys()
            .filter(|name| !fetched.contains_key(*name))
            .map(|name| format!("delete {}\n", of(name)));
        let made = fetched
            .iter()
            .filter(|(name, object)| !unmoved(name, object))
            .map(|(name, object)| format!("update {} {object}\n", of(name)));
        let lines: String = deleted.chain(made).collect();
        if !lines.is_empty() {
            self.transaction(&lines)?;
        }
        for (name, object) in fetched
            .iter()
            .filter(|(name, object)| unmoved(name, object))
        {
            self.git(
                &["reflog", "write", &of(name), object, object, FETCHED],
                &[0],
            )?;
        }
        if fetched.is_empty() {
            return Ok(());
        }
        let mut expire = vec![
            "reflog".to_string(),
            "expire".to_string(),
            format!("--expire=@{at}"),
            format!("--expire-unreachable=@{at}"),
        ];
        expire.extend(fetched.keys().map(|name| of(name)));
        self.git(&expire, &[0])?;
        Ok(())
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
    /// since the Unix epoch, from its reflog's newest entry; `None` where
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
