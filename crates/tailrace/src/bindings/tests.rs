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
