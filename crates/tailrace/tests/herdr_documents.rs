//! Spec herdr-plugin: the documents the herdr plugin's tests render are
//! the binary's own (57, 132; design D6 part 2). A scratch host on a
//! machine that never published: a remote and its clone, the policy
//! declaring `beta` and `stable`, two promotions of `beta` in the git ref
//! ledger, no record of a publish, and a second named repository whose
//! checkout is gone. The root's and the repository's answers are asked of
//! the process's routes with no port bound, each time, revision, digest
//! and scratch path replaced by a fixed value of the same form, and
//! compared with `plugins/herdr/tests/docs/never-published/`; with
//! `TAILRACE_HERDR_DOCS=write` they are written there instead. Needs git
//! alone, so it is in the fast layer.

mod scratch;

use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use scratch::{Scratch, POLICY_FILE};
use serde_json::Value;
use tailrace::commands::serve::host::{name, Host};
use tailrace::commands::serve::respond_with;
use tailrace::{run, Cli};
use tailrace_core::RepositoryName;
use tailrace_fakes::FakeSigner;
use tailrace_seams::{
    ChannelId, ChannelName, Digest, Moved, Promotion, Revision, Satisfied, Signer, SignerId,
    Version,
};

/// Where the plugin's tests read the documents.
fn docs() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins/herdr/tests/docs/never-published")
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
        state: dir.join("state/tailrace"),
        config: dir.join("config/tailrace/config.toml"),
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
    let channel = ChannelId::new(None, ChannelName::new(channel));
    let binding = host.bind(at).expect("the host binds");
    let keeper = &binding.seams.keeper;
    let gate = FakeSigner::new();
    gate.hold(&SignerId::new("gate"));
    let mut head = tailrace_core::channel(keeper.as_ref(), &channel).unwrap();
    for (n, version) in versions.iter().enumerate() {
        let (from, predecessor) = match &head {
            tailrace_core::Channel::At { head, chain, .. } => {
                (Some(head.to.clone()), chain.last().map(|(d, _)| d.clone()))
            }
            tailrace_core::Channel::Empty { .. } => (None, None),
        };
        let promotion = Promotion {
            channel: channel.name.clone(),
            component: channel.component_name(),
            to: Moved {
                version: Version::new(*version),
                index: Digest::of(format!("{version}'s index").as_bytes()),
            },
            from,
            predecessor: predecessor.clone(),
            asked_by: SignerId::new("gate"),
            policy: Satisfied {
                revision: policy.clone(),
                claims: vec![Digest::of(format!("{version}'s acceptance").as_bytes())],
            },
            at: 1_793_000_000 + n as u64,
        };
        let envelope = gate
            .sign(&promotion.claim(policy.clone()), &SignerId::new("gate"))
            .unwrap();
        keeper
            .append(&channel, &envelope, predecessor.as_ref())
            .unwrap();
        head = tailrace_core::channel(keeper.as_ref(), &channel).unwrap();
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

/// `served` written to `file` under the documents with
/// `TAILRACE_HERDR_DOCS=write`, else compared with what is there.
fn committed(file: &str, served: &Value) {
    let path = docs().join(file);
    if std::env::var("TAILRACE_HERDR_DOCS").as_deref() == Ok("write") {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut text = serde_json::to_string_pretty(served).unwrap();
        text.push('\n');
        fs::write(&path, text).unwrap();
        return;
    }
    let text = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "{}: {e}; TAILRACE_HERDR_DOCS=write writes it",
            path.display()
        )
    });
    let want: Value = serde_json::from_str(&text).expect("a committed document is JSON");
    if let Some(differs) = difference(file, &want, served) {
        panic!(
            "the served document differs from the one the plugin's tests render, at {differs}; \
             TAILRACE_HERDR_DOCS=write writes it again"
        );
    }
}

/// The JSON body of `target` answered by `host`'s routes, with `status`.
fn served(host: &Host, target: &str, status: u16) -> Value {
    let response = respond_with(host, &|checkout| host.bind(checkout), "GET", target);
    assert_eq!(response.status, status, "{target}: {}", response.body);
    assert_eq!(response.content_type, "application/json");
    serde_json::from_str(&response.body).expect("a JSON document")
}

// Requirement: The pane shows what status prints on a machine that never published

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
    assert_eq!(
        repository["value"]["environments"]["answer"], "unreadable",
        "{repository}"
    );
    assert_eq!(
        repository["value"]["published"]["answer"], "absent",
        "{repository}"
    );

    committed("status.json", &root);
    committed("agentplot/widget/status.json", &repository);

    // The repository's document is what `tailrace status --json` prints
    // in its checkout.
    let cli = Cli::try_parse_from(["tailrace", "status", "--json"]).unwrap();
    let mut out = Vec::new();
    assert_eq!(run(&cli, &host.bind(&at), &at, &mut out), 0);
    let mut printed: Value = serde_json::from_slice(&out).unwrap();
    let mut again = served(&host, &format!("{}/status.json", widget().path()), 200);
    Fixed::new(&paths).value(&mut printed);
    Fixed::new(&paths).value(&mut again);
    assert_eq!(again, printed);
}
