//! sluice itself, gating a scratch repository's `main` (design D9): the
//! branch's policy requires moon tasks and trusts the signer `box`, the
//! host signs as `box`, and a submission is landed by `sluice submit`
//! and `sluice step`, as a host lands one, each place prepared by the
//! toolchain a test gives it, if any. sluice runs with a home, a
//! configuration, a state and a cache of its own in the scratch
//! directory, and none of the `SLUICE_*` or `MOON_*` variables of a gate
//! that may be running these tests.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use super::recorder::Recorder;
use super::{git_command, sluice, Repo, Scratch, MAIN};

/// The signer the gate signs as.
pub const SIGNER: &str = "box";

/// A toolchain that links the place's moon outputs directory to
/// `moon-outputs` under the repository's common directory and prints
/// nothing, as this repository's devenv shell links each checkout's, so
/// the archives of a landing's checks outlive its place.
pub const LINK_OUTPUTS: [&str; 3] = [
    "sh",
    "-c",
    "root=\"$(git rev-parse --show-toplevel)\" && \
     common=\"$(git rev-parse --path-format=absolute --git-common-dir)\" && \
     mkdir -p \"$common/moon-outputs\" \"$root/.moon/cache\" && \
     if [ ! -L \"$root/.moon/cache/outputs\" ]; then \
     rm -rf \"$root/.moon/cache/outputs\" && \
     ln -s \"$common/moon-outputs\" \"$root/.moon/cache/outputs\"; fi",
];

/// sluice gating a scratch repository.
pub struct Gate {
    repo: Repo,
    home: PathBuf,
    sessions: PathBuf,
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

impl Gate {
    /// sluice over `scratch`'s repository, whose `main` on `origin`
    /// requires `required` and trusts `box`, committed and pushed; the
    /// host's configuration names `box` and the key `recorder` keeps for
    /// it. The repository's moon workspace is the caller's.
    pub fn new(scratch: &Scratch, recorder: &Recorder, required: &[&str]) -> Gate {
        Gate::with_toolchain(scratch, recorder, required, &[])
    }

    /// The gate of [`Gate::new`], sluice running `toolchain` in each place
    /// before its run where it names a command.
    pub fn with_toolchain(
        scratch: &Scratch,
        recorder: &Recorder,
        required: &[&str],
        toolchain: &[&str],
    ) -> Gate {
        let home = scratch.path().join("sluice-home");
        for dir in ["home", "config/sluice", "state", "cache", "data"] {
            fs::create_dir_all(home.join(dir)).unwrap();
        }
        let (key, _) = recorder.key(SIGNER);
        fs::write(
            home.join("config/sluice/config.toml"),
            format!(
                "[signing]\nsigner = \"{SIGNER}\"\nkey = {:?}\n\n[peers]\nname = \"box-host\"\n",
                key.display().to_string()
            ),
        )
        .unwrap();
        let required: Vec<String> = required.iter().map(|r| format!("\"{r}\"")).collect();
        let toolchain = match toolchain {
            [] => String::new(),
            parts => {
                let parts: Vec<String> = parts.iter().map(|p| format!("{p:?}")).collect();
                format!("toolchain = [{}]\n", parts.join(", "))
            }
        };
        let repo = scratch.repo.clone();
        repo.write(
            ".config/sluice.toml",
            &format!(
                "[tests]\nrunner = \"moon\"\n{toolchain}\n\
                 [signers.{SIGNER}]\nssh = \"{SIGNER} {}\"\n\n\
                 [branches.main]\nrequired = [{}]\ntimeout = \"5m\"\nremote = \"origin\"\ntrust = [\"{SIGNER}\"]\n",
                recorder.public_key(SIGNER),
                required.join(", ")
            ),
        );
        repo.commit("chore: sluice gates main");
        repo.push();
        let sessions = scratch.path().join("sessions");
        fs::create_dir_all(&sessions).unwrap();
        Gate {
            repo,
            home,
            sessions,
        }
    }

    /// sluice in `dir`, as the scratch host runs it.
    fn sluice(&self, dir: &Path, args: &[&str]) -> Output {
        let mut command = Command::new(sluice());
        for (name, _) in std::env::vars_os() {
            let name = name.to_string_lossy().to_string();
            if name.starts_with("SLUICE_") || name.starts_with("MOON_") {
                command.env_remove(&name);
            }
        }
        // git as the scratch repositories are made: none of the host's
        // configuration, a fixed author; the home is sluice's own.
        let git = git_command();
        for (name, value) in git.get_envs() {
            match value {
                Some(value) => command.env(name, value),
                None => command.env_remove(name),
            };
        }
        command
            .args(args)
            .current_dir(dir)
            .env("HOME", self.home.join("home"))
            .env("XDG_CONFIG_HOME", self.home.join("config"))
            .env("XDG_STATE_HOME", self.home.join("state"))
            .env("XDG_CACHE_HOME", self.home.join("cache"))
            .env("XDG_DATA_HOME", self.home.join("data"))
            .stdin(Stdio::null());
        command.output().expect("sluice runs")
    }

    /// A session's unit off `main` writing `files`, submitted to `main`
    /// and stepped until sluice landed it; answers the landing commit,
    /// `main`'s head on the remote.
    pub fn land(&self, name: &str, files: &[(&str, &str)]) -> String {
        let dir = self.sessions.join(name);
        self.repo.git(&["fetch", "-q", "origin"]);
        self.repo.git(&[
            "worktree",
            "add",
            "-q",
            "-b",
            &format!("unit/{name}"),
            dir.to_str().unwrap(),
            "origin/main",
        ]);
        let unit = Repo { path: dir.clone() };
        for (file, contents) in files {
            unit.write(file, contents);
        }
        unit.commit(&format!("feat: {name}"));
        let out = self.sluice(&dir, &["submit", MAIN, "--no-wait", "--json"]);
        assert!(out.status.success(), "{}", text(&out));
        let submitted: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        let id = submitted["id"].as_u64().unwrap().to_string();
        let deadline = Instant::now() + Duration::from_secs(180);
        loop {
            let out = self.sluice(&self.repo.path, &["step", MAIN, "--json"]);
            assert!(out.status.success(), "{}", text(&out));
            let out = self.sluice(&self.repo.path, &["status", &id, "--json"]);
            assert!(out.status.success(), "{}", text(&out));
            let status: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
            match status["state"].as_str() {
                Some("landed") => break,
                Some("failed" | "conflict" | "withdrawn") => {
                    panic!("sluice did not land the submission: {status}")
                }
                _ => {}
            }
            assert!(Instant::now() < deadline, "sluice never landed: {status}");
            std::thread::sleep(Duration::from_millis(200));
        }
        self.repo.git(&["fetch", "-q", "origin"]);
        self.repo.git(&["merge", "-q", "--ff-only", "origin/main"]);
        self.repo.git(&["rev-parse", "origin/main"])
    }
}
