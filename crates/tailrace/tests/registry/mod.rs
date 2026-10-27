//! The registries the registry store's tests run against (design D12): a
//! zot and a reference distribution, each started on a free loopback
//! port over a temporary directory and a config the test writes, by the
//! binary the devenv shell pins, waited on until `GET /v2/` answers, and
//! stopped when dropped; the store group's rig over a fresh repository of
//! one; a proxy in front of one that keeps every request it passes and
//! alters a response body when told to; and the cache manifest's
//! scenarios a registry plays through the store alone, which the run
//! against ECR pushes as the tools layer does.

#![allow(dead_code)]

use std::cell::{Cell, RefCell};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tailrace::profiles::store::registry::Registry;
use tailrace_conformance::seams::store::{settled, StoreRig};
use tailrace_conformance::Rig;
use tailrace_seams::{Digest, Held, Name, Not, Store, Why};
use tempfile::TempDir;

/// The one user the registries that ask for a credential take, and the
/// bcrypt of its password, as `htpasswd -nbB` writes it.
pub const USER: &str = "tester";
pub const PASSWORD: &str = "secret";
const HTPASSWD: &str = "tester:$2y$05$Os.CNqLFyqk1SjpFlJuGueXa1/FJ6j6H62Lehi3JZe1iZIEuG4zPm\n";

/// How long a registry is given to answer once started.
const STARTING: Duration = Duration::from_secs(30);

/// The registries the tools layer starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// zot, with the referrers API.
    Zot,
    /// The reference distribution, with none.
    Distribution,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Zot => "zot",
            Kind::Distribution => "the reference distribution",
        }
    }

    /// The pinned binary the devenv shell names; a panic naming the
    /// variable where it names none.
    fn binary(self) -> PathBuf {
        let (variable, tool) = match self {
            Kind::Zot => ("TAILRACE_TOOLS_ZOT", "zot"),
            Kind::Distribution => ("TAILRACE_TOOLS_DISTRIBUTION", "the reference distribution"),
        };
        match std::env::var_os(variable) {
            Some(path) if !path.is_empty() => PathBuf::from(path),
            _ => panic!(
                "{tool} is not in the shell ({variable} is unset): the tools layer runs in the \
                 devenv shell, which pins it"
            ),
        }
    }
}

/// A port on the loopback interface nothing listens on now, held by the
/// listener answered until it is dropped.
pub fn free_port() -> (u16, TcpListener) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a free loopback port");
    (listener.local_addr().unwrap().port(), listener)
}

