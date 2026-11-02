//! Spec npm-projection, npm against a served layout (design D12): a
//! publisher's family packages, `@family/widget`, each version requiring
//! `@family/base` by a range, packed by npm's own packager through the
//! npm projection and put in a layout store as a publish writes them,
//! each with the publisher's provenance but one; and a consumer whose
//! policy file names the publisher's host key signer for provenance,
//! whose `package.json` declares `@family/widget` by a range, with no
//! lockfile, and whose `.npmrc` sends `@family` to the served address.
//! `tailrace sync` starts each host's serving process at an address of
//! the test's own, with an idle of moments, and `npm install` and `npm ci`
//! run against it (137, 140, 86, 99, 114, 115, 49).
//!
//! npm runs under a home, a cache and a configuration of each host's own,
//! its public registry an address nothing answers at, so nothing of the
//! family is asked of any registry but the served layout. The checkout's
//! `.npmrc` names the core's address, as a consumer writes it and as the
//! projection reads it; the test's npm configuration sends the scope to
//! the test's own address in its place, since a test starts no process at
//! the core's (rule core.36). Needs npm, so it is in the tools layer.

#![cfg(feature = "tools")]

mod scratch;

use std::fs;
use std::net::SocketAddr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use scratch::{on_path, Repo, Scratch, POLICY_FILE};
use serde_json::{json, Value};
use sha2::Digest as _;
use ssh_key::rand_core::OsRng;
use ssh_key::{Algorithm, LineEnding, PrivateKey};
use tailrace::bindings::For;
use tailrace::commands::serve::host::{find_or_start, Host};
use tailrace::commands::serve::{serve_on, Served};
use tailrace::commands::sync::{run_on, SyncArgs};
use tailrace::profiles::projection::npm::{NpmProjection, ECOSYSTEM};
use tailrace::profiles::signer::host_key::HostKey;
use tailrace::profiles::store::layout::Layout;
use tailrace_core::{origin, Provenance};
use tailrace_fakes::Alone;
use tailrace_seams::{
    artifact_name, Answer, Artifact, BuildId, ComponentName, Ecosystem, IndexMeta, Manifest,
    Outcome, Output, OutputName, Projection, Revision, Signer, SignerId, Store,
};

/// The publisher's signer, as the consumer's policy file names it.
const PUBLISHER: &str = "publisher";

/// The versions of `@family/widget` the store holds, the one without the
/// publisher's provenance, which the consumer's policy refuses, and the
/// newest the policy admits within `^1.2.0`.
const WIDGETS: [&str; 3] = ["1.2.0", "1.3.0", "1.5.0"];
const REFUSED: &str = "1.5.0";
const WIDGET: &str = "1.3.0";

/// The versions of `@family/base` the store holds, and the newest within
/// `^2.0.0`, which every widget requires.
const BASES: [&str; 3] = ["2.0.0", "2.1.0", "3.0.0"];
const BASE: &str = "2.1.0";

/// A version of `@family/widget` the store does not name.
const UNNAMED: &str = "1.4.0";

/// The time a process waits unasked before it asks whether a layout is
/// served: moments, so the test sees it stay past it.
const IDLE: Duration = Duration::from_millis(500);

/// The served layout's path below the process's address.
const SERVED: &str = "agentplot/consumer/npm";

/// The consumer's manifest, declaring the family package by a range and
/// nothing of any other registry.
const CONSUMER: &str = r#"{
  "name": "consumer",
  "version": "1.0.0",
  "private": true,
  "dependencies": {
    "@family/widget": "^1.2.0"
  }
}
"#;

/// The publisher: its packages committed in a scratch repository, as the
/// build leaves them.
struct Publisher {
    scratch: Scratch,
}

