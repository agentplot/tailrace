//! Spec herdr-plugin: the documents the herdr plugin's tests render are
//! the binary's own (57, 132; design D6 part 2). A scratch host on a
//! machine that never published: a remote and its clone, the policy
//! declaring `beta` and `stable`, two promotions of `beta` in the git ref
//! ledger, no record of a publish, and a second named repository whose
//! checkout is gone. The root's and the repository's answers are asked of
//! the process's routes with no port bound, each time, revision, digest
//! and scratch path replaced by a fixed value of the same form, and
//! compared with `plugins/herdr/tests/docs/never-published/`; with
//! `TIMBER_HERDR_DOCS=write` they are written there instead. A second
//! scratch host holds a repository with an environment behind, one
//! waiting and one unread, its reports in the host's given layout, whose
//! documents are `plugins/herdr/tests/docs/environments/` alike (98, 134).
//! Needs git alone, so it is in the fast layer.

mod scratch;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use clap::Parser;
use scratch::{Scratch, MAIN, POLICY_FILE};
use serde_json::Value;
use ssh_key::rand_core::OsRng;
use ssh_key::{Algorithm, LineEnding, PrivateKey};
use timber::commands::serve::host::{name, Host};
use timber::commands::serve::{address, respond_with};
use timber::profiles::signer::host_key::HostKey;
use timber::profiles::store::layout::Layout;
use timber::{run, Cli};
use timber_core::{Reported, RepositoryName};
use timber_fakes::FakeSigner;
use timber_seams::{
    ChannelId, ChannelName, ComponentName, Digest, EnvironmentName, IndexMeta, Manifest, Moved,
    Promotion, Revision, Satisfied, Signer, SignerId, Store, Version,
};

/// Where the plugin's tests read the documents of `scene`.
fn docs(scene: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../plugins/herdr/tests/docs")
        .join(scene)
}

/// The moment every asked moment is fixed at: after the promotions.
const ASKED: u64 = 1_793_000_100;

/// The repository the host knows whole, and the one whose checkout is
/// gone.
fn widget() -> RepositoryName {
    RepositoryName {
        owner: "agentplot".into(),
        name: "widget".into(),
    }
}

fn gadget() -> RepositoryName {
    RepositoryName {
        owner: "agentplot".into(),
        name: "gadget".into(),
    }
}

/// The host of the scratch directory `dir`: its state, its bindings file
/// on a plain branch landing and the git ref ledger on `origin`, a home
/// and a data directory.
fn host_in(dir: &Path) -> Host {
    let host = Host {
        state: dir.join("state/timber"),
        config: dir.join("config/timber/config.toml"),
        home: Some(dir.join("home")),
        data: Some(dir.join("data")),
    };
    fs::create_dir_all(host.config.parent().unwrap()).unwrap();
    fs::write(
        &host.config,
        "[landing]\nbranch = { remote = \"origin\" }\n\n[ledger]\ngit_ref = { remote = \"origin\" }\n",
    )
    .unwrap();
    host
}

/// A scratch repository whose policy declares `beta` and `stable`,
/// `stable` bound to `beta`, landed on `main`; its policy revision.
fn declared(scratch: &Scratch) -> Revision {
    let repo = &scratch.repo;
    repo.write(
        POLICY_FILE,
        "outputs = [\"site\"]\n\n[projections]\nsite = \"file\"\n\n\
         [channels.beta]\nrequire = []\n\n[channels.stable]\nbound_to = \"beta\"\nrequire = []\n",
    );
    let head = Revision::new(repo.commit("feat: the channels"));
    repo.push();
    head
}

/// `channel` promoted in the ledger to each version in turn, from the
/// head it is at, sealed as the fake signer seals, under the policy at
/// `policy`.
fn promote(host: &Host, at: &Path, policy: &Revision, channel: &str, versions: &[&str]) {
    let moves: Vec<Moved> = versions
        .iter()
        .map(|version| Moved {
            version: Version::new(*version),
            index: Digest::of(format!("{version}'s index").as_bytes()),
        })
        .collect();
    promote_to(host, at, policy, channel, &moves);
}

