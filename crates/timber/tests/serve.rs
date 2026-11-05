//! Spec serving-process: the host's one serving process over real
//! profiles, each test over a `Host` of temporary directories and
//! `serve_on` on a port the system picks, reading no environment
//! variable of its own and never binding the core's port. A scratch
//! remote and its clone, bound to a plain branch landing and the git ref
//! ledger on `origin`, with promotions appended to the ledger as the
//! fake signer seals them. Needs git alone, so it is in the fast layer.

mod scratch;

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use clap::Parser;
use scratch::{Repo, Scratch, MAIN, POLICY_FILE};
use serde_json::Value;
use timber::bindings::{For, Teller};
use timber::commands::serve::host::{checkout_of, find_or_start, name, Host, SERVING};
use timber::commands::serve::page::escape;
use timber::commands::serve::{respond_with, serve_on, serve_with, Binder, Response, Served};
use timber::commands::status::page;
use timber::{run, Binding, Cli, REVISION};
use timber_core::{
    answer_json, artifact_name, place, sync_checkout, CheckoutAsk, Entry, History, Kept, Layouts,
    Noted, OverviewAsk, RepositoryName,
};
use timber_fakes::{FakeProjection, FakeSigner, FakeStore, Fakes};
use timber_seams::{
    Answer, Artifact, ArtifactMeta, AskedOf, Audience, Branch, BuildId, ChannelId, ChannelName,
    Claim, ClaimKind, ComponentName, Contents, Digest, Ecosystem, EcosystemName, IndexMeta, Layout,
    Manifest, Miss, Moved, NamedSigner, Not, Output, PackAt, Packed, Pin, Pinned, Projection,
    Promotion, Range, Rendered, Request, Required, RequiredClaim, Requirement, Revision, Satisfied,
    Seam, Signer, SignerId, Source, Store, Version,
};

/// The host the tests serve: a state directory, a bindings file, a home
/// and a data directory, all in the scratch directory.
fn host_in(dir: &Path) -> Host {
    Host {
        state: dir.join("state/timber"),
        config: dir.join("config/timber/config.toml"),
        home: Some(dir.join("home")),
        data: Some(dir.join("data")),
    }
}

/// The bindings file of `host`: a plain branch landing and the git ref
/// ledger, each on `ledger_remote`'s remote.
fn bind(host: &Host, ledger_remote: &str) {
    fs::create_dir_all(host.config.parent().unwrap()).unwrap();
    fs::write(
        &host.config,
        format!(
            "[landing]\nbranch = {{ remote = \"origin\" }}\n\n[ledger]\ngit_ref = {{ remote = {ledger_remote:?} }}\n"
        ),
    )
    .unwrap();
}

/// A scratch remote and its clone, the policy declaring `beta` and
/// `stable`, bound to `beta`, landed on `main`; the host's record holding
/// one entry; and the host bound to the branch and the ledger on
/// `origin`.
struct World {
    scratch: Scratch,
    host: Host,
    head: Revision,
}

impl World {
    fn new() -> World {
        World::on(|dir| {
            let host = host_in(dir);
            bind(&host, "origin");
            host
        })
    }

    /// A second repository, of its own remote, on `host`.
    fn beside(host: &Host) -> World {
        World::on(|_| host.clone())
    }

    fn on(host: impl FnOnce(&Path) -> Host) -> World {
        let scratch = Scratch::new();
        let repo = &scratch.repo;
        repo.write(
            POLICY_FILE,
            "outputs = [\"site\"]\n\n[projections]\nsite = \"file\"\n\n\
             [channels.beta]\nrequire = []\n\n[channels.stable]\nbound_to = \"beta\"\nrequire = []\n",
        );
        let head = Revision::new(repo.commit("feat: the channels"));
        repo.push();
        let host = host(&scratch.path());
        let history = History::open(&repo.path).unwrap();
        Kept::at(&history)
            .write(&Entry {
                component: ComponentName::new("repository"),
                version: Version::new("0.1.0"),
                tag: "v0.1.0".into(),
                revision: head.clone(),
                branch: Branch::new(MAIN),
                index: Digest::of(b"0.1.0's index"),
                store: Source::new(Seam::Store, "layout /srv/timber/layout"),
                notes: None,
            })
            .unwrap();
        World {
            scratch,
            host,
            head,
        }
    }

    fn repo(&self) -> &Repo {
        &self.scratch.repo
    }

    /// The checkout named to the host as `o/r`.
    fn named(&self) -> RepositoryName {
        self.named_as("o", "r")
    }

    /// The checkout named to the host as `owner/name`.
    fn named_as(&self, owner: &str, name: &str) -> RepositoryName {
        let repository = RepositoryName {
            owner: owner.into(),
            name: name.into(),
        };
        self::name(&self.host, &repository, &self.repo().path).unwrap();
        repository
    }

    /// The core's row of `repository` from the status answer in the
    /// checkout, as a program reads it.
    fn row(&self, repository: &RepositoryName) -> Value {
        let at = &self.repo().path;
        let binding = self.host.bind(at).expect("the host binds");
        let history = History::open(at).unwrap();
        let overview = timber_core::overview(
            &binding.seams,
            &history,
            &Kept::at(&history),
            &OverviewAsk {
                branch: Branch::new(MAIN),
                component: None,
                revision: None,
                at: 0,
            },
        );
        answer_json(&Ok::<_, Not>(timber_core::row(repository, &overview)))
    }

    /// `channel` promoted in the ledger to each version in turn, from the
    /// head it is at, sealed as the fake signer seals; every claim's
    /// digest.
    fn promote(&self, channel: &str, versions: &[&str]) -> Vec<Digest> {
        let channel = ChannelId::new(None, ChannelName::new(channel));
        let binding = self.host.bind(&self.repo().path).expect("the host binds");
        let keeper = &binding.seams.keeper;
        let gate = FakeSigner::new();
        gate.hold(&SignerId::new("gate"));
        let mut head = timber_core::channel(keeper.as_ref(), &channel).unwrap();
        let mut claims = Vec::new();
        for version in versions {
            let (from, predecessor) = match &head {
                timber_core::Channel::At { head, chain, .. } => {
                    (Some(head.to.clone()), chain.last().map(|(d, _)| d.clone()))
                }
                timber_core::Channel::Empty { .. } => (None, None),
            };
            let promotion = Promotion {
                channel: channel.name.clone(),
                component: channel.component_name(),
                to: moved(version),
                from,
                predecessor: predecessor.clone(),
                asked_by: SignerId::new("gate"),
                policy: Satisfied {
                    revision: self.head.clone(),
                    claims: vec![acceptance(version)],
                },
                at: 1_793_000_000 + claims.len() as u64,
            };
            let envelope = gate
                .sign(&promotion.claim(self.head.clone()), &SignerId::new("gate"))
                .unwrap();
            let claim = keeper
                .append(&channel, &envelope, predecessor.as_ref())
                .unwrap();
            claims.push(claim);
            head = timber_core::channel(keeper.as_ref(), &channel).unwrap();
        }
        claims
    }

