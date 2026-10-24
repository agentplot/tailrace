//! The one way the profiles run a tool, git or moon, answering what it
//! printed or unreadable with the tool's own words (58, 59, rule core.19;
//! design D2).

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tailrace_seams::{Answer, Not, Source};

/// The variables that point git at another repository, work tree or
/// index than the one a profile names, as a hook or a gate running
/// tailrace may have set them: every profile's git, and moon, runs with
/// them removed.
pub(crate) const GIT_REPOSITORY_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_COMMON_DIR",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_PREFIX",
];

/// A program a profile runs, and the source its answers name: the seam
/// and the profile that ran it.
#[derive(Clone, Debug)]
pub(crate) struct Tool {
    program: PathBuf,
    source: Source,
}

/// What a tool's run printed, and the exit it ended with, `None` when a
/// signal ended it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Ran {
    pub code: Option<i32>,
    /// The exit as the operating system words it.
    pub status: String,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl Tool {
    /// `program`, a path or a name found on `PATH`, answering as `source`.
    pub fn new(program: impl Into<PathBuf>, source: Source) -> Tool {
        Tool {
            program: program.into(),
            source,
        }
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    /// The tool run with `args`, in `dir` where one is given, stdin from
    /// `/dev/null`, and every variable of tailrace's own environment that
    /// `env_drop` names removed, a name ending `*` naming every variable
    /// it begins. A run that exits with one of `allowed` answers what it
    /// printed; one that cannot start is unreadable carrying the
    /// operating system's words, and one that exits otherwise is
    /// unreadable carrying its stderr, trimmed, or its exit where it
    /// printed nothing.
    pub fn output<S: AsRef<OsStr>>(
        &self,
        dir: Option<&Path>,
        args: &[S],
        env_drop: &[&str],
        allowed: &[i32],
    ) -> Answer<Ran> {
        let ran = self.ran(dir, args, env_drop)?;
        if ran.code.is_some_and(|code| allowed.contains(&code)) {
            return Ok(ran);
        }
        Err(self.failed(args, &ran))
    }

    /// The tool run as [`output`](Tool::output) runs it, whatever it
    /// exits with: unreadable only when it cannot start.
    pub fn ran<S: AsRef<OsStr>>(
        &self,
        dir: Option<&Path>,
        args: &[S],
        env_drop: &[&str],
    ) -> Answer<Ran> {
        let mut command = Command::new(&self.program);
        command
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(dir) = dir {
            command.current_dir(dir);
        }
        for (name, _) in std::env::vars_os() {
            let name_text = name.to_string_lossy();
            let dropped = env_drop.iter().any(|drop| match drop.strip_suffix('*') {
                Some(prefix) => name_text.starts_with(prefix),
                None => name_text == *drop,
            });
            if dropped {
                command.env_remove(&name);
            }
        }
        let out = command
            .output()
            .map_err(|e| self.unreadable(args, e.to_string()))?;
        Ok(Ran {
            code: out.status.code(),
            status: out.status.to_string(),
            stdout: out.stdout,
            stderr: out.stderr,
        })
    }

    /// Unreadable naming the command `ran` came from, carrying its
    /// stderr, trimmed, or its exit where it printed nothing.
    pub fn failed<S: AsRef<OsStr>>(&self, args: &[S], ran: &Ran) -> Not {
        let words = String::from_utf8_lossy(&ran.stderr).trim().to_string();
        self.unreadable(
            args,
            if words.is_empty() {
                ran.status.clone()
            } else {
                words
            },
        )
    }

    /// Unreadable naming the command run and carrying `said`.
    pub fn unreadable<S: AsRef<OsStr>>(&self, args: &[S], said: String) -> Not {
        let mut what = self.program.display().to_string();
        for arg in args {
            what.push(' ');
            what.push_str(&arg.as_ref().to_string_lossy());
        }
        Not::unreadable(&self.source, what, Some(said))
    }
}

#[cfg(test)]
mod tests {
    use tailrace_seams::Seam;

    use super::*;

    fn tool(program: &str) -> Tool {
        Tool::new(program, Source::new(Seam::Build, "a tool"))
    }

    fn said(answer: Answer<Ran>) -> String {
        match answer {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, Source::new(Seam::Build, "a tool"));
                u.said.expect("the tool's words")
            }
            other => panic!("answered {other:?}, not unreadable"),
        }
    }

    #[test]
    fn a_program_that_does_not_exist_carries_the_operating_systems_words() {
        let dir = tempfile::TempDir::new().unwrap();
        let gone = dir.path().join("moon");
        let words = said(tool(gone.to_str().unwrap()).output(None, &["--version"], &[], &[0]));
        assert_eq!(words, "No such file or directory (os error 2)");
    }

    #[test]
    fn a_command_that_fails_carries_its_own_words_trimmed_and_unchanged() {
        let dir = tempfile::TempDir::new().unwrap();
        let missing = dir.path().join("repository");
        let args = [
            OsStr::new("-C"),
            missing.as_os_str(),
            OsStr::new("rev-parse"),
            OsStr::new("--git-common-dir"),
        ];
        let theirs = Command::new("git").args(args).output().unwrap();
        assert!(!theirs.status.success());
        let words = said(tool("git").output(None, &args, &[], &[0]));
        assert_eq!(words, String::from_utf8_lossy(&theirs.stderr).trim());
        assert!(words.starts_with("fatal: cannot change to"), "{words}");
    }

    #[test]
    fn an_exit_the_caller_allows_answers_its_output() {
        let script = "echo printed; echo warned >&2; exit 3";
        let ran = tool("sh")
            .output(None, &["-c", script], &[], &[0, 3])
            .unwrap();
        assert_eq!(ran.code, Some(3));
        assert_eq!(ran.stdout, b"printed\n");
        assert_eq!(ran.stderr, b"warned\n");
        assert_eq!(
            said(tool("sh").output(None, &["-c", script], &[], &[0])),
            "warned"
        );
        assert_eq!(
            said(tool("sh").output(None, &["-c", "exit 4"], &[], &[0])),
            "exit status: 4"
        );
    }

    #[test]
    fn the_variables_dropped_are_not_in_its_environment() {
        let ran = tool("sh")
            .output(
                Some(Path::new("/")),
                &["-c", "pwd; env"],
                &["LOGNAME", "HOM*"],
                &[0],
            )
            .unwrap();
        let printed = String::from_utf8(ran.stdout).unwrap();
        assert!(printed.starts_with("/\n"), "{printed}");
        for dropped in ["HOME=", "LOGNAME="] {
            assert!(
                !printed.lines().any(|l| l.starts_with(dropped)),
                "{printed}"
            );
        }
    }
}
