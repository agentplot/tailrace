//! Spec cache-push: a check job's push replays on a fresh host after the
//! landing (52, 135, 139; design D6). A clone runs a candidate's check
//! tasks in one `moon exec`, as a pool's run job does on a runner that
//! keeps nothing, and the built binary's `timber push --left` there,
//! bound by a file naming a layout and nothing else, writes what the run
//! left; the clone is deleted; sluice lands the candidate; and a fresh
//! clone bound to the same layout, sluice, a key and the job that tells
//! the gate publishes the landed revision, offering the bytes the first
//! clone's tasks wrote, with moon running none of them. With one archive gone before the push, the
//! publish is refused naming its output. The tasks write random bytes,
//! so bytes a task made again would differ. The hosts are homes and
//! configuration directories in the scratch directory, and the binary
//! runs with none of the `SLUICE_*` or `MOON_*` variables of a gate that
//! may be running these tests. Needs moon, sluice and ssh-keygen, so it
//! is in the tools layer.

#![cfg(feature = "tools")]

mod scratch;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use scratch::gate::Gate;
use scratch::recorder::Recorder;
use scratch::{git_command, moon_exec, policy_file, ssh_keygen, Repo, Scratch};
use serde_json::Value;
use timber::profiles::store::layout::Layout;
use timber_seams::{Digest, Held, Manifest, Store};

/// The identity the publisher signs as.
const PUBLISHER: &str = "publisher";

/// The two outputs, the project and the task that makes each, and the
/// file it leaves; each is packed as a file.
const OUTPUTS: [(&str, &str, &str); 2] = [
    ("notes", "notes:build", "notes/dist/notes.txt"),
    ("site", "site:make", "site/dist/site.txt"),
];

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

    /// `text` as the host's bindings file.
    fn bind(&self, text: &str) {
        let file = self.config.join("timber/config.toml");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, text).unwrap();
    }

    /// The built binary run in `repo` with `args`: its exit and the one
    /// JSON document it printed.
    fn timber(&self, repo: &Repo, args: &[&str]) -> (Option<i32>, Value) {
        let mut command = Command::new(env!("CARGO_BIN_EXE_timber"));
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
                "timber {args:?} printed no JSON document ({e}): {}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            )
        });
        (out.status.code(), json)
    }
}

/// An SSH key made by ssh-keygen at `path`, mode 0600.
fn keygen(path: &Path, comment: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let out = Command::new(ssh_keygen())
        .args(["-q", "-t", "ed25519", "-N", "", "-C", comment, "-f"])
        .arg(path)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
}