    /// `timber status --json` in the checkout, over the host's binding.
    fn status(&self) -> Value {
        let cli = Cli::try_parse_from(["timber", "status", "--json"]).unwrap();
        let mut out = Vec::new();
        let at = &self.repo().path;
        assert_eq!(run(&cli, &self.host.bind(at), at, &mut out), 0);
        serde_json::from_slice(&out).unwrap()
    }
}

fn moved(version: &str) -> Moved {
    Moved {
        version: Version::new(version),
        index: Digest::of(format!("{version}'s index").as_bytes()),
    }
}

/// The digest `promote` names as the claim that satisfied the policy for
/// `version`'s promotion.
fn acceptance(version: &str) -> Digest {
    Digest::of(format!("{version}'s acceptance").as_bytes())
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// A port for one test's process, never the core's: below the range the
/// system hands to connections, so no connection is given it while the
/// process starts, and one of its own per test. Nothing binds it to see
/// it free: a git a test runs at that moment could inherit the socket
/// and hold the port after it is let go.
fn free() -> SocketAddr {
    use std::sync::atomic::{AtomicU16, Ordering};
    static NEXT: AtomicU16 = AtomicU16::new(0);
    let base = 20_000 + (std::process::id() % 400) as u16 * 20;
    let port = base + NEXT.fetch_add(1, Ordering::SeqCst);
    SocketAddr::from(([127, 0, 0, 1], port))
}

/// `method` of `path` asked at `address`: the status and the body.
fn ask(address: SocketAddr, method: &str, path: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(address).unwrap();
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut answered = String::new();
    stream.read_to_string(&mut answered).unwrap();
    let (head, body) = answered.split_once("\r\n\r\n").expect("a head and a body");
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .expect("a status");
    (status, body.to_string())
}

fn get(address: SocketAddr, path: &str) -> (u16, String) {
    ask(address, "GET", path)
}

fn json(address: SocketAddr, path: &str) -> (u16, Value) {
    let (status, body) = get(address, path);
    (
        status,
        serde_json::from_str(&body).expect("a JSON document"),
    )
}

/// `serve_on` over `host` at a port the system picks, in a thread, once
/// it answers: its address and the thread.
fn serving(host: &Host, idle: Duration) -> (SocketAddr, JoinHandle<Answer<Served>>) {
    let address = free();
    let host = host.clone();
    let server = thread::spawn(move || serve_on(address, &host, idle));
    find_or_start(address, || Ok(())).expect("the process answers");
    (address, server)
}

/// `value` with each moment asked set to 0.
fn unasked(mut value: Value) -> Value {
    value["value"]["asked_at"] = Value::from(0);
    value
}

/// Every file under `dir` but git's own, with its contents.
fn files(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut all = Vec::new();
    if !dir.exists() {
        return all;
    }
    let mut dirs = vec![dir.to_path_buf()];
    while let Some(at) = dirs.pop() {
        for entry in fs::read_dir(&at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n == ".git") {
                    continue;
                }
                dirs.push(path);
            } else {
                all.push((path.clone(), fs::read(&path).unwrap()));
            }
        }
    }
    all.sort();
    all
}

// Requirement: One process answers a host at one fixed address

#[test]
fn the_process_names_itself_its_revision_and_its_process_id() {
    let world = World::new();
    let (address, _) = serving(&world.host, Duration::from_secs(60));
    let (status, named) = json(address, "/serving.json");
    assert_eq!(status, 200);
    assert_eq!(named["serving"], "timber");
    assert_eq!(named["revision"], REVISION);
    assert_eq!(named["pid"], std::process::id());
}

// Requirement: A repository's page and its JSON are its status answer

#[test]
fn a_named_repositorys_json_is_its_status_answer_in_its_checkout() {
    let world = World::new();
    let repository = world.named();
    world.promote("stable", &["0.1.0", "0.2.0"]);
    let (address, _) = serving(&world.host, Duration::from_secs(60));
    let before = now();
    let (status, served) = json(address, &format!("{}/status.json", repository.path()));
    let after = now();
    assert_eq!(status, 200, "{served}");
    let asked = served["value"]["asked_at"].as_u64().unwrap();
    assert!((before..=after).contains(&asked), "{asked}");
    let printed = world.status();
    assert_eq!(unasked(served.clone()), unasked(printed));
    let channels = &served["value"]["channels"]["value"]["channels"];
    assert_eq!(channels[1]["channel"]["name"], "stable", "{served}");
    assert_eq!(channels[1]["at"]["value"]["head"]["to"]["version"], "0.2.0");
    assert_eq!(served["value"]["published"]["answer"], "ok", "{served}");
}

