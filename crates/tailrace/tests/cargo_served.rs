//! Spec cargo-projection, the proof as one loop (design D11): a
//! publisher's family crates, `fam-a` depending by path on `fam-b`,
//! packed by the Cargo projection at four versions and put in a layout
//! store as a publish writes them, one version's index lacking the
//! acceptance the consumer's policy requires; the host's one serving
//! process run in the test at an address of the test's own over the
//! test's host state; and a consumer checkout with no lockfile, synced
//! against that process and built by Cargo, which resolves, downloads and
//! writes its lockfile from the process alone; then synced and built
//! again from the lockfile Cargo wrote (137, 140, 86, 99, 114, 115, 49).
//! cargo runs under a `CARGO_HOME` and `HOME` of the test's own. Needs
//! cargo, so it is in the tools layer.

#![cfg(feature = "tools")]

mod scratch;

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use scratch::{sealed_cargo, Repo, Scratch, POLICY_FILE};
use serde_json::Value;
use ssh_key::rand_core::OsRng;
use ssh_key::{Algorithm, LineEnding, PrivateKey};
use tailrace::bindings::For;
use tailrace::commands::serve::host::{find_or_start, name, Host};
use tailrace::commands::serve::serve_on;
use tailrace::commands::sync::{run_on, SyncArgs};
use tailrace::profiles::projection::cargo::{CargoProjection, ECOSYSTEM};
use tailrace::profiles::signer::host_key::HostKey;
use tailrace::profiles::store::layout::Layout;
use tailrace_core::{paths_hold, RepositoryName};
use tailrace_seams::{
    artifact_name, Artifact, BuildId, Claim, ClaimKind, Component, ComponentName, Digest,
    Ecosystem, IndexMeta, Manifest, Not, Output, OutputName, PackAt, Projection, Refusal, Revision,
    Signer, SignerId, Store, Version,
};

/// The publisher's served Cargo address, which its workspace's cargo
/// configuration names `tailrace`, so every packed crate names its
/// family dependencies at a served Cargo address.
const PUBLISHER: &str = "sparse+http://127.0.0.1:7420/agentplot/family/cargo/";

/// The versions the publisher packs, and the one whose index the gate
/// never accepted.
const VERSIONS: [&str; 4] = ["1.0.0", "1.1.0", "1.2.0", "2.0.0"];
const UNACCEPTED: &str = "1.2.0";

/// The newest version of `1` and of `fam-a` 1.1.0's `fam-b` the gate
/// accepted.
const HELD: &str = "1.1.0";

/// The family crates, in the order the publisher's component declares
/// their outputs.
const CRATES: [&str; 2] = ["fam-a", "fam-b"];

/// The publisher's workspace manifest, which holds the version its
/// crates inherit.
const WORKSPACE: &str = "[workspace]\n\
resolver = \"2\"\n\
members = [\"crates/*\"]\n\
\n\
[workspace.package]\n\
version = \"0.0.0\" # the tag's, written at publish\n\
edition = \"2021\"\n\
license = \"MIT\"\n";

const FAM_A: &str = "[package]\n\
name = \"fam-a\"\n\
version.workspace = true\n\
edition.workspace = true\n\
license.workspace = true\n\
description = \"a family crate\"\n\
\n\
[dependencies]\n\
fam-b = { path = \"../fam-b\", version = \"0.0.0\", registry = \"tailrace\" }\n";

const FAM_B: &str = "[package]\n\
name = \"fam-b\"\n\
version.workspace = true\n\
edition.workspace = true\n\
license.workspace = true\n\
description = \"a family crate\"\n";

/// The consumer's manifest, declaring the family crate by its
/// requirement and no lockfile beside it.
const CONSUMER: &str = "[package]\n\
name = \"consumer\"\n\
version = \"0.1.0\"\n\
edition = \"2021\"\n\
\n\
[dependencies]\n\
fam-a = { version = \"1\", registry = \"tailrace\" }\n";

/// The publisher: its workspace committed in a scratch repository, every
/// crate's packaging task's crate file made, and cargo sealed under a home
/// of the test's own.
struct Publisher {
    scratch: Scratch,
    cargo: PathBuf,
}

