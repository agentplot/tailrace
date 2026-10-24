//! The store seam (B.1) over an OCI image layout on disk, read and
//! written with no server (C.2, ADR 0003): the `oci-layout` file,
//! `index.json`, and every blob at `blobs/sha256/<hex>`, as the OCI image
//! layout specification lays them out, so another tool can copy what the
//! layout holds to a registry and a copy of the directory carries every
//! artifact and attestation with it.
//!
//! A layout is made only by [`Layout::create`]. Every operation begins by
//! reading `oci-layout`, so a layout that is gone or unreadable is
//! answered unreadable with the operating system's words before anything
//! is read or written (58, 59, rule core.19), and nothing an operation
//! does makes one.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tailrace_seams::{
    canonical, Answer, Digest, Envelope, Held, Manifest, Name, Not, Put, Seam, Source, Store, Why,
};

mod blobs;
mod index;
mod oci;

use oci::{Descriptor, ENVELOPE, OCTETS, REF_NAME, SUBJECT};

/// The layout version this store reads and writes.
const LAYOUT_VERSION: &str = "1.0.0";

/// A store held in an OCI image layout at a path. Every answer names the
/// store seam and the layout by its path.
#[derive(Clone, Debug)]
pub struct Layout {
    root: PathBuf,
    source: Source,
}

#[derive(Deserialize)]
struct OciLayout {
    #[serde(rename = "imageLayoutVersion")]
    version: String,
}

impl Layout {
    fn at(root: &Path) -> Layout {
        Layout {
            root: root.to_path_buf(),
            source: Source::new(Seam::Store, format!("OCI layout {}", root.display())),
        }
    }

    /// A layout made at `path`, which is missing or an empty directory:
    /// `blobs/sha256/`, `index.json` naming no manifest, and `oci-layout`
    /// last, so a layout half made is never read as one. A directory that
    /// already holds anything is refused, naming it.
    pub fn create(path: impl AsRef<Path>) -> Answer<Layout> {
        let layout = Layout::at(path.as_ref());
        let root = &layout.root;
        fs::create_dir_all(root).map_err(|e| layout.unreadable(root, &e))?;
        let first = fs::read_dir(root)
            .and_then(|mut entries| entries.next().transpose())
            .map_err(|e| layout.unreadable(root, &e))?;
        if let Some(entry) = first {
            return Err(Not::refused(
                &layout.source,
                Why::Said,
                Some(format!(
                    "{} already holds {}, and a layout is made only in an empty directory",
                    root.display(),
                    entry.file_name().to_string_lossy()
                )),
            ));
        }
        let blobs = root.join("blobs").join(tailrace_seams::ALGORITHM);
        fs::create_dir_all(&blobs).map_err(|e| layout.unreadable(&blobs, &e))?;
        let index = layout.index_path();
        fs::write(
            &index,
            r#"{"schemaVersion":2,"mediaType":"application/vnd.oci.image.index.v1+json","manifests":[]}"#,
        )
        .map_err(|e| layout.unreadable(&index, &e))?;
        let marker = layout.marker_path();
        fs::write(
            &marker,
            format!(r#"{{"imageLayoutVersion":"{LAYOUT_VERSION}"}}"#),
        )
        .map_err(|e| layout.unreadable(&marker, &e))?;
        Ok(layout)
    }

    /// The layout at `path`, once its `oci-layout` names the version this
    /// store reads.
    pub fn open(path: impl AsRef<Path>) -> Answer<Layout> {
        let layout = Layout::at(path.as_ref());
        layout.opened()?;
        Ok(layout)
    }

    /// The directory the layout is held in.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The store seam and this profile, naming the layout by its path.
    pub fn source(&self) -> &Source {
        &self.source
    }

    /// What every operation begins with: `oci-layout` opened and read,
    /// naming the version this store reads; the open file is what a
    /// change to `index.json` locks (design D5).
    fn opened(&self) -> Answer<File> {
        let marker = self.marker_path();
        let mut file = File::open(&marker).map_err(|e| self.unreadable(&marker, &e))?;
        let mut text = String::new();
        file.read_to_string(&mut text)
            .map_err(|e| self.unreadable(&marker, &e))?;
        let read: OciLayout = serde_json::from_str(&text)
            .map_err(|e| Not::unreadable(&self.source, path_text(&marker), Some(e.to_string())))?;
        if read.version != LAYOUT_VERSION {
            return Err(Not::unreadable(
                &self.source,
                path_text(&marker),
                Some(format!(
                    "imageLayoutVersion is \"{}\", and this store reads \"{LAYOUT_VERSION}\"",
                    read.version
                )),
            ));
        }
        Ok(file)
    }

    fn marker_path(&self) -> PathBuf {
        self.root.join("oci-layout")
    }

    fn index_path(&self) -> PathBuf {
        self.root.join("index.json")
    }

    /// `path` could not be read or written, in the operating system's
    /// words.
    fn unreadable(&self, path: &Path, error: &io::Error) -> Not {
        Not::unreadable(&self.source, path_text(path), Some(error.to_string()))
    }
}

impl Store for Layout {
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        self.opened()?;
        self.write_blob(bytes)
    }