/// `channel` promoted in the ledger to each of `moves` in turn, as
/// [`promote`] promotes it.
fn promote_to(host: &Host, at: &Path, policy: &Revision, channel: &str, moves: &[Moved]) {
    let channel = ChannelId::new(None, ChannelName::new(channel));
    let binding = host.bind(at).expect("the host binds");
    let keeper = &binding.seams.keeper;
    let gate = FakeSigner::new();
    gate.hold(&SignerId::new("gate"));
    let mut head = timber_core::channel(keeper.as_ref(), &channel).unwrap();
    for (n, to) in moves.iter().enumerate() {
        let (from, predecessor) = match &head {
            timber_core::Channel::At { head, chain, .. } => {
                (Some(head.to.clone()), chain.last().map(|(d, _)| d.clone()))
            }
            timber_core::Channel::Empty { .. } => (None, None),
        };
        let promotion = Promotion {
            channel: channel.name.clone(),
            component: channel.component_name(),
            to: to.clone(),
            from,
            predecessor: predecessor.clone(),
            asked_by: SignerId::new("gate"),
            policy: Satisfied {
                revision: policy.clone(),
                claims: vec![Digest::of(
                    format!("{}'s acceptance", to.version).as_bytes(),
                )],
            },
            at: 1_793_000_000 + n as u64,
        };
        let envelope = gate
            .sign(&promotion.claim(policy.clone()), &SignerId::new("gate"))
            .unwrap();
        keeper
            .append(&channel, &envelope, predecessor.as_ref())
            .unwrap();
        head = timber_core::channel(keeper.as_ref(), &channel).unwrap();
    }
}

/// The values a document holds that differ from run to run, each
/// replaced by a fixed one of the same form, the same value always by the
/// same one: the moments asked, every digest and revision in order of
/// first sight, and each scratch directory by its fixed path.
struct Fixed {
    paths: Vec<(String, String)>,
    digests: Vec<String>,
    revisions: Vec<String>,
}

impl Fixed {
    fn new(paths: &[(&Path, &str)]) -> Fixed {
        Fixed {
            paths: paths
                .iter()
                .map(|(path, fixed)| (path.display().to_string(), (*fixed).to_string()))
                .collect(),
            digests: Vec::new(),
            revisions: Vec::new(),
        }
    }

    fn value(&mut self, value: &mut Value) {
        match value {
            Value::String(text) => *text = self.text(text),
            Value::Array(items) => items.iter_mut().for_each(|item| self.value(item)),
            Value::Object(fields) => {
                for (key, field) in fields.iter_mut() {
                    if key == "asked_at" && field.is_u64() {
                        *field = Value::from(ASKED);
                    } else {
                        self.value(field);
                    }
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) => {}
        }
    }

    /// `text` with each scratch path, digest and revision in it fixed.
    fn text(&mut self, text: &str) -> String {
        let mut text = text.to_string();
        for (path, fixed) in &self.paths {
            text = text.replace(path.as_str(), fixed);
        }
        let mut out = String::new();
        let mut rest = text.as_str();
        while let Some(c) = rest.chars().next() {
            if let Some(hex) = rest
                .strip_prefix("sha256:")
                .and_then(|after| hex_run(after, 64))
            {
                out.push_str(&format!(
                    "sha256:{}",
                    fixed(seen(&mut self.digests, hex), 64)
                ));
                rest = &rest["sha256:".len() + 64..];
            } else if let Some(hex) = hex_run(rest, 40).filter(|_| !word_before(&out)) {
                out.push_str(&fixed(seen(&mut self.revisions, hex), 40));
                rest = &rest[40..];
            } else {
                out.push(c);
                rest = &rest[c.len_utf8()..];
            }
        }
        out
    }
}

/// The run of exactly `n` lowercase hex digits `text` starts with, ended
/// by no further letter or digit.
fn hex_run(text: &str, n: usize) -> Option<&str> {
    let run = text.get(..n)?;
    let hex = run
        .bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    let ended = !text[n..]
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphanumeric());
    (hex && ended).then_some(run)
}

/// Whether `out` ends inside a word, so a hex run would not start one.
fn word_before(out: &str) -> bool {
    out.chars()
        .last()
        .is_some_and(|c| c.is_ascii_alphanumeric())
}