/// A port for a registry: below the ephemeral ranges every host hands a
/// listener on port 0 from (32768 on Linux, 49152 on macOS), so no
/// listener of a test takes it before the registry binds it.
fn server_port() -> u16 {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    loop {
        let n = NEXT.fetch_add(1, Ordering::Relaxed) + u64::from(std::process::id()) * 131;
        let port = 30000 + (n % 2700) as u16;
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
}

/// A registry started for a test, stopped when dropped.
pub struct Server {
    pub kind: Kind,
    pub port: u16,
    child: Child,
    dir: TempDir,
}

impl Server {
    /// `kind` started with no credential asked.
    pub fn start(kind: Kind) -> Server {
        Server::started(kind, false)
    }

    /// `kind` started asking every request for [`USER`]'s credential, by
    /// `Basic`.
    pub fn asking(kind: Kind) -> Server {
        Server::started(kind, true)
    }

    fn started(kind: Kind, asks: bool) -> Server {
        let dir = TempDir::new().expect("a temporary directory");
        let root = dir.path().join("root");
        fs::create_dir_all(&root).unwrap();
        let htpasswd = dir.path().join("htpasswd");
        fs::write(&htpasswd, HTPASSWD).unwrap();
        for _ in 0..5 {
            let port = server_port();
            match Server::spawn(kind, asks, port, &dir) {
                Ok(child) => {
                    return Server {
                        kind,
                        port,
                        child,
                        dir,
                    }
                }
                Err(log) if log.contains("address already in use") => continue,
                Err(log) => panic!("{} did not start: {log}", kind.name()),
            }
        }
        panic!("{} found no free port in five tries", kind.name());
    }

    /// `kind` started on `port`, once `GET /v2/` answers anything at all;
    /// its log when it exits first or does not answer within
    /// [`STARTING`].
    fn spawn(kind: Kind, asks: bool, port: u16, dir: &TempDir) -> Result<Child, String> {
        let root = dir.path().join("root");
        let htpasswd = dir.path().join("htpasswd");
        let (config, args): (PathBuf, Vec<&str>) = match kind {
            Kind::Zot => {
                let mut http =
                    serde_json::json!({"address": "127.0.0.1", "port": port.to_string()});
                if asks {
                    http["auth"] = serde_json::json!({"htpasswd": {"path": htpasswd}});
                }
                let config = serde_json::json!({
                    "distSpecVersion": "1.1.1",
                    "storage": {"rootDirectory": root, "gc": false, "dedupe": false},
                    "http": http,
                    "log": {"level": "error"},
                });
                let path = dir.path().join("config.json");
                fs::write(&path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();
                (path, vec!["serve"])
            }
            Kind::Distribution => {
                let auth = if asks {
                    format!(
                        "auth:\n  htpasswd:\n    realm: tailrace\n    path: {}\n",
                        htpasswd.display()
                    )
                } else {
                    String::new()
                };
                let config = format!(
                    "version: 0.1\nlog:\n  level: error\n  accesslog:\n    disabled: true\n\
                     storage:\n  filesystem:\n    rootdirectory: {}\n{auth}\
                     http:\n  addr: 127.0.0.1:{port}\n",
                    root.display()
                );
                let path = dir.path().join("config.yml");
                fs::write(&path, config).unwrap();
                (path, vec!["serve"])
            }
        };
        let log_path = dir.path().join("log");
        let log = fs::File::create(&log_path).unwrap();
        let mut child = Command::new(kind.binary())
            .args(&args)
            .arg(&config)
            .stdin(Stdio::null())
            .stdout(log.try_clone().unwrap())
            .stderr(log)
            .spawn()
            .unwrap_or_else(|e| panic!("{} could not be started: {e}", kind.name()));
        let log = || fs::read_to_string(&log_path).unwrap_or_default();
        let start = Instant::now();
        let url = format!("http://127.0.0.1:{port}/v2/");
        while start.elapsed() < STARTING {
            if let Ok(Some(status)) = child.try_wait() {
                return Err(format!("it exited {status}: {}", log()));
            }
            if let Ok(_) | Err(ureq::Error::StatusCode(_)) = ureq::get(&url).call() {
                return Ok(child);
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = child.kill();
        let _ = child.wait();
        Err(format!(
            "it did not answer {url} within {STARTING:?}: {}",
            log()
        ))
    }

    /// The registry's host, as a repository names it.
    pub fn host(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }

    /// A repository no other asking has been given.
    pub fn repository(&self) -> String {
        static MADE: AtomicU64 = AtomicU64::new(0);
        format!(
            "{}/store-{}",
            self.host(),
            MADE.fetch_add(1, Ordering::Relaxed)
        )
    }

    /// A registry store over a fresh repository.
    pub fn store(&self) -> Registry {
        Registry::new(&self.repository()).expect("a repository text")
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The store group's rig: a registry store over a fresh repository of a
/// server the test started, and one at a port nothing listens on, which
/// [`StoreRig::unreachable`] makes the rig's store.
pub struct RegistryRig {
    _server: Rc<Server>,
    live: Registry,
    dead: Registry,
    held: RefCell<Option<TcpListener>>,
    down: Cell<bool>,
}

impl RegistryRig {
    pub fn new(server: &Rc<Server>) -> RegistryRig {
        let (port, held) = free_port();
        RegistryRig {
            live: server.store(),
            dead: Registry::new(&format!("127.0.0.1:{port}/store")).unwrap(),
            held: RefCell::new(Some(held)),
            _server: server.clone(),
            down: Cell::new(false),
        }
    }
}

impl Rig for RegistryRig {
    fn profile(&self) -> String {
        self.live.source().profile.clone()
    }
}

impl StoreRig for RegistryRig {
    fn store(&self) -> &dyn Store {
        if self.down.get() {
            &self.dead
        } else {
            &self.live
        }
    }

    /// The rig's store pointed at a port nothing listens on: every
    /// operation answers with what a client asking that port is told.
    fn unreachable(&self) -> String {
        self.held.borrow_mut().take();
        self.down.set(true);
        ureq::get(&format!("{}tags/list", self.dead.base()))
            .call()
            .expect_err("nothing listens on the port")
            .to_string()
    }

    /// The registry's bound: its tag reads may answer a moved name's
    /// former place for that long (C.1, ADR 0015).
    fn settles_within(&self) -> Duration {
        self.live.bound()
    }

    /// The registry's word: its names take no condition (C.1, ADR 0015).
    fn names_conditional(&self) -> bool {
        self.live.names_conditional()
    }
}

/// One request a [`Proxy`] passed: its method, its path, and the status
/// the registry answered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Passed {
    pub method: String,
    pub path: String,
    pub status: u16,
}

/// A proxy on a loopback port in front of a registry: it passes each
/// request on a connection of its own, keeps its method and path, and
/// answers a `GET` of a path it is told to alter with the body's bytes
/// each flipped.
pub struct Proxy {
    pub port: u16,
    passed: Arc<Mutex<Vec<Passed>>>,
    alter: Arc<Mutex<Option<String>>>,
}

impl Proxy {
    pub fn before(server: &Server) -> Proxy {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let passed = Arc::new(Mutex::new(Vec::new()));
        let alter: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let upstream = server.port;
        let (kept, altered) = (passed.clone(), alter.clone());
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { return };
                let (kept, altered) = (kept.clone(), altered.clone());
                std::thread::spawn(move || {
                    let _ = pass(stream, upstream, &kept, &altered);
                });
            }
        });
        Proxy {
            port,
            passed,
            alter,
        }
    }

    /// A registry store over `repository`, reached through the proxy.
    pub fn store(&self, repository: &str) -> Registry {
        let (_, path) = repository.split_once('/').unwrap();
        Registry::new(&format!("127.0.0.1:{}/{path}", self.port)).unwrap()
    }

    /// Every request passed since the last asking, in order.
    pub fn passed(&self) -> Vec<Passed> {
        std::mem::take(&mut *self.passed.lock().unwrap())
    }

    /// Every `GET` of a path ending in `suffix` answered with other bytes.
    pub fn alter(&self, suffix: &str) {
        *self.alter.lock().unwrap() = Some(suffix.to_string());
    }
}

fn pass(
    client: TcpStream,
    upstream: u16,
    passed: &Mutex<Vec<Passed>>,
    alter: &Mutex<Option<String>>,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(client.try_clone()?);
    let mut head = Vec::new();
    let mut length = 0usize;
    let mut first = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(());
        }
        if first.is_empty() {
            first = line.clone();
        }
        if line == "\r\n" {
            break;
        }
        let lower = line.to_ascii_lowercase();
        if let Some(value) = lower.strip_prefix("content-length:") {
            length = value.trim().parse().unwrap_or(0);
        }
        if lower.starts_with("connection:") {
            continue;
        }
        head.extend_from_slice(line.as_bytes());
    }
    head.extend_from_slice(b"connection: close\r\n\r\n");
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    let mut parts = first.split_whitespace();
    let (method, path) = (
        parts.next().unwrap_or_default().to_string(),
        parts.next().unwrap_or_default().to_string(),
    );
    let mut server = TcpStream::connect(("127.0.0.1", upstream))?;
    server.write_all(&head)?;
    server.write_all(&body)?;
    let mut answer = Vec::new();
    server.read_to_end(&mut answer)?;
    let status = String::from_utf8_lossy(&answer[..answer.len().min(16)])
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    passed.lock().unwrap().push(Passed {
        method: method.clone(),
        path: path.clone(),
        status,
    });
    let altering = alter.lock().unwrap().as_ref().is_some_and(|suffix| {
        method == "GET"
            && path
                .split('?')
                .next()
                .unwrap_or("")
                .ends_with(suffix.as_str())
    });
    if altering {
        if let Some(at) = answer.windows(4).position(|w| w == b"\r\n\r\n") {
            for byte in &mut answer[at + 4..] {
                *byte = !*byte;
            }
        }
    }
    let mut client = client;
    client.write_all(&answer)?;
    client.flush()
}

