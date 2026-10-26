//! Spec host-bindings: the bindings file read from a temporary
//! directory, its path and the home directory passed in, never read
//! from the process's environment.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tailrace_seams::{Chain, ChannelId, ChannelName, SignerId};
use tempfile::TempDir;

use super::*;

/// A configuration directory and a home in a temporary directory.
struct Host {
    dir: TempDir,
}

impl Host {
    fn new() -> Host {
        Host {
            dir: TempDir::new().unwrap(),
        }
    }

    fn home(&self) -> PathBuf {
        self.dir.path().join("home")
    }

    fn file(&self) -> PathBuf {
        path(Some(self.dir.path().join("config").as_os_str()), None).unwrap()
    }

    /// `text` as the bindings file.
    fn write(&self, text: &str) {
        let file = self.file();
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, text).unwrap();
    }

    /// The user's data directory, `XDG_DATA_HOME`.
    fn data(&self) -> PathBuf {
        self.dir.path().join("data")
    }

    /// Where the given layout is, under the data directory.
    fn given_layout(&self) -> PathBuf {
        self.data().join("tailrace/layout")
    }

    fn read(&self) -> Answer<Bindings> {
        read(&self.file(), Some(&self.home()))
    }

    /// The binding the host's file, or its lack, makes for `command` in
    /// the repository at `repository`.
    fn bind(&self, repository: &Path, command: For) -> Answer<Binding> {
        bind(
            &self.file(),
            Some(&self.home()),
            Some(self.data().as_os_str()),
            repository,
            command,
        )
    }

    /// The words of an unreadable answer, once it names the file and
    /// `at`.
    fn unreadable(&self, at: Option<&str>) -> String {
        let file = self.file();
        match self.read() {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, Source::host(&file));
                let what = match at {
                    Some(at) => format!("{}, {at}", file.display()),
                    None => file.display().to_string(),
                };
                assert_eq!(u.what, what);
                u.said.expect("the words it was read with")
            }
            other => panic!("answered {other:?}, not unreadable"),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Remote {
    remote: Option<String>,
}

#[test]
fn the_file_is_under_the_configuration_directory_or_the_home() {
    let home = Path::new("/home/a");
    assert_eq!(
        path(Some(OsStr::new("/xdg")), Some(home)),
        Some(PathBuf::from("/xdg/tailrace/config.toml"))
    );
    assert_eq!(
        path(Some(OsStr::new("")), Some(home)),
        Some(PathBuf::from("/home/a/.config/tailrace/config.toml"))
    );
    assert_eq!(
        path(None, Some(home)),
        Some(PathBuf::from("/home/a/.config/tailrace/config.toml"))
    );
    assert_eq!(
        path(Some(OsStr::new("/xdg")), None),
        Some(PathBuf::from("/xdg/tailrace/config.toml"))
    );
    assert_eq!(path(None, None), None);
}

#[test]
fn each_table_is_read() {
    let host = Host::new();
    host.write(
        "[store]\nlayout = \"~/.local/share/tailrace/layout\"\n\n\
         [landing]\nbranch = { remote = \"origin\" }\n\n\
         [signing]\nsigner = \"laptop\"\nkey = \"/keys/laptop\"\n",
    );
    let bindings = host.read().unwrap();
    assert_eq!(bindings.file, host.file());
    let store = bindings.store.unwrap();
    assert_eq!(store.key(), "layout");
    assert_eq!(
        store.path(),
        Ok(host.home().join(".local/share/tailrace/layout"))
    );
    let landing = bindings.landing.unwrap();
    assert_eq!(landing.key(), "branch");
    assert_eq!(
        landing.options::<Remote>(),
        Ok(Remote {
            remote: Some("origin".into())
        })
    );
    assert_eq!(
        bindings.signing,
        Some(Signing::HostKey {
            signer: SignerId::new("laptop"),
            key: PathBuf::from("/keys/laptop"),
        })
    );
}

#[test]
fn a_key_under_the_home_is_expanded_and_tables_left_out_are_none() {
    let host = Host::new();
    host.write("[signing]\nsigner = \"laptop\"\nkey = \"~/.config/tailrace/signing\"\n");
    let bindings = host.read().unwrap();
    assert_eq!(bindings.store, None);
    assert_eq!(bindings.landing, None);
    assert_eq!(
        bindings.signing,
        Some(Signing::HostKey {
            signer: SignerId::new("laptop"),
            key: host.home().join(".config/tailrace/signing"),
        })
    );
    host.write("[landing]\nbranch = {}\n");
    let landing = host.read().unwrap().landing.unwrap();
    assert_eq!(landing.options::<Remote>(), Ok(Remote { remote: None }));
    assert_eq!(host.read().unwrap().signing, None);
}

#[test]
fn no_file_reads_as_an_empty_file() {
    let host = Host::new();
    let empty = Bindings {
        file: host.file(),
        store: None,
        landing: None,
        ledger: None,
        gate: None,
        signing: None,
    };
    assert_eq!(host.read(), Ok(empty.clone()));
    host.write("");
    assert_eq!(host.read(), Ok(empty));
}

#[test]
fn a_malformed_file_is_unreadable_in_the_parsers_words() {
    let host = Host::new();
    let text = "[store\nlayout = \"/layout\"\n";
    host.write(text);
    let words = toml::from_str::<toml::Table>(text).unwrap_err().to_string();
    assert_eq!(host.unreadable(None), words);
}

#[test]
fn a_seams_table_naming_no_profile_or_two_is_unreadable_naming_it() {
    let host = Host::new();
    host.write("[store]\n");
    assert_eq!(host.unreadable(Some("[store]")), "[store] names no profile");
    host.write("[landing]\nqueue = {}\nbranch = {}\n");
    assert_eq!(
        host.unreadable(Some("[landing]")),
        "[landing] names branch and queue, and a seam is bound to one profile"
    );
}

#[test]
fn a_relative_path_is_unreadable_naming_its_key() {
    let host = Host::new();
    host.write("[signing]\nsigner = \"laptop\"\nkey = \"keys/laptop\"\n");
    assert_eq!(
        host.unreadable(Some("[signing] key")),
        "keys/laptop is a relative path; a path here is absolute or begins ~/"
    );
    host.write("[store]\nlayout = \"layout\"\n");
    let store = host.read().unwrap().store.unwrap();
    match store.path() {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.what, format!("{}, [store] layout", host.file().display()));
            assert_eq!(
                u.said.as_deref(),
                Some("layout is a relative path; a path here is absolute or begins ~/")
            );
        }
        other => panic!("a relative layout answered {other:?}"),
    }
}

#[test]
fn a_given_tables_answers_name_the_file_and_the_table_as_given() {
    let host = Host::new();
    let file = host.file();
    let given = Table::given(&file, "store", "layout", "data/tailrace/layout".into());
    match given.path() {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, Source::host(&file));
            assert_eq!(
                u.what,
                format!("{}, [store] layout (given)", file.display())
            );
            assert_eq!(
                u.said.as_deref(),
                Some(
                    "data/tailrace/layout is a relative path; a path here is absolute or begins ~/"
                )
            );
        }
        other => panic!("a relative given layout answered {other:?}"),
    }
    let given = Table::given(&file, "store", "layout", "/data/tailrace/layout".into());
    assert_eq!(given.path(), Ok(PathBuf::from("/data/tailrace/layout")));
    assert_eq!(
        given.unreadable("said"),
        Not::unreadable(
            &Source::host(&file),
            format!("{}, [store] layout (given)", file.display()),
            Some("said".into())
        )
    );
}

