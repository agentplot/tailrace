//! Spec host-bindings: the laptop end to end, S1 and S2 played by the
//! built binary bound by a bindings file, over a scratch repository that
//! sluice gates and whose builds moon runs, then a second clone with a
//! copy of the layout and no key verifying what the laptop published
//! (S1, S2, 80, 118; design D8); and a laptop with nothing written, which
//! publishes with the key its first publish makes and, once its policy
//! lists that key for provenance alone, accepts as one host and moves the
//! given `beta` in the same run (95, 108, 118, C.10). The binary runs
//! with its home and its configuration and data directories in the
//! scratch directory, and none of the `SLUICE_*` or `MOON_*` variables of
//! a gate that may be running these tests. Needs moon, sluice and
//! ssh-keygen, so it is in the tools layer.

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
use scratch::{
    git_command, moon_workspace, policy_file, ssh_keygen, Repo, Scratch, Task, POLICY_FILE,
};
use serde_json::{json, Value};
use tailrace::profiles::ledger::git_ref::GitRef;
use tailrace::profiles::store::layout::Layout;
use tailrace_core::{History, Kept, Provenance};
use tailrace_seams::{ChannelId, ChannelName, Claim, Digest, Keeper, Promotion, Store};

/// The identity the laptop signs as.
const LAPTOP: &str = "laptop";

/// The two outputs, the task that makes each, and the projection that
/// packs it.
const OUTPUTS: [(&str, &str, &str); 2] = [
    ("notes", "notes:build", "file"),
    ("image", "image:build", "image"),
];

/// A host as the binary sees it: a home and a configuration directory
/// of its own, in the scratch directory, and a data directory where it
/// names one.
struct Host {
    home: PathBuf,
    config: PathBuf,
    data: Option<PathBuf>,
}

impl Host {
    fn new(scratch: &Scratch, name: &str) -> Host {
        let host = Host {
            home: scratch.path().join(format!("{name}-home")),
            config: scratch.path().join(format!("{name}-config")),
            data: None,
        };
        fs::create_dir_all(&host.home).unwrap();
        host
    }

