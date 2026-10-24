//! The scratch repositories the profiles' tests run over (design D9): a
//! repository in a temporary directory, made with the git command under a
//! fixed author, committer and date and none of the host's
//! configuration, as the suite's fixture is made; a bare remote and a
//! second clone of it; and a moon workspace in it whose shell tasks write
//! their declared outputs and exit as they are told.

#![allow(dead_code)]

pub mod gate;
pub mod recorder;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

/// Who writes every commit, and when.
pub const AUTHOR: &str = "A. Uthor";
pub const EMAIL: &str = "author@example.com";
pub const DATE: &str = "2026-01-01T00:00:00+00:00";

/// The branch every scratch repository lands on.
pub const MAIN: &str = "main";

/// The policy file, as the core reads it.
pub const POLICY_FILE: &str = ".config/tailrace.toml";

/// The variables that would point git at another repository than the one
/// a test names.
const GIT_REPOSITORY_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_COMMON_DIR",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_PREFIX",
];

/// The git command, under the fixed author, committer and date and none
/// of the host's configuration.
pub fn git_command() -> Command {
    let mut command = Command::new("git");
    for name in GIT_REPOSITORY_ENV {
        command.env_remove(name);
    }
    command
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_AUTHOR_NAME", AUTHOR)
        .env("GIT_AUTHOR_EMAIL", EMAIL)
        .env("GIT_AUTHOR_DATE", DATE)
        .env("GIT_COMMITTER_NAME", AUTHOR)
        .env("GIT_COMMITTER_EMAIL", EMAIL)
        .env("GIT_COMMITTER_DATE", DATE);
    command
}

/// git at `dir`; panics with git's words when it fails.
pub fn git(dir: &Path, args: &[&str]) -> String {
    let out = git_command()
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?} in {} failed: {}",
        dir.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .expect("git answers text")
        .trim_end()
        .to_string()
}

/// What git says to stderr when `args` fail at `dir`, trimmed, as a
/// profile carries it.
pub fn git_fails(dir: &Path, args: &[&str]) -> String {
    let out = git_command()
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git runs");
    assert!(!out.status.success(), "git {args:?} succeeded");
    String::from_utf8_lossy(&out.stderr).trim().to_string()
}

/// The program `name` as the shell finds it on `PATH`; panics naming it
/// when the shell holds none, as on a system it is not built for.
pub fn on_path(name: &str) -> PathBuf {
    std::env::var_os("PATH")
        .iter()
        .flat_map(std::env::split_paths)
        .map(|dir| dir.join(name))
        .find(|path| path.is_file())
        .unwrap_or_else(|| {
            panic!("{name} is not on PATH: the tools layer runs in the devenv shell, which pins it")
        })
}

/// One repository with a working tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repo {
    pub path: PathBuf,
}

impl Repo {
    pub fn git(&self, args: &[&str]) -> String {
        git(&self.path, args)
    }

    /// `contents` written at `file` in the working tree, nothing
    /// committed.
    pub fn write(&self, file: &str, contents: &str) {
        let at = self.path.join(file);
        fs::create_dir_all(at.parent().expect("a file is under the tree"))
            .expect("the tree is writable");
        fs::write(&at, contents).expect("the tree is writable");
    }

    /// Everything written, committed with `message`; answers the commit.
    pub fn commit(&self, message: &str) -> String {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "--allow-empty", "-m", message]);
        self.head()
    }

    pub fn head(&self) -> String {
        self.git(&["rev-parse", "HEAD"])
    }

    /// `main` pushed to the remote.
    pub fn push(&self) {
        self.git(&["push", "-q", "origin", MAIN]);
    }
}

/// A scratch directory holding a bare remote, the repository a publisher
/// works in, a clone of it, and whatever else a test keeps there.
pub struct Scratch {
    dir: TempDir,
    pub remote: PathBuf,
    pub repo: Repo,
}

impl Scratch {
    /// A repository on `main` with one commit, and a bare remote holding
    /// it as `origin`.
    pub fn new() -> Scratch {
        let dir = TempDir::new().expect("a temporary directory");
        // The canonical path, as git and moon name it back.
        let root = dir.path().canonicalize().expect("the directory is there");
        let remote = root.join("remote.git");
        git(&root, &["init", "-q", "--bare", "-b", MAIN, "remote.git"]);
        git(&root, &["init", "-q", "-b", MAIN, "repo"]);
        let repo = Repo {
            path: root.join("repo"),
        };
        repo.write(".gitignore", ".moon/cache\n");
        repo.commit("chore: the repository");
        repo.git(&["remote", "add", "origin", remote.to_str().unwrap()]);
        repo.push();
        Scratch { dir, remote, repo }
    }