#[test]
fn what_the_repository_binds_and_no_table_of_the_file_are_unreadable_naming_the_table() {
    let host = Host::new();
    for (table, said) in [
        (
            "build",
            "[build] is the repository's to bind, in its policy file, and not the host's",
        ),
        (
            "projections",
            "[projections] is the repository's to bind, in its policy file, and not the host's",
        ),
        (
            "targets",
            "[targets] is no table of the bindings file, which binds [store], [landing], [ledger], [signing] and [gate]",
        ),
    ] {
        host.write(&format!("[store]\nlayout = \"/layout\"\n\n[{table}]\nx = 1\n"));
        assert_eq!(host.unreadable(Some(&format!("[{table}]"))), said);
    }
}

#[test]
fn signing_reads_its_signer_and_key_and_nothing_else() {
    let host = Host::new();
    host.write("[signing]\nkey = \"/keys/laptop\"\n");
    assert_eq!(
        host.unreadable(Some("[signing] signer")),
        "[signing] names no signer"
    );
    host.write("[signing]\nsigner = \"laptop\"\nkey = \"/k\"\npassphrase = \"x\"\n");
    assert_eq!(
        host.unreadable(Some("[signing] passphrase")),
        "[signing] names passphrase, and reads signer and key alone"
    );
    host.write("[signing]\nsigner = 7\nkey = \"/k\"\n");
    assert_eq!(
        host.unreadable(Some("[signing] signer")),
        "[signing] signer is not a string"
    );
}

#[test]
fn a_profile_no_family_knows_is_unreadable_naming_the_table() {
    let host = Host::new();
    host.write("[store]\nbucket = \"/b\"\n");
    let store = host.read().unwrap().store.unwrap();
    match store.unknown(&["layout"]) {
        Not::Unreadable(u) => {
            assert_eq!(u.what, format!("{}, [store]", host.file().display()));
            assert_eq!(
                u.said.as_deref(),
                Some("[store] names bucket, which no store profile is bound by; it reads layout")
            );
        }
        other => panic!("answered {other:?}"),
    }
}

#[test]
fn a_registry_binds_a_store_named_by_its_repository() {
    let host = Host::new();
    host.write("[store]\nregistry = \"localhost:5000/tailrace\"\n\n[landing]\nbranch = {}\n");
    let binding = host.bind(host.dir.path(), For::Verify).unwrap();
    assert_eq!(
        binding.store,
        Source::new(Seam::Store, "registry localhost:5000/tailrace")
    );

    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    host.write(&format!(
        "[store]\nregistry = \"127.0.0.1:{port}/tailrace\"\n\n[landing]\nbranch = {{}}\n\n\
         [signing]\nsigner = \"laptop\"\nkey = \"~/.config/tailrace/signing\"\n"
    ));
    let binding = host.bind(host.dir.path(), For::Publish).unwrap();
    match binding
        .seams
        .store
        .resolve(&tailrace_seams::Name::new("v1.0.0"))
    {
        Err(Not::Unreadable(u)) => assert_eq!(
            u.source,
            Source::new(
                Seam::Store,
                format!("OCI registry 127.0.0.1:{port}/tailrace")
            )
        ),
        other => panic!("a registry nothing listens for answered {other:?}"),
    }
}

#[test]
fn a_registry_named_by_what_is_no_repository_is_unreadable_naming_the_key_and_the_value() {
    let host = Host::new();
    for (value, names) in [("\"tailrace\"", "\"tailrace\""), ("5000", "5000")] {
        host.write(&format!(
            "[store]\nregistry = {value}\n\n[landing]\nbranch = {{}}\n"
        ));
        match host.bind(host.dir.path(), For::Verify) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, Source::host(host.file()));
                assert_eq!(
                    u.what,
                    format!("{}, [store] registry", host.file().display())
                );
                let said = u.said.unwrap();
                assert!(said.contains(names), "{said}");
            }
            other => panic!("registry = {value} answered {:?}", other.err()),
        }
    }
}

#[test]
fn a_store_key_no_profile_reads_is_unreadable_listing_both_keys() {
    let host = Host::new();
    host.write("[store]\nbucket = \"/b\"\n\n[landing]\nbranch = {}\n");
    match host.bind(host.dir.path(), For::Verify) {
        Err(Not::Unreadable(u)) => assert_eq!(
            u.said.as_deref(),
            Some(
                "[store] names bucket, which no store profile is bound by; \
                 it reads layout or registry"
            )
        ),
        other => panic!("answered {:?}", other.err()),
    }
}

/// A file binding the layout at `layout`, a branch with no remote and,
/// where `signing`, the signer `laptop`; and the binding it makes for
/// `command`.
fn bound(host: &Host, layout: &Path, signing: bool, command: For) -> Answer<Binding> {
    let signing = if signing {
        "\n[signing]\nsigner = \"laptop\"\nkey = \"~/.config/tailrace/signing\"\n"
    } else {
        ""
    };
    host.write(&format!(
        "[store]\nlayout = \"{}\"\n\n[landing]\nbranch = {{}}\n{signing}",
        layout.display()
    ));
    host.bind(host.dir.path(), command)
}

fn claim() -> tailrace_seams::Claim {
    tailrace_seams::Claim {
        subject: Digest::of(b"an artifact"),
        kind: tailrace_seams::ClaimKind::Provenance,
        revision: tailrace_seams::Revision::new("4f2a9c1"),
        build: None,
        body: serde_json::json!({}),
    }
}

#[test]
fn publish_makes_the_layout_it_names_and_verify_never_does() {
    let host = Host::new();
    let layout = host.dir.path().join("layouts").join("laptop");
    let verify = bound(&host, &layout, true, For::Verify);
    assert!(
        matches!(&verify, Err(Not::Unreadable(u)) if u.source.seam == Seam::Store
            && u.said.as_deref().is_some_and(|s| s.contains("No such file or directory"))),
        "{:?}",
        verify.err()
    );
    assert!(!layout.exists(), "verify made {}", layout.display());

    let publish = bound(&host, &layout, true, For::Publish).unwrap();
    assert!(
        profiles::store::layout::Layout::open(&layout).is_ok(),
        "publish made no layout"
    );
    assert_eq!(publish.signer, Some(SignerId::new("laptop")));
    let held = publish.seams.store.put_bytes(b"abc").unwrap();
    assert_eq!(held.digest, Digest::of(b"abc"));
    assert!(bound(&host, &layout, false, For::Verify).is_ok());
}

#[test]
fn the_store_is_named_by_its_key_and_its_path_under_the_home() {
    let host = Host::new();
    host.write(
        "[store]\nlayout = \"~/layout\"\n\n[landing]\nbranch = {}\n\n\
         [signing]\nsigner = \"laptop\"\nkey = \"~/.config/tailrace/signing\"\n",
    );
    let binding = host.bind(host.dir.path(), For::Publish).unwrap();
    assert_eq!(
        binding.store,
        Source::new(
            Seam::Store,
            format!("layout {}", host.home().join("layout").display())
        )
    );
}

