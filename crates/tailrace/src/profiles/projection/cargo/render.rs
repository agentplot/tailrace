//! render (85, 48; design D5): a Cargo local registry at the place
//! given, holding exactly the crates given. Every artifact is checked
//! before anything is written: of the ecosystem `cargo`, its manifest
//! naming one layer, the layer's bytes of that digest, and the crate's
//! own manifest naming the artifact's name and version. Then each crate
//! is written as `<name>-<version>.crate` and each name's index file
//! whole, its lines in Cargo's version order (23), each beside its place
//! and renamed onto it, and every crate and index file of a version not
//! given is removed.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use tailrace_seams::{
    Answer, Artifact, ArtifactMeta, Digest, Held, Holds, Layout, Manifest, Not, Rendered, Why,
};

use super::crate_file::{self, CrateManifest};
use super::{index, CargoProjection, ECOSYSTEM};

/// Where a local registry keeps its index, under its root.
const INDEX: &str = "index";

/// What a crate's file name ends with.
const CRATE: &str = ".crate";

/// A crate checked and ready to write: its artifact's metadata, its
/// bytes, its index line, and its version as Cargo orders versions.
struct Checked {
    meta: ArtifactMeta,
    digest: Digest,
    bytes: Vec<u8>,
    line: String,
    order: semver::Version,
}

impl CargoProjection {
    /// 85: the local registry of `artifacts` at `at`, and the versions it
    /// holds, in the order given.
    pub(super) fn render_crates(&self, artifacts: &[Artifact], at: &str) -> Answer<Rendered> {
        let mut checked: Vec<Checked> = Vec::new();
        for artifact in artifacts {
            if checked.iter().any(|c| c.digest == artifact.digest) {
                continue;
            }
            let crate_ = self.check(artifact)?;
            if let Some(other) = checked.iter().find(|c| c.meta == crate_.meta) {
                return Err(refused(format!(
                    "{} and {} are both {} {}, and a registry holds one crate of a version",
                    other.digest,
                    crate_.digest,
                    crate_.meta.name(),
                    crate_.meta.version()
                )));
            }
            checked.push(crate_);
        }
        let dir = self.under_root(at);
        write(&dir, &files(&checked))?;
        Ok(Rendered {
            layout: Layout {
                at: dir.display().to_string(),
            },
            holds: Holds::Versions(checked.into_iter().map(|c| c.meta).collect()),
        })
    }

    /// `artifact` read through the store and checked, or refused naming
    /// it; the store's own answer where it cannot be read.
    fn check(&self, artifact: &Artifact) -> Answer<Checked> {
        let digest = &artifact.digest;
        let (layer, bytes) = self.crate_bytes(artifact)?;
        let manifest = crate_file::manifest(&bytes)
            .map_err(|said| refused(format!("the crate of {digest} cannot be read: {said}")))?;
        names_artifact(&manifest, artifact)?;
        if !is_crate_name(&manifest.name) {
            return Err(refused(format!(
                "the crate of {digest} is named {}, which is no name Cargo gives a crate",
                manifest.name
            )));
        }
        let order = semver::Version::parse(&manifest.version).map_err(|e| {
            refused(format!(
                "the crate of {digest} names the version {}, which Cargo cannot read: {e}",
                manifest.version
            ))
        })?;
        let line = index::line(&manifest, layer.hex()).map_err(|said| {
            refused(format!(
                "the crate of {digest} writes no index line: {said}"
            ))
        })?;
        Ok(Checked {
            meta: artifact.meta.clone(),
            digest: digest.clone(),
            bytes,
            line,
            order,
        })
    }