impl Publisher {
    fn new() -> Publisher {
        let scratch = Scratch::new();
        let repo = &scratch.repo;
        repo.write("Cargo.toml", WORKSPACE);
        repo.write(
            ".cargo/config.toml",
            &format!("[registries.tailrace]\nindex = \"{PUBLISHER}\"\n"),
        );
        repo.write("crates/fam-a/Cargo.toml", FAM_A);
        repo.write(
            "crates/fam-a/src/lib.rs",
            "pub fn hello() -> &'static str {\n    fam_b::hello()\n}\n",
        );
        repo.write("crates/fam-b/Cargo.toml", FAM_B);
        repo.write(
            "crates/fam-b/src/lib.rs",
            "pub fn hello() -> &'static str {\n    \"hello\"\n}\n",
        );
        repo.commit("feat: the family crates");
        let cargo = sealed_cargo(&scratch.path());
        let publisher = Publisher { scratch, cargo };
        publisher.packaging_task();
        publisher
    }

    fn repo(&self) -> &Repo {
        &self.scratch.repo
    }

    /// The build's packaging task, stood in for: every crate of the
    /// workspace packed by cargo outside the tree.
    fn packaging_task(&self) {
        let ran = Command::new(&self.cargo)
            .current_dir(&self.repo().path)
            .args([
                "package",
                "--workspace",
                "--no-verify",
                "--offline",
                "--exclude-lockfile",
                "--registry",
                "tailrace",
                "--target-dir",
            ])
            .arg(self.build())
            .output()
            .unwrap();
        assert!(
            ran.status.success(),
            "the packaging task failed: {}",
            String::from_utf8_lossy(&ran.stderr)
        );
    }

    fn build(&self) -> PathBuf {
        self.scratch.path().join("build")
    }

    /// The component `fam`, whose paths hold `crates/`, and both crates'
    /// packaging outputs.
    fn fam(&self) -> (Component, Vec<Output>) {
        let outputs: Vec<Output> = CRATES
            .iter()
            .map(|name| Output {
                name: OutputName::new(*name),
                files: vec![self
                    .build()
                    .join(format!("package/{name}-0.0.0.crate"))
                    .display()
                    .to_string()],
            })
            .collect();
        let component = Component {
            name: ComponentName::new("fam"),
            outputs: outputs.iter().map(|o| o.name.clone()).collect(),
            paths: vec!["crates/".into()],
            prefix: "fam-v".into(),
        };
        (component, outputs)
    }

    /// Each crate of `fam` packed at `version` and put in `layout` as a
    /// publish puts it: its bytes, its artifact, the version's index
    /// naming both, and each crate's `artifact/cargo/<name>@<version>` at
    /// the index; accepted by `gate` on the index where `accepted`. The
    /// bytes packed, by crate.
    fn publish(
        &self,
        layout: &Layout,
        gate: &HostKey,
        version: &str,
        accepted: bool,
    ) -> Vec<Vec<u8>> {
        let projection = CargoProjection::new(&self.repo().path, Box::new(layout.clone()))
            .cargo_program(&self.cargo);
        let (component, outputs) = self.fam();
        let holds = paths_hold(&component.paths).unwrap();
        let version = Version::new(version);
        let mut artifacts = Vec::new();
        let mut packed = Vec::new();
        for output in &outputs {
            let crate_ = projection
                .pack(
                    output,
                    &BuildId::new("b7e1"),
                    Some(&PackAt {
                        version: &version,
                        component: &component,
                        outputs: &outputs,
                        holds: &holds,
                    }),
                )
                .unwrap_or_else(|not| panic!("{} at {version}: {not}", output.name));
            let meta = crate_.meta.unwrap();
            let layer = layout.put_bytes(&crate_.bytes).unwrap().digest;
            let digest = layout
                .put_artifact(&Manifest::Artifact {
                    meta: meta.clone(),
                    layers: vec![layer],
                })
                .unwrap()
                .digest;
            artifacts.push(Artifact { digest, meta });
            packed.push(crate_.bytes);
        }
        let index =
            IndexMeta::new(component.name.clone(), version.clone(), artifacts.clone()).unwrap();
        let index = layout.put_artifact(&Manifest::Index(index)).unwrap().digest;
        for artifact in &artifacts {
            let name = artifact_name(&Ecosystem::new(ECOSYSTEM), artifact.meta.name(), &version);
            layout.name(&name, &index, None).unwrap();
        }
        if accepted {
            let claim = Claim {
                subject: index.clone(),
                kind: ClaimKind::Acceptance,
                revision: Revision::new(self.repo().head()),
                build: None,
                body: Value::Null,
            };
            let envelope = gate.sign(&claim, &SignerId::new("gate")).unwrap();
            layout.attach(&index, &envelope).unwrap();
        }
        packed
    }
}