    /// The manifest's blobs first, its config and then itself, and its
    /// entry in `index.json` last; held already when that entry is there
    /// (5, 67, design D4, D5).
    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        let marker = self.opened()?;
        let written = oci::write(manifest, &self.source, &|d| self.blob_size(d))?;
        if let Some(config) = &written.config {
            self.write_blob(config)?;
        }
        let digest = self.write_blob(&written.bytes)?.digest;
        let entry = Descriptor {
            media_type: written.media_type.into(),
            digest: digest.clone(),
            size: written.bytes.len() as u64,
            artifact_type: Some(written.artifact_type),
            annotations: BTreeMap::new(),
        };
        let held_already = self.change_index(&marker, |index| {
            let held = index.manifest(&entry.digest).is_some();
            if !held {
                index.add(entry);
            }
            Ok(held)
        })?;
        Ok(Put {
            digest,
            held_already,
        })
    }

    /// The envelope's canonical JSON held under its own digest, and one
    /// entry in `index.json` naming it, annotated with `subject`; absent
    /// when `subject` is not held, since an attestation is never kept
    /// where its digest is not (8, 68, rule core.27).
    fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
        let marker = self.opened()?;
        self.blob_size(subject)?;
        let bytes = canonical(attestation);
        let digest = self.write_blob(&bytes)?.digest;
        let held_already = self.change_index(&marker, |index| {
            let held = index.attestations(subject).any(|d| d.digest == digest);
            if !held {
                index.add(Descriptor {
                    media_type: ENVELOPE.into(),
                    digest: digest.clone(),
                    size: bytes.len() as u64,
                    artifact_type: None,
                    annotations: BTreeMap::from([(SUBJECT.into(), subject.to_string())]),
                });
            }
            Ok(held)
        })?;
        Ok(Put {
            digest,
            held_already,
        })
    }

    /// Every envelope whose entry names `digest` as its subject, in the
    /// order of `index.json`, which is the order attached (69).
    fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>> {
        self.opened()?;
        self.blob_size(digest)?;
        let index = self.read_index()?;
        index
            .attestations(digest)
            .map(|entry| {
                let bytes = self.read_blob(&entry.digest)?;
                serde_json::from_slice(&bytes).map_err(|e| {
                    Not::unreadable(
                        &self.source,
                        path_text(&self.blob_path(&entry.digest)),
                        Some(e.to_string()),
                    )
                })
            })
            .collect()
    }

    /// `name`'s one entry in `index.json` moved to `to`, only when it is
    /// at `expected`, read under the lock the change is written under, so
    /// two writers cannot both move it; refused naming where it is
    /// otherwise, with nothing written and nothing retried (70, rule
    /// core.6). The entry carries the media type and size of the manifest
    /// `to` names, or of its bytes.
    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
        let marker = self.opened()?;
        let size = self.blob_size(to)?;
        self.change_index(&marker, |index| {
            let at = index.named(name.as_str()).map(|d| d.digest.clone());
            if at.as_ref() != expected {
                return Err(Not::refused(&self.source, Why::NameAt { at }, None));
            }
            let mut entry = index.manifest(to).cloned().unwrap_or_else(|| Descriptor {
                media_type: OCTETS.into(),
                digest: to.clone(),
                size,
                artifact_type: None,
                annotations: BTreeMap::new(),
            });
            entry.annotations.insert(REF_NAME.into(), name.to_string());
            index.set_name(name.as_str(), entry);
            Ok(())
        })
    }

    /// The digest `name`'s entry names (71).
    fn resolve(&self, name: &Name) -> Answer<Digest> {
        self.opened()?;
        self.read_index()?
            .named(name.as_str())
            .map(|d| d.digest.clone())
            .ok_or_else(|| Not::absent(&self.source, format!("name {name}")))
    }

    /// The manifest put-artifact held under `digest`, read by its entry's
    /// media type, its metadata from its config and never from its
    /// artifact type; any other blob's bytes (72, design D4, D5).
    fn read(&self, digest: &Digest) -> Answer<Held> {
        self.opened()?;
        let bytes = self.read_blob(digest)?;
        let index = self.read_index()?;
        let Some(entry) = index.manifest(digest) else {
            return Ok(Held::Bytes(bytes));
        };
        let none = |digest: &Digest, words: String| {
            Not::unreadable(
                &self.source,
                path_text(&self.blob_path(digest)),
                Some(words),
            )
        };
        let manifest = match oci::read(&entry.media_type, &bytes).map_err(|w| none(digest, w))? {
            oci::Read::Index(index) => Manifest::Index(index),
            oci::Read::Artifact { config, layers } => Manifest::Artifact {
                meta: oci::meta(&self.read_blob(&config)?).map_err(|w| none(&config, w))?,
                layers,
            },
        };
        Ok(Held::Manifest(manifest))
    }
}