#[test]
fn the_page_names_every_channel_version_and_promotion_and_escapes_the_words() {
    let world = World::new();
    let repository = world.named();
    let claims = world.promote("stable", &["0.1.0", "0.2.0"]);
    let (address, _) = serving(&world.host, Duration::from_secs(60));
    let (_, served) = json(address, "/o/r/status.json");
    let (status, page) = get(address, &format!("{}/status", repository.path()));
    assert_eq!(status, 200, "{page}");
    assert!(page.starts_with("<!doctype html>"), "{page}");
    assert!(
        page.contains("href=\"/\""),
        "the page links no root: {page}"
    );
    for channel in served["value"]["channels"]["value"]["channels"]
        .as_array()
        .unwrap()
    {
        let name = channel["channel"]["name"].as_str().unwrap();
        assert!(page.contains(&format!("<td>{name}</td>")), "{name}: {page}");
    }
    for version in ["0.1.0", "0.2.0"] {
        assert!(page.contains(version), "{version}: {page}");
    }
    for claim in &claims {
        assert!(page.contains(claim.as_str()), "{claim}: {page}");
    }
    assert!(
        page.contains(&format!(
            "<code>{}</code>, 2026-10-26T07:33:21Z; claim <code>{}</code> after <code>{}</code>",
            acceptance("0.2.0"),
            claims[1],
            claims[0]
        )),
        "each promotion names the claims that satisfied its policy and its predecessor: {page}"
    );
    assert!(
        page.contains(&format!(
            "under the policy at <code>{}</code> satisfied by <code>{}</code>",
            world.head,
            acceptance("0.1.0")
        )),
        "{page}"
    );
    assert!(page.contains("no target is read"), "{page}");
    assert!(page.contains("v0.1.0"), "the record's part: {page}");

    // A ledger whose remote's name holds markup: each channel's chain is
    // unreadable with git's words, which the page escapes.
    bind(&world.host, "<script>alert(1)</script>");
    let (status, page) = get(address, "/o/r/status");
    assert_eq!(status, 200, "{page}");
    assert!(
        page.contains("&lt;script&gt;alert(1)&lt;/script&gt;"),
        "{page}"
    );
    assert!(!page.contains("<script>"), "{page}");
}

#[test]
fn the_page_and_its_json_say_the_landing_was_read_as_last_fetched_as_the_command_line_does() {
    let world = World::new();
    let repository = world.named();
    world.promote("stable", &["0.1.0"]);
    let (address, _) = serving(&world.host, Duration::from_secs(60));
    let (_, first) = json(address, &format!("{}/status.json", repository.path()));
    assert_eq!(
        first["value"]["channels"]["value"]["landing"], "source",
        "{first}"
    );
    let (_, page) = get(address, &format!("{}/status", repository.path()));
    assert!(!page.contains("landing:"), "{page}");
    fs::remove_dir_all(&world.scratch.remote).unwrap();

    let (status, served) = json(address, &format!("{}/status.json", repository.path()));
    assert_eq!(status, 200, "{served}");
    let kept = &served["value"]["channels"]["value"]["landing"]["kept"];
    let at = kept["at"].as_u64().unwrap_or_else(|| panic!("{served}"));
    let said = kept["said"].as_str().unwrap();
    assert!(
        said.contains("does not appear to be a git repository"),
        "{said}"
    );
    assert_eq!(unasked(served.clone()), unasked(world.status()));
    let (status, page) = get(address, &format!("{}/status", repository.path()));
    assert_eq!(status, 200, "{page}");
    assert!(
        page.contains(&format!(
            "<p>landing: as last fetched {}</p>\n<pre>{}</pre>",
            timber::render::utc(at),
            timber::commands::serve::page::escape(said)
        )),
        "{page}"
    );
    assert!(page.contains("<td>stable</td>"), "{page}");
}

#[test]
fn a_promotion_after_one_request_is_shown_by_the_next() {
    let world = World::new();
    world.named();
    world.promote("stable", &["0.1.0"]);
    let (address, _) = serving(&world.host, Duration::from_secs(60));
    let head = |served: &Value| {
        served["value"]["channels"]["value"]["channels"][1]["at"]["value"]["head"]["to"]["version"]
            .clone()
    };
    let (_, first) = json(address, "/o/r/status.json");
    assert_eq!(head(&first), "0.1.0", "{first}");
    world.promote("stable", &["0.2.0"]);
    let (_, second) = json(address, "/o/r/status.json");
    assert_eq!(head(&second), "0.2.0", "{second}");
    let (_, page) = get(address, "/o/r/status");
    assert!(page.contains("0.2.0"), "{page}");
}

#[test]
fn the_changelog_beside_the_channel_shows_each_promotions_notes_escaped_or_why_they_cannot_be_read()
{
    let world = World::new();
    let repo = world.repo();
    repo.write("site/index.html", "<p>the page</p>");
    let feature = repo.commit("feat: <b>the</b> page & its notes");
    repo.push();
    repo.git(&["tag", "v0.1.0", &feature]);
    let repository = world.named();
    // 0.2.0 was never tagged here: its notes cannot be rendered.
    world.promote("stable", &["0.1.0", "0.2.0"]);
    let (address, _) = serving(&world.host, Duration::from_secs(60));
    let (_, served) = json(address, &format!("{}/status.json", repository.path()));
    assert_eq!(unasked(served.clone()), unasked(world.status()));
    let notes = &served["value"]["channels"]["value"]["channels"][1]["notes"];
    assert_eq!(notes[0]["version"], "0.1.0", "{served}");
    assert_eq!(notes[0]["notes"]["answer"], "ok", "{served}");
    let text = notes[0]["notes"]["value"].as_str().unwrap();
    assert!(text.contains("<b>the</b> page & its notes"), "{text}");
    assert_eq!(notes[1]["version"], "0.2.0", "{served}");
    assert_eq!(notes[1]["notes"]["answer"], "unreadable", "{served}");
    let said = notes[1]["notes"]["unreadable"]["said"].as_str().unwrap();
    assert!(said.contains("no tag v0.2.0"), "{said}");

    let (status, page) = get(address, &format!("{}/status", repository.path()));
    assert_eq!(status, 200, "{page}");
    // Inside each promotion's item, after who asked, the notes of the
    // version it moved to, the same text the JSON carries, escaped.
    assert!(
        page.contains(&format!(
            "asked by gate, under the policy at <code>{}</code> satisfied by <code>{}</code>, \
             2026-10-26T07:33:20Z; claim",
            world.head,
            acceptance("0.1.0")
        )),
        "{page}"
    );
    assert!(
        page.contains(&format!("<pre>{}</pre></li>", escape(text))),
        "{page}"
    );
    assert!(
        page.contains("&lt;b&gt;the&lt;/b&gt; page &amp; its notes"),
        "{page}"
    );
    assert!(!page.contains("<b>the</b>"), "{page}");
    // In the unreadable version's place, the answer with the history's
    // words.
    assert!(
        page.contains(&format!(
            "could not read the notes of repository 0.2.0</span><pre>{}</pre></li>",
            escape(said)
        )),
        "{page}"
    );
}

