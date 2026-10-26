//! Spec host-bindings: the bindings file read from a temporary
//! directory, its path and the home directory passed in, never read
//! from the process's environment.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tailrace_seams::SignerId;
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

    fn read(&self) -> Answer<Bindings> {
        read(&self.file(), Some(&self.home()))
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
fn no_file_is_absent_naming_its_path() {
    let host = Host::new();
    let file = host.file();
    assert_eq!(
        host.read(),
        Err(Not::absent(
            &Source::host(&file),
            format!("bindings file at {}", file.display())
        ))
    );
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
            "[targets] is no table of the bindings file, which binds [store], [landing], [ledger] and [signing]",
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
    let binding = bind(
        &host.file(),
        Some(&host.home()),
        host.dir.path(),
        For::Verify,
    )
    .unwrap();
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
    let binding = bind(
        &host.file(),
        Some(&host.home()),
        host.dir.path(),
        For::Publish,
    )
    .unwrap();
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
        match bind(
            &host.file(),
            Some(&host.home()),
            host.dir.path(),
            For::Verify,
        ) {
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
    match bind(
        &host.file(),
        Some(&host.home()),
        host.dir.path(),
        For::Verify,
    ) {
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
    bind(&host.file(), Some(&host.home()), host.dir.path(), command)
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
    let binding = bind(
        &host.file(),
        Some(&host.home()),
        host.dir.path(),
        For::Publish,
    )
    .unwrap();
    assert_eq!(
        binding.store,
        Source::new(
            Seam::Store,
            format!("layout {}", host.home().join("layout").display())
        )
    );
}

#[test]
fn publish_with_no_signing_is_absent_naming_it_and_makes_nothing() {
    let host = Host::new();
    let layout = host.dir.path().join("layout");
    match bound(&host, &layout, false, For::Publish) {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source.seam, Seam::Host);
            assert!(a.what.contains("[signing]"), "{a:?}");
        }
        other => panic!("answered {:?}", other.err()),
    }
    assert!(!layout.exists(), "a publish with no signer made the layout");
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

fn envelope() -> Envelope {
    Envelope::signed(
        "a type",
        vec![],
        tailrace_seams::Signature {
            signer: SignerId::new("laptop"),
            keyid: String::new(),
            sig: vec![],
        },
    )
}

#[test]
fn a_file_naming_no_ledger_binds_one_absent_naming_the_table() {
    let host = Host::new();
    let layout = host.dir.path().join("layout");
    let binding = bound(&host, &layout, true, For::Publish).unwrap();
    let keeper = &binding.seams.keeper;
    let absent = Not::absent(
        &Source::new(Seam::Keeper, "unbound"),
        format!(
            "binding of the ledger, [ledger] in {}",
            host.file().display()
        ),
    );
    assert_eq!(
        keeper.append(&ChannelName::new("stable"), &envelope(), None),
        Err(absent.clone())
    );
    assert_eq!(keeper.chain(&ChannelName::new("stable")), Err(absent));
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
    bind(&host.file(), Some(&host.home()), repo, command)
}

/// What the bound ledger answers for `stable`'s chain.
fn chain_of(binding: Answer<Binding>) -> Answer<Chain> {
    binding
        .unwrap_or_else(|not| panic!("the file bound nothing: {not}"))
        .seams
        .keeper
        .chain(&ChannelName::new("stable"))
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
fn a_promote_with_no_ledger_or_no_signing_is_absent_naming_it_before_anything_is_read() {
    let host = Host::new();
    let repo = repository(&host);
    let layout = host.dir.path().join("layout");
    match bound_with(&host, &repo, &layout, "", true, For::Promote) {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source, Source::host(host.file()));
            assert_eq!(
                a.what,
                format!(
                    "binding of the ledger, [ledger] in {}",
                    host.file().display()
                )
            );
        }
        other => panic!("a promote with no ledger answered {:?}", other.err()),
    }
    match bound_with(&host, &repo, &layout, "git_ref = {}", false, For::Promote) {
        Err(Not::Absent(a)) => assert!(a.what.contains("[signing]"), "{a:?}"),
        other => panic!("a promote with no signing answered {:?}", other.err()),
    }
    host.write("[ledger]\ngit_ref = {}\n");
    match bind(&host.file(), Some(&host.home()), &repo, For::Promote) {
        Err(Not::Absent(a)) => assert!(a.what.contains("[signing]"), "{a:?}"),
        other => panic!("a promote with no signing answered {:?}", other.err()),
    }
    assert!(
        !layout.exists(),
        "a promote that cannot bind made the layout"
    );
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
            publish.seams.keeper.chain(&ChannelName::new("stable")),
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
fn a_file_naming_no_store_or_landing_is_absent_naming_the_table() {
    let host = Host::new();
    host.write("[landing]\nbranch = {}\n");
    match bind(
        &host.file(),
        Some(&host.home()),
        host.dir.path(),
        For::Verify,
    ) {
        Err(Not::Absent(a)) => assert!(a.what.contains("[store]"), "{a:?}"),
        other => panic!("answered {:?}", other.err()),
    }
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
    match bind(
        &host.file(),
        Some(&host.home()),
        host.dir.path(),
        For::Publish,
    ) {
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

fn stable() -> ChannelName {
    ChannelName::new("stable")
}

#[test]
fn a_file_naming_a_landing_alone_binds_status_with_the_chain_and_the_store_absent_naming_them() {
    let host = Host::new();
    let repo = repository(&host);
    host.write("[landing]\nbranch = {}\n");
    let binding = bind(&host.file(), Some(&host.home()), &repo, For::Status)
        .unwrap_or_else(|not| panic!("status bound nothing: {not}"));
    assert_eq!(
        binding.seams.keeper.chain(&stable()),
        Err(Not::absent(
            &Source::new(Seam::Keeper, "unbound"),
            format!(
                "binding of the ledger, [ledger] in {}",
                host.file().display()
            ),
        ))
    );
    assert_eq!(
        binding.seams.store.resolve(&Name::new("v0.1.0")),
        Err(Not::absent(
            &Source::new(Seam::Store, "unbound"),
            format!("binding of the store, [store] in {}", host.file().display()),
        ))
    );
    assert_eq!(binding.signer, None);
    assert!(binding.seams.projections.is_empty() && binding.seams.targets.is_empty());

    host.write("[store]\nlayout = \"/layout\"\n");
    match bind(&host.file(), Some(&host.home()), &repo, For::Status) {
        Err(Not::Absent(a)) => assert!(a.what.contains("[landing]"), "{a:?}"),
        other => panic!("status with no [landing] answered {:?}", other.err()),
    }
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
    let missing = bind(&host.file(), Some(&host.home()), &repo, For::Status);
    assert!(
        matches!(&missing, Err(Not::Unreadable(u)) if u.source.seam == Seam::Store),
        "{:?}",
        missing.err()
    );
    assert!(!layout.exists(), "status made {}", layout.display());
    profiles::store::layout::Layout::create(&layout).unwrap();
    match chain_of(bind(&host.file(), Some(&host.home()), &repo, For::Status)) {
        Err(Not::Absent(a)) => assert_eq!(a.source, Source::new(Seam::Keeper, "store")),
        other => panic!("a store ledger answered {other:?}"),
    }
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

/// What the landing bound for status answers of `head` on `main`.
fn landed(binding: &Binding, head: &tailrace_seams::Revision) -> Answer<bool> {
    binding
        .seams
        .landing
        .landed(&tailrace_seams::Branch::new("main"), head)
}

// Requirement: Status is bound with nothing written

#[test]
fn no_file_in_a_clone_with_origin_binds_the_plain_branch_and_the_git_ref_read_from_origin() {
    let host = Host::new();
    let gone = host.dir.path().join("gone.git");
    let (repo, head) = committed(&host, Some(&gone));
    let before = files(host.dir.path());
    let binding = bind(&host.file(), Some(&host.home()), &repo, For::Status)
        .unwrap_or_else(|not| panic!("status bound nothing: {not}"));
    let named = gone.display().to_string();
    match landed(&binding, &head) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, Source::new(Seam::Landing, "branch"));
            assert!(
                u.said.as_deref().is_some_and(|s| s.contains(&named)),
                "{u:?}"
            );
        }
        other => panic!("the landing answered {other:?}, not from origin"),
    }
    match binding.seams.keeper.chain(&stable()) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, Source::new(Seam::Keeper, "git_ref"));
            assert!(
                u.said.as_deref().is_some_and(|s| s.contains(&named)),
                "{u:?}"
            );
        }
        other => panic!("the ledger answered {other:?}, not from origin"),
    }
    assert!(matches!(
        binding.seams.store.resolve(&Name::new("v0.1.0")),
        Err(Not::Absent(a)) if a.source == Source::new(Seam::Store, "unbound")
    ));
    assert_eq!(binding.signer, None);
    assert_eq!(files(host.dir.path()), before, "binding status made a file");
}