fn path_text(path: &Path) -> String {
    path.display().to_string()
}

#[cfg(test)]
pub(super) mod tests {
    use tailrace_seams::{
        Artifact, ArtifactMeta, ComponentName, Ecosystem, EcosystemName, IndexMeta, Version,
    };
    use tempfile::TempDir;

    use super::*;

    /// A layout made in a new temporary directory.
    pub(crate) fn fresh() -> (TempDir, Layout) {
        let dir = TempDir::new().expect("a temporary directory");
        let layout = Layout::create(dir.path().join("layout")).expect("a layout made");
        (dir, layout)
    }

    fn unreadable(answer: Answer<Layout>) -> tailrace_seams::Unreadable {
        match answer {
            Err(Not::Unreadable(u)) => u,
            other => panic!("answered {other:?}, not unreadable"),
        }
    }

    #[test]
    fn a_layout_made_holds_what_the_specification_names() {
        let (_dir, layout) = fresh();
        let marker: serde_json::Value =
            serde_json::from_slice(&fs::read(layout.root().join("oci-layout")).unwrap()).unwrap();
        assert_eq!(marker["imageLayoutVersion"], "1.0.0");
        let index: serde_json::Value =
            serde_json::from_slice(&fs::read(layout.root().join("index.json")).unwrap()).unwrap();
        assert_eq!(index["schemaVersion"], 2);
        assert_eq!(
            index["mediaType"],
            "application/vnd.oci.image.index.v1+json"
        );
        assert_eq!(index["manifests"], serde_json::json!([]));
        let blobs = layout.root().join("blobs/sha256");
        assert!(blobs.is_dir());
        assert_eq!(fs::read_dir(blobs).unwrap().count(), 0);
        assert_eq!(
            layout.source(),
            &Source::new(
                Seam::Store,
                format!("OCI layout {}", layout.root().display())
            )
        );
        Layout::open(layout.root()).expect("the layout made opens");
    }

