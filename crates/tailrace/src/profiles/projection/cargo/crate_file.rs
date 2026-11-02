//! A crate's own bytes read (design D1, D8): the manifest Cargo
//! normalized into the top of the crate's one directory, and the place in
//! the repository it was packed from, read from the gzipped archive in
//! memory, with nothing extracted to disk.

use std::io::Read;

/// A crate's normalized manifest: the package's name and version, and
/// the whole of it, which an index line is written from.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct CrateManifest {
    pub name: String,
    pub version: String,
    pub table: toml::Table,
}

/// The normalized manifest `bytes`, a crate, hold at
/// `<directory>/Cargo.toml`; the words of the archive's reader or the
/// parser where it holds none that can be read.
pub(super) fn manifest(bytes: &[u8]) -> Result<CrateManifest, String> {
    let [text, _] = top_files(bytes, ["Cargo.toml", VCS_INFO])?;
    let text = text.ok_or("the crate holds no Cargo.toml in its directory")?;
    read_manifest(&text)
}

/// What a crate's `.cargo_vcs_info.json` names of where `cargo package`
/// packed it from, in a git checkout.
const VCS_INFO: &str = ".cargo_vcs_info.json";

/// The normalized manifest `bytes`, a crate, hold, and the package's
/// directory in the repository it was packed from, as its
/// `.cargo_vcs_info.json` names it in `path_in_vcs`, the empty path for
/// the repository's root (design D1); none where the crate holds no such
/// file or the file names none, as for a crate packed outside git.
pub(super) fn packed_from(bytes: &[u8]) -> Result<(CrateManifest, Option<String>), String> {
    let [text, vcs] = top_files(bytes, ["Cargo.toml", VCS_INFO])?;
    let text = text.ok_or("the crate holds no Cargo.toml in its directory")?;
    let manifest = read_manifest(&text)?;
    let path = match vcs {
        None => None,
        Some(vcs) => {
            let info: serde_json::Value = serde_json::from_str(&vcs)
                .map_err(|e| format!("the crate's {VCS_INFO} is no JSON: {e}"))?;
            info.get("path_in_vcs")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        }
    };
    Ok((manifest, path))
}

fn read_manifest(text: &str) -> Result<CrateManifest, String> {
    let table: toml::Table = toml::from_str(text).map_err(|e| e.to_string())?;
    let package = table
        .get("package")
        .and_then(toml::Value::as_table)
        .ok_or("the crate's Cargo.toml declares no [package]")?;
    let text_of = |key: &str| {
        package
            .get(key)
            .and_then(toml::Value::as_str)
            .map(str::to_string)
            .ok_or(format!("the crate's Cargo.toml names no package.{key}"))
    };
    Ok(CrateManifest {
        name: text_of("name")?,
        version: text_of("version")?,
        table: table.clone(),
    })
}

/// The text of each of `names` at the top of the crate's one directory,
/// none for one it does not hold.
fn top_files<const N: usize>(
    bytes: &[u8],
    names: [&str; N],
) -> Result<[Option<String>; N], String> {
    let mut found: [Option<String>; N] = std::array::from_fn(|_| None);
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(bytes));
    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?.into_owned();
        if path.components().count() != 2 {
            continue;
        }
        let Some(at) = names.iter().position(|name| path.ends_with(name)) else {
            continue;
        };
        let mut text = String::new();
        entry.read_to_string(&mut text).map_err(|e| e.to_string())?;
        found[at] = Some(text);
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::super::fixture::{crate_of, packed_crate};
    use super::*;

    #[test]
    fn the_normalized_manifest_names_the_package() {
        let bytes = crate_of(
            "fam-lib",
            "1.4.0",
            "[package]\nname = \"fam-lib\"\nversion = \"1.4.0\"\n",
        );
        let found = manifest(&bytes).unwrap();
        assert_eq!(
            (found.name.as_str(), found.version.as_str()),
            ("fam-lib", "1.4.0")
        );
    }

    #[test]
    fn bytes_that_are_no_crate_are_answered_in_the_readers_words() {
        let said = manifest(b"not gzip").unwrap_err();
        assert!(!said.is_empty());
        let no_manifest = crate_of("fam-lib", "1.4.0", "");
        let said = manifest(&no_manifest).unwrap_err();
        assert_eq!(said, "the crate's Cargo.toml declares no [package]");
    }

    #[test]
    fn a_crate_names_the_place_it_was_packed_from() {
        let manifest = "[package]\nname = \"fam-a\"\nversion = \"0.0.0\"\n";
        let (found, path) = packed_from(&packed_crate(
            "fam-a",
            "0.0.0",
            manifest,
            Some("crates/fam-a"),
        ))
        .unwrap();
        assert_eq!(
            (found.name.as_str(), path.as_deref()),
            ("fam-a", Some("crates/fam-a"))
        );
        let (_, path) = packed_from(&packed_crate("fam-a", "0.0.0", manifest, Some(""))).unwrap();
        assert_eq!(path.as_deref(), Some(""));
        let (_, path) = packed_from(&crate_of("fam-a", "0.0.0", manifest)).unwrap();
        assert_eq!(path, None);
    }
}
