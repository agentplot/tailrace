//! Spec npm-projection, the projection's contract: the conformance
//! suite's projection group over the npm projection, its tarballs made
//! by the test and held in a fake store (design D12).
//!
//! Without the `tools` feature every scenario but the pack ones runs,
//! the projection's `PATH` holding no npm, so the unreachable one finds
//! none; with it the pack ones run with the npm the tree pins, under the
//! scratch home, over a package whose `prepack` and `prepare` scripts
//! would each leave a mark. The group renders at a place relative to the
//! working directory, so the fast test runs from a directory of its own.

mod scratch;

use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::{json, Value};
use tempfile::TempDir;
use timber::profiles::projection::npm::{NpmProjection, ECOSYSTEM};
use timber_conformance::seams::projection::{self, ProjectionRig};
use timber_conformance::{run, Rig, Scenario};
use timber_fakes::FakeStore;
use timber_seams::{
    Artifact, ArtifactMeta, Ecosystem, EcosystemName, Manifest, Output, OutputName, Pinned,
    Projection, Range, Requirement, Store, Version,
};

/// The host's serving process, which the checkout's `.npmrc` sends the
/// family's scope to.
const ORIGIN: &str = "http://127.0.0.1:7420/";

/// The scenarios that run npm's own packager.
const PACK: [&str; 1] = ["packed the same again"];

/// A gzipped tar laid out as `npm pack` lays one out, holding
/// `manifest` as its `package/package.json` and an `index.js`.
fn tarball(manifest: &Value) -> Vec<u8> {
    let mut archive = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
    for (path, bytes) in [
        (
            "package/package.json",
            serde_json::to_vec_pretty(manifest).unwrap(),
        ),
        ("package/index.js", b"module.exports = {};\n".to_vec()),
    ] {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(499_162_500);
        header.set_cksum();
        archive
            .append_data(&mut header, path, bytes.as_slice())
            .unwrap();
    }
    archive.into_inner().unwrap().finish().unwrap()
}

/// `manifest`'s package packed and put in `store`: its tarball and the
/// artifact's manifest naming it.
fn held(store: &dyn Store, manifest: &Value) -> Artifact {
    let meta = ArtifactMeta::new(
        Ecosystem::new(ECOSYSTEM),
        EcosystemName::new(manifest["name"].as_str().unwrap()),
        Version::new(manifest["version"].as_str().unwrap()),
    )
    .unwrap();
    let layer = store.put_bytes(&tarball(manifest)).unwrap().digest;
    let digest = store
        .put_artifact(&Manifest::Artifact {
            meta: meta.clone(),
            layers: vec![layer],
        })
        .unwrap()
        .digest;
    Artifact { digest, meta }
}

/// A checkout whose `.npmrc` sends `@family` to the host's serving
/// process, holding the output of a package `@family/widget` whose
/// scripts would leave their marks under `marks`; a fake store; and the
/// npm projection bound in the checkout, until it is made unreachable.
struct NpmRig {
    dir: TempDir,
    store: FakeStore,
    projection: NpmProjection,
    unreachable: NpmProjection,
    gone: Cell<bool>,
    empty: PathBuf,
}

impl NpmRig {
    fn new(marks: &Path) -> NpmRig {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join(".npmrc"),
            "@family:registry=http://127.0.0.1:7420/agentplot/consumer/npm/\n",
        )
        .unwrap();
        let package = dir.path().join("dist");
        fs::create_dir_all(&package).unwrap();
        let mark = |script: &str| format!("echo ran > {}", marks.join(script).display());
        let manifest = json!({
            "name": "@family/widget",
            "version": "0.0.0",
            "main": "index.js",
            "scripts": { "prepack": mark("prepack"), "prepare": mark("prepare") },
        });
        fs::write(
            package.join("package.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(package.join("index.js"), "module.exports = {};\n").unwrap();
        let empty = dir.path().join("bin");
        fs::create_dir_all(&empty).unwrap();
        let store = FakeStore::new();
        let bound = || NpmProjection::new(dir.path(), Box::new(store.clone()), ORIGIN);
        let projection = if cfg!(feature = "tools") {
            bound()
        } else {
            bound().with_path(&empty)
        };
        let unreachable = bound().with_path(&empty);
        NpmRig {
            dir,
            store,
            projection,
            unreachable,
            gone: Cell::new(false),
            empty,
        }
    }
}

impl Rig for NpmRig {
    fn profile(&self) -> String {
        ECOSYSTEM.into()
    }
}

impl ProjectionRig for NpmRig {
    fn projection(&self) -> &dyn Projection {
        if self.gone.get() {
            &self.unreachable
        } else {
            &self.projection
        }
    }

