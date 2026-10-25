//! Specs cache-push and host-bindings: a landing on one host replays on
//! another (52, C.3; design D6). sluice lands a revision whose checks are
//! two moon tasks, one declaring an output and one declaring none, in
//! places whose moon outputs directory its toolchain links under the
//! repository's common directory, as this repository's devenv shell
//! does; the built binary's `tailrace push` on that host, bound by a
//! file naming a layout not yet made, sluice and no `[signing]`, writes
//! both tasks' archives as cache entries; and a clone on a second host,
//! bound to the same layout and sluice, syncs the revision, after which
//! moon answers both tasks from the cache with the first host's bytes.
//! The hosts are two homes and two configuration directories in the
//! scratch directory, and the binary and moon run with none of the
//! `SLUICE_*` or `MOON_*` variables of a gate that may be running these
//! tests. Needs moon and sluice, so it is in the tools layer.

#![cfg(feature = "tools")]

mod scratch;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use scratch::gate::{Gate, LINK_OUTPUTS};
use scratch::recorder::Recorder;
use scratch::{git, git_command, moon_hash, moon_workspace, Repo, Scratch, Task};
use serde_json::{json, Value};
use tailrace::profiles::store::layout::Layout;
use tailrace_seams::{Digest, Name, Store};

/// The task that declares an output, and the file it leaves.
const SITE: &str = "site:make";
const OUTPUT: &str = "site/dist/site.txt";

/// The task that declares none.
const LINT: &str = "site:lint";

/// A host as the binary sees it: a home and a configuration directory
/// of its own, in the scratch directory.
struct Host {
    home: PathBuf,
    config: PathBuf,
}

impl Host {
    fn new(scratch: &Scratch, name: &str) -> Host {
        let host = Host {
            home: scratch.path().join(format!("{name}-home")),
            config: scratch.path().join(format!("{name}-config")),
        };
        fs::create_dir_all(&host.home).unwrap();
        host
    }

    /// A bindings file naming the layout at `layout` and sluice with the
    /// remote `origin`, and no `[signing]`.
    fn bind(&self, layout: &Path) {
        let file = self.config.join("tailrace/config.toml");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(
            file,
            format!(
                "[store]\nlayout = {:?}\n\n[landing]\nsluice = {{ remote = \"origin\" }}\n",
                layout.display().to_string()
            ),
        )
        .unwrap();
    }

    /// The built binary run in `repo` with `args`: its exit and the one
    /// JSON document it printed.
    fn tailrace(&self, repo: &Repo, args: &[&str]) -> (Option<i32>, Value) {
        let mut command = Command::new(env!("CARGO_BIN_EXE_tailrace"));
        for (name, _) in std::env::vars_os() {
            let name = name.to_string_lossy().to_string();
            if name.starts_with("SLUICE_") || name.starts_with("MOON_") {
                command.env_remove(&name);
            }
        }
        // git as the scratch repositories are made: none of the host's
        // configuration, a fixed author.
        for (name, value) in git_command().get_envs() {
            match value {
                Some(value) => command.env(name, value),
                None => command.env_remove(name),
            };
        }
        let out = command
            .args(args)
            .current_dir(&repo.path)
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", &self.config)
            .output()
            .expect("the binary runs");
        let json = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
            panic!(
                "tailrace {args:?} printed no JSON document ({e}): {}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            )
        });
        (out.status.code(), json)
    }
}

