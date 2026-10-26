//! What the host keeps for its one serving process (62, 114, 132; design
//! D5, D7): the state directory, `$XDG_STATE_HOME/tailrace` or
//! `~/.local/state/tailrace`; in it the file of the repositories a
//! checkout named, owner and name with the path of the checkout that
//! last named each, a TOML file any reader opens; and the serving
//! process found at its address, or started where none answers.
//!
//! The file is read on each request and written only by a checkout
//! naming its repository, under a lock of its own, by a temporary file
//! renamed over it, so a reader never sees half of it. It is not rebuilt
//! from the store and git: the next checkout to name its repository
//! writes its entry again, and until then the process answers that
//! repository absent, never a wrong answer.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read as _, Write as _};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tailrace_core::{History, RepositoryName};
use tailrace_seams::{Answer, Not, Seam, Source};

use crate::bindings::{self, For};
use crate::Binding;

/// The file of the repositories a checkout named, in the state directory.
pub const REPOSITORIES: &str = "repositories";

/// The lock a checkout holds while it names its repository.
const NAMING: &str = "repositories.lock";

/// The lock the serving process holds for as long as it runs.
pub const SERVING: &str = "serve.lock";

/// How long an asker waits for any answer at the address.
const ASKED: Duration = Duration::from_secs(1);

/// How often an asker that started the process asks again, and for how
/// long.
const AGAIN: Duration = Duration::from_millis(100);
const STARTING: Duration = Duration::from_secs(5);

/// The host as the serving process and its askers read it: the state
/// directory, the bindings file's path, the home its `~/` paths are
/// expanded against, and the data directory the given layout is under
/// (118). `tailrace serve` and `tailrace status --page` read
/// them from the environment once; a test gives temporary directories,
/// so it reads no file of the host running it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Host {
    pub state: PathBuf,
    pub config: PathBuf,
    pub home: Option<PathBuf>,
    pub data: Option<PathBuf>,
}

impl Host {
    /// The host this process's `XDG_STATE_HOME`, `XDG_CONFIG_HOME`,
    /// `XDG_DATA_HOME` and `HOME` place; absent naming them where none places a directory.
    pub fn of_environment() -> Answer<Host> {
        let home = std::env::var_os("HOME")
            .filter(|home| !home.is_empty())
            .map(PathBuf::from);
        let unplaced = |what: &str| {
            Not::absent(
                &Source::new(Seam::Host, "serve"),
                format!("{what}: neither its variable nor HOME is set"),
            )
        };
        let state = state(
            std::env::var_os("XDG_STATE_HOME").as_deref(),
            home.as_deref(),
        )
        .ok_or_else(|| unplaced("state directory, XDG_STATE_HOME"))?;
        let config = bindings::path(
            std::env::var_os("XDG_CONFIG_HOME").as_deref(),
            home.as_deref(),
        )
        .ok_or_else(|| unplaced("bindings file, XDG_CONFIG_HOME"))?;
        let data = std::env::var_os("XDG_DATA_HOME")
            .filter(|data| !data.is_empty())
            .map(PathBuf::from);
        Ok(Host {
            state,
            config,
            home,
            data,
        })
    }

    /// Status's binding in the checkout at `checkout`, from this host's
    /// bindings file or, where it has none, as 118 binds such a host.
    pub fn bind(&self, checkout: &Path) -> Answer<Binding> {
        bindings::bind(
            &self.config,
            self.home.as_deref(),
            self.data.as_deref().map(Path::as_os_str),
            checkout,
            For::Status,
        )
    }

    fn repositories(&self) -> PathBuf {
        self.state.join(REPOSITORIES)
    }
}

/// The state directory: `tailrace` under `xdg_state_home` where it is set
/// and not empty, else `~/.local/state/tailrace` under `home`.
pub fn state(xdg_state_home: Option<&OsStr>, home: Option<&Path>) -> Option<PathBuf> {
    let state = match xdg_state_home {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => home?.join(".local/state"),
    };
    Some(state.join("tailrace"))
}

