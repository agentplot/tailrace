//! `index.json`: one entry per manifest put, per name and per attestation
//! (design D5), read by any reader with no lock, and changed only under
//! an exclusive lock on `oci-layout`, through a temporary file renamed
//! over it, keeping every entry the change does not touch as it was.

use std::fs::{self, File};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tailrace_seams::{Answer, Digest, Not};

use super::oci::{self, Descriptor, ENVELOPE, IMAGE_INDEX, IMAGE_MANIFEST, REF_NAME, SUBJECT};
use super::{path_text, Layout};

/// `index.json` as the layout holds it: its entries as written, whoever
/// wrote them, and every other field it holds.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IndexFile {
    schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    media_type: Option<String>,
    manifests: Vec<Value>,
    #[serde(flatten)]
    rest: Map<String, Value>,
}

/// What an entry of `index.json` is, read from its annotations (design
/// D5).
#[derive(Clone, Debug, PartialEq, Eq)]
enum Entry {
    /// A manifest put-artifact held.
    Manifest(Descriptor),
    /// A name, by its ref name.
    Name(String, Descriptor),
    /// An attestation, about the digest its subject annotation names.
    Attestation(Digest, Descriptor),
    /// Another tool's: kept as it is and never answered.
    Other,
}

impl Entry {
    fn of(value: &Value) -> Entry {
        let Ok(d) = Descriptor::deserialize(value) else {
            return Entry::Other;
        };
        if let Some(subject) = d.annotations.get(SUBJECT) {
            return match Digest::parse(subject) {
                Ok(subject) if d.media_type == ENVELOPE => Entry::Attestation(subject, d),
                _ => Entry::Other,
            };
        }
        if let Some(name) = d.annotations.get(REF_NAME) {
            return Entry::Name(name.clone(), d);
        }
        let manifest = d.media_type == IMAGE_MANIFEST || d.media_type == IMAGE_INDEX;
        match &d.artifact_type {
            Some(t) if manifest && oci::is_tailraces(t) => Entry::Manifest(d),
            _ => Entry::Other,
        }
    }
}

/// `index.json` read, and what a change makes of it.
pub(super) struct Index {
    file: IndexFile,
    entries: Vec<Entry>,
    changed: bool,
}

impl Index {
    fn parse(bytes: &[u8]) -> Result<Index, String> {
        let file: IndexFile = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if file.schema_version != 2 {
            return Err(format!(
                "its schemaVersion is {}, and an image index's is 2",
                file.schema_version
            ));
        }
        if let Some(other) = file.media_type.as_deref().filter(|t| *t != IMAGE_INDEX) {
            return Err(format!(
                "its mediaType is {other}, and an image index's is {IMAGE_INDEX}"
            ));
        }
        let entries = file.manifests.iter().map(Entry::of).collect();
        Ok(Index {
            file,
            entries,
            changed: false,
        })
    }

    /// The entry of the manifest put-artifact held under `digest`.
    pub(super) fn manifest(&self, digest: &Digest) -> Option<&Descriptor> {
        self.entries.iter().find_map(|e| match e {
            Entry::Manifest(d) if &d.digest == digest => Some(d),
            _ => None,
        })
    }

    /// The entry of `name`.
    pub(super) fn named(&self, name: &str) -> Option<&Descriptor> {
        self.entries.iter().find_map(|e| match e {
            Entry::Name(n, d) if n == name => Some(d),
            _ => None,
        })
    }

    /// The entries of the attestations about `subject`, in the order
    /// attached.
    pub(super) fn attestations<'a>(
        &'a self,
        subject: &'a Digest,
    ) -> impl Iterator<Item = &'a Descriptor> + 'a {
        self.entries.iter().filter_map(move |e| match e {
            Entry::Attestation(s, d) if s == subject => Some(d),
            _ => None,
        })
    }

    /// `descriptor` added as the last entry.
    pub(super) fn add(&mut self, descriptor: Descriptor) {
        self.file.manifests.push(json(&descriptor));
        self.entries
            .push(Entry::of(self.file.manifests.last().unwrap()));
        self.changed = true;
    }

    /// `name`'s one entry made `descriptor`, in its place where it has
    /// one, else added last.
    pub(super) fn set_name(&mut self, name: &str, descriptor: Descriptor) {
        let at = self
            .entries
            .iter()
            .position(|e| matches!(e, Entry::Name(n, _) if n == name));
        match at {
            Some(i) => {
                self.file.manifests[i] = json(&descriptor);
                self.entries[i] = Entry::of(&self.file.manifests[i]);
                self.changed = true;
            }
            None => self.add(descriptor),
        }
    }
}

fn json(descriptor: &Descriptor) -> Value {
    serde_json::to_value(descriptor).expect("a descriptor serializes to JSON")
}

impl Layout {
    /// `index.json` as it is now, with no lock: a change renames a whole
    /// new file over it, so a reader sees it before or after a change and
    /// never part of one. Unreadable naming it, with the parser's words,
    /// when it holds no image index.
    pub(super) fn read_index(&self) -> Answer<Index> {
        let path = self.index_path();
        let bytes = fs::read(&path).map_err(|e| self.unreadable(&path, &e))?;
        Index::parse(&bytes)
            .map_err(|words| Not::unreadable(&self.source, path_text(&path), Some(words)))
    }