    /// The bytes of `artifact`'s crate read through the store, and their
    /// digest: of the ecosystem `cargo`, its manifest naming one layer,
    /// the layer's bytes of that digest, or refused naming the artifact;
    /// the store's own answer where it cannot be read.
    pub(super) fn crate_bytes(&self, artifact: &Artifact) -> Answer<(Digest, Vec<u8>)> {
        let (digest, meta) = (&artifact.digest, &artifact.meta);
        if meta.ecosystem().as_str() != ECOSYSTEM {
            return Err(refused(format!(
                "the artifact {digest} is of the ecosystem {}, and the cargo projection renders \
                 crates",
                meta.ecosystem()
            )));
        }
        let layers = match self.store.read(digest)? {
            Held::Manifest(Manifest::Artifact { meta: held, layers }) => {
                if &held != meta {
                    return Err(refused(format!(
                        "the store's manifest at {digest} names {} {} {}, and the artifact {} {} \
                         {}",
                        held.ecosystem(),
                        held.name(),
                        held.version(),
                        meta.ecosystem(),
                        meta.name(),
                        meta.version()
                    )));
                }
                layers
            }
            _ => {
                return Err(refused(format!(
                    "the artifact {digest} holds no artifact's manifest in the store"
                )))
            }
        };
        let [layer] = layers.as_slice() else {
            return Err(refused(format!(
                "the artifact {digest} names {} layers, and a crate is one",
                layers.len()
            )));
        };
        let bytes = match self.store.read(layer)? {
            Held::Bytes(bytes) => bytes,
            Held::Manifest(_) => {
                return Err(refused(format!(
                    "the layer {layer} of {digest} holds a manifest and not a crate"
                )))
            }
        };
        let answered = Digest::of(&bytes);
        if &answered != layer {
            return Err(Not::refused(
                &CargoProjection::source(),
                Why::OtherBytes {
                    expected: layer.clone(),
                    answered,
                },
                None,
            ));
        }
        Ok((answered, bytes))
    }
}

/// Refused naming both where the crate's own `manifest` names another
/// name or version than `artifact`'s metadata.
pub(super) fn names_artifact(manifest: &CrateManifest, artifact: &Artifact) -> Answer<()> {
    let meta = &artifact.meta;
    if manifest.name == meta.name().as_str() && manifest.version == meta.version().as_str() {
        return Ok(());
    }
    Err(refused(format!(
        "the crate of {} names {} {}, and its artifact {} {}",
        artifact.digest,
        manifest.name,
        manifest.version,
        meta.name(),
        meta.version()
    )))
}

/// Refused, by this projection, for `said`.
fn refused(said: String) -> Not {
    Not::refused(&CargoProjection::source(), Why::Said, Some(said))
}

/// Whether `name` is a name Cargo gives a crate, so it names no path.
fn is_crate_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Where Cargo looks for `name`'s index file, under the index: by its
/// length and its first letters, lower-cased (Cargo's registry index
/// reference).
fn index_path(name: &str) -> PathBuf {
    let name = name.to_ascii_lowercase();
    let path = match name.len() {
        1 => format!("1/{name}"),
        2 => format!("2/{name}"),
        3 => format!("3/{}/{name}", &name[..1]),
        _ => format!("{}/{}/{name}", &name[..2], &name[2..4]),
    };
    Path::new(INDEX).join(path)
}

/// Every file the registry of `checked` holds, by its path under the
/// registry, with its bytes.
fn files(checked: &[Checked]) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut lines: BTreeMap<PathBuf, Vec<&Checked>> = BTreeMap::new();
    for crate_ in checked {
        let file = format!("{}-{}{CRATE}", crate_.meta.name(), crate_.meta.version());
        files.insert(PathBuf::from(file), crate_.bytes.clone());
        lines
            .entry(index_path(crate_.meta.name().as_str()))
            .or_default()
            .push(crate_);
    }
    for (path, mut crates) in lines {
        crates.sort_by(|a, b| a.order.cmp(&b.order));
        let text: String = crates.iter().map(|c| format!("{}\n", c.line)).collect();
        files.insert(path, text.into_bytes());
    }
    files
}

/// `files` written under `dir`, each beside its place and renamed onto
/// it, and every crate at `dir` and file under its index that `files`
/// does not name removed, with the directories that leaves empty.
fn write(dir: &Path, files: &BTreeMap<PathBuf, Vec<u8>>) -> Answer<()> {
    let index = dir.join(INDEX);
    fs::create_dir_all(&index).map_err(|e| unreadable(&index, &e))?;
    for (path, bytes) in files {
        let at = dir.join(path);
        let parent = at.parent().expect("a file is under the registry");
        fs::create_dir_all(parent).map_err(|e| unreadable(parent, &e))?;
        let name = at.file_name().expect("a file has a name").to_string_lossy();
        let partial = parent.join(format!(".{name}.{}.partial", std::process::id()));
        fs::write(&partial, bytes)
            .and_then(|()| fs::rename(&partial, &at))
            .map_err(|e| {
                let _ = fs::remove_file(&partial);
                unreadable(&at, &e)
            })?;
    }
    for entry in fs::read_dir(dir).map_err(|e| unreadable(dir, &e))? {
        let path = entry.map_err(|e| unreadable(dir, &e))?.path();
        let kept = path
            .strip_prefix(dir)
            .is_ok_and(|rel| files.contains_key(rel));
        let stale = path.is_file() && path.to_string_lossy().ends_with(CRATE) && !kept;
        if stale {
            fs::remove_file(&path).map_err(|e| unreadable(&path, &e))?;
        }
    }
    prune(dir, &index, files)?;
    Ok(())
}

