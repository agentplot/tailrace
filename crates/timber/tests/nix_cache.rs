//! Spec nix-cache-target, the proof (design D12): an attic server started
//! for the test, monolithic, its database, storage and token secret in
//! the test's directory; a cache made public in it by a token of the
//! server's own, so another host reads it with no token, as a fleet's
//! machines read theirs; and the environment's token, for push and pull on
//! it alone, minted with `atticadm make-token` and written into an attic
//! configuration under an `XDG_CONFIG_HOME` of the test's own, the
//! environment's own place, which only attic's process is handed (45,
//! rule core.21). The build host's part: a closure of two store paths, a
//! root whose output names a leaf, built with the host's Nix; its root's
//! line packed by the file projection into a layout store with its index,
//! and applied through the Nix cache target with `Verified` as the core
//! makes it (42, 90, C.12). Another host's part: a fresh Nix store in the
//! test's directory realises the root from the cache's address alone,
//! trusting only the cache's key, and holds the leaf after. Then the
//! target group runs over a rig of the profile against the same server,
//! holding no marker, its foreign settings a `token` key (45, 58, 88, 89,
//! 119; design D9, D10). Needs attic, atticd and atticadm, which the
//! devenv shell pins, and the host's Nix, so it is in the tools layer.

#![cfg(feature = "tools")]

use std::cell::Cell;
use std::fs;
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output as Ran, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use base64::Engine;
use serde_json::json;
use ssh_key::rand_core::{OsRng, RngCore};
use tempfile::TempDir;
use timber::profiles::projection::file::FileProjection;
use timber::profiles::store::layout::Layout;
use timber::profiles::target::nix_cache::{self, NixCache};
use timber_conformance::seams::target::{self, TargetRig};
use timber_conformance::Rig;
use timber_core::paths_hold;
use timber_seams::{
    Artifact, BuildId, ChannelId, ChannelName, Component, ComponentName, Digest, Environment,
    EnvironmentName, IndexMeta, Manifest, Output, OutputName, PackAt, Projection, Store, Target,
    Verified, Version,
};

/// The server as the environment's attic configuration names it, and the
/// cache made on it: the table's `attic` is `fleet:fleet`.
const SERVER: &str = "fleet";
const CACHE: &str = "fleet";

/// The machine's component, its one output, a closure file, and the
/// version published.
const COMPONENT: &str = "fleet-srv";
const VERSION: &str = "0.4.0";

/// A closure of two store paths built with the host's Nix and nothing
/// else: a leaf, and a root whose output names it, so the root refers to
/// the leaf. `/bin/sh` is every builder's, the sandbox's among them.
const CLOSURE: &str = r#"let
  sh = name: script: derivation {
    inherit name;
    system = builtins.currentSystem;
    builder = "/bin/sh";
    args = [ "-c" script ];
  };
  leaf = sh "timber-nix-cache-leaf" "echo timber-nix-cache-leaf > $out";
in
sh "timber-nix-cache-root" "echo ${leaf} > $out"
"#;

/// How long atticd is given to answer once started.
const STARTING: Duration = Duration::from_secs(30);

/// The pinned binary the devenv shell names by `variable`; a panic naming
/// the variable where it names none.
fn pinned(variable: &str) -> PathBuf {
    match std::env::var_os(variable) {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => panic!(
            "{variable} is unset: the tools layer runs in the devenv shell, which pins attic"
        ),
    }
}

/// `program` run with `args`, its output whatever its exit; a panic
/// naming it where it cannot be started.
fn run(program: &mut Command) -> Ran {
    program
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("{program:?} could not be started: {e}"))
}

/// `program`'s stdout where it exits 0, else a panic with its words.
fn succeeds(program: &mut Command) -> String {
    let ran = run(program);
    assert!(
        ran.status.success(),
        "{program:?} exited {}: {}{}",
        ran.status,
        String::from_utf8_lossy(&ran.stdout),
        String::from_utf8_lossy(&ran.stderr)
    );
    String::from_utf8(ran.stdout).expect("the output is UTF-8")
}

