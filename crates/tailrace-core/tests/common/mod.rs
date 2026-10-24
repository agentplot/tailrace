//! Repositories made with the git command in a temporary directory, by a
//! fixed author, for the core's tests over real history (design D8).

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tailrace_core::{History, Landed};
use tailrace_fakes::FakeLanding;
use tailrace_seams::{Branch, Revision};
use tempfile::TempDir;

/// A directory holding the repositories of one test.
pub struct Fixture {
    root: TempDir,
}

/// One repository of a fixture: a working tree, or a bare one standing
/// for the shared history.
#[derive(Clone, Debug)]
pub struct Repo {
    pub path: PathBuf,
}

/// The git command at `dir`, by the fixed author, with no configuration
/// of the host's, so no host's signing or hooks reach a test.
pub fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "A. Uthor")
        .env("GIT_AUTHOR_EMAIL", "author@example.com")
        .env("GIT_COMMITTER_NAME", "A. Uthor")
        .env("GIT_COMMITTER_EMAIL", "author@example.com")
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .unwrap()
        .trim_end()
        .to_string()
}

impl Fixture {
    pub fn new() -> Self {
        Fixture {
            root: TempDir::new().unwrap(),
        }
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.root.path().join(name)
    }

    /// A new repository with a working tree, on `main`.
    pub fn repo(&self, name: &str) -> Repo {
        let path = self.path(name);
        git(self.root.path(), &["init", "-q", "-b", "main", name]);
        Repo { path }
    }

    /// A new bare repository, the history every publisher shares.
    pub fn bare(&self, name: &str) -> Repo {
        let path = self.path(name);
        git(
            self.root.path(),
            &["init", "-q", "--bare", "-b", "main", name],
        );
        Repo { path }
    }

    /// A clone of `from` named `name`, its `main` following `origin`.
    pub fn clone_of(&self, from: &Repo, name: &str) -> Repo {
        let path = self.path(name);
        git(
            self.root.path(),
            &["clone", "-q", from.path.to_str().unwrap(), name],
        );
        let repo = Repo { path };
        // A clone of an empty repository sets no upstream; a clone of one
        // holding `main` would.
        repo.git(&["config", "branch.main.remote", "origin"]);
        repo.git(&["config", "branch.main.merge", "refs/heads/main"]);
        repo
    }
}

impl Repo {
    pub fn git(&self, args: &[&str]) -> String {
        git(&self.path, args)
    }

    /// A commit with `message` writing each file, and its revision.
    pub fn commit(&self, message: &str, files: &[(&str, &str)]) -> Revision {
        for (file, contents) in files {
            let at = self.path.join(file);
            fs::create_dir_all(at.parent().unwrap()).unwrap();
            fs::write(&at, contents).unwrap();
            self.git(&["add", "--", file]);
        }
        self.git(&["commit", "-q", "--allow-empty", "-m", message]);
        self.head()
    }

    /// A commit with `message` changing one file under each of `paths`.
    pub fn touch(&self, message: &str, paths: &[&str]) -> Revision {
        let files: Vec<(String, String)> = paths
            .iter()
            .map(|p| (format!("{p}/touched"), message.to_string()))
            .collect();
        let files: Vec<(&str, &str)> = files
            .iter()
            .map(|(f, c)| (f.as_str(), c.as_str()))
            .collect();
        self.commit(message, &files)
    }

    pub fn head(&self) -> Revision {
        Revision::new(self.git(&["rev-parse", "HEAD"]))
    }

    /// A lightweight tag `name` at `at`.
    pub fn tag(&self, name: &str, at: &Revision) {
        self.git(&["tag", name, at.as_str()]);
    }

    /// An annotated tag `name` at `at`.
    pub fn annotated(&self, name: &str, at: &Revision) {
        self.git(&["tag", "-a", "-m", name, name, at.as_str()]);
    }

    /// The revision the tag `name` names, peeled, or none.
    pub fn tag_at(&self, name: &str) -> Option<String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .args([
                "rev-parse",
                "-q",
                "--verify",
                &format!("refs/tags/{name}^{{commit}}"),
            ])
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output()
            .unwrap();
        out.status
            .success()
            .then(|| String::from_utf8(out.stdout).unwrap().trim().to_string())
    }

    pub fn history(&self) -> History {
        History::open(&self.path).unwrap()
    }
}

/// `revision` as landed on `main`, by a fake landing that says so.
pub fn landed(revision: &Revision) -> Landed {
    let landing = FakeLanding::new();
    let main = Branch::new("main");
    landing.land(&main, revision);
    Landed::check(&landing, &main, revision).unwrap()
}