#[test]
fn no_file_in_a_tree_holding_sluices_file_binds_sluice() {
    let host = Host::new();
    let gone = host.dir.path().join("gone.git");
    let (repo, head) = committed(&host, Some(&gone));
    let file = repo.join(profiles::landing::SLUICE_FILE);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, "").unwrap();
    let binding = bind(&host.file(), Some(&host.home()), &repo, For::Status).unwrap();
    match landed(&binding, &head) {
        Err(Not::Unreadable(u)) => assert_eq!(u.source, Source::new(Seam::Landing, "sluice")),
        other => panic!("the landing answered {other:?}, not sluice's"),
    }
}

#[test]
fn no_file_and_no_origin_binds_the_branch_and_the_git_ref_with_no_remote() {
    let host = Host::new();
    let (repo, head) = committed(&host, None);
    let before = files(host.dir.path());
    let binding = bind(&host.file(), Some(&host.home()), &repo, For::Status).unwrap();
    assert_eq!(landed(&binding, &head), Ok(true));
    match binding.seams.keeper.chain(&stable()) {
        Err(Not::Absent(a)) => assert_eq!(a.source, Source::new(Seam::Keeper, "git_ref")),
        other => panic!("the ledger answered {other:?}"),
    }
    assert_eq!(files(host.dir.path()), before, "binding status made a file");
}