#[test]
fn the_notes_of_a_version_this_host_published_are_shown_beneath_its_row() {
    let world = World::new();
    let repo = world.repo();
    repo.write("site/index.html", "<p>the record</p>");
    let fix = Revision::new(repo.commit("fix: <i>the</i> record & its notes"));
    repo.push();
    let history = History::open(&repo.path).unwrap();
    let kept = Kept::at(&history);
    fs::remove_dir_all(kept.dir()).unwrap();
    kept.write(&Entry {
        component: ComponentName::new("repository"),
        version: Version::new("0.1.0"),
        tag: "v0.1.0".into(),
        revision: fix.clone(),
        branch: Branch::new(MAIN),
        index: Digest::of(b"0.1.0's index"),
        store: Source::new(Seam::Store, "layout /srv/timber/layout"),
        notes: Some(Noted {
            last: None,
            commits: vec![fix.clone(), world.head.clone()],
        }),
    })
    .unwrap();
    let repository = world.named();
    let (address, _) = serving(&world.host, Duration::from_secs(60));
    let (_, served) = json(address, &format!("{}/status.json", repository.path()));
    let version = &served["value"]["published"]["value"]["components"][0]["versions"][0];
    assert_eq!(version["notes"]["answer"], "ok", "{served}");
    let text = version["notes"]["value"].as_str().unwrap();
    assert!(text.contains("<i>the</i> record & its notes"), "{text}");

    let (status, page) = get(address, &format!("{}/status", repository.path()));
    assert_eq!(status, 200, "{page}");
    let row = page
        .find(&format!("<td><code>{fix}</code></td>"))
        .unwrap_or_else(|| panic!("no row for the version: {page}"));
    let notes = format!(
        "</tr>\n<tr><td colspan=\"7\"><pre>{}</pre></td></tr>",
        escape(text)
    );
    assert_eq!(
        page[row..]
            .find(&notes)
            .map(|at| page[row..row + at].contains("</tr>")),
        Some(false),
        "the notes are not the row beneath the version's: {page}"
    );
    assert!(!page.contains("<i>the</i>"), "{page}");
}

#[test]
fn a_repository_none_named_is_404_with_its_absent_answer() {
    let world = World::new();
    world.named();
    let (address, _) = serving(&world.host, Duration::from_secs(60));
    let (status, served) = json(address, "/agentplot/timber/status.json");
    assert_eq!(status, 404, "{served}");
    assert_eq!(served["answer"], "absent", "{served}");
    let what = served["absent"]["what"].as_str().unwrap();
    assert!(what.contains("agentplot/timber"), "{what}");
    assert!(what.contains("has named it"), "{what}");
    let (status, page) = get(address, "/agentplot/timber/status");
    assert_eq!(status, 404);
    assert!(page.contains("has named it"), "{page}");
}

#[test]
fn another_path_is_404_and_another_method_405() {
    let world = World::new();
    world.named();
    let (address, _) = serving(&world.host, Duration::from_secs(60));
    for path in ["/nothing", "/o/r", "//r/status", "/o//npm/x"] {
        assert_eq!(get(address, path).0, 404, "{path}");
    }
    assert_eq!(ask(address, "POST", "/serving.json").0, 405);
    assert_eq!(ask(address, "HEAD", "/o/r/status").0, 405);
}

#[test]
fn no_request_writes_a_file_of_the_record_or_the_repository() {
    let world = World::new();
    world.named();
    world.promote("stable", &["0.1.0"]);
    let record = Kept::at(&History::open(&world.repo().path).unwrap())
        .dir()
        .to_path_buf();
    let repositories = world.host.state.join("repositories");
    let before = (
        files(&world.repo().path),
        files(&record),
        fs::read(&repositories).unwrap(),
    );
    let (address, _) = serving(&world.host, Duration::from_secs(60));
    for path in [
        "/serving.json",
        "/",
        "/status.json",
        "/o/r/status.json",
        "/o/r/status",
        "/x/y/status.json",
        "/elsewhere",
    ] {
        get(address, path);
    }
    assert_eq!(
        (
            files(&world.repo().path),
            files(&record),
            fs::read(&repositories).unwrap()
        ),
        before,
        "a request wrote a file of the record or the repository"
    );
}

// Requirement: The root answers every repository it knows together

#[test]
fn two_repositories_named_are_answered_together_at_the_root() {
    let first = World::new();
    let second = World::beside(&first.host);
    let two = second.named_as("b", "two");
    let one = first.named_as("a", "one");
    first.promote("stable", &["0.1.0", "0.2.0"]);
    second.promote("beta", &["0.3.0"]);
    let (address, _) = serving(&first.host, Duration::from_secs(60));
    let before = now();
    let (status, served) = json(address, "/status.json");
    let after = now();
    assert_eq!(status, 200, "{served}");
    assert_eq!(served["answer"], "ok", "{served}");
    let asked = served["value"]["asked_at"].as_u64().unwrap();
    assert!((before..=after).contains(&asked), "{asked}");
    let rows = served["value"]["repositories"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "{served}");
    assert_eq!(rows[0], first.row(&one));
    assert_eq!(rows[1], second.row(&two));
    assert_eq!(rows[0]["value"]["repository"]["owner"], "a");

    let (status, page) = get(address, "/");
    assert_eq!(status, 200, "{page}");
    for linked in ["href=\"/a/one/status\"", "href=\"/b/two/status\""] {
        assert!(page.contains(linked), "{linked}: {page}");
    }
    for head in [
        "repository/beta empty",
        "repository/stable 0.2.0",
        "repository/beta 0.3.0",
        "repository/stable empty",
    ] {
        assert!(page.contains(head), "{head}: {page}");
    }
    assert_eq!(page.matches("no target is read").count(), 2, "{page}");
}

