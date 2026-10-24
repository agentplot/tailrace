//! Spec host-bindings: the laptop end to end, S1 and S2 played by the
//! built binary bound by a bindings file, over a scratch repository that
//! sluice gates and whose builds moon runs, then a second clone with a
//! copy of the layout and no key verifying what the laptop published
//! (S1, S2, 80, 118; design D8). The binary runs with its home and its
//! configuration in the scratch directory, and none of the `SLUICE_*` or
//! `MOON_*` variables of a gate that may be running these tests. Needs
//! moon, sluice and ssh-keygen, so it is in the tools layer.

#![cfg(feature = "tools")]

mod fixtures;
mod scratch;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use scratch::gate::{Gate, SIGNER};
use scratch::recorder::Recorder;
use scratch::{git_command, moon_workspace, policy_file, ssh_keygen, Repo, Scratch, Task};
use serde_json::{json, Value};
use tailrace::profiles::store::layout::Layout;
use tailrace_core::{History, Kept, Provenance};
use tailrace_seams::{Claim, Digest, Store};

/// The identity the laptop signs as.
const LAPTOP: &str = "laptop";

/// The two outputs, the task that makes each, and the projection that
/// packs it.
const OUTPUTS: [(&str, &str, &str); 2] = [
    ("notes", "notes:build", "file"),
    ("image", "image:build", "image"),
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
        let file = self.config.join("tailrace/config.toml");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, text).unwrap();
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

/// An SSH key made by ssh-keygen at `path`, mode 0600; answers its
/// public half as a policy lists it.
fn keygen(path: &Path, comment: &str) -> String {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let out = Command::new(ssh_keygen())
        .args(["-q", "-t", "ed25519", "-N", "", "-C", comment, "-f"])
        .arg(path)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::read_to_string(path.with_extension("pub"))
        .unwrap()
        .trim()
        .to_string()
}

/// Every file under `dir`, by its path there, with its size and its
/// bytes' digest.
fn listing(dir: &Path) -> BTreeMap<PathBuf, (u64, Digest)> {
    let mut files = BTreeMap::new();
    let mut dirs = vec![dir.to_path_buf()];
    while let Some(at) = dirs.pop() {
        for entry in fs::read_dir(&at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
            } else {
                let bytes = fs::read(&path).unwrap();
                let rel = path.strip_prefix(dir).unwrap().to_path_buf();
                files.insert(rel, (bytes.len() as u64, Digest::of(&bytes)));
            }
        }
    }
    files
}

/// `from` copied to `to`, every file and directory under it.
fn copy(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        let into = to.join(path.file_name().unwrap());
        if path.is_dir() {
            copy(&path, &into);
        } else {
            fs::copy(&path, &into).unwrap();
        }
    }
}

/// The scratch world: a repository sluice gates, the laptop's host, its
/// key, and the revision sluice landed.
struct Laptop {
    scratch: Scratch,
    host: Host,
    layout: PathBuf,
    landed: String,
}

