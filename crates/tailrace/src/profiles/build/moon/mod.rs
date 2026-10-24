//! The build seam (B.2) bound to moon (C.3): the repository names, in
//! its policy file at the revision, the moon task that makes each
//! output; ensure is one moon run of those tasks in a checkout of exactly
//! that revision, and report reads moon's own record of the run, the
//! task's hash its build identity (74 to 76).
//!
//! The policy file names an output's task in a table of its own, which
//! the core's reader leaves to this profile (design D3):
//!
//! ```toml
//! [build.outputs.site]
//! task = "web:build"
//! ```
//!
//! Every operation begins by reading the repository's common git
//! directory, so a repository that cannot be read answers every
//! operation unreadable with git's words (58, 59).

mod checkout;

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use serde::Deserialize;
use tailrace_seams::{Answer, Not, OutputName, Revision, Seam, Source};

use crate::profiles::run::{Ran, Tool, GIT_REPOSITORY_ENV};

/// The policy file, read at the revision and never from a checkout (65,
/// rule core.22).
const POLICY_FILE: &str = ".config/tailrace.toml";

/// The build seam bound to moon over the repository at a path.
#[derive(Clone, Debug)]
pub struct Moon {
    repository: PathBuf,
    git: Tool,
    moon: Tool,
}

/// What this profile reads of the policy file: each output's table under
/// `[build.outputs]`, and its `task`.
#[derive(Default, Deserialize)]
struct PolicyFile {
    #[serde(default)]
    build: BuildTable,
}

#[derive(Default, Deserialize)]
struct BuildTable {
    #[serde(default)]
    outputs: BTreeMap<OutputName, OutputTable>,
}

#[derive(Deserialize)]
struct OutputTable {
    task: Option<String>,
}

fn source() -> Source {
    Source::new(Seam::Build, "moon")
}

impl Moon {
    /// moon over the repository at `repository`, running `moon` and
    /// `git` as found on `PATH`.
    pub fn new(repository: impl Into<PathBuf>) -> Moon {
        Moon {
            repository: repository.into(),
            git: Tool::new("git", source()),
            moon: Tool::new("moon", source()),
        }
    }

    /// moon run as `program`.
    pub fn moon_program(mut self, program: impl Into<PathBuf>) -> Moon {
        self.moon = Tool::new(program, source());
        self
    }

    /// git run as `program`.
    pub fn git_program(mut self, program: impl Into<PathBuf>) -> Moon {
        self.git = Tool::new(program, source());
        self
    }

    /// The build seam and this profile, as every answer names them.
    pub fn source(&self) -> &Source {
        self.git.source()
    }
}

// Ensure and report read through these from the tasks that write them.
#[allow(dead_code)]
impl Moon {
    /// git run in the repository with `args`, answering when it exits
    /// with one of `allowed`.
    fn git<S: AsRef<OsStr>>(&self, args: &[S], allowed: &[i32]) -> Answer<Ran> {
        let mut all = vec![OsStr::new("-C"), self.repository.as_os_str()];
        all.extend(args.iter().map(AsRef::as_ref));
        self.git.output(None, &all, GIT_REPOSITORY_ENV, allowed)
    }

    /// The opening read of every operation: the repository's common git
    /// directory, where the checkouts are kept.
    fn common(&self) -> Answer<PathBuf> {
        let ran = self.git(
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
            &[0],
        )?;
        Ok(PathBuf::from(
            String::from_utf8_lossy(&ran.stdout).trim_end_matches('\n'),
        ))
    }

    /// The commit `revision` names, resolved before anything is read at
    /// it (rule core.4); absent when the repository holds none.
    fn commit(&self, revision: &Revision) -> Answer<String> {
        let ran = self.git(
            &[
                "rev-parse",
                "--verify",
                "--quiet",
                &format!("{revision}^{{commit}}"),
            ],
            &[0, 1],
        )?;
        if ran.code != Some(0) {
            return Err(Not::absent(self.source(), format!("revision {revision}")));
        }
        Ok(String::from_utf8_lossy(&ran.stdout).trim().to_string())
    }

    /// The task the policy file at `commit` names for each output that
    /// names one (design D3). A file that is missing, or an output with
    /// no table or no `task`, names none.
    fn tasks(&self, commit: &str) -> Answer<BTreeMap<OutputName, String>> {
        let at = format!("{commit}:{POLICY_FILE}");
        let listed = self.git(&["ls-tree", "-z", commit, "--", POLICY_FILE], &[0])?;
        let Some(blob) = String::from_utf8_lossy(&listed.stdout)
            .split('\0')
            .find_map(|entry| entry.split_whitespace().nth(2).map(str::to_string))
        else {
            return Ok(BTreeMap::new());
        };
        let bytes = self.git(&["cat-file", "blob", &blob], &[0])?.stdout;
        let text = String::from_utf8(bytes)
            .map_err(|e| Not::unreadable(self.source(), &at, Some(e.to_string())))?;
        let file: PolicyFile = toml::from_str(&text)
            .map_err(|e| Not::unreadable(self.source(), &at, Some(e.to_string())))?;
        Ok(file
            .build
            .outputs
            .into_iter()
            .filter_map(|(output, table)| table.task.map(|task| (output, task)))
            .collect())
    }
}

