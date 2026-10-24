//! The store seam's profiles (B.1), and the binding of the one the
//! host's `[store]` names.

pub mod layout;
mod oci;
pub mod registry;

use std::fs;
use std::path::Path;

use tailrace_seams::{Answer, Not, Seam, Source, Store};

use crate::bindings::Table;
use layout::Layout;
use registry::Registry;

/// The keys of `[store]`, one per profile.
const KEYS: [&str; 2] = ["layout", "registry"];

/// The store `[store]` names (92, C.1, C.2): `layout`, the OCI layout at
/// its path, made there when `make` and the path is missing or an empty
/// directory, as a publish's is the first time, and opened otherwise, so
/// a layout that is not there is unreadable in the operating system's
/// words; `registry`, the repository it names, which no binding makes,
/// whatever `make`; unreadable naming any other key.
pub fn bind(table: &Table, make: bool) -> Answer<Box<dyn Store>> {
    match table.key() {
        "layout" => {
            let path = table.path()?;
            let layout = if make && missing_or_empty(&path) {
                Layout::create(&path)?
            } else {
                Layout::open(&path)?
            };
            Ok(Box::new(layout))
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

/// Whether nothing is at `path`, or an empty directory is; a directory
/// that cannot be read is neither, and its open says why.
fn missing_or_empty(path: &Path) -> bool {
    match fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_none(),
        Err(e) => e.kind() == std::io::ErrorKind::NotFound,
    }
}