    /// `change` made to `index.json` under an exclusive lock on
    /// `marker`, the open `oci-layout`, which nothing rewrites: the index
    /// is read, changed, and written whole through a temporary file
    /// renamed over it before the lock is released. A change that answers
    /// otherwise than what was asked, or changes nothing, writes nothing.
    pub(super) fn change_index<T>(
        &self,
        marker: &File,
        change: impl FnOnce(&mut Index) -> Answer<T>,
    ) -> Answer<T> {
        marker
            .lock()
            .map_err(|e| self.unreadable(&self.marker_path(), &e))?;
        let answer = self.change_locked(change);
        // A lock this leaves held is released when the operation closes
        // `marker`, and the change it guarded is whole either way.
        let _ = marker.unlock();
        answer
    }

    fn change_locked<T>(&self, change: impl FnOnce(&mut Index) -> Answer<T>) -> Answer<T> {
        let mut index = self.read_index()?;
        let answer = change(&mut index)?;
        if index.changed {
            let bytes = serde_json::to_vec(&index.file).expect("an index serializes to JSON");
            let temporary = self.write_temporary(&bytes)?;
            let path = self.index_path();
            if let Err(e) = fs::rename(&temporary, &path) {
                let _ = fs::remove_file(&temporary);
                return Err(self.unreadable(&path, &e));
            }
        }
        Ok(answer)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::*;
    use crate::profiles::store::layout::tests::fresh;

    fn descriptor(bytes: &[u8]) -> Descriptor {
        Descriptor {
            media_type: oci::OCTETS.into(),
            digest: Digest::of(bytes),
            size: bytes.len() as u64,
            artifact_type: None,
            annotations: BTreeMap::new(),
        }
    }

    fn manifests(layout: &Layout) -> Vec<Value> {
        let index: Value =
            serde_json::from_slice(&fs::read(layout.root().join("index.json")).unwrap()).unwrap();
        index["manifests"].as_array().unwrap().clone()
    }

    #[test]
    fn another_tools_entry_survives_a_rewrite_unchanged() {
        let (_dir, layout) = fresh();
        let theirs = json!({
            "mediaType": "application/vnd.docker.distribution.manifest.list.v2+json",
            "digest": "sha512:00ff",
            "size": 7143,
            "platform": {"architecture": "arm64", "os": "linux"},
            "annotations": {"org.example.kept": "as written"},
        });
        let path = layout.root().join("index.json");
        fs::write(
            &path,
            json!({
                "schemaVersion": 2,
                "mediaType": IMAGE_INDEX,
                "manifests": [theirs],
                "annotations": {"org.example.index": "theirs"},
            })
            .to_string(),
        )
        .unwrap();
        let marker = layout.opened().unwrap();
        layout
            .change_index(&marker, |index| {
                index.add(descriptor(b"ours"));
                Ok(())
            })
            .unwrap();
        let held: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(held["manifests"][0], theirs);
        assert_eq!(held["manifests"][1], json(&descriptor(b"ours")));
        assert_eq!(held["annotations"]["org.example.index"], "theirs");
        assert_eq!(
            layout.read_index().unwrap().entries,
            [Entry::Other, Entry::Other]
        );
    }

    #[test]
    fn a_malformed_index_is_unreadable_naming_it_and_left_as_found() {
        let (_dir, layout) = fresh();
        let path = layout.root().join("index.json");
        fs::write(&path, b"{\"schemaVersion\": 2, \"manifests\": [").unwrap();
        let words = serde_json::from_slice::<IndexFile>(&fs::read(&path).unwrap())
            .err()
            .unwrap()
            .to_string();
        let expected = Not::unreadable(layout.source(), path.display().to_string(), Some(words));
        assert_eq!(layout.read_index().err(), Some(expected.clone()));
        let marker = layout.opened().unwrap();
        let changed = layout.change_index(&marker, |index| {
            index.add(descriptor(b"ours"));
            Ok(())
        });
        assert_eq!(changed.err(), Some(expected));
        assert_eq!(
            fs::read(&path).unwrap(),
            b"{\"schemaVersion\": 2, \"manifests\": ["
        );
    }

    #[test]
    fn a_change_that_answers_otherwise_writes_nothing() {
        let (_dir, layout) = fresh();
        let marker = layout.opened().unwrap();
        let refused: Answer<()> = layout.change_index(&marker, |index| {
            index.add(descriptor(b"ours"));
            Err(Not::absent(layout.source(), "it"))
        });
        assert!(refused.is_err());
        assert_eq!(manifests(&layout), Vec::<Value>::new());
    }

    #[test]
    fn a_name_has_one_entry_replaced_in_place() {
        let (_dir, layout) = fresh();
        let marker = layout.opened().unwrap();
        let named = |bytes: &[u8]| {
            let mut d = descriptor(bytes);
            d.annotations.insert(REF_NAME.into(), "site-v1".into());
            d
        };
        layout
            .change_index(&marker, |index| {
                index.add(descriptor(b"first"));
                index.set_name("site-v1", named(b"a"));
                index.add(descriptor(b"last"));
                index.set_name("site-v1", named(b"b"));
                Ok(())
            })
            .unwrap();
        let held = manifests(&layout);
        assert_eq!(held.len(), 3);
        assert_eq!(held[1], json(&named(b"b")));
        let index = layout.read_index().unwrap();
        assert_eq!(index.named("site-v1"), Some(&named(b"b")));
    }
}