impl Publisher {
    fn new() -> Publisher {
        let scratch = Scratch::new();
        let repo = &scratch.repo;
        repo.write(
            "packages/widget/package.json",
            &json!({
                "name": "@family/widget",
                "version": "0.0.0",
                "main": "index.js",
                "dependencies": { "@family/base": "^2.0.0" },
            })
            .to_string(),
        );
        repo.write(
            "packages/widget/index.js",
            "module.exports = () => require('@family/base')() + ' widget';\n",
        );
        repo.write(
            "packages/base/package.json",
            &json!({ "name": "@family/base", "version": "0.0.0", "main": "index.js" }).to_string(),
        );
        repo.write(
            "packages/base/index.js",
            "module.exports = () => 'base ' + require('./package.json').version;\n",
        );
        repo.commit("feat: the family packages");
        Publisher { scratch }
    }

    fn repo(&self) -> &Repo {
        &self.scratch.repo
    }

    /// The output of `package`: its `package.json` and its `index.js`, as
    /// the build left them.
    fn output(&self, package: &str) -> Output {
        let dir = self.repo().path.join("packages").join(package);
        Output {
            name: OutputName::new(package),
            files: ["package.json", "index.js"]
                .iter()
                .map(|f| dir.join(f).display().to_string())
                .collect(),
        }
    }

    /// `package` packed at `version` by npm through the projection and put
    /// in `layout` as a publish puts it: the tarball, its artifact, the
    /// version's index naming it and its `artifact/npm/<name>@<version>`
    /// at the index; with the publisher's provenance on the artifact,
    /// signed by `signer`, where `made`. The tarball's bytes.
    fn publish(
        &self,
        layout: &Layout,
        signer: &HostKey,
        package: &str,
        version: &str,
        made: bool,
    ) -> Vec<u8> {
        let projection = NpmProjection::new(&self.repo().path, Box::new(layout.clone()), origin());
        let output = self.output(package);
        let build = BuildId::new("b7e1");
        let packed = projection
            .pack(&output, &build, Some(&Alone::new(&output, version).at()))
            .unwrap_or_else(|not| panic!("{package} at {version}: {not}"));
        let meta = packed.meta.unwrap();
        let layer = layout.put_bytes(&packed.bytes).unwrap().digest;
        let digest = layout
            .put_artifact(&Manifest::Artifact {
                meta: meta.clone(),
                layers: vec![layer],
            })
            .unwrap()
            .digest;
        let artifacts = vec![Artifact {
            digest: digest.clone(),
            meta: meta.clone(),
        }];
        let index = IndexMeta::new(
            ComponentName::new(package),
            meta.version().clone(),
            artifacts,
        )
        .unwrap();
        let index = layout.put_artifact(&Manifest::Index(index)).unwrap().digest;
        let name = artifact_name(&Ecosystem::new(ECOSYSTEM), meta.name(), meta.version());
        layout.name(&name, &index, None).unwrap();
        if made {
            let provenance = Provenance {
                output: output.name.clone(),
                ecosystem: meta.ecosystem().clone(),
                name: meta.name().clone(),
                version: meta.version().clone(),
                outcome: Outcome::Pass,
                carried: vec![],
            };
            let claim = provenance.claim(digest.clone(), Revision::new(self.repo().head()), build);
            let envelope = signer.sign(&claim, &SignerId::new(PUBLISHER)).unwrap();
            layout.attach(&digest, &envelope).unwrap();
        }
        packed.bytes
    }
}

/// The publisher's key, made in `dir`, as the host key profile signs with
/// it, and its public half as a policy file lists it.
fn publisher_key(dir: &Path) -> (HostKey, String) {
    fs::create_dir_all(dir).unwrap();
    let path = dir.join(PUBLISHER);
    let key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
    key.write_openssh_file(&path, LineEnding::LF).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let public = key.public_key().to_openssh().unwrap();
    (HostKey::new(SignerId::new(PUBLISHER), &path), public)
}

/// A host: its state directory, a bindings file binding the layout store
/// at `layout`, the plain branch landing and the git ref ledger on
/// `origin`, and a home and a data directory, all under `dir`.
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

/// A port for the test's processes, never the core's: below the range the
/// system hands to connections, and apart from every port the other
/// tests' processes take.
fn free() -> SocketAddr {
    let port = 31_000 + (std::process::id() % 400) as u16 * 5 + 2;
    SocketAddr::from(([127, 0, 0, 1], port))
}

