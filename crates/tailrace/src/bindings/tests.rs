//! Spec host-bindings: the bindings file read from a temporary
//! directory, its path and the home directory passed in, never read
//! from the process's environment.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
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
        Some(Signing {
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
        bindings.signing.unwrap().key,
        host.home().join(".config/tailrace/signing")
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
fn what_the_repository_binds_and_no_ledger_are_unreadable_naming_the_table() {
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
            "ledger",
            "[ledger] names a ledger, and no ledger profile is built yet",
        ),
        (
            "targets",
            "[targets] is no table of the bindings file, which binds [store], [landing], [signing] and [trust]",
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
    let [signer] = binding.seams.signers.as_slice() else {
        panic!("{} signers are bound", binding.seams.signers.len())
    };
    match signer.sign(&claim(), &SignerId::new("laptop")) {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source.seam, Seam::Signer);
            assert!(a.what.contains("no identity is at hand"), "{a:?}");
        }
        other => panic!("the verify-only host key answered {other:?}"),
    }
}

#[test]
fn the_ledger_is_bound_to_nothing() {
    let host = Host::new();
    let layout = host.dir.path().join("layout");
    let binding = bound(&host, &layout, true, For::Publish).unwrap();
    let keeper = &binding.seams.keeper;
    let envelope = Envelope::signed(
        "a type",
        vec![],
        tailrace_seams::Signature {
            signer: SignerId::new("laptop"),
            keyid: String::new(),
            sig: vec![],
        },
    );
    let absent = Not::absent(
        &Source::new(Seam::Keeper, "unbound"),
        "binding of the ledger",
    );
    assert_eq!(
        keeper.append(&ChannelName::new("stable"), &envelope, None),
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

/// The host's key for the gate, as `[trust.gate]` names it.
const GATE_KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGt gate@box";

#[test]
fn a_signer_trusted_is_named_with_its_key_and_may_claim_a_layout_policy_alone() {
    let host = Host::new();
    let layout = host.dir.path().join("layout");
    profiles::store::layout::Layout::create(&layout).unwrap();
    host.write(&format!(
        "[store]\nlayout = \"{}\"\n\n[landing]\nbranch = {{}}\n\n[trust.gate]\nssh = \"{GATE_KEY}\"\n",
        layout.display()
    ));
    let gate = NamedSigner {
        id: SignerId::new("gate"),
        verified_by: GATE_KEY.into(),
        kinds: vec![ClaimKind::LayoutPolicy],
        checks: vec![],
    };
    assert_eq!(host.read().unwrap().trust, std::slice::from_ref(&gate));
    let binding = bind(&host.file(), Some(&host.home()), host.dir.path(), For::Sync).unwrap();
    assert_eq!(binding.trust, [gate]);
}

#[test]
fn a_file_naming_no_trust_trusts_no_signer_and_binds_as_before() {
    let host = Host::new();
    let layout = host.dir.path().join("layout");
    assert_eq!(bound(&host, &layout, true, For::Publish).unwrap().trust, []);
    assert_eq!(host.read().unwrap().trust, []);
    assert_eq!(bound(&host, &layout, true, For::Verify).unwrap().trust, []);
}

#[test]
fn a_trust_table_naming_no_key_two_or_no_string_is_unreadable_naming_it() {
    let host = Host::new();
    host.write("[trust.gate]\nssh = \"ssh-ed25519 AAAA\"\nkms = \"a key's name\"\n");
    assert_eq!(
        host.unreadable(Some("[trust.gate]")),
        "[trust.gate] names kms, ssh, and a signer is verified by one key"
    );
    host.write("[trust.gate]\n");
    assert_eq!(
        host.unreadable(Some("[trust.gate]")),
        "[trust.gate] names no key that verifies it"
    );
    host.write("[trust.gate]\nssh = 7\n");
    assert_eq!(
        host.unreadable(Some("[trust.gate]")),
        "[trust.gate] ssh is not a string"
    );
    host.write("[trust]\ngate = \"ssh-ed25519 AAAA\"\n");
    assert!(host
        .unreadable(Some("[trust.gate]"))
        .starts_with("[trust.gate] is not a table"));
}

#[test]
fn sync_on_a_broken_trust_table_is_unreadable_and_makes_nothing() {
    let host = Host::new();
    let layout = host.dir.path().join("layout");
    host.write(&format!(
        "[store]\nlayout = \"{}\"\n\n[landing]\nbranch = {{}}\n\n\
         [trust.gate]\nssh = \"ssh-ed25519 AAAA\"\nkms = \"a key's name\"\n",
        layout.display()
    ));
    match bind(&host.file(), Some(&host.home()), host.dir.path(), For::Sync) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, Source::host(host.file()));
            assert_eq!(u.what, format!("{}, [trust.gate]", host.file().display()));
        }
        other => panic!("answered {:?}", other.err()),
    }
    assert!(!layout.exists(), "sync made {}", layout.display());
}