    /// The scratch directory itself.
    pub fn path(&self) -> PathBuf {
        self.repo.path.parent().unwrap().to_path_buf()
    }

    /// A clone of the remote at `name`, as a second host keeps one.
    pub fn clone_as(&self, name: &str) -> Repo {
        git(
            &self.path(),
            &["clone", "-q", self.remote.to_str().unwrap(), name],
        );
        Repo {
            path: self.path().join(name),
        }
    }

    /// A link at `name` in the scratch directory to `program` as the
    /// shell finds it, so a test can take the program away.
    pub fn link(&self, name: &str, program: &str) -> PathBuf {
        let bin = self.path().join("bin");
        fs::create_dir_all(&bin).unwrap();
        let link = bin.join(name);
        std::os::unix::fs::symlink(on_path(program), &link).unwrap();
        link
    }
}

/// A shell task of the scratch moon workspace, `<project>:<task>`,
/// declaring `dist/<file>` as its output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub project: String,
    pub task: String,
    pub file: String,
    /// Whether it writes its declared file.
    pub writes: bool,
    /// What it exits with.
    pub exit: i32,
}

impl Task {
    pub fn new(project: &str, task: &str, file: &str, writes: bool, exit: i32) -> Task {
        Task {
            project: project.into(),
            task: task.into(),
            file: file.into(),
            writes,
            exit,
        }
    }

    pub fn target(&self) -> String {
        format!("{}:{}", self.project, self.task)
    }
}

/// The moon workspace `tasks` make, written in `repo` and nothing
/// committed: `.moon/workspace.yml` naming each project and git as its
/// version control, and each project's `moon.yml`, whose tasks read
/// their own `moon.yml` alone, so a task's hash is the same in every
/// checkout of one revision.
pub fn moon_workspace(repo: &Repo, tasks: &[Task]) {
    let mut projects: Vec<&str> = tasks.iter().map(|t| t.project.as_str()).collect();
    projects.sort_unstable();
    projects.dedup();
    let mut workspace = String::from("projects:\n");
    for project in &projects {
        workspace.push_str(&format!("  {project}: '{project}'\n"));
    }
    workspace.push_str("vcs:\n  client: 'git'\n  defaultBranch: 'main'\ntelemetry: false\n");
    repo.write(".moon/workspace.yml", &workspace);
    for project in &projects {
        let mut config = String::from("tasks:\n");
        for t in tasks.iter().filter(|t| t.project == *project) {
            let write = if t.writes {
                format!(
                    "mkdir -p dist && printf {} > dist/{} && ",
                    t.target(),
                    t.file
                )
            } else {
                String::new()
            };
            config.push_str(&format!(
                "  {}:\n    command: 'sh -c \"{write}exit {}\"'\n    inputs: ['moon.yml']\n    outputs: ['dist/{}']\n    options: {{shell: false}}\n",
                t.task, t.exit, t.file
            ));
        }
        repo.write(&format!("{project}/moon.yml"), &config);
    }
}

/// The policy file naming the task that makes each of `outputs`, as
/// `(output, target)`, and declaring them the repository's outputs,
/// written in `repo` and nothing committed; `extra` follows it, for the
/// tables a test adds.
pub fn policy_file(repo: &Repo, outputs: &[(&str, &str)], extra: &str) {
    let names: Vec<String> = outputs.iter().map(|(o, _)| format!("\"{o}\"")).collect();
    let mut text = format!("outputs = [{}]\n", names.join(", "));
    text.push_str(extra);
    for (output, target) in outputs {
        text.push_str(&format!(
            "\n[build.outputs.{output}]\ntask = \"{target}\"\n"
        ));
    }
    repo.write(POLICY_FILE, &text);
}

/// The hash moon gives `target` when it runs it in `dir` itself, read
/// from its run report: what the profile's build identity must be.
pub fn moon_hash(dir: &Path, target: &str) -> String {
    let out = Command::new(on_path("moon"))
        .args(["exec", target, "--on-failure", "continue"])
        .current_dir(dir)
        .env_remove("MOON_HOME")
        .stdin(std::process::Stdio::null())
        .output()
        .expect("moon runs");
    let report: serde_json::Value = serde_json::from_slice(
        &fs::read(dir.join(".moon/cache/runReport.json")).unwrap_or_else(|e| {
            panic!(
                "moon left no run report ({e}): {}",
                String::from_utf8_lossy(&out.stderr)
            )
        }),
    )
    .unwrap();
    report["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["label"] == format!("RunTask({target})"))
        .and_then(|a| {
            a["operations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|o| o["meta"]["type"] == "hash-generation")
        })
        .and_then(|o| o["meta"]["hash"].as_str())
        .unwrap_or_else(|| panic!("moon's run report holds no hash of {target}: {report}"))
        .to_string()
}
