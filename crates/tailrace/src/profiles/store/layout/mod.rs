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

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tailrace_seams::{Answer, Not, Seam, Source, Why};

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

fn path_text(path: &Path) -> String {
    path.display().to_string()
}

#[cfg(test)]
pub(super) mod tests {
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
}