/// The fixed value of `n` hex digits for the `n`th seen: its number in
/// hex, repeated, so values shown short still differ.
fn fixed(seen: usize, n: usize) -> String {
    let hex = format!("{seen:x}");
    hex.repeat(n / hex.len() + 1)[..n].to_string()
}

/// The number `value` is fixed as: its place among those seen, from 1.
fn seen(all: &mut Vec<String>, value: &str) -> usize {
    match all.iter().position(|v| v == value) {
        Some(at) => at + 1,
        None => {
            all.push(value.to_string());
            all.len()
        }
    }
}

/// The first place `got` differs from `want`, by its path in the
/// document, with both values there.
fn difference(at: &str, want: &Value, got: &Value) -> Option<String> {
    match (want, got) {
        (Value::Object(w), Value::Object(g)) => {
            let mut keys: Vec<&String> = w.keys().chain(g.keys()).collect();
            keys.sort();
            keys.dedup();
            keys.into_iter().find_map(|key| {
                let here = format!("{at}.{key}");
                match (w.get(key), g.get(key)) {
                    (Some(w), Some(g)) => difference(&here, w, g),
                    (w, g) => Some(format!("{here}: committed {w:?}, served {g:?}")),
                }
            })
        }
        (Value::Array(w), Value::Array(g)) if w.len() == g.len() => w
            .iter()
            .zip(g)
            .enumerate()
            .find_map(|(i, (w, g))| difference(&format!("{at}[{i}]"), w, g)),
        _ if want == got => None,
        _ => Some(format!("{at}: committed {want}, served {got}")),
    }
}

/// `served` written to `file` under the documents of `scene` with
/// `TIMBER_HERDR_DOCS=write`, else compared with what is there.
fn committed(scene: &str, file: &str, served: &Value) {
    let path = docs(scene).join(file);
    if std::env::var("TIMBER_HERDR_DOCS").as_deref() == Ok("write") {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut text = serde_json::to_string_pretty(served).unwrap();
        text.push('\n');
        fs::write(&path, text).unwrap();
        return;
    }
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}; TIMBER_HERDR_DOCS=write writes it", path.display()));
    let want: Value = serde_json::from_str(&text).expect("a committed document is JSON");
    if let Some(differs) = difference(file, &want, served) {
        panic!(
            "the served document differs from the one the plugin's tests render, at {differs}; \
             TIMBER_HERDR_DOCS=write writes it again"
        );
    }
}

/// The JSON body of `target` answered by `host`'s routes, with `status`.
fn served(host: &Host, target: &str, status: u16) -> Value {
    let response = respond_with(
        address(),
        host,
        &|checkout, command| host.bind_for(checkout, command),
        "GET",
        target,
    );
    let body = String::from_utf8_lossy(&response.body);
    assert_eq!(response.status, status, "{target}: {body}");
    assert_eq!(response.content_type, "application/json");
    serde_json::from_slice(&response.body).expect("a JSON document")
}

// Requirement: The pane shows what status prints on a machine that never
// published

