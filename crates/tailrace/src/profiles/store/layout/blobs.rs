//! Blobs: bytes written once, whole, under their own digest, and answered
//! only while they are still those bytes (design D2, D3; 2, 5, 66, 73,
//! rule core.5).

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use tailrace_seams::{Answer, Digest, Not, Put, Why};

use super::Layout;

/// How many temporary files this process has made, so no two of its
/// writers name one alike.
static TEMPORARIES: AtomicU64 = AtomicU64::new(0);

impl Layout {
    /// Where the bytes of `digest` are held: `blobs/sha256/<hex>`.
    pub(super) fn blob_path(&self, digest: &Digest) -> PathBuf {
        self.root
            .join("blobs")
            .join(digest.algorithm())
            .join(digest.hex())
    }

    /// `bytes` held under their own digest and nowhere else. A blob not
    /// held is written to a temporary file and hard-linked to its path,
    /// which never replaces a file, so no reader sees part of one; a blob
    /// held, or linked first by another writer, is held already once the
    /// bytes there are read and found to be these (5, 66).
    pub(super) fn write_blob(&self, bytes: &[u8]) -> Answer<Put> {
        let digest = Digest::of(bytes);
        let path = self.blob_path(&digest);
        let written = match fs::metadata(&path) {
            Ok(_) => false,
            Err(e) if e.kind() == io::ErrorKind::NotFound => self.link(bytes, &path)?,
            Err(e) => return Err(self.unreadable(&path, &e)),
        };
        if !written {
            self.read_blob(&digest)?;
        }
        Ok(Put {
            digest,
            held_already: !written,
        })
    }

    /// `bytes` linked to `path` through a temporary file: true when this
    /// write made the blob, false when the path was taken first.
    fn link(&self, bytes: &[u8], path: &Path) -> Answer<bool> {
        let temporary = self.write_temporary(bytes)?;
        let linked = fs::hard_link(&temporary, path);
        // The blob is whole at its path or absent whatever this removal
        // answers, and a temporary file left behind is one the layout
        // specification allows and no reader reads.
        let _ = fs::remove_file(&temporary);
        match linked {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(false),
            Err(e) => Err(self.unreadable(path, &e)),
        }
    }

    /// `bytes` in a new file of the layout's directory, synced, under a
    /// name no other writer takes (design D2).
    pub(super) fn write_temporary(&self, bytes: &[u8]) -> Answer<PathBuf> {
        let n = TEMPORARIES.fetch_add(1, Ordering::Relaxed);
        let path = self
            .root
            .join(format!(".tailrace-{}-{n}.tmp", std::process::id()));
        let written = File::options()
            .write(true)
            .create_new(true)
            .open(&path)
            .and_then(|mut file| {
                file.write_all(bytes)?;
                file.sync_all()
            });
        if let Err(e) = written {
            let _ = fs::remove_file(&path);
            return Err(self.unreadable(&path, &e));
        }
        Ok(path)
    }

    /// The bytes held under `digest`, once their own digest is found to
    /// be it: absent when nothing is held there, and refused naming both
    /// digests when other bytes are, which are left as found (2, 73).
    pub(super) fn read_blob(&self, digest: &Digest) -> Answer<Vec<u8>> {
        let path = self.blob_path(digest);
        let bytes = fs::read(&path).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => Not::absent(&self.source, digest.to_string()),
            _ => self.unreadable(&path, &e),
        })?;
        let found = Digest::of(&bytes);
        if &found != digest {
            return Err(Not::refused(
                &self.source,
                Why::OtherBytes {
                    expected: digest.clone(),
                    answered: found,
                },
                None,
            ));
        }
        Ok(bytes)
    }

    /// The size of the blob held under `digest`; absent when none is.
    pub(super) fn blob_size(&self, digest: &Digest) -> Answer<u64> {
        let path = self.blob_path(digest);
        match fs::metadata(&path) {
            Ok(metadata) => Ok(metadata.len()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                Err(Not::absent(&self.source, digest.to_string()))
            }
            Err(e) => Err(self.unreadable(&path, &e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use tailrace_seams::{Held, Store};

    use super::*;
    use crate::profiles::store::layout::tests::fresh;

    fn other_bytes(answer: Answer<impl std::fmt::Debug>) -> (Digest, Digest) {
        match answer {
            Err(Not::Refused(r)) => match r.why {
                Why::OtherBytes { expected, answered } => (expected, answered),
                why => panic!("refused for {why:?}"),
            },
            other => panic!("answered {other:?}, not refused"),
        }
    }

    #[test]
    fn the_same_bytes_put_twice_are_one_blob_written_once() {
        let (_dir, layout) = fresh();
        let first = layout.put_bytes(b"site").unwrap();
        let path = layout.blob_path(&first.digest);
        let modified = fs::metadata(&path).unwrap().modified().unwrap();
        let second = layout.put_bytes(b"site").unwrap();
        assert_eq!(first.digest, Digest::of(b"site"));
        assert!(!first.held_already);
        assert_eq!(second.digest, first.digest);
        assert!(second.held_already);
        assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), modified);
        assert_eq!(fs::read(&path).unwrap(), b"site");
        assert_eq!(
            path,
            layout
                .root()
                .join("blobs/sha256")
                .join(Digest::of(b"site").hex())
        );
        assert_eq!(
            layout.read(&first.digest),
            Ok(Held::Bytes(b"site".to_vec()))
        );
    }

    #[test]
    fn a_blob_altered_on_disk_is_refused_naming_both_and_left_as_found() {
        let (_dir, layout) = fresh();
        let digest = layout.put_bytes(b"site").unwrap().digest;
        let path = layout.blob_path(&digest);
        fs::write(&path, b"altered").unwrap();
        let modified = fs::metadata(&path).unwrap().modified().unwrap();
        let both = (digest.clone(), Digest::of(b"altered"));
        assert_eq!(other_bytes(layout.read(&digest)), both);
        assert_eq!(other_bytes(layout.put_bytes(b"site")), both);
        assert_eq!(fs::read(&path).unwrap(), b"altered");
        assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), modified);
    }

    #[test]
    fn a_digest_never_put_is_absent_naming_it() {
        let (_dir, layout) = fresh();
        let digest = Digest::of(b"never put");
        assert_eq!(
            layout.read(&digest),
            Err(Not::absent(layout.source(), digest.to_string()))
        );
        assert_eq!(
            layout.blob_size(&digest),
            Err(Not::absent(layout.source(), digest.to_string()))
        );
    }

    #[test]
    fn no_temporary_file_remains_after_a_put() {
        let (_dir, layout) = fresh();
        layout.put_bytes(b"site").unwrap();
        layout.put_bytes(b"site").unwrap();
        layout.put_bytes(b"docs").unwrap();
        let mut names: Vec<String> = fs::read_dir(layout.root())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(names, ["blobs", "index.json", "oci-layout"]);
        assert_eq!(
            fs::read_dir(layout.root().join("blobs/sha256"))
                .unwrap()
                .count(),
            2
        );
    }
}