#[test]
fn with_no_signing_the_host_only_verifies() {
    let host = Host::new();
    let layout = host.dir.path().join("layout");
    profiles::store::layout::Layout::create(&layout).unwrap();
    let binding = bound(&host, &layout, false, For::Verify).unwrap();
    assert_eq!(binding.signer, None);
    assert_eq!(binding.seams.signers.len(), 2);
    for signer in &binding.seams.signers {
        match signer.sign(&claim(), &SignerId::new("laptop")) {
            Err(Not::Absent(a)) => {
                assert_eq!(a.source.seam, Seam::Signer);
                assert!(a.what.contains("no identity is at hand"), "{a:?}");
            }
            other => panic!("a verify-only signer answered {other:?}"),
        }
    }
}

#[test]
fn a_file_naming_no_ledger_binds_the_git_ref_in_the_repository() {
    let host = Host::new();
    let repo = repository(&host);
    let layout = host.dir.path().join("layout");
    let binding = bound_with(&host, &repo, &layout, "", true, For::Publish).unwrap();
    match binding.seams.keeper.chain(&stable()) {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source, Source::new(Seam::Keeper, "git_ref"));
            assert_eq!(a.what, "chain of stable");
        }
        other => panic!("the given ledger answered {other:?}"),
    }
    assert!(binding.seams.targets.is_empty());
    let ecosystems: Vec<&str> = binding
        .seams
        .projections
        .keys()
        .map(|e| e.as_str())
        .collect();
    assert_eq!(ecosystems, ["file", "image", "moon-cache"]);
}

/// A repository in the host's directory, made with git, and its path.
fn repository(host: &Host) -> PathBuf {
    let repo = host.dir.path().join("repo");
    let made = std::process::Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .arg(&repo)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .status()
        .unwrap();
    assert!(made.success());
    repo.canonicalize().unwrap()
}

/// A file binding the layout at `layout`, a branch with no remote,
/// `ledger` as its `[ledger]` where it is not empty and, where `signing`,
/// the signer `laptop`; and the binding it makes for `command` in
/// `repo`.
fn bound_with(
    host: &Host,
    repo: &Path,
    layout: &Path,
    ledger: &str,
    signing: bool,
    command: For,
) -> Answer<Binding> {
    let signing = if signing {
        "\n[signing]\nsigner = \"laptop\"\nkey = \"~/.config/tailrace/signing\"\n"
    } else {
        ""
    };
    let ledger = if ledger.is_empty() {
        String::new()
    } else {
        format!("\n[ledger]\n{ledger}\n")
    };
    host.write(&format!(
        "[store]\nlayout = \"{}\"\n\n[landing]\nbranch = {{}}\n{ledger}{signing}",
        layout.display()
    ));
    host.bind(repo, command)
}

/// What the bound ledger answers for `stable`'s chain.
fn chain_of(binding: Answer<Binding>) -> Answer<Chain> {
    binding
        .unwrap_or_else(|not| panic!("the file bound nothing: {not}"))
        .seams
        .keeper
        .chain(&ChannelId::new(None, ChannelName::new("stable")))
}

#[test]
fn a_git_ref_ledger_with_and_without_a_remote_and_a_store_ledger_are_bound() {
    let host = Host::new();
    let repo = repository(&host);
    let layout = host.dir.path().join("layout");
    profiles::store::layout::Layout::create(&layout).unwrap();

    match chain_of(bound_with(
        &host,
        &repo,
        &layout,
        "git_ref = {}",
        true,
        For::Promote,
    )) {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source, Source::new(Seam::Keeper, "git_ref"));
            assert_eq!(a.what, "chain of stable");
        }
        other => panic!("a git ref ledger with no remote answered {other:?}"),
    }
    let remote = "git_ref = { remote = \"nowhere\" }";
    match chain_of(bound_with(
        &host,
        &repo,
        &layout,
        remote,
        true,
        For::Promote,
    )) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, Source::new(Seam::Keeper, "git_ref"));
            assert!(
                u.said.as_deref().is_some_and(|s| s.contains("'nowhere'")),
                "{u:?}"
            );
        }
        other => panic!("a git ref ledger with a remote answered {other:?}"),
    }
    match chain_of(bound_with(
        &host,
        &repo,
        &layout,
        "store = {}",
        true,
        For::Promote,
    )) {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source, Source::new(Seam::Keeper, "store"));
            assert_eq!(a.what, "chain of stable");
        }
        other => panic!("a store ledger answered {other:?}"),
    }
    let binding = bound_with(&host, &repo, &layout, "store = {}", false, For::Verify).unwrap();
    assert_eq!(
        binding.signer, None,
        "a verify binds the ledger and no signer"
    );
}

#[test]
fn a_ledger_table_naming_two_profiles_none_or_an_option_its_profile_does_not_read_is_unreadable() {
    let host = Host::new();
    host.write("[ledger]\ngit_ref = {}\nstore = {}\n");
    assert_eq!(
        host.unreadable(Some("[ledger]")),
        "[ledger] names git_ref and store, and a seam is bound to one profile"
    );
    host.write("[ledger]\n");
    assert_eq!(
        host.unreadable(Some("[ledger]")),
        "[ledger] names no profile"
    );

    let repo = repository(&host);
    let layout = host.dir.path().join("layout");
    profiles::store::layout::Layout::create(&layout).unwrap();
    for (ledger, at, words) in [
        (
            "git_ref = { remote = \"origin\", branch = \"main\" }",
            "[ledger] git_ref",
            "branch",
        ),
        (
            "store = { remote = \"origin\" }",
            "[ledger] store",
            "remote",
        ),
        ("table = {}", "[ledger]", "it reads git_ref or store"),
    ] {
        for command in [For::Verify, For::Promote] {
            match bound_with(&host, &repo, &layout, ledger, true, command) {
                Err(Not::Unreadable(u)) => {
                    assert_eq!(u.what, format!("{}, {at}", host.file().display()));
                    assert!(
                        u.said.as_deref().is_some_and(|s| s.contains(words)),
                        "{u:?}"
                    );
                }
                other => panic!("[ledger] {ledger} answered {:?}", other.err()),
            }
        }
    }
}

#[test]
fn the_store_ledgers_binding_never_makes_the_layout() {
    let host = Host::new();
    let repo = repository(&host);
    let layout = host.dir.path().join("layouts").join("laptop");
    let promote = bound_with(&host, &repo, &layout, "store = {}", true, For::Promote);
    assert!(
        matches!(&promote, Err(Not::Unreadable(u)) if u.source.seam == Seam::Store
            && u.said.as_deref().is_some_and(|s| s.contains("No such file or directory"))),
        "{:?}",
        promote.err()
    );
    assert!(
        !layout.exists(),
        "the store ledger made {}",
        layout.display()
    );
    let publish = bound_with(&host, &repo, &layout, "store = {}", true, For::Publish).unwrap();
    assert!(
        matches!(
            publish
                .seams
                .keeper
                .chain(&ChannelId::new(None, ChannelName::new("stable"))),
            Err(Not::Absent(_))
        ),
        "the store ledger did not read the layout a publish made"
    );
}

#[test]
fn a_publishs_first_binding_makes_the_layout_and_the_moon_caches_store_reads_it() {
    let host = Host::new();
    let layout = host.dir.path().join("layouts").join("laptop");
    let binding = bound(&host, &layout, true, For::Publish).unwrap();
    let digest = binding.seams.store.put_bytes(b"an archive").unwrap().digest;
    let moon = binding
        .seams
        .projection(&tailrace_seams::Ecosystem::new("moon-cache"))
        .expect("the moon cache is bound");
    let entries = [tailrace_seams::CacheEntry {
        build: tailrace_seams::BuildId::new("b7e1"),
        digest,
    }];
    let checkout = host.dir.path().display().to_string();
    moon.render(tailrace_seams::Contents::CacheEntries(&entries), &checkout)
        .unwrap();
    let archive = host.dir.path().join(".moon/cache/outputs/b7e1.tar.gz");
    assert_eq!(fs::read(&archive).unwrap(), b"an archive");
}