impl Laptop {
    /// The repository's moon tasks and policy file, committed and pushed;
    /// sluice gating `main` on both tasks lands a submission; and the
    /// laptop's bindings file names a fresh layout, sluice with the
    /// remote, and the laptop's key.
    fn new() -> Laptop {
        let scratch = Scratch::new();
        let repo = &scratch.repo;
        // The shared history the version's name is claimed in (22).
        repo.git(&["config", "branch.main.remote", "origin"]);
        repo.git(&["config", "branch.main.merge", "refs/heads/main"]);

        let host = Host::new(&scratch, LAPTOP);
        let key = host.home.join(".config/tailrace/signing");
        let laptop_key = keygen(&key, "host@laptop");
        let recorder = Recorder::new(repo, true, &scratch.path().join("keys"));

        moon_workspace(
            repo,
            &[
                Task::new("notes", "build", "notes.txt", true, 0),
                Task::new("image", "build", "image.tar", true, 0),
            ],
        );
        // The image task copies the archive the repository holds.
        repo.write(
            "image/moon.yml",
            "tasks:\n  build:\n    command: 'sh -c \"mkdir -p dist && cp image.tar dist/image.tar\"'\n    \
             inputs: ['moon.yml', 'image.tar']\n    outputs: ['dist/image.tar']\n    options: {shell: false}\n",
        );
        fs::write(
            repo.path.join("image/image.tar"),
            fixtures::image::archive(),
        )
        .unwrap();
        let projections: String = OUTPUTS
            .iter()
            .map(|(output, _, ecosystem)| format!("{output} = \"{ecosystem}\"\n"))
            .collect();
        let checks: Vec<String> = OUTPUTS.iter().map(|(_, t, _)| format!("\"{t}\"")).collect();
        policy_file(
            repo,
            &OUTPUTS.map(|(output, task, _)| (output, task)),
            &format!(
                "\n[projections]\n{projections}\n\
                 [signers.{LAPTOP}]\nssh = \"{laptop_key}\"\nkinds = [\"provenance\"]\n\n\
                 [signers.{SIGNER}]\nssh = \"{}\"\nkinds = [\"verdict\"]\nchecks = [{}]\n\n\
                 [acceptance]\nrequire = [{{ kind = \"provenance\", signers = [\"{LAPTOP}\"] }}]\n",
                recorder.public_key(SIGNER),
                checks.join(", ")
            ),
        );
        repo.commit("feat: the notes and the image");
        repo.push();

        let gate = Gate::new(&scratch, &recorder, &OUTPUTS.map(|(_, task, _)| task));
        let landed = gate.land("x", &[("notes/x.txt", "x")]);

        let layout = host.home.join(".local/share/tailrace/layout");
        host.bind(&format!(
            "[store]\nlayout = \"~/.local/share/tailrace/layout\"\n\n\
             [landing]\nsluice = {{ remote = \"origin\" }}\n\n\
             [signing]\nsigner = \"{LAPTOP}\"\nkey = \"~/.config/tailrace/signing\"\n"
        ));
        Laptop {
            scratch,
            host,
            layout,
            landed,
        }
    }

    fn publish(&self) -> (Option<i32>, Value) {
        self.host
            .tailrace(&self.scratch.repo, &["publish", &self.landed, "--json"])
    }

    /// The host's record of the versions it published.
    fn record(&self) -> PathBuf {
        let history = History::open(&self.scratch.repo.path).unwrap();
        Kept::at(&history).dir().to_path_buf()
    }
}

/// The one component's published version, from publish's answer.
fn published(json: &Value) -> &Value {
    assert_eq!(json["answer"], "ok", "{json}");
    let components = json["value"]["components"].as_array().unwrap();
    assert_eq!(components.len(), 1, "{json}");
    assert_eq!(components[0]["answer"], "ok", "{json}");
    let value = &components[0]["value"];
    assert_eq!(value["published"], "version", "{json}");
    value
}

