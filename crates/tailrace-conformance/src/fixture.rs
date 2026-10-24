//! The git fixture the core's groups run over: a bare repository standing
//! for the history every publisher shares and a clone of it, in a
//! temporary directory, made with the git command under no configuration
//! of the host's and a fixed author, committer and date, so the same
//! steps give the same revisions on every run (design D4).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tailrace_core::POLICY_FILE;
use tailrace_seams::Revision;
use tempfile::TempDir;

/// Who writes every commit, and when.
pub const AUTHOR: &str = "A. Uthor";
pub const EMAIL: &str = "author@example.com";
pub const DATE: &str = "2026-01-01T00:00:00+00:00";

/// The branch every repository of the fixture lands on.
pub const MAIN: &str = "main";

/// The git command at `dir`, under the fixed author, committer and date
/// and none of the host's configuration; panics with git's words when it
/// fails.
pub fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", AUTHOR)
        .env("GIT_AUTHOR_EMAIL", EMAIL)
        .env("GIT_AUTHOR_DATE", DATE)
        .env("GIT_COMMITTER_NAME", AUTHOR)
        .env("GIT_COMMITTER_EMAIL", EMAIL)
        .env("GIT_COMMITTER_DATE", DATE)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .expect("git answers text")
        .trim_end()
        .to_string()
}

/// One repository of the fixture: the shared history, or a clone of it
/// with a working tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repo {
    pub path: PathBuf,
}

impl Repo {
    pub fn git(&self, args: &[&str]) -> String {
        git(&self.path, args)
    }

    /// A commit with `message` writing each of `files`, and its revision.
    pub fn commit(&self, files: &[(&str, &str)], message: &str) -> Revision {
        for (file, contents) in files {
            self.write(file, contents);
            self.git(&["add", "--", file]);
        }
        self.git(&["commit", "-q", "--allow-empty", "-m", message]);
        self.head()
    }

    /// `main` and every tag pushed to the shared history, in one push.
    pub fn push(&self) {
        self.git(&["push", "-q", "origin", MAIN, "--tags"]);
    }

    /// `contents` written at `file` in the working tree, and nothing
    /// committed.
    pub fn write(&self, file: &str, contents: &str) {
        let at = self.path.join(file);
        fs::create_dir_all(at.parent().expect("a file is under the tree"))
            .expect("the tree is writable");
        fs::write(&at, contents).expect("the tree is writable");
    }

    /// The policy file committed with `text`, and its revision.
    pub fn policy_file(&self, text: &str) -> Revision {
        self.commit(&[(POLICY_FILE, text)], "chore: the policy")
    }

    /// A lightweight tag `name` at `at`, in this repository alone.
    pub fn tag(&self, name: &str, at: &Revision) {
        self.git(&["tag", name, at.as_str()]);
    }

    pub fn head(&self) -> Revision {
        Revision::new(self.git(&["rev-parse", "HEAD"]))
    }

    /// Every tag this repository holds, by name.
    pub fn tags(&self) -> Vec<String> {
        self.git(&["tag", "-l"])
            .lines()
            .map(str::to_string)
            .collect()
    }

    /// The revision the tag `name` names here, or none.
    pub fn tag_at(&self, name: &str) -> Option<Revision> {
        let names = self.git(&["tag", "-l", name]);
        (!names.is_empty()).then(|| {
            Revision::new(self.git(&["rev-parse", &format!("refs/tags/{name}^{{commit}}")]))
        })
    }
}

/// A shared history and a clone of it, in a directory of their own that
/// is removed with the fixture.
pub struct Fixture {
    root: TempDir,
    pub shared: Repo,
    pub repo: Repo,
}

impl Default for Fixture {
    fn default() -> Self {
        Self::new()
    }
}

impl Fixture {
    /// An empty shared history on `main`, and a clone whose `main`
    /// follows it.
    pub fn new() -> Fixture {
        let root = TempDir::new().expect("a temporary directory");
        git(
            root.path(),
            &["init", "-q", "--bare", "-b", MAIN, "shared.git"],
        );
        let shared = Repo {
            path: root.path().join("shared.git"),
        };
        let fixture = Fixture {
            repo: Repo {
                path: root.path().join("repo"),
            },
            root,
            shared,
        };
        fixture.clone_into("repo");
        fixture
    }

    fn clone_into(&self, name: &str) -> Repo {
        git(self.root.path(), &["clone", "-q", "shared.git", name]);
        let repo = Repo {
            path: self.root.path().join(name),
        };
        // A clone of an empty history sets no upstream; one of a history
        // holding `main` would.
        repo.git(&["config", &format!("branch.{MAIN}.remote"), "origin"]);
        repo.git(&[
            "config",
            &format!("branch.{MAIN}.merge"),
            &format!("refs/heads/{MAIN}"),
        ]);
        repo
    }

    /// A second clone of the shared history as it stands, for a second
    /// publisher.
    pub fn clone_again(&self) -> Repo {
        let mut n = 2;
        while self.root.path().join(format!("clone-{n}")).exists() {
            n += 1;
        }
        self.clone_into(&format!("clone-{n}"))
    }

    /// The directory the fixture's repositories are in.
    pub fn root(&self) -> &Path {
        self.root.path()
    }
}