#[test]
fn the_documents_the_plugin_renders_are_the_binarys_own() {
    let first = Scratch::new();
    let policy = declared(&first);
    let host = host_in(&first.path());
    let second = Scratch::new();
    declared(&second);
    let at = first.repo.path.clone();
    name(&host, &widget(), &at).unwrap();
    name(&host, &gadget(), &second.repo.path).unwrap();
    promote(&host, &at, &policy, "beta", &["0.1.0", "0.2.0"]);
    fs::remove_dir_all(&second.repo.path).unwrap();

    let paths = [
        (first.path(), "/scratch/widget"),
        (second.path(), "/scratch/gadget"),
    ];
    let paths: Vec<(&Path, &str)> = paths.iter().map(|(p, f)| (p.as_path(), *f)).collect();
    let mut fixed = Fixed::new(&paths);
    let mut root = served(&host, "/status.json", 200);
    let mut repository = served(&host, &format!("{}/status.json", widget().path()), 200);
    fixed.value(&mut root);
    fixed.value(&mut repository);

    let rows = root["value"]["repositories"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "{root}");
    assert_eq!(rows[0]["answer"], "unreadable", "{root}");
    assert_eq!(rows[1]["value"]["repository"]["name"], "widget", "{root}");
    let channels = &repository["value"]["channels"]["value"]["channels"];
    assert_eq!(
        channels[0]["at"]["value"]["head"]["to"]["version"], "0.2.0",
        "{repository}"
    );
    assert_eq!(
        channels[0]["at"]["value"]["chain"].as_array().map(Vec::len),
        Some(2),
        "{repository}"
    );
    assert_eq!(channels[1]["at"]["value"]["state"], "empty", "{repository}");
    // The policy declares no environment: an empty list, four counts of
    // none.
    assert_eq!(
        repository["value"]["environments"]["value"]["environments"],
        serde_json::json!([]),
        "{repository}"
    );
    assert_eq!(
        repository["value"]["published"]["answer"], "absent",
        "{repository}"
    );

    committed("never-published", "status.json", &root);
    committed(
        "never-published",
        "agentplot/widget/status.json",
        &repository,
    );

    // The repository's document is what `timber status --json` prints
    // in its checkout.
    let cli = Cli::try_parse_from(["timber", "status", "--json"]).unwrap();
    let mut out = Vec::new();
    assert_eq!(run(&cli, &host.bind(&at), &at, &mut out), 0);
    let mut printed: Value = serde_json::from_slice(&out).unwrap();
    let mut again = served(&host, &format!("{}/status.json", widget().path()), 200);
    Fixed::new(&paths).value(&mut printed);
    Fixed::new(&paths).value(&mut again);
    assert_eq!(again, printed);
}

// Requirement (herdr-plugin): Opening a row shows that repository whole,
// every environment with the notes between

fn gizmo() -> RepositoryName {
    RepositoryName {
        owner: "agentplot".into(),
        name: "gizmo".into(),
    }
}

/// When the deployer's apply reported, fixed: after the promotions.
const REPORTED: u64 = 1_793_000_050;

/// A key of `dir`'s own at `name`, its file's mode the owner's alone: the
/// key and its file.
fn key_in(dir: &Path, name: &str) -> (PrivateKey, PathBuf) {
    let key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).expect("an Ed25519 key");
    let file = dir.join(name);
    key.write_openssh_file(&file, LineEnding::LF).unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
    (key, file)
}

/// The deployer's report of `environment`, which follows `channel`,
/// running `to`, tagged at `revision`, signed by its key at `key` and
/// attached to `to`'s index in `layout`.
fn reported(
    layout: &Layout,
    key: &Path,
    (environment, channel): (&str, &str),
    (to, revision): (&Moved, &Revision),
) {
    let report = Reported {
        environment: EnvironmentName::new(environment),
        component: ComponentName::new("repository"),
        channel: ChannelName::new(channel),
        version: to.version.clone(),
        index: to.index.clone(),
        runs: to.index.clone(),
        converged_at: "2026-10-26T07:34:00Z".into(),
        at: REPORTED,
    };
    let deployer = SignerId::new("deployer");
    let envelope = HostKey::new(deployer.clone(), key)
        .sign(&report.claim(revision.clone()), &deployer)
        .expect("the deployer's key signs");
    layout.attach(&to.index, &envelope).unwrap();
}