#[test]
fn sync_is_bound_as_verify_is_and_never_makes_the_layout() {
    let host = Host::new();
    let layout = host.dir.path().join("layout");
    assert!(
        matches!(bound(&host, &layout, false, For::Sync), Err(Not::Unreadable(u)) if u.source.seam == Seam::Store),
        "sync bound a layout that is not there"
    );
    assert!(!layout.exists(), "sync made {}", layout.display());
    profiles::store::layout::Layout::create(&layout).unwrap();
    let binding = bound(&host, &layout, false, For::Sync).unwrap();
    assert_eq!(binding.signer, None);
}

#[test]
fn a_push_needs_no_signing_and_makes_a_missing_layout() {
    let host = Host::new();
    let layout = host.dir.path().join("layouts").join("landing");
    let binding = bound(&host, &layout, false, For::Push).unwrap();
    assert_eq!(binding.signer, None);
    assert!(
        profiles::store::layout::Layout::open(&layout).is_ok(),
        "push made no layout"
    );
    let digest = binding.seams.store.put_bytes(b"an archive").unwrap().digest;
    assert_eq!(digest, Digest::of(b"an archive"));
    assert!(
        binding
            .seams
            .projection(&tailrace_seams::Ecosystem::new("moon-cache"))
            .is_some(),
        "the moon cache is not bound"
    );
}

#[test]
fn rebuild_is_bound_as_verify_is_and_never_makes_the_layout() {
    let host = Host::new();
    let layout = host.dir.path().join("layout");
    let rebuild = bound(&host, &layout, false, For::Rebuild);
    assert!(
        matches!(&rebuild, Err(Not::Unreadable(u)) if u.source.seam == Seam::Store
            && u.said.as_deref().is_some_and(|s| s.contains("No such file or directory"))),
        "{:?}",
        rebuild.err()
    );
    assert!(!layout.exists(), "rebuild made {}", layout.display());
    profiles::store::layout::Layout::create(&layout).unwrap();
    let binding = bound(&host, &layout, false, For::Rebuild).unwrap();
    assert_eq!(binding.signer, None);
}

#[test]
fn a_profile_no_family_reads_is_unreadable_and_makes_nothing() {
    let host = Host::new();
    let layout = host.dir.path().join("layout");
    host.write(&format!(
        "[store]\nlayout = \"{}\"\n\n[landing]\nqueue = {{}}\n\n\
         [signing]\nsigner = \"laptop\"\nkey = \"/k\"\n",
        layout.display()
    ));
    match host.bind(host.dir.path(), For::Publish) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.what, format!("{}, [landing]", host.file().display()));
            assert!(
                u.said.as_deref().is_some_and(|s| s.contains("queue")),
                "{u:?}"
            );
        }
        other => panic!("answered {:?}", other.err()),
    }
    assert!(!layout.exists(), "a file that cannot bind made the layout");
}

/// A file binding the layout at `layout`, a branch with no remote, the
/// signer `laptop` and `gate` as its `[gate]` where it is not empty; and
/// the binding it makes for `command`.
fn bound_telling(host: &Host, layout: &Path, gate: &str, command: For) -> Answer<Binding> {
    let gate = if gate.is_empty() {
        String::new()
    } else {
        format!("\n[gate]\n{gate}\n")
    };
    host.write(&format!(
        "[store]\nlayout = \"{}\"\n\n[landing]\nbranch = {{}}\n\n\
         [signing]\nsigner = \"laptop\"\nkey = \"~/.config/tailrace/signing\"\n{gate}",
        layout.display()
    ));
    host.bind(host.dir.path(), command)
}

/// Where in the file an unreadable binding names, and its words.
fn unreadable_at(host: &Host, answer: Answer<Binding>) -> (String, String) {
    match answer {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, Source::host(host.file()));
            let at = u
                .what
                .strip_prefix(&format!("{}, ", host.file().display()))
                .unwrap_or_else(|| panic!("{} names no place in the file", u.what))
                .to_string();
            (at, u.said.expect("the words it was read with"))
        }
        Err(other) => panic!("answered {other:?}, not unreadable"),
        Ok(_) => panic!("bound, not unreadable"),
    }
}

#[test]
fn the_hosts_publish_tells_its_own_gate() {
    let host = Host::new();
    let layout = host.dir.path().join("layout");
    let binding = bound_telling(&host, &layout, "publish = {}", For::Publish).unwrap();
    assert_eq!(binding.gate, Teller::Publish);
}

#[test]
fn a_job_tells_a_separated_gate_in_its_words_as_written() {
    let host = Host::new();
    let layout = host.dir.path().join("layout");
    let words = "the release job a push to main wakes";
    for command in [For::Publish, For::Verify] {
        let binding =
            bound_telling(&host, &layout, &format!("job = \"{words}\""), command).unwrap();
        assert_eq!(binding.gate, Teller::Job(words.to_string()));
    }
}

#[test]
fn two_tellers_are_unreadable_naming_both_and_make_no_layout() {
    let host = Host::new();
    let layout = host.dir.path().join("layout");
    for command in [For::Publish, For::Verify] {
        let answer = bound_telling(&host, &layout, "publish = {}\njob = \"a job\"", command);
        let (at, said) = unreadable_at(&host, answer);
        assert_eq!(at, "[gate]");
        assert_eq!(
            said,
            "[gate] names job and publish, and a seam is bound to one profile"
        );
        assert!(!layout.exists(), "{command:?} made {}", layout.display());
    }
}

#[test]
fn a_teller_the_file_does_not_know_is_unreadable_naming_the_keys_it_reads() {
    let host = Host::new();
    let layout = host.dir.path().join("layout");
    let answer = bound_telling(&host, &layout, "webhook = \"a hook\"", For::Publish);
    let (at, said) = unreadable_at(&host, answer);
    assert_eq!(at, "[gate]");
    assert_eq!(
        said,
        "[gate] names webhook, which no gate profile is bound by; it reads publish or job"
    );
    assert!(!layout.exists());
}

#[test]
fn a_publish_teller_with_options_or_a_job_with_no_words_is_unreadable_naming_its_key() {
    let host = Host::new();
    let layout = host.dir.path().join("layout");
    for (gate, at, said) in [
        (
            "publish = { now = true }",
            "[gate] publish",
            "[gate] publish takes no options: publish = {}",
        ),
        (
            "publish = true",
            "[gate] publish",
            "[gate] publish takes no options: publish = {}",
        ),
        (
            "job = \"\"",
            "[gate] job",
            "[gate] job holds no words naming the job",
        ),
        (
            "job = \"  \"",
            "[gate] job",
            "[gate] job holds no words naming the job",
        ),
        (
            "job = 7",
            "[gate] job",
            "[gate] job is not the job's words in a string",
        ),
    ] {
        let answer = bound_telling(&host, &layout, gate, For::Publish);
        assert_eq!(
            unreadable_at(&host, answer),
            (at.to_string(), said.to_string()),
            "{gate}"
        );
        assert!(!layout.exists(), "{gate} made {}", layout.display());
    }
}