/// The status moon gave `target` in the run report it left in `dir`.
fn status(dir: &Path, target: &str) -> String {
    let report: Value =
        serde_json::from_slice(&fs::read(dir.join(".moon/cache/runReport.json")).unwrap()).unwrap();
    report["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["label"] == format!("RunTask({target})"))
        .and_then(|a| a["status"].as_str())
        .unwrap_or_else(|| panic!("moon's run report holds no run of {target}: {report}"))
        .to_string()
}

/// `checkout`'s moon outputs directory linked as the gate's places link
/// theirs, by the same command.
fn link_outputs(checkout: &Path) {
    let [program, args @ ..] = LINK_OUTPUTS;
    let out = git_command()
        .get_envs()
        .fold(Command::new(program), |mut command, (name, value)| {
            match value {
                Some(value) => command.env(name, value),
                None => command.env_remove(name),
            };
            command
        })
        .args(args)
        .current_dir(checkout)
        .output()
        .expect("the shell runs");
    assert!(out.status.success(), "{out:?}");
}

/// Each entry a push or a sync answered, by build identity, with the
/// value it answered; panics naming the entry that is no answer.
fn answered(json: &Value) -> Vec<(String, Value)> {
    assert_eq!(json["answer"], "ok", "{json}");
    json["value"]["entries"]
        .as_array()
        .unwrap_or_else(|| panic!("no entries: {json}"))
        .iter()
        .map(|e| {
            assert_eq!(e["answer"], "ok", "{} answered {e}", e["build"]);
            (e["build"].as_str().unwrap().to_string(), e["value"].clone())
        })
        .collect()
}

#[test]
fn a_landing_pushed_on_its_host_replays_on_another_host() {
    let scratch = Scratch::new();
    let repo = &scratch.repo;
    repo.git(&["config", "branch.main.remote", "origin"]);
    repo.git(&["config", "branch.main.merge", "refs/heads/main"]);
    let recorder = Recorder::new(repo, true, &scratch.path().join("keys"));
    moon_workspace(
        repo,
        &[
            Task::new("site", "make", "site.txt", true, 0),
            Task::without_output("site", "lint", 0),
        ],
    );
    repo.commit("feat: the site, made and linted");
    repo.push();
    let gate = Gate::with_toolchain(&scratch, &recorder, &[SITE, LINT], &LINK_OUTPUTS);
    // Host A's own checkout reads the archives where the places left
    // them.
    link_outputs(&repo.path);
    let landed = gate.land("x", &[("notes/x.txt", "x")]);
    let common = PathBuf::from(git(
        &repo.path,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    ));
    let outputs = common.join("moon-outputs");

    // Host A pushes the landing: its local main is the landing, and the
    // layout its file names is made by the push.
    let layout = scratch.path().join("layouts/shared");
    let a = Host::new(&scratch, "a");
    a.bind(&layout);
    assert!(!layout.exists());
    let (code, pushed) = a.tailrace(repo, &["push", "--json"]);
    assert_eq!(code, Some(0), "{pushed}");
    assert_eq!(pushed["value"]["revision"], landed.as_str(), "{pushed}");
    let entries = answered(&pushed);
    assert_eq!(entries.len(), 2, "{pushed}");
    let store = Layout::open(&layout).expect("the push made the layout");
    let mut builds = BTreeSet::new();
    for (build, written) in &entries {
        assert_eq!(written["held_at"], Value::Null, "{pushed}");
        assert_eq!(written["projection"], "moon-cache", "{pushed}");
        let digest = Digest::parse(written["entry"]["digest"].as_str().unwrap()).unwrap();
        assert_eq!(
            store.resolve(&Name::new(format!("cache/{build}"))),
            Ok(digest.clone()),
            "the layout names no entry of {build}"
        );
        let archive = fs::read(outputs.join(format!("{build}.tar.gz"))).unwrap();
        assert_eq!(Digest::of(&archive), digest, "{build} pushed other bytes");
        builds.insert(build.clone());
    }

    // Host B: a clone of its own, its own common directory, a file naming
    // the same layout and sluice.
    let clone = scratch.clone_as("second");
    assert!(!clone.path.join(".moon/cache").exists());
    let b = Host::new(&scratch, "b");
    b.bind(&layout);
    let (code, synced) = b.tailrace(&clone, &["sync", &landed, "--json"]);
    assert_eq!(code, Some(0), "{synced}");
    let filled = answered(&synced);
    let rendered: BTreeSet<String> = filled.iter().map(|(b, _)| b.clone()).collect();
    assert_eq!(rendered, builds, "{synced}");
    for (build, value) in &filled {
        assert_eq!(
            value["rendered"],
            json!(["moon-cache"]),
            "{build}: {synced}"
        );
    }

    // moon on host B answers both tasks from the cache, with host A's
    // bytes.
    for target in [SITE, LINT] {
        let hash = moon_hash(&clone.path, target);
        assert!(builds.contains(&hash), "{target} ran at {hash}, not pushed");
        assert_eq!(status(&clone.path, target), "cached", "{target}");
        assert_eq!(
            fs::read(
                clone
                    .path
                    .join(".moon/cache/outputs")
                    .join(format!("{hash}.tar.gz"))
            )
            .unwrap(),
            fs::read(outputs.join(format!("{hash}.tar.gz"))).unwrap(),
            "{target}'s archive on host B is not host A's"
        );
    }
    assert_eq!(fs::read(clone.path.join(OUTPUT)).unwrap(), SITE.as_bytes());
}
