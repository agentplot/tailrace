//! The store seam's profiles (B.1), and the binding of the one the
//! host's `[store]` names.

pub mod layout;
mod oci;
pub mod registry;

use std::fs;
use std::path::Path;

use tailrace_seams::{Answer, Store};

use crate::bindings::Table;
use layout::Layout;

/// The store `[store]` names (92, C.2): `layout`, the OCI layout at its
/// path, made there when `make` and the path is missing or an empty
/// directory, as a publish's is the first time, and opened otherwise, so
/// a layout that is not there is unreadable in the operating system's
/// words; unreadable naming any other key.
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
        _ => Err(table.unknown(&["layout"])),
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