/// Every file under `at`, in the index of the registry at `dir`, that
/// `files` does not name removed, and every directory left empty but the
/// index itself.
fn prune(dir: &Path, at: &Path, files: &BTreeMap<PathBuf, Vec<u8>>) -> Answer<()> {
    for entry in fs::read_dir(at).map_err(|e| unreadable(at, &e))? {
        let path = entry.map_err(|e| unreadable(at, &e))?.path();
        if path.is_dir() {
            prune(dir, &path, files)?;
            let empty = fs::read_dir(&path)
                .map_err(|e| unreadable(&path, &e))?
                .next()
                .is_none();
            if empty {
                fs::remove_dir(&path).map_err(|e| unreadable(&path, &e))?;
            }
        } else {
            let kept = path
                .strip_prefix(dir)
                .is_ok_and(|rel| files.contains_key(rel));
            if !kept {
                fs::remove_file(&path).map_err(|e| unreadable(&path, &e))?;
            }
        }
    }
    Ok(())
}

fn unreadable(path: &Path, e: &io::Error) -> Not {
    Not::unreadable(
        &CargoProjection::source(),
        path.display().to_string(),
        Some(e.to_string()),
    )
}

#[cfg(test)]
mod tests {
    use tailrace_fakes::FakeStore;
    use tailrace_seams::{Contents, Ecosystem, EcosystemName, Projection, Store, Version};
    use tempfile::TempDir;

    use super::super::fixture::crate_of;
    use super::*;
    use crate::profiles::store::layout::Layout as OciLayout;

    /// Where the tests render, under the checkout.
    const AT: &str = "layouts/cargo";

    /// A checkout and a layout store, each in its own temporary directory.
    struct Rig {
        _dir: TempDir,
        checkout: PathBuf,
        store: OciLayout,
    }

    impl Rig {
        fn new() -> Rig {
            let dir = TempDir::new().unwrap();
            let checkout = dir.path().join("checkout");
            fs::create_dir_all(&checkout).unwrap();
            let store = OciLayout::create(dir.path().join("store")).unwrap();
            Rig {
                _dir: dir,
                checkout,
                store,
            }
        }

        fn projection(&self) -> CargoProjection {
            CargoProjection::new(&self.checkout, Box::new(self.store.clone()))
        }

        fn render(&self, artifacts: &[Artifact]) -> Answer<Rendered> {
            self.projection().render(Contents::Artifacts(artifacts), AT)
        }

        fn registry(&self) -> PathBuf {
            self.checkout.join(AT)
        }
    }

    fn meta(name: &str, version: &str) -> ArtifactMeta {
        ArtifactMeta::new(
            Ecosystem::new(ECOSYSTEM),
            EcosystemName::new(name),
            Version::new(version),
        )
        .unwrap()
    }

    fn manifest_of(name: &str, version: &str) -> String {
        format!("[package]\nname = \"{name}\"\nversion = \"{version}\"\n")
    }

    /// The crate of `name` at `version` put in `store` as a publish puts
    /// an artifact, under `meta`.
    fn held_as(store: &dyn Store, name: &str, version: &str, meta: ArtifactMeta) -> Artifact {
        let bytes = crate_of(name, version, &manifest_of(name, version));
        let layer = store.put_bytes(&bytes).unwrap().digest;
        let digest = store
            .put_artifact(&Manifest::Artifact {
                meta: meta.clone(),
                layers: vec![layer],
            })
            .unwrap()
            .digest;
        Artifact { digest, meta }
    }

    fn held(store: &dyn Store, name: &str, version: &str) -> Artifact {
        held_as(store, name, version, meta(name, version))
    }