#[test]
fn a_file_naming_no_gate_binds_as_before_and_the_hosts_publish_tells_its_gate() {
    let host = Host::new();
    let layout = host.dir.path().join("layout");
    let binding = bound_telling(&host, &layout, "", For::Publish).unwrap();
    assert_eq!(binding.signer, Some(SignerId::new("laptop")));
    assert!(profiles::store::layout::Layout::open(&layout).is_ok());
    assert_eq!(binding.gate, Teller::Publish);
    let named = bound_telling(&host, &layout, "publish = {}", For::Publish).unwrap();
    assert_eq!(binding.gate, named.gate);
}

// Requirement: Status needs only what it asks

/// Every file under `dir` with its contents, to tell nothing was made.
fn files(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut all = Vec::new();
    let mut dirs = vec![dir.to_path_buf()];
    while let Some(at) = dirs.pop() {
        for entry in fs::read_dir(&at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                all.push((path.clone(), Vec::new()));
                dirs.push(path);
            } else {
                all.push((path.clone(), fs::read(&path).unwrap()));
            }
        }
    }
    all.sort();
    all
}

fn stable() -> ChannelId {
    ChannelId::new(None, ChannelName::new("stable"))
}

#[test]
fn a_file_naming_a_store_ledger_binds_status_opening_the_layout_and_never_making_it() {
    let host = Host::new();
    let repo = repository(&host);
    let layout = host.dir.path().join("layout");
    host.write(&format!(
        "[store]\nlayout = \"{}\"\n\n[landing]\nbranch = {{}}\n\n[ledger]\nstore = {{}}\n",
        layout.display()
    ));
    let missing = host.bind(&repo, For::Status);
    assert!(
        matches!(&missing, Err(Not::Unreadable(u)) if u.source.seam == Seam::Store),
        "{:?}",
        missing.err()
    );
    assert!(!layout.exists(), "status made {}", layout.display());
    profiles::store::layout::Layout::create(&layout).unwrap();
    match chain_of(host.bind(&repo, For::Status)) {
        Err(Not::Absent(a)) => assert_eq!(a.source, Source::new(Seam::Keeper, "store")),
        other => panic!("a store ledger answered {other:?}"),
    }
}

#[test]
fn a_git_ref_ledger_binds_status_with_the_layout_it_names_missing_and_opens_no_store() {
    let host = Host::new();
    let repo = repository(&host);
    let layout = host.dir.path().join("layout");
    host.write(&format!(
        "[store]\nlayout = \"{}\"\n\n[landing]\nbranch = {{}}\n\n[ledger]\ngit_ref = {{}}\n",
        layout.display()
    ));
    let binding = host
        .bind(&repo, For::Status)
        .unwrap_or_else(|not| panic!("a missing layout bound nothing: {not}"));
    assert!(!layout.exists(), "status made {}", layout.display());
    assert_eq!(
        binding.store,
        Source::new(Seam::Store, format!("layout {}", layout.display())),
        "the store is named as the file names it"
    );
    match binding.seams.keeper.chain(&stable()) {
        Err(Not::Absent(a)) => assert_eq!(a.source, Source::new(Seam::Keeper, "git_ref")),
        other => panic!("a git ref ledger answered {other:?}"),
    }
    match binding.seams.store.resolve(&Name::new("v0.1.0")) {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source, Source::new(Seam::Store, "unbound"));
            assert!(a.what.contains("[store]"), "{a:?}");
        }
        other => panic!("status's store answered {other:?}"),
    }
}

#[test]
fn the_policy_command_binds_no_store_and_no_signer_whatever_the_file_names() {
    let host = Host::new();
    let repo = repository(&host);
    let layout = host.dir.path().join("layout");
    host.write(&format!(
        "[store]\nlayout = \"{}\"\n\n[landing]\nbranch = {{}}\n\n\
         [signing]\nsigner = \"laptop\"\nkey = \"/keys/laptop\"\n",
        layout.display()
    ));
    let no_store = || {
        Err(Not::absent(
            &Source::new(Seam::Store, "unbound"),
            "binding of the store, which tailrace policy binds none",
        ))
    };
    let binding = host
        .bind(&repo, For::Policy)
        .unwrap_or_else(|not| panic!("policy bound nothing: {not}"));
    assert_eq!(
        binding.seams.store.resolve(&Name::new("v0.1.0")),
        no_store()
    );
    assert_eq!(binding.store, Source::new(Seam::Store, "unbound"));
    assert_eq!(binding.signer, None);
    assert!(binding.seams.projections.is_empty() && binding.seams.targets.is_empty());
    assert!(!layout.exists(), "policy made {}", layout.display());

    fs::remove_file(host.file()).unwrap();
    let before = files(host.dir.path());
    let binding = host
        .bind(&repo, For::Policy)
        .unwrap_or_else(|not| panic!("policy on a host with no file bound nothing: {not}"));
    assert_eq!(
        binding.seams.store.resolve(&Name::new("v0.1.0")),
        no_store()
    );
    assert_eq!(binding.signer, None);
    assert_eq!(files(host.dir.path()), before, "binding policy made a file");
}

/// A repository in the host's directory with one commit on `main`, its
/// remote `origin` at `origin` where one is given, and its path and the
/// commit.
fn committed(host: &Host, origin: Option<&Path>) -> (PathBuf, tailrace_seams::Revision) {
    let repo = repository(host);
    let git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(&repo)
            .args(args)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_AUTHOR_NAME", "A. Uthor")
            .env("GIT_AUTHOR_EMAIL", "author@example.com")
            .env("GIT_COMMITTER_NAME", "A. Uthor")
            .env("GIT_COMMITTER_EMAIL", "author@example.com")
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {out:?}");
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    };
    git(&["commit", "-q", "--allow-empty", "-m", "feat: a start"]);
    if let Some(origin) = origin {
        git(&["remote", "add", "origin", origin.to_str().unwrap()]);
    }
    let head = git(&["rev-parse", "HEAD"]);
    (repo, tailrace_seams::Revision::new(head))
}

#[test]
fn the_landings_remote_is_the_files_or_origin_with_no_file_or_none() {
    let host = Host::new();
    let gone = host.dir.path().join("gone.git");
    let (repo, _) = committed(&host, Some(&gone));
    let remote = || landing_remote(&host.file(), Some(&host.home()), &repo);
    assert_eq!(remote(), Ok(Some("origin".to_string())));
    host.write("[landing]\nbranch = { remote = \"upstream\" }\n");
    assert_eq!(remote(), Ok(Some("upstream".to_string())));
    host.write("[landing]\nsluice = {}\n");
    assert_eq!(remote(), Ok(None));
    host.write("[landing]\nqueue = {}\n");
    assert!(matches!(remote(), Err(Not::Unreadable(_))));

    let bare = Host::new();
    let (repo, _) = committed(&bare, None);
    assert_eq!(
        landing_remote(&bare.file(), Some(&bare.home()), &repo),
        Ok(None)
    );
}

// Requirement: The given landing is sluice where the repository is set up
// for it

/// git in `repo` under a fixed author and none of the host's
/// configuration; panics with git's words where it fails.
fn git_in(repo: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "A. Uthor")
        .env("GIT_AUTHOR_EMAIL", "author@example.com")
        .env("GIT_COMMITTER_NAME", "A. Uthor")
        .env("GIT_COMMITTER_EMAIL", "author@example.com")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// sluice's policy file written into `repo`'s working tree.