/// The gate's key, made in `dir`, as the host key profile signs with it,
/// and its public half as a policy lists it.
fn gate_key(dir: &Path) -> (HostKey, String) {
    fs::create_dir_all(dir).unwrap();
    let path = dir.join("gate");
    let key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
    key.write_openssh_file(&path, LineEnding::LF).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let public = key.public_key().to_openssh().unwrap();
    (HostKey::new(SignerId::new("gate"), &path), public)
}

/// The test's host: a state directory, a bindings file binding the
/// layout store, the plain branch landing and the git ref ledger on
/// `origin`, a home and a data directory, all under `dir`.
fn host_in(dir: &Path, layout: &Path) -> Host {
    let host = Host {
        state: dir.join("state/tailrace"),
        config: dir.join("config/tailrace/config.toml"),
        home: Some(dir.join("home")),
        data: Some(dir.join("data")),
    };
    fs::create_dir_all(host.config.parent().unwrap()).unwrap();
    fs::write(
        &host.config,
        format!(
            "[store]\nlayout = {:?}\n\n[landing]\nbranch = {{ remote = \"origin\" }}\n\n\
             [ledger]\ngit_ref = {{ remote = \"origin\" }}\n",
            layout.display().to_string()
        ),
    )
    .unwrap();
    host
}

/// A port for the test's process, never the core's: below the range the
/// system hands to connections, and above every port the other tests'
/// processes take.
fn free() -> SocketAddr {
    let port = 30_000 + (std::process::id() % 400) as u16 * 5;
    SocketAddr::from(([127, 0, 0, 1], port))
}

/// `GET` of `path` asked of the process at `address`: the status and the
/// body.
fn get(address: SocketAddr, path: &str) -> (u16, Vec<u8>) {
    let mut stream = TcpStream::connect(address).unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut answered = Vec::new();
    stream.read_to_end(&mut answered).unwrap();
    let split = answered
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("a head and a body");
    let head = String::from_utf8_lossy(&answered[..split]).to_string();
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .expect("a status");
    (status, answered[split + 4..].to_vec())
}

/// The consumer: a checkout of `agentplot/consumer` with no lockfile,
/// whose landed policy requires the gate's acceptance of every version
/// laid out and whose cargo configuration names `tailrace` at the test's
/// process.
struct Consumer {
    scratch: Scratch,
    cargo: PathBuf,
}

impl Consumer {
    fn new(address: SocketAddr, gate: &str) -> Consumer {
        let scratch = Scratch::new();
        let repo = &scratch.repo;
        repo.write("Cargo.toml", CONSUMER);
        repo.write(
            "src/lib.rs",
            "pub fn greeting() -> &'static str {\n    fam_a::hello()\n}\n",
        );
        repo.write(
            ".cargo/config.toml",
            &format!(
                "[registries.tailrace]\nindex = \"sparse+http://{address}/agentplot/consumer/cargo/\"\n"
            ),
        );
        repo.write(
            POLICY_FILE,
            &format!(
                "[signers.gate]\nssh = \"{gate}\"\nkinds = [\"acceptance\"]\n\n\
                 [layout]\nrequire = [{{ kind = \"acceptance\", signers = [\"gate\"] }}]\n"
            ),
        );
        repo.commit("feat: a consumer of the family");
        repo.push();
        // The remote's address names the repository; git reads it from
        // the scratch remote.
        let named = "https://example.com/agentplot/consumer.git";
        repo.git(&["remote", "set-url", "origin", named]);
        repo.git(&[
            "config",
            &format!("url.{}.insteadOf", scratch.remote.display()),
            named,
        ]);
        let cargo = sealed_cargo(&scratch.path());
        Consumer { scratch, cargo }
    }

    fn at(&self) -> &Path {
        &self.scratch.repo.path
    }