/// The served layout's base at `address`, as the process answers it.
fn base(address: SocketAddr) -> String {
    format!("http://{address}/{SERVED}")
}

/// `repo`, a checkout of the consumer, its landing's remote named
/// `agentplot/consumer` and reached at `remote`.
fn named(repo: &Repo, remote: &Path) {
    let named = "https://example.com/agentplot/consumer.git";
    repo.git(&["remote", "set-url", "origin", named]);
    repo.git(&[
        "config",
        &format!("url.{}.insteadOf", remote.display()),
        named,
    ]);
}

/// The consumer: `agentplot/consumer`, with no lockfile, its landed
/// policy file reading provenance through the publisher's signer and
/// requiring it of every version laid out, and its `.npmrc` sending
/// `@family` to the core's served address.
fn consumer(publisher: &str) -> Scratch {
    let scratch = Scratch::new();
    let repo = &scratch.repo;
    repo.write("package.json", CONSUMER);
    repo.write(
        ".npmrc",
        &format!("@family:registry={}{SERVED}/\n", origin()),
    );
    repo.write(".gitignore", "node_modules/\n");
    repo.write(
        POLICY_FILE,
        &format!(
            "[signers.{PUBLISHER}]\nssh = \"{publisher}\"\nkinds = [\"provenance\"]\n\n\
             [layout]\nrequire = [{{ kind = \"provenance\", signers = [\"{PUBLISHER}\"] }}]\n"
        ),
    );
    repo.commit("feat: a consumer of the family");
    repo.push();
    named(repo, &scratch.remote);
    scratch
}

/// `tailrace sync --json` in the checkout at `at` over `host`'s binding
/// for sync, its serving process found at `address`, or started by
/// `start`: the exit and the answer.
fn sync(
    at: &Path,
    host: &Host,
    address: SocketAddr,
    start: impl FnOnce() -> Answer<()>,
) -> (i32, Value) {
    let args = SyncArgs {
        revision: None,
        branch: "main".into(),
        json: true,
    };
    let mut out = Vec::new();
    let code = run_on(
        &args,
        &host.bind_for(at, For::Sync),
        at,
        &mut out,
        &Ok(host.clone()),
        address,
        start,
    );
    (code, serde_json::from_slice(&out).unwrap())
}

/// A sync that finds the process and starts none.
fn found() -> Answer<()> {
    panic!("sync started a second process")
}

/// The host's serving process at `address` with an idle of moments,
/// started on a thread of its own as sync starts it.
fn started(host: &Host, address: SocketAddr) -> JoinHandle<Answer<Served>> {
    let host = host.clone();
    thread::spawn(move || serve_on(address, &host, IDLE))
}

