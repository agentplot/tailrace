//! Spec cargo-projection: the Cargo projection over a scratch git
//! repository holding a workspace whose one crate inherits its version,
//! cargo run under a `CARGO_HOME` and `HOME` of each test's own (design
//! D7). Needs cargo, so it is in the tools layer.

#![cfg(feature = "tools")]

mod scratch;

use std::fs;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use scratch::{on_path, Repo, Scratch};
use tailrace::profiles::projection::cargo::{CargoProjection, ECOSYSTEM};
use tailrace_fakes::FakeStore;
use tailrace_seams::{
    ArtifactMeta, BuildId, Ecosystem, EcosystemName, Output, OutputName, Projection, Store, Version,
};

/// The family crate the workspace holds.
const CRATE: &str = "fam-lib";

/// The workspace's manifest, which holds the version its crate inherits.
const WORKSPACE: &str = "[workspace]\n\
resolver = \"2\"\n\
members = [\"crates/fam\"]\n\
\n\
[workspace.package]\n\
version = \"0.0.0\" # the tag's, written at publish\n\
edition = \"2021\"\n\
license = \"MIT\"\n";

const MEMBER: &str = "[package]\n\
name = \"fam-lib\"\n\
version.workspace = true\n\
edition.workspace = true\n\
license.workspace = true\n\
description = \"a family crate\"\n";

const LIB: &str = "pub fn hello() -> &'static str {\n    \"hello\"\n}\n";

/// The workspace committed in a scratch repository, and cargo sealed
/// under a home of the test's own.
struct Workspace {
    scratch: Scratch,
    cargo: PathBuf,
}

impl Workspace {
    fn new() -> Workspace {
        let scratch = Scratch::new();
        let repo = &scratch.repo;
        repo.write("Cargo.toml", WORKSPACE);
        repo.write("crates/fam/Cargo.toml", MEMBER);
        repo.write("crates/fam/src/lib.rs", LIB);
        repo.commit("feat: the family crate");
        let cargo = sealed(&scratch.path());
        Workspace { scratch, cargo }
    }

    fn repo(&self) -> &Repo {
        &self.scratch.repo
    }

    fn projection(&self, store: Box<dyn Store>) -> CargoProjection {
        CargoProjection::new(&self.repo().path, store).cargo_program(&self.cargo)
    }

    /// The crate's output: its manifest and its source, as a build names
    /// the files a task declared.
    fn output(&self) -> Output {
        let at = |file: &str| self.repo().path.join(file).display().to_string();
        Output {
            name: OutputName::new("fam"),
            files: vec![at("crates/fam/Cargo.toml"), at("crates/fam/src/lib.rs")],
        }
    }
}

/// A script at `dir/bin/cargo` running the shell's cargo with `CARGO_HOME`
/// and `HOME` under `dir`, so nothing of the host's home is read.
fn sealed(dir: &Path) -> PathBuf {
    let home = dir.join("home");
    fs::create_dir_all(home.join("cargo")).unwrap();
    let bin = dir.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let script = bin.join("cargo");
    fs::write(
        &script,
        format!(
            "#!/bin/sh\nCARGO_HOME='{home}/cargo' HOME='{home}' exec '{cargo}' \"$@\"\n",
            home = home.display(),
            cargo = on_path("cargo").display()
        ),
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    script
}

/// The text of the crate's own `Cargo.toml`, as cargo normalized it.
fn crate_manifest(bytes: &[u8]) -> String {
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(bytes));
    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        let path = entry.path().unwrap().into_owned();
        if path.components().count() == 2 && path.ends_with("Cargo.toml") {
            let mut text = String::new();
            entry.read_to_string(&mut text).unwrap();
            return text;
        }
    }
    panic!("the crate holds no Cargo.toml")
}

// Requirement: A pack leaves the checkout as it was and refuses what it
// cannot promise

#[test]
fn a_crate_inheriting_its_version_packs_at_the_version_and_leaves_the_checkout_as_it_was() {
    let workspace = Workspace::new();
    let repo = workspace.repo();
    let packed = workspace
        .projection(Box::new(FakeStore::new()))
        .pack(
            &workspace.output(),
            &BuildId::new("b7e1"),
            Some(&Version::new("1.4.0")),
        )
        .unwrap();
    assert_eq!(
        packed.meta,
        Some(
            ArtifactMeta::new(
                Ecosystem::new(ECOSYSTEM),
                EcosystemName::new(CRATE),
                Version::new("1.4.0"),
            )
            .unwrap()
        )
    );
    assert!(packed.same_again);
    let manifest = crate_manifest(&packed.bytes);
    assert!(
        manifest.contains("name = \"fam-lib\"\nversion = \"1.4.0\"\n"),
        "{manifest}"
    );
    assert_eq!(
        fs::read_to_string(repo.path.join("Cargo.toml")).unwrap(),
        WORKSPACE
    );
    assert_eq!(
        fs::read_to_string(repo.path.join("crates/fam/Cargo.toml")).unwrap(),
        MEMBER
    );
    assert_eq!(repo.git(&["status", "--porcelain", "--ignored"]), "");
}

#[test]
fn a_version_cargo_refuses_is_unreadable_in_cargos_words_and_the_manifest_is_written_back() {
    let workspace = Workspace::new();
    let repo = workspace.repo();
    let answer = workspace.projection(Box::new(FakeStore::new())).pack(
        &workspace.output(),
        &BuildId::new("b7e1"),
        Some(&Version::new("not.a.version")),
    );
    match answer {
        Err(tailrace_seams::Not::Unreadable(u)) => {
            assert_eq!(u.source, CargoProjection::source());
            assert!(
                u.what.contains(" package --no-verify --allow-dirty"),
                "{u:?}"
            );
            let said = u.said.unwrap();
            assert!(said.contains("error"), "{said}");
        }
        other => panic!("answered {other:?}"),
    }
    assert_eq!(
        fs::read_to_string(repo.path.join("Cargo.toml")).unwrap(),
        WORKSPACE
    );
    assert_eq!(repo.git(&["status", "--porcelain", "--ignored"]), "");
}