    /// `tailrace sync --json` over `host`'s binding for sync, its serving
    /// process found at `address` and none started: the exit and the
    /// answer.
    fn sync(&self, host: &Host, address: SocketAddr) -> (i32, Value) {
        let args = SyncArgs {
            revision: None,
            branch: "main".into(),
            json: true,
        };
        let mut out = Vec::new();
        let code = run_on(
            &args,
            &host.bind_for(self.at(), For::Sync),
            self.at(),
            &mut out,
            &Ok(host.clone()),
            address,
            || panic!("sync started a process of its own"),
        );
        (code, serde_json::from_slice(&out).unwrap())
    }

    /// `cargo build` with `args` in the checkout, its target outside the
    /// tree.
    fn build(&self, args: &[&str]) {
        let ran = Command::new(&self.cargo)
            .current_dir(self.at())
            .env("CARGO_TARGET_DIR", self.scratch.path().join("target"))
            .arg("build")
            .args(args)
            .output()
            .unwrap();
        assert!(
            ran.status.success(),
            "cargo build {args:?} failed: {}",
            String::from_utf8_lossy(&ran.stderr)
        );
    }

    fn lockfile(&self) -> String {
        fs::read_to_string(self.at().join("Cargo.lock")).unwrap()
    }
}

/// The Cargo layout's answer in sync's `answer`, rendered now.
fn refreshed(answer: &Value) -> &Value {
    let layout = answer["value"]["layouts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["ecosystem"] == ECOSYSTEM)
        .unwrap_or_else(|| panic!("no layout of cargo: {answer}"));
    assert_eq!(layout["answer"], "ok", "{layout}");
    &layout["value"]["refreshed"]
}

/// The refusal `answer` gives, as the answer a person reads.
fn refusal(answer: &Value) -> Not {
    assert_eq!(answer["answer"], "refused", "{answer}");
    Not::Refused(serde_json::from_value::<Refusal>(answer["refused"].clone()).unwrap())
}

/// The versions the index file at `path` lists, as Cargo reads its lines.
fn listed(address: SocketAddr, path: &str) -> Vec<String> {
    let (status, body) = get(address, &format!("/agentplot/consumer/cargo/{path}"));
    let body = String::from_utf8(body).unwrap();
    assert_eq!(status, 200, "{body}");
    body.lines()
        .map(|line| {
            let line: Value = serde_json::from_str(line).unwrap();
            line["vers"].as_str().unwrap().to_string()
        })
        .collect()
}

/// The `[[package]]` tables of `lockfile`, by name.
fn packages(lockfile: &str) -> Vec<toml::Table> {
    let lock: toml::Table = toml::from_str(lockfile).unwrap();
    lock["package"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_table().unwrap().clone())
        .collect()
}

fn checksum(bytes: &[u8]) -> String {
    Digest::of(bytes)
        .as_str()
        .strip_prefix("sha256:")
        .unwrap()
        .to_string()
}

#[test]
fn a_consumer_with_no_lockfile_builds_from_the_hosts_process_holding_the_newest_accepted_crates() {
    let publisher = Publisher::new();
    let dir = publisher.scratch.path();
    let layout_at = dir.join("layout");
    let layout = Layout::create(&layout_at).unwrap();
    let (gate, gate_public) = gate_key(&dir.join("keys"));
    let mut packed = Vec::new();
    for version in VERSIONS {
        packed.push(publisher.publish(&layout, &gate, version, version != UNACCEPTED));
    }
    let held = &packed[VERSIONS.iter().position(|v| *v == HELD).unwrap()];

    let host = host_in(&dir.join("host"), &layout_at);
    let address = free();
    let process = host.clone();
    thread::spawn(move || serve_on(address, &process, Duration::from_secs(60)));
    find_or_start(address, || Ok(())).expect("the test's process answers");

    let consumer = Consumer::new(address, &gate_public);
    let repository = RepositoryName {
        owner: "agentplot".into(),
        name: "consumer".into(),
    };
    name(&host, &repository, consumer.at()).unwrap();
    assert!(!consumer.at().join("Cargo.lock").exists());

    // The ranges: fam-a's from the manifest, and fam-b's from fam-a
    // 1.1.0, each held at 1.1.0 with 1.2.0 refused.
    let (code, answer) = consumer.sync(&host, address);
    assert_eq!(code, 0, "{answer:#}");
    assert_eq!(answer["serving"]["answer"], "ok", "{answer:#}");
    let layout_answer = refreshed(&answer);
    assert_eq!(
        layout_answer["pins"],
        serde_json::json!([]),
        "{layout_answer:#}"
    );
    let ranges: Vec<(Value, Value, Value, Value)> = layout_answer["ranges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["name"].clone(),
                r["range"].clone(),
                r["by"].clone(),
                r["value"].clone(),
            )
        })
        .collect();
    assert_eq!(
        ranges,
        [
            ("fam-a".into(), "1".into(), Value::Null, HELD.into()),
            (
                "fam-b".into(),
                HELD.into(),
                serde_json::json!({ "name": "fam-a", "version": HELD }),
                HELD.into()
            ),
        ],
        "{layout_answer:#}"
    );
    let asked = layout_answer["asked"].as_array().unwrap();
    let refused: Vec<(&str, &str)> = asked
        .iter()
        .filter(|a| a["answer"] == "refused")
        .map(|a| (a["name"].as_str().unwrap(), a["version"].as_str().unwrap()))
        .collect();
    assert_eq!(
        refused,
        [("fam-a", UNACCEPTED), ("fam-b", UNACCEPTED)],
        "{layout_answer:#}"
    );
    let wanting: Vec<Not> = asked
        .iter()
        .filter(|a| a["answer"] == "refused")
        .map(refusal)
        .collect();
    for not in &wanting {
        let said = not.to_string();
        assert!(
            said.contains("acceptance") && said.contains("gate"),
            "the refusal names no claim found wanting: {said}"
        );
    }

    // What the process serves: the configuration's downloads under the
    // test's own address, each index file listing 1.1.0 alone, and the
    // refused crate's download its refusal in the same words.
    let (status, config) = get(address, "/agentplot/consumer/cargo/config.json");
    assert_eq!(status, 200);
    let config: Value = serde_json::from_slice(&config).unwrap();
    assert_eq!(
        config["dl"],
        format!("http://{address}/agentplot/consumer/cargo/crates/{{crate}}/{{version}}/download")
    );
    assert_eq!(listed(address, "fa/m-/fam-a"), [HELD]);
    assert_eq!(listed(address, "fa/m-/fam-b"), [HELD]);
    let (status, body) = get(
        address,
        &format!("/agentplot/consumer/cargo/crates/fam-a/{UNACCEPTED}/download"),
    );
    let body = String::from_utf8(body).unwrap();
    assert_eq!(status, 403, "{body}");
    assert_eq!(body, format!("{}\n", wanting[0]));

    // Cargo resolves, downloads and writes the lockfile from the process.
    consumer.build(&[]);
    let lockfile = consumer.lockfile();
    let source = format!("sparse+http://{address}/agentplot/consumer/cargo/");
    let mut family: Vec<(String, String, String, String)> = packages(&lockfile)
        .into_iter()
        .filter(|p| p["name"].as_str() != Some("consumer"))
        .map(|p| {
            let field = |key: &str| {
                p.get(key)
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| panic!("{key} missing from {p}"))
                    .to_string()
            };
            (
                field("name"),
                field("version"),
                field("source"),
                field("checksum"),
            )
        })
        .collect();
    family.sort();
    assert_eq!(
        family,
        CRATES
            .iter()
            .zip(held)
            .map(|(name, bytes)| (
                name.to_string(),
                HELD.to_string(),
                source.clone(),
                checksum(bytes)
            ))
            .collect::<Vec<_>>(),
        "{lockfile}"
    );
    assert!(!lockfile.contains("path"), "{lockfile}");

    // Synced again, both are pins held; built locked, the lockfile is as
    // Cargo wrote it.
    let (code, answer) = consumer.sync(&host, address);
    assert_eq!(code, 0, "{answer:#}");
    let layout_answer = refreshed(&answer);
    let pins: Vec<(&str, &str, &str)> = layout_answer["pins"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p["name"].as_str().unwrap(),
                p["version"].as_str().unwrap(),
                p["answer"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        pins,
        [("fam-a", HELD, "ok"), ("fam-b", HELD, "ok")],
        "{layout_answer:#}"
    );
    consumer.build(&["--locked"]);
    assert_eq!(consumer.lockfile(), lockfile);
}
