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
use tailrace::profiles::store::layout::Layout;
use tailrace_conformance::seams::projection::{self, ProjectionRig};
use tailrace_conformance::{run, Rig};
use tailrace_fakes::{Alone, FakeStore};
use tailrace_seams::{
    Artifact, ArtifactMeta, BuildId, Ecosystem, EcosystemName, Manifest, Output, OutputName,
    Pinned, Projection, Range, Requirement, Store, Version,
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

/// A crate of `name` at `version` whose normalized manifest is
/// `manifest`, as `cargo package` lays one out, made in memory.
fn crate_of(name: &str, version: &str, manifest: &str) -> Vec<u8> {
    let mut archive = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::default(),
    ));
    for (file, text) in [("Cargo.toml", manifest), ("src/lib.rs", LIB)] {
        let mut header = tar::Header::new_gnu();
        header.set_size(text.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive
            .append_data(
                &mut header,
                format!("{name}-{version}/{file}"),
                text.as_bytes(),
            )
            .unwrap();
    }
    archive.into_inner().unwrap().finish().unwrap()
}

/// The served Cargo address of the family repository the rig's crates
/// depend on.
const SERVED: &str = "sparse+http://127.0.0.1:7420/agentplot/family/cargo/";

/// The workspace, a layout store beside it, and the projection of the
/// workspace reading crates from the store.
struct CargoRig {
    workspace: Workspace,
    layout: Layout,
    projection: CargoProjection,
}

impl CargoRig {
    fn new() -> CargoRig {
        let workspace = Workspace::new();
        let layout = Layout::create(workspace.scratch.path().join("layout")).unwrap();
        let projection = workspace.projection(Box::new(layout.clone()));
        CargoRig {
            workspace,
            layout,
            projection,
        }
    }
}

impl Rig for CargoRig {
    fn profile(&self) -> String {
        ECOSYSTEM.into()
    }
}

impl ProjectionRig for CargoRig {
    fn projection(&self) -> &dyn Projection {
        &self.projection
    }

    fn output(&self) -> Output {
        self.workspace.output()
    }

    /// The crate packed at `1.0.0`, `1.1.0` and on, each put in the layout
    /// store as a publish puts an artifact: its bytes, then a manifest
    /// naming them as its one layer.
    fn artifacts(&self, n: usize) -> Vec<Artifact> {
        (0..n)
            .map(|i| {
                let version = Version::new(format!("1.{i}.0"));
                let packed = self
                    .projection
                    .pack(
                        &self.output(),
                        &BuildId::new("b7e1"),
                        Some(&Alone::new(&self.output(), version.as_str()).at()),
                    )
                    .unwrap();
                let meta = packed.meta.unwrap();
                let layer = self.layout.put_bytes(&packed.bytes).unwrap().digest;
                let digest = self
                    .layout
                    .put_artifact(&Manifest::Artifact {
                        meta: meta.clone(),
                        layers: vec![layer],
                    })
                    .unwrap()
                    .digest;
                Artifact { digest, meta }
            })
            .collect()
    }

    fn serves(&self) -> bool {
        false
    }

    fn caches(&self) -> bool {
        false
    }

    fn has_lockfile(&self) -> bool {
        true
    }

    /// A consumer's checkout beside the workspace, its `Cargo.lock`
    /// pinning a family crate from the served address, a crate of
    /// crates.io and its own package, where `lockfile`.
    fn checkout(&self, lockfile: bool) -> (String, Vec<Pinned>) {
        let at = self.workspace.scratch.path().join("consumer");
        fs::create_dir_all(&at).unwrap();
        let pin = |name: &str, version: &str| Pinned {
            name: EcosystemName::new(name),
            version: Version::new(version),
        };
        let pins = if lockfile {
            fs::write(
                at.join("Cargo.lock"),
                format!(
                    "version = 4\n\n\
                     [[package]]\nname = \"consumer\"\nversion = \"0.1.0\"\n\n\
                     [[package]]\nname = \"fam-lib\"\nversion = \"1.1.0\"\n\
                     source = \"{SERVED}\"\n\n\
                     [[package]]\nname = \"serde\"\nversion = \"1.0.228\"\n\
                     source = \"registry+https://github.com/rust-lang/crates.io-index\"\n"
                ),
            )
            .unwrap();
            vec![pin("fam-lib", "1.1.0"), pin("serde", "1.0.228")]
        } else {
            Vec::new()
        };
        (at.display().to_string(), pins)
    }