/// A port for atticd: below the ephemeral ranges every host hands a
/// listener on port 0 from (32768 on Linux, 49152 on macOS), and below
/// the registries' (tests/registry), so no listener of a test takes it
/// before atticd binds it.
fn server_port() -> u16 {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    loop {
        let n = NEXT.fetch_add(1, Ordering::Relaxed) + u64::from(std::process::id()) * 131;
        let port = 27000 + (n % 2700) as u16;
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
}

/// An attic server started for the test, monolithic, everything it keeps
/// under `dir`; stopped when dropped.
struct Atticd {
    dir: PathBuf,
    port: u16,
    child: Child,
}

impl Atticd {
    /// atticd started over a database, a store and a token secret of its
    /// own under `dir`, once it answers.
    fn start(dir: &Path) -> Atticd {
        fs::create_dir_all(dir.join("storage")).unwrap();
        let mut secret = [0u8; 48];
        OsRng.fill_bytes(&mut secret);
        let secret = base64::engine::general_purpose::STANDARD.encode(secret);
        for _ in 0..5 {
            let port = server_port();
            let config = format!(
                "listen = \"127.0.0.1:{port}\"\n\
                 api-endpoint = \"http://127.0.0.1:{port}/\"\n\
                 \n\
                 [database]\n\
                 url = \"sqlite://{db}?mode=rwc\"\n\
                 \n\
                 [storage]\n\
                 type = \"local\"\n\
                 path = \"{storage}\"\n\
                 \n\
                 [chunking]\n\
                 nar-size-threshold = 0\n\
                 min-size = 16384\n\
                 avg-size = 65536\n\
                 max-size = 262144\n\
                 \n\
                 [jwt.signing]\n\
                 token-hs256-secret-base64 = \"{secret}\"\n",
                db = dir.join("server.db").display(),
                storage = dir.join("storage").display(),
            );
            fs::write(dir.join("server.toml"), config).unwrap();
            match Atticd::spawn(dir, port) {
                Ok(child) => {
                    return Atticd {
                        dir: dir.to_path_buf(),
                        port,
                        child,
                    }
                }
                Err(log) if log.contains("already in use") => continue,
                Err(log) => panic!("atticd did not start: {log}"),
            }
        }
        panic!("atticd found no free port in five tries");
    }

    /// atticd started on `port`, once `GET /` answers anything at all;
    /// its log where it exits first or does not answer within
    /// [`STARTING`].
    fn spawn(dir: &Path, port: u16) -> Result<Child, String> {
        let log_path = dir.join("log");
        let log = fs::File::create(&log_path).unwrap();
        let mut child = Command::new(pinned("TIMBER_TOOLS_ATTICD"))
            .args(["--mode", "monolithic", "--config"])
            .arg(dir.join("server.toml"))
            .stdin(Stdio::null())
            .stdout(log.try_clone().unwrap())
            .stderr(log)
            .spawn()
            .unwrap_or_else(|e| panic!("atticd could not be started: {e}"));
        let log = || fs::read_to_string(&log_path).unwrap_or_default();
        let start = Instant::now();
        let url = format!("http://127.0.0.1:{port}/");
        while start.elapsed() < STARTING {
            if let Ok(Some(status)) = child.try_wait() {
                return Err(format!("it exited {status}: {}", log()));
            }
            if let Ok(_) | Err(ureq::Error::StatusCode(_)) = ureq::get(&url).call() {
                return Ok(child);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let _ = child.kill();
        let _ = child.wait();
        Err(format!("it did not answer within {STARTING:?}: {}", log()))
    }

    fn endpoint(&self) -> String {
        format!("http://127.0.0.1:{}/", self.port)
    }

    /// The cache's address, as a Nix store names a binary cache.
    fn cache_address(&self) -> String {
        format!("http://127.0.0.1:{}/{CACHE}", self.port)
    }

    /// A token for `sub` holding `permissions`, as `atticadm make-token`
    /// mints it under the server's secret.
    fn token(&self, sub: &str, permissions: &[&str]) -> String {
        let mut make = Command::new(pinned("TIMBER_TOOLS_ATTICADM"));
        make.arg("--config")
            .arg(self.dir.join("server.toml"))
            .args(["make-token", "--sub", sub, "--validity", "1 day"]);
        for permission in permissions {
            make.args([*permission, CACHE]);
        }
        succeeds(&mut make).trim().to_string()
    }

    /// Whether the cache holds the store path `path`, by its narinfo.
    fn holds(&self, path: &str) -> bool {
        let hash = path
            .rsplit('/')
            .next()
            .and_then(|base| base.split('-').next())
            .unwrap_or_else(|| panic!("{path} is no store path"));
        let url = format!("{}/{hash}.narinfo", self.cache_address());
        match ureq::get(&url).call() {
            Ok(_) => true,
            Err(ureq::Error::StatusCode(404)) => false,
            Err(e) => panic!("GET {url}: {e}"),
        }
    }
}

impl Drop for Atticd {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// An attic configuration under `home`, as `XDG_CONFIG_HOME` names it,
/// naming the server as [`SERVER`] and holding `token`.
fn attic_config(home: &Path, server: &Atticd, token: &str) {
    fs::create_dir_all(home.join("attic")).unwrap();
    fs::write(
        home.join("attic/config.toml"),
        format!(
            "[servers.{SERVER}]\nendpoint = \"{}\"\ntoken = \"{token}\"\n",
            server.endpoint()
        ),
    )
    .unwrap();
}

/// The fleet the test stands up: the server, its cache and its key, the
/// environment's token, attic as the host runs it, the closure built and
/// the version publishing it.
struct Fleet {
    dir: TempDir,
    server: Atticd,
    /// The cache's public key, as a Nix store trusts it.
    key: String,
    /// The tokens minted: the server's own, which made the cache, and the
    /// environment's, for push and pull on it.
    tokens: [String; 2],
    /// attic as this host runs it: a script that records the command
    /// line and the environment timber ran it with, then hands attic its
    /// configuration under the test's own `XDG_CONFIG_HOME` and runs it.
    attic: PathBuf,
    root: String,
    leaf: String,
    layout: PathBuf,
    verified: Verified,
}

impl Fleet {
    fn new() -> Fleet {
        let dir = TempDir::new().expect("a temporary directory");
        // Nix refuses a store under a path through a link, as macOS's
        // temporary directory is.
        let at = fs::canonicalize(dir.path()).unwrap();
        let server = Atticd::start(&at.join("server"));
        let admin = server.token("timber-test", &["--create-cache", "--pull"]);
        let token = server.token("fleet-srv-cache", &["--push", "--pull"]);
        let made = at.join("server/client");
        attic_config(&made, &server, &admin);
        let client = pinned("TIMBER_TOOLS_ATTIC");
        let cache = format!("{SERVER}:{CACHE}");
        succeeds(
            Command::new(&client)
                .env("XDG_CONFIG_HOME", &made)
                .args(["cache", "create", "--public", &cache]),
        );
        // attic prints the cache's information on its stderr.
        let info = run(Command::new(&client)
            .env("XDG_CONFIG_HOME", &made)
            .args(["cache", "info", &cache]));
        let info = format!(
            "{}{}",
            String::from_utf8_lossy(&info.stdout),
            String::from_utf8_lossy(&info.stderr)
        );
        let key = info
            .lines()
            .find_map(|line| line.trim().strip_prefix("Public Key: "))
            .unwrap_or_else(|| panic!("attic cache info named no public key: {info}"))
            .to_string();

        let own = at.join("attic-home");
        attic_config(&own, &server, &token);
        let ran = at.join("ran");
        fs::create_dir_all(&ran).unwrap();
        let attic = at.join("bin/attic");
        fs::create_dir_all(attic.parent().unwrap()).unwrap();
        fs::write(
            &attic,
            format!(
                "#!/bin/sh\n\
                 for arg in \"$@\"; do printf '%s\\n' \"$arg\"; done > '{ran}/attic.args'\n\
                 env > '{ran}/attic.env'\n\
                 XDG_CONFIG_HOME='{own}'\n\
                 export XDG_CONFIG_HOME\n\
                 exec '{client}' \"$@\"\n",
                ran = ran.display(),
                own = own.display(),
                client = client.display(),
            ),
        )
        .unwrap();
        fs::set_permissions(&attic, fs::Permissions::from_mode(0o755)).unwrap();

        let (root, leaf) = build(&at.join("build"));
        let layout = at.join("layout");
        let verified = publish(&layout, &at.join("build/closure"), &root);
        Fleet {
            dir,
            server,
            key,
            tokens: [admin, token],
            attic,
            root,
            leaf,
            layout,
            verified,
        }
    }

    fn at(&self) -> PathBuf {
        fs::canonicalize(self.dir.path()).unwrap()
    }

    /// The Nix cache target over a store binding of its own, opened and
    /// never made, running attic as this host runs it and the host's
    /// nix-store.
    fn target(&self) -> NixCache {
        let store = Layout::open(&self.layout).expect("the layout the test published into");
        NixCache::with_programs(Box::new(store), &self.attic, "nix-store")
    }

    /// The cache environment `name` of the machine's component, following
    /// `beta`, its table naming the cache and the closure.
    fn environment(&self, name: &str) -> Environment {
        Environment {
            name: EnvironmentName::new(name),
            channel: ChannelId::new(
                Some(ComponentName::new(COMPONENT)),
                ChannelName::new("beta"),
            ),
            waits: false,
            settings: json!({ "attic": format!("{SERVER}:{CACHE}"), "closures": [COMPONENT] }),
        }
    }

    /// What attic was last run with, as timber ran it: its arguments one
    /// to a line, and its environment as `env` printed it.
    fn attic_ran(&self) -> (Vec<String>, String) {
        let ran = self.at().join("ran");
        let args = fs::read_to_string(ran.join("attic.args")).expect("attic ran");
        let env = fs::read_to_string(ran.join("attic.env")).expect("attic ran");
        (args.lines().map(str::to_string).collect(), env)
    }
}

/// The closure built with the host's Nix under `dir`, its root held by a
/// link there; the root's store path and the leaf's, which it refers to.
fn build(dir: &Path) -> (String, String) {
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join("closure.nix"), CLOSURE).unwrap();
    succeeds(
        Command::new("nix-build")
            .arg(dir.join("closure.nix"))
            .arg("--out-link")
            .arg(dir.join("result")),
    );
    let root = fs::read_link(dir.join("result"))
        .expect("nix-build linked its output")
        .display()
        .to_string();
    let references = succeeds(Command::new("nix-store").args(["--query", "--references", &root]));
    let references: Vec<&str> = references.lines().collect();
    let [leaf] = references.as_slice() else {
        panic!("the root {root} refers to {references:?}, not to its leaf alone");
    };
    assert!(leaf.ends_with("-timber-nix-cache-leaf"), "{leaf}");
    (root, leaf.to_string())
}

/// The closure file naming `root`, written at `file` as the machine's
/// build task leaves it, packed by the file projection at [`VERSION`] and
/// put in a layout store made at `layout` as a publish puts it: its
/// bytes, its artifact and the version's index naming it; the version
/// verified, as the core makes it after its verify (42).
fn publish(layout: &Path, file: &Path, root: &str) -> Verified {
    fs::write(file, format!("{root}\n")).unwrap();
    let store = Layout::create(layout).expect("a layout made");
    let output = Output {
        name: OutputName::new(COMPONENT),
        files: vec![file.display().to_string()],
    };
    let component = Component {
        name: ComponentName::new(COMPONENT),
        outputs: vec![output.name.clone()],
        paths: vec!["machines/srv/".into()],
        prefix: format!("{COMPONENT}-v"),
    };
    let holds = paths_hold(&component.paths).unwrap();
    let version = Version::new(VERSION);
    let packed = FileProjection::new()
        .pack(
            &output,
            &BuildId::new("b7e1"),
            Some(&PackAt {
                version: &version,
                component: &component,
                outputs: std::slice::from_ref(&output),
                holds: &holds,
            }),
        )
        .unwrap_or_else(|not| panic!("the closure file packed: {not}"));
    let meta = packed.meta.expect("the file projection names its artifact");
    let layer = store.put_bytes(&packed.bytes).unwrap().digest;
    let digest = store
        .put_artifact(&Manifest::Artifact {
            meta: meta.clone(),
            layers: vec![layer],
        })
        .unwrap()
        .digest;
    let index = IndexMeta::new(
        component.name.clone(),
        version.clone(),
        vec![Artifact { digest, meta }],
    )
    .unwrap();
    let index = store.put_artifact(&Manifest::Index(index)).unwrap().digest;
    Verified::new(index, version)
}

/// Every file under `dir` but those under `except`, links unfollowed.
fn files(dir: &Path, except: &[PathBuf], into: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if except.contains(&path) {
            continue;
        }
        let kind = fs::symlink_metadata(&path).unwrap().file_type();
        if kind.is_dir() {
            files(&path, except, into);
        } else if kind.is_file() {
            into.push(path);
        }
    }
}

/// `nix` run as another host runs it: none of this host's configuration
/// read, its cache of what binary caches hold its own under `at`, so the
/// only key it trusts and the only source it reads are its arguments'.
fn fresh_nix(at: &Path) -> Command {
    let conf = at.join("fresh-conf");
    fs::create_dir_all(&conf).unwrap();
    fs::write(conf.join("nix.conf"), "").unwrap();
    let mut nix = Command::new("nix");
    nix.env_remove("NIX_CONFIG")
        .env("NIX_CONF_DIR", &conf)
        .env("NIX_USER_CONF_FILES", conf.join("nix.conf"))
        .env("XDG_CACHE_HOME", at.join("fresh-cache"))
        .args(["--extra-experimental-features", "nix-command"]);
    nix
}

/// The proof: the version applied through the Nix cache target from the
/// host that built its closure, with the token in attic's own
/// configuration alone; then a fresh Nix store realises the root and its
/// leaf from the cache alone (C.12, 42, 45, 90; design D12).
fn realised_from_the_cache_alone(fleet: &Fleet) {
    let server = &fleet.server;
    let at = fleet.at();
    assert!(
        !server.holds(&fleet.root) && !server.holds(&fleet.leaf),
        "the cache held the closure before the apply"
    );

    let target = fleet.target();
    let environment = fleet.environment("cache");
    target
        .apply(&environment, &fleet.verified)
        .unwrap_or_else(|not| panic!("the apply: {not}"));
    assert!(
        server.holds(&fleet.root) && server.holds(&fleet.leaf),
        "the cache holds the root and its leaf after the apply"
    );

    // The token stays attic's: its command line names the cache and the
    // root and nothing else, and neither its environment as timber set it
    // nor anything timber wrote holds a token or names attic's
    // configuration (45, rule core.21).
    let (args, env) = fleet.attic_ran();
    assert_eq!(
        args,
        [
            "push".to_string(),
            format!("{SERVER}:{CACHE}"),
            fleet.root.clone()
        ],
        "attic was given another command line"
    );
    let own = at.join("attic-home");
    assert!(
        !env.contains(&own.display().to_string()),
        "timber handed attic its configuration: {env}"
    );
    let mut written = Vec::new();
    files(&at, &[at.join("server"), own], &mut written);
    assert!(
        written.iter().any(|file| file.starts_with(&fleet.layout)),
        "no file of the store timber wrote was read: {written:?}"
    );
    for token in &fleet.tokens {
        assert!(
            !env.contains(token.as_str()),
            "attic's environment holds a token"
        );
        for file in &written {
            let bytes = fs::read(file).unwrap();
            assert!(
                !bytes
                    .windows(token.len())
                    .any(|window| window == token.as_bytes()),
                "{} holds a token",
                file.display()
            );
        }
    }

    // Another host's part: a fresh store, the cache its only source and
    // the cache's key the only one it trusts.
    let fresh = at.join("fresh");
    succeeds(
        fresh_nix(&at)
            .args(["copy", "--option", "substituters", ""])
            .args(["--option", "trusted-public-keys", &fleet.key])
            .args(["--from", &server.cache_address()])
            .arg("--to")
            .arg(format!("local?root={}", fresh.display()))
            .arg(&fleet.root),
    );
    for path in [&fleet.root, &fleet.leaf] {
        let realised = fresh.join(path.trim_start_matches('/'));
        assert_eq!(
            fs::read(&realised).unwrap_or_else(|e| panic!("the fresh store holds no {path}: {e}")),
            fs::read(path).unwrap(),
            "the fresh store realised other bytes at {path}"
        );
    }
}

/// The target group's rig over the Nix cache target and the test's
/// server: the target holds no marker, every environment names the
/// closure the version published, and its foreign settings add a `token`
/// (119, 45; design D9, D10).
struct CacheRig {
    target: NixCache,
    verified: Verified,
    settings: serde_json::Value,
    environments: Cell<u32>,
}

impl CacheRig {
    fn new(fleet: &Fleet) -> CacheRig {
        CacheRig {
            target: fleet.target(),
            verified: fleet.verified.clone(),
            settings: fleet.environment("cache").settings,
            environments: Cell::new(0),
        }
    }

    fn next(&self) -> Environment {
        self.environments.set(self.environments.get() + 1);
        Environment {
            name: EnvironmentName::new(format!("cache-{}", self.environments.get())),
            channel: ChannelId::new(
                Some(ComponentName::new(COMPONENT)),
                ChannelName::new("beta"),
            ),
            waits: false,
            settings: self.settings.clone(),
        }
    }
}

impl Rig for CacheRig {
    fn profile(&self) -> String {
        nix_cache::PROFILE.into()
    }
}

impl TargetRig for CacheRig {
    fn target(&self) -> &dyn Target {
        &self.target
    }

    /// The version the test published, whatever `digest` the suite
    /// names: a store holds an index only at its own digest.
    fn verified(&self, _digest: &Digest) -> Verified {
        self.verified.clone()
    }

    /// The cache is pushed to, handed the head at desired, so `head` is
    /// left unread.
    fn environment(&self, _head: Option<&Digest>) -> Environment {
        self.next()
    }

    /// No environment of a cache can be read, for the cache keeps no
    /// marker of what was pushed to it.
    fn unreadable_environment(&self) -> (Environment, String) {
        (
            self.next(),
            format!(
                "the Nix binary cache {SERVER}:{CACHE} holds store paths by their hash and keeps \
                 no marker of what was pushed to it"
            ),
        )
    }

    fn holds_marker(&self) -> bool {
        false
    }

    fn foreign_settings(&self) -> (Environment, String) {
        let mut environment = self.next();
        environment.settings["token"] = json!("a credential");
        (environment, "token".into())
    }
}

#[test]
fn a_closure_built_here_is_realised_from_the_cache_alone_and_the_target_group_passes() {
    let fleet = Fleet::new();
    realised_from_the_cache_alone(&fleet);

    let report = timber_conformance::run(target::GROUP, &target::scenarios(), &|| {
        Box::new(CacheRig::new(&fleet)) as Box<dyn TargetRig>
    });
    report.assert();
    let inapplicable: Vec<&str> = report.not_applicable.iter().map(|n| n.scenario).collect();
    assert_eq!(inapplicable, ["converged and reported"], "{report}");
}