/// One repository's entry: the checkout that last named it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    checkout: PathBuf,
}

/// The file as it is held, keyed by `owner/name`.
type Named = BTreeMap<String, Entry>;

fn source(path: &Path) -> Source {
    Source::host(path)
}

/// The file at `path`; empty where there is none, unreadable with the
/// system's or the parser's words otherwise.
fn read(path: &Path) -> Answer<Named> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Named::new()),
        Err(e) => {
            return Err(Not::unreadable(
                &source(path),
                path.display().to_string(),
                Some(e.to_string()),
            ))
        }
    };
    toml::from_str(&text).map_err(|e| {
        Not::unreadable(
            &source(path),
            path.display().to_string(),
            Some(e.to_string()),
        )
    })
}

fn written(path: &Path, e: io::Error) -> Not {
    Not::unreadable(
        &source(path),
        format!("{} as written", path.display()),
        Some(e.to_string()),
    )
}

/// 132: `repository` named to the host's serving process by the checkout
/// at `checkout`, the one entry for it naming that checkout; under the
/// naming lock, held for the read and the write.
pub fn name(host: &Host, repository: &RepositoryName, checkout: &Path) -> Answer<()> {
    fs::create_dir_all(&host.state).map_err(|e| written(&host.state, e))?;
    let lock_path = host.state.join(NAMING);
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&lock_path)
        .map_err(|e| written(&lock_path, e))?;
    lock.lock().map_err(|e| written(&lock_path, e))?;
    let path = host.repositories();
    let mut named = read(&path)?;
    named.insert(
        repository.to_string(),
        Entry {
            checkout: checkout.to_path_buf(),
        },
    );
    let text = toml::to_string(&named).expect("the file of repositories is plain data");
    let beside = host
        .state
        .join(format!("{REPOSITORIES}.{}", std::process::id()));
    fs::write(&beside, text).map_err(|e| written(&beside, e))?;
    fs::rename(&beside, &path).map_err(|e| written(&path, e))?;
    drop(lock);
    Ok(())
}

/// What a repository none named answers.
fn unnamed(path: &Path, repository: &RepositoryName) -> Not {
    Not::absent(
        &source(path),
        format!(
            "repository {repository}: no command run in a checkout of it on this host has named it"
        ),
    )
}

/// The checkout `entry` names for `repository`, unreadable naming its
/// path where it is no repository.
fn checkout(repository: &RepositoryName, entry: &Entry) -> Answer<PathBuf> {
    match History::open(&entry.checkout) {
        Ok(_) => Ok(entry.checkout.clone()),
        Err(not) => {
            let said = match &not {
                Not::Unreadable(u) => u.said.clone(),
                _ => None,
            };
            Err(Not::unreadable(
                &source(&entry.checkout),
                format!(
                    "the checkout of {repository} at {}",
                    entry.checkout.display()
                ),
                said,
            ))
        }
    }
}

/// 58, 132: the checkout that last named `repository`: absent where none
/// did, unreadable naming the file with the parser's words where it
/// cannot be read, and naming the path where the checkout is gone. It
/// takes no lock and writes nothing.
pub fn checkout_of(host: &Host, repository: &RepositoryName) -> Answer<PathBuf> {
    let path = host.repositories();
    let named = read(&path)?;
    let entry = named
        .get(&repository.to_string())
        .ok_or_else(|| unnamed(&path, repository))?;
    checkout(repository, entry)
}

