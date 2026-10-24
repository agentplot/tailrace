//! The projection for moon's cache (C.11; design D5): a task's output
//! archive, the one file moon leaves for a task that passed at
//! `.moon/cache/outputs/<task hash>.tar.gz`, packed as a cache entry by
//! its task hash, and moon's outputs directory rendered from cache
//! entries, so moon replays a task another host ran.
//!
//! pack answers the archive's bytes unchanged with no metadata, and says
//! it cannot promise the same bytes again, since moon writes each file's
//! time and owner into the archive (84, 87, 100). render reads each
//! entry's bytes through the store seam, checks they are the entry's
//! digest, and writes them as the archive of its task hash, following a
//! link at the outputs directory and leaving an archive already there as
//! it is (85). A build identity that is no task hash, a non-empty string
//! of lower-case hexadecimal digits, is refused before anything is read
//! or written, so no verdict places a file outside moon's directory.
//!
//! It packs no artifact: a pack given a version and a render of
//! artifacts answer absent naming artifacts and read nothing (53). It
//! reaches the store only through the store seam it is handed (rule
//! core.2), and serves nothing, since moon reads its outputs directory
//! before any remote.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use tailrace_seams::{
    Answer, BuildId, CacheEntry, Contents, Digest, Held, Holds, Layout, Not, Output, Packed,
    Projection, Rendered, Seam, Serving, Source, Store, Version, Why,
};

/// The ecosystem this projection packs in, as the bindings bind it.
pub const ECOSYSTEM: &str = "moon-cache";

/// Where moon keeps a checkout's output archives, under the checkout.
const OUTPUTS: &str = ".moon/cache/outputs";

/// What an archive's name ends with after its task hash.
const ARCHIVE: &str = ".tar.gz";

/// The moon cache projection of one checkout, reading entries from the
/// store it is handed.
pub struct MoonCache {
    root: PathBuf,
    store: Box<dyn Store>,
}

impl MoonCache {
    /// The projection of the checkout at `root`, reading entries from
    /// `store`.
    pub fn new(root: impl Into<PathBuf>, store: Box<dyn Store>) -> MoonCache {
        MoonCache {
            root: root.into(),
            store,
        }
    }

    /// The projection seam and this profile, as every answer names them.
    pub fn source() -> Source {
        Source::new(Seam::Projection, ECOSYSTEM)
    }

    /// The archive of `build` in the outputs directory under `checkout`.
    fn archive(checkout: &Path, build: &BuildId) -> PathBuf {
        checkout.join(OUTPUTS).join(format!("{build}{ARCHIVE}"))
    }

    /// `at` as a checkout: itself where absolute, else under the root.
    fn checkout(&self, at: &str) -> PathBuf {
        let at = Path::new(at);
        if at.is_absolute() {
            at.to_path_buf()
        } else {
            self.root.join(at)
        }
    }

    /// One entry's archive written in `dir`, the outputs directory, unless
    /// one is there already: the bytes read through the store, checked
    /// against the entry's digest, written beside it and renamed onto it,
    /// so moon never reads half an archive.
    fn fill(&self, dir: &Path, entry: &CacheEntry) -> Answer<()> {
        let source = MoonCache::source();
        let archive = dir.join(format!("{}{ARCHIVE}", entry.build));
        if archive.exists() {
            return Ok(());
        }
        let bytes = match self.store.read(&entry.digest)? {
            Held::Bytes(bytes) => bytes,
            Held::Manifest(_) => {
                return Err(Not::refused(
                    &source,
                    Why::Said,
                    Some(format!(
                        "the cache entry of {} names {}, which holds a manifest and not an \
                         archive",
                        entry.build, entry.digest
                    )),
                ))
            }
        };
        let answered = Digest::of(&bytes);
        if answered != entry.digest {
            return Err(Not::refused(
                &source,
                Why::OtherBytes {
                    expected: entry.digest.clone(),
                    answered,
                },
                None,
            ));
        }
        let partial = dir.join(format!(
            ".{}{ARCHIVE}.{}.partial",
            entry.build,
            std::process::id()
        ));
        let written = fs::write(&partial, &bytes).and_then(|()| fs::rename(&partial, &archive));
        written.map_err(|e| {
            let _ = fs::remove_file(&partial);
            unreadable(&archive, &e)
        })
    }
}

/// `build` as a task hash, or refused naming it.
fn task_hash(build: &BuildId) -> Answer<()> {
    let text = build.as_str();
    if !text.is_empty() && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        return Ok(());
    }
    Err(Not::refused(
        &MoonCache::source(),
        Why::Said,
        Some(format!(
            "the build identity {build} is not a task hash, which is lower-case hexadecimal \
             digits"
        )),
    ))
}

fn unreadable(path: &Path, e: &io::Error) -> Not {
    Not::unreadable(
        &MoonCache::source(),
        path.display().to_string(),
        Some(e.to_string()),
    )
}

fn no_artifacts() -> Not {
    Not::absent(&MoonCache::source(), "artifacts")
}

impl Projection for MoonCache {
    /// 84, 87: the archive moon left for the task hash `at`, its bytes
    /// unchanged with no metadata, never promised the same again; absent
    /// naming the archive's path where moon left none. `output` is not
    /// read: the archive holds the task's outputs as moon declared them.
    fn pack(&self, _: &Output, at: &BuildId, version: Option<&Version>) -> Answer<Packed> {
        if version.is_some() {
            return Err(no_artifacts());
        }
        task_hash(at)?;
        let archive = MoonCache::archive(&self.root, at);
        let bytes = fs::read(&archive).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => {
                Not::absent(&MoonCache::source(), archive.display().to_string())
            }
            _ => unreadable(&archive, &e),
        })?;
        Ok(Packed {
            bytes,
            meta: None,
            same_again: false,
        })
    }

    /// 85: each entry's archive in the outputs directory of the checkout
    /// `at`, and the build identities given, in the order given.
    fn render(&self, contents: Contents<'_>, at: &str) -> Answer<Rendered> {
        let Contents::CacheEntries(entries) = contents else {
            return Err(no_artifacts());
        };
        for entry in entries {
            task_hash(&entry.build)?;
        }
        let dir = self.checkout(at).join(OUTPUTS);
        fs::create_dir_all(&dir).map_err(|e| unreadable(&dir, &e))?;
        for entry in entries {
            self.fill(&dir, entry)?;
        }
        Ok(Rendered {
            layout: Layout {
                at: dir.display().to_string(),
            },
            holds: Holds::BuildIds(entries.iter().map(|e| e.build.clone()).collect()),
        })
    }

    fn serve(&self, _: &Layout) -> Answer<Serving> {
        Err(Not::absent(
            &MoonCache::source(),
            "serve of the moon-cache projection, whose outputs directory moon reads itself",
        ))
    }
}

#[cfg(test)]
mod tests;