fn sluice_file(repo: &Path) {
    fs::create_dir_all(repo.join(".config")).unwrap();
    fs::write(repo.join(".config/sluice.toml"), "[branches.main]\n").unwrap();
}

/// Which profile `landing` is, by the source of its answer once the
/// repository at `repo` is moved away and git cannot open it.
fn landing_source(landing: &dyn tailrace_seams::Landing, repo: &Path) -> Source {
    let moved = repo.with_extension("moved");
    fs::rename(repo, &moved).unwrap();
    let answer = landing.landed(
        &tailrace_seams::Branch::new("main"),
        &tailrace_seams::Revision::new("HEAD"),
    );
    fs::rename(&moved, repo).unwrap();
    match answer {
        Err(Not::Unreadable(u)) => u.source,
        other => panic!("a landing over no repository answered {other:?}"),
    }
}

fn sluice() -> Source {
    Source::new(Seam::Landing, "sluice")
}

fn branch() -> Source {
    Source::new(Seam::Landing, "branch")
}

#[test]
fn sluices_file_in_the_head_commit_binds_sluice() {
    let host = Host::new();
    let repo = repository(&host);
    sluice_file(&repo);
    git_in(&repo, &["add", ".config/sluice.toml"]);
    git_in(&repo, &["commit", "-q", "-m", "chore: sluice gates main"]);
    let (landing, _) = profiles::landing::given(&host.file(), &repo).unwrap();
    assert_eq!(landing_source(landing.as_ref(), &repo), sluice());
}

#[test]
fn sluices_file_only_in_the_working_tree_no_file_and_no_commit_bind_a_branch() {
    let host = Host::new();
    let repo = repository(&host);
    // No commit yet, sluice's file in the working tree alone.
    sluice_file(&repo);
    let (landing, _) = profiles::landing::given(&host.file(), &repo).unwrap();
    assert_eq!(landing_source(landing.as_ref(), &repo), branch());
    // A commit that holds no sluice file, with one in the working tree.
    git_in(
        &repo,
        &["commit", "-q", "--allow-empty", "-m", "feat: a start"],
    );
    let (landing, _) = profiles::landing::given(&host.file(), &repo).unwrap();
    assert_eq!(landing_source(landing.as_ref(), &repo), branch());
    // No file anywhere.
    fs::remove_dir_all(repo.join(".config")).unwrap();
    let (landing, _) = profiles::landing::given(&host.file(), &repo).unwrap();
    assert_eq!(landing_source(landing.as_ref(), &repo), branch());
}

#[test]
fn a_directory_git_cannot_open_is_unreadable_in_gits_words() {
    let host = Host::new();
    let dir = host.dir.path().join("plain");
    fs::create_dir(&dir).unwrap();
    match profiles::landing::given(&host.file(), &dir) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, sluice());
            let said = u.said.unwrap();
            assert!(said.starts_with("fatal: not a git repository"), "{said}");
        }
        other => panic!("no repository answered {:?}", other.err()),
    }
}

// Requirement: The given landing remote is the checkout's origin or its
// one remote

/// A repository in the host's directory with one commit on `main` and
/// each of `remotes`, each at a path in the host's directory named for
/// it and holding nothing, so a landing that reads one names its path;
/// its path and the commit.
fn with_remotes(host: &Host, remotes: &[&str]) -> (PathBuf, tailrace_seams::Revision) {
    let (repo, head) = committed(host, None);
    for remote in remotes {
        let at = host.dir.path().join(format!("{remote}.git"));
        git_in(&repo, &["remote", "add", remote, at.to_str().unwrap()]);
    }
    (repo, head)
}

/// The remote `landing` reads, by the path its answer of `head` on
/// `main` names, or none where it reads the repository's own refs and
/// finds `head` landed.
fn remote_read(
    host: &Host,
    landing: &dyn tailrace_seams::Landing,
    head: &tailrace_seams::Revision,
) -> Option<String> {
    match landing.landed(&tailrace_seams::Branch::new("main"), head) {
        Ok(true) => None,
        Err(Not::Unreadable(u)) => {
            let said = u.said.unwrap_or_default();
            let read: Vec<&str> = ["origin", "upstream", "github", "gitlab"]
                .into_iter()
                .filter(|remote| {
                    let at = host.dir.path().join(format!("{remote}.git"));
                    said.contains(&at.display().to_string())
                })
                .collect();
            match read.as_slice() {
                [remote] => Some(remote.to_string()),
                _ => panic!("the landing read no one remote: {said}"),
            }
        }
        other => panic!("the landing answered {other:?}"),
    }
}

#[test]
fn a_clone_reads_origin_beside_another_remote() {
    let host = Host::new();
    let (repo, head) = with_remotes(&host, &["upstream", "origin"]);
    assert_eq!(
        profiles::landing::given_remote(&host.file(), &repo),
        Ok(Some("origin".to_string()))
    );
    let (landing, _) = profiles::landing::given(&host.file(), &repo).unwrap();
    assert_eq!(
        remote_read(&host, landing.as_ref(), &head).as_deref(),
        Some("origin")
    );
}

#[test]
fn one_remote_of_another_name_is_read() {
    let host = Host::new();
    let (repo, head) = with_remotes(&host, &["github"]);
    assert_eq!(
        profiles::landing::given_remote(&host.file(), &repo),
        Ok(Some("github".to_string()))
    );
    let (landing, _) = profiles::landing::given(&host.file(), &repo).unwrap();
    assert_eq!(
        remote_read(&host, landing.as_ref(), &head).as_deref(),
        Some("github")
    );
}

#[test]
fn no_remote_reads_the_repositorys_own_refs() {
    let host = Host::new();
    let (repo, head) = with_remotes(&host, &[]);
    assert_eq!(
        profiles::landing::given_remote(&host.file(), &repo),
        Ok(None)
    );
    let (landing, _) = profiles::landing::given(&host.file(), &repo).unwrap();
    assert_eq!(remote_read(&host, landing.as_ref(), &head), None);
}

#[test]
fn several_remotes_none_origin_are_unreadable_naming_each_and_the_landing_table() {
    let host = Host::new();
    let (repo, _) = with_remotes(&host, &["gitlab", "github"]);
    let unreadable = Not::unreadable(
        &branch(),
        format!("the landing remote of the checkout at {}", repo.display()),
        Some(format!(
            "the checkout has the remotes github, gitlab, none of them origin; name one in \
             [landing] remote of {}",
            host.file().display()
        )),
    );
    assert_eq!(
        profiles::landing::given_remote(&host.file(), &repo),
        Err(unreadable.clone())
    );
    assert_eq!(
        profiles::landing::given(&host.file(), &repo).err(),
        Some(unreadable)
    );
}

#[test]
fn a_written_landing_with_no_remote_reads_its_own_refs_in_a_clone() {
    let host = Host::new();
    let (repo, head) = with_remotes(&host, &["origin"]);
    let layout = host.dir.path().join("layout");
    profiles::store::layout::Layout::create(&layout).unwrap();
    let binding = bound_with(&host, &repo, &layout, "", false, For::Verify).unwrap();
    assert_eq!(
        remote_read(&host, binding.seams.landing.as_ref(), &head),
        None
    );
}

// Requirement: A table the file leaves out is bound to its given profile,
// the given ledger kept on the landing's remote

/// `stable`'s ref in the ledger's git refs.
const STABLE_REF: &str = "refs/tailrace/ledger/stable";

