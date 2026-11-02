//! Spec cargo-projection: the Cargo projection over a scratch git
//! repository holding a workspace whose family crates inherit their
//! version and depend on each other by path, each packaging task's crate
//! file made by cargo itself, cargo run under a `CARGO_HOME` and `HOME` of
//! each test's own, the projection group run whole over it, its serve
//! scenarios asking for a version's crate as Cargo downloads one (design
//! D11). Needs cargo, so it is in the tools layer.

#![cfg(feature = "tools")]

mod scratch;

use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

use scratch::{sealed_cargo, Repo, Scratch};
use tailrace::profiles::projection::cargo::{CargoProjection, ECOSYSTEM};
use tailrace::profiles::store::layout::Layout;
use tailrace_conformance::seams::projection::{self, ProjectionRig};
use tailrace_conformance::{run, Rig};
use tailrace_core::paths_hold;
use tailrace_fakes::FakeStore;
use tailrace_seams::{
    artifact_name, Answer, Artifact, ArtifactMeta, BuildId, Component, ComponentName, Ecosystem,
    EcosystemName, Manifest, Not, Output, OutputName, PackAt, Packed, Pinned, Projection, Range,
    Requirement, Store, Version,
};

/// The family crate packed, which depends by path on its sibling.
const CRATE: &str = "fam-a";

/// The served Cargo address of the family repository, which the
/// workspace's cargo configuration names `tailrace`.
const SERVED: &str = "sparse+http://127.0.0.1:7420/agentplot/family/cargo/";

/// The workspace's manifest, which holds the version its crates inherit.
const WORKSPACE: &str = "[workspace]\n\
resolver = \"2\"\n\
members = [\"crates/*\", \"base/*\"]\n\
\n\
[workspace.package]\n\
version = \"0.0.0\" # the tag's, written at publish\n\
edition = \"2021\"\n\
license = \"MIT\"\n";

/// The cargo configuration naming the registry `tailrace`.
fn config() -> String {
    format!("[registries.tailrace]\nindex = \"{SERVED}\"\n")
}

/// `fam-a`, inheriting its version and depending by path on `fam-b`, and
/// on `base-c` of the component `base` too where `base` is given.
fn fam_a(base: bool) -> String {
    let mut manifest = "[package]\n\
name = \"fam-a\"\n\
version.workspace = true\n\
edition.workspace = true\n\
license.workspace = true\n\
description = \"a family crate\"\n\
\n\
[dependencies]\n\
fam-b = { path = \"../fam-b\", version = \"0.0.0\", registry = \"tailrace\" }\n"
        .to_string();
    if base {
        manifest.push_str("base-c = { path = \"../../base/base-c\", version = \"0.3\" }\n");
    }
    manifest
}

const FAM_B: &str = "[package]\n\
name = \"fam-b\"\n\
version.workspace = true\n\
edition.workspace = true\n\
license.workspace = true\n\
description = \"a family crate\"\n";

const BASE_C: &str = "[package]\n\
name = \"base-c\"\n\
version = \"0.3.1\"\n\
edition = \"2021\"\n\
license = \"MIT\"\n\
description = \"a crate of another component\"\n";

const LIB: &str = "pub fn hello() -> &'static str {\n    \"hello\"\n}\n";

/// The workspace committed in a scratch repository, each crate's
/// packaging task's crate file made, and cargo sealed under a home of the
/// test's own.
struct Workspace {
    scratch: Scratch,
    cargo: PathBuf,
    base: bool,
}

impl Workspace {
    /// The workspace of `fam-a` and `fam-b`, and of `base-c`, on which
    /// `fam-a` depends where `base`.
    fn new(base: bool) -> Workspace {
        let scratch = Scratch::new();
        let repo = &scratch.repo;
        repo.write("Cargo.toml", WORKSPACE);
        repo.write(".cargo/config.toml", &config());
        repo.write("crates/fam-a/Cargo.toml", &fam_a(base));
        repo.write("crates/fam-a/src/lib.rs", LIB);
        repo.write("crates/fam-b/Cargo.toml", FAM_B);
        repo.write("crates/fam-b/src/lib.rs", LIB);
        repo.write("base/base-c/Cargo.toml", BASE_C);
        repo.write("base/base-c/src/lib.rs", LIB);
        repo.commit("feat: the family crates");
        let cargo = sealed_cargo(&scratch.path());
        let workspace = Workspace {
            scratch,
            cargo,
            base,
        };
        workspace.packaging_task();
        workspace
    }

    fn repo(&self) -> &Repo {
        &self.scratch.repo
    }

    fn projection(&self, store: Box<dyn Store>) -> CargoProjection {
        CargoProjection::new(&self.repo().path, store).cargo_program(&self.cargo)
    }

