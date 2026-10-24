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
            "[targets] is no table of the bindings file, which binds [store], [landing] and [signing]",
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
    assert_eq!(ecosystems, ["file", "image"]);
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
