//! What moon recorded of a run, read from its run report, and where a
//! task says its outputs are, read from `moon task <target> --json`
//! (75, C.3; design D4). sluice reads the same field of the same report
//! for the key it signs (C.5).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tailrace_seams::Outcome;

/// The run report, relative to the workspace's root.
pub(super) const RUN_REPORT: &str = ".moon/cache/runReport.json";

/// What the run report records of one task's action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Action {
    /// The action's status: `passed`, `failed`, `cached`,
    /// `cached-from-remote`, or another moon gives a task it did not
    /// run.
    pub status: String,
    /// The task's hash, from its `hash-generation` operation.
    pub hash: Option<String>,
}

impl Action {
    /// A pass when moon ran the task to success or answered it from its
    /// cache, which holds only a task that passed; a fail when it failed;
    /// none when moon did neither.
    pub fn outcome(&self) -> Option<Outcome> {
        match self.status.as_str() {
            "passed" | "cached" | "cached-from-remote" => Some(Outcome::Pass),
            "failed" => Some(Outcome::Fail),
            _ => None,
        }
    }
}

/// The tasks a run report records, by target: only `RunTask` actions are
/// a task's run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RunReport {
    tasks: BTreeMap<String, Action>,
}

impl RunReport {
    /// The report in `bytes`, or why it is not one, in the parser's
    /// words.
    pub fn read(bytes: &[u8]) -> Result<RunReport, String> {
        let report: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        let actions = report["actions"]
            .as_array()
            .ok_or("the run report holds no list of actions")?;
        let mut tasks = BTreeMap::new();
        for action in actions {
            let Some(label) = action["label"].as_str() else {
                continue;
            };
            let Some(inner) = label
                .strip_prefix("RunTask(")
                .and_then(|l| l.strip_suffix(')'))
            else {
                continue;
            };
            let target = action["node"]["params"]["target"]
                .as_str()
                .unwrap_or(inner)
                .to_string();
            let hash = action["operations"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|op| op["meta"]["type"].as_str() == Some("hash-generation"))
                .and_then(|op| op["meta"]["hash"].as_str())
                .map(str::to_string);
            let status = action["status"].as_str().unwrap_or_default().to_string();
            tasks.insert(target, Action { status, hash });
        }
        Ok(RunReport { tasks })
    }

    /// The action of the task `target`, where the run holds one.
    pub fn task(&self, target: &str) -> Option<&Action> {
        self.tasks.get(target)
    }
}

/// The files and globs a task declares as its outputs, relative to the
/// workspace's root, as `moon task <target> --json` gives them in
/// `outputFiles` and `outputGlobs`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Declared {
    files: Vec<String>,
    globs: Vec<String>,
}

impl Declared {
    /// The outputs `moon task <target> --json` printed in `bytes`, or
    /// why it printed no task, in the parser's words.
    pub fn read(bytes: &[u8]) -> Result<Declared, String> {
        let task: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if !task.is_object() {
            return Err("moon described no task".into());
        }
        let keys = |field: &str| -> Vec<String> {
            task[field]
                .as_object()
                .map(|o| o.keys().cloned().collect())
                .unwrap_or_default()
        };
        Ok(Declared {
            files: keys("outputFiles"),
            globs: keys("outputGlobs"),
        })
    }