    fn output(&self) -> Output {
        let at = |file: &str| {
            self.dir
                .path()
                .join("dist")
                .join(file)
                .display()
                .to_string()
        };
        Output {
            name: OutputName::new("widget"),
            files: vec![at("package.json"), at("index.js")],
        }
    }

    fn artifacts(&self, n: usize) -> Vec<Artifact> {
        (0..n)
            .map(|i| {
                held(
                    &self.store,
                    &json!({ "name": "@family/widget", "version": format!("1.{i}.0") }),
                )
            })
            .collect()
    }

    fn serves(&self) -> bool {
        true
    }

    /// npm's request for a version's tarball, by the path its packument
    /// names.
    fn request(&self, meta: &ArtifactMeta) -> String {
        let name = meta.name().as_str();
        let base = name.rsplit_once('/').map_or(name, |(_, base)| base);
        format!("{name}/-/{base}-{}.tgz", meta.version())
    }

    fn caches(&self) -> bool {
        false
    }

    fn has_lockfile(&self) -> bool {
        true
    }

    /// A checkout holding a `package-lock.json` npm 7 and later write,
    /// pinning `@family/widget` under two paths and `left-pad`, where
    /// `lockfile`; an empty one otherwise.
    fn checkout(&self, lockfile: bool) -> (String, Vec<Pinned>) {
        let at = self
            .dir
            .path()
            .join(if lockfile { "locked" } else { "unlocked" });
        fs::create_dir_all(&at).unwrap();
        if !lockfile {
            return (at.display().to_string(), Vec::new());
        }
        let lock = json!({
            "name": "consumer",
            "lockfileVersion": 3,
            "requires": true,
            "packages": {
                "": { "name": "consumer", "dependencies": { "@family/widget": "^1.4.0" } },
                "node_modules/@family/widget": { "version": "1.4.0" },
                "node_modules/@family/tool/node_modules/@family/widget": { "version": "1.4.0" },
                "node_modules/left-pad": { "version": "1.3.0" },
            },
        });
        fs::write(
            at.join("package-lock.json"),
            serde_json::to_vec_pretty(&lock).unwrap(),
        )
        .unwrap();
        let pin = |name: &str, version: &str| Pinned {
            name: EcosystemName::new(name),
            version: Version::new(version),
        };
        (
            at.display().to_string(),
            vec![pin("@family/widget", "1.4.0"), pin("left-pad", "1.3.0")],
        )
    }

    fn has_manifest(&self) -> bool {
        true
    }

    /// `@family/widget` requiring `@family/base` and `left-pad`, and the
    /// peer `@family/ui`: the family's two, as its `package.json` spells
    /// them.
    fn held(&self) -> (Artifact, Vec<Requirement>) {
        let artifact = held(
            &self.store,
            &json!({
                "name": "@family/widget",
                "version": "1.4.0",
                "dependencies": { "@family/base": "^2.1.0", "left-pad": "^1.3.0" },
                "peerDependencies": { "@family/ui": "^3.0.0" },
            }),
        );
        let required = |name: &str, range: &str| Requirement {
            name: EcosystemName::new(name),
            range: Range::new(range),
        };
        (
            artifact,
            vec![
                required("@family/base", "^2.1.0"),
                required("@family/ui", "^3.0.0"),
            ],
        )
    }

    /// A caret range, and versions npm orders otherwise than their
    /// spelling, a prerelease and a major beyond it among them.
    fn ranged(&self) -> (Range, Vec<Version>, Vec<Version>) {
        let versions = |spelled: &[&str]| spelled.iter().map(|v| Version::new(*v)).collect();
        (
            Range::new("^2.1.0"),
            versions(&["2.9.0", "2.10.0", "2.3.0", "3.0.0", "2.4.0-rc.1", "2.0.9"]),
            versions(&["2.10.0", "2.9.0", "2.3.0"]),
        )
    }