#[test]
fn a_row_whose_checkout_is_gone_is_unreadable_naming_the_path_beside_the_other() {
    let first = World::new();
    let second = World::beside(&first.host);
    first.named_as("a", "one");
    second.named_as("b", "two");
    let gone = second.repo().path.clone();
    fs::remove_dir_all(&gone).unwrap();
    let (address, _) = serving(&first.host, Duration::from_secs(60));
    let (status, served) = json(address, "/status.json");
    assert_eq!(status, 200, "{served}");
    let rows = &served["value"]["repositories"];
    assert_eq!(rows[0]["answer"], "ok", "{served}");
    assert_eq!(rows[1]["answer"], "unreadable", "{served}");
    let what = rows[1]["unreadable"]["what"].as_str().unwrap();
    assert!(what.contains(&gone.display().to_string()), "{what}");
    let (_, page) = get(address, "/");
    assert!(page.contains("href=\"/a/one/status\""), "{page}");
    assert!(page.contains(&gone.display().to_string()), "{page}");
}

#[test]
fn a_host_that_named_nothing_answers_an_empty_list_and_says_how_to_name_one() {
    let world = World::new();
    let (address, _) = serving(&world.host, Duration::from_secs(60));
    let (status, served) = json(address, "/status.json");
    assert_eq!(status, 200, "{served}");
    assert_eq!(served["value"]["repositories"], serde_json::json!([]));
    let (status, page) = get(address, "/");
    assert_eq!(status, 200);
    assert!(page.contains("timber status --page"), "{page}");
}

#[test]
fn a_file_of_repositories_that_does_not_parse_is_the_roots_unreadable_answer() {
    let world = World::new();
    world.named();
    fs::write(
        world.host.state.join("repositories"),
        "[\"o/r\"\ncheckout = 3\n",
    )
    .unwrap();
    let (address, _) = serving(&world.host, Duration::from_secs(60));
    let (status, served) = json(address, "/status.json");
    assert_eq!(status, 500, "{served}");
    assert_eq!(served["answer"], "unreadable", "{served}");
    let said = served["unreadable"]["said"].as_str().unwrap();
    assert!(said.contains("TOML parse error"), "{said}");
    let (status, page) = get(address, "/");
    assert_eq!(status, 500);
    assert!(page.contains("TOML parse error"), "{page}");
}

// Requirement: The first asker that finds none starts it, and one lock
// keeps it one

#[test]
fn the_page_is_named_and_the_process_started_where_none_answers() {
    let world = World::new();
    let repo = world.repo();
    // The remote's address names the repository; git reads it from the
    // scratch remote.
    let address_of = "https://example.com/o/r.git";
    repo.git(&["remote", "set-url", "origin", address_of]);
    repo.git(&[
        "config",
        &format!("url.{}.insteadOf", world.scratch.remote.display()),
        address_of,
    ]);
    let address = free();
    let host = world.host.clone();
    let answered = page(&repo.path, &world.host, address, move || {
        thread::spawn(move || serve_on(address, &host, Duration::from_secs(60)));
        Ok(())
    });
    assert_eq!(answered, Ok(format!("http://{address}/o/r/status")));
    let named = RepositoryName {
        owner: "o".into(),
        name: "r".into(),
    };
    assert_eq!(checkout_of(&world.host, &named), Ok(repo.path.clone()));
    let (status, served) = json(address, "/o/r/status.json");
    assert_eq!(status, 200, "{served}");
    assert_eq!(
        served["value"]["channels"]["answer"], "ok",
        "the landing and ledger answer through the rewritten remote: {served}"
    );

    // Asked again, the process answers and nothing is started.
    let again = page(&repo.path, &world.host, address, || {
        panic!("started a second")
    });
    assert_eq!(again, Ok(format!("http://{address}/o/r/status")));
}

#[test]
fn a_second_serve_with_the_lock_held_returns_at_once_saying_the_first_answers() {
    let world = World::new();
    let (address, _) = serving(&world.host, Duration::from_secs(60));
    let other = free();
    let started = Instant::now();
    assert_eq!(
        serve_on(other, &world.host, Duration::from_secs(60)),
        Ok(Served::Held)
    );
    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(
        TcpStream::connect(other).is_err(),
        "the second bound {other}"
    );
    assert_eq!(get(address, "/serving.json").0, 200);
}

#[test]
fn two_askers_that_find_none_each_start_one_and_one_serves_both() {
    let world = World::new();
    let address = free();
    // Each start waits for the other, so both askers find none.
    let starts = Arc::new((Mutex::new(0usize), Condvar::new()));
    let ended = Arc::new(Mutex::new(Vec::new()));
    let askers: Vec<_> = (0..2)
        .map(|_| {
            let (host, starts, ended) = (world.host.clone(), starts.clone(), ended.clone());
            thread::spawn(move || {
                find_or_start(address, move || {
                    let (count, both) = &*starts;
                    let mut count = count.lock().unwrap();
                    *count += 1;
                    both.notify_all();
                    let _ = both
                        .wait_timeout_while(count, Duration::from_secs(5), |c| *c < 2)
                        .unwrap();
                    thread::spawn(move || {
                        let served = serve_on(address, &host, Duration::from_secs(1));
                        ended.lock().unwrap().push(served);
                    });
                    Ok(())
                })
            })
        })
        .collect();
    let found: Vec<_> = askers
        .into_iter()
        .map(|asker| asker.join().unwrap().expect("each asker found one"))
        .collect();
    assert_eq!(*starts.0.lock().unwrap(), 2, "both askers started one");
    assert_eq!(found[0].pid, found[1].pid);
    let deadline = Instant::now() + Duration::from_secs(10);
    while ended.lock().unwrap().len() < 2 && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(50));
    }
    let mut ended = ended.lock().unwrap().clone();
    ended.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(ended, [Ok(Served::Held), Ok(Served::Idle)]);
}

#[test]
fn an_idle_process_ends_and_lets_the_lock_go() {
    let world = World::new();
    let (address, server) = serving(&world.host, Duration::from_secs(1));
    let asked = Instant::now();
    assert_eq!(get(address, "/serving.json").0, 200);
    assert_eq!(server.join().unwrap(), Ok(Served::Idle));
    assert!(asked.elapsed() >= Duration::from_secs(1));
    let lock = fs::File::open(world.host.state.join(SERVING)).unwrap();
    // A git another test spawns at that moment may hold a copy of the
    // lock's descriptor until it runs, a few milliseconds.
    let ended = Instant::now();
    while lock.try_lock().is_err() {
        assert!(
            ended.elapsed() < Duration::from_secs(2),
            "the lock was not let go"
        );
        thread::sleep(Duration::from_millis(10));
    }
    drop(lock);

    // The next asker starts one again.
    let again = free();
    let host = world.host.clone();
    let found = find_or_start(again, move || {
        thread::spawn(move || serve_on(again, &host, Duration::from_secs(1)));
        Ok(())
    });
    assert_eq!(found.map(|s| s.pid), Ok(std::process::id()));
}