    #[test]
    fn a_layout_is_made_in_an_empty_directory_only() {
        let dir = TempDir::new().unwrap();
        Layout::create(dir.path()).expect("made in an empty directory");

        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("notes.txt"), b"mine").unwrap();
        match Layout::create(dir.path()) {
            Err(Not::Refused(r)) => {
                assert_eq!(r.why, Why::Said);
                let said = r.said.unwrap();
                assert!(said.contains("notes.txt"), "{said}");
                assert!(said.contains(&dir.path().display().to_string()), "{said}");
            }
            other => panic!("a directory holding a file answered {other:?}"),
        }
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn a_layout_removed_is_unreadable_in_the_operating_systems_words() {
        let (_dir, layout) = fresh();
        fs::remove_dir_all(layout.root()).unwrap();
        let marker = layout.root().join("oci-layout");
        let words = File::open(&marker).unwrap_err().to_string();
        let u = unreadable(Layout::open(layout.root()));
        assert_eq!(u.source, *layout.source());
        assert_eq!(u.what, marker.display().to_string());
        assert_eq!(u.said.as_deref(), Some(words.as_str()));
        assert!(!layout.root().exists(), "opening made the layout");
    }

    #[test]
    fn an_oci_layout_of_another_version_or_not_json_is_unreadable_naming_it() {
        let (_dir, layout) = fresh();
        let marker = layout.root().join("oci-layout");
        fs::write(&marker, r#"{"imageLayoutVersion":"2.0.0"}"#).unwrap();
        let u = unreadable(Layout::open(layout.root()));
        assert_eq!(u.what, marker.display().to_string());
        assert!(u.said.as_deref().unwrap().contains("2.0.0"), "{u:?}");

        fs::write(&marker, "not json").unwrap();
        let u = unreadable(Layout::open(layout.root()));
        assert_eq!(u.what, marker.display().to_string());
        let words = serde_json::from_str::<serde_json::Value>("not json")
            .unwrap_err()
            .to_string();
        assert_eq!(u.said.as_deref(), Some(words.as_str()));
    }

    fn meta(name: &str) -> ArtifactMeta {
        ArtifactMeta::new(
            Ecosystem::new("npm"),
            EcosystemName::new(name),
            Version::new("1.3.0"),
        )
        .unwrap()
    }

    /// The blobs a layout holds, and its `index.json` entries.
    fn held(layout: &Layout) -> (usize, serde_json::Value) {
        let blobs = fs::read_dir(layout.root().join("blobs/sha256"))
            .unwrap()
            .count();
        let index: serde_json::Value =
            serde_json::from_slice(&fs::read(layout.root().join("index.json")).unwrap()).unwrap();
        (blobs, index["manifests"].clone())
    }

    #[test]
    fn an_artifact_reads_back_as_its_metadata_and_layers_in_order() {
        let (_dir, layout) = fresh();
        let layers = vec![
            layout.put_bytes(b"two").unwrap().digest,
            layout.put_bytes(b"one").unwrap().digest,
        ];
        let manifest = Manifest::Artifact {
            meta: meta("@acme/site"),
            layers,
        };
        let put = layout.put_artifact(&manifest).unwrap();
        assert!(!put.held_already);
        assert_eq!(
            layout.read(&put.digest),
            Ok(Held::Manifest(manifest.clone()))
        );
        let again = layout.put_artifact(&manifest).unwrap();
        assert_eq!(again.digest, put.digest);
        assert!(again.held_already);
        assert_eq!(held(&layout).1.as_array().unwrap().len(), 1);
    }

    #[test]
    fn an_index_reads_back_as_its_component_version_and_artifacts_in_order() {
        let (_dir, layout) = fresh();
        let artifacts: Vec<Artifact> = ["web", "cli"]
            .into_iter()
            .map(|name| {
                let layer = layout.put_bytes(name.as_bytes()).unwrap().digest;
                let manifest = Manifest::Artifact {
                    meta: meta(name),
                    layers: vec![layer],
                };
                Artifact {
                    digest: layout.put_artifact(&manifest).unwrap().digest,
                    meta: meta(name),
                }
            })
            .collect();
        let index = Manifest::Index(
            IndexMeta::new(ComponentName::new("acme"), Version::new("1.3.0"), artifacts).unwrap(),
        );
        let put = layout.put_artifact(&index).unwrap();
        assert_eq!(layout.read(&put.digest), Ok(Held::Manifest(index)));
    }

    #[test]
    fn a_manifest_naming_a_digest_not_held_is_absent_and_writes_nothing() {
        let (_dir, layout) = fresh();
        let held_layer = layout.put_bytes(b"one").unwrap().digest;
        let missing = Digest::of(b"never put");
        let before = held(&layout);
        let artifact = Manifest::Artifact {
            meta: meta("site"),
            layers: vec![held_layer, missing.clone()],
        };
        let index = Manifest::Index(
            IndexMeta::new(
                ComponentName::new("acme"),
                Version::new("1.3.0"),
                vec![Artifact {
                    digest: missing.clone(),
                    meta: meta("site"),
                }],
            )
            .unwrap(),
        );
        for manifest in [artifact, index] {
            assert_eq!(
                layout.put_artifact(&manifest),
                Err(Not::absent(layout.source(), missing.to_string()))
            );
            assert_eq!(held(&layout), before);
        }
    }

    #[test]
    fn an_artifacts_config_reads_back_as_bytes() {
        let (_dir, layout) = fresh();
        let manifest = Manifest::Artifact {
            meta: meta("site"),
            layers: vec![],
        };
        layout.put_artifact(&manifest).unwrap();
        let config = tailrace_seams::canonical(&meta("site"));
        assert_eq!(layout.read(&Digest::of(&config)), Ok(Held::Bytes(config)));
    }

    fn envelope(says: &str) -> Envelope {
        Envelope::signed(
            tailrace_seams::IN_TOTO,
            says.as_bytes().to_vec(),
            tailrace_seams::Signature {
                signer: tailrace_seams::SignerId::new("a tester"),
                keyid: String::new(),
                sig: vec![7, 7],
            },
        )
    }

    #[test]
    fn two_attestations_answer_in_the_order_attached_byte_for_byte() {
        let (_dir, layout) = fresh();
        let subject = layout.put_bytes(b"site").unwrap().digest;
        let (first, second) = (envelope("it passed"), envelope("it passed again"));
        for attestation in [&first, &second] {
            let put = layout.attach(&subject, attestation).unwrap();
            assert_eq!(put.digest, attestation.digest());
            assert!(!put.held_already);
        }
        let claims = layout.claims_of(&subject).unwrap();
        assert_eq!(claims, [first.clone(), second.clone()]);
        assert_eq!(canonical(&claims[0]), canonical(&first));
        let entries = held(&layout).1;
        assert_eq!(entries.as_array().unwrap().len(), 2);
        for (i, attestation) in [&first, &second].into_iter().enumerate() {
            assert_eq!(entries[i]["digest"], attestation.digest().as_str());
            assert_eq!(entries[i]["mediaType"], ENVELOPE);
            assert_eq!(entries[i]["annotations"][SUBJECT], subject.as_str());
        }
    }

    #[test]
    fn the_same_attestation_attached_twice_is_held_once() {
        let (_dir, layout) = fresh();
        let subject = layout.put_bytes(b"site").unwrap().digest;
        let attestation = envelope("it passed");
        layout.attach(&subject, &attestation).unwrap();
        let before = held(&layout);
        let again = layout.attach(&subject, &attestation).unwrap();
        assert_eq!(again.digest, attestation.digest());
        assert!(again.held_already);
        assert_eq!(layout.claims_of(&subject), Ok(vec![attestation]));
        assert_eq!(held(&layout), before);
    }

    #[test]
    fn a_digest_not_held_takes_no_attestation_and_has_no_claims() {
        let (_dir, layout) = fresh();
        let missing = Digest::of(b"never put");
        let before = held(&layout);
        let absent = Not::absent(layout.source(), missing.to_string());
        assert_eq!(
            layout.attach(&missing, &envelope("nothing")),
            Err(absent.clone())
        );
        assert_eq!(layout.claims_of(&missing), Err(absent));
        assert_eq!(held(&layout), before);
    }

    /// The entries of `index.json` with `name` as their ref name.
    fn entries_named(layout: &Layout, name: &str) -> Vec<serde_json::Value> {
        held(layout)
            .1
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["annotations"][REF_NAME] == name)
            .cloned()
            .collect()
    }