/// 132: every repository the file names, ordered by owner and name, each
/// with its checkout or why it cannot be read; none where there is no
/// file, and unreadable with the parser's words where it cannot be read.
pub fn named(host: &Host) -> Answer<Vec<(RepositoryName, Answer<PathBuf>)>> {
    let path = host.repositories();
    let mut all: Vec<(RepositoryName, Answer<PathBuf>)> = Vec::new();
    for (key, entry) in read(&path)? {
        let repository = key
            .split_once('/')
            .map(|(owner, name)| RepositoryName {
                owner: owner.to_string(),
                name: name.to_string(),
            })
            .filter(|r| !r.owner.is_empty() && !r.name.is_empty() && !r.name.contains('/'))
            .ok_or_else(|| {
                Not::unreadable(
                    &source(&path),
                    path.display().to_string(),
                    Some(format!("{key:?} names no owner and name")),
                )
            })?;
        let at = checkout(&repository, &entry);
        all.push((repository, at));
    }
    all.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(all)
}

/// The serving process as it names itself at its address (114).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Serving {
    pub serving: String,
    pub revision: String,
    pub pid: u32,
}

/// The document a serving process answers `/serving.json` with.
pub const SERVING_NAME: &str = "tailrace";

fn asker(address: SocketAddr) -> Source {
    Source::new(Seam::Host, format!("serve at http://{address}"))
}

/// What one ask of `/serving.json` at `address` found.
enum Found {
    Serving(Serving),
    Nothing,
}

/// One `GET /serving.json` at `address`, waiting a second for any
/// answer: nothing where the connection is refused, unreadable naming
/// the address and what answered where it is not tailrace's document.
fn ask(address: SocketAddr) -> Answer<Found> {
    let mut stream = match TcpStream::connect_timeout(&address, ASKED) {
        Ok(stream) => stream,
        Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => return Ok(Found::Nothing),
        Err(e) => {
            return Err(Not::unreadable(
                &asker(address),
                format!("the serving process at http://{address}"),
                Some(e.to_string()),
            ))
        }
    };
    let unread = |said: String| {
        Not::unreadable(
            &asker(address),
            format!("the serving process at http://{address}, which answered as something else"),
            Some(said),
        )
    };
    let mut answered = Vec::new();
    let asked = stream
        .set_read_timeout(Some(ASKED))
        .and_then(|()| {
            stream.write_all(
                format!(
                    "GET /serving.json HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
                )
                .as_bytes(),
            )
        })
        .and_then(|()| stream.read_to_end(&mut answered));
    let answered = String::from_utf8_lossy(&answered).into_owned();
    if let Err(e) = asked {
        return Err(unread(format!("{e}: {answered}")));
    }
    let (head, body) = answered.split_once("\r\n\r\n").unwrap_or((&answered, ""));
    let ok = head
        .lines()
        .next()
        .is_some_and(|line| line.split(' ').nth(1) == Some("200"));
    match serde_json::from_str::<Serving>(body) {
        Ok(serving) if ok && serving.serving == SERVING_NAME => Ok(Found::Serving(serving)),
        _ => Err(unread(answered)),
    }
}

/// 114: the serving process at `address`, found by its `/serving.json`;
/// where nothing answers, `start` is run once and the address asked
/// again every 100 ms for up to five seconds. Anything else answering is
/// unreadable naming the address and what it answered, and starts
/// nothing; a process started that has not answered in five seconds is
/// unreadable saying so.
pub fn find_or_start(address: SocketAddr, start: impl FnOnce() -> Answer<()>) -> Answer<Serving> {
    if let Found::Serving(serving) = ask(address)? {
        return Ok(serving);
    }
    start()?;
    let started = Instant::now();
    loop {
        if let Found::Serving(serving) = ask(address)? {
            return Ok(serving);
        }
        if started.elapsed() >= STARTING {
            return Err(Not::unreadable(
                &asker(address),
                format!(
                    "the serving process at http://{address}, which was started and did not answer in {} seconds",
                    STARTING.as_secs()
                ),
                None,
            ));
        }
        std::thread::sleep(AGAIN);
    }
}

