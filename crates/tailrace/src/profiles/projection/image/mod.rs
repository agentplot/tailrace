//! The projection for an image (C.11; design D5): an output the build
//! left as an OCI image archive, the image as it would be pushed: a tar
//! holding an OCI image layout at its root, as `docker save` writes one
//! since Docker Engine 25 and as `skopeo copy … oci-archive:` and buildah
//! write. pack checks the file is such a tar by reading its members'
//! names from the POSIX ustar headers until it has seen `oci-layout` and
//! `index.json` at the root, and answers the file's bytes unchanged; the
//! version is in the metadata and never in the bytes (24, 84, 87). A pax
//! or GNU long-name header is skipped by its size like any member, and an
//! archive that hides the two names is refused, never accepted.
//!
//! It packs no cache entries: a pack with no version and a render of
//! cache entries answer absent naming them, and read and write nothing
//! (84, 85). Its render of artifacts and serve are not built and answer
//! absent. An image has no lockfile, so pinned answers absent naming the
//! lockfile for every checkout, and reads nothing (137). An image has no
//! manifest either, so requires and satisfying answer none, an empty
//! list, and read nothing (140).

use tailrace_seams::{
    no_cache_entries, no_lockfile, Answer, Artifact, BuildId, Contents, Layout, Not, Output,
    Packed, Pinned, Projection, Range, Rendered, Requirement, Seam, Serving, Source, Version, Why,
};

use super::{meta, not_built, one_file, read};

#[cfg(test)]
#[path = "../../../../tests/fixtures/image.rs"]
mod fixture;

/// The ecosystem this projection packs in, as the policy file's
/// `[projections]` names it.
pub const ECOSYSTEM: &str = "image";

/// The names an OCI image layout holds at its root.
const LAYOUT: [&str; 2] = ["oci-layout", "index.json"];

/// A tar block, and the size of every header.
const BLOCK: usize = 512;

/// The image projection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImageProjection;

impl ImageProjection {
    pub fn new() -> ImageProjection {
        ImageProjection
    }

    /// The projection seam and this profile, as every answer names them.
    pub fn source() -> Source {
        Source::new(Seam::Projection, ECOSYSTEM)
    }
}

