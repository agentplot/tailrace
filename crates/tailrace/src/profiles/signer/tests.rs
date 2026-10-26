//! Spec host-bindings: `[signing]` read by the signer profiles' family,
//! a host key or a KMS key, and the order every signer profile is bound
//! in, none of it reading a file or reaching a service.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use serde_json::json;
use tailrace_seams::{Answer, Claim, ClaimKind, Digest, Not, Revision, Seam, Source};
use tempfile::TempDir;

use super::*;
use crate::bindings::{self, For};
use crate::profiles::store::layout::Layout;

const ARN: &str = "arn:aws:kms:us-west-2:111122223333:key/1234abcd-12ab-34cd-56ef-1234567890ab";

fn file() -> PathBuf {
    PathBuf::from("/config/tailrace/config.toml")
}

/// `[signing]` as `text` holds it, read by the family.
fn signing(text: &str) -> Answer<Signing> {
    let table: toml::Table = toml::from_str(text).unwrap();
    read(&file(), Some(Path::new("/home/a")), &table)
}

/// The words of an unreadable `[signing]`, once it names the file and
/// `at`.
fn unreadable(text: &str, at: &str) -> String {
    match signing(text) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, Source::host(file()));
            assert_eq!(u.what, format!("{}, {at}", file().display()));
            u.said.expect("the words it was read with")
        }
        other => panic!("{text} read as {other:?}, not unreadable"),
    }
}

fn key() -> KeyArn {
    KeyArn::parse(ARN).unwrap()
}

#[test]
fn kms_alone_reads_as_the_key_service_signer_named_by_its_arn() {
    let read = signing(&format!("kms = \"{ARN}\"")).unwrap();
    assert_eq!(read, Signing::Kms(key()));
    assert_eq!(read.signer(), SignerId::new(ARN));
}

#[test]
fn a_host_key_reads_as_before() {
    let read = signing("signer = \"laptop\"\nkey = \"~/.config/tailrace/signing\"").unwrap();
    assert_eq!(
        read,
        Signing::HostKey {
            signer: SignerId::new("laptop"),
            key: PathBuf::from("/home/a/.config/tailrace/signing"),
        }
    );
    assert_eq!(read.signer(), SignerId::new("laptop"));
}

#[test]
fn kms_beside_a_host_keys_signer_or_key_is_unreadable_naming_the_table() {
    for (text, others) in [
        (format!("kms = \"{ARN}\"\nsigner = \"laptop\""), "signer"),
        (format!("kms = \"{ARN}\"\nkey = \"/keys/laptop\""), "key"),
        (
            format!("kms = \"{ARN}\"\nsigner = \"laptop\"\nkey = \"/keys/laptop\""),
            "key and signer",
        ),
    ] {
        assert_eq!(
            unreadable(&text, "[signing]"),
            format!(
                "[signing] names kms with {others}, and a key service signer's name is its \
                 key's: the table names kms alone"
            )
        );
    }
}

#[test]
fn kms_naming_an_alias_or_no_key_arn_is_unreadable_naming_the_value() {
    let alias = "arn:aws:kms:us-west-2:111122223333:alias/fleet";
    let said = unreadable(&format!("kms = \"{alias}\""), "[signing] kms");
    assert!(said.starts_with(alias), "{said}");
    assert!(said.contains("a key is named by its key ARN"), "{said}");
    let said = unreadable("kms = \"alias/fleet\"", "[signing] kms");
    assert!(said.starts_with("alias/fleet is no KMS key ARN"), "{said}");
    assert_eq!(
        unreadable("kms = 7", "[signing] kms"),
        "[signing] kms is not a string"
    );
}

#[test]
fn the_profile_signing_is_bound_first_and_the_other_verify_only() {
    let host_key = Signing::HostKey {
        signer: SignerId::new("laptop"),
        key: PathBuf::from("/no/such/key"),
    };
    assert_eq!(
        plan(Some(&host_key)),
        [
            Bound::HostKey(HostKey::new(SignerId::new("laptop"), "/no/such/key")),
            Bound::Kms(None),
        ]
    );
    assert_eq!(
        plan(Some(&Signing::Kms(key()))),
        [
            Bound::Kms(Some(key())),
            Bound::HostKey(HostKey::verifying()),
        ]
    );
    assert_eq!(
        plan(None),
        [Bound::HostKey(HostKey::verifying()), Bound::Kms(None)]
    );
    assert_eq!(bind(Some(&host_key)).len(), 2);
    assert_eq!(bind(None).len(), 2);
}

