//! pack (84, 87; design D1 to D3): the output's one package manifest
//! found among its files.

use std::fs;
use std::path::{Path, PathBuf};

use tailrace_seams::{Answer, Not, Output, Why};

use super::CargoProjection;

/// What the manifest of a package is called.
const MANIFEST: &str = "Cargo.toml";

impl CargoProjection {
    /// The one manifest among `output`'s files that declares `[package]`
    /// (design D1); refused naming the output and its files where it left
    /// none or several, and unreadable naming a manifest that cannot be
    /// read as TOML, with the parser's words.
    pub(super) fn package_manifest(&self, output: &Output) -> Answer<PathBuf> {
        let mut found = Vec::new();
        for file in &output.files {
            let path = self.under_root(file);
            if path.file_name().is_none_or(|name| name != MANIFEST) {
                continue;
            }
            if declares_package(&path)? {
                found.push(path);
            }
        }
        match found.as_slice() {
            [one] => Ok(one.clone()),
            several => {
                let declared = match several {
                    [] => "no manifest declaring [package]".to_string(),
                    _ => format!(
                        "{} manifests declaring [package], {}",
                        several.len(),
                        several
                            .iter()
                            .map(|p| p.display().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                };
                let files = match output.files.as_slice() {
                    [] => "no file".to_string(),
                    files => files.join(", "),
                };
                Err(Not::refused(
                    &CargoProjection::source(),
                    Why::Said,
                    Some(format!(
                        "the output {} left {declared} among its files, {files}, and the cargo \
                         projection packs one crate",
                        output.name
                    )),
                ))
            }
        }
    }
}

/// Whether the manifest at `path` declares `[package]`, and not only a
/// workspace; unreadable naming it with the operating system's or the
/// parser's words.
fn declares_package(path: &Path) -> Answer<bool> {
    let unreadable = |said: String| {
        Not::unreadable(
            &CargoProjection::source(),
            path.display().to_string(),
            Some(said),
        )
    };
    let text = fs::read_to_string(path).map_err(|e| unreadable(e.to_string()))?;
    let table: toml::Table = toml::from_str(&text).map_err(|e| unreadable(e.to_string()))?;
    Ok(table.get("package").is_some_and(toml::Value::is_table))
}

#[cfg(test)]
mod tests {
    use tailrace_fakes::FakeStore;
    use tailrace_seams::OutputName;
    use tempfile::TempDir;

    use super::*;

    const PACKAGE: &str = "[package]\nname = \"fam\"\nversion = \"0.0.0\"\n";
    const WORKSPACE: &str = "[workspace]\nmembers = [\"fam\"]\n";

    /// A checkout holding `files`, and the projection of it.
    fn checkout(files: &[(&str, &str)]) -> (TempDir, CargoProjection) {
        let dir = TempDir::new().unwrap();
        for (path, text) in files {
            let at = dir.path().join(path);
            fs::create_dir_all(at.parent().unwrap()).unwrap();
            fs::write(at, text).unwrap();
        }
        let projection = CargoProjection::new(dir.path(), Box::new(FakeStore::new()));
        (dir, projection)
    }

    fn output(files: &[&str]) -> Output {
        Output {
            name: OutputName::new("fam"),
            files: files.iter().map(|f| f.to_string()).collect(),
        }
    }

    fn refused(answer: Answer<PathBuf>) -> String {
        match answer {
            Err(Not::Refused(r)) => {
                assert_eq!(r.source, CargoProjection::source());
                assert_eq!(r.why, Why::Said);
                r.said.unwrap()
            }
            other => panic!("answered {other:?}"),
        }
    }

    #[test]
    fn the_one_manifest_declaring_a_package_is_found_beside_a_workspaces() {
        let (dir, projection) = checkout(&[
            ("Cargo.toml", WORKSPACE),
            ("fam/Cargo.toml", PACKAGE),
            ("fam/src/lib.rs", ""),
        ]);
        let found = projection
            .package_manifest(&output(&["Cargo.toml", "fam/Cargo.toml", "fam/src/lib.rs"]))
            .unwrap();
        assert_eq!(found, dir.path().join("fam/Cargo.toml"));
        let absolute = dir.path().join("fam/Cargo.toml").display().to_string();
        assert_eq!(
            projection.package_manifest(&output(&[&absolute])),
            Ok(dir.path().join("fam/Cargo.toml"))
        );
    }

    #[test]
    fn an_output_with_no_package_manifest_is_refused_naming_it_and_its_files() {
        let (_dir, projection) = checkout(&[("Cargo.toml", WORKSPACE), ("fam/src/lib.rs", "")]);
        assert_eq!(
            refused(projection.package_manifest(&output(&["Cargo.toml", "fam/src/lib.rs"]))),
            "the output fam left no manifest declaring [package] among its files, Cargo.toml, \
             fam/src/lib.rs, and the cargo projection packs one crate"
        );
        assert!(refused(projection.package_manifest(&output(&[]))).contains("its files, no file"));
    }

    #[test]
    fn an_output_with_two_package_manifests_is_refused_naming_both() {
        let (dir, projection) = checkout(&[("a/Cargo.toml", PACKAGE), ("b/Cargo.toml", PACKAGE)]);
        let said = refused(projection.package_manifest(&output(&["a/Cargo.toml", "b/Cargo.toml"])));
        assert_eq!(
            said,
            format!(
                "the output fam left 2 manifests declaring [package], {}, {} among its files, \
                 a/Cargo.toml, b/Cargo.toml, and the cargo projection packs one crate",
                dir.path().join("a/Cargo.toml").display(),
                dir.path().join("b/Cargo.toml").display()
            )
        );
    }

    #[test]
    fn a_manifest_that_is_not_toml_is_unreadable_with_the_parsers_words() {
        let (dir, projection) = checkout(&[("fam/Cargo.toml", "[package\n")]);
        match projection.package_manifest(&output(&["fam/Cargo.toml"])) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, CargoProjection::source());
                assert_eq!(
                    u.what,
                    dir.path().join("fam/Cargo.toml").display().to_string()
                );
                let words = toml::from_str::<toml::Table>("[package\n")
                    .unwrap_err()
                    .to_string();
                assert_eq!(u.said, Some(words));
            }
            other => panic!("answered {other:?}"),
        }
    }
}
