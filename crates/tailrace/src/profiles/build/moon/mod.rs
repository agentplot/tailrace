//! The build seam (B.2) bound to moon (C.3): the repository names, in
//! its policy file at the revision, the moon task that makes each
//! output; ensure is one moon run of those tasks in a checkout of exactly
//! that revision, and report reads moon's own record of the run, the
//! task's hash its build identity (74 to 76). Left reads a checkout's own
//! run report and the archives moon's outputs directory holds, and runs
//! no task (139).
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
//! directory and then moon's version, so a repository or a moon that
//! cannot be reached answers every operation unreadable with the tool's
//! or the operating system's words, before any answer of absent or
//! refused (58, 59).

mod checkout;
mod report;

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tailrace_seams::{
    Answer, Build, BuildId, Left, Not, OutputName, Report, Revision, Seam, Source, Why,
};

use crate::profiles::run::{Ran, Tool, GIT_REPOSITORY_ENV};
use report::{Declared, RunReport, OUTPUTS, RUN_REPORT};

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

    /// The opening read of every operation: the repository's common git
    /// directory, then moon's version in the repository, so neither
    /// absent nor refused is answered by a build that cannot run moon
    /// (design D4).
    fn open(&self) -> Answer<PathBuf> {
        let common = self.common()?;
        let args = ["--version"];
        let ran = self.moon(&self.repository, &args)?;
        if ran.code != Some(0) {
            return Err(self.moon.failed(&args, &ran));
        }
        Ok(common)
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

    /// moon run in `checkout` with `args`, with no variable of
    /// tailrace's own environment that names moon's settings or points
    /// git elsewhere, whatever it exits with.
    fn moon<S: AsRef<OsStr>>(&self, checkout: &Path, args: &[S]) -> Answer<Ran> {
        let mut env_drop = vec!["MOON_*"];
        env_drop.extend(GIT_REPOSITORY_ENV);
        self.moon.ran(Some(checkout), args, &env_drop)
    }

    /// Where `target` says its outputs are, from `moon task <target>
    /// --json` in `checkout`.
    fn declared(&self, checkout: &Path, target: &str) -> Answer<Declared> {
        let args = ["task", target, "--json"];
        let ran = self.moon(checkout, &args)?;
        if ran.code != Some(0) {
            return Err(self.moon.failed(&args, &ran));
        }
        Declared::read(&ran.stdout).map_err(|why| self.moon.unreadable(&args, why))
    }

    /// The run report moon left in `checkout`, `None` when it left none.
    fn run_report(&self, checkout: &Path) -> Answer<Option<RunReport>> {
        let at = checkout.join(RUN_REPORT);
        let bytes = match fs::read(&at) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(self.unreadable_file(&at, e.to_string())),
        };
        RunReport::read(&bytes)
            .map(Some)
            .map_err(|why| self.unreadable_file(&at, why))
    }

    fn unreadable_file(&self, path: &Path, said: String) -> Not {
        Not::unreadable(self.source(), path.display().to_string(), Some(said))
    }
}