#[test]
fn a_key_service_host_is_bound_for_publish_signing_as_the_arn() {
    let dir = TempDir::new().unwrap();
    let file = bindings::path(Some(dir.path().join("config").as_os_str()), None).unwrap();
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(
        &file,
        format!(
            "[store]\nlayout = \"{}\"\n\n[landing]\nbranch = {{}}\n\n[signing]\nkms = \"{ARN}\"\n",
            dir.path().join("layout").display()
        ),
    )
    .unwrap();
    let binding = bindings::bind(&file, None, None, dir.path(), For::Publish).unwrap();
    assert_eq!(binding.signer, Some(SignerId::new(ARN)));
    let claim = Claim {
        subject: Digest::of(b"an artifact"),
        kind: ClaimKind::Provenance,
        revision: Revision::new("4f2a9c1"),
        build: None,
        body: json!({}),
    };
    // Asked as another identity, each answers absent, reaching nothing,
    // and names itself: the KMS key first, then the verify-only host key.
    let [first, second] = binding.seams.signers.as_slice() else {
        panic!("{} signers are bound", binding.seams.signers.len())
    };
    for (signer, source) in [(first, Kms::source()), (second, HostKey::source())] {
        match signer.sign(&claim, &SignerId::new("laptop")) {
            Err(Not::Absent(a)) => {
                assert_eq!(a.source, source);
                assert_eq!(a.source.seam, Seam::Signer);
            }
            other => panic!("{source} answered {other:?}"),
        }
    }
}

// Requirement: The given host key signs as the host's name

/// The words of an unreadable answer from the host key profile, once it
/// names `what`.
fn unreadable_naming(answer: Answer<SignerId>, what: &str) -> String {
    match answer {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, HostKey::source());
            assert_eq!(u.what, what);
            u.said.expect("the words it was read with")
        }
        other => panic!("answered {other:?}, not unreadable naming {what}"),
    }
}

#[test]
fn a_host_signs_as_its_first_label_in_lower_case() {
    assert_eq!(host_signer("Laptop.local"), Ok(SignerId::new("laptop")));
    assert_eq!(host_signer("desk"), Ok(SignerId::new("desk")));
    assert_eq!(
        host_signer("Build-7.ci.example.com"),
        Ok(SignerId::new("build-7"))
    );
    for empty in ["", ".local"] {
        let said = unreadable_naming(host_signer(empty), "the host's name");
        assert!(said.contains("has no first label"), "{said}");
    }
}

#[test]
fn the_public_halfs_name_wins_over_the_hosts() {
    let dir = TempDir::new().unwrap();
    let public = dir.path().join("signing.pub");
    let key =
        ssh_key::PrivateKey::random(&mut ssh_key::rand_core::OsRng, ssh_key::Algorithm::Ed25519)
            .unwrap();
    let line = key.public_key().to_openssh().unwrap();
    fs::write(&public, format!("{line} laptop\n")).unwrap();
    assert_eq!(given_signer(&public), Ok(SignerId::new("laptop")));
    fs::remove_file(&public).unwrap();
    // With no public half, the host's own name as it is now.
    let host = given_signer(&public).unwrap();
    assert!(!host.as_str().is_empty());
    assert!(!host.as_str().contains('.'), "{host}");
    assert_eq!(host.as_str(), host.as_str().to_lowercase());
}

#[test]
fn a_public_half_with_no_key_line_or_no_signer_is_unreadable_naming_it() {
    let dir = TempDir::new().unwrap();
    let public = dir.path().join("signing.pub");
    let what = format!("the public key file {}", public.display());
    fs::write(&public, "\n").unwrap();
    assert_eq!(
        unreadable_naming(given_signer(&public), &what),
        format!("{} holds no key line", public.display())
    );
    fs::write(&public, "no key here\n").unwrap();
    let words = ssh_key::PublicKey::from_openssh("no key here")
        .unwrap_err()
        .to_string();
    assert_eq!(unreadable_naming(given_signer(&public), &what), words);
    let key =
        ssh_key::PrivateKey::random(&mut ssh_key::rand_core::OsRng, ssh_key::Algorithm::Ed25519)
            .unwrap();
    fs::write(
        &public,
        format!("{}\n", key.public_key().to_openssh().unwrap()),
    )
    .unwrap();
    assert_eq!(
        unreadable_naming(given_signer(&public), &what),
        format!(
            "{}'s key line names no signer after its key",
            public.display()
        )
    );
}

// Requirement: Every host binds every signer profile for verify

/// A bindings file under `dir` naming a layout made there and a branch,
/// and no `[signing]`; its path.
fn unsigned_file(dir: &Path) -> PathBuf {
    let file = bindings::path(Some(dir.join("config").as_os_str()), None).unwrap();
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    let layout = dir.join("layout");
    Layout::create(&layout).unwrap();
    fs::write(
        &file,
        format!(
            "[store]\nlayout = \"{}\"\n\n[landing]\nbranch = {{}}\n",
            layout.display()
        ),
    )
    .unwrap();
    file
}

/// An OpenSSH public key line naming `signer`.
fn public_line(signer: &str) -> String {
    let key =
        ssh_key::PrivateKey::random(&mut ssh_key::rand_core::OsRng, ssh_key::Algorithm::Ed25519)
            .unwrap();
    format!("{} {signer}\n", key.public_key().to_openssh().unwrap())
}

/// The names of the files in `dir`.
fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    names
}

/// The given host key's path under the data directory `data`.
fn given_key(data: &Path) -> PathBuf {
    data.join("tailrace/host/signing.key")
}

