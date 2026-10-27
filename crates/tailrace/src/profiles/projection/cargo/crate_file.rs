//! A crate's own bytes read (design D5): the manifest Cargo normalized
//! into the top of the crate's one directory, read from the gzipped
//! archive in memory, with nothing extracted to disk.

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
    let text = manifest_text(bytes)?;
    let table: toml::Table = toml::from_str(&text).map_err(|e| e.to_string())?;
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

fn manifest_text(bytes: &[u8]) -> Result<String, String> {
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(bytes));
    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?.into_owned();
        if path.components().count() == 2 && path.ends_with("Cargo.toml") {
            let mut text = String::new();
            entry.read_to_string(&mut text).map_err(|e| e.to_string())?;
            return Ok(text);
        }
    }
    Err("the crate holds no Cargo.toml in its directory".into())
}

#[cfg(test)]
mod tests {
    use super::super::fixture::crate_of;
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
}