/// The serving lock in `host`'s state directory, opened and made where it
/// is not there, and not yet taken.
pub fn serving_lock(host: &Host) -> Answer<File> {
    fs::create_dir_all(&host.state).map_err(|e| written(&host.state, e))?;
    let path = host.state.join(SERVING);
    OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(|e| written(&path, e))
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use tempfile::TempDir;

    use super::*;

    fn host(dir: &TempDir) -> Host {
        Host {
            state: dir.path().join("state/tailrace"),
            config: dir.path().join("config/tailrace/config.toml"),
            home: Some(dir.path().join("home")),
            data: None,
        }
    }

    fn named_as(text: &str) -> RepositoryName {
        let (owner, name) = text.split_once('/').unwrap();
        RepositoryName {
            owner: owner.into(),
            name: name.into(),
        }
    }

    /// A git repository at `path`.
    fn repository(path: &Path) -> PathBuf {
        fs::create_dir_all(path).unwrap();
        let out = std::process::Command::new("git")
            .args(["init", "-q"])
            .arg(path)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output()
            .unwrap();
        assert!(out.status.success(), "{out:?}");
        path.to_path_buf()
    }

    #[test]
    fn the_state_directory_is_under_xdg_state_home_or_the_home() {
        let home = Path::new("/home/a");
        assert_eq!(
            state(Some(OsStr::new("/xdg")), Some(home)),
            Some(PathBuf::from("/xdg/tailrace"))
        );
        assert_eq!(
            state(Some(OsStr::new("")), Some(home)),
            Some(PathBuf::from("/home/a/.local/state/tailrace"))
        );
        assert_eq!(state(None, None), None);
    }

    #[test]
    fn two_names_of_one_repository_leave_one_entry_naming_the_second() {
        let dir = TempDir::new().unwrap();
        let host = host(&dir);
        let first = repository(&dir.path().join("first"));
        let second = repository(&dir.path().join("second"));
        let tailrace = named_as("agentplot/tailrace");
        name(&host, &tailrace, &first).unwrap();
        assert_eq!(checkout_of(&host, &tailrace), Ok(first));
        name(&host, &tailrace, &second).unwrap();
        assert_eq!(checkout_of(&host, &tailrace), Ok(second.clone()));
        let text = fs::read_to_string(host.state.join(REPOSITORIES)).unwrap();
        let held: toml::Table = toml::from_str(&text).unwrap();
        assert_eq!(held.len(), 1, "{text}");
        assert_eq!(
            held["agentplot/tailrace"]["checkout"].as_str(),
            Some(second.to_str().unwrap())
        );
    }

    #[test]
    fn a_repository_none_named_is_absent_saying_so() {
        let dir = TempDir::new().unwrap();
        let host = host(&dir);
        let other = named_as("o/r");
        match checkout_of(&host, &other) {
            Err(Not::Absent(a)) => {
                assert!(a.what.contains("o/r"), "{a:?}");
                assert!(a.what.contains("has named it"), "{a:?}");
            }
            answered => panic!("answered {answered:?}"),
        }
        name(&host, &named_as("o/s"), &repository(&dir.path().join("s"))).unwrap();
        assert!(matches!(checkout_of(&host, &other), Err(Not::Absent(_))));
        assert!(!host.state.join("o").exists());
    }

    #[test]
    fn an_entry_whose_checkout_is_gone_is_unreadable_naming_the_path() {
        let dir = TempDir::new().unwrap();
        let host = host(&dir);
        let gone = repository(&dir.path().join("gone"));
        let repository = named_as("o/r");
        name(&host, &repository, &gone).unwrap();
        fs::remove_dir_all(&gone).unwrap();
        match checkout_of(&host, &repository) {
            Err(Not::Unreadable(u)) => {
                assert!(u.what.contains(&gone.display().to_string()), "{u:?}");
            }
            answered => panic!("answered {answered:?}"),
        }
        let all = named(&host).unwrap();
        assert!(matches!(&all[0].1, Err(Not::Unreadable(_))), "{all:?}");
    }

    #[test]
    fn a_file_that_does_not_parse_is_unreadable_with_the_parsers_words() {
        let dir = TempDir::new().unwrap();
        let host = host(&dir);
        fs::create_dir_all(&host.state).unwrap();
        let text = "[\"o/r\"\ncheckout = 3\n";
        fs::write(host.state.join(REPOSITORIES), text).unwrap();
        let words = toml::from_str::<Named>(text).unwrap_err().to_string();
        for answered in [
            checkout_of(&host, &named_as("o/r")).map(|_| ()),
            named(&host).map(|_| ()),
        ] {
            match answered {
                Err(Not::Unreadable(u)) => {
                    assert_eq!(u.source, Source::host(host.state.join(REPOSITORIES)));
                    assert_eq!(u.said.as_deref(), Some(words.as_str()));
                }
                answered => panic!("answered {answered:?}"),
            }
        }
    }

    #[test]
    fn every_repository_named_is_ordered_by_owner_and_name() {
        let dir = TempDir::new().unwrap();
        let host = host(&dir);
        assert_eq!(named(&host), Ok(vec![]));
        let checkout = repository(&dir.path().join("c"));
        for repository in ["a/x", "a-b/x", "a/b"] {
            name(&host, &named_as(repository), &checkout).unwrap();
        }
        let order: Vec<String> = named(&host)
            .unwrap()
            .iter()
            .map(|(r, _)| r.to_string())
            .collect();
        assert_eq!(order, ["a/b", "a/x", "a-b/x"]);
    }

    /// A listener on a port the system picks answering each connection
    /// with `answer`, and its address.
    fn answering(answer: &'static str) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut asked = [0u8; 1024];
                let _ = stream.read(&mut asked);
                let _ = stream.write_all(answer.as_bytes());
            }
        });
        address
    }

    /// A port the system picked and no one listens on.
    fn nothing() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap()
    }

    #[test]
    fn a_process_answering_as_tailrace_is_found_and_nothing_started() {
        let body = r#"{"serving":"tailrace","revision":"4f2a9c1","pid":42}"#;
        let answer: &'static str = Box::leak(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .into_boxed_str(),
        );
        let address = answering(answer);
        let found = find_or_start(address, || panic!("started a process")).unwrap();
        assert_eq!(
            found,
            Serving {
                serving: "tailrace".into(),
                revision: "4f2a9c1".into(),
                pid: 42
            }
        );
    }

    #[test]
    fn anything_else_answering_is_unreadable_naming_the_address_and_starts_nothing() {
        let address = answering("HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n<p>hello</p>");
        match find_or_start(address, || panic!("started a process")) {
            Err(Not::Unreadable(u)) => {
                assert!(u.what.contains(&address.to_string()), "{u:?}");
                assert!(
                    u.said.as_deref().is_some_and(|s| s.contains("hello")),
                    "{u:?}"
                );
            }
            answered => panic!("answered {answered:?}"),
        }
    }

    #[test]
    fn nothing_running_runs_start_once_and_answers_when_it_does() {
        let address = nothing();
        let started = Arc::new(AtomicUsize::new(0));
        let counted = Arc::clone(&started);
        let found = find_or_start(address, move || {
            counted.fetch_add(1, Ordering::SeqCst);
            let listener = TcpListener::bind(address).unwrap();
            std::thread::spawn(move || {
                let body = r#"{"serving":"tailrace","revision":"r","pid":7}"#;
                for stream in listener.incoming() {
                    let mut stream = stream.unwrap();
                    let mut asked = [0u8; 1024];
                    let _ = stream.read(&mut asked);
                    let _ = write!(
                        stream,
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                        body.len()
                    );
                }
            });
            Ok(())
        })
        .unwrap();
        assert_eq!(found.pid, 7);
        assert_eq!(started.load(Ordering::SeqCst), 1);
    }
}
