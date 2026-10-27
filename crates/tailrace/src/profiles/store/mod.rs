//! The store seam's profiles (B.1), and the binding of the one the
//! host's `[store]` names.

pub mod layout;
mod oci;
pub mod registry;

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use tailrace_seams::{
    Answer, Digest, Envelope, Held, Manifest, Name, Not, Put, Seam, Source, Store,
};

use crate::bindings::{self, Table};
use layout::Layout;
use registry::Registry;

/// The keys of `[store]`, one per profile.
const KEYS: [&str; 2] = ["layout", "registry"];

/// Where the given layout is, under the user's data directory.
pub const GIVEN_LAYOUT: &str = "tailrace/layout";

/// The store a host binds where its file at `file` names no `[store]`
/// (118, C.2): the layout at [`GIVEN_LAYOUT`] under the user's data
/// directory, `xdg_data_home` where it is set and not empty and
/// `.local/share` under `home` otherwise, as a `[store] layout` table
/// the file left out, so it is bound, named and made exactly as a
/// written one is. Absent naming both variables where neither is set,
/// since no given layout can be placed; nothing is read.
pub fn given(file: &Path, xdg_data_home: Option<&OsStr>, home: Option<&Path>) -> Answer<Table> {
    let data = bindings::data(xdg_data_home, home).ok_or_else(|| {
        Not::absent(
            &bindings::source(file),
            format!(
                "binding of the store: neither XDG_DATA_HOME nor HOME is set to place \
                 {GIVEN_LAYOUT}"
            ),
        )
    })?;
    let layout = data.join(GIVEN_LAYOUT);
    let text = layout.to_str().ok_or_else(|| {
        bindings::unreadable(
            file,
            Some("[store] layout (given)"),
            format!("{} is not a path in UTF-8", layout.display()),
        )
    })?;
    Ok(Table::given(file, "store", "layout", text.into()))
}

/// The store `[store]` names (92, C.1, C.2): `layout`, the OCI layout at
/// its path, made there when `make` and the path is missing or an empty
/// directory, as a publish's is the first time, at the first operation
/// asked of it and never at its binding, so a publish refused before it
/// asks the store anything leaves nothing behind (60, rule core.11); and
/// opened otherwise, so a layout that is not there is unreadable in the
/// operating system's words; `registry`, the repository it names, which
/// no binding makes, whatever `make`; unreadable naming any other key.
pub fn bind(table: &Table, make: bool) -> Answer<Box<dyn Store>> {
    match table.key() {
        "layout" => {
            let path = table.path()?;
            if make && missing_or_empty(&path) {
                return Ok(Box::new(Unmade {
                    path,
                    layout: OnceLock::new(),
                }));
            }
            Ok(Box::new(Layout::open(&path)?))
        }
        "registry" => Ok(Box::new(registry(table)?)),
        _ => Err(table.unknown(&KEYS)),
    }
}

/// The store as the host's file names it, which the record keeps (61,
/// 118): its key, and the layout's path with `~/` expanded or the
/// registry's repository; nothing is read or made.
pub fn named(table: &Table) -> Answer<Source> {
    let at = match table.key() {
        "layout" => table.path()?.display().to_string(),
        "registry" => registry(table)?.repository().to_string(),
        _ => return Err(table.unknown(&KEYS)),
    };
    Ok(Source::new(Seam::Store, format!("{} {at}", table.key())))
}

/// The registry `[store] registry` names by its repository; unreadable
/// naming the file, the key and the value when it names none. The file
/// names no credential: the host's Docker configuration holds it (64).
fn registry(table: &Table) -> Answer<Registry> {
    let text = table.value().as_str().ok_or_else(|| {
        table.unreadable(format!("{} is not a repository in a string", table.value()))
    })?;
    Registry::new(text).map_err(|not| match not {
        Not::Refused(r) => table.unreadable(r.said.unwrap_or_else(|| r.why.to_string())),
        other => other,
    })
}

/// A layout a publish or a push makes at `path`, missing or an empty
/// directory when it was bound, at the first operation asked of it: made
/// there where the path is still missing or empty, and opened where
/// another binding of it made it first.
struct Unmade {
    path: PathBuf,
    layout: OnceLock<Answer<Layout>>,
}

impl Unmade {
    fn layout(&self) -> Answer<&Layout> {
        self.layout
            .get_or_init(|| {
                if missing_or_empty(&self.path) {
                    Layout::create(&self.path)
                } else {
                    Layout::open(&self.path)
                }
            })
            .as_ref()
            .map_err(Clone::clone)
    }
}

impl Store for Unmade {
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        self.layout()?.put_bytes(bytes)
    }

    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        self.layout()?.put_artifact(manifest)
    }

    fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
        self.layout()?.attach(subject, attestation)
    }

    fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>> {
        self.layout()?.claims_of(digest)
    }

    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
        self.layout()?.name(name, to, expected)
    }

    fn resolve(&self, name: &Name) -> Answer<Digest> {
        self.layout()?.resolve(name)
    }

    fn read(&self, digest: &Digest) -> Answer<Held> {
        self.layout()?.read(digest)
    }
}

/// Whether nothing is at `path`, or an empty directory is; a directory
/// that cannot be read is neither, and its open says why.
fn missing_or_empty(path: &Path) -> bool {
    match fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_none(),
        Err(e) => e.kind() == std::io::ErrorKind::NotFound,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file() -> PathBuf {
        PathBuf::from("/config/tailrace/config.toml")
    }

    #[test]
    fn the_given_layout_is_under_the_data_directory_or_the_home() {
        let home = Path::new("/home/a");
        for (xdg, layout) in [
            (Some(OsStr::new("/data")), "/data/tailrace/layout"),
            (Some(OsStr::new("")), "/home/a/.local/share/tailrace/layout"),
            (None, "/home/a/.local/share/tailrace/layout"),
        ] {
            let table = given(&file(), xdg, Some(home)).unwrap();
            assert_eq!(table.key(), "layout");
            assert_eq!(table.path(), Ok(PathBuf::from(layout)));
            assert_eq!(
                named(&table),
                Ok(Source::new(Seam::Store, format!("layout {layout}")))
            );
        }
        let table = given(&file(), Some(OsStr::new("/data")), None).unwrap();
        assert_eq!(table.path(), Ok(PathBuf::from("/data/tailrace/layout")));
    }

    #[test]
    fn a_publishs_layout_is_made_at_its_first_operation_and_never_at_its_binding() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("data/tailrace/layout");
        let text = path.to_str().unwrap();
        let table = Table::given(&file(), "store", "layout", text.into());
        let store = bind(&table, true).unwrap();
        let cache = bind(&table, true).unwrap();
        assert!(!path.exists(), "binding made {}", path.display());
        assert!(matches!(
            cache.resolve(&Name::new("v0.1.0")),
            Err(Not::Absent(_))
        ));
        assert!(
            path.join("oci-layout").is_file(),
            "the first operation made no layout"
        );
        let put = store.put_bytes(b"abc").unwrap();
        assert_eq!(cache.read(&put.digest), Ok(Held::Bytes(b"abc".to_vec())));
        assert!(bind(&table, false).is_ok());
    }

    #[test]
    fn with_neither_the_data_directory_nor_the_home_the_given_layout_is_absent() {
        assert_eq!(
            given(&file(), None, None),
            Err(Not::absent(
                &Source::host(file()),
                "binding of the store: neither XDG_DATA_HOME nor HOME is set to place \
                 tailrace/layout"
            ))
        );
        assert!(given(&file(), Some(OsStr::new("")), None).is_err());
    }
}