/// A bare repository at `<name>.git` in the host's directory, as a
/// remote holds a checkout's refs, and its path.
fn bare(host: &Host, name: &str) -> PathBuf {
    let at = host.dir.path().join(format!("{name}.git"));
    let made = std::process::Command::new("git")
        .args(["init", "-q", "--bare", "-b", "main"])
        .arg(&at)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .status()
        .unwrap();
    assert!(made.success());
    at
}

/// The commit `name` names in the repository at `git_dir`, a checkout
/// or a bare one, or none where it names none.
fn ref_at(git_dir: &Path, name: &str) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(git_dir)
        .args(["rev-parse", "--verify", "--quiet", name])
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    out.status
        .success()
        .then(|| String::from_utf8(out.stdout).unwrap().trim().to_string())
}

/// The first promotion of `stable`, signed as the gate.
fn promotion() -> Envelope {
    let body = tailrace_seams::Promotion {
        channel: ChannelName::new("stable"),
        component: stable().component_name(),
        to: tailrace_seams::Moved {
            version: tailrace_seams::Version::new("0.1.0"),
            index: Digest::of(b"0.1.0's index"),
        },
        from: None,
        predecessor: None,
        asked_by: SignerId::new("gate"),
        policy: tailrace_seams::Satisfied {
            revision: tailrace_seams::Revision::new(format!("{:040x}", 1)),
            claims: vec![],
        },
        at: 1_767_225_600,
    };
    let signer = tailrace_fakes::FakeSigner::new();
    signer.hold(&SignerId::new("gate"));
    tailrace_seams::Signer::sign(
        &signer,
        &body.claim(tailrace_seams::Revision::new(format!("{:040x}", 1))),
        &SignerId::new("gate"),
    )
    .unwrap()
}

/// `stable`'s first promotion appended through the ledger the host binds
/// for a promote in `repo`, its given layout made first, since a promote
/// never makes it.
fn promoted(host: &Host, repo: &Path) -> Envelope {
    profiles::store::layout::Layout::create(host.given_layout()).unwrap();
    let binding = host
        .bind(repo, For::Promote)
        .unwrap_or_else(|not| panic!("promote bound nothing: {not}"));
    let promotion = promotion();
    binding
        .seams
        .keeper
        .append(&stable(), &promotion, None)
        .unwrap();
    promotion
}

#[test]
fn no_ledger_table_keeps_the_chain_on_origin_and_a_second_clone_with_no_file_reads_it() {
    let host = Host::new();
    let origin = bare(&host, "origin");
    let (repo, _) = committed(&host, Some(&origin));
    let promotion = promoted(&host, &repo);
    let held = ref_at(&origin, STABLE_REF).expect("origin holds stable's chain");
    assert_eq!(ref_at(&repo, STABLE_REF), Some(held), "the copy follows");

    // A second machine, with no bindings file, in another clone of origin.
    let other = Host::new();
    let clone = other.dir.path().join("clone");
    git_in(
        other.dir.path(),
        &[
            "clone",
            "-q",
            origin.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    let chain = other
        .bind(&clone, For::Status)
        .unwrap_or_else(|not| panic!("status bound nothing: {not}"))
        .seams
        .keeper
        .chain(&stable())
        .unwrap_or_else(|not| panic!("the second clone read no chain: {not}"));
    assert_eq!(chain.claims, vec![promotion]);
    assert_eq!(chain.read, tailrace_seams::Read::Source);
}

#[test]
fn a_checkout_with_no_remote_and_no_ledger_table_appends_to_its_own_ref() {
    let host = Host::new();
    let (repo, _) = committed(&host, None);
    promoted(&host, &repo);
    assert!(ref_at(&repo, STABLE_REF).is_some(), "its own ref holds it");
    assert_eq!(ref_at(&repo, "refs/tailrace/fetched/stable"), None);
}

#[test]
fn a_written_landing_with_no_remote_keeps_the_given_ledger_in_its_own_refs() {
    let host = Host::new();
    let origin = bare(&host, "origin");
    let (repo, _) = committed(&host, Some(&origin));
    host.write("[landing]\nbranch = {}\n");
    promoted(&host, &repo);
    assert!(ref_at(&repo, STABLE_REF).is_some(), "its own ref holds it");
    assert_eq!(ref_at(&origin, STABLE_REF), None, "origin was written");
}

#[test]
fn a_written_landings_remote_carries_the_given_ledger() {
    let host = Host::new();
    let origin = bare(&host, "origin");
    let upstream = bare(&host, "upstream");
    let (repo, _) = committed(&host, Some(&origin));
    git_in(
        &repo,
        &["remote", "add", "upstream", upstream.to_str().unwrap()],
    );
    host.write("[landing]\nbranch = { remote = \"upstream\" }\n");
    promoted(&host, &repo);
    assert!(ref_at(&upstream, STABLE_REF).is_some(), "upstream holds it");
    assert_eq!(ref_at(&origin, STABLE_REF), None, "origin was written");
}

#[test]
fn a_fresh_repositorys_given_ledger_answers_that_no_chain_is_held() {
    let host = Host::new();
    let repo = repository(&host);
    match chain_of(host.bind(&repo, For::Status)) {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source, Source::new(Seam::Keeper, "git_ref"));
            assert_eq!(a.what, "chain of stable");
        }
        other => panic!("a fresh repository's given ledger answered {other:?}"),
    }
}

// Requirement: A host with no bindings file is bound to the given
// profiles

/// The store's name for the given layout under the host's data
/// directory.
fn given_store(host: &Host) -> Source {
    Source::new(
        Seam::Store,
        format!("layout {}", host.given_layout().display()),
    )
}

/// The signer the given host key signs as on this host.
fn given_signer(host: &Host) -> SignerId {
    profiles::signer::given(&host.data()).unwrap().signer()
}

/// That `binding`, in `repo`, a repository with no remote and no file
/// of the landing's own, at `head`, is bound to the given profiles: the
/// given layout, a plain branch reading the repository's own refs, the
/// git ref with no remote, both signers, and the host's own publish
/// telling its gate.
fn assert_given(host: &Host, binding: &Binding, repo: &Path, head: &tailrace_seams::Revision) {
    assert_eq!(binding.store, given_store(host));
    assert_eq!(
        remote_read(host, binding.seams.landing.as_ref(), head),
        None
    );
    assert_eq!(
        landing_source(binding.seams.landing.as_ref(), repo),
        branch()
    );
    match binding.seams.keeper.chain(&stable()) {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source, Source::new(Seam::Keeper, "git_ref"));
            assert_eq!(a.what, "chain of stable");
        }
        other => panic!("the given ledger answered {other:?}"),
    }
    assert_eq!(binding.seams.signers.len(), 2);
    assert_eq!(binding.gate, Teller::Publish);
}

#[test]
fn no_file_and_an_empty_file_bind_the_given_profiles() {
    for text in [None, Some("")] {
        let host = Host::new();
        let (repo, head) = committed(&host, None);
        if let Some(text) = text {
            host.write(text);
        }
        let publish = host.bind(&repo, For::Publish).unwrap();
        assert_given(&host, &publish, &repo, &head);
        assert_eq!(publish.signer, Some(given_signer(&host)), "{text:?}");
        for command in [For::Verify, For::Sync, For::Rebuild, For::Push] {
            let binding = host.bind(&repo, command).unwrap();
            assert_given(&host, &binding, &repo, &head);
            assert_eq!(binding.signer, None, "{command:?}");
        }
        let promote = host.bind(&repo, For::Promote).unwrap();
        assert_given(&host, &promote, &repo, &head);
        assert_eq!(promote.signer, Some(given_signer(&host)));
        assert_eq!(host.file().exists(), text.is_some(), "a file was written");
        assert!(
            !host.data().join(profiles::signer::HOST_KEY).exists(),
            "binding made a key"
        );
    }
}