/// Why an archive is not an image as pushed.
#[derive(Debug, PartialEq, Eq)]
enum NotAnImage {
    /// It is not a tar, for the reason given.
    NotTar(String),
    /// A tar without these names at its root.
    Lacks(Vec<&'static str>),
}

/// The octal number a ustar header field holds, or in GNU's base-256
/// form where its first byte's high bit is set.
fn number(field: &[u8]) -> Option<u64> {
    if field.first().is_some_and(|b| b & 0x80 != 0) {
        return field[1..]
            .iter()
            .try_fold(u64::from(field[0] & 0x7f), |n, b| {
                n.checked_mul(256)?.checked_add(u64::from(*b))
            });
    }
    let digits: Vec<u8> = field
        .iter()
        .copied()
        .skip_while(|b| *b == b' ')
        .take_while(|b| *b != 0 && *b != b' ')
        .collect();
    if digits.is_empty() {
        return Some(0);
    }
    u64::from_str_radix(std::str::from_utf8(&digits).ok()?, 8).ok()
}

/// That `header` is a ustar header whose checksum holds, its sum taken
/// unsigned or, as some old writers took it, signed.
fn is_header(header: &[u8]) -> bool {
    if &header[257..262] != b"ustar" {
        return false;
    }
    let Some(sum) = number(&header[148..156]) else {
        return false;
    };
    let spaced = |i: usize, b: u8| if (148..156).contains(&i) { b' ' } else { b };
    let unsigned: u64 = header
        .iter()
        .enumerate()
        .map(|(i, b)| u64::from(spaced(i, *b)))
        .sum();
    let signed: i64 = header
        .iter()
        .enumerate()
        .map(|(i, b)| i64::from(spaced(i, *b) as i8))
        .sum();
    sum == unsigned || i64::try_from(sum).is_ok_and(|sum| sum == signed)
}

/// The member's name: the ustar `prefix` and `name` fields joined, with
/// a leading `./` dropped.
fn name(header: &[u8]) -> String {
    let field = |bytes: &[u8]| {
        let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
        String::from_utf8_lossy(&bytes[..end]).into_owned()
    };
    let (prefix, name) = (field(&header[345..500]), field(&header[..100]));
    let full = if prefix.is_empty() {
        name
    } else {
        format!("{prefix}/{name}")
    };
    full.strip_prefix("./").map(str::to_string).unwrap_or(full)
}

/// That `bytes` are a tar holding a regular file of each of the layout's
/// names at its root, read header by header until both are seen.
fn holds_a_layout(bytes: &[u8]) -> Result<(), NotAnImage> {
    if bytes.starts_with(&[0x1f, 0x8b]) {
        return Err(NotAnImage::NotTar("it is compressed with gzip".into()));
    }
    let mut seen = [false; LAYOUT.len()];
    let mut at = 0;
    while !seen.iter().all(|s| *s) {
        let Some(header) = bytes.get(at..at + BLOCK) else {
            if at == 0 {
                return Err(NotAnImage::NotTar(format!(
                    "it is {} bytes, shorter than one tar header",
                    bytes.len()
                )));
            }
            break;
        };
        if header.iter().all(|b| *b == 0) {
            break;
        }
        if !is_header(header) {
            return Err(NotAnImage::NotTar(format!(
                "the {BLOCK} bytes at offset {at} are no ustar header"
            )));
        }
        let size = number(&header[124..136]).ok_or_else(|| {
            NotAnImage::NotTar(format!("the header at offset {at} names no size"))
        })?;
        if matches!(header[156], b'0' | 0 | b'7') {
            let name = name(header);
            for (i, wanted) in LAYOUT.iter().enumerate() {
                seen[i] |= name == *wanted;
            }
        }
        let blocks = usize::try_from(size.div_ceil(BLOCK as u64)).unwrap_or(usize::MAX);
        at = blocks
            .checked_mul(BLOCK)
            .and_then(|data| at.checked_add(BLOCK + data))
            .unwrap_or(usize::MAX);
    }
    let lacks: Vec<&'static str> = LAYOUT
        .iter()
        .zip(seen)
        .filter(|(_, seen)| !seen)
        .map(|(name, _)| *name)
        .collect();
    if lacks.is_empty() {
        Ok(())
    } else {
        Err(NotAnImage::Lacks(lacks))
    }
}

impl Projection for ImageProjection {
    /// 84, 87: the output's one file's bytes unchanged once it is a tar
    /// holding an OCI image layout at its root, named by the output and
    /// the version, packing the same again at every build identity;
    /// refused naming the output and what the file holds instead; absent
    /// for a cache entry, which it packs none of.
    fn pack(&self, output: &Output, _: &BuildId, version: Option<&Version>) -> Answer<Packed> {
        let source = ImageProjection::source();
        let Some(version) = version else {
            return Err(no_cache_entries(&source));
        };
        let file = one_file(output, &source, ECOSYSTEM)?;
        let bytes = read(file, &source)?;
        let refused = |why: String| {
            Not::refused(
                &source,
                Why::Said,
                Some(format!("the output {}'s file {file} {why}", output.name)),
            )
        };
        match holds_a_layout(&bytes) {
            Ok(()) => {}
            Err(NotAnImage::NotTar(why)) => {
                return Err(refused(format!(
                    "is not a tar holding an OCI image layout: {why}"
                )))
            }
            Err(NotAnImage::Lacks(names)) => {
                let names: Vec<String> = names.iter().map(|n| format!("no {n}")).collect();
                return Err(refused(format!(
                    "is a tar holding {} at its root, so no OCI image layout",
                    names.join(" and ")
                )));
            }
        }
        Ok(Packed {
            bytes,
            meta: Some(meta(ECOSYSTEM, output, version, &source)?),
            same_again: true,
        })
    }

    fn render(&self, contents: Contents<'_>, _: &str) -> Answer<Rendered> {
        let source = ImageProjection::source();
        match contents {
            Contents::Artifacts(_) => Err(not_built("render", ECOSYSTEM, &source)),
            Contents::CacheEntries(_) => Err(no_cache_entries(&source)),
        }
    }

    fn serve(&self, _: &Layout) -> Answer<Serving> {
        Err(not_built("serve", ECOSYSTEM, &ImageProjection::source()))
    }

    /// 137: absent naming the lockfile for every checkout, since an image
    /// has no lockfile.
    fn pinned(&self, _: &str) -> Answer<Vec<Pinned>> {
        Err(no_lockfile(&ImageProjection::source()))
    }