#[cfg(test)]
pub(super) mod tests {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    use tempfile::TempDir;

    use super::*;

    /// A git repository in a temporary directory, committed to under a
    /// fixed author and date with no global or system configuration.
    pub(in crate::profiles::build::moon) struct Repo {
        dir: TempDir,
    }

    impl Repo {
        pub fn new() -> Repo {
            let repo = Repo {
                dir: TempDir::new().unwrap(),
            };
            repo.git(&["init", "-q", "-b", "main"]);
            repo
        }

        pub fn path(&self) -> &Path {
            self.dir.path()
        }

        pub fn git(&self, args: &[&str]) -> String {
            let mut command = Command::new("git");
            for name in GIT_REPOSITORY_ENV {
                command.env_remove(name);
            }
            let out = command
                .arg("-C")
                .arg(self.path())
                .args(args)
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_AUTHOR_NAME", "an author")
                .env("GIT_AUTHOR_EMAIL", "author@example.org")
                .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z")
                .env("GIT_COMMITTER_NAME", "an author")
                .env("GIT_COMMITTER_EMAIL", "author@example.org")
                .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z")
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            String::from_utf8(out.stdout).unwrap().trim().to_string()
        }

        pub fn write(&self, path: &str, text: &str) {
            let at = self.path().join(path);
            fs::create_dir_all(at.parent().unwrap()).unwrap();
            fs::write(at, text).unwrap();
        }

        /// Everything written, committed; answers the commit.
        pub fn commit(&self, message: &str) -> String {
            self.git(&["add", "-A"]);
            self.git(&["commit", "-q", "--allow-empty", "-m", message]);
            self.git(&["rev-parse", "HEAD"])
        }

        pub fn moon(&self) -> Moon {
            Moon::new(self.path())
        }
    }

    fn output(name: &str) -> OutputName {
        OutputName::new(name)
    }

    #[test]
    fn the_task_named_at_a_revision_not_the_one_an_uncommitted_edit_names() {
        let repo = Repo::new();
        repo.write(
            POLICY_FILE,
            "outputs = [\"site\"]\n\n[build.outputs.site]\ntask = \"web:build\"\n",
        );
        let landed = repo.commit("the site's task");
        repo.write(POLICY_FILE, "[build.outputs.site]\ntask = \"web:other\"\n");
        assert_eq!(
            repo.moon().tasks(&landed),
            Ok(BTreeMap::from([(output("site"), "web:build".to_string())]))
        );
        let resolved = repo.moon().commit(&Revision::new("HEAD")).unwrap();
        assert_eq!(resolved, landed);
    }

    #[test]
    fn an_output_with_no_table_names_no_task() {
        let repo = Repo::new();
        let none = repo.commit("no policy file");
        assert_eq!(repo.moon().tasks(&none), Ok(BTreeMap::new()));
        repo.write(
            POLICY_FILE,
            "outputs = [\"site\", \"server\", \"checks\"]\n\n\
             [projections]\nsite = \"file\"\n\n\
             [build.outputs.site]\ntask = \"web:build\"\n\n\
             [build.outputs.checks]\n",
        );
        let some = repo.commit("one output's task");
        assert_eq!(
            repo.moon().tasks(&some),
            Ok(BTreeMap::from([(output("site"), "web:build".to_string())]))
        );
    }

    #[test]
    fn a_policy_file_that_is_not_toml_is_unreadable_naming_it() {
        let repo = Repo::new();
        repo.write(POLICY_FILE, "[build.outputs.site\n");
        let commit = repo.commit("a broken policy file");
        match repo.moon().tasks(&commit) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, source());
                assert_eq!(u.what, format!("{commit}:{POLICY_FILE}"));
                assert!(u.said.is_some_and(|s| !s.is_empty()));
            }
            other => panic!("answered {other:?}"),
        }
    }

    #[test]
    fn a_revision_the_repository_does_not_hold_is_absent() {
        let repo = Repo::new();
        repo.commit("one commit");
        let missing = Revision::new("0000000000000000000000000000000000000001");
        assert_eq!(
            repo.moon().commit(&missing),
            Err(Not::absent(&source(), format!("revision {missing}")))
        );
    }
}