#[test]
fn the_laptop_publishes_a_sluice_landing_and_a_second_machine_verifies_a_copy() {
    let laptop = Laptop::new();

    // S1: the landed revision offered, each artifact with its provenance
    // signed by the laptop and sluice's verdicts carried in it.
    let (code, first) = laptop.publish();
    assert_eq!(code, Some(0), "{first}");
    let version = published(&first);
    assert_eq!(version["version"], "0.1.0", "{first}");
    assert_eq!(version["tag"], "v0.1.0");
    assert_eq!(version["revision"], laptop.landed.as_str());
    let index = version["index"].as_str().unwrap().to_string();
    let artifacts = version["artifacts"].as_array().unwrap();
    let packed: BTreeSet<(&str, &str)> = artifacts
        .iter()
        .map(|a| {
            (
                a["ecosystem"].as_str().unwrap(),
                a["name"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        packed,
        BTreeSet::from([("file", "notes"), ("image", "image")]),
        "{first}"
    );
    let layout = Layout::open(&laptop.layout).expect("the publish made the layout");
    for artifact in artifacts {
        let digest = Digest::parse(artifact["digest"].as_str().unwrap()).unwrap();
        let claims = artifact["claims"].as_array().unwrap();
        assert_eq!(claims.len(), 1, "{artifact}");
        assert_eq!(claims[0]["kind"], "provenance");
        assert_eq!(claims[0]["signer"], LAPTOP);
        let held = layout.claims_of(&digest).unwrap();
        let [envelope] = held.as_slice() else {
            panic!("{} claims are attached to {digest}", held.len())
        };
        assert_eq!(envelope.sole().map(|s| s.signer.as_str()), Some(LAPTOP));
        let provenance = Claim::from_envelope(envelope)
            .as_ref()
            .and_then(Provenance::of)
            .expect("a provenance of tailrace's form");
        let task = OUTPUTS
            .iter()
            .find(|(output, _, _)| *output == provenance.output.as_str())
            .map(|(_, task, _)| *task)
            .unwrap();
        assert!(!provenance.carried.is_empty(), "no verdict was carried");
        for carried in &provenance.carried {
            assert_eq!(carried.check.as_str(), task);
            assert_eq!(carried.signer.as_str(), SIGNER);
            assert_eq!(carried.build.as_str(), artifact["build"].as_str().unwrap());
        }
    }
    let named: Vec<String> =
        serde_json::from_slice::<Value>(&fs::read(laptop.layout.join("index.json")).unwrap())
            .unwrap()["manifests"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|m| m["annotations"]["org.opencontainers.image.ref.name"].as_str())
            .map(str::to_string)
            .collect();
    assert_eq!(named, ["v0.1.0"], "the layout names another ref, a channel");

    // The host's record names the store as the bindings file names it,
    // the layout's path with ~/ expanded (61, 118).
    let store = json!({
        "seam": "store",
        "profile": format!("layout {}", laptop.layout.display()),
    });
    assert_eq!(version["store"], store, "{first}");
    let (code, status) = laptop
        .host
        .tailrace(&laptop.scratch.repo, &["status", "--json"]);
    assert_eq!(code, Some(0), "{status}");
    let entry = &status["value"]["components"][0]["versions"][0];
    assert_eq!(entry["version"], "0.1.0", "{status}");
    assert_eq!(entry["store"], store, "{status}");

    // S2: the same revision again answers the same index, and no file of
    // the layout or the record changed.
    let before = (listing(&laptop.layout), listing(&laptop.record()));
    assert!(!before.1.is_empty(), "the host recorded nothing");
    let (code, again) = laptop.publish();
    assert_eq!(code, Some(0), "{again}");
    assert_eq!(published(&again)["index"], index.as_str(), "{again}");
    assert_eq!(
        (listing(&laptop.layout), listing(&laptop.record())),
        before,
        "the repeat changed a file of the layout or the record"
    );

    // A second machine: a clone, a copy of the layout, and a bindings
    // file naming the copy and sluice and no key.
    let clone = laptop.scratch.clone_as("second");
    let copied = laptop.scratch.path().join("layouts/copied");
    copy(&laptop.layout, &copied);
    let second = Host::new(&laptop.scratch, "second");
    second.bind(&format!(
        "[store]\nlayout = {:?}\n\n[landing]\nsluice = {{ remote = \"origin\" }}\n",
        copied.display().to_string()
    ));
    let (code, verified) = second.tailrace(&clone, &["verify", "0.1.0", "--json"]);
    assert_eq!(code, Some(0), "{verified}");
    assert_eq!(verified["answer"], "ok", "{verified}");
    let value = &verified["value"];
    assert_eq!(value["index"], index.as_str());
    let on: BTreeSet<&str> = artifacts
        .iter()
        .map(|a| a["digest"].as_str().unwrap())
        .collect();
    let told = value["claims"].as_array().unwrap();
    let provenances: Vec<&Value> = told.iter().filter(|t| t["kind"] == "provenance").collect();
    assert_eq!(provenances.len(), artifacts.len(), "{verified}");
    for provenance in &provenances {
        assert!(
            on.contains(provenance["on"].as_str().unwrap()),
            "{provenance}"
        );
        assert_eq!(provenance["made_by"], LAPTOP, "{provenance}");
        assert_eq!(provenance["counts"], true, "{provenance}");
    }
    let verdicts: Vec<&Value> = told.iter().filter(|t| t["kind"] == "verdict").collect();
    assert!(verdicts.len() >= artifacts.len(), "{verified}");
    for verdict in verdicts {
        let carrier = provenances
            .iter()
            .find(|p| p["envelope"] == verdict["carried_in"])
            .unwrap_or_else(|| panic!("{verdict} names no provenance it was carried in"));
        assert_eq!(carrier["on"], verdict["on"], "{verdict}");
    }
    assert_eq!(
        listing(&copied),
        before.0,
        "verify wrote to the copied layout"
    );
}