    /// The build's packaging task, stood in for: every crate of the
    /// workspace packed by cargo outside the tree, as a family repository's
    /// task leaves its crate files.
    fn packaging_task(&self) {
        let ran = Command::new(&self.cargo)
            .current_dir(&self.repo().path)
            .args([
                "package",
                "--workspace",
                "--no-verify",
                "--offline",
                "--exclude-lockfile",
                "--registry",
                "tailrace",
                "--target-dir",
            ])
            .arg(self.build())
            .output()
            .unwrap();
        assert!(
            ran.status.success(),
            "the packaging task failed: {}",
            String::from_utf8_lossy(&ran.stderr)
        );
    }

    /// Where the packaging task builds, outside the tree.
    fn build(&self) -> PathBuf {
        self.scratch.path().join("build")
    }

    /// The output of `name`'s packaging task: the crate file it left.
    fn output_of(&self, name: &str) -> Output {
        let file = self.build().join(format!("package/{name}-0.0.0.crate"));
        Output {
            name: OutputName::new(name),
            files: vec![file.display().to_string()],
        }
    }

    /// The crate packed.
    fn output(&self) -> Output {
        self.output_of(CRATE)
    }

    /// The component `fam`, whose paths hold `crates/`, and the outputs
    /// it declares, `fam-a`'s and `fam-b`'s, or `fam-a`'s alone where
    /// `fam_b` is not.
    fn fam(&self, fam_b: bool) -> (Component, Vec<Output>) {
        let mut outputs = vec![self.output()];
        if fam_b {
            outputs.push(self.output_of("fam-b"));
        }
        let component = Component {
            name: ComponentName::new("fam"),
            outputs: outputs.iter().map(|o| o.name.clone()).collect(),
            paths: vec!["crates/".into()],
            prefix: "fam-v".into(),
        };
        (component, outputs)
    }

    /// `fam-a` packed at `version` by `projection`, as a publish of the
    /// component `fam` hands it.
    fn pack(&self, projection: &dyn Projection, version: &str, fam_b: bool) -> Answer<Packed> {
        let (component, outputs) = self.fam(fam_b);
        let holds = paths_hold(&component.paths).unwrap();
        projection.pack(
            &self.output(),
            &BuildId::new("b7e1"),
            Some(&PackAt {
                version: &Version::new(version),
                component: &component,
                outputs: &outputs,
                holds: &holds,
            }),
        )
    }

