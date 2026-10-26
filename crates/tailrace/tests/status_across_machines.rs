//! Spec channel-status, two machines and one answer, and S12, and spec
//! host-bindings, status with nothing written, over real profiles: a
//! bare remote; machine A, a clone bound by a file of its own to a plain
//! branch landing, an OCI layout, a host key and the git ref ledger on
//! `origin`, which promotes through `tailrace::run`; and machine B, a
//! second clone on a host with no bindings file, bound as 118 binds such
//! a host; and machine B cut off from that remote, its landing read from
//! the host's copy of the branch as last fetched and each chain as last
//! read, each saying when (60, 61, 77, C.6). Needs git alone, so it is in
//! the fast layer.

mod scratch;

use std::fs;
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::Parser;
use scratch::{Repo, Scratch, MAIN, POLICY_FILE};
use serde_json::Value;
use ssh_key::rand_core::OsRng;
use ssh_key::{Algorithm, LineEnding, PrivateKey};
use tailrace::bindings::{self, For};
use tailrace::commands::serve::host::{find_or_start, name, Host};
use tailrace::commands::serve::serve_on;
use tailrace::profiles::store::layout::Layout;
use tailrace::{run, Binding, Cli};
use tailrace_core::{History, Kept, RepositoryName};
use tailrace_seams::{
    Answer, Claim, ClaimKind, ComponentName, IndexMeta, Manifest, Name, Revision, SignerId, Version,
};

const GATE: &str = "gate";

/// `args` asked through `tailrace::run` in the repository at `at` over
/// `binding`: the exit and what it printed.
fn ask(binding: &Answer<Binding>, at: &Path, args: &[&str]) -> (i32, String) {
    let cli = Cli::try_parse_from(std::iter::once("tailrace").chain(args.iter().copied()))
        .expect("the arguments parse");
    let mut out = Vec::new();
    let code = run(&cli, binding, at, &mut out);
    (code, String::from_utf8(out).unwrap())
}

/// `tailrace status --json` in the repository at `at` over `binding`.
fn status(binding: &Answer<Binding>, at: &Path) -> Value {
    let (code, printed) = ask(binding, at, &["status", "--json"]);
    assert_eq!(code, 0, "{printed}");
    serde_json::from_str(&printed).unwrap()
}

/// `value` with the moment asked set to 0.
fn unasked(mut value: Value) -> Value {
    value["value"]["asked_at"] = Value::from(0);
    value
}

/// The channel named `name` in status's answer `value`.
fn channel<'a>(value: &'a Value, name: &str) -> &'a Value {
    value["value"]["channels"]["value"]["channels"]
        .as_array()
        .unwrap_or_else(|| panic!("no channels answered: {value}"))
        .iter()
        .find(|c| c["channel"]["name"] == name)
        .unwrap_or_else(|| panic!("no channel {name}: {value}"))
}

/// Every file under `dir`, with git's directory left out where `git` is
/// false.
fn files(dir: &Path, git: bool) -> Vec<(PathBuf, Vec<u8>)> {
    let mut all = Vec::new();
    if !dir.exists() {
        return all;
    }
    let mut dirs = vec![dir.to_path_buf()];
    while let Some(at) = dirs.pop() {
        for entry in fs::read_dir(&at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                if git || path.file_name().is_none_or(|n| n != ".git") {
                    dirs.push(path);
                }
            } else {
                all.push((path.clone(), fs::read(&path).unwrap()));
            }
        }
    }
    all.sort();
    all
}

/// The scratch remote; machine A, which put two versions of the one
/// component in its layout, each index carrying the gate's acceptance,
/// and promoted `rc`, which moves by a promotion alone, to the first,
/// then the second, `beta` and `stable` left empty; and machine B, a
/// clone of the remote with no bindings file and an empty home.
struct Two {
    scratch: Scratch,
    a: Repo,
    a_binding: PathBuf,
    a_home: PathBuf,
    b: Repo,
    b_home: PathBuf,
    nowhere: PathBuf,
}