    /// The files under `root` the task's outputs name: each declared file
    /// that is there, and each file a glob matches, less those a glob
    /// beginning `!` matches, sorted.
    pub fn files_in(&self, root: &Path) -> Vec<PathBuf> {
        let base = glob::Pattern::escape(&root.to_string_lossy());
        let mut found: Vec<PathBuf> = self
            .files
            .iter()
            .map(|file| root.join(file))
            .filter(|path| path.is_file())
            .collect();
        for pattern in self.globs.iter().filter(|g| !g.starts_with('!')) {
            if let Ok(paths) = glob::glob(&format!("{base}/{pattern}")) {
                found.extend(paths.flatten().filter(|path| path.is_file()));
            }
        }
        let excluded: Vec<glob::Pattern> = self
            .globs
            .iter()
            .filter_map(|g| g.strip_prefix('!'))
            .filter_map(|g| glob::Pattern::new(&format!("{base}/{g}")).ok())
            .collect();
        found.retain(|path| !excluded.iter().any(|p| p.matches_path(path)));
        found.sort();
        found.dedup();
        found
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    /// moon 2.6.1's run report of `web:build` passing, `app:test` failing
    /// and a workspace sync, its actions captured whole.
    const RAN: &str = r#"{"actions": [
        {"allowFailure":false,"createdAt":"2026-10-24T03:21:11.720162","duration":{"secs":0,"nanos":14716333},"expectFailure":false,"finishedAt":"2026-10-24T03:21:11.734884","flaky":false,"label":"SyncWorkspace","node":{"action":"sync-workspace"},"nodeIndex":0,"operations":[{"duration":{"secs":0,"nanos":14407833},"finishedAt":"2026-10-24T03:21:11.734745","id":"config-schemas","meta":{"type":"sync-operation"},"startedAt":"2026-10-24T03:21:11.720338","status":"passed"}],"startedAt":"2026-10-24T03:21:11.720168","status":"passed"},
        {"allowFailure":false,"createdAt":"2026-10-24T03:21:11.735435","duration":{"secs":0,"nanos":66764542},"expectFailure":false,"finishedAt":"2026-10-24T03:21:11.802200","flaky":false,"label":"RunTask(web:build)","node":{"action":"run-task","params":{"args":[],"env":{},"interactive":false,"persistent":false,"priority":2,"target":"web:build","id":624690443369036665}},"nodeIndex":2,"operations":[{"duration":{"secs":0,"nanos":28314292},"finishedAt":"2026-10-24T03:21:11.764782","meta":{"type":"hash-generation","hash":"2d0b92021fe57b605be721ea3e1c2cf94ae717484f6068f2ff7253d372dbcc0c"},"startedAt":"2026-10-24T03:21:11.736468","status":"passed"},{"duration":{"secs":0,"nanos":19916},"finishedAt":"2026-10-24T03:21:11.764863","meta":{"type":"output-hydration"},"startedAt":"2026-10-24T03:21:11.764843","status":"skipped"},{"duration":{"secs":0,"nanos":33566208},"finishedAt":"2026-10-24T03:21:11.798547","meta":{"type":"task-execution","command":"sh -c \"mkdir -p dist && echo site > dist/site.txt\"","exitCode":0},"startedAt":"2026-10-24T03:21:11.764981","status":"passed"},{"duration":{"secs":0,"nanos":2460375},"finishedAt":"2026-10-24T03:21:11.801360","meta":{"type":"archive-creation"},"startedAt":"2026-10-24T03:21:11.798900","status":"passed"}],"startedAt":"2026-10-24T03:21:11.735436","status":"passed"},
        {"allowFailure":false,"createdAt":"2026-10-24T03:21:11.736381","duration":{"secs":0,"nanos":87354292},"error":"Task app:test failed to run.","expectFailure":false,"finishedAt":"2026-10-24T03:21:11.823737","flaky":false,"label":"RunTask(app:test)","node":{"action":"run-task","params":{"args":[],"env":{},"interactive":false,"persistent":false,"priority":2,"target":"app:test","id":12387781639968458340}},"nodeIndex":4,"operations":[{"duration":{"secs":0,"nanos":55268334},"finishedAt":"2026-10-24T03:21:11.793479","meta":{"type":"hash-generation","hash":"1fddf98dab8c928b12bf9fd0bf43b7b453d58708b3faa30e75c2c67fd54f5ba5"},"startedAt":"2026-10-24T03:21:11.738210","status":"passed"},{"duration":{"secs":0,"nanos":29375},"finishedAt":"2026-10-24T03:21:11.793562","meta":{"type":"output-hydration"},"startedAt":"2026-10-24T03:21:11.793533","status":"skipped"},{"duration":{"secs":0,"nanos":29456000},"finishedAt":"2026-10-24T03:21:11.823137","meta":{"type":"task-execution","command":"sh -c \"mkdir -p out && echo app > out/app.txt && exit 1\"","exitCode":1},"startedAt":"2026-10-24T03:21:11.793682","status":"failed"}],"startedAt":"2026-10-24T03:21:11.736383","status":"failed"}
    ]}"#;

    /// moon 2.6.1's run report of `web:build` answered from its cache.
    const CACHED: &str = r#"{"actions": [
        {"allowFailure":false,"createdAt":"2026-10-24T03:21:29.077693","duration":{"secs":0,"nanos":65809792},"expectFailure":false,"finishedAt":"2026-10-24T03:21:29.143504","flaky":false,"label":"RunTask(web:build)","node":{"action":"run-task","params":{"args":[],"env":{},"interactive":false,"persistent":false,"priority":2,"target":"web:build","id":624690443369036665}},"nodeIndex":2,"operations":[{"duration":{"secs":0,"nanos":64043917},"finishedAt":"2026-10-24T03:21:29.143214","meta":{"type":"hash-generation","hash":"2d0b92021fe57b605be721ea3e1c2cf94ae717484f6068f2ff7253d372dbcc0c"},"startedAt":"2026-10-24T03:21:29.079170","status":"passed"},{"duration":{"secs":0,"nanos":90084},"finishedAt":"2026-10-24T03:21:29.143370","meta":{"type":"output-hydration","command":"sh -c \"mkdir -p dist && echo site > dist/site.txt\"","exitCode":0},"startedAt":"2026-10-24T03:21:29.143280","status":"cached"}],"startedAt":"2026-10-24T03:21:29.077694","status":"cached"}
    ]}"#;

    const WEB_BUILD: &str = "2d0b92021fe57b605be721ea3e1c2cf94ae717484f6068f2ff7253d372dbcc0c";
    const APP_TEST: &str = "1fddf98dab8c928b12bf9fd0bf43b7b453d58708b3faa30e75c2c67fd54f5ba5";

    #[test]
    fn a_task_that_passed_and_one_that_failed() {
        let report = RunReport::read(RAN.as_bytes()).unwrap();
        let passed = report.task("web:build").unwrap();
        assert_eq!(passed.hash.as_deref(), Some(WEB_BUILD));
        assert_eq!(passed.outcome(), Some(Outcome::Pass));
        let failed = report.task("app:test").unwrap();
        assert_eq!(failed.hash.as_deref(), Some(APP_TEST));
        assert_eq!(failed.outcome(), Some(Outcome::Fail));
        assert_eq!(report.task("app:lint"), None);
    }

    #[test]
    fn a_task_answered_from_the_cache_is_a_pass_on_the_same_hash() {
        let report = RunReport::read(CACHED.as_bytes()).unwrap();
        let cached = report.task("web:build").unwrap();
        assert_eq!(cached.status, "cached");
        assert_eq!(cached.hash.as_deref(), Some(WEB_BUILD));
        assert_eq!(cached.outcome(), Some(Outcome::Pass));
        let remote = Action {
            status: "cached-from-remote".into(),
            hash: None,
        };
        assert_eq!(remote.outcome(), Some(Outcome::Pass));
        let skipped = Action {
            status: "skipped".into(),
            hash: None,
        };
        assert_eq!(skipped.outcome(), None);
    }

    #[test]
    fn an_action_that_is_not_a_task_is_no_task() {
        let report = RunReport::read(RAN.as_bytes()).unwrap();
        assert_eq!(report.tasks.len(), 2, "{report:?}");
        assert_eq!(report.task("SyncWorkspace"), None);
        assert_eq!(report.task("sync-workspace"), None);
    }

    #[test]
    fn a_report_that_is_not_one_says_why() {
        assert!(RunReport::read(b"not json").is_err());
        assert_eq!(
            RunReport::read(b"{}"),
            Err("the run report holds no list of actions".into())
        );
    }

    #[test]
    fn the_paths_an_output_names() {
        let root = tempfile::TempDir::new().unwrap();
        for file in [
            "web/dist/site.txt",
            "app/out/app.txt",
            "app/out/deep/more.txt",
            "app/out/deep/skip.log",
            "app/src/main.txt",
        ] {
            let at = root.path().join(file);
            fs::create_dir_all(at.parent().unwrap()).unwrap();
            fs::write(at, file).unwrap();
        }
        let file = Declared::read(br#"{"target":"web:build","outputFiles":{"web/dist/site.txt":{"optional":false}},"outputGlobs":null}"#).unwrap();
        assert_eq!(
            file.files_in(root.path()),
            [root.path().join("web/dist/site.txt")]
        );
        let glob = Declared::read(br#"{"target":"app:test","outputFiles":null,"outputGlobs":{"app/out/**/*":{},"!app/out/**/*.log":{}}}"#).unwrap();
        assert_eq!(
            glob.files_in(root.path()),
            [
                root.path().join("app/out/app.txt"),
                root.path().join("app/out/deep/more.txt"),
            ]
        );
        let none =
            Declared::read(br#"{"target":"app:lint","outputFiles":null,"outputGlobs":null}"#)
                .unwrap();
        assert_eq!(none.files_in(root.path()), Vec::<PathBuf>::new());
        let missing = Declared::read(br#"{"target":"web:build","outputFiles":{"web/dist/gone.txt":{"optional":false}},"outputGlobs":null}"#).unwrap();
        assert_eq!(missing.files_in(root.path()), Vec::<PathBuf>::new());
        assert!(Declared::read(b"null").is_err());
    }
}