    /// 140: none for every artifact, since an image has no manifest; it
    /// reads nothing.
    fn requires(&self, _: &Artifact) -> Answer<Vec<Requirement>> {
        Ok(Vec::new())
    }

    /// 140: none for any range and any versions, since an image has no
    /// manifest to require a range.
    fn satisfying(&self, _: &Range, _: &[Version]) -> Answer<Vec<Version>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tailrace_seams::{ArtifactMeta, CacheEntry, Digest, Ecosystem, EcosystemName, OutputName};
    use tempfile::TempDir;

    use super::*;

    /// An output named `api` whose one file holds `bytes`.
    fn api(dir: &Path, bytes: &[u8]) -> Output {
        let path = dir.join("image.tar");
        fs::write(&path, bytes).unwrap();
        Output {
            name: OutputName::new("api"),
            files: vec![path.display().to_string()],
        }
    }

    fn b7e1() -> BuildId {
        BuildId::new("b7e1")
    }

    /// `output` packed at `b7e1` with `version`.
    fn pack(output: &Output, version: &str) -> Answer<Packed> {
        ImageProjection.pack(output, &b7e1(), Some(&Version::new(version)))
    }

    /// The words of a refusal of `output`, once it names this profile.
    fn refused(output: &Output) -> String {
        match pack(output, "0.3.0") {
            Err(Not::Refused(r)) => {
                assert_eq!(r.source, ImageProjection::source());
                assert_eq!(r.why, Why::Said);
                r.said.expect("the refusal's words")
            }
            other => panic!("answered {other:?}, not refused"),
        }
    }

    #[test]
    fn an_archive_as_docker_save_writes_one_packs_as_its_own_bytes() {
        let dir = TempDir::new().unwrap();
        let archive = fixture::archive();
        let output = api(dir.path(), &archive);
        for version in ["0.3.0", "0.4.0"] {
            let packed = pack(&output, version).unwrap();
            assert_eq!(
                packed,
                Packed {
                    bytes: archive.clone(),
                    meta: Some(
                        ArtifactMeta::new(
                            Ecosystem::new("image"),
                            EcosystemName::new("api"),
                            Version::new(version),
                        )
                        .unwrap()
                    ),
                    same_again: true,
                }
            );
        }
    }

    #[test]
    fn a_compressed_archive_is_refused_as_no_tar() {
        let dir = TempDir::new().unwrap();
        let output = api(dir.path(), &fixture::gzip(&fixture::archive()));
        let said = refused(&output);
        assert_eq!(
            said,
            format!(
                "the output api's file {} is not a tar holding an OCI image layout: \
                 it is compressed with gzip",
                output.files[0]
            )
        );
    }

    #[test]
    fn a_legacy_docker_archive_is_refused_naming_the_missing_oci_layout() {
        let dir = TempDir::new().unwrap();
        let output = api(dir.path(), &fixture::legacy());
        let said = refused(&output);
        assert_eq!(
            said,
            format!(
                "the output api's file {} is a tar holding no oci-layout and no index.json \
                 at its root, so no OCI image layout",
                output.files[0]
            )
        );
        let index_only = fixture::tar(&[("index.json", Some(b"{}"))]);
        let said = refused(&api(dir.path(), &index_only));
        assert!(said.contains("holding no oci-layout at its root"), "{said}");
    }

    #[test]
    fn a_layout_below_the_root_is_not_at_it() {
        let dir = TempDir::new().unwrap();
        let nested = fixture::tar(&[
            ("image/oci-layout", Some(b"{}")),
            ("image/index.json", Some(b"{}")),
            ("./index.json", Some(b"{}")),
        ]);
        let said = refused(&api(dir.path(), &nested));
        assert!(said.contains("holding no oci-layout at its root"), "{said}");
    }

    #[test]
    fn another_file_than_a_tar_is_refused() {
        let dir = TempDir::new().unwrap();
        let text = "not an archive\n".repeat(64);
        let said = refused(&api(dir.path(), text.as_bytes()));
        assert!(
            said.ends_with(
                "is not a tar holding an OCI image layout: the 512 bytes at offset 0 are no \
                 ustar header"
            ),
            "{said}"
        );
        let said = refused(&api(dir.path(), b"short"));
        assert!(
            said.ends_with("it is 5 bytes, shorter than one tar header"),
            "{said}"
        );
        let mut broken = fixture::archive();
        broken[0] ^= 1;
        let said = refused(&api(dir.path(), &broken));
        assert!(said.contains("at offset 0 are no ustar header"), "{said}");
    }