    fn refused_at(answer: Answer<()>) -> Option<Digest> {
        match answer {
            Err(Not::Refused(r)) => match r.why {
                Why::NameAt { at } => at,
                why => panic!("refused for {why:?}"),
            },
            other => panic!("answered {other:?}, not refused"),
        }
    }

    #[test]
    fn a_name_made_and_moved_is_held_in_one_entry() {
        let (_dir, layout) = fresh();
        let name = Name::new("site-v1");
        let bytes = layout.put_bytes(b"site").unwrap().digest;
        let artifact = layout
            .put_artifact(&Manifest::Artifact {
                meta: meta("site"),
                layers: vec![bytes.clone()],
            })
            .unwrap()
            .digest;
        layout.name(&name, &bytes, None).unwrap();
        assert_eq!(layout.resolve(&name), Ok(bytes.clone()));
        let entries = entries_named(&layout, "site-v1");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["mediaType"], OCTETS);
        assert_eq!(entries[0]["size"], 4);

        layout.name(&name, &artifact, Some(&bytes)).unwrap();
        assert_eq!(layout.resolve(&name), Ok(artifact.clone()));
        let entries = entries_named(&layout, "site-v1");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["digest"], artifact.as_str());
        assert_eq!(entries[0]["mediaType"], oci::IMAGE_MANIFEST);
        assert_eq!(
            entries[0]["artifactType"],
            "application/vnd.agentplot.tailrace.artifact.npm"
        );
    }

    #[test]
    fn a_name_moved_from_where_it_is_not_is_refused_naming_where_it_is() {
        let (_dir, layout) = fresh();
        let name = Name::new("site-v1");
        let [at, to, elsewhere] =
            [b"at" as &[u8], b"to", b"elsewhere"].map(|b| layout.put_bytes(b).unwrap().digest);
        layout.name(&name, &at, None).unwrap();
        let before = held(&layout);
        assert_eq!(
            refused_at(layout.name(&name, &to, Some(&elsewhere))),
            Some(at.clone())
        );
        assert_eq!(refused_at(layout.name(&name, &to, None)), Some(at.clone()));
        assert_eq!(
            refused_at(layout.name(&Name::new("never-made"), &to, Some(&at))),
            None
        );
        assert_eq!(held(&layout), before);
        assert_eq!(layout.resolve(&name), Ok(at));
    }

    #[test]
    fn a_name_moved_to_a_digest_not_held_is_absent_and_stays() {
        let (_dir, layout) = fresh();
        let name = Name::new("site-v1");
        let at = layout.put_bytes(b"at").unwrap().digest;
        layout.name(&name, &at, None).unwrap();
        let missing = Digest::of(b"never put");
        assert_eq!(
            layout.name(&name, &missing, Some(&at)),
            Err(Not::absent(layout.source(), missing.to_string()))
        );
        assert_eq!(layout.resolve(&name), Ok(at));
    }

    #[test]
    fn a_name_never_made_is_absent_naming_it() {
        let (_dir, layout) = fresh();
        assert_eq!(
            layout.resolve(&Name::new("never-made")),
            Err(Not::absent(layout.source(), "name never-made"))
        );
    }
}