/// The cache manifest the registry store names `bytes` through, written
/// out here apart from the profile's own form, so a test that finds these
/// bytes in a registry proves the form (design D14).
pub fn cache_manifest(bytes: &[u8]) -> Vec<u8> {
    format!(
        r#"{{"schemaVersion":2,"mediaType":"application/vnd.oci.image.manifest.v1+json","artifactType":"application/vnd.agentplot.tailrace.cache.v1","config":{{"mediaType":"application/vnd.oci.empty.v1+json","digest":"{}","size":2}},"layers":[{{"mediaType":"application/octet-stream","digest":"{}","size":{}}}]}}"#,
        Digest::of(b"{}"),
        Digest::of(bytes),
        bytes.len()
    )
    .into_bytes()
}

/// The cache manifest's scenarios a registry plays through the store
/// alone (spec registry-store, design D14): bytes named from nothing
/// resolve to their digest, and so do two names at the same bytes; a
/// name at bytes moved with their digest as expected to other bytes; the
/// cache manifest's own digest read as bytes, never named, and the bytes'
/// digest with no claims; a digest held neither way absent. A moved
/// name is read until it settles within `within`, the registry's bound,
/// as the store group reads one (C.1, 36). A move from bytes the name has
/// left is refused only by the check's best effort, so it is proven
/// where every read shows a move at once, in `registry_store.rs`, and
/// never against ECR (C.1, ADR 0015).
pub fn cache_entries(store: &dyn Store, within: Duration) {
    let (entry, other): (&[u8], &[u8]) = (b"a cache entry", b"another cache entry");
    let bytes = store.put_bytes(entry).unwrap().digest;
    let moved = store.put_bytes(other).unwrap().digest;
    let (one, two) = (Name::new("cache/one"), Name::new("cache/two"));
    for name in [&one, &two] {
        store.name(name, &bytes, None).unwrap();
    }
    let resolved =
        |name: &Name, to: &Digest, was: &[Option<&Digest>]| settled(store, within, name, to, was);
    for name in [&one, &two] {
        assert_eq!(
            resolved(name, &bytes, &[None]).as_ref(),
            Ok(&bytes),
            "{name}"
        );
    }

    store.name(&two, &moved, Some(&bytes)).unwrap();
    let was = [None, Some(&bytes)];
    assert_eq!(resolved(&two, &moved, &was).as_ref(), Ok(&moved));

    let manifest = cache_manifest(entry);
    let own = Digest::of(&manifest);
    assert_eq!(store.read(&own), Ok(Held::Bytes(manifest)));
    assert_eq!(store.claims_of(&bytes), Ok(vec![]));
    match store.name(&one, &own, Some(&bytes)) {
        Err(Not::Refused(r)) => {
            assert_eq!(r.why, Why::Said);
            assert!(r.said.unwrap().contains(bytes.as_str()));
        }
        answer => panic!("a name at a cache manifest's own digest answered {answer:?}"),
    }
    assert_eq!(resolved(&one, &bytes, &[None]).as_ref(), Ok(&bytes));

    let missing = Digest::of(b"bytes no name names");
    let three = Name::new("cache/three");
    match store.name(&three, &missing, None) {
        Err(Not::Absent(a)) => assert_eq!(a.what, missing.to_string()),
        answer => panic!("a name at a digest not held answered {answer:?}"),
    }
    assert!(matches!(store.resolve(&three), Err(Not::Absent(_))));
}