/// `process` ended at its next idle once nothing it serves holds a
/// layout: the layouts of `checkouts` removed, and its thread joined.
fn ended(process: JoinHandle<Answer<Served>>, checkouts: &[&Path]) {
    for at in checkouts {
        fs::remove_dir_all(at.join(".tailrace")).unwrap();
    }
    let asked = Instant::now();
    while !process.is_finished() {
        assert!(
            asked.elapsed() < Duration::from_secs(30),
            "the process kept on with no layout served"
        );
        thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(process.join().unwrap(), Ok(Served::Idle));
}

/// The pid the process answered sync with.
fn pid(answer: &Value) -> u64 {
    assert_eq!(answer["serving"]["answer"], "ok", "{answer:#}");
    answer["serving"]["value"]["pid"].as_u64().unwrap()
}

/// The npm layout's answer in sync's `answer`, rendered now.
fn refreshed(answer: &Value) -> &Value {
    let layout = answer["value"]["layouts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["ecosystem"] == ECOSYSTEM)
        .unwrap_or_else(|| panic!("no layout of npm: {answer:#}"));
    assert_eq!(layout["answer"], "ok", "{layout:#}");
    &layout["value"]["refreshed"]
}

/// One host's npm: a home, a cache and a configuration of its own under
/// `home`, the public registry an address nothing answers at, asked once,
/// and the family's scope sent to the test's process at `address`.
struct Npm {
    home: PathBuf,
    address: SocketAddr,
}

/// What npm answered: whether it succeeded, and what it printed.
struct Ran {
    ok: bool,
    said: String,
}

impl Npm {
    fn new(home: PathBuf, address: SocketAddr) -> Npm {
        fs::create_dir_all(&home).unwrap();
        Npm { home, address }
    }

    fn command(&self, program: &str, at: &Path) -> Command {
        let mut command = Command::new(on_path(program));
        command
            .current_dir(at)
            .env("HOME", &self.home)
            .env("npm_config_cache", self.home.join(".npm"))
            .env("npm_config_userconfig", self.home.join(".npmrc"))
            .env("npm_config_globalconfig", self.home.join("global.npmrc"))
            .env("npm_config_registry", "http://127.0.0.1:9/")
            .env("npm_config_fetch_retries", "0")
            .env("npm_config_prefer_online", "true")
            .env("npm_config_update_notifier", "false")
            .env("npm_config_fund", "false")
            .env("npm_config_audit", "false")
            .env(
                "npm_config_@family:registry",
                format!("{}/", base(self.address)),
            );
        command
    }

    /// `npm` with `args` in the checkout at `at`.
    fn npm(&self, at: &Path, args: &[&str]) -> Ran {
        let ran = self.command("npm", at).args(args).output().unwrap();
        Ran {
            ok: ran.status.success(),
            said: format!(
                "{}{}",
                String::from_utf8_lossy(&ran.stdout),
                String::from_utf8_lossy(&ran.stderr)
            ),
        }
    }

    /// `npm` with `args` in the checkout at `at`, which succeeds.
    fn installs(&self, at: &Path, args: &[&str]) {
        let ran = self.npm(at, args);
        assert!(ran.ok, "npm {args:?} failed: {}", ran.said);
    }

    /// What the checkout at `at`'s installed widget answers, run by node.
    fn greets(&self, at: &Path) -> String {
        let ran = self
            .command("node", at)
            .args(["-e", "process.stdout.write(require('@family/widget')())"])
            .output()
            .unwrap();
        assert!(
            ran.status.success(),
            "{}",
            String::from_utf8_lossy(&ran.stderr)
        );
        String::from_utf8(ran.stdout).unwrap()
    }
}

/// The lockfile npm wrote in the checkout at `at`.
fn lockfile(at: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(at.join("package-lock.json")).unwrap()).unwrap()
}

/// The family's packages `lockfile` pins: each name, version, `resolved`
/// and `integrity`, in order of name.
fn family(lockfile: &Value) -> Vec<(String, String, String, String)> {
    let mut pinned: Vec<(String, String, String, String)> = lockfile["packages"]
        .as_object()
        .unwrap()
        .iter()
        .filter_map(|(key, entry)| {
            let name = key.strip_prefix("node_modules/")?;
            let text = |field: &str| {
                entry[field]
                    .as_str()
                    .unwrap_or_else(|| panic!("{key} names no {field}: {lockfile:#}"))
                    .to_string()
            };
            Some((
                name.to_string(),
                text("version"),
                text("resolved"),
                text("integrity"),
            ))
        })
        .collect();
    pinned.sort();
    pinned
}

/// What npm's lockfile records of a tarball of `bytes` at `version` of
/// `name`, served at `address`.
fn recorded(
    address: SocketAddr,
    name: &str,
    version: &str,
    bytes: &[u8],
) -> (String, String, String, String) {
    let file = name.rsplit('/').next().unwrap();
    (
        name.to_string(),
        version.to_string(),
        format!("{}/{name}/-/{file}-{version}.tgz", base(address)),
        format!("sha512-{}", STANDARD.encode(sha2::Sha512::digest(bytes))),
    )
}

/// The checkout at `at`'s lockfile with `@family/widget` at `version`,
/// its tarball at the served address and no integrity, as another
/// checkout might have written it.
fn pinning(at: &Path, address: SocketAddr, version: &str) {
    let mut lock = lockfile(at);
    let widget = &mut lock["packages"]["node_modules/@family/widget"];
    widget["version"] = json!(version);
    widget["resolved"] = json!(format!(
        "{}/@family/widget/-/widget-{version}.tgz",
        base(address)
    ));
    widget.as_object_mut().unwrap().remove("integrity");
    fs::write(
        at.join("package-lock.json"),
        serde_json::to_string_pretty(&lock).unwrap(),
    )
    .unwrap();
}

#[test]
fn npm_installs_the_family_from_the_hosts_process_alone_and_its_lockfile_names_no_host() {
    scratch::home();
    let publisher = Publisher::new();
    let dir = publisher.scratch.path();
    let layout_at = dir.join("layout");
    let layout = Layout::create(&layout_at).unwrap();
    let (signer, public) = publisher_key(&dir.join("keys"));
    let mut widgets = Vec::new();
    for version in WIDGETS {
        let made = version != REFUSED;
        widgets.push(publisher.publish(&layout, &signer, "widget", version, made));
    }
    let mut bases = Vec::new();
    for version in BASES {
        bases.push(publisher.publish(&layout, &signer, "base", version, true));
    }
    let widget = &widgets[WIDGETS.iter().position(|v| *v == WIDGET).unwrap()];
    let base_held = &bases[BASES.iter().position(|v| *v == BASE).unwrap()];
    let address = free();
    let expected = vec![
        recorded(address, "@family/base", BASE, base_held),
        recorded(address, "@family/widget", WIDGET, widget),
    ];

    // A first install with no lockfile: sync holds the newest admitted
    // widget within the manifest's range and the newest base within what
    // it requires, names 1.5.0 refused for want of provenance, and starts
    // the host's process.
    let consumer = consumer(&public);
    let at = consumer.repo.path.clone();
    let host = host_in(&dir.join("host-a"), &layout_at);
    let npm = Npm::new(dir.join("host-a/npm"), address);
    assert!(!at.join("package-lock.json").exists());
    let mut process = None;
    let (code, answer) = sync(&at, &host, address, || {
        process = Some(started(&host, address));
        Ok(())
    });
    let process = process.expect("sync started the host's process");
    assert_eq!(code, 0, "{answer:#}");
    let first = pid(&answer);
    let layout_answer = refreshed(&answer);
    assert_eq!(layout_answer["pins"], json!([]), "{layout_answer:#}");
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
            (
                "@family/widget".into(),
                "^1.2.0".into(),
                Value::Null,
                WIDGET.into()
            ),
            (
                "@family/base".into(),
                "^2.0.0".into(),
                json!({ "name": "@family/widget", "version": WIDGET }),
                BASE.into()
            ),
        ],
        "{layout_answer:#}"
    );
    let asked: Vec<(&str, &str, &str)> = layout_answer["asked"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| {
            (
                a["name"].as_str().unwrap(),
                a["version"].as_str().unwrap(),
                a["answer"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        asked,
        [
            ("@family/widget", REFUSED, "refused"),
            ("@family/widget", WIDGET, "ok"),
            ("@family/base", BASE, "ok"),
        ],
        "{layout_answer:#}"
    );
    let refusal = layout_answer["asked"][0]["refused"].to_string();
    assert!(
        refusal.contains("provenance") && refusal.contains(PUBLISHER),
        "the refusal names no claim found wanting: {refusal}"
    );

    // Past the process's idle time, npm installs from the process the sync
    // found, with no other registry answering, and writes the lockfile.
    thread::sleep(IDLE * 4);
    assert!(
        !process.is_finished(),
        "the process ended with a layout served"
    );
    npm.installs(&at, &["install"]);
    assert_eq!(u64::from(find_or_start(address, found).unwrap().pid), first);
    let written = lockfile(&at);
    assert_eq!(family(&written), expected, "{written:#}");
    assert_eq!(npm.greets(&at), format!("base {BASE} widget"));

    // Synced again from the lockfile npm wrote, both are pins held, the
    // widget's range of base met by the base pinned, and `npm ci` installs
    // exactly them, the lockfile as npm wrote it.
    let (code, answer) = sync(&at, &host, address, found);
    assert_eq!(code, 0, "{answer:#}");
    let pins: Vec<(&str, &str, &str)> = refreshed(&answer)["pins"]
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
        [
            ("@family/base", BASE, "ok"),
            ("@family/widget", WIDGET, "ok")
        ],
        "{answer:#}"
    );
    assert_eq!(
        refreshed(&answer)["ranges"],
        json!([{
            "answer": "ok",
            "value": BASE,
            "name": "@family/base",
            "range": "^2.0.0",
            "by": { "name": "@family/widget", "version": WIDGET },
        }]),
        "{answer:#}"
    );
    npm.installs(&at, &["ci"]);
    assert_eq!(lockfile(&at), written);
    assert_eq!(npm.greets(&at), format!("base {BASE} widget"));

    // A second worktree of the repository is answered by the same process.
    let two = dir.join("consumer-two");
    consumer
        .repo
        .git(&["worktree", "add", "-q", "-b", "two", two.to_str().unwrap()]);
    let (code, answer) = sync(&two, &host, address, found);
    assert_eq!(code, 0, "{answer:#}");
    assert_eq!(pid(&answer), first);
    npm.installs(&two, &["install"]);
    assert_eq!(family(&lockfile(&two)), expected);
    npm.installs(&at, &["ci"]);

    // A version the store does not name fails the install naming it as
    // absent; one the policy refuses fails it with the claim's words.
    pinning(&at, address, UNNAMED);
    let ran = npm.npm(&at, &["ci"]);
    assert!(!ran.ok, "an install of a version no store names succeeded");
    assert!(
        ran.said.contains("404")
            && ran
                .said
                .contains(&format!("artifact/npm/@family/widget@{UNNAMED}")),
        "npm said: {}",
        ran.said
    );
    pinning(&at, address, REFUSED);
    let ran = npm.npm(&at, &["ci"]);
    assert!(!ran.ok, "an install of a refused version succeeded");
    assert!(
        ran.said.contains("403")
            && ran.said.contains(&format!("@family/widget {REFUSED}"))
            && ran.said.contains("provenance")
            && ran.said.contains(PUBLISHER),
        "npm said: {}",
        ran.said
    );
    fs::write(
        at.join("package-lock.json"),
        serde_json::to_string_pretty(&written).unwrap(),
    )
    .unwrap();
    assert_eq!(u64::from(find_or_start(address, found).unwrap().pid), first);
    ended(process, &[&at, &two]);

    // A second host, under another home at another path, syncs and installs
    // a checkout of its own, and its lockfile records the same addresses
    // and integrities, naming no home, checkout or store.
    let elsewhere = consumer.clone_as("elsewhere");
    named(&elsewhere, &consumer.remote);
    let other = host_in(&dir.join("host-b"), &layout_at);
    let other_npm = Npm::new(dir.join("host-b/npm"), address);
    let mut process = None;
    let (code, answer) = sync(&elsewhere.path, &other, address, || {
        process = Some(started(&other, address));
        Ok(())
    });
    assert_eq!(code, 0, "{answer:#}");
    let process = process.expect("sync started the second host's process");
    other_npm.installs(&elsewhere.path, &["install"]);
    let theirs = lockfile(&elsewhere.path);
    assert_eq!(family(&theirs), expected, "{theirs:#}");
    for lock in [&written, &theirs] {
        let text = lock.to_string();
        for place in [
            dir.join("host-a"),
            dir.join("host-b"),
            at.clone(),
            elsewhere.path.clone(),
            layout_at.clone(),
        ] {
            assert!(
                !text.contains(&place.display().to_string()),
                "the lockfile names {}: {text}",
                place.display()
            );
        }
    }
    ended(process, &[&elsewhere.path]);
}
