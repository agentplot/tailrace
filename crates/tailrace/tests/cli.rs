//! Spec command-line: each command calls the one core and renders its
//! answer, over the fakes and as the built binary (design D11, D12).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Parser;
use serde_json::Value;
use tailrace::bindings::Teller;
use tailrace::profiles::policy::file::PolicyFile;
use tailrace::{run, Binding, Cli};
use tailrace_core::{
    accept, answer_json, artifact_name, in_force, promote, publish, push, sync_checkout, write,
    AcceptAsk, Ask, CheckoutAsk, Entry, History, InForceAsk, Judged, Kept, Layouts, Moved,
    PromoteAsk, PushAsk, To, ToldBy,
};
use tailrace_fakes::{Fakes, Fault, StoreOp, Times};
use tailrace_seams::{
    Answer, Artifact, ArtifactMeta, AskedOf, Audience, Branch, BuildId, ChannelId, ChannelName,
    Check, Claim, ClaimKind, ComponentName, Digest, Ecosystem, EcosystemName, Held, IndexMeta,
    Manifest, NamedSigner, Outcome, Output, OutputName, Pinned, Promotion, Report, Required,
    RequiredClaim, Revision, Seam, Signer, SignerId, Source, Store, Verdict, Version,
};
use tempfile::TempDir;

/// The store as the host's bindings name it, which the record keeps.
fn store() -> Source {
    Source::new(Seam::Store, "layout /srv/tailrace/layout")
}

/// The job the host's bindings name as what tells the gate, so no
/// publish here asks it.
const JOB: &str = "the gate's job in another place";

/// The one component of the repository, its one output packed in `eco`.
const POLICY: &str = "outputs = [\"site\"]\n\n[projections]\nsite = \"eco\"\n";

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "A. Uthor")
        .env("GIT_AUTHOR_EMAIL", "author@example.com")
        .env("GIT_COMMITTER_NAME", "A. Uthor")
        .env("GIT_COMMITTER_EMAIL", "author@example.com")
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .unwrap()
        .trim_end()
        .to_string()
}

/// The day `at` was committed in the repository at `dir`, in UTC, as a
/// template's date filter prints its time.
fn day(dir: &Path, at: &Revision) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "show",
            "-s",
            "--date=format-local:%Y-%m-%d",
            "--format=%cd",
            at.as_str(),
        ])
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("TZ", "UTC")
        .output()
        .expect("git runs");
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

/// A repository on `main` whose shared history is a bare one beside it,
/// with the policy and one feature committed, and the fakes bound to
/// publish its head.
struct Host {
    _root: TempDir,
    repo: PathBuf,
    head: Revision,
    fakes: Fakes,
}

impl Host {
    fn new() -> Host {
        let root = TempDir::new().unwrap();
        git(
            root.path(),
            &["init", "-q", "--bare", "-b", "main", "shared.git"],
        );
        git(root.path(), &["clone", "-q", "shared.git", "repo"]);
        let repo = root.path().join("repo");
        git(&repo, &["config", "branch.main.remote", "origin"]);
        git(&repo, &["config", "branch.main.merge", "refs/heads/main"]);
        fs::create_dir_all(repo.join(".config")).unwrap();
        fs::write(repo.join(".config/tailrace.toml"), POLICY).unwrap();
        fs::create_dir_all(repo.join("site")).unwrap();
        fs::write(repo.join("site/index.html"), "<p>hello</p>").unwrap();
        git(
            &repo,
            &["add", "--", ".config/tailrace.toml", "site/index.html"],
        );
        git(&repo, &["commit", "-q", "-m", "feat: the site"]);
        let head = Revision::new(git(&repo, &["rev-parse", "HEAD"]));
        let fakes = fakes_at(&head);
        Host {
            _root: root,
            repo,
            head,
            fakes,
        }
    }

    fn binding(&self) -> Binding {
        Binding {
            seams: self.fakes.seams(),
            store: store(),
            signer: Some(SignerId::new("host")),
            gate: Teller::Job(JOB.into()),
            trust: Vec::new(),
        }
    }

    /// `args` asked through `run` over the fakes: the exit code and what
    /// it printed.
    fn run(&self, args: &[&str]) -> (i32, String) {
        let cli = Cli::try_parse_from(std::iter::once("tailrace").chain(args.iter().copied()))
            .expect("the arguments parse");
        let mut out = Vec::new();
        let code = run(&cli, &Ok(self.binding()), &self.repo, &mut out);
        (code, String::from_utf8(out).unwrap())
    }

    /// The scratch directory the repository is in.
    fn root(&self) -> PathBuf {
        self.repo.parent().unwrap().to_path_buf()
    }

    /// Where the built binary looks for the host's bindings file: under
    /// a configuration directory of the scratch directory's own.
    fn bindings_file(&self) -> PathBuf {
        self.root().join("config/tailrace/config.toml")
    }

    /// Where the built binary's given layout is: under a data directory
    /// of the scratch directory's own.
    fn given_layout(&self) -> PathBuf {
        self.root().join("data/tailrace/layout")
    }

    /// Where the built binary's given host key is: under the same data
    /// directory.
    fn given_key(&self) -> PathBuf {
        self.root().join("data/tailrace/host/signing.key")
    }

    /// `text` as the host's bindings file.
    fn bind(&self, text: &str) {
        let file = self.bindings_file();
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, text).unwrap();
    }

    /// The built binary run in the repository, its home and its
    /// configuration, data and state directories in the scratch
    /// directory, so it reads no bindings file and makes no layout or
    /// record of the host running the tests.
    fn binary(&self, args: &[&str]) -> std::process::Output {
        let home = self.root().join("home");
        fs::create_dir_all(&home).unwrap();
        Command::new(env!("CARGO_BIN_EXE_tailrace"))
            .args(args)
            .current_dir(&self.repo)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("HOME", &home)
            .env("XDG_CONFIG_HOME", self.root().join("config"))
            .env("XDG_DATA_HOME", self.root().join("data"))
            .env("XDG_STATE_HOME", self.root().join("state"))
            .output()
            .expect("the binary runs")
    }

    fn calls(&self) -> usize {
        self.fakes.store.calls().len()
            + self.fakes.build.calls().len()
            + self.fakes.landing.calls().len()
            + self.fakes.signer.calls().len()
            + self.fakes.policy.calls().len()
    }
}

/// The fakes bound to publish `head`: landed on `main`, its one output
/// built and passing, the host's identity at hand, and acceptance
/// requiring nothing.
fn fakes_at(head: &Revision) -> Fakes {
    let mut fakes = Fakes::new();
    fakes.landing.land(&Branch::new("main"), head);
    fakes.signer.hold(&SignerId::new("host"));
    fakes.projection(&Ecosystem::new("eco"));
    fakes.build.made(
        head,
        &OutputName::new("site"),
        Report {
            build: tailrace_seams::BuildId::new("site-build"),
            outcome: tailrace_seams::Outcome::Pass,
            files: vec!["dist/site.tar".into()],
        },
    );
    fakes.policy.require(
        Audience::Acceptance,
        Required::Acceptance { claims: vec![] },
    );
    fakes
}

// Requirement: Each command calls the one core and renders its answer