    /// Every file under `dir`, by its path under `dir`, with its bytes.
    fn files(dir: &Path) -> Vec<(String, Vec<u8>)> {
        fn walk(root: &Path, dir: &Path, found: &mut Vec<(String, Vec<u8>)>) {
            let Ok(entries) = fs::read_dir(dir) else {
                return;
            };
            for entry in entries {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk(root, &path, found);
                } else {
                    let rel = path.strip_prefix(root).unwrap().display().to_string();
                    found.push((rel, fs::read(&path).unwrap()));
                }
            }
        }
        let mut found = Vec::new();
        walk(dir, dir, &mut found);
        found.sort();
        found
    }

    fn paths(dir: &Path) -> Vec<String> {
        files(dir).into_iter().map(|(path, _)| path).collect()
    }

    /// The versions of `name`'s index file, line by line.
    fn indexed(rig: &Rig, file: &str) -> Vec<String> {
        fs::read_to_string(rig.registry().join(file))
            .unwrap()
            .lines()
            .map(|line| {
                let line: serde_json::Value = serde_json::from_str(line).unwrap();
                line["vers"].as_str().unwrap().to_string()
            })
            .collect()
    }

    fn refused_said(answer: Answer<Rendered>) -> String {
        match answer {
            Err(Not::Refused(r)) => {
                assert_eq!(r.source, CargoProjection::source());
                r.said.expect("the refusal's words")
            }
            other => panic!("answered {other:?}"),
        }
    }

    // Requirement: A local registry rendered from the store

    #[test]
    fn the_layout_holds_exactly_the_crates_given_with_lines_in_cargos_order() {
        let rig = Rig::new();
        let given = [
            held(&rig.store, "fam-lib", "1.10.0"),
            held(&rig.store, "fam-lib", "1.2.0"),
            held(&rig.store, "fam-lib", "1.2.0-pre.1"),
            held(&rig.store, "ab", "0.1.0"),
            held(&rig.store, "Fam", "3.0.0"),
        ];
        let rendered = rig.render(&given).unwrap();
        assert_eq!(rendered.layout.at, rig.registry().display().to_string());
        assert_eq!(
            rendered.holds,
            Holds::Versions(given.iter().map(|a| a.meta.clone()).collect())
        );
        assert_eq!(
            paths(&rig.registry()),
            [
                "Fam-3.0.0.crate",
                "ab-0.1.0.crate",
                "fam-lib-1.10.0.crate",
                "fam-lib-1.2.0-pre.1.crate",
                "fam-lib-1.2.0.crate",
                "index/2/ab",
                "index/3/f/fam",
                "index/fa/m-/fam-lib",
            ]
        );
        assert_eq!(
            indexed(&rig, "index/fa/m-/fam-lib"),
            ["1.2.0-pre.1", "1.2.0", "1.10.0"]
        );
        let bytes = crate_of("ab", "0.1.0", &manifest_of("ab", "0.1.0"));
        assert_eq!(
            fs::read(rig.registry().join("ab-0.1.0.crate")).unwrap(),
            bytes
        );
        let line = fs::read_to_string(rig.registry().join("index/2/ab")).unwrap();
        assert_eq!(
            line,
            format!(
                "{{\"name\":\"ab\",\"vers\":\"0.1.0\",\"deps\":[],\"cksum\":\"{}\",\
                 \"features\":{{}},\"yanked\":false}}\n",
                Digest::of(&bytes).hex()
            )
        );
    }

    #[test]
    fn a_second_render_of_the_same_crates_is_the_same_layout() {
        let rig = Rig::new();
        let given = [
            held(&rig.store, "fam-lib", "1.0.0"),
            held(&rig.store, "fam-lib", "1.1.0"),
        ];
        let first = rig.render(&given).unwrap();
        let layout = files(&rig.registry());
        let again = rig.render(&given).unwrap();
        assert_eq!(first, again);
        assert_eq!(files(&rig.registry()), layout);
    }

    #[test]
    fn a_version_no_longer_given_leaves_the_layout() {
        let rig = Rig::new();
        let one = held(&rig.store, "fam-lib", "1.0.0");
        let two = held(&rig.store, "fam-lib", "1.1.0");
        let other = held(&rig.store, "fam-app", "2.0.0");
        rig.render(&[one.clone(), two, other]).unwrap();
        let rendered = rig.render(std::slice::from_ref(&one)).unwrap();
        assert_eq!(rendered.holds, Holds::Versions(vec![one.meta]));
        assert_eq!(
            paths(&rig.registry()),
            ["fam-lib-1.0.0.crate", "index/fa/m-/fam-lib"]
        );
        assert_eq!(indexed(&rig, "index/fa/m-/fam-lib"), ["1.0.0"]);
        rig.render(&[]).unwrap();
        assert_eq!(paths(&rig.registry()), Vec::<String>::new());
        assert!(rig.registry().join("index").is_dir());
    }

    #[test]
    fn one_artifact_given_twice_is_held_once() {
        let rig = Rig::new();
        let one = held(&rig.store, "fam-lib", "1.0.0");
        let rendered = rig.render(&[one.clone(), one.clone()]).unwrap();
        assert_eq!(rendered.holds, Holds::Versions(vec![one.meta]));
    }

    // Requirement: A render refuses what it cannot hold before writing

    #[test]
    fn bytes_of_another_digest_are_refused_naming_both_and_nothing_is_written() {
        let rig = Rig::new();
        let store = FakeStore::new();
        let good = held(&store, "fam-app", "2.0.0");
        let bad = held(&store, "fam-lib", "1.0.0");
        let Held::Manifest(Manifest::Artifact { layers, .. }) = store.read(&bad.digest).unwrap()
        else {
            panic!("the artifact holds no manifest")
        };
        store.corrupt(&layers[0], b"other bytes");
        let projection = CargoProjection::new(&rig.checkout, Box::new(store));
        match projection.render(Contents::Artifacts(&[good, bad]), AT) {
            Err(Not::Refused(r)) => {
                assert_eq!(r.source, CargoProjection::source());
                assert_eq!(
                    r.why,
                    Why::OtherBytes {
                        expected: layers[0].clone(),
                        answered: Digest::of(b"other bytes"),
                    }
                );
            }
            other => panic!("answered {other:?}"),
        }
        assert_eq!(
            paths(&rig.checkout),
            Vec::<String>::new(),
            "a file was written"
        );
        assert!(!rig.registry().exists(), "the render made a directory");
    }

    #[test]
    fn a_crate_of_another_ecosystem_name_or_version_is_refused_and_nothing_is_written() {
        let rig = Rig::new();
        let site = Artifact {
            digest: Digest::of(b"site"),
            meta: ArtifactMeta::new(
                Ecosystem::new("file"),
                EcosystemName::new("site"),
                Version::new("1.0.0"),
            )
            .unwrap(),
        };
        let said = refused_said(rig.render(std::slice::from_ref(&site)));
        assert!(said.contains("is of the ecosystem file"), "{said}");

        let misnamed = held_as(&rig.store, "fam-lib", "1.0.0", meta("fam-lib", "1.0.1"));
        let said = refused_said(rig.render(std::slice::from_ref(&misnamed)));
        assert_eq!(
            said,
            format!(
                "the crate of {} names fam-lib 1.0.0, and its artifact fam-lib 1.0.1",
                misnamed.digest
            )
        );
        assert!(!rig.registry().exists(), "the render made a directory");
    }

    #[test]
    fn two_crates_of_one_version_are_refused_naming_both() {
        let rig = Rig::new();
        let first = held(&rig.store, "fam-lib", "1.0.0");
        let bytes = crate_of(
            "fam-lib",
            "1.0.0",
            "[package]\nname = \"fam-lib\"\nversion = \"1.0.0\"\ndescription = \"another\"\n",
        );
        let layer = rig.store.put_bytes(&bytes).unwrap().digest;
        let second = Artifact {
            digest: rig
                .store
                .put_artifact(&Manifest::Artifact {
                    meta: meta("fam-lib", "1.0.0"),
                    layers: vec![layer],
                })
                .unwrap()
                .digest,
            meta: meta("fam-lib", "1.0.0"),
        };
        let said = refused_said(rig.render(&[first.clone(), second.clone()]));
        assert_eq!(
            said,
            format!(
                "{} and {} are both fam-lib 1.0.0, and a registry holds one crate of a version",
                first.digest, second.digest
            )
        );
    }

    #[test]
    fn a_manifest_of_two_layers_is_refused() {
        let rig = Rig::new();
        let layer = rig.store.put_bytes(b"one").unwrap().digest;
        let other = rig.store.put_bytes(b"two").unwrap().digest;
        let artifact = Artifact {
            digest: rig
                .store
                .put_artifact(&Manifest::Artifact {
                    meta: meta("fam-lib", "1.0.0"),
                    layers: vec![layer, other],
                })
                .unwrap()
                .digest,
            meta: meta("fam-lib", "1.0.0"),
        };
        let said = refused_said(rig.render(std::slice::from_ref(&artifact)));
        assert!(
            said.ends_with("names 2 layers, and a crate is one"),
            "{said}"
        );
    }

    #[test]
    fn the_index_path_is_cargos() {
        assert_eq!(index_path("a"), Path::new("index/1/a"));
        assert_eq!(index_path("ab"), Path::new("index/2/ab"));
        assert_eq!(index_path("Abc"), Path::new("index/3/a/abc"));
        assert_eq!(
            index_path("Serde_json"),
            Path::new("index/se/rd/serde_json")
        );
        assert!(!is_crate_name("../escape") && !is_crate_name(""));
    }
}