// Requirement: The process serves each served layout under its repository's path

const ECO: &str = "eco";

fn eco() -> Ecosystem {
    Ecosystem::new(ECO)
}

fn pinned(version: &str) -> Pinned {
    Pinned {
        name: EcosystemName::new("pkg"),
        version: Version::new(version),
    }
}

/// Acceptance by the gate's signer, `gate`.
fn accepted_by_the_gate() -> RequiredClaim {
    RequiredClaim {
        kind: ClaimKind::Acceptance,
        check: None,
        signers: vec![NamedSigner {
            id: SignerId::new("gate"),
            verified_by: "gate's key".into(),
            kinds: vec![ClaimKind::Acceptance],
            checks: vec![],
        }],
        asked_of: AskedOf::Every,
    }
}

/// `version` of `pkg` published to `store` with its index named where a
/// pin resolves, accepted by the gate where `accepted`.
fn publish(store: &FakeStore, version: &str, accepted: bool) {
    let meta = ArtifactMeta::new(eco(), EcosystemName::new("pkg"), Version::new(version)).unwrap();
    let layer = store
        .put_bytes(format!("pkg {version}").as_bytes())
        .unwrap();
    let artifact = Artifact {
        digest: store
            .put_artifact(&Manifest::Artifact {
                meta: meta.clone(),
                layers: vec![layer.digest],
            })
            .unwrap()
            .digest,
        meta,
    };
    let index = IndexMeta::new(
        ComponentName::new("pkg"),
        Version::new(version),
        vec![artifact],
    )
    .unwrap();
    let index = store.put_artifact(&Manifest::Index(index)).unwrap().digest;
    let name = artifact_name(&eco(), &EcosystemName::new("pkg"), &Version::new(version));
    store.name(&name, &index, None).unwrap();
    if accepted {
        let gate = SignerId::new("gate");
        let signer = FakeSigner::new();
        signer.hold(&gate);
        let claim = Claim {
            subject: index.clone(),
            kind: ClaimKind::Acceptance,
            revision: Revision::new("p1b11s4"),
            build: None,
            body: Value::Null,
        };
        store
            .attach(&index, &signer.sign(&claim, &gate).unwrap())
            .unwrap();
    }
}

/// A checkout of `agentplot/consumer` named to the host, over fakes
/// binding the ecosystem's fake projection: its head landed on `main`,
/// whose policy for layouts requires acceptance; `pkg 1.2.0` and `1.1.0`
/// published and accepted and `1.0.0` published unaccepted; the layout
/// synced from a lockfile pinning `1.2.0`.
struct Consumer {
    world: World,
    fakes: Fakes,
    projection: FakeProjection,
    tree: PathBuf,
}

impl Consumer {
    fn new() -> Consumer {
        let world = World::new();
        world.named_as("agentplot", "consumer");
        let mut fakes = Fakes::new();
        let projection = fakes.projection(&eco());
        let main = Branch::new(MAIN);
        fakes.landing.land(&main, &world.head);
        fakes.policy.require_at(
            &world.head,
            Audience::Layout,
            Required::Layout {
                claims: vec![accepted_by_the_gate()],
                given: false,
                signers: vec![],
            },
        );
        publish(&fakes.store, "1.2.0", true);
        publish(&fakes.store, "1.1.0", true);
        publish(&fakes.store, "1.0.0", false);
        let history = History::open(&world.repo().path).unwrap();
        let tree = history.workdir().unwrap();
        projection.lockfile(&tree.display().to_string(), &[pinned("1.2.0")]);
        let synced = sync_checkout(
            &fakes.seams(),
            &Layouts::at(&history),
            &CheckoutAsk {
                branch: main,
                revision: world.head.clone(),
                tree: tree.clone(),
                when: now(),
                trusted: vec![],
                repository: Ok(RepositoryName {
                    owner: "agentplot".into(),
                    name: "consumer".into(),
                }),
            },
        )
        .unwrap();
        assert_eq!(
            synced.served.get(&eco()),
            Some(&Ok(
                "http://127.0.0.1:7420/agentplot/consumer/eco/".to_string()
            ))
        );
        Consumer {
            world,
            fakes,
            projection,
            tree,
        }
    }

    /// The fakes as the one binding a checkout is bound to, for sync.
    fn binding(&self) -> Binding {
        Binding {
            seams: self.fakes.seams(),
            store: Source::new(Seam::Store, "fake"),
            signer: None,
            gate: Teller::Job("the gate's job".into()),
            trust: vec![],
        }
    }

    /// `GET` of `path` asked of the process's own handling of a request,
    /// each checkout bound to the fakes, for sync alone.
    fn get(&self, path: &str) -> Response {
        self.ask(path, "GET")
    }

    /// `method` of `path`, asked as [`Consumer::get`] asks, of the process
    /// at [`asked_at`].
    fn ask(&self, path: &str, method: &str) -> Response {
        respond_with(
            asked_at(),
            &self.world.host,
            &|checkout, command| {
                assert_eq!(command, For::Sync, "a served layout bound for another");
                assert_eq!(checkout, self.world.repo().path, "another checkout bound");
                Ok(self.binding())
            },
            method,
            path,
        )
    }

    /// The versions the layout's record holds.
    fn held(&self) -> Vec<String> {
        let history = History::open(&self.world.repo().path).unwrap();
        Layouts::at(&history)
            .read(&eco())
            .unwrap()
            .held
            .iter()
            .map(|h| h.version.to_string())
            .collect()
    }
}

/// The address the process's own handling of a request is asked at in
/// these tests, bound by nothing and never the core's.
fn asked_at() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 7499))
}

fn text(response: &Response) -> String {
    String::from_utf8(response.body.clone()).unwrap()
}