#[test]
fn publish_through_the_command_line_prints_the_cores_answer_value_for_value() {
    let host = Host::new();
    let (code, printed) = host.run(&["publish", "HEAD", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let mut printed: Value = serde_json::from_str(&printed).unwrap();
    // The one thing beside the core's answer: the job that tells the
    // gate.
    let gate = printed["value"]["components"][0]
        .as_object_mut()
        .unwrap()
        .remove("gate")
        .expect("the offered version names what tells the gate");
    assert_eq!(gate, serde_json::json!({ "told_by": "job", "job": JOB }));

    let history = History::open(&host.repo).unwrap();
    let core = publish(
        &host.fakes.seams(),
        &history,
        &Kept::at(&history),
        &Ask {
            branch: Branch::new("main"),
            revision: host.head.clone(),
            component: None,
            signer: SignerId::new("host"),
            store: store(),
        },
    );
    assert_eq!(printed, answer_json(&core));
    assert_eq!(
        printed["value"]["components"][0]["value"]["version"],
        "0.1.0"
    );
}

// Requirement: A publish answers what a program reads

#[test]
fn publish_prints_the_notes_beneath_the_versions_line_and_names_them_in_its_json() {
    let host = Host::new();
    let (code, printed) = host.run(&["publish", "HEAD"]);
    assert_eq!(code, 0, "{printed}");
    let (code, json) = host.run(&["publish", "HEAD", "--json"]);
    assert_eq!(code, 0, "{json}");
    let json: Value = serde_json::from_str(&json).unwrap();
    let notes = &json["value"]["components"][0]["value"]["notes"];
    let text = notes["text"].as_str().expect("the notes' text");
    // The repository keeps no template, so git-cliff's own default
    // renders them, dated the day of the tagged revision.
    let day = day(&host.repo, &host.head);
    assert_eq!(
        text,
        format!("## [0.1.0] - {day}\n\n### 🚀 Features\n\n- The site\n")
    );

    let Held::Bytes(bytes) = host
        .fakes
        .store
        .read(&Digest::parse(notes["bytes"].as_str().unwrap()).unwrap())
        .unwrap()
    else {
        panic!("no bytes under {}", notes["bytes"])
    };
    assert_eq!(bytes, text.as_bytes());
    let envelope = notes["claims"][0]["envelope"].as_str().unwrap();
    let digest = Digest::parse(notes["digest"].as_str().unwrap()).unwrap();
    let attached = host.fakes.store.claims_of(&digest).unwrap();
    assert_eq!(
        attached
            .iter()
            .map(|e| e.digest().as_str().to_string())
            .collect::<Vec<_>>(),
        [envelope]
    );

    let lines: Vec<&str> = printed.lines().collect();
    let version = lines
        .iter()
        .position(|l| l.starts_with("repository 0.1.0 (v0.1.0)"))
        .expect(&printed);
    let at = lines
        .iter()
        .position(|l| *l == format!("  notes {digest}"))
        .expect(&printed);
    assert!(version < at, "{printed}");
    assert_eq!(
        lines[at + 1],
        format!("    provenance by host, envelope {envelope}")
    );
    let shown: Vec<String> = text
        .lines()
        .map(|l| format!("    {l}").trim_end().to_string())
        .collect();
    let end = at + 2 + shown.len();
    assert_eq!(lines[at + 2..end], shown, "{printed}");
    assert!(lines[end].starts_with("  gate: "), "{printed}");
}

// Requirement: Sync fills the checkout's cache from a landed revision

/// A passing verdict of `test` on `build` at the host's head, signed as
/// the landing signs, recorded.
fn passed(host: &Host, build: &str) {
    host.fakes
        .landing
        .verdict(&host.head, verdict(&host.head, build));
}

/// A passing verdict of `test` on `build` at `head`, signed as the
/// landing signs.
fn verdict(head: &Revision, build: &str) -> Verdict {
    let id = SignerId::new("landing");
    let theirs = tailrace_fakes::FakeSigner::new();
    theirs.hold(&id);
    let claim = Claim {
        subject: Digest::of(build.as_bytes()),
        kind: ClaimKind::Verdict,
        revision: head.clone(),
        build: Some(BuildId::new(build)),
        body: serde_json::json!({ "check": "test", "outcome": Outcome::Pass }),
    };
    Verdict {
        check: Check::new("test"),
        build: BuildId::new(build),
        outcome: Outcome::Pass,
        signer: id.clone(),
        envelope: theirs.sign(&claim, &id).unwrap(),
        found: format!("the fake landing's record of test on {build}"),
    }
}

#[test]
fn sync_with_one_entry_gone_prints_one_rendered_and_one_absent_and_exits_0() {
    let host = Host::new();
    passed(&host, "b7e1");
    passed(&host, "c3d4");
    let site = Output {
        name: OutputName::new("site"),
        files: vec![],
    };
    write(&host.fakes.seams(), &site, &BuildId::new("c3d4")).unwrap();

    let (code, printed) = host.run(&["sync", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    let entries = &printed["value"]["entries"];
    assert_eq!(entries[0]["build"], "b7e1");
    assert_eq!(entries[0]["answer"], "absent");
    assert_eq!(entries[1]["build"], "c3d4");
    assert_eq!(entries[1]["answer"], "ok");
    assert_eq!(printed["value"]["revision"], host.head.as_str());

    let history = History::open(&host.repo).unwrap();
    let core = sync_checkout(
        &host.fakes.seams(),
        &Layouts::at(&history),
        &CheckoutAsk {
            branch: Branch::new("main"),
            revision: host.head.clone(),
            tree: history.workdir().unwrap(),
            when: 0,
            trusted: vec![],
        },
    );
    assert_eq!(printed, answer_json(&core));
    assert_eq!(printed["value"]["layouts"], serde_json::json!([]));

    let (code, printed) = host.run(&["sync"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    assert!(lines[0].starts_with("b7e1 absent: "), "{printed}");
    assert!(
        lines[1].starts_with("c3d4 rendered by eco, sha256:"),
        "{printed}"
    );
}

#[test]
fn sync_on_a_branch_the_repository_lacks_is_absent_and_one_that_did_not_land_it_refuses() {
    let host = Host::new();
    let (code, printed) = host.run(&["sync", "--branch", "release", "--json"]);
    assert_eq!(code, 4, "the repository holds no branch release: {printed}");
    let (code, printed) = host.run(&["sync", "HEAD", "--branch", "release", "--json"]);
    assert_eq!(code, 1, "{printed}");
    let json: Value = serde_json::from_str(&printed).unwrap();
    assert_eq!(json["refused"]["why"]["not_landed"]["branch"], "release");
}

/// `version` of `name` in `eco` put in the host's store with its index,
/// the name a pin resolves through moved to it, and, where `accepted`,
/// the gate's acceptance attached to the index.
fn published(host: &Host, name: &str, version: &str, accepted: bool) {
    let store = &host.fakes.store;
    let meta = ArtifactMeta::new(
        Ecosystem::new("eco"),
        EcosystemName::new(name),
        Version::new(version),
    )
    .unwrap();
    let layer = store
        .put_bytes(format!("{name} {version}").as_bytes())
        .unwrap()
        .digest;
    let artifact = store
        .put_artifact(&Manifest::Artifact {
            meta: meta.clone(),
            layers: vec![layer],
        })
        .unwrap()
        .digest;
    let index = IndexMeta::new(
        ComponentName::new(name),
        Version::new(version),
        vec![Artifact {
            digest: artifact,
            meta,
        }],
    )
    .unwrap();
    let index = store.put_artifact(&Manifest::Index(index)).unwrap().digest;
    let pointer = artifact_name(
        &Ecosystem::new("eco"),
        &EcosystemName::new(name),
        &Version::new(version),
    );
    store.name(&pointer, &index, None).unwrap();
    if accepted {
        let gate = SignerId::new("gate");
        let theirs = tailrace_fakes::FakeSigner::new();
        theirs.hold(&gate);
        let claim = Claim {
            subject: index.clone(),
            kind: ClaimKind::Acceptance,
            revision: Revision::new("p1b11s4"),
            build: None,
            body: Value::Null,
        };
        store
            .attach(&index, &theirs.sign(&claim, &gate).unwrap())
            .unwrap();
    }
}

/// The host's checkout pinning `pkg 1.1.0`, which the gate accepted,
/// `lib 2.0.0`, which it did not, and `left-pad 1.3.0`, which the store
/// does not name, under a policy for layouts requiring the gate's
/// acceptance.
fn pinning_one_of_each(host: &Host) {
    published(host, "pkg", "1.1.0", true);
    published(host, "lib", "2.0.0", false);
    let tree = History::open(&host.repo).unwrap().workdir().unwrap();
    let pin = |name: &str, version: &str| Pinned {
        name: EcosystemName::new(name),
        version: Version::new(version),
    };
    host.fakes.projections[&Ecosystem::new("eco")].lockfile(
        &tree.display().to_string(),
        &[
            pin("pkg", "1.1.0"),
            pin("lib", "2.0.0"),
            pin("left-pad", "1.3.0"),
        ],
    );
    host.fakes.policy.require(
        Audience::Layout,
        Required::Layout {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Acceptance,
                check: None,
                signers: vec![NamedSigner {
                    id: SignerId::new("gate"),
                    verified_by: "the gate's key".into(),
                    kinds: vec![ClaimKind::Acceptance],
                    checks: vec![],
                }],
                asked_of: AskedOf::Every,
            }],
            given: false,
            signers: vec![],
        },
    );
}

#[test]
fn sync_with_a_version_refused_prints_the_cores_answer_and_exits_1() {
    let host = Host::new();
    pinning_one_of_each(&host);
    let (code, printed) = host.run(&["sync", "--json"]);
    assert_eq!(code, 1, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    let layout = &printed["value"]["layouts"][0];
    assert_eq!(layout["ecosystem"], "eco");
    let pins = &layout["value"]["refreshed"]["pins"];
    let answers: Vec<(&str, &str)> = pins
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p["name"].as_str().unwrap(), p["answer"].as_str().unwrap()))
        .collect();
    assert_eq!(
        answers,
        [("pkg", "ok"), ("lib", "refused"), ("left-pad", "absent")]
    );
    let wanting = &pins[1]["refused"]["why"]["version_wanting"];
    assert_eq!(wanting["version"]["version"], "2.0.0");
    assert_eq!(wanting["wanting"]["kind"], "acceptance");
    assert_eq!(wanting["checked"], serde_json::json!(["gate"]));

    let history = History::open(&host.repo).unwrap();
    let when = layout["value"]["refreshed"]["rendered_at"]
        .as_u64()
        .unwrap();
    let core = sync_checkout(
        &host.fakes.seams(),
        &Layouts::at(&history),
        &CheckoutAsk {
            branch: Branch::new("main"),
            revision: host.head.clone(),
            tree: history.workdir().unwrap(),
            when,
            trusted: vec![],
        },
    );
    assert_eq!(printed, answer_json(&core));

    let (code, printed) = host.run(&["sync"]);
    assert_eq!(code, 1, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    assert!(
        lines[1].starts_with("layout eco at ")
            && lines[1].ends_with(&format!(", rendered under its own policy at {}", host.head)),
        "{printed}"
    );
    assert!(
        lines[2].starts_with("  pkg 1.1.0 held, sha256:"),
        "{printed}"
    );
    assert!(lines[3].starts_with("  lib 2.0.0: "), "{printed}");
    assert!(lines[3].contains("acceptance"), "{printed}");
    assert_eq!(lines[4], "  1 pin the store does not name", "{printed}");
}

#[test]
fn sync_whose_layout_cannot_be_refreshed_prints_what_it_holds_and_the_stores_words_and_exits_3() {
    let host = Host::new();
    pinning_one_of_each(&host);
    let (code, printed) = host.run(&["sync"]);
    assert_eq!(code, 1, "{printed}");
    host.fakes.store.fault(
        StoreOp::Resolve,
        Fault::Unreadable("connection refused".into()),
        Times::UntilHealed,
    );
    let (code, printed) = host.run(&["sync"]);
    assert_eq!(code, 3, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    assert!(
        lines[1].contains(" left as it was, rendered 20")
            && lines[1].ends_with(&format!("Z under its own policy at {}", host.head)),
        "{printed}"
    );
    assert!(
        lines[2].starts_with("  pkg 1.1.0 held, sha256:"),
        "{printed}"
    );
    assert!(lines.contains(&"connection refused"), "{printed}");
    assert!(
        !printed.contains("lib 2.0.0"),
        "a refusal was printed: {printed}"
    );
}

// Requirement: Push takes a landed revision

/// `fakes` with the host's head landed, passing verdicts on `b7e1` and
/// `c3d4` there, and no archive of `b7e1` on the host.
fn landed_with_one_archive_missing(fakes: &mut Fakes, head: &Revision) {
    fakes.landing.land(&Branch::new("main"), head);
    for build in ["b7e1", "c3d4"] {
        fakes.landing.verdict(head, verdict(head, build));
    }
    fakes
        .projection(&Ecosystem::new("eco"))
        .no_archive(&BuildId::new("b7e1"));
}

#[test]
fn push_with_one_archive_missing_prints_one_written_and_one_absent_and_exits_0() {
    let mut host = Host::new();
    let head = host.head.clone();
    landed_with_one_archive_missing(&mut host.fakes, &head);

    let (code, printed) = host.run(&["push", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    let entries = &printed["value"]["entries"];
    assert_eq!(entries[0]["build"], "b7e1");
    assert_eq!(entries[0]["answer"], "absent");
    assert_eq!(entries[1]["build"], "c3d4");
    assert_eq!(entries[1]["answer"], "ok");
    assert_eq!(entries[1]["value"]["held_at"], Value::Null);
    let digest = entries[1]["value"]["entry"]["digest"].as_str().unwrap();
    assert!(digest.starts_with("sha256:"), "{printed}");
    assert_eq!(printed["value"]["revision"], host.head.as_str());

    let mut twin = Fakes::new();
    landed_with_one_archive_missing(&mut twin, &head);
    let core = push(
        &twin.seams(),
        &PushAsk {
            branch: Branch::new("main"),
            revision: head,
        },
    );
    assert_eq!(printed, answer_json(&core));

    let (code, printed) = host.run(&["push"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    assert_eq!(lines[0], "b7e1 absent: the archive of b7e1", "{printed}");
    assert_eq!(
        lines[1],
        format!("c3d4 held already at {digest}"),
        "{printed}"
    );
}

#[test]
fn the_built_binary_on_a_host_with_no_bindings_file_pushes_through_the_given_landing() {
    let host = Host::new();
    let file = host.bindings_file();
    let out = host.binary(&["push", "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["answer"], "refused");
    assert_eq!(
        json["refused"]["why"]["not_landed"]["revision"],
        host.head.as_str()
    );
    assert_eq!(git(&host.repo, &["status", "--porcelain"]), "");
    let history = History::open(&host.repo).unwrap();
    assert!(
        !Kept::at(&history).dir().exists(),
        "the push kept something"
    );
    assert!(!file.exists(), "the push wrote a bindings file");
    assert!(!host.given_key().exists(), "the push made a key");
}

// Requirement: Rebuild takes a landed revision

#[test]
fn rebuild_after_the_record_is_deleted_writes_its_entry_and_status_answers_as_before() {
    let host = Host::new();
    assert_eq!(host.run(&["publish", "HEAD"]).0, 0);
    let published = || {
        let (code, printed) = host.run(&["status", "--json"]);
        assert_eq!(code, 0, "{printed}");
        let json: Value = serde_json::from_str(&printed).unwrap();
        json["value"]["published"].clone()
    };
    let before = published();
    assert_eq!(before["answer"], "ok", "{before}");
    let history = History::open(&host.repo).unwrap();
    let record = Kept::at(&history).dir().to_path_buf();
    fs::remove_dir_all(&record).unwrap();
    assert_eq!(
        published()["answer"],
        "absent",
        "the record was not deleted"
    );

    let (code, printed) = host.run(&["rebuild", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let json: Value = serde_json::from_str(&printed).unwrap();
    assert_eq!(json["answer"], "ok");
    assert_eq!(json["value"]["record"], record.display().to_string());
    let entries = &json["value"]["entries"];
    assert_eq!(entries.as_array().map(Vec::len), Some(1), "{printed}");
    assert_eq!(entries[0]["answer"], "ok");
    assert_eq!(entries[0]["value"], "wrote");
    assert_eq!(entries[0]["entry"]["tag"], "v0.1.0");
    assert_eq!(entries[0]["entry"]["revision"], host.head.as_str());

    assert_eq!(
        published(),
        before,
        "status answers otherwise after the rebuild"
    );

    let (code, printed) = host.run(&["rebuild"]);
    assert_eq!(code, 0, "{printed}");
    assert_eq!(
        printed.trim_end(),
        format!("repository 0.1.0 (v0.1.0) from {}: held", host.head)
    );
}

// Requirement: Every answer is readable by a program on request

#[test]
fn a_refusal_is_one_json_document_naming_what_its_clause_says() {
    let host = Host::new();
    let (code, printed) = host.run(&["publish", "HEAD", "--branch", "release", "--json"]);
    assert_eq!(code, 1);
    let json: Value = serde_json::from_str(&printed).expect("one JSON document");
    assert_eq!(json["answer"], "refused");
    let refused = &json["refused"];
    assert_eq!(refused["source"]["seam"], "landing");
    assert_eq!(refused["why"]["not_landed"]["branch"], "release");
    assert_eq!(refused["why"]["not_landed"]["revision"], host.head.as_str());
}

// Requirement: Absent, refused and unreadable are three exits

#[test]
fn three_answers_exit_three_ways_and_the_sources_words_are_printed() {
    let host = Host::new();
    let (refused, _) = host.run(&["publish", "HEAD", "--branch", "release"]);
    let (absent, _) = host.run(&["verify", "9.9.9"]);
    assert_eq!(host.run(&["publish", "HEAD"]).0, 0);
    host.fakes.store.fault(
        StoreOp::ClaimsOf,
        Fault::Unreadable("connection refused".into()),
        Times::UntilHealed,
    );
    let (unreadable, printed) = host.run(&["verify", "0.1.0"]);
    assert_eq!((refused, absent, unreadable), (1, 4, 3));
    assert!(
        printed.lines().any(|l| l == "connection refused"),
        "{printed}"
    );
}

// Requirement: Publish and verify take what the statement gives them

#[test]
fn a_version_given_to_publish_is_an_argument_it_cannot_read() {
    let host = Host::new();
    let err = Cli::try_parse_from(["tailrace", "publish", "HEAD", "--version", "1.0.0"])
        .expect_err("publish took a version");
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("--version"), "{err}");

    let out = host.binary(&["publish", "HEAD", "--version", "1.0.0"]);
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--version"), "{stderr}");
    assert_eq!(host.calls(), 0, "the core was asked");
}

#[test]
fn the_binary_says_its_revision_and_no_version() {
    let host = Host::new();
    let out = host.binary(&["--version"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim_end(),
        format!("tailrace {}", tailrace::VERSION_LINE)
    );
}

// Requirement: Status needs no seam

#[test]
fn the_built_binary_answers_status_from_the_hosts_record() {
    let host = Host::new();
    let index = Digest::of(b"the index");
    let history = History::open(&host.repo).unwrap();
    Kept::at(&history)
        .write(&Entry {
            component: ComponentName::new("repository"),
            version: Version::new("0.1.0"),
            tag: "v0.1.0".into(),
            revision: host.head.clone(),
            branch: Branch::new("main"),
            index: index.clone(),
            store: store(),
            notes: None,
        })
        .unwrap();

    assert!(!host.bindings_file().exists());
    let out = host.binary(&["status", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["answer"], "ok");
    let published = &json["value"]["published"];
    assert_eq!(published["answer"], "ok", "{json}");
    let entry = &published["value"]["components"][0]["versions"][0];
    assert_eq!(entry["component"], "repository");
    assert_eq!(entry["version"], "0.1.0");
    assert_eq!(entry["revision"], host.head.as_str());
    assert_eq!(entry["index"], index.as_str());
    assert!(
        json["value"]["channels"]["answer"].is_string(),
        "no channels' part: {json}"
    );
    assert_eq!(json["value"]["environments"]["answer"], "unreadable");
}

// Requirement: Verify shows the notes the index names

#[test]
fn verify_prints_the_notes_after_the_claims() {
    let host = Host::new();
    let (code, published) = host.run(&["publish", "HEAD", "--json"]);
    assert_eq!(code, 0, "{published}");
    let published: Value = serde_json::from_str(&published).unwrap();
    let notes = &published["value"]["components"][0]["value"]["notes"];
    let text = notes["text"].as_str().expect("the notes' text");

    let (code, printed) = host.run(&["verify", "0.1.0"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    let at = lines
        .iter()
        .position(|l| *l == format!("notes {}", notes["digest"].as_str().unwrap()))
        .expect(&printed);
    assert!(
        lines[1..at].iter().all(|l| l.starts_with("  ")) && at > 1,
        "the notes are not after the claims: {printed}"
    );
    let shown: Vec<String> = text
        .lines()
        .map(|l| format!("  {l}").trim_end().to_string())
        .collect();
    assert_eq!(lines[at + 1..], shown, "{printed}");

    let (code, json) = host.run(&["verify", "0.1.0", "--json"]);
    assert_eq!(code, 0, "{json}");
    let json: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(json["value"]["notes"]["value"]["text"], text);
}

// Requirement: Status shows each version's notes from git

#[test]
fn status_prints_the_notes_beneath_the_versions_line_and_carries_them_in_its_json() {
    let host = Host::new();
    let (code, published) = host.run(&["publish", "HEAD", "--json"]);
    assert_eq!(code, 0, "{published}");
    let published: Value = serde_json::from_str(&published).unwrap();
    let carried = published["value"]["components"][0]["value"]["notes"]["text"]
        .as_str()
        .expect("the notes' text")
        .to_string();

    let (code, printed) = host.run(&["status"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.starts_with("  repository 0.1.0 (v0.1.0) from "))
        .expect(&printed);
    let shown: Vec<String> = carried
        .lines()
        .map(|l| format!("    {l}").trim_end().to_string())
        .collect();
    assert_eq!(lines[at + 1..], shown, "{printed}");

    let (code, json) = host.run(&["status", "--json"]);
    assert_eq!(code, 0, "{json}");
    let json: Value = serde_json::from_str(&json).unwrap();
    let version = &json["value"]["published"]["value"]["components"][0]["versions"][0];
    assert_eq!(version["version"], "0.1.0");
    assert_eq!(version["notes"]["answer"], "ok");
    assert_eq!(version["notes"]["value"], carried.as_str());
}

// Requirement: An unbound host answers absent
/// The moment now, in seconds since the Unix epoch.
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// `channel` declared over `fakes`' policy in the one component, bound
/// to nothing.
fn declared(fakes: &Fakes, channel: &str) {
    fakes.policy.require(
        Audience::Channel(ChannelId::new(None, ChannelName::new(channel))),
        Required::Channel {
            claims: vec![],
            bound_to: None,
            promoted_by: vec![],
            approvers: vec![],
        },
    );
}

/// `channel` promoted over `fakes`' keeper to each version in turn, each
/// from the one before, signed by the host as the fake signer signs,
/// each satisfied by its version's acceptance; every claim's digest.
fn chained(host: &Host, channel: &str, versions: &[&str]) -> Vec<Digest> {
    let channel = ChannelId::new(None, ChannelName::new(channel));
    let moved = |v: &str| tailrace_seams::Moved {
        version: Version::new(v),
        index: Digest::of(format!("{v}'s index").as_bytes()),
    };
    let mut predecessor: Option<Digest> = None;
    let mut claims = Vec::new();
    for (i, version) in versions.iter().enumerate() {
        let promotion = Promotion {
            channel: channel.name.clone(),
            component: ComponentName::new("repository"),
            to: moved(version),
            from: i.checked_sub(1).map(|j| moved(versions[j])),
            predecessor: predecessor.clone(),
            asked_by: SignerId::new("host"),
            policy: tailrace_seams::Satisfied {
                revision: host.head.clone(),
                claims: vec![acceptance(version)],
            },
            at: 1_793_000_000 + i as u64,
        };
        let envelope = host
            .fakes
            .signer
            .sign(&promotion.claim(host.head.clone()), &SignerId::new("host"))
            .unwrap();
        let digest = tailrace_seams::Keeper::append(
            &host.fakes.keeper,
            &channel,
            &envelope,
            predecessor.as_ref(),
        )
        .unwrap();
        claims.push(digest.clone());
        predecessor = Some(digest);
    }
    claims
}

/// The digest `chained` names as the claim that satisfied the policy for
/// `version`'s promotion.
fn acceptance(version: &str) -> Digest {
    Digest::of(format!("{version}'s acceptance").as_bytes())
}

#[test]
fn status_through_the_command_line_prints_the_cores_overview_value_for_value() {
    let host = Host::new();
    assert_eq!(host.run(&["publish", "HEAD"]).0, 0);
    declared(&host.fakes, "beta");
    declared(&host.fakes, "stable");
    chained(&host, "stable", &["0.1.0", "0.2.0"]);
    let before = now();
    let (code, printed) = host.run(&["status", "--json"]);
    let after = now();
    assert_eq!(code, 0, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    let asked_at = printed["value"]["asked_at"].as_u64().expect("asked_at");
    assert!(
        (before..=after).contains(&asked_at),
        "asked at {asked_at}, not between {before} and {after}"
    );

    let history = History::open(&host.repo).unwrap();
    let core = tailrace_core::overview(
        &host.fakes.seams(),
        &history,
        &Kept::at(&history),
        &tailrace_core::OverviewAsk {
            branch: Branch::new("main"),
            component: None,
            revision: None,
            at: asked_at,
        },
    );
    assert_eq!(printed, answer_json(&Ok::<_, tailrace_seams::Not>(core)));
    let channels = &printed["value"]["channels"]["value"]["channels"];
    assert_eq!(channels[1]["at"]["value"]["head"]["to"]["version"], "0.2.0");
    assert_eq!(printed["value"]["published"]["answer"], "ok");
}

#[test]
fn status_as_text_names_each_channel_its_chain_newest_first_and_each_answer_apart() {
    let host = Host::new();
    for channel in ["edge", "preview", "stable"] {
        declared(&host.fakes, channel);
    }
    let claims = chained(&host, "stable", &["0.1.0", "0.2.0"]);
    // The first chain asked, edge's, cannot be read.
    host.fakes.keeper.fault(
        tailrace_fakes::KeeperOp::Chain,
        Fault::Unreadable("fatal: unable to access".into()),
        Times::Once,
    );
    let (code, printed) = host.run(&["status"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    assert!(
        lines[0].starts_with(&format!("main at {}, as of 20", host.head)),
        "{printed}"
    );
    let at = |starts: &str| {
        lines
            .iter()
            .position(|l| l.starts_with(starts))
            .unwrap_or_else(|| panic!("no line starts {starts:?}: {printed}"))
    };
    assert_eq!(lines[at("repository")], "repository");
    let edge = at("  edge ");
    assert!(lines[edge].contains("could not read"), "{printed}");
    assert_eq!(lines[edge + 1], "fatal: unable to access", "{printed}");
    assert!(lines[at("  preview ")].ends_with("empty"), "{printed}");
    let stable = at("  stable ");
    assert!(
        lines[stable].contains(&format!(
            "0.2.0 ({})",
            Digest::of("0.2.0's index".as_bytes())
        )),
        "{printed}"
    );
    assert_eq!(
        lines[stable + 1].trim_start(),
        format!(
            "0.2.0 from 0.1.0, asked by host, under the policy at {} satisfied by {}, \
             2026-10-26T07:33:21Z, claim {} after {}",
            host.head,
            acceptance("0.2.0"),
            claims[1],
            claims[0]
        ),
        "{printed}"
    );
    assert_eq!(
        lines[stable + 2].trim_start(),
        format!(
            "0.1.0, the first, asked by host, under the policy at {} satisfied by {}, \
             2026-10-26T07:33:20Z, claim {}",
            host.head,
            acceptance("0.1.0"),
            claims[0]
        ),
        "{printed}"
    );
    let environments = at("environments");
    assert!(
        lines[environments + 1].contains("no target is read"),
        "{printed}"
    );
    let published = at("published from this host");
    assert!(lines[published + 1].contains("holds no"), "{printed}");
}

// Requirement: Status says when the landing was fetched

/// What the fake landing's remote says once it is cut off: git's words
/// over two lines, as a remote that cannot be reached answers.
const UNREACHABLE: &str = "fatal: unable to access 'https://forge.example/o/r.git/': \
                           Could not resolve host: forge.example\n\
                           fatal: Could not read from remote repository.\n";

#[test]
fn status_says_the_landing_was_read_as_last_fetched_and_shows_every_channel() {
    let host = Host::new();
    declared(&host.fakes, "beta");
    declared(&host.fakes, "stable");
    chained(&host, "stable", &["0.1.0", "0.2.0"]);
    let before = now();
    let (code, first) = host.run(&["status"]);
    let after = now();
    assert_eq!(code, 0, "{first}");
    assert!(
        !first.contains("landing:"),
        "a landing read from itself was named: {first}"
    );
    host.fakes.landing.unreachable(UNREACHABLE);

    let (code, printed) = host.run(&["status"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    assert!(
        lines[0].starts_with(&format!("main at {}, as of 20", host.head)),
        "{printed}"
    );
    let at = lines[1]
        .strip_prefix("  landing: as last fetched ")
        .unwrap_or_else(|| panic!("the landing's read is not under the branch: {printed}"));
    let fetched = (before..=after)
        .map(tailrace::render::utc)
        .find(|t| t == at);
    assert!(fetched.is_some(), "fetched at {at}, not when first read");
    assert!(
        printed.contains(&format!("{at}\n{UNREACHABLE}")),
        "the remote's words are not beneath, unchanged: {printed}"
    );
    let shown: Vec<&str> = first.lines().filter(|l| l.starts_with("  ")).collect();
    for line in shown {
        assert!(printed.contains(line), "{line:?} is not shown: {printed}");
    }
    assert!(!printed.contains("refused"), "{printed}");

    let (code, json) = host.run(&["status", "--json"]);
    assert_eq!(code, 0, "{json}");
    let json: Value = serde_json::from_str(&json).unwrap();
    let channels = &json["value"]["channels"];
    assert_eq!(channels["answer"], "ok", "{json}");
    let kept = &channels["value"]["landing"]["kept"];
    assert_eq!(kept["said"], UNREACHABLE, "{json}");
    assert_eq!(
        kept["at"].as_u64().map(tailrace::render::utc).as_deref(),
        Some(at),
        "{json}"
    );
    assert_eq!(
        channels["value"]["channels"][1]["at"]["value"]["head"]["to"]["version"],
        "0.2.0"
    );
}

#[test]
fn status_prints_an_empty_channel_read_from_the_copy_as_last_read_with_its_words() {
    let host = Host::new();
    declared(&host.fakes, "preview");
    let before = now();
    assert_eq!(host.run(&["status"]).0, 0);
    let after = now();
    host.fakes.keeper.unreachable("fatal: unable to access");

    let (code, printed) = host.run(&["status"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    let preview = lines
        .iter()
        .position(|l| l.starts_with("  preview "))
        .unwrap_or_else(|| panic!("no preview line: {printed}"));
    let (said, when) = lines[preview]
        .split_once("empty, as last read ")
        .unwrap_or_else(|| panic!("preview is not empty as last read: {printed}"));
    assert!(said.trim() == "preview", "{printed}");
    assert!(when.starts_with("20") && when.ends_with('Z'), "{printed}");
    assert_eq!(
        lines[preview + 1].trim(),
        "fatal: unable to access",
        "{printed}"
    );

    let (code, printed) = host.run(&["status", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    let at = &printed["value"]["channels"]["value"]["channels"][0]["at"]["value"];
    assert_eq!(at["state"], "empty", "{printed}");
    let kept = &at["read"]["kept"];
    assert_eq!(kept["said"], "fatal: unable to access", "{printed}");
    let read_at = kept["at"].as_u64().expect("the copy's time");
    assert!((before..=after).contains(&read_at), "{printed}");
}

#[test]
fn a_broken_bindings_file_is_the_channels_answer_and_the_record_still_answers() {
    let host = Host::new();
    let history = History::open(&host.repo).unwrap();
    Kept::at(&history)
        .write(&Entry {
            component: ComponentName::new("repository"),
            version: Version::new("0.1.0"),
            tag: "v0.1.0".into(),
            revision: host.head.clone(),
            branch: Branch::new("main"),
            index: Digest::of(b"the index"),
            store: store(),
            notes: None,
        })
        .unwrap();
    host.bind("[landing\n");
    let out = host.binary(&["status", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    let channels = &json["value"]["channels"];
    assert_eq!(channels["answer"], "unreadable", "{json}");
    let file = host.bindings_file().display().to_string();
    assert!(
        channels["unreadable"]["what"]
            .as_str()
            .is_some_and(|w| w.starts_with(&file)),
        "{json}"
    );
    assert!(
        channels["unreadable"]["said"]
            .as_str()
            .is_some_and(|s| !s.is_empty()),
        "{json}"
    );
    assert_eq!(json["value"]["published"]["answer"], "ok", "{json}");
}

#[test]
fn status_in_a_directory_that_is_no_repository_exits_3_naming_it() {
    let host = Host::new();
    let elsewhere = host.root().join("elsewhere");
    fs::create_dir_all(&elsewhere).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_tailrace"))
        .args(["status"])
        .current_dir(&elsewhere)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CEILING_DIRECTORIES", host.root())
        .env("HOME", host.root().join("home"))
        .env("XDG_CONFIG_HOME", host.root().join("config"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let printed = String::from_utf8_lossy(&out.stdout);
    assert!(printed.contains("elsewhere"), "{printed}");
}

// Requirement: Status names the page

#[test]
fn status_page_where_the_remote_names_no_owner_and_name_is_unreadable_and_names_nothing() {
    let host = Host::new();
    let out = host.binary(&["status", "--page", "--json"]);
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["answer"], "unreadable", "{json}");
    let what = json["unreadable"]["what"].as_str().unwrap();
    assert!(
        what.contains("remote origin") && what.contains("shared.git"),
        "{what}"
    );
    assert!(!host.root().join("state").exists(), "an entry was written");
}

// Requirement: Serve is a command

#[test]
fn serve_takes_no_argument_that_would_move_it() {
    let host = Host::new();
    let out = host.binary(&["serve", "--port", "8080"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--port"), "{stderr}");
    assert!(!host.root().join("state").exists(), "serve took the lock");
}

#[test]
fn a_moment_is_rendered_in_utc() {
    assert_eq!(tailrace::render::utc(0), "1970-01-01T00:00:00Z");
    assert_eq!(tailrace::render::utc(1_793_000_000), "2026-10-26T07:33:20Z");
}

// Requirement: A host with no bindings file is bound to the defaults

/// The store's unreadable answer in `out`, once it names `layout`.
fn unreadable_layout(out: &std::process::Output, layout: &Path) {
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["answer"], "unreadable", "{json}");
    assert_eq!(json["unreadable"]["source"]["seam"], "store", "{json}");
    assert!(
        json["unreadable"]["what"]
            .as_str()
            .is_some_and(|what| what.starts_with(&layout.display().to_string())),
        "{json}"
    );
}

#[test]
fn the_built_binary_on_a_host_with_no_bindings_file_is_bound_to_the_given_profiles() {
    let host = Host::new();
    let file = host.bindings_file();
    let layout = host.given_layout();
    let data = host.root().join("data");
    unreadable_layout(&host.binary(&["verify", "0.1.0", "--json"]), &layout);
    assert!(!data.exists(), "verify made {}", data.display());

    let out = host.binary(&["publish", "HEAD"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(&format!("main has not landed {}", host.head)),
        "{stdout}"
    );
    assert_eq!(git(&host.repo, &["tag", "-l"]), "");
    assert_eq!(git(&host.repo, &["status", "--porcelain"]), "");
    let history = History::open(&host.repo).unwrap();
    assert!(!Kept::at(&history).dir().exists());
    assert!(
        !layout.exists(),
        "the refused publish made the given layout"
    );
    assert!(!file.exists(), "the publish wrote a bindings file");
    assert!(
        !host.given_key().exists(),
        "a publish that signed nothing made a key"
    );
    assert!(
        !data.exists(),
        "the refused publish made {}",
        data.display()
    );
}

#[test]
fn the_built_binary_in_a_checkout_with_several_remotes_and_no_origin_names_them_and_writes_nothing()
{
    let host = Host::new();
    git(&host.repo, &["remote", "rename", "origin", "github"]);
    git(
        &host.repo,
        &["remote", "add", "gitlab", "/nowhere/gitlab.git"],
    );
    let out = host.binary(&["publish", "HEAD"]);
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("github, gitlab"), "{stdout}");
    assert!(
        stdout.contains(&format!(
            "[landing] remote of {}",
            host.bindings_file().display()
        )),
        "{stdout}"
    );
    assert_eq!(git(&host.repo, &["tag", "-l"]), "");
    assert_eq!(git(&host.repo, &["status", "--porcelain"]), "");
    let history = History::open(&host.repo).unwrap();
    assert!(!Kept::at(&history).dir().exists());
    assert!(
        !host.root().join("data").exists(),
        "a layout or a key was made"
    );
    assert!(!host.bindings_file().exists());
}

#[test]
fn the_built_binary_with_no_home_and_no_configuration_directory_answers_absent_naming_both() {
    let host = Host::new();
    let data = host.root().join("data");
    let state = host.root().join("state");
    let out = Command::new(env!("CARGO_BIN_EXE_tailrace"))
        .args(["publish", "HEAD"])
        .current_dir(&host.repo)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env_remove("HOME")
        .env_remove("XDG_CONFIG_HOME")
        .env("XDG_DATA_HOME", &data)
        .env("XDG_STATE_HOME", &state)
        .output()
        .expect("the binary runs");
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("neither XDG_CONFIG_HOME nor HOME is set"),
        "{stdout}"
    );
    assert_eq!(git(&host.repo, &["tag", "-l"]), "");
    assert_eq!(git(&host.repo, &["status", "--porcelain"]), "");
    let history = History::open(&host.repo).unwrap();
    assert!(!Kept::at(&history).dir().exists());
    assert!(!data.exists(), "a layout or a key was made");
    assert!(!state.exists(), "a record or a lock was made");
    assert!(!host.bindings_file().exists());
}

#[test]
fn the_built_binary_on_a_host_with_no_bindings_file_shows_status_from_the_record_and_the_ledger() {
    let host = Host::new();
    git(&host.repo, &["push", "-q", "origin", "main"]);
    let index = Digest::of(b"the index");
    let history = History::open(&host.repo).unwrap();
    Kept::at(&history)
        .write(&Entry {
            component: ComponentName::new("repository"),
            version: Version::new("0.1.0"),
            tag: "v0.1.0".into(),
            revision: host.head.clone(),
            branch: Branch::new("main"),
            index: index.clone(),
            store: store(),
            notes: None,
        })
        .unwrap();
    // One promotion of stable on origin's ledger refs, where the given
    // ledger is kept, and none in the checkout's own.
    let promotion = Promotion {
        channel: ChannelName::new("stable"),
        component: ComponentName::new("repository"),
        to: tailrace_seams::Moved {
            version: Version::new("0.1.0"),
            index: index.clone(),
        },
        from: None,
        predecessor: None,
        asked_by: SignerId::new("host"),
        policy: tailrace_seams::Satisfied {
            revision: host.head.clone(),
            claims: vec![acceptance("0.1.0")],
        },
        at: 1_793_000_000,
    };
    let envelope = host
        .fakes
        .signer
        .sign(&promotion.claim(host.head.clone()), &SignerId::new("host"))
        .unwrap();
    let origin =
        tailrace::profiles::ledger::git_ref::GitRef::new(&host.repo, Some("origin".into()));
    tailrace_seams::Keeper::append(&origin, &stable(), &envelope, None).unwrap();
    git(
        &host.repo,
        &["update-ref", "-d", "refs/tailrace/ledger/stable"],
    );

    let out = host.binary(&["status", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    let entry = &json["value"]["published"]["value"]["components"][0]["versions"][0];
    assert_eq!(entry["version"], "0.1.0", "{json}");
    assert_eq!(entry["index"], index.as_str());
    let channels = &json["value"]["channels"];
    assert_eq!(channels["answer"], "ok", "{json}");
    let stable = channels["value"]["channels"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["channel"]["name"] == "stable")
        .unwrap_or_else(|| panic!("no stable: {json}"));
    assert_eq!(
        stable["at"]["value"]["head"]["to"]["version"], "0.1.0",
        "{json}"
    );
    assert!(!host.bindings_file().exists());
    assert!(
        !host.root().join("data").exists(),
        "status made a layout or a key"
    );
}

#[test]
fn the_built_binary_on_a_host_with_no_bindings_file_syncs_from_the_given_layout_and_makes_none() {
    let host = Host::new();
    let file = host.bindings_file();
    let layout = host.given_layout();
    unreadable_layout(&host.binary(&["sync", "--json"]), &layout);
    assert_eq!(git(&host.repo, &["status", "--porcelain"]), "");
    assert!(!host.repo.join(".moon").exists(), "the sync wrote a cache");
    assert!(!layout.exists(), "the sync made the given layout");
    assert!(!file.exists(), "the sync wrote a bindings file");
}

// Requirement: A host with no bindings file is bound to the defaults, a
// broken file

#[test]
fn a_bindings_file_that_is_not_toml_is_one_json_document_in_the_parsers_words() {
    let host = Host::new();
    let text = "[store\nlayout = \"/layout\"\n";
    host.bind(text);
    let words = toml::from_str::<toml::Table>(text).unwrap_err().to_string();
    let out = host.binary(&["verify", "0.1.0", "--json"]);
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
    assert_eq!(json["answer"], "unreadable");
    let unreadable = &json["unreadable"];
    assert_eq!(unreadable["source"]["seam"], "host");
    assert_eq!(
        unreadable["what"],
        host.bindings_file().display().to_string()
    );
    assert_eq!(unreadable["said"], words);
}

#[test]
fn a_bindings_file_naming_two_landings_is_unreadable_naming_the_table() {
    let host = Host::new();
    host.bind(&format!(
        "[store]\nlayout = \"{}\"\n\n[landing]\nsluice = {{}}\nbranch = {{}}\n\n\
         [signing]\nsigner = \"host\"\nkey = \"~/.config/tailrace/signing\"\n",
        host.root().join("layout").display()
    ));
    let out = host.binary(&["publish", "HEAD"]);
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(&format!("{}, [landing]", host.bindings_file().display())),
        "{stdout}"
    );
    assert!(
        stdout.contains("[landing] names branch and sluice"),
        "{stdout}"
    );
    assert!(!host.root().join("layout").exists(), "the layout was made");
    assert_eq!(git(&host.repo, &["tag", "-l"]), "");
}

// Requirement: The host's bindings are one file beside the landing's own

#[test]
fn with_no_configuration_directory_set_the_file_under_the_home_binds_verify() {
    let host = Host::new();
    let home = host.root().join("home");
    let layout = host.root().join("no-layout");
    let file = home.join(".config/tailrace/config.toml");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(
        &file,
        format!(
            "[store]\nlayout = \"{}\"\n\n[landing]\nbranch = {{}}\n",
            layout.display()
        ),
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_tailrace"))
        .args(["verify", "0.1.0", "--json"])
        .current_dir(&host.repo)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("HOME", &home)
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("the binary runs");
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["answer"], "unreadable", "{json}");
    assert_eq!(json["unreadable"]["source"]["seam"], "store", "{json}");
    let what = json["unreadable"]["what"].as_str().unwrap();
    assert!(what.starts_with(&layout.display().to_string()), "{json}");
    assert!(!layout.exists(), "verify made the layout");
}

// Requirement: Promote takes a version and a channel

fn stable() -> ChannelId {
    ChannelId::new(None, ChannelName::new("stable"))
}

/// A second feature committed on `main` after the host's head, and its
/// revision.
fn grown(host: &Host) -> Revision {
    fs::write(host.repo.join("site/index.html"), "<p>hello again</p>").unwrap();
    git(&host.repo, &["commit", "-q", "-am", "feat: the site grows"]);
    Revision::new(git(&host.repo, &["rev-parse", "HEAD"]))
}

/// Over `fakes`, the host's head published as 0.1.0 and `second`, landed
/// and built, as 0.2.0, each index carrying the host's acceptance, and
/// the policy for `stable` requiring it of the host, which it names for
/// acceptance and promotion (108).
fn promotable(host: &Host, fakes: &Fakes, second: &Revision) {
    fakes.landing.land(&Branch::new("main"), second);
    fakes.build.made(
        second,
        &OutputName::new("site"),
        Report {
            build: BuildId::new("site-build-2"),
            outcome: Outcome::Pass,
            files: vec!["dist/site.tar".into()],
        },
    );
    let history = History::open(&host.repo).unwrap();
    for revision in [&host.head, second] {
        let published = publish(
            &fakes.seams(),
            &history,
            &Kept::at(&history),
            &Ask {
                branch: Branch::new("main"),
                revision: revision.clone(),
                component: None,
                signer: SignerId::new("host"),
                store: store(),
            },
        )
        .unwrap();
        let Ok(tailrace_core::Published::Version { index, .. }) = &published.components[0].1 else {
            panic!("nothing was published: {published:?}")
        };
        let acceptance = Claim {
            subject: index.clone(),
            kind: ClaimKind::Acceptance,
            revision: revision.clone(),
            build: None,
            body: serde_json::json!({}),
        };
        let envelope = fakes
            .signer
            .sign(&acceptance, &SignerId::new("host"))
            .unwrap();
        tailrace_seams::Store::attach(&fakes.store, index, &envelope).unwrap();
    }
    let host_signer = NamedSigner {
        id: SignerId::new("host"),
        verified_by: "the host's key".into(),
        kinds: vec![ClaimKind::Acceptance, ClaimKind::Promotion],
        checks: vec![],
    };
    fakes.policy.require(
        Audience::Channel(stable()),
        Required::Channel {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Acceptance,
                check: None,
                signers: vec![host_signer.clone()],
                asked_of: AskedOf::Every,
            }],
            bound_to: None,
            promoted_by: vec![host_signer],
            approvers: vec![],
        },
    );
}

#[test]
fn promote_through_the_command_line_prints_the_cores_answer_value_for_value() {
    let host = Host::new();
    let second = grown(&host);
    promotable(&host, &host.fakes, &second);
    let (code, printed) = host.run(&["promote", "0.2.0", "--to", "stable", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();

    let chain = tailrace_seams::Keeper::chain(&host.fakes.keeper, &stable()).unwrap();
    let head = chain.claims.last().unwrap();
    let asked = Promotion::of(&Claim::from_envelope(head).unwrap()).unwrap();
    assert_eq!(printed["value"]["claim"], head.digest().as_str());

    // The same question over an identical binding, at the moment the
    // command asked it.
    let other = fakes_at(&host.head);
    promotable(&host, &other, &second);
    let core = promote(
        &other.seams(),
        &History::open(&host.repo).unwrap(),
        &PromoteAsk {
            branch: Branch::new("main"),
            component: None,
            channel: stable().name,
            to: To::Version(Version::new("0.2.0")),
            from: None,
            signer: SignerId::new("host"),
            at: asked.at,
        },
    );
    assert_eq!(printed, answer_json(&core));
    assert_eq!(printed["value"]["to"]["version"], "0.2.0");
}

#[test]
fn promote_from_a_version_the_channel_has_left_exits_1_naming_where_it_is() {
    let host = Host::new();
    let second = grown(&host);
    promotable(&host, &host.fakes, &second);
    let (code, printed) = host.run(&["promote", "0.2.0", "--to", "stable"]);
    assert_eq!(code, 0, "{printed}");
    assert!(
        printed.starts_with("stable moved to 0.2.0 (sha256:")
            && printed.contains("its first promotion"),
        "{printed}"
    );
    let (code, printed) = host.run(&["promote", "0.1.0", "--to", "stable", "--from", "0.1.0"]);
    assert_eq!(code, 1, "{printed}");
    assert!(printed.contains("stable is at 0.2.0"), "{printed}");
    let (code, printed) = host.run(&["promote", "--back", "--to", "stable"]);
    assert_eq!(code, 4, "{printed}");
    assert!(printed.contains("a version stable moved from"), "{printed}");
}

#[test]
fn a_version_and_back_together_are_arguments_promote_cannot_take() {
    let host = Host::new();
    let err = Cli::try_parse_from(["tailrace", "promote", "0.2.0", "--back", "--to", "stable"])
        .expect_err("promote took a version and --back");
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("--back"), "{err}");
    let err = Cli::try_parse_from(["tailrace", "promote", "--to", "stable"])
        .expect_err("promote took neither a version nor --back");
    assert_eq!(err.exit_code(), 2);

    let out = host.binary(&["promote", "0.2.0", "--back", "--to", "stable"]);
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--back"), "{stderr}");
    assert_eq!(host.calls(), 0, "the core was asked");
}

// Requirement: A prerelease moves only the channels that take prereleases

/// The version published over the fakes at a commit with `subject`,
/// tagged after `v0.1.0`, whose policy file switches `line` on, and what
/// the gate accepting it over that file moved, each channel having moved
/// to its index, and the fakes.
fn accepted_with(line: &str, subject: &str) -> (Version, Vec<(ChannelId, Answer<Moved>)>, Fakes) {
    let host = Host::new();
    git(&host.repo, &["tag", "v0.1.0", host.head.as_str()]);
    fs::write(
        host.repo.join(".config/tailrace.toml"),
        format!(
            "{POLICY}\n[signers.host]\nssh = \"ssh-ed25519 HHHH\"\nkinds = [\"acceptance\"]\n\n\
             [acceptance]\nrequire = []\n\n{line}\n"
        ),
    )
    .unwrap();
    git(&host.repo, &["commit", "-q", "-am", subject]);
    let head = Revision::new(git(&host.repo, &["rev-parse", "HEAD"]));
    let fakes = fakes_at(&head);
    let history = History::open(&host.repo).unwrap();
    let published = publish(
        &fakes.seams(),
        &history,
        &Kept::at(&history),
        &Ask {
            branch: Branch::new("main"),
            revision: head.clone(),
            component: None,
            signer: SignerId::new("host"),
            store: store(),
        },
    )
    .unwrap();
    let Ok(tailrace_core::Published::Version { version, index, .. }) = &published.components[0].1
    else {
        panic!("nothing was published: {published:?}")
    };

    let mut seams = fakes.seams();
    seams.policy = Box::new(PolicyFile::new(&host.repo));
    let judged = accept(
        &seams,
        &history,
        &AcceptAsk {
            branch: Branch::new("main"),
            component: None,
            version: version.clone(),
            told: ToldBy::Publish {
                index: index.clone(),
            },
            gate: SignerId::new("host"),
            when: 1_793_000_000,
        },
    );
    let Ok(Judged::Accepted { channels, .. }) = judged else {
        panic!("{version} was not accepted: {judged:?}")
    };
    for (channel, moved) in &channels {
        assert!(
            matches!(moved, Ok(Moved::Promoted { to, .. }) if to == index),
            "{channel} did not move to {version}: {channels:?}"
        );
    }
    (version.clone(), channels, fakes)
}

/// Whether the fake keeper holds a chain of `name`.
fn holds_chain(fakes: &Fakes, name: &str) -> bool {
    !matches!(
        tailrace_seams::Keeper::chain(&fakes.keeper, &ChannelId::new(None, ChannelName::new(name))),
        Err(tailrace_seams::Not::Absent(_))
    )
}

#[test]
fn a_prerelease_accepted_with_alpha_written_moves_alpha_and_not_beta() {
    let (version, channels, fakes) = accepted_with("[channels.alpha]", "chore: alpha switched on");
    assert!(
        version.as_str().contains("-pre."),
        "{version} is no prerelease"
    );
    let alpha = ChannelId::new(None, ChannelName::new("alpha"));
    assert_eq!(
        channels.iter().map(|(c, _)| c).collect::<Vec<_>>(),
        [&alpha],
        "{channels:?}"
    );
    assert!(!holds_chain(&fakes, "beta"));
}

// Requirement: Who may ask for a channel

#[test]
fn lts_written_is_moved_by_no_publish_and_beta_is() {
    let (version, channels, fakes) = accepted_with("[channels.lts]", "feat: lts switched on");
    assert!(!version.as_str().contains('-'), "{version} is no release");
    let beta = ChannelId::new(None, ChannelName::new("beta"));
    assert_eq!(
        channels.iter().map(|(c, _)| c).collect::<Vec<_>>(),
        [&beta],
        "{channels:?}"
    );
    assert!(holds_chain(&fakes, "beta"));
    assert!(!holds_chain(&fakes, "lts"), "a publish moved lts");
}

// Requirement: Policy shows the policy in force

/// `args` asked through `run` over `binding`: the exit code and what it
/// printed.
fn run_over(host: &Host, binding: Binding, args: &[&str]) -> (i32, String) {
    let cli = Cli::try_parse_from(std::iter::once("tailrace").chain(args.iter().copied()))
        .expect("the arguments parse");
    let mut out = Vec::new();
    let code = run(&cli, &Ok(binding), &host.repo, &mut out);
    (code, String::from_utf8(out).unwrap())
}

/// The host's fakes with the policy file of its repository in place of
/// the fake policy.
fn over_the_file(host: &Host) -> Binding {
    let mut seams = host.fakes.seams();
    seams.policy = Box::new(PolicyFile::new(&host.repo));
    Binding {
        seams,
        store: store(),
        signer: None,
        gate: Teller::Job(JOB.into()),
        trust: Vec::new(),
    }
}

#[test]
fn policy_through_the_command_line_prints_the_givens_beside_a_line_value_for_value() {
    let host = Host::new();
    fs::write(
        host.repo.join(".config/tailrace.toml"),
        format!(
            "{POLICY}\n[signers.gate]\nssh = \"ssh-ed25519 GGGG\"\nkinds = [\"acceptance\"]\n\n\
             [signers.verifier]\nssh = \"ssh-ed25519 VVVV\"\nkinds = [\"rebuild\"]\n\n\
             [acceptance]\nrequire = []\n\n\
             [channels.stable]\nrequire = [{{ kind = \"rebuild\", signers = [\"verifier\"] }}]\n"
        ),
    )
    .unwrap();
    git(&host.repo, &["commit", "-q", "-am", "chore: the policy"]);
    let head = Revision::new(git(&host.repo, &["rev-parse", "HEAD"]));
    host.fakes.landing.land(&Branch::new("main"), &head);

    let (code, printed) = run_over(&host, over_the_file(&host), &["policy", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    let history = History::open(&host.repo).unwrap();
    let core = in_force(
        &over_the_file(&host).seams,
        &history,
        &InForceAsk {
            branch: Branch::new("main"),
        },
    );
    assert_eq!(printed, answer_json(&core));

    let value = &printed["value"];
    assert_eq!(value["revision"], head.as_str());
    let channels = value["channels"]["value"]["in_force"]
        .as_array()
        .expect("the channels");
    let beta = &channels[0];
    assert_eq!(
        (&beta["channel"]["name"], &beta["moves"]["by"]),
        (&Value::from("beta"), &Value::from("each_publish"))
    );
    let stable = &channels[1];
    assert_eq!(stable["channel"]["name"], "stable");
    assert_eq!(
        stable["moves"],
        serde_json::json!({ "by": "bound", "to": "beta" })
    );
    let policy = &stable["policy"]["value"]["channel"];
    assert_eq!(policy["bound_to"], "beta");
    assert_eq!(policy["promoted_by"][0]["id"], "gate");
    let claims = &policy["claims"];
    let kinds: Vec<&str> = claims
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, ["acceptance", "rebuild"]);
    assert_eq!(claims[0]["signers"][0]["id"], "gate");
    assert!(claims[0].get("asked_of").is_none(), "{claims}");
    assert_eq!(claims[1]["asked_of"], "built", "{claims}");
    assert_eq!(channels.len(), 2, "{printed}");
}

#[test]
fn policy_shows_a_verdict_asked_of_built_artifacts_alone() {
    let host = Host::new();
    fs::write(
        host.repo.join(".config/tailrace.toml"),
        format!(
            "{POLICY}\n[signers.laptop]\nssh = \"ssh-ed25519 AAAA\"\nkinds = [\"provenance\"]\n\n\
             [signers.box]\nssh = \"ssh-ed25519 BBBB\"\nkinds = [\"verdict\"]\nchecks = [\"cli:test\"]\n\n\
             [acceptance]\nrequire = [\n  {{ kind = \"provenance\", signers = [\"laptop\"] }},\n  \
             {{ kind = \"verdict\", check = \"cli:test\", signers = [\"box\"] }},\n]\n"
        ),
    )
    .unwrap();
    git(&host.repo, &["commit", "-q", "-am", "chore: the policy"]);
    let head = Revision::new(git(&host.repo, &["rev-parse", "HEAD"]));
    host.fakes.landing.land(&Branch::new("main"), &head);

    let (lines, json) = policy_lines(&host);
    let printed = lines.join("\n");
    let acceptance = lines
        .iter()
        .position(|l| l == "acceptance")
        .unwrap_or_else(|| panic!("no acceptance: {printed}"));
    assert_eq!(
        lines[acceptance + 1..acceptance + 3],
        [
            "  provenance by laptop",
            "  verdict of cli:test by box, of built artifacts alone",
        ],
        "{printed}"
    );
    let claims = &json["value"]["acceptance"]["value"]["acceptance"]["claims"];
    assert!(claims[0].get("asked_of").is_none(), "{claims}");
    assert_eq!(claims[1]["asked_of"], "built", "{claims}");
}

#[test]
fn policy_as_text_shows_the_layout_absent_beside_the_channels_and_exits_0() {
    let host = Host::new();
    declared(&host.fakes, "beta");
    host.fakes
        .policy
        .follows(&ChannelId::new(None, ChannelName::new("beta")), false);
    host.fakes.policy.require(
        Audience::Channel(stable()),
        Required::Channel {
            claims: vec![],
            bound_to: Some(ChannelName::new("beta")),
            promoted_by: vec![],
            approvers: vec![],
        },
    );
    let (code, printed) = host.run(&["policy"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    assert_eq!(lines[0], format!("main at {}", host.head), "{printed}");
    let at = |line: &str| {
        lines
            .iter()
            .position(|l| *l == line)
            .unwrap_or_else(|| panic!("no line {line:?}: {printed}"))
    };
    assert_eq!(
        lines[at("acceptance") + 1],
        "  requires no claim",
        "{printed}"
    );
    let layout = lines[at("layout") + 1];
    assert!(
        layout.starts_with("  ") && layout.contains("holds no") && layout.contains("layout"),
        "{printed}"
    );
    let channels = at("channels");
    assert_eq!(lines[channels + 1], "  beta, each publish", "{printed}");
    assert_eq!(lines[channels + 5], "  stable, bound to beta", "{printed}");
    assert_eq!(
        lines[channels + 7],
        "    promoted by no signer",
        "{printed}"
    );
    assert_eq!(
        lines[channels + 8],
        "    may be asked for by any asker",
        "{printed}"
    );
    assert!(
        !lines.contains(&"held off"),
        "a policy holding nothing off showed a held part: {printed}"
    );
}

/// The host's repository with `channels` written beneath an acceptance
/// requiring nothing, committed and landed.
fn landed_with(host: &Host, channels: &str) {
    fs::write(
        host.repo.join(".config/tailrace.toml"),
        format!(
            "{POLICY}\n[signers.gate]\nssh = \"ssh-ed25519 GGGG\"\nkinds = [\"acceptance\"]\n\n\
             [acceptance]\nrequire = []\n\n{channels}"
        ),
    )
    .unwrap();
    git(&host.repo, &["commit", "-q", "-am", "chore: the policy"]);
    let head = Revision::new(git(&host.repo, &["rev-parse", "HEAD"]));
    host.fakes.landing.land(&Branch::new("main"), &head);
}

/// The lines `tailrace policy` prints over the host's policy file, and
/// its JSON compared value for value with the core's answer.
fn policy_lines(host: &Host) -> (Vec<String>, Value) {
    let (code, printed) = run_over(host, over_the_file(host), &["policy"]);
    assert_eq!(code, 0, "{printed}");
    let (code, json) = run_over(host, over_the_file(host), &["policy", "--json"]);
    assert_eq!(code, 0, "{json}");
    let json: Value = serde_json::from_str(&json).unwrap();
    let history = History::open(&host.repo).unwrap();
    let core = in_force(
        &over_the_file(host).seams,
        &history,
        &InForceAsk {
            branch: Branch::new("main"),
        },
    );
    assert_eq!(json, answer_json(&core));
    (printed.lines().map(str::to_string).collect(), json)
}

#[test]
fn policy_shows_the_givens_held_off_beneath_the_table_with_their_lines() {
    let host = Host::new();
    landed_with(&host, "");
    let (lines, json) = policy_lines(&host);
    let printed = lines.join("\n");
    let at = |line: &str| {
        lines
            .iter()
            .position(|l| l == line)
            .unwrap_or_else(|| panic!("no line {line:?}: {printed}"))
    };
    let held = at("held off");
    assert!(at("  beta, each publish") < held, "{printed}");
    assert!(at("  stable, bound to beta") < held, "{printed}");
    assert_eq!(
        lines[held + 1..],
        [
            "  alpha, each publish, prereleases too, switch on with [channels.alpha]",
            "  lts, bound to stable, switch on with [channels.lts]",
        ],
        "{printed}"
    );

    let channels = &json["value"]["channels"]["value"];
    assert_eq!(
        channels["held"],
        serde_json::json!([
            {
                "channel": { "component": null, "name": "alpha" },
                "moves": { "by": "each_publish", "prereleases": true },
                "switch": "[channels.alpha]",
            },
            {
                "channel": { "component": null, "name": "lts" },
                "moves": { "by": "bound", "to": "stable" },
                "switch": "[channels.lts]",
            },
        ])
    );
}

#[test]
fn an_unreadable_channels_answer_is_shown_once_in_place_of_the_channels_and_the_held_part() {
    let host = Host::new();
    host.fakes.policy.holds(
        &ChannelId::new(None, ChannelName::new("canary")),
        tailrace_seams::Moves::Promotion,
        "[channels.canary]",
    );
    host.fakes.policy.fault(
        tailrace_fakes::PolicyOp::Channels,
        Fault::Unreadable("the channels' table cannot be read".into()),
        Times::Once,
    );
    let (code, printed) = host.run(&["policy"]);
    assert_eq!(code, 0, "{printed}");
    assert_eq!(
        printed
            .matches("the channels' table cannot be read")
            .count(),
        1,
        "{printed}"
    );
    assert!(printed.contains("\nchannels\n"), "{printed}");
    assert!(!printed.contains("held off"), "{printed}");
    assert!(!printed.contains("canary"), "{printed}");
    assert!(
        printed.contains("acceptance\n  requires no claim"),
        "{printed}"
    );
}

#[test]
fn a_given_switched_on_leaves_the_held_part() {
    let host = Host::new();
    landed_with(&host, "[channels.alpha]\n");
    let (lines, json) = policy_lines(&host);
    let printed = lines.join("\n");
    let held = lines
        .iter()
        .position(|l| l == "held off")
        .unwrap_or_else(|| panic!("no held part: {printed}"));
    assert!(
        lines[..held]
            .iter()
            .any(|l| l == "  alpha, each publish, prereleases too"),
        "{printed}"
    );
    assert_eq!(
        lines[held + 1..],
        ["  lts, bound to stable, switch on with [channels.lts]"],
        "{printed}"
    );
    let held: Vec<&Value> = json["value"]["channels"]["value"]["held"]
        .as_array()
        .expect("the held givens")
        .iter()
        .map(|h| &h["channel"]["name"])
        .collect();
    assert_eq!(held, [&Value::from("lts")]);
}

#[test]
fn policy_shows_alpha_taking_prereleases_and_stables_approvers_value_for_value() {
    let host = Host::new();
    fs::write(
        host.repo.join(".config/tailrace.toml"),
        format!(
            "{POLICY}\n[signers.gate]\nssh = \"ssh-ed25519 GGGG\"\nkinds = [\"acceptance\"]\n\n\
             [signers.ci]\nssh = \"ssh-ed25519 CCCC\"\nkinds = [\"acceptance\"]\n\n\
             [channels.alpha]\n\n\
             [channels.stable]\napprovers = [\"ci\"]\n"
        ),
    )
    .unwrap();
    git(&host.repo, &["commit", "-q", "-am", "chore: the policy"]);
    let head = Revision::new(git(&host.repo, &["rev-parse", "HEAD"]));
    host.fakes.landing.land(&Branch::new("main"), &head);

    let (code, printed) = run_over(&host, over_the_file(&host), &["policy"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    let at = |line: &str| {
        lines
            .iter()
            .position(|l| *l == line)
            .unwrap_or_else(|| panic!("no line {line:?}: {printed}"))
    };
    let alpha = at("  alpha, each publish, prereleases too");
    assert_eq!(
        lines[alpha + 3],
        "    may be asked for by any asker",
        "{printed}"
    );
    let stable = at("  stable, bound to beta");
    assert_eq!(lines[stable + 2], "    promoted by ci, gate", "{printed}");
    assert_eq!(lines[stable + 3], "    may be asked for by ci", "{printed}");

    let (code, printed) = run_over(&host, over_the_file(&host), &["policy", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    let history = History::open(&host.repo).unwrap();
    let core = in_force(
        &over_the_file(&host).seams,
        &history,
        &InForceAsk {
            branch: Branch::new("main"),
        },
    );
    assert_eq!(printed, answer_json(&core));
    let channels = printed["value"]["channels"]["value"]["in_force"]
        .as_array()
        .expect("the channels");
    assert_eq!(
        channels[0]["moves"],
        serde_json::json!({ "by": "each_publish", "prereleases": true })
    );
    let stable = channels
        .iter()
        .find(|c| c["channel"]["name"] == "stable")
        .expect("stable");
    assert_eq!(
        stable["policy"]["value"]["channel"]["approvers"][0]["id"],
        "ci"
    );
}

#[test]
fn policy_at_a_head_not_landed_exits_1_naming_the_branch_and_the_revision() {
    let host = Host::new();
    let second = grown(&host);
    let (code, printed) = host.run(&["policy"]);
    assert_eq!(code, 1, "{printed}");
    assert!(
        printed.contains("main") && printed.contains(second.as_str()),
        "{printed}"
    );
    assert!(
        host.fakes.policy.calls().is_empty(),
        "the policy was asked of a head not landed"
    );
}
