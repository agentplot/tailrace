//! The projection for a file (C.11; design D4): an output the build left
//! as one file, packed as that file's own bytes, so the bytes the checks
//! passed are the bytes offered. It runs no packager and opens no
//! archive; the version is in the metadata and never in the bytes, so
//! one output packs to the same bytes under every version (24, 84, 87).
//! A directory is published by a task that leaves an archive of it,
//! since tailrace builds nothing (15).
//!
//! It packs no cache entries: a pack with no version and a render of
//! cache entries answer absent naming them, and read and write nothing
//! (84, 85). Its render of artifacts and serve are not built and answer
//! absent.

use tailrace_seams::{
    no_cache_entries, no_lockfile, Answer, BuildId, Contents, Layout, Output, Packed, Pinned,
    Projection, Rendered, Seam, Serving, Source, Version,
};

use super::{meta, not_built, one_file, read};

/// The ecosystem this projection packs in, as the policy file's
/// `[projections]` names it.
pub const ECOSYSTEM: &str = "file";

/// The file projection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FileProjection;

impl FileProjection {
    pub fn new() -> FileProjection {
        FileProjection
    }

    /// The projection seam and this profile, as every answer names them.
    pub fn source() -> Source {
        Source::new(Seam::Projection, ECOSYSTEM)
    }
}

impl Projection for FileProjection {
    /// 84, 87: the output's one file's bytes unchanged, named by the
    /// output and the version, packing the same again at every build
    /// identity; absent for a cache entry, which it packs none of.
    fn pack(&self, output: &Output, _: &BuildId, version: Option<&Version>) -> Answer<Packed> {
        let source = FileProjection::source();
        let Some(version) = version else {
            return Err(no_cache_entries(&source));
        };
        let file = one_file(output, &source, ECOSYSTEM)?;
        let bytes = read(file, &source)?;
        Ok(Packed {
            bytes,
            meta: Some(meta(ECOSYSTEM, output, version, &source)?),
            same_again: true,
        })
    }

    fn render(&self, contents: Contents<'_>, _: &str) -> Answer<Rendered> {
        let source = FileProjection::source();
        match contents {
            Contents::Artifacts(_) => Err(not_built("render", ECOSYSTEM, &source)),
            Contents::CacheEntries(_) => Err(no_cache_entries(&source)),
        }
    }

    fn serve(&self, _: &Layout) -> Answer<Serving> {
        Err(not_built("serve", ECOSYSTEM, &FileProjection::source()))
    }