    #[test]
    fn a_file_gone_is_unreadable_in_the_operating_systems_words() {
        let dir = TempDir::new().unwrap();
        let output = api(dir.path(), &fixture::archive());
        fs::remove_file(&output.files[0]).unwrap();
        let words = fs::read(&output.files[0]).unwrap_err().to_string();
        match pack(&output, "0.3.0") {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, ImageProjection::source());
                assert_eq!(u.what, output.files[0]);
                assert_eq!(u.said, Some(words));
            }
            other => panic!("a removed file answered {other:?}"),
        }
    }

    #[test]
    fn the_pins_of_a_checkout_are_absent_naming_the_lockfile() {
        let dir = TempDir::new().unwrap();
        let output = api(dir.path(), &fixture::archive());
        let absent = Not::absent(&ImageProjection::source(), "lockfile");
        for checkout in [dir.path().to_path_buf(), dir.path().join("no checkout")] {
            assert_eq!(
                ImageProjection.pinned(checkout.to_str().unwrap()),
                Err(absent.clone()),
                "the pins of {}",
                checkout.display()
            );
        }
        assert!(Path::new(&output.files[0]).exists());
        let held: Vec<_> = fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(held.len(), 1, "pinned wrote into the checkout");
    }

    #[test]
    fn what_an_artifact_requires_and_which_versions_satisfy_a_range_are_none() {
        let dir = TempDir::new().unwrap();
        let output = api(dir.path(), &fixture::archive());
        let held = Artifact {
            digest: Digest::of(&fixture::archive()),
            meta: ArtifactMeta::new(
                Ecosystem::new("image"),
                EcosystemName::new("api"),
                Version::new("0.3.0"),
            )
            .unwrap(),
        };
        assert_eq!(ImageProjection.requires(&held), Ok(vec![]));
        let versions = [Version::new("0.3.0"), Version::new("0.10.0")];
        assert_eq!(
            ImageProjection.satisfying(&Range::new("^0.3"), &versions),
            Ok(vec![])
        );
        assert_eq!(fs::read(&output.files[0]).unwrap(), fixture::archive());
        let held: Vec<_> = fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(held.len(), 1, "requires or satisfying wrote");
    }

    #[test]
    fn render_and_serve_are_absent() {
        let layout = Layout {
            at: "layouts/image".into(),
        };
        assert_eq!(
            ImageProjection.serve(&layout),
            Err(Not::absent(
                &ImageProjection::source(),
                "serve of the image projection"
            ))
        );
        assert_eq!(
            ImageProjection.render(Contents::Artifacts(&[]), "layouts/image"),
            Err(Not::absent(
                &ImageProjection::source(),
                "render of the image projection"
            ))
        );
    }

    #[test]
    fn a_cache_entry_is_absent_and_nothing_is_read_or_written() {
        let dir = TempDir::new().unwrap();
        let output = api(dir.path(), &fixture::archive());
        fs::remove_file(&output.files[0]).unwrap();
        let absent = Not::absent(&ImageProjection::source(), "cache entries");
        assert_eq!(
            ImageProjection.pack(&output, &b7e1(), None),
            Err(absent.clone())
        );
        let at = dir.path().join("layouts").join("image");
        let entries = [CacheEntry {
            build: b7e1(),
            digest: Digest::of(&fixture::archive()),
        }];
        assert_eq!(
            ImageProjection.render(Contents::CacheEntries(&entries), at.to_str().unwrap()),
            Err(absent)
        );
        assert!(!at.exists(), "render wrote under {}", at.display());
    }

    #[test]
    fn a_size_in_octal_or_base_256_is_read() {
        assert_eq!(number(b"00000001750\0"), Some(0o1750));
        assert_eq!(number(b"   1750 \0\0\0\0"), Some(0o1750));
        let mut base256 = [0u8; 12];
        base256[0] = 0x80;
        base256[10] = 1;
        base256[11] = 2;
        assert_eq!(number(&base256), Some(258));
        assert_eq!(number(b"0000008\0"), None);
    }
}