impl Build for Moon {
    /// 74: the tasks the outputs name, run in one `moon exec` in the
    /// revision's checkout, continuing past a task that fails; moon's
    /// exit is no answer, since a task that failed is an outcome (75).
    /// An output is unmade when the policy file names it no task, when
    /// its task has no action in the run report, or when the files its
    /// task declares are not there.
    fn ensure(&self, revision: &Revision, outputs: &[OutputName]) -> Answer<()> {
        let common = self.open()?;
        let commit = self.commit(revision)?;
        let tasks = self.tasks(&commit)?;
        let checkout = self.checkout(&common, &commit)?;
        let left = checkout.join(RUN_REPORT);
        match fs::remove_file(&left) {
            Err(e) if e.kind() != ErrorKind::NotFound => {
                return Err(self.unreadable_file(&left, e.to_string()))
            }
            _ => {}
        }
        let targets: BTreeSet<&str> = outputs
            .iter()
            .filter_map(|output| tasks.get(output).map(String::as_str))
            .collect();
        let mut made = BTreeSet::new();
        if !targets.is_empty() {
            let mut args = vec!["exec"];
            args.extend(&targets);
            args.extend(["--on-failure", "continue"]);
            let ran = self.moon(&checkout, &args)?;
            let Some(report) = self.run_report(&checkout)? else {
                return Err(self.moon.failed(&args, &ran));
            };
            for target in targets {
                if report.task(target).is_some()
                    && !self
                        .declared(&checkout, target)?
                        .files_in(&checkout)
                        .is_empty()
                {
                    made.insert(target);
                }
            }
        }
        let unmade: Vec<OutputName> = outputs
            .iter()
            .filter(|output| {
                !tasks
                    .get(*output)
                    .is_some_and(|task| made.contains(task.as_str()))
            })
            .cloned()
            .collect();
        if unmade.is_empty() {
            Ok(())
        } else {
            Err(Not::refused(
                self.source(),
                Why::Unmade { outputs: unmade },
                None,
            ))
        }
    }

    /// 75: the task's hash as the build identity, its status as the
    /// outcome, and the files its declared outputs name in the checkout,
    /// from what moon recorded of the last ensure at the revision; absent
    /// naming the output where moon recorded nothing of its task.
    fn report(&self, revision: &Revision, output: &OutputName) -> Answer<Report> {
        let common = self.open()?;
        let commit = self.commit(revision)?;
        let tasks = self.tasks(&commit)?;
        let absent = || Not::absent(self.source(), format!("output {output} at {commit}"));
        let Some(target) = tasks.get(output) else {
            return Err(absent());
        };
        let Some(checkout) = self.made(&common, &commit) else {
            return Err(absent());
        };
        let declared = self.declared(&checkout, target)?;
        let Some(report) = self.run_report(&checkout)? else {
            return Err(absent());
        };
        let Some(action) = report.task(target) else {
            return Err(absent());
        };
        let (Some(hash), Some(outcome)) = (&action.hash, action.outcome()) else {
            return Err(absent());
        };
        Ok(Report {
            build: BuildId::new(hash.clone()),
            outcome,
            files: declared
                .files_in(&checkout)
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
        })
    }

    /// 139: each task the run report in the checkout at `at` records with
    /// a hash and an outcome, whose archive moon's outputs directory there
    /// holds, grouped by hash in the report's order: the hash as the
    /// build identity, the tasks with it, its outcome, and the files they
    /// declare that are there. moon archives a task that passes, with
    /// declared outputs or none, and never one that fails, so a failed
    /// task is never named. A checkout with no run report ran nothing and
    /// answers an empty list. No task is run.
    fn left(&self, at: &str) -> Answer<Vec<Left>> {
        self.open()?;
        let checkout = Path::new(at);
        let Some(report) = self.run_report(checkout)? else {
            return Ok(Vec::new());
        };
        let outputs = checkout.join(OUTPUTS);
        let mut left: Vec<Left> = Vec::new();
        for (target, action) in report.tasks() {
            let (Some(hash), Some(outcome)) = (&action.hash, action.outcome()) else {
                continue;
            };
            let archive = outputs.join(format!("{hash}.tar.gz"));
            match fs::metadata(&archive) {
                Ok(meta) if meta.is_file() => {}
                Ok(_) => continue,
                Err(e) if e.kind() == ErrorKind::NotFound => continue,
                Err(e) => return Err(self.unreadable_file(&archive, e.to_string())),
            }
            let files: Vec<String> = self
                .declared(checkout, target)?
                .files_in(checkout)
                .iter()
                .map(|path| path.display().to_string())
                .collect();
            match left.iter_mut().find(|l| l.build.as_str() == hash) {
                Some(same) => {
                    same.tasks.push(target.to_string());
                    for file in files {
                        if !same.files.contains(&file) {
                            same.files.push(file);
                        }
                    }
                }
                None => left.push(Left {
                    build: BuildId::new(hash.clone()),
                    tasks: vec![target.to_string()],
                    outcome,
                    files,
                }),
            }
        }
        Ok(left)
    }
}