#[test]
fn a_host_naming_no_signer_publishes_with_the_given_key_in_the_data_directory_as_its_public_halfs_name(
) {
    let dir = TempDir::new().unwrap();
    let file = unsigned_file(dir.path());
    let data = dir.path().join("data");
    let key = given_key(&data);
    fs::create_dir_all(key.parent().unwrap()).unwrap();
    // A key file no read could open, so a bind that read it would say so.
    fs::write(&key, "not a key").unwrap();
    fs::set_permissions(&key, fs::Permissions::from_mode(0o000)).unwrap();
    fs::write(public_half(&key), public_line("laptop")).unwrap();
    let laptop = SignerId::new("laptop");
    for command in [For::Publish, For::Promote] {
        let binding =
            bindings::bind(&file, None, Some(data.as_os_str()), dir.path(), command).unwrap();
        assert_eq!(binding.signer, Some(laptop.clone()), "{command:?}");
        assert_eq!(binding.seams.signers.len(), 2);
    }
    let signing = given(&data).unwrap();
    assert_eq!(
        signing,
        Signing::Given {
            signer: laptop.clone(),
            key: key.clone(),
        }
    );
    assert_eq!(
        plan(Some(&signing)),
        [
            Bound::HostKey(HostKey::given(laptop, &key)),
            Bound::Kms(None),
        ]
    );
    fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(fs::read(&key).unwrap(), b"not a key");
    assert_eq!(
        names(key.parent().unwrap()),
        ["signing.key", "signing.key.pub"]
    );
    assert_eq!(names(file.parent().unwrap()), ["config.toml"]);
}

#[test]
fn with_no_public_half_the_given_signer_is_the_hosts_name_and_nothing_is_made() {
    let dir = TempDir::new().unwrap();
    let file = unsigned_file(dir.path());
    let data = dir.path().join("data");
    let host = given_signer(&public_half(&given_key(&data))).unwrap();
    let binding = bindings::bind(
        &file,
        None,
        Some(data.as_os_str()),
        dir.path(),
        For::Publish,
    )
    .unwrap();
    assert_eq!(binding.signer, Some(host));
    assert!(!data.exists(), "binding made {}", data.display());
    assert_eq!(names(file.parent().unwrap()), ["config.toml"]);
}

#[test]
fn a_host_naming_no_signer_binds_both_verify_only_for_every_other_command() {
    let dir = TempDir::new().unwrap();
    let file = unsigned_file(dir.path());
    let data = dir.path().join("data");
    // A public half no read could parse, so a bind that read it would
    // say so.
    let public = public_half(&given_key(&data));
    fs::create_dir_all(public.parent().unwrap()).unwrap();
    fs::write(&public, "no key here\n").unwrap();
    for command in [For::Verify, For::Sync, For::Push, For::Rebuild] {
        let binding =
            bindings::bind(&file, None, Some(data.as_os_str()), dir.path(), command).unwrap();
        assert_eq!(binding.signer, None, "{command:?}");
        assert_eq!(binding.seams.signers.len(), 2);
    }
    assert_eq!(
        plan(None),
        [Bound::HostKey(HostKey::verifying()), Bound::Kms(None)]
    );
    assert_eq!(names(public.parent().unwrap()), ["signing.key.pub"]);
    assert_eq!(names(file.parent().unwrap()), ["config.toml"]);
}

#[test]
fn with_neither_the_data_directory_nor_the_home_a_publish_naming_no_signer_is_absent() {
    let dir = TempDir::new().unwrap();
    let file = unsigned_file(dir.path());
    match bindings::bind(&file, None, None, dir.path(), For::Publish) {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source, Source::host(&file));
            assert_eq!(
                a.what,
                "binding of the signer: neither XDG_DATA_HOME nor HOME is set to place \
                 tailrace/host/signing.key"
            );
        }
        other => panic!(
            "a publish with no data directory answered {:?}",
            other.err()
        ),
    }
}

// Requirement: A table the file leaves out is bound to its given profile

#[test]
fn a_key_service_alone_signs_through_it_into_the_given_layout_and_makes_no_host_key() {
    let dir = TempDir::new().unwrap();
    let file = bindings::path(Some(dir.path().join("config").as_os_str()), None).unwrap();
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, format!("[signing]\nkms = \"{ARN}\"\n")).unwrap();
    let data = dir.path().join("data");
    let repo = dir.path().join("repo");
    let made = std::process::Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .arg(&repo)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .status()
        .unwrap();
    assert!(made.success());
    let binding = bindings::bind(&file, None, Some(data.as_os_str()), &repo, For::Publish).unwrap();
    assert_eq!(binding.signer, Some(SignerId::new(ARN)));
    assert_eq!(
        plan(Some(&Signing::Kms(key()))),
        [
            Bound::Kms(Some(key())),
            Bound::HostKey(HostKey::verifying()),
        ]
    );
    let layout = data.join("tailrace/layout");
    assert_eq!(
        binding.store,
        Source::new(Seam::Store, format!("layout {}", layout.display()))
    );
    assert!(
        layout.join("oci-layout").is_file(),
        "publish made no layout"
    );
    assert_eq!(names(file.parent().unwrap()), ["config.toml"]);
}
