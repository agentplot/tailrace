//! Spec command-line: each command calls the one core and renders its
//! answer, over the fakes and as the built binary (design D11, D12).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Parser;
use serde_json::Value;
use tailrace::bindings::Teller;
use tailrace::{run, Binding, Cli};
use tailrace_core::{
    answer_json, promote, publish, push, sync, write, Ask, Entry, History, Kept, PromoteAsk,
    PushAsk, SyncAsk, To,
};
use tailrace_fakes::{Fakes, Fault, StoreOp, Times};
use tailrace_seams::{
    Audience, Branch, BuildId, ChannelId, ChannelName, Check, Claim, ClaimKind, ComponentName,
    Digest, Ecosystem, NamedSigner, Outcome, Output, OutputName, Promotion, Report, Required,
    RequiredClaim, Revision, Seam, Signer, SignerId, Source, Verdict, Version,
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

    let at = History::open(&host.repo).unwrap().workdir().unwrap();
    let core = sync(
        &host.fakes.seams(),
        &SyncAsk {
            branch: Branch::new("main"),
            revision: host.head.clone(),
            at: at.display().to_string(),
        },
    );
    assert_eq!(printed, answer_json(&core));

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
    assert!(
        !file.with_file_name("signing").exists(),
        "the push made a key"
    );
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
        },
    );
}

/// `channel` promoted over `fakes`' keeper to each version in turn, each
/// from the one before, signed by the host as the fake signer signs.
fn chained(host: &Host, channel: &str, versions: &[&str]) {
    let channel = ChannelId::new(None, ChannelName::new(channel));
    let moved = |v: &str| tailrace_seams::Moved {
        version: Version::new(v),
        index: Digest::of(format!("{v}'s index").as_bytes()),
    };
    let mut predecessor: Option<Digest> = None;
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
                claims: vec![],
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
        predecessor = Some(digest);
    }
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
    chained(&host, "stable", &["0.1.0", "0.2.0"]);
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
    assert!(
        lines[stable + 1]
            .trim_start()
            .starts_with("0.2.0 from 0.1.0, asked by host, 2026-10-26T07:33:21Z, claim sha256:"),
        "{printed}"
    );
    assert!(
        lines[stable + 2]
            .trim_start()
            .starts_with("0.1.0, the first, asked by host, 2026-10-26T07:33:20Z, claim sha256:"),
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

// Requirement: A host with no bindings file is bound to the given
// profiles (spec host-bindings)

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
    unreadable_layout(&host.binary(&["verify", "0.1.0", "--json"]), &layout);
    assert!(!layout.exists(), "verify made the given layout");

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
    assert!(!file.exists(), "the publish wrote a bindings file");
    assert!(
        !file.with_file_name("signing").exists(),
        "a publish that signed nothing made a key"
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

// Requirement: An unbound host answers absent

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
            }],
            bound_to: None,
            promoted_by: vec![host_signer],
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