impl Two {
    fn new() -> Two {
        let scratch = Scratch::new();
        let key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).expect("an Ed25519 key");
        let a = scratch.repo.clone();
        a.write(
            POLICY_FILE,
            &format!(
                "outputs = [\"site\"]\n\n[projections]\nsite = \"file\"\n\n\
                 [signers.{GATE}]\nssh = \"{}\"\nkinds = [\"acceptance\", \"promotion\"]\n\n\
                 [channels.beta]\nrequire = [{{ kind = \"acceptance\", signers = [\"{GATE}\"] }}]\n\n\
                 [channels.stable]\nrequire = [{{ kind = \"acceptance\", signers = [\"{GATE}\"] }}]\n\n\
                 [channels.rc]\nrequire = [{{ kind = \"acceptance\", signers = [\"{GATE}\"] }}]\n",
                key.public_key().to_openssh().unwrap()
            ),
        );
        a.write("site/index.html", "<p>hello</p>");
        let first = Revision::new(a.commit("feat: the site"));
        a.write("site/index.html", "<p>hello again</p>");
        let second = Revision::new(a.commit("feat: the site grows"));
        a.git(&["tag", "v0.1.0", first.as_str()]);
        a.git(&["tag", "v0.2.0", second.as_str()]);
        a.git(&["push", "-q", "origin", MAIN, "v0.1.0", "v0.2.0"]);

        let dir = scratch.path().join("a");
        let a_home = dir.join("home");
        let layout = dir.join("layout");
        Layout::create(&layout).unwrap();
        let signing = a_home.join(".config/tailrace/signing");
        fs::create_dir_all(signing.parent().unwrap()).unwrap();
        key.write_openssh_file(&signing, LineEnding::LF).unwrap();
        fs::set_permissions(&signing, fs::Permissions::from_mode(0o600)).unwrap();
        let a_binding = bindings::path(Some(dir.join("config").as_os_str()), None).unwrap();
        fs::create_dir_all(a_binding.parent().unwrap()).unwrap();
        fs::write(
            &a_binding,
            format!(
                "[store]\nlayout = {:?}\n\n[landing]\nbranch = {{ remote = \"origin\" }}\n\n\
                 [ledger]\ngit_ref = {{ remote = \"origin\" }}\n\n\
                 [signing]\nsigner = \"{GATE}\"\nkey = \"~/.config/tailrace/signing\"\n",
                layout.display().to_string()
            ),
        )
        .unwrap();
        let two = Two {
            b: scratch.clone_as("b"),
            b_home: scratch.path().join("b-host/home"),
            nowhere: scratch.path().join("b-host/config/tailrace/config.toml"),
            scratch,
            a,
            a_binding,
            a_home,
        };
        fs::create_dir_all(&two.b_home).unwrap();

        let binding = two.bind_a(For::Promote).expect("machine A binds");
        for (version, revision) in [("0.1.0", &first), ("0.2.0", &second)] {
            let meta = IndexMeta::new(
                ComponentName::new("repository"),
                Version::new(version),
                vec![],
            )
            .unwrap();
            let store = &binding.seams.store;
            let index = store.put_artifact(&Manifest::Index(meta)).unwrap().digest;
            store
                .name(&Name::new(format!("v{version}")), &index, None)
                .unwrap();
            let acceptance = Claim {
                subject: index.clone(),
                kind: ClaimKind::Acceptance,
                revision: revision.clone(),
                build: None,
                body: serde_json::json!({ "version": version }),
            };
            let envelope = binding.seams.signers[0]
                .sign(&acceptance, &SignerId::new(GATE))
                .expect("the gate's key signs");
            store.attach(&index, &envelope).unwrap();
        }
        let promote = Ok(binding);
        for args in [
            &["promote", "0.1.0", "--to", "rc"][..],
            &["promote", "0.2.0", "--to", "rc", "--from", "0.1.0"][..],
        ] {
            let (code, printed) = ask(&promote, &two.a.path, args);
            assert_eq!(code, 0, "{args:?}: {printed}");
        }
        two
    }

    fn bind_a(&self, command: For) -> Answer<Binding> {
        bindings::bind(&self.a_binding, Some(&self.a_home), &self.a.path, command)
    }

    /// Machine B's binding: a bindings file's path that is not there,
    /// and the empty home.
    fn bind_b(&self) -> Answer<Binding> {
        assert!(!self.nowhere.exists());
        bindings::bind(&self.nowhere, Some(&self.b_home), &self.b.path, For::Status)
    }

    /// Machine B as its serving process reads the host.
    fn b_host(&self) -> Host {
        Host {
            state: self.scratch.path().join("b-host/state/tailrace"),
            config: self.nowhere.clone(),
            home: Some(self.b_home.clone()),
        }
    }
}