    /// 137: absent naming the lockfile for every checkout, since a file
    /// has no lockfile.
    fn pinned(&self, _: &str) -> Answer<Vec<Pinned>> {
        Err(no_lockfile(&FileProjection::source()))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tailrace_seams::{
        Artifact, ArtifactMeta, CacheEntry, Digest, Ecosystem, EcosystemName, Not, OutputName,
        Seam, Why,
    };
    use tempfile::TempDir;

    use super::*;

    /// An output named `site` whose one file holds `bytes`.
    fn site(dir: &TempDir, bytes: &[u8]) -> Output {
        let path = dir.path().join("site.tar");
        fs::write(&path, bytes).unwrap();
        Output {
            name: OutputName::new("site"),
            files: vec![path.display().to_string()],
        }
    }

    fn meta(version: &str) -> ArtifactMeta {
        ArtifactMeta::new(
            Ecosystem::new("file"),
            EcosystemName::new("site"),
            Version::new(version),
        )
        .unwrap()
    }

    fn b7e1() -> BuildId {
        BuildId::new("b7e1")
    }

    /// `output` packed at `b7e1` with `version`.
    fn pack(output: &Output, version: &str) -> Answer<Packed> {
        FileProjection.pack(output, &b7e1(), Some(&Version::new(version)))
    }

    #[test]
    fn one_file_packs_as_its_own_bytes_named_by_the_output() {
        let dir = TempDir::new().unwrap();
        let packed = pack(&site(&dir, b"abc"), "1.4.0").unwrap();
        assert_eq!(
            packed,
            Packed {
                bytes: b"abc".to_vec(),
                meta: Some(meta("1.4.0")),
                same_again: true,
            }
        );
    }

    #[test]
    fn two_versions_pack_one_set_of_bytes() {
        let dir = TempDir::new().unwrap();
        let output = site(&dir, b"abc");
        let first = pack(&output, "1.4.0").unwrap();
        let second = pack(&output, "1.5.0-pre.1").unwrap();
        assert_eq!(Digest::of(&first.bytes), Digest::of(&second.bytes));
        assert_eq!(second.meta, Some(meta("1.5.0-pre.1")));
        assert!(first.same_again && second.same_again);
    }

    #[test]
    fn an_output_of_three_files_is_refused_naming_them() {
        let files = vec![
            "dist/a.html".into(),
            "dist/b.css".into(),
            "dist/c.js".into(),
        ];
        let output = Output {
            name: OutputName::new("site"),
            files,
        };
        match pack(&output, "1.4.0") {
            Err(Not::Refused(r)) => {
                assert_eq!(r.source, FileProjection::source());
                assert_eq!(r.why, Why::Said);
                assert_eq!(
                    r.said.as_deref(),
                    Some(
                        "the output site left 3 files, dist/a.html, dist/b.css, dist/c.js, \
                         and the file projection packs one"
                    )
                );
            }
            other => panic!("three files answered {other:?}"),
        }
        let none = Output {
            name: OutputName::new("site"),
            files: vec![],
        };
        assert!(matches!(
            pack(&none, "1.4.0"),
            Err(Not::Refused(r)) if r.said.as_deref().is_some_and(|s| s.contains("left no file"))
        ));
    }

    #[test]
    fn a_file_gone_is_unreadable_in_the_operating_systems_words() {
        let dir = TempDir::new().unwrap();
        let output = site(&dir, b"abc");
        fs::remove_file(&output.files[0]).unwrap();
        let words = fs::read(&output.files[0]).unwrap_err().to_string();
        match pack(&output, "1.4.0") {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source.seam, Seam::Projection);
                assert_eq!(u.what, output.files[0]);
                assert_eq!(u.said, Some(words));
            }
            other => panic!("a removed file answered {other:?}"),
        }
    }

    #[test]
    fn render_and_serve_are_absent_and_write_nothing() {
        let dir = TempDir::new().unwrap();
        let at = dir.path().join("layouts").join("file");
        let artifacts = [Artifact {
            digest: Digest::of(b"abc"),
            meta: meta("1.4.0"),
        }];
        let rendered = FileProjection.render(Contents::Artifacts(&artifacts), at.to_str().unwrap());
        assert_eq!(
            rendered,
            Err(Not::absent(
                &FileProjection::source(),
                "render of the file projection"
            ))
        );
        assert!(!at.exists(), "render wrote under {}", at.display());
        let layout = Layout {
            at: at.display().to_string(),
        };
        assert_eq!(
            FileProjection.serve(&layout),
            Err(Not::absent(
                &FileProjection::source(),
                "serve of the file projection"
            ))
        );
    }

    #[test]
    fn a_cache_entry_is_absent_and_nothing_is_read_or_written() {
        let dir = TempDir::new().unwrap();
        let output = site(&dir, b"abc");
        fs::remove_file(&output.files[0]).unwrap();
        let absent = Not::absent(&FileProjection::source(), "cache entries");
        assert_eq!(
            FileProjection.pack(&output, &b7e1(), None),
            Err(absent.clone())
        );
        let at = dir.path().join("layouts").join("file");
        let entries = [CacheEntry {
            build: b7e1(),
            digest: Digest::of(b"abc"),
        }];
        assert_eq!(
            FileProjection.render(Contents::CacheEntries(&entries), at.to_str().unwrap()),
            Err(absent)
        );
        assert!(!at.exists(), "render wrote under {}", at.display());
    }
}