    /// A host with a home in the scratch directory and its configuration
    /// and data directories under it, holding nothing.
    fn fresh(scratch: &Scratch, name: &str) -> Host {
        let home = scratch.path().join(format!("{name}-home"));
        fs::create_dir_all(&home).unwrap();
        Host {
            config: home.join("config"),
            data: Some(home.join("data")),
            home,
        }
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
        if let Some(data) = &self.data {
            command.env("XDG_DATA_HOME", data);
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

/// The repository's moon tasks, the image's archive and a policy file
/// naming the outputs and their projections with `tables` after them,
/// committed and pushed, and sluice gating `main` on both tasks, its
/// landing's verdicts signed by `recorder`'s key for [`SIGNER`].
fn gated(scratch: &Scratch, recorder: &Recorder, tables: &str) -> Gate {
    let repo = &scratch.repo;
    // The shared history the version's name is claimed in (22).
    repo.git(&["config", "branch.main.remote", "origin"]);
    repo.git(&["config", "branch.main.merge", "refs/heads/main"]);
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
    policy_file(
        repo,
        &OUTPUTS.map(|(output, task, _)| (output, task)),
        &format!("\n[projections]\n{projections}\n{tables}"),
    );
    repo.commit("feat: the notes and the image");
    repo.push();
    Gate::new(scratch, recorder, &OUTPUTS.map(|(_, task, _)| task))
}

impl Laptop {
    /// The gated repository, its policy naming the laptop's key for
    /// provenance, the landing's verdicts, and a gate of its own for
    /// acceptance; sluice lands a submission; and the laptop's bindings
    /// file names a fresh layout, sluice with the remote, the laptop's
    /// key, and the job that tells the gate, since the policy lets the
    /// laptop offer and not accept (130, C.16).
    fn new() -> Laptop {
        let scratch = Scratch::new();
        let host = Host::new(&scratch, LAPTOP);
        let key = host.home.join(".config/tailrace/signing");
        let laptop_key = keygen(&key, "host@laptop");
        let gate_key = keygen(&scratch.path().join("gate-key/signing"), "gate@box");
        let recorder = Recorder::new(&scratch.repo, true, &scratch.path().join("keys"));
        let checks: Vec<String> = OUTPUTS.iter().map(|(_, t, _)| format!("\"{t}\"")).collect();
        let gate = gated(
            &scratch,
            &recorder,
            &format!(
                "[signers.{LAPTOP}]\nssh = \"{laptop_key}\"\nkinds = [\"provenance\"]\n\n\
                 [signers.{SIGNER}]\nssh = \"{}\"\nkinds = [\"verdict\"]\nchecks = [{}]\n\n\
                 [signers.gate]\nssh = \"{gate_key}\"\nkinds = [\"acceptance\", \"promotion\"]\n\n\
                 [acceptance]\nrequire = [{{ kind = \"provenance\", signers = [\"{LAPTOP}\"] }}]\n",
                recorder.public_key(SIGNER),
                checks.join(", ")
            ),
        );
        let landed = gate.land("x", &[("notes/x.txt", "x")]);

        let layout = host.home.join(".local/share/tailrace/layout");
        host.bind(&format!(
            "[store]\nlayout = \"~/.local/share/tailrace/layout\"\n\n\
             [landing]\nsluice = {{ remote = \"origin\" }}\n\n\
             [signing]\nsigner = \"{LAPTOP}\"\nkey = \"~/.config/tailrace/signing\"\n\n\
             [gate]\njob = \"the gate's job a push to main wakes\"\n"
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
    let entry = &status["value"]["published"]["value"]["components"][0]["versions"][0];
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

// Requirement: A laptop with nothing written publishes and moves beta

/// The provenance signers of every artifact in publish's answer.
fn provenance_signers(version: &Value) -> BTreeSet<String> {
    version["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|a| a["claims"].as_array().unwrap())
        .filter(|c| c["kind"] == "provenance")
        .map(|c| c["signer"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn a_laptop_with_nothing_written_publishes_and_moves_beta_once_its_made_key_is_listed() {
    let scratch = Scratch::new();
    let host = Host::fresh(&scratch, "fresh");
    let data = host.data.clone().unwrap();
    let recorder = Recorder::new(&scratch.repo, true, &scratch.path().join("keys"));
    // A policy that lists no signer and requires nothing of acceptance.
    let gate = gated(&scratch, &recorder, "");
    let repo = &scratch.repo;
    assert_eq!(repo.git(&["remote"]), "origin");

    // The first publish on the fresh laptop makes the layout and the key,
    // both in the data directory, and nothing in the configuration's.
    let landed = gate.land("x", &[("notes/x.txt", "x")]);
    let (_, first) = host.tailrace(repo, &["publish", &landed, "--json"]);
    let version = published(&first);
    assert!(data.join("tailrace/layout/oci-layout").is_file(), "{first}");
    let key = data.join("tailrace/host/signing.key");
    assert_eq!(
        fs::metadata(&key).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let line = fs::read_to_string(data.join("tailrace/host/signing.key.pub"))
        .unwrap()
        .trim()
        .to_string();
    let name = line.rsplit(' ').next().unwrap().to_string();
    assert!(
        !host.config.exists(),
        "the publish wrote under XDG_CONFIG_HOME"
    );
    assert_eq!(provenance_signers(version), BTreeSet::from([name.clone()]));
    let told = &first["value"]["components"][0]["gate"];
    assert_eq!(told["told_by"], "publish", "{first}");
    assert!(
        told["answer"] != "ok" || told["value"]["judged"] == "refused",
        "the gate accepted a version the policy does not list the host for: {told}"
    );

    // The policy lists the made key's line for provenance alone, names no
    // other signer and writes no channel; a change to the notes task,
    // which reads its own moon.yml, lands with it.
    let mut policy = fs::read_to_string(repo.path.join(POLICY_FILE)).unwrap();
    policy.push_str(&format!(
        "\n[signers.{name}]\nssh = \"{line}\"\nkinds = [\"provenance\"]\n\n\
         [acceptance]\nrequire = [{{ kind = \"provenance\", signers = [\"{name}\"] }}]\n"
    ));
    let mut notes = fs::read_to_string(repo.path.join("notes/moon.yml")).unwrap();
    notes.push_str("# it grows\n");
    let landed = gate.land(
        "listed",
        &[
            (POLICY_FILE, policy.as_str()),
            ("notes/moon.yml", notes.as_str()),
        ],
    );
    let (code, second) = host.tailrace(repo, &["publish", &landed, "--json"]);
    assert_eq!(code, Some(0), "{second}");
    let version = published(&second);
    let index = Digest::parse(version["index"].as_str().unwrap()).unwrap();
    let told = &second["value"]["components"][0]["gate"];
    assert_eq!(told["answer"], "ok", "{second}");
    assert_eq!(told["value"]["judged"], "accepted", "{second}");
    assert_eq!(told["value"]["one_host"], true, "{second}");

    // beta's chain holds one promotion to the version's index, in the
    // repository's ledger refs and the remote's alike.
    let beta = ChannelId::new(None, ChannelName::new("beta"));
    let chain = GitRef::new(&repo.path, Some("origin".into()))
        .chain(&beta)
        .unwrap();
    let [promotion] = chain.claims.as_slice() else {
        panic!("beta's chain holds {} promotions", chain.claims.len())
    };
    let promoted = Promotion::of(&Claim::from_envelope(promotion).unwrap()).unwrap();
    assert_eq!(promoted.to.index, index);
    assert_eq!(
        repo.git(&[
            "for-each-ref",
            "--format=%(refname)",
            "refs/tailrace/ledger/"
        ]),
        "refs/tailrace/ledger/beta"
    );
    let remote = Repo {
        path: scratch.remote.clone(),
    };
    assert_eq!(
        remote.git(&["rev-parse", "refs/tailrace/ledger/beta"]),
        repo.git(&["rev-parse", "refs/tailrace/ledger/beta"]),
        "the remote holds another chain"
    );

    // verify counts every provenance as made by the host's name.
    let asked = version["version"].as_str().unwrap();
    let (code, verified) = host.tailrace(repo, &["verify", asked, "--json"]);
    assert_eq!(code, Some(0), "{verified}");
    let provenances: Vec<&Value> = verified["value"]["claims"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|t| t["kind"] == "provenance")
        .collect();
    assert!(!provenances.is_empty(), "{verified}");
    for provenance in provenances {
        assert_eq!(provenance["made_by"], name.as_str(), "{provenance}");
        assert_eq!(provenance["counts"], true, "{provenance}");
    }
}