#[test]
fn a_served_layout_is_its_projections_response_from_the_checkout_that_named_it() {
    let consumer = Consumer::new();
    let response = consumer.get("/agentplot/consumer/eco/pkg/1.2.0");
    assert_eq!(response.status, 200, "{}", text(&response));
    assert_eq!(response.content_type, "application/json");
    let served: Value = serde_json::from_slice(&response.body).unwrap();
    assert_eq!(served["name"], "pkg", "{served}");
    assert_eq!(served["version"], "1.2.0", "{served}");
    assert_eq!(
        consumer.projection.requests(),
        [
            Request {
                method: "GET".into(),
                path: String::new(),
                base: "http://127.0.0.1:7420/agentplot/consumer/eco".into(),
            },
            Request {
                method: "GET".into(),
                path: "pkg/1.2.0".into(),
                base: "http://127.0.0.1:7499/agentplot/consumer/eco".into(),
            }
        ],
        "sync's probe at the core's address, then the request below the layout's prefix \
         at the address the process was asked at"
    );
    assert_eq!(consumer.projection.misses(), []);
    let rendered: Vec<String> = consumer
        .projection
        .renders()
        .into_iter()
        .map(|(at, _)| at)
        .collect();
    assert_eq!(
        rendered,
        [place(&consumer.tree, &eco()).display().to_string()],
        "served from a layout the checkout's sync did not render"
    );
}

#[test]
fn a_version_the_layout_lacks_is_rendered_into_it_by_the_core_on_the_request() {
    let consumer = Consumer::new();
    let response = consumer.get("/agentplot/consumer/eco/pkg/1.1.0");
    assert_eq!(response.status, 200, "{}", text(&response));
    assert_eq!(consumer.projection.misses(), [pinned("1.1.0")]);
    assert_eq!(consumer.held(), ["1.2.0", "1.1.0"]);
    let renders: Vec<Vec<String>> = consumer
        .projection
        .renders()
        .into_iter()
        .map(|(at, artifacts)| {
            assert_eq!(at, place(&consumer.tree, &eco()).display().to_string());
            artifacts
                .iter()
                .map(|a| a.meta.version().to_string())
                .collect()
        })
        .collect();
    assert_eq!(renders, [vec!["1.2.0"], vec!["1.2.0", "1.1.0"]]);
}

#[test]
fn a_version_the_policy_refuses_is_the_projections_refusal_and_the_layout_is_unchanged() {
    let consumer = Consumer::new();
    let response = consumer.get("/agentplot/consumer/eco/pkg/1.0.0");
    assert_eq!(response.status, 403, "{}", text(&response));
    let words = text(&response);
    assert!(words.contains("pkg 1.0.0"), "{words}");
    assert!(words.contains("acceptance"), "{words}");
    assert_eq!(consumer.held(), ["1.2.0"]);
    assert_eq!(consumer.projection.renders().len(), 1, "rendered again");
}

#[test]
fn a_served_layouts_method_is_its_projections_to_answer() {
    let consumer = Consumer::new();
    let response = consumer.ask("/agentplot/consumer/eco/pkg/1.2.0", "HEAD");
    assert_eq!(response.status, 200, "{}", text(&response));
    let asked = consumer.projection.requests();
    assert_eq!(asked.last().map(|r| r.method.as_str()), Some("HEAD"));
    let mut written = Vec::new();
    response.write_head(&mut written).unwrap();
    let head = String::from_utf8(written).unwrap();
    assert!(
        head.ends_with("\r\n\r\n")
            && head.contains(&format!("\r\nContent-Length: {}\r\n", response.body.len())),
        "{head}"
    );
    let posted = consumer.ask("/agentplot/consumer/eco/pkg/1.2.0", "POST");
    assert_eq!(
        consumer
            .projection
            .requests()
            .last()
            .map(|r| r.method.clone()),
        Some("POST".to_string()),
        "the process answered {} itself",
        posted.status
    );
}

#[test]
fn a_response_is_written_with_its_bytes_exactly_and_their_length() {
    let response = Response {
        status: 200,
        content_type: "application/octet-stream".into(),
        body: (0..=255).collect(),
    };
    let mut written = Vec::new();
    response.write(&mut written).unwrap();
    let blank = written
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("a head");
    let head = String::from_utf8(written[..blank].to_vec()).unwrap();
    assert!(head.starts_with("HTTP/1.1 200 OK\r\n"), "{head}");
    assert!(head.contains("\r\nContent-Length: 256\r\n"), "{head}");
    assert!(
        head.contains("\r\nContent-Type: application/octet-stream\r\n"),
        "{head}"
    );
    assert_eq!(&written[blank + 4..], response.body.as_slice());
    for (status, reason) in [(403, "Forbidden"), (502, "Bad Gateway")] {
        let mut written = Vec::new();
        Response {
            status,
            ..response.clone()
        }
        .write(&mut written)
        .unwrap();
        let line = format!("HTTP/1.1 {status} {reason}\r\n");
        assert!(written.starts_with(line.as_bytes()), "{status}");
    }
}

#[test]
fn an_ecosystem_nothing_serves_is_404_naming_it_and_nothing_is_rendered() {
    let mut consumer = Consumer::new();
    let file = consumer.fakes.projection(&Ecosystem::new("file"));
    file.serves_none();
    let response = consumer.get("/agentplot/consumer/file/notes.txt");
    assert_eq!(response.status, 404, "{}", text(&response));
    assert!(text(&response).contains("file"), "{}", text(&response));
    assert!(file.renders().is_empty());
    let response = consumer.get("/agentplot/consumer/other/x");
    assert_eq!(response.status, 404);
    assert!(text(&response).contains("other"), "{}", text(&response));
    assert_eq!(consumer.projection.misses(), []);
}

#[test]
fn a_repository_none_named_and_a_checkout_that_cannot_be_bound_are_their_own_answers() {
    let consumer = Consumer::new();
    let response = consumer.get("/agentplot/timber/eco/pkg/1.2.0");
    assert_eq!(response.status, 404);
    assert!(
        text(&response).contains("has named it"),
        "{}",
        text(&response)
    );
    let unbound = Not::unreadable(
        &Source::new(Seam::Host, "bindings"),
        "the bindings file",
        Some("expected `=`".into()),
    );
    let response = respond_with(
        asked_at(),
        &consumer.world.host,
        &|_, _| Err(unbound.clone()),
        "GET",
        "/agentplot/consumer/eco/pkg/1.2.0",
    );
    assert_eq!(response.status, 500);
    assert_eq!(text(&response), format!("{unbound}\n"));
    assert!(consumer.projection.requests().len() == 1, "served unbound");
}