#[test]
fn status_with_no_file_binds_the_given_landing_and_the_git_ref_on_its_remote_and_makes_nothing() {
    let host = Host::new();
    let (repo, head) = with_remotes(&host, &["origin"]);
    let before = files(host.dir.path());
    let binding = host
        .bind(&repo, For::Status)
        .unwrap_or_else(|not| panic!("status bound nothing: {not}"));
    assert_eq!(
        remote_read(&host, binding.seams.landing.as_ref(), &head).as_deref(),
        Some("origin")
    );
    let origin = host.dir.path().join("origin.git");
    match binding.seams.keeper.chain(&stable()) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, Source::new(Seam::Keeper, "git_ref"));
            let said = u.said.unwrap_or_default();
            assert!(said.contains(&origin.display().to_string()), "{said}");
        }
        other => panic!("the given ledger answered {other:?}, not from origin"),
    }
    assert_eq!(binding.store, given_store(&host));
    match binding.seams.store.resolve(&Name::new("v0.1.0")) {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source, Source::new(Seam::Store, "unbound"));
            assert!(a.what.contains("[store] layout (given)"), "{a:?}");
        }
        other => panic!("status's given store answered {other:?}"),
    }
    assert_eq!(binding.signer, None);
    assert_eq!(binding.seams.signers.len(), 2);
    assert!(binding.seams.projections.is_empty() && binding.seams.targets.is_empty());
    assert_eq!(binding.gate, Teller::Publish);
    assert_eq!(files(host.dir.path()), before, "binding status made a file");
}

#[test]
fn a_registry_alone_binds_it_and_the_given_rest() {
    let host = Host::new();
    let (repo, head) = committed(&host, None);
    host.write("[store]\nregistry = \"localhost:5000/tailrace\"\n");
    let binding = host.bind(&repo, For::Publish).unwrap();
    assert_eq!(
        binding.store,
        Source::new(Seam::Store, "registry localhost:5000/tailrace")
    );
    assert_eq!(
        remote_read(&host, binding.seams.landing.as_ref(), &head),
        None
    );
    assert_eq!(
        landing_source(binding.seams.landing.as_ref(), &repo),
        branch()
    );
    match binding.seams.keeper.chain(&stable()) {
        Err(Not::Absent(a)) => assert_eq!(a.source, Source::new(Seam::Keeper, "git_ref")),
        other => panic!("the given ledger answered {other:?}"),
    }
    assert_eq!(binding.signer, Some(given_signer(&host)));
    assert_eq!(binding.gate, Teller::Publish);
    assert!(
        !host.data().exists(),
        "a registry host made the given layout"
    );
}

#[test]
fn the_given_layout_is_under_the_data_directory_or_the_home() {
    let host = Host::new();
    let (repo, _) = committed(&host, None);
    let binding = host.bind(&repo, For::Publish).unwrap();
    assert_eq!(binding.store, given_store(&host));
    assert!(host.given_layout().join("oci-layout").is_file());
    let home = host.home().join(".local/share/tailrace/layout");
    for data in [None, Some(OsStr::new(""))] {
        let binding = bind(&host.file(), Some(&host.home()), data, &repo, For::Publish).unwrap();
        assert_eq!(
            binding.store,
            Source::new(Seam::Store, format!("layout {}", home.display()))
        );
    }
}

#[test]
fn a_store_ledger_over_the_given_layout_holds_the_chain_there_and_never_makes_it() {
    let host = Host::new();
    let (repo, _) = committed(&host, None);
    host.write("[ledger]\nstore = {}\n");
    for command in [For::Verify, For::Status] {
        match host.bind(&repo, command) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source.seam, Seam::Store, "{command:?}");
                assert!(
                    u.what
                        .starts_with(&host.given_layout().display().to_string()),
                    "{u:?}"
                );
            }
            other => panic!(
                "{command:?} over no given layout answered {:?}",
                other.err()
            ),
        }
        assert!(!host.given_layout().exists(), "{command:?} made the layout");
    }
    let binding = host.bind(&repo, For::Publish).unwrap();
    assert_eq!(binding.store, given_store(&host));
    match binding.seams.keeper.chain(&stable()) {
        Err(Not::Absent(a)) => assert_eq!(a.source, Source::new(Seam::Keeper, "store")),
        other => panic!("the store ledger answered {other:?}"),
    }
    match chain_of(host.bind(&repo, For::Status)) {
        Err(Not::Absent(a)) => assert_eq!(a.source, Source::new(Seam::Keeper, "store")),
        other => panic!("status's store ledger answered {other:?}"),
    }
}

#[test]
fn a_malformed_file_is_unreadable_and_binds_nothing_given() {
    let host = Host::new();
    let (repo, _) = committed(&host, None);
    let text = "[store\nlayout = \"/layout\"\n";
    host.write(text);
    let words = toml::from_str::<toml::Table>(text).unwrap_err().to_string();
    for command in [For::Publish, For::Verify, For::Status] {
        match host.bind(&repo, command) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, Source::host(host.file()));
                assert_eq!(u.what, host.file().display().to_string());
                assert_eq!(u.said.as_deref(), Some(words.as_str()));
            }
            other => panic!(
                "{command:?} over a malformed file answered {:?}",
                other.err()
            ),
        }
    }
    assert!(
        !host.data().exists(),
        "a malformed file made the given layout"
    );
    assert!(!host.data().join(profiles::signer::HOST_KEY).exists());
}

#[test]
fn several_remotes_none_origin_are_unreadable_for_every_command_and_make_no_layout() {
    let host = Host::new();
    let (repo, _) = with_remotes(&host, &["github", "gitlab"]);
    let said = format!(
        "the checkout has the remotes github, gitlab, none of them origin; name one in \
         [landing] remote of {}",
        host.file().display()
    );
    for command in [For::Publish, For::Verify, For::Status] {
        match host.bind(&repo, command) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, branch(), "{command:?}");
                assert_eq!(u.said.as_deref(), Some(said.as_str()));
            }
            other => panic!("{command:?} answered {:?}", other.err()),
        }
    }
    assert!(!host.data().exists(), "an unreadable landing left a layout");
    host.write("[landing]\nbranch = { remote = \"github\" }\n");
    assert!(host.bind(&repo, For::Publish).is_ok());
}

#[test]
fn publish_makes_the_given_layout_with_its_parents_and_verify_over_none_makes_nothing() {
    let host = Host::new();
    let (repo, _) = committed(&host, None);
    let layout = host.given_layout();
    match host.bind(&repo, For::Verify) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source.seam, Seam::Store);
            assert!(u.what.starts_with(&layout.display().to_string()), "{u:?}");
        }
        other => panic!("verify over no given layout answered {:?}", other.err()),
    }
    assert!(
        !host.data().exists(),
        "verify made {}",
        host.data().display()
    );
    let binding = host.bind(&repo, For::Publish).unwrap();
    assert!(
        layout.join("oci-layout").is_file(),
        "publish made no layout"
    );
    let held = binding.seams.store.put_bytes(b"abc").unwrap();
    assert_eq!(held.digest, Digest::of(b"abc"));
    assert!(host.bind(&repo, For::Verify).is_ok());
}