/// A scratch host knowing `agentplot/gizmo` alone, whose policy declares
/// `beta` and `stable`, a deployer it lets claim a report and `ops`, and
/// three environments: `srv` on `beta`, behind, running 0.1.0 while
/// `beta` is at 0.2.0, whose commit holds a feature of its own; `desk`,
/// which waits on `ops`, its own channel at 0.1.0 and running it while
/// `beta` is at 0.2.0; and `db` on `beta`, which no report names. Both
/// versions are tagged and landed, their indexes put in the host's given
/// layout.
#[test]
fn the_environments_the_plugin_renders_are_the_binarys_own() {
    let scratch = Scratch::new();
    let host = host_in(&scratch.path());
    let (deployer, deployer_key) = key_in(&scratch.path(), "deployer.key");
    let (ops, _) = key_in(&scratch.path(), "ops.key");
    let repo = &scratch.repo;
    repo.write(
        POLICY_FILE,
        &format!(
            "outputs = [\"site\"]\n\n[projections]\nsite = \"file\"\n\n\
             [signers.deployer]\nssh = \"{}\"\nkinds = [\"report\"]\n\n\
             [signers.ops]\nssh = \"{}\"\nkinds = [\"review\"]\n\n\
             [channels.beta]\nrequire = []\n\n[channels.stable]\nbound_to = \"beta\"\nrequire = []\n\n\
             [environments.srv]\nchannel = \"beta\"\n\n\
             [environments.desk]\nchannel = \"beta\"\npolicy = \"manual\"\napprovers = [\"ops\"]\n\n\
             [environments.db]\nchannel = \"beta\"\n",
            deployer.public_key().to_openssh().unwrap(),
            ops.public_key().to_openssh().unwrap()
        ),
    );
    let policy = Revision::new(repo.commit("feat: the environments"));
    repo.write("site/index.html", "<p>gizmo</p>");
    let second = repo.commit("feat: the gizmo turns");
    repo.git(&["tag", "v0.1.0", policy.as_str()]);
    repo.git(&["tag", "v0.2.0", &second]);
    repo.git(&["push", "-q", "origin", MAIN, "v0.1.0", "v0.2.0"]);
    let at = repo.path.clone();
    name(&host, &gizmo(), &at).unwrap();

    let layout = Layout::create(scratch.path().join("data/timber/layout")).unwrap();
    let moves: Vec<Moved> = ["0.1.0", "0.2.0"]
        .iter()
        .map(|version| {
            let meta = IndexMeta::new(
                ComponentName::new("repository"),
                Version::new(*version),
                vec![],
            )
            .unwrap();
            Moved {
                version: Version::new(*version),
                index: layout.put_artifact(&Manifest::Index(meta)).unwrap().digest,
            }
        })
        .collect();
    promote_to(&host, &at, &policy, "beta", &moves);
    promote_to(&host, &at, &policy, "desk", &moves[..1]);
    reported(
        &layout,
        &deployer_key,
        ("srv", "beta"),
        (&moves[0], &policy),
    );
    reported(
        &layout,
        &deployer_key,
        ("desk", "desk"),
        (&moves[0], &policy),
    );

    let paths = [(scratch.path(), "/scratch/gizmo")];
    let paths: Vec<(&Path, &str)> = paths.iter().map(|(p, f)| (p.as_path(), *f)).collect();
    let mut fixed = Fixed::new(&paths);
    let mut root = served(&host, "/status.json", 200);
    let mut repository = served(&host, &format!("{}/status.json", gizmo().path()), 200);
    fixed.value(&mut root);
    fixed.value(&mut repository);

    let row = &root["value"]["repositories"][0];
    assert_eq!(
        row["value"]["environments"]["value"],
        serde_json::json!({ "converged": 0, "behind": 1, "waiting": 1, "unreadable": 1 }),
        "{root}"
    );
    let part = &repository["value"]["environments"]["value"];
    let state = |name: &str| {
        part["environments"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["environment"] == name)
            .unwrap_or_else(|| panic!("no environment {name}: {repository}"))["state"]
            .clone()
    };
    assert_eq!(state("srv")["value"], "behind", "{repository}");
    assert_eq!(state("desk")["value"], "waiting", "{repository}");
    assert_eq!(state("db")["answer"], "absent", "{repository}");
    let srv = part["environments"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["environment"] == "srv")
        .unwrap();
    let notes = srv["between"]["value"]["versions"][0]["notes"]["value"]
        .as_str()
        .expect("0.2.0's notes");
    assert!(notes.contains("- The gizmo turns\n"), "{notes}");

    committed("environments", "status.json", &root);
    committed("environments", "agentplot/gizmo/status.json", &repository);

    // The repository's document is what `timber status --json` prints
    // in its checkout.
    let cli = Cli::try_parse_from(["timber", "status", "--json"]).unwrap();
    let mut out = Vec::new();
    assert_eq!(run(&cli, &host.bind(&at), &at, &mut out), 0);
    let mut printed: Value = serde_json::from_slice(&out).unwrap();
    let mut again = served(&host, &format!("{}/status.json", gizmo().path()), 200);
    Fixed::new(&paths).value(&mut printed);
    Fixed::new(&paths).value(&mut again);
    assert_eq!(again, printed);
}