#[test]
fn the_landings_remote_is_the_files_or_origin_with_no_file_or_none() {
    let host = Host::new();
    let gone = host.dir.path().join("gone.git");
    let (repo, _) = committed(&host, Some(&gone));
    let remote = || landing_remote(&host.file(), Some(&host.home()), &repo);
    assert_eq!(remote(), Ok(Some(ORIGIN.to_string())));
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
    let landing = profiles::landing::given(&repo).unwrap();
    assert_eq!(landing_source(landing.as_ref(), &repo), sluice());
}

#[test]
fn sluices_file_only_in_the_working_tree_no_file_and_no_commit_bind_a_branch() {
    let host = Host::new();
    let repo = repository(&host);
    // No commit yet, sluice's file in the working tree alone.
    sluice_file(&repo);
    let landing = profiles::landing::given(&repo).unwrap();
    assert_eq!(landing_source(landing.as_ref(), &repo), branch());
    // A commit that holds no sluice file, with one in the working tree.
    git_in(
        &repo,
        &["commit", "-q", "--allow-empty", "-m", "feat: a start"],
    );
    let landing = profiles::landing::given(&repo).unwrap();
    assert_eq!(landing_source(landing.as_ref(), &repo), branch());
    // No file anywhere.
    fs::remove_dir_all(repo.join(".config")).unwrap();
    let landing = profiles::landing::given(&repo).unwrap();
    assert_eq!(landing_source(landing.as_ref(), &repo), branch());
}

#[test]
fn a_directory_git_cannot_open_is_unreadable_in_gits_words() {
    let host = Host::new();
    let dir = host.dir.path().join("plain");
    fs::create_dir(&dir).unwrap();
    match profiles::landing::given(&dir) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, sluice());
            let said = u.said.unwrap();
            assert!(said.starts_with("fatal: not a git repository"), "{said}");
        }
        other => panic!("no repository answered {:?}", other.err()),
    }
}