    /// The projection run under a `PATH` holding no npm, and the
    /// system's words for running one there.
    fn unreachable(&self) -> String {
        self.gone.set(true);
        Command::new("npm")
            .env("PATH", &self.empty)
            .output()
            .unwrap_err()
            .to_string()
    }
}

/// The marks the package's scripts would leave.
fn left(at: &Path) -> Vec<String> {
    fs::read_dir(at)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect()
}

#[cfg(not(feature = "tools"))]
#[test]
fn the_projection_group_over_npm_but_its_pack() {
    let place = TempDir::new().unwrap();
    std::env::set_current_dir(place.path()).unwrap();
    let marks = TempDir::new().unwrap();
    let scenarios: Vec<Scenario<dyn ProjectionRig>> = projection::scenarios()
        .into_iter()
        .filter(|s| !PACK.contains(&s.name))
        .collect();
    let ran = scenarios.len();
    let report = run(projection::GROUP, &scenarios, &|| {
        Box::new(NpmRig::new(marks.path()))
    });
    assert_eq!((report.profile.as_str(), report.ran), (ECOSYSTEM, ran));
    report.assert();
    let inapplicable: Vec<(&str, &str)> = report
        .not_applicable
        .iter()
        .map(|n| (n.scenario, n.why.as_str()))
        .collect();
    assert_eq!(
        inapplicable,
        [
            (
                "a cache entry packed with no version",
                "the projection packs no cache entries"
            ),
            (
                "a layout of the build identities given",
                "the projection packs no cache entries"
            ),
            ("no manifest, none", "the ecosystem has a manifest"),
        ],
        "{report}"
    );
    assert_eq!(left(marks.path()), Vec::<String>::new());
}

#[cfg(feature = "tools")]
#[test]
fn the_projection_groups_pack_over_npm() {
    scratch::home();
    let marks = TempDir::new().unwrap();
    let scenarios: Vec<Scenario<dyn ProjectionRig>> = projection::scenarios()
        .into_iter()
        .filter(|s| PACK.contains(&s.name))
        .collect();
    let report = run(projection::GROUP, &scenarios, &|| {
        Box::new(NpmRig::new(marks.path()))
    });
    assert_eq!(
        (report.profile.as_str(), report.ran),
        (ECOSYSTEM, PACK.len())
    );
    report.assert();
    assert!(report.not_applicable.is_empty(), "{report}");
    assert_eq!(
        left(marks.path()),
        Vec::<String>::new(),
        "a script of the package ran"
    );
}

/// Every path in the tarball `bytes`, and its `package/package.json`.
#[cfg(feature = "tools")]
fn unpacked(bytes: &[u8]) -> (Vec<String>, Value) {
    use std::io::Read as _;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(bytes));
    let (mut paths, mut manifest) = (Vec::new(), Value::Null);
    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        let path = entry.path().unwrap().display().to_string();
        if path == "package/package.json" {
            let mut text = String::new();
            entry.read_to_string(&mut text).unwrap();
            manifest = serde_json::from_str(&text).unwrap();
        }
        paths.push(path);
    }
    paths.sort();
    (paths, manifest)
}

#[cfg(feature = "tools")]
#[test]
fn a_package_is_packed_at_its_version_by_npm_running_no_script_and_editing_nothing() {
    use timber_fakes::Alone;
    use timber_seams::BuildId;
    scratch::home();
    let marks = TempDir::new().unwrap();
    let rig = NpmRig::new(marks.path());
    let output = rig.output();
    let before = fs::read(&output.files[0]).unwrap();
    let packed = rig
        .projection()
        .pack(
            &output,
            &BuildId::new("b7e1"),
            Some(&Alone::new(&output, "1.4.0").at()),
        )
        .unwrap();
    let meta = packed.meta.expect("a pack at a version names its metadata");
    assert_eq!(
        (
            meta.ecosystem().as_str(),
            meta.name().as_str(),
            meta.version().as_str()
        ),
        ("npm", "@family/widget", "1.4.0")
    );
    assert!(!packed.same_again, "npm promises no same bytes");
    let (paths, manifest) = unpacked(&packed.bytes);
    assert_eq!(paths, ["package/index.js", "package/package.json"]);
    assert_eq!(manifest["name"], "@family/widget");
    assert_eq!(manifest["version"], "1.4.0");
    assert_eq!(
        fs::read(&output.files[0]).unwrap(),
        before,
        "the build's package.json was edited"
    );
    assert_eq!(
        left(marks.path()),
        Vec::<String>::new(),
        "a script of the package ran"
    );
}