/// The status moon gave `target` in the run report at `report`.
fn status(report: &Path, target: &str) -> String {
    let report: Value = serde_json::from_slice(&fs::read(report).unwrap()).unwrap();
    report["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["label"] == format!("RunTask({target})"))
        .and_then(|a| a["status"].as_str())
        .unwrap_or_else(|| panic!("moon's run report holds no run of {target}: {report}"))
        .to_string()
}

/// The scratch world: a repository sluice gates on both tasks, a
/// candidate not yet submitted, and the layout every host binds.
struct Pool {
    scratch: Scratch,
    gate: Gate,
    candidate: Repo,
    layout: PathBuf,
}

impl Pool {
    /// The repository's two tasks, each writing random bytes to the file
    /// it declares, and the policy file declaring their outputs, packed
    /// as files, committed and pushed; sluice gating `main` on both; and
    /// a candidate unit off `main`.
    fn new() -> Pool {
        let scratch = Scratch::new();
        let repo = &scratch.repo;
        repo.git(&["config", "branch.main.remote", "origin"]);
        repo.git(&["config", "branch.main.merge", "refs/heads/main"]);
        let recorder = Recorder::new(repo, true, &scratch.path().join("keys"));
        repo.write(
            ".moon/workspace.yml",
            "projects:\n  notes: 'notes'\n  site: 'site'\n\
             vcs:\n  client: 'git'\n  defaultBranch: 'main'\ntelemetry: false\n",
        );
        for (_, target, file) in OUTPUTS {
            let (project, task) = target.split_once(':').unwrap();
            let name = Path::new(file).file_name().unwrap().to_str().unwrap();
            repo.write(
                &format!("{project}/moon.yml"),
                &format!(
                    "tasks:\n  {task}:\n    command: 'sh -c \"mkdir -p dist && \
                     od -An -N16 -tx1 /dev/urandom > dist/{name}\"'\n    \
                     inputs: ['moon.yml']\n    outputs: ['dist/{name}']\n    options: {{shell: false}}\n"
                ),
            );
        }
        let projections: String = OUTPUTS
            .iter()
            .map(|(output, _, _)| format!("{output} = \"file\"\n"))
            .collect();
        policy_file(
            repo,
            &OUTPUTS.map(|(output, task, _)| (output, task)),
            &format!("\n[projections]\n{projections}"),
        );
        repo.commit("feat: the notes and the site");
        repo.push();
        let gate = Gate::new(&scratch, &recorder, &OUTPUTS.map(|(_, task, _)| task));
        let candidate = gate.unit("x", &[("notes/x.txt", "x")]);
        let layout = scratch.path().join("layouts/shared");
        Pool {
            scratch,
            gate,
            candidate,
            layout,
        }
    }

    /// A clone at the candidate, both tasks run there in one `moon exec`,
    /// as a pool's run job runs the checks; the clone, and the bytes each
    /// output's task wrote, by output.
    fn job(&self) -> (Repo, Vec<(String, Vec<u8>)>) {
        let job = self.scratch.clone_as("job");
        job.git(&["fetch", "-q", self.candidate.path.to_str().unwrap(), "HEAD"]);
        job.git(&["checkout", "-q", "--detach", "FETCH_HEAD"]);
        moon_exec(&job.path, &OUTPUTS.map(|(_, task, _)| task));
        let wrote = OUTPUTS
            .iter()
            .map(|(output, _, file)| (output.to_string(), fs::read(job.path.join(file)).unwrap()))
            .collect();
        (job, wrote)
    }

    /// The built binary's `push --left` in `job`, bound by a file naming
    /// the layout alone; its exit and its answer.
    fn push_left(&self, job: &Repo) -> (Option<i32>, Value) {
        let host = Host::new(&self.scratch, "job");
        host.bind(&format!(
            "[store]\nlayout = {:?}\n",
            self.layout.display().to_string()
        ));
        host.timber(job, &["push", "--left", "--json"])
    }

    /// A fresh clone, bound to the layout, sluice on `origin`, a key of
    /// its own and the job that tells the gate, since the policy lets the
    /// publisher offer and names no acceptance (130), publishing `landed`;
    /// the clone, and the publish's exit and answer.
    fn publish(&self, landed: &str) -> (Repo, Option<i32>, Value) {
        let clone = self.scratch.clone_as("publisher");
        let host = Host::new(&self.scratch, PUBLISHER);
        keygen(&host.home.join(".config/timber/signing"), "host@publisher");
        host.bind(&format!(
            "[store]\nlayout = {:?}\n\n\
             [landing]\nsluice = {{ remote = \"origin\" }}\n\n\
             [signing]\nsigner = \"{PUBLISHER}\"\nkey = \"~/.config/timber/signing\"\n\n\
             [gate]\njob = \"the gate's job a push to main wakes\"\n",
            self.layout.display().to_string()
        ));
        let (code, published) = host.timber(&clone, &["publish", landed, "--json"]);
        (clone, code, published)
    }
}

/// The one component's answer of a publish.
fn component(json: &Value) -> &Value {
    assert_eq!(json["answer"], "ok", "{json}");
    let components = json["value"]["components"].as_array().unwrap();
    assert_eq!(components.len(), 1, "{json}");
    &components[0]
}

#[test]
fn a_check_jobs_push_is_published_from_the_store_by_a_fresh_clone() {
    let pool = Pool::new();
    let (job, wrote) = pool.job();
    let (code, pushed) = pool.push_left(&job);
    assert_eq!(code, Some(0), "{pushed}");
    let entries = pushed["value"]["entries"].as_array().unwrap();
    assert_eq!(entries.len(), OUTPUTS.len(), "{pushed}");
    for entry in entries {
        assert_eq!(entry["answer"], "ok", "{pushed}");
        assert_eq!(entry["value"]["held_at"], Value::Null, "{pushed}");
    }
    fs::remove_dir_all(&job.path).unwrap();

    let landed = pool.gate.submit(&pool.candidate);
    let (clone, code, published) = pool.publish(&landed);
    assert_eq!(code, Some(0), "{published}");
    let component = component(&published);
    assert_eq!(component["answer"], "ok", "{published}");
    let artifacts = component["value"]["artifacts"].as_array().unwrap();
    assert_eq!(artifacts.len(), OUTPUTS.len(), "{published}");

    // Each artifact's bytes are what the job's task wrote, read from the
    // store, and moon on the fresh clone ran no task's command.
    let layout = Layout::open(&pool.layout).unwrap();
    for artifact in artifacts {
        let output = artifact["output"].as_str().unwrap();
        let bytes = Digest::parse(artifact["bytes"].as_str().unwrap()).unwrap();
        let Ok(Held::Bytes(held)) = layout.read(&bytes) else {
            panic!("the layout holds no bytes at {bytes}")
        };
        let (_, written) = wrote.iter().find(|(o, _)| o == output).unwrap();
        assert_eq!(
            &held, written,
            "{output} offered bytes the job did not write"
        );
        let Ok(Held::Manifest(Manifest::Artifact { layers, .. })) =
            layout.read(&Digest::parse(artifact["digest"].as_str().unwrap()).unwrap())
        else {
            panic!("the layout holds no artifact for {output}")
        };
        assert_eq!(layers, [bytes]);
    }
    let report = clone
        .path
        .join(".git/timber/builds")
        .join(&landed)
        .join(".moon/cache/runReport.json");
    for (_, task, _) in OUTPUTS {
        assert_eq!(status(&report, task), "cached", "{task}");
    }
}

#[test]
fn an_archive_gone_before_the_push_refuses_its_output() {
    let pool = Pool::new();
    let (job, _) = pool.job();
    let report: Value =
        serde_json::from_slice(&fs::read(job.path.join(".moon/cache/runReport.json")).unwrap())
            .unwrap();
    let hash = report["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["label"] == "RunTask(site:make)")
        .and_then(|a| {
            a["operations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|o| o["meta"]["type"] == "hash-generation")
        })
        .and_then(|o| o["meta"]["hash"].as_str())
        .unwrap()
        .to_string();
    fs::remove_file(
        job.path
            .join(".moon/cache/outputs")
            .join(format!("{hash}.tar.gz")),
    )
    .unwrap();
    let (code, pushed) = pool.push_left(&job);
    assert_eq!(code, Some(0), "{pushed}");
    let entries = pushed["value"]["entries"].as_array().unwrap();
    assert!(
        entries.iter().all(|e| e["build"] != hash.as_str()),
        "the build whose archive is gone was pushed: {pushed}"
    );
    fs::remove_dir_all(&job.path).unwrap();

    let landed = pool.gate.submit(&pool.candidate);
    let (_, code, published) = pool.publish(&landed);
    assert_eq!(code, Some(1), "{published}");
    let component = component(&published);
    assert_eq!(component["answer"], "refused", "{published}");
    let why = &component["refused"]["why"]["not_in_store"];
    assert_eq!(why["output"], "site", "{published}");
    assert_eq!(why["build"], hash.as_str(), "{published}");
    let names: Vec<String> =
        serde_json::from_slice::<Value>(&fs::read(pool.layout.join("index.json")).unwrap())
            .unwrap()["manifests"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|m| m["annotations"]["org.opencontainers.image.ref.name"].as_str())
            .map(str::to_string)
            .collect();
    assert!(
        names.iter().all(|n| n.starts_with("cache/")),
        "the refused publish named {names:?}"
    );
}