// Requirement: The first asker that finds none starts it, and one lock keeps it one

/// A binding over fakes made anew on each call, on any thread: the
/// ecosystem's fake projection, serving or scripted to serve none.
fn fresh(serving: bool) -> Binder {
    Arc::new(move |_, _| {
        let mut fakes = Fakes::new();
        let projection = fakes.projection(&eco());
        if !serving {
            projection.serves_none();
        }
        Ok(Binding {
            seams: fakes.seams(),
            store: Source::new(Seam::Store, "fake"),
            signer: None,
            gate: Teller::Job("the gate's job".into()),
            trust: vec![],
        })
    })
}

/// An idle of moments, which the tests wait out several times over.
const MOMENTS: Duration = Duration::from_millis(200);

/// The consumer's layout rendered: its place made, as a projection that
/// writes files makes it, beside the record the sync wrote.
fn rendered(consumer: &Consumer) -> PathBuf {
    let at = place(&consumer.tree, &eco());
    fs::create_dir_all(&at).unwrap();
    at
}

/// `serve_with` over `host` at a port of the test's own, each checkout
/// bound by `bind`, in a thread, once it answers.
fn serving_with(host: &Host, bind: Binder) -> (SocketAddr, JoinHandle<Answer<Served>>) {
    let address = free();
    let host = host.clone();
    let server = thread::spawn(move || serve_with(address, &host, MOMENTS, bind));
    find_or_start(address, || Ok(())).expect("the process answers");
    (address, server)
}

/// The process's answer once it ends within a few seconds.
fn ends(server: JoinHandle<Answer<Served>>) -> Answer<Served> {
    let asked = Instant::now();
    while !server.is_finished() {
        assert!(
            asked.elapsed() < Duration::from_secs(5),
            "the process did not end"
        );
        thread::sleep(Duration::from_millis(10));
    }
    server.join().unwrap()
}

/// A projection answering every request it is asked with the base it was
/// handed, the fake answering all else.
struct Based(FakeProjection);

impl Projection for Based {
    fn pack(&self, output: &Output, at: &BuildId, version: Option<&PackAt<'_>>) -> Answer<Packed> {
        self.0.pack(output, at, version)
    }

    fn render(&self, contents: Contents<'_>, at: &str) -> Answer<Rendered> {
        self.0.render(contents, at)
    }

    fn serve(&self, _: &Layout, request: &Request, _: Miss<'_>) -> Answer<timber_seams::Response> {
        Ok(timber_seams::Response {
            status: 200,
            content_type: "text/plain; charset=utf-8".into(),
            body: request.base.clone().into_bytes(),
        })
    }

    fn pinned(&self, checkout: &str) -> Answer<Vec<Pin>> {
        self.0.pinned(checkout)
    }

    fn requires(&self, held: &Artifact) -> Answer<Vec<Requirement>> {
        self.0.requires(held)
    }

    fn satisfying(&self, range: &Range, versions: &[Version]) -> Answer<Vec<Version>> {
        self.0.satisfying(range, versions)
    }
}

#[test]
fn a_served_layout_is_handed_the_base_of_the_address_the_process_is_bound_at() {
    let consumer = Consumer::new();
    let bind: Binder = Arc::new(|_, _| {
        let mut seams = Fakes::new().seams();
        seams
            .projections
            .insert(eco(), Box::new(Based(FakeProjection::new(eco()))));
        Ok(Binding {
            seams,
            store: Source::new(Seam::Store, "fake"),
            signer: None,
            gate: Teller::Job("the gate's job".into()),
            trust: vec![],
        })
    });
    let (address, server) = serving_with(&consumer.world.host, bind);
    let (status, body) = get(address, "/agentplot/consumer/eco/pkg/1.2.0");
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        body,
        format!("http://{address}/agentplot/consumer/eco"),
        "the base names another address than the one the process is bound at"
    );
    assert_eq!(ends(server), Ok(Served::Idle));
}

#[test]
fn a_process_stays_past_its_idle_time_while_a_layout_is_served() {
    let consumer = Consumer::new();
    rendered(&consumer);
    let (address, server) = serving_with(&consumer.world.host, fresh(true));
    let (_, first) = json(address, "/serving.json");
    thread::sleep(MOMENTS * 5);
    assert!(
        !server.is_finished(),
        "the process ended while a layout is served"
    );
    let (status, body) = get(address, "/agentplot/consumer/eco/pkg/1.2.0");
    assert_eq!(status, 200, "{body}");
    let (_, again) = json(address, "/serving.json");
    assert_eq!(again["pid"], first["pid"]);
    assert!(!server.is_finished());
}

#[test]
fn a_process_ends_at_its_next_idle_once_the_served_layout_is_removed() {
    let consumer = Consumer::new();
    let at = rendered(&consumer);
    let (_, server) = serving_with(&consumer.world.host, fresh(true));
    thread::sleep(MOMENTS * 3);
    assert!(
        !server.is_finished(),
        "the process ended while a layout is served"
    );
    fs::remove_dir_all(&at).unwrap();
    assert_eq!(ends(server), Ok(Served::Idle));
    let lock = fs::File::open(consumer.world.host.state.join(SERVING)).unwrap();
    lock.try_lock().expect("the lock was let go");
}

#[test]
fn a_process_ends_at_its_next_idle_once_the_checkout_holding_it_is_removed() {
    let consumer = Consumer::new();
    rendered(&consumer);
    let (_, server) = serving_with(&consumer.world.host, fresh(true));
    thread::sleep(MOMENTS * 3);
    assert!(
        !server.is_finished(),
        "the process ended while a layout is served"
    );
    fs::remove_dir_all(&consumer.world.repo().path).unwrap();
    assert_eq!(ends(server), Ok(Served::Idle));
}

#[test]
fn a_process_ends_at_its_idle_where_the_only_layout_is_of_a_projection_serving_none() {
    let consumer = Consumer::new();
    rendered(&consumer);
    let (_, server) = serving_with(&consumer.world.host, fresh(false));
    assert_eq!(ends(server), Ok(Served::Idle));
}