    /// Every manifest of the workspace holds its committed bytes, and the
    /// tree holds nothing else.
    fn as_it_was(&self) {
        let repo = self.repo();
        let read = |file: &str| fs::read_to_string(repo.path.join(file)).unwrap();
        assert_eq!(read("Cargo.toml"), WORKSPACE);
        assert_eq!(read("crates/fam-a/Cargo.toml"), fam_a(self.base));
        assert_eq!(read("crates/fam-b/Cargo.toml"), FAM_B);
        assert_eq!(read("base/base-c/Cargo.toml"), BASE_C);
        assert_eq!(repo.git(&["status", "--porcelain", "--ignored"]), "");
    }
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

/// The paths a crate holds.
fn crate_files(bytes: &[u8]) -> Vec<String> {
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(bytes));
    archive
        .entries()
        .unwrap()
        .map(|entry| entry.unwrap().path().unwrap().display().to_string())
        .collect()
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

/// The workspace, a layout store beside it, and the projection of the
/// workspace reading crates from the store.
struct CargoRig {
    workspace: Workspace,
    layout: Layout,
    projection: CargoProjection,
}

impl CargoRig {
    fn new() -> CargoRig {
        let workspace = Workspace::new(false);
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

    /// The component `fam`, declaring both crates' packaging outputs.
    fn component(&self) -> (Component, Vec<Output>) {
        self.workspace.fam(true)
    }

    /// The crate packed at `1.0.0`, `1.1.0` and on, each put in the layout
    /// store as a publish puts an artifact: its bytes, then a manifest
    /// naming them as its one layer.
    fn artifacts(&self, n: usize) -> Vec<Artifact> {
        (0..n)
            .map(|i| {
                let version = Version::new(format!("1.{i}.0"));
                let packed = self
                    .workspace
                    .pack(&self.projection, version.as_str(), true)
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

    /// Cargo reads its registry from the host's serving process (86).
    fn serves(&self) -> bool {
        true
    }

    /// Cargo's download of the crate `meta` names, as `config.json`'s
    /// `dl` spells it below the layout's base.
    fn request(&self, meta: &ArtifactMeta) -> String {
        format!("crates/{}/{}/download", meta.name(), meta.version())
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

// Requirement: A crate packed by Cargo's own packager, and a pack leaves
// the checkout as it was

/// The normalized manifest of `bytes` as a table.
fn normalized(bytes: &[u8]) -> toml::Table {
    toml::from_str(&crate_manifest(bytes)).unwrap()
}

/// The dependency `name` the normalized manifest `table` writes.
fn dependency<'a>(table: &'a toml::Table, name: &str) -> &'a toml::Table {
    table["dependencies"][name]
        .as_table()
        .unwrap_or_else(|| panic!("{name} is no dependency of {table}"))
}

#[test]
fn a_crate_and_its_sibling_pack_at_the_version_in_the_registry_and_the_checkout_is_as_it_was() {
    let workspace = Workspace::new(false);
    let packed = workspace
        .pack(
            &workspace.projection(Box::new(FakeStore::new())),
            "1.4.0",
            true,
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
    let manifest = normalized(&packed.bytes);
    assert_eq!(manifest["package"]["version"].as_str(), Some("1.4.0"));
    let fam_b = dependency(&manifest, "fam-b");
    assert_eq!(fam_b["version"].as_str(), Some("1.4.0"), "{fam_b}");
    assert_eq!(fam_b["registry-index"].as_str(), Some(SERVED), "{fam_b}");
    assert!(!fam_b.contains_key("path"), "{fam_b}");
    let files = crate_files(&packed.bytes);
    assert!(
        !files.iter().any(|f| f.ends_with("Cargo.lock")),
        "the crate holds a lockfile: {files:?}"
    );
    workspace.as_it_was();
}

#[test]
fn a_sibling_none_of_the_components_outputs_packs_is_refused_naming_it_and_the_component() {
    let workspace = Workspace::new(false);
    match workspace.pack(
        &workspace.projection(Box::new(FakeStore::new())),
        "1.4.0",
        false,
    ) {
        Err(Not::Refused(r)) => {
            let said = r.said.unwrap();
            assert!(said.starts_with("the crate fam-b, at "), "{said}");
            assert!(said.contains("of the component fam by its paths"), "{said}");
        }
        other => panic!("answered {other:?}"),
    }
    workspace.as_it_was();
}

#[test]
fn a_version_cargo_refuses_is_unreadable_in_cargos_words_and_the_manifests_are_written_back() {
    let workspace = Workspace::new(false);
    match workspace.pack(
        &workspace.projection(Box::new(FakeStore::new())),
        "not.a.version",
        true,
    ) {
        Err(Not::Unreadable(u)) => {
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
    workspace.as_it_was();
}

// Requirement: A crate of another component is depended on as the
// manifest writes

/// A layout store beside the workspace naming `base-c` at each of
/// `versions`.
fn holding(workspace: &Workspace, versions: &[&str]) -> Layout {
    let layout = Layout::create(workspace.scratch.path().join("layout")).unwrap();
    let digest = layout.put_bytes(b"base-c's index").unwrap().digest;
    for version in versions {
        let name = artifact_name(
            &Ecosystem::new(ECOSYSTEM),
            &EcosystemName::new("base-c"),
            &Version::new(*version),
        );
        layout.name(&name, &digest, None).unwrap();
    }
    layout
}

fn refused(answer: Answer<Packed>) -> String {
    match answer {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source, CargoProjection::source());
            r.said.unwrap()
        }
        other => panic!("answered {other:?}"),
    }
}

#[test]
fn another_components_crate_is_packed_at_its_requirement_the_same_whatever_the_store_holds() {
    let workspace = Workspace::new(true);
    let layout = holding(&workspace, &["0.3.1"]);
    let projection = workspace.projection(Box::new(layout.clone()));
    let packed = workspace.pack(&projection, "1.4.0", true).unwrap();
    assert!(packed.same_again);
    let manifest = normalized(&packed.bytes);
    let base_c = dependency(&manifest, "base-c");
    assert_eq!(base_c["version"].as_str(), Some("0.3"), "{base_c}");
    assert_eq!(base_c["registry-index"].as_str(), Some(SERVED), "{base_c}");
    assert!(!base_c.contains_key("path"), "{base_c}");
    workspace.as_it_was();

    let digest = layout.put_bytes(b"base-c's next index").unwrap().digest;
    let next = artifact_name(
        &Ecosystem::new(ECOSYSTEM),
        &EcosystemName::new("base-c"),
        &Version::new("0.3.2"),
    );
    layout.name(&next, &digest, None).unwrap();
    let again = workspace.pack(&projection, "1.4.0", true).unwrap();
    assert_eq!(again.bytes, packed.bytes, "the bytes moved with the store");
    workspace.as_it_was();
}

#[test]
fn another_components_crate_the_store_holds_only_outside_its_requirement_is_refused() {
    let workspace = Workspace::new(true);
    let projection = workspace.projection(Box::new(holding(&workspace, &["0.4.0"])));
    assert_eq!(
        refused(workspace.pack(&projection, "1.4.0", true)),
        "base-c of another component is required at 0.3, and the store holds it only at 0.4.0"
    );
    workspace.as_it_was();
}

#[test]
fn another_components_crate_the_store_names_at_no_version_is_refused_saying_so() {
    let workspace = Workspace::new(true);
    let projection = workspace.projection(Box::new(holding(&workspace, &[])));
    assert_eq!(
        refused(workspace.pack(&projection, "1.4.0", true)),
        "base-c of another component is required at 0.3, and the store holds no version of it"
    );
    workspace.as_it_was();
}