    fn has_manifest(&self) -> bool {
        true
    }

    /// A crate held in the layout store requiring two family crates at the
    /// served address, beside one of crates.io and a dev dependency.
    fn held(&self) -> (Artifact, Vec<Requirement>) {
        let manifest = format!(
            "[package]\nname = \"fam-app\"\nversion = \"2.0.0\"\n\n\
             [dependencies.fam-lib]\nversion = \"^1.1\"\nregistry-index = \"{SERVED}\"\n\n\
             [dependencies.serde]\nversion = \"1\"\n\n\
             [dev-dependencies.fam-test]\nversion = \"1\"\nregistry-index = \"{SERVED}\"\n\n\
             [build-dependencies.fam-build]\nversion = \"0.3\"\n\
             registry-index = \"{SERVED}\"\n"
        );
        let bytes = crate_of("fam-app", "2.0.0", &manifest);
        let layer = self.layout.put_bytes(&bytes).unwrap().digest;
        let meta = ArtifactMeta::new(
            Ecosystem::new(ECOSYSTEM),
            EcosystemName::new("fam-app"),
            Version::new("2.0.0"),
        )
        .unwrap();
        let digest = self
            .layout
            .put_artifact(&Manifest::Artifact {
                meta: meta.clone(),
                layers: vec![layer],
            })
            .unwrap()
            .digest;
        let requirement = |name: &str, range: &str| Requirement {
            name: EcosystemName::new(name),
            range: Range::new(range),
        };
        (
            Artifact { digest, meta },
            vec![
                requirement("fam-lib", "^1.1"),
                requirement("fam-build", "0.3"),
            ],
        )
    }

    /// The range `1`, satisfied by three of the versions listed, newest
    /// first by Cargo's order, which is neither the order listed nor the
    /// spelling's.
    fn ranged(&self) -> (Range, Vec<Version>, Vec<Version>) {
        let versions = |texts: &[&str]| texts.iter().map(|v| Version::new(*v)).collect();
        (
            Range::new("1"),
            versions(&["1.2.0", "2.0.0", "1.10.0", "1.5.0-rc.1", "1.9.0"]),
            versions(&["1.10.0", "1.9.0", "1.2.0"]),
        )
    }

    /// The cargo the projection runs removed, and the operating system's
    /// words for running it.
    fn unreachable(&self) -> String {
        let cargo = &self.workspace.cargo;
        fs::remove_file(cargo).unwrap();
        std::process::Command::new(cargo)
            .spawn()
            .unwrap_err()
            .to_string()
    }
}

/// The scenarios a projection that packs no cache entries, and whose
/// ecosystem has a manifest, is not held to, by the group's own word.
const CACHE: [&str; 3] = [
    "a cache entry packed with no version",
    "a layout of the build identities given",
    "no manifest, none",
];

#[test]
fn every_scenario_of_the_projection_group_over_the_cargo_projection() {
    let scenarios = projection::scenarios();
    let report = run(projection::GROUP, &scenarios, &|| Box::new(CargoRig::new()));
    assert_eq!(
        (report.profile.as_str(), report.ran),
        (ECOSYSTEM, scenarios.len())
    );
    report.assert();
    let inapplicable: Vec<&str> = report.not_applicable.iter().map(|n| n.scenario).collect();
    assert_eq!(inapplicable, CACHE, "{report}");
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
            Some(&Alone::new(&workspace.output(), "1.4.0").at()),
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
        Some(&Alone::new(&workspace.output(), "not.a.version").at()),
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