#[cfg(test)]
pub(super) mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::process::Command;

    use tailrace_seams::Outcome;
    use tempfile::TempDir;

    use super::report::tests::{APP_TEST, RAN, WEB_BUILD};
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

    /// A moon that runs nothing: it prints its version, and for `moon
    /// task <target> --json` the outputs `declared` gives the target, none
    /// for any other; every asking is logged, so a test sees no task run.
    struct StubMoon {
        dir: TempDir,
    }

    impl StubMoon {
        fn new(declared: &[(&str, &str)]) -> StubMoon {
            let dir = TempDir::new().unwrap();
            let log = dir.path().join("asked");
            let mut cases = String::new();
            for (target, json) in declared {
                cases.push_str(&format!("  '{target}') printf '%s\\n' '{json}' ;;\n"));
            }
            let script = format!(
                "#!/bin/sh\n\
                 printf '%s\\n' \"$*\" >> '{log}'\n\
                 case \"$1\" in\n\
                 --version) echo 'moon 2.6.1' ;;\n\
                 task)\n\
                 case \"$2\" in\n\
                 {cases}\
                 *) printf '%s\\n' '{{\"outputFiles\":null,\"outputGlobs\":null}}' ;;\n\
                 esac ;;\n\
                 *) exit 1 ;;\n\
                 esac\n",
                log = log.display(),
            );
            let program = dir.path().join("moon");
            fs::write(&program, script).unwrap();
            fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
            StubMoon { dir }
        }

        fn program(&self) -> PathBuf {
            self.dir.path().join("moon")
        }

        /// Every asking, its arguments joined by spaces.
        fn asked(&self) -> Vec<String> {
            fs::read_to_string(self.dir.path().join("asked"))
                .unwrap_or_default()
                .lines()
                .map(str::to_string)
                .collect()
        }
    }

    /// moon's run report of `tasks`, each a target, its hash and its
    /// status, in moon's form.
    fn ran(tasks: &[(&str, &str, &str)]) -> String {
        let actions: Vec<serde_json::Value> = tasks
            .iter()
            .map(|(target, hash, status)| {
                serde_json::json!({
                    "label": format!("RunTask({target})"),
                    "node": {"action": "run-task", "params": {"target": target}},
                    "operations": [{"meta": {"type": "hash-generation", "hash": hash}}],
                    "status": status,
                })
            })
            .collect();
        serde_json::json!({ "actions": actions }).to_string()
    }

    /// `report` left as the run report of the checkout at `at`, and an
    /// archive in its outputs directory for each of `archived`.
    fn left_by_a_run(at: &Path, report: &str, archived: &[&str]) {
        fs::create_dir_all(at.join(OUTPUTS)).unwrap();
        fs::write(at.join(RUN_REPORT), report).unwrap();
        for hash in archived {
            fs::write(at.join(OUTPUTS).join(format!("{hash}.tar.gz")), hash).unwrap();
        }
    }

    fn at(repo: &Repo) -> &str {
        repo.path().to_str().unwrap()
    }

    #[test]
    fn a_run_that_passed_one_task_and_failed_another() {
        let repo = Repo::new();
        let stub = StubMoon::new(&[(
            "web:build",
            r#"{"outputFiles":{"web/dist/site.txt":{"optional":false}},"outputGlobs":null}"#,
        )]);
        repo.write("web/dist/site.txt", "site");
        repo.write("app/out/app.txt", "app");
        left_by_a_run(repo.path(), RAN, &[WEB_BUILD]);
        let moon = repo.moon().moon_program(stub.program());
        assert_eq!(
            moon.left(at(&repo)),
            Ok(vec![Left {
                build: BuildId::new(WEB_BUILD),
                tasks: vec!["web:build".into()],
                outcome: Outcome::Pass,
                files: vec![repo.path().join("web/dist/site.txt").display().to_string()],
            }])
        );
        assert!(
            !moon
                .left(at(&repo))
                .unwrap()
                .iter()
                .any(|l| l.build.as_str() == APP_TEST),
            "the failed task is named"
        );
        assert!(
            stub.asked()
                .iter()
                .all(|a| a == "--version" || a == "task web:build --json"),
            "{:?}",
            stub.asked()
        );
    }

    #[test]
    fn a_task_with_no_declared_outputs() {
        let repo = Repo::new();
        let stub = StubMoon::new(&[]);
        left_by_a_run(repo.path(), &ran(&[("app:lint", "a1", "passed")]), &["a1"]);
        assert_eq!(
            repo.moon().moon_program(stub.program()).left(at(&repo)),
            Ok(vec![Left {
                build: BuildId::new("a1"),
                tasks: vec!["app:lint".into()],
                outcome: Outcome::Pass,
                files: vec![],
            }])
        );
    }

    #[test]
    fn tasks_of_one_hash_are_one_build_in_the_reports_order() {
        let repo = Repo::new();
        let stub = StubMoon::new(&[
            (
                "web:build",
                r#"{"outputFiles":{"web/dist/site.txt":{}},"outputGlobs":null}"#,
            ),
            (
                "web:copy",
                r#"{"outputFiles":{"web/dist/site.txt":{},"web/dist/copy.txt":{}},"outputGlobs":null}"#,
            ),
        ]);
        repo.write("web/dist/site.txt", "site");
        repo.write("web/dist/copy.txt", "copy");
        left_by_a_run(
            repo.path(),
            &ran(&[
                ("web:build", "b2", "passed"),
                ("app:lint", "a1", "cached"),
                ("web:copy", "b2", "cached"),
            ]),
            &["a1", "b2"],
        );
        let left = repo
            .moon()
            .moon_program(stub.program())
            .left(at(&repo))
            .unwrap();
        let named: Vec<(&str, Vec<String>, usize)> = left
            .iter()
            .map(|l| (l.build.as_str(), l.tasks.clone(), l.files.len()))
            .collect();
        assert_eq!(
            named,
            [
                ("b2", vec!["web:build".into(), "web:copy".into()], 2),
                ("a1", vec!["app:lint".into()], 0),
            ]
        );
    }

    #[test]
    fn a_checkout_that_ran_nothing() {
        let repo = Repo::new();
        let stub = StubMoon::new(&[]);
        assert_eq!(
            repo.moon().moon_program(stub.program()).left(at(&repo)),
            Ok(vec![])
        );
        assert_eq!(stub.asked(), ["--version"]);
    }

    #[test]
    fn an_archive_removed_after_the_run() {
        let repo = Repo::new();
        let stub = StubMoon::new(&[]);
        left_by_a_run(repo.path(), RAN, &[WEB_BUILD]);
        fs::remove_file(
            repo.path()
                .join(OUTPUTS)
                .join(format!("{WEB_BUILD}.tar.gz")),
        )
        .unwrap();
        assert_eq!(
            repo.moon().moon_program(stub.program()).left(at(&repo)),
            Ok(vec![])
        );
    }

    #[test]
    fn a_moon_that_cannot_run_is_unreadable_before_the_report_is_read() {
        let repo = Repo::new();
        left_by_a_run(repo.path(), "not a run report", &[]);
        let moon = repo.moon().moon_program(repo.path().join("no-moon"));
        match moon.left(at(&repo)) {
            Err(Not::Unreadable(u)) => assert!(!u.what.contains(RUN_REPORT), "{u:?}"),
            other => panic!("answered {other:?}"),
        }
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