/// A port for one test's process, never the core's: below the range the
/// system hands to connections, one of its own per test, and never
/// bound to see it free (see `serve.rs`).
fn free() -> SocketAddr {
    use std::sync::atomic::{AtomicU16, Ordering};
    static NEXT: AtomicU16 = AtomicU16::new(0);
    let port = 29_000 + (std::process::id() % 300) as u16 * 3 + NEXT.fetch_add(1, Ordering::SeqCst);
    SocketAddr::from(([127, 0, 0, 1], port))
}

/// `GET path` at `address`: the body.
fn body(address: SocketAddr, path: &str) -> String {
    use std::io::{Read, Write};
    let mut stream = TcpStream::connect(address).unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut answered = String::new();
    stream.read_to_string(&mut answered).unwrap();
    let (_, body) = answered.split_once("\r\n\r\n").unwrap();
    body.to_string()
}

/// `GET path` at `address`: the body as JSON.
fn get(address: SocketAddr, path: &str) -> Value {
    serde_json::from_str(&body(address, path)).unwrap()
}

/// B's serving process, named `o/r`, answering at a port of its own.
fn serve_b(two: &Two) -> SocketAddr {
    let host = two.b_host();
    let repository = RepositoryName {
        owner: "o".into(),
        name: "r".into(),
    };
    name(&host, &repository, &two.b.path).unwrap();
    let address = free();
    std::thread::spawn(move || serve_on(address, &host, Duration::from_secs(30)));
    find_or_start(address, || Ok(())).expect("B's process answers");
    address
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[test]
fn a_machine_that_never_published_answers_every_channel_as_the_one_that_promoted() {
    let two = Two::new();
    let ours = status(&two.bind_a(For::Status), &two.a.path);

    // Machine B, bound with nothing written, answers the same channels.
    let b_tree = files(&two.b.path, false);
    let theirs = status(&two.bind_b(), &two.b.path);
    let rc = channel(&theirs, "rc");
    assert_eq!(rc["at"]["answer"], "ok", "{theirs}");
    assert_eq!(rc["at"]["value"]["head"]["to"]["version"], "0.2.0");
    assert_eq!(rc["at"]["value"]["chain"].as_array().map(Vec::len), Some(2));
    assert_eq!(rc["at"], channel(&ours, "rc")["at"], "{theirs}");
    for empty in ["beta", "stable"] {
        assert_eq!(
            channel(&theirs, empty)["at"]["value"]["state"],
            "empty",
            "{theirs}"
        );
    }
    assert_eq!(
        theirs["value"]["channels"]["value"]["revision"],
        ours["value"]["channels"]["value"]["revision"]
    );
    assert_eq!(theirs["value"]["published"]["answer"], "absent", "{theirs}");
    assert_eq!(theirs["value"]["environments"]["answer"], "unreadable");
    assert_eq!(files(&two.b_home, true), [], "B's home was written");
    assert!(!two.nowhere.exists(), "a bindings file was made");
    assert_eq!(files(&two.b.path, false), b_tree, "B's tree was written");
    let record = Kept::at(&History::open(&two.b.path).unwrap())
        .dir()
        .to_path_buf();
    assert!(!record.exists(), "B's record was made");
    let refs = two
        .b
        .git(&["for-each-ref", "--format=%(refname)", "refs/tailrace"]);
    assert!(
        refs.lines().all(|r| r.starts_with("refs/tailrace/ledger/")
            || r.starts_with("refs/tailrace/fetched/")
            || r.starts_with("refs/tailrace/landing/")),
        "{refs}"
    );

    // The page's JSON for B is B's status answer.
    let address = serve_b(&two);
    let page = get(address, "/o/r/status.json");
    assert_eq!(unasked(page), unasked(status(&two.bind_b(), &two.b.path)));
}

#[test]
fn everything_kept_deleted_answers_the_same_channels_and_the_same_record_once_rebuilt() {
    let two = Two::new();
    let a = two.bind_a(For::Rebuild);
    let (code, printed) = ask(&a, &two.a.path, &["rebuild"]);
    assert_eq!(code, 0, "{printed}");
    let a_status = two.bind_a(For::Status);
    let ours = status(&a_status, &two.a.path);
    assert_eq!(ours["value"]["published"]["answer"], "ok", "{ours}");
    let theirs = status(&two.bind_b(), &two.b.path);

    // B's copies of the ledger and A's record deleted.
    for line in two
        .b
        .git(&["for-each-ref", "--format=%(refname)", "refs/tailrace"])
        .lines()
    {
        two.b.git(&["update-ref", "-d", line]);
    }
    let record = Kept::at(&History::open(&two.a.path).unwrap())
        .dir()
        .to_path_buf();
    fs::remove_dir_all(&record).unwrap();

    let again = status(&two.bind_b(), &two.b.path);
    assert_eq!(
        again["value"]["channels"], theirs["value"]["channels"],
        "B answers other channels with its copies deleted"
    );
    let unrebuilt = status(&a_status, &two.a.path);
    assert_eq!(unrebuilt["value"]["published"]["answer"], "absent");
    assert_eq!(unrebuilt["value"]["channels"], ours["value"]["channels"]);
    let (code, printed) = ask(&a, &two.a.path, &["rebuild"]);
    assert_eq!(code, 0, "{printed}");
    assert_eq!(unasked(status(&a_status, &two.a.path)), unasked(ours));
}

/// Machine B, cut off from the one remote holding its branch and its
/// ledger, shows every channel from the ledger's chain as last read, the
/// landing of its head from the host's copy as last fetched, each saying
/// when and with git's words, on the command line, in its JSON and on
/// the page alike (36, 60, 61, 77).
#[test]
fn a_machine_cut_off_from_its_remote_shows_every_channel_and_says_when() {
    let two = Two::new();
    let promote = two.bind_a(For::Promote);
    let (code, printed) = ask(&promote, &two.a.path, &["promote", "0.1.0", "--to", "beta"]);
    assert_eq!(code, 0, "{printed}");
    let before = now();
    let ours = status(&two.bind_b(), &two.b.path);
    let after = now();
    assert_eq!(
        ours["value"]["channels"]["value"]["landing"], "source",
        "{ours}"
    );
    fs::remove_dir_all(&two.scratch.remote).unwrap();
    let words = scratch::git_fails(&two.b.path, &["ls-remote", "origin", "refs/heads/*"]);

    let theirs = status(&two.bind_b(), &two.b.path);
    let when = |read: &Value, what: &str| -> u64 {
        let at = read["kept"]["at"]
            .as_u64()
            .unwrap_or_else(|| panic!("{what} does not say when: {theirs}"));
        assert!((before..=after).contains(&at), "{what} fetched at {at}");
        assert_eq!(read["kept"]["said"], words.as_str(), "{what}: {theirs}");
        at
    };
    let channels = &theirs["value"]["channels"];
    assert_eq!(channels["answer"], "ok", "{theirs}");
    let landed = when(&channels["value"]["landing"], "the landing");
    assert_eq!(
        channels["value"]["revision"], ours["value"]["channels"]["value"]["revision"],
        "{theirs}"
    );
    let mut chains = Vec::new();
    for name in ["beta", "stable"] {
        let at = &channel(&theirs, name)["at"];
        assert_eq!(at["answer"], "ok", "{name}: {theirs}");
        chains.push((name, when(&at["value"]["read"], name)));
        let first = &channel(&ours, name)["at"]["value"];
        assert_eq!(at["value"]["head"], first["head"], "{name}");
        assert_eq!(at["value"]["chain"], first["chain"], "{name}");
    }

    let (code, text) = ask(&two.bind_b(), &two.b.path, &["status"]);
    assert_eq!(code, 0, "{text}");
    assert!(
        text.contains(&format!(
            "  landing: as last fetched {}\n",
            tailrace::render::utc(landed)
        )),
        "{text}"
    );
    for (name, at) in chains {
        assert!(
            text.lines().any(|l| l.starts_with(&format!("  {name} "))
                && l.ends_with(&format!(", as last read {}", tailrace::render::utc(at)))),
            "{name}: {text}"
        );
    }

    let address = serve_b(&two);
    let served = get(address, "/o/r/status.json");
    assert_eq!(unasked(served), unasked(status(&two.bind_b(), &two.b.path)));
    let page = body(address, "/o/r/status");
    assert!(
        page.contains(&format!(
            "landing: as last fetched {}",
            tailrace::render::utc(landed)
        )),
        "{page}"
    );
}
