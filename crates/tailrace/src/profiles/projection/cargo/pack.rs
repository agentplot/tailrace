//! pack (84, 87; design D1 to D3): the output's one package manifest
//! found among its files, its package's name and workspace read with
//! `cargo metadata`, the version written into the one value that holds
//! it, the crate packed by `cargo package` into a temporary target
//! directory, the manifest's bytes written back whatever cargo answered,
//! and the crate refused where its own manifest names another version,
//! since every pack of a revision edits one checkout.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tailrace_seams::{Answer, Not, Output, Version, Why};
use toml_edit::{DocumentMut, Item};

use super::{crate_file, CargoProjection};
use crate::profiles::run::GIT_REPOSITORY_ENV;

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

    /// The crate of the package whose manifest is `manifest`, packed at
    /// `version`, and the package's name (design D2): the version written
    /// with `toml_edit` into `package.version`, or into the workspace's
    /// `workspace.package.version` where the package inherits it, cargo
    /// run, and the file's own bytes written back before this answers,
    /// whatever cargo answered.
    pub(super) fn packed(&self, manifest: &Path, version: &Version) -> Answer<(String, Vec<u8>)> {
        let package = self.metadata(manifest)?;
        let (holder, held) = holder(manifest, &package.workspace_root)?;
        let original = fs::read(&holder).map_err(|e| unreadable(&holder, e.to_string()))?;
        let edited = with_version(&holder, &original, held, version)?;
        let restore =
            || fs::write(&holder, &original).map_err(|e| unreadable(&holder, e.to_string()));
        let crate_file = match fs::write(&holder, edited) {
            Ok(()) => self.package(manifest),
            Err(e) => Err(unreadable(&holder, e.to_string())),
        };
        restore()?;
        let bytes = crate_file?;
        names_package(&bytes, manifest, &package.name, version)?;
        Ok((package.name, bytes))
    }

    /// The name of the package whose manifest is `manifest`, and the root
    /// of its workspace, as `cargo metadata` answers them (design D1).
    fn metadata(&self, manifest: &Path) -> Answer<Package> {
        let args = [
            OsStr::new("metadata"),
            OsStr::new("--no-deps"),
            OsStr::new("--format-version"),
            OsStr::new("1"),
            OsStr::new("--manifest-path"),
            manifest.as_os_str(),
        ];
        let ran = self
            .cargo
            .output(manifest.parent(), &args, GIT_REPOSITORY_ENV, &[0])?;
        let metadata: Metadata = serde_json::from_slice(&ran.stdout)
            .map_err(|e| self.cargo.unreadable(&args, e.to_string()))?;
        let wanted = canonical(manifest);
        let name = metadata
            .packages
            .into_iter()
            .find(|p| canonical(&p.manifest_path) == wanted)
            .map(|p| p.name)
            .ok_or_else(|| {
                self.cargo.unreadable(
                    &args,
                    format!("cargo metadata names no package at {}", manifest.display()),
                )
            })?;
        Ok(Package {
            name,
            workspace_root: metadata.workspace_root,
        })
    }

    /// The bytes of the one crate `cargo package` makes of the package at
    /// `manifest`, in a temporary target directory removed after, so
    /// nothing is left in the checkout's own.
    fn package(&self, manifest: &Path) -> Answer<Vec<u8>> {
        let target = tempfile::TempDir::new()
            .map_err(|e| unreadable(&std::env::temp_dir(), e.to_string()))?;
        let args = [
            OsStr::new("package"),
            OsStr::new("--no-verify"),
            OsStr::new("--allow-dirty"),
            OsStr::new("--manifest-path"),
            manifest.as_os_str(),
            OsStr::new("--target-dir"),
            target.path().as_os_str(),
        ];
        self.cargo
            .output(manifest.parent(), &args, GIT_REPOSITORY_ENV, &[0])?;
        let dir = target.path().join("package");
        let entries = fs::read_dir(&dir).map_err(|e| unreadable(&dir, e.to_string()))?;
        let mut crates = Vec::new();
        for entry in entries {
            let path = entry.map_err(|e| unreadable(&dir, e.to_string()))?.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "crate") {
                crates.push(path);
            }
        }
        match crates.as_slice() {
            [one] => fs::read(one).map_err(|e| unreadable(one, e.to_string())),
            several => Err(unreadable(
                &dir,
                format!("cargo package left {} crates, and packs one", several.len()),
            )),
        }
    }
}

/// Refused naming both where the crate `bytes`, packed of the package at
/// `manifest`, names another package or version than `name` and
/// `version` (design D2), as when two packs edit one checkout at once;
/// unreadable where its manifest cannot be read, with the reader's words.
fn names_package(bytes: &[u8], manifest: &Path, name: &str, version: &Version) -> Answer<()> {
    let what = || format!("the crate cargo packed of {}", manifest.display());
    let found = crate_file::manifest(bytes)
        .map_err(|said| Not::unreadable(&CargoProjection::source(), what(), Some(said)))?;
    if found.name == name && found.version == version.as_str() {
        return Ok(());
    }
    Err(Not::refused(
        &CargoProjection::source(),
        Why::Said,
        Some(format!(
            "{} names {} {}, and the version given is {name} {version}",
            what(),
            found.name,
            found.version
        )),
    ))
}

/// What `cargo metadata` answers that pack reads.
#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Listed>,
    workspace_root: PathBuf,
}

#[derive(Deserialize)]
struct Listed {
    name: String,
    manifest_path: PathBuf,
}

/// A package's name, and the root of its workspace.
struct Package {
    name: String,
    workspace_root: PathBuf,
}

/// The value that holds a package's version: its own `package.version`,
/// or its workspace's `workspace.package.version`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Held {
    Package,
    Workspace,
}

impl Held {
    fn keys(self) -> &'static [&'static str] {
        match self {
            Held::Package => &["package", "version"],
            Held::Workspace => &["workspace", "package", "version"],
        }
    }
}

/// The file that holds the version of the package at `manifest`, and
/// which value of it: the workspace's manifest under `workspace_root`
/// where the package writes `version.workspace = true`, else its own.
fn holder(manifest: &Path, workspace_root: &Path) -> Answer<(PathBuf, Held)> {
    let text = fs::read_to_string(manifest).map_err(|e| unreadable(manifest, e.to_string()))?;
    let doc: DocumentMut = text
        .parse()
        .map_err(|e: toml_edit::TomlError| unreadable(manifest, e.to_string()))?;
    let version = doc.get("package").and_then(|p| p.get("version"));
    match version {
        None => Ok((manifest.to_path_buf(), Held::Package)),
        Some(item) if item.is_str() => Ok((manifest.to_path_buf(), Held::Package)),
        Some(item)
            if item
                .as_table_like()
                .and_then(|t| t.get("workspace"))
                .and_then(Item::as_bool)
                == Some(true) =>
        {
            Ok((workspace_root.join("Cargo.toml"), Held::Workspace))
        }
        Some(_) => Err(Not::refused(
            &CargoProjection::source(),
            Why::Said,
            Some(format!(
                "{} names its package's version neither as a string nor as its workspace's",
                manifest.display()
            )),
        )),
    }
}

/// `original`, the bytes of the manifest at `path`, with the value
/// `held` names set to `version` and the rest of the file as it was:
/// the value's own comments and spacing kept, and the value written where
/// a package's own manifest names none.
fn with_version(path: &Path, original: &[u8], held: Held, version: &Version) -> Answer<String> {
    let text = std::str::from_utf8(original).map_err(|e| unreadable(path, e.to_string()))?;
    let mut doc: DocumentMut = text
        .parse()
        .map_err(|e: toml_edit::TomlError| unreadable(path, e.to_string()))?;
    let keys = held.keys();
    let (last, tables) = keys.split_last().expect("a value is named by keys");
    let mut table = doc.as_item_mut();
    for key in tables {
        table = table.get_mut(key).ok_or_else(|| names_none(path, keys))?;
    }
    let table = table
        .as_table_like_mut()
        .ok_or_else(|| names_none(path, keys))?;
    let mut value = toml_edit::Value::from(version.as_str());
    match table.get_mut(last) {
        Some(item) => {
            if let Some(old) = item.as_value() {
                *value.decor_mut() = old.decor().clone();
            }
            *item = Item::Value(value);
        }
        None if held == Held::Package => {
            table.insert(last, Item::Value(value));
        }
        None => return Err(names_none(path, keys)),
    }
    Ok(doc.to_string())
}

fn names_none(path: &Path, keys: &[&str]) -> Not {
    unreadable(path, format!("the manifest names no {}", keys.join(".")))
}

/// `path` canonical where it can be made so, as cargo names manifests.
fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

/// `path` could not be read or written, for `said`.
fn unreadable(path: &Path, said: String) -> Not {
    Not::unreadable(
        &CargoProjection::source(),
        path.display().to_string(),
        Some(said),
    )
}

/// Whether the manifest at `path` declares `[package]`, and not only a
/// workspace; unreadable naming it with the operating system's or the
/// parser's words.
fn declares_package(path: &Path) -> Answer<bool> {
    let text = fs::read_to_string(path).map_err(|e| unreadable(path, e.to_string()))?;
    let table: toml::Table = toml::from_str(&text).map_err(|e| unreadable(path, e.to_string()))?;
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

    #[test]
    fn a_packages_own_version_is_set_and_the_rest_of_the_file_kept() {
        let original =
            "# the crate\n[package]\nname = \"fam\"\nversion = \"0.0.0\" # set at publish\n\
                        edition = \"2021\"\n\n[dependencies]\nserde = \"1\"\n";
        let edited = with_version(
            Path::new("Cargo.toml"),
            original.as_bytes(),
            Held::Package,
            &Version::new("1.4.0"),
        )
        .unwrap();
        assert_eq!(
            edited,
            original.replace("\"0.0.0\"", "\"1.4.0\""),
            "more than the version changed"
        );
    }

    #[test]
    fn a_workspaces_version_is_set_where_the_package_inherits_it() {
        let (dir, _projection) = checkout(&[
            (
                "Cargo.toml",
                "[workspace]\nmembers = [\"fam\"]\n\n[workspace.package]\nversion = \"0.0.0\"\n",
            ),
            (
                "fam/Cargo.toml",
                "[package]\nname = \"fam\"\nversion.workspace = true\n",
            ),
        ]);
        let manifest = dir.path().join("fam/Cargo.toml");
        let (holder_path, held) = holder(&manifest, dir.path()).unwrap();
        assert_eq!(
            (holder_path.clone(), held),
            (dir.path().join("Cargo.toml"), Held::Workspace)
        );
        let original = fs::read(&holder_path).unwrap();
        let edited =
            with_version(&holder_path, &original, held, &Version::new("2.0.0-pre.1")).unwrap();
        assert_eq!(
            edited,
            "[workspace]\nmembers = [\"fam\"]\n\n[workspace.package]\nversion = \"2.0.0-pre.1\"\n"
        );
        let inline = dir.path().join("fam/Cargo.toml");
        fs::write(
            &inline,
            "[package]\nname = \"fam\"\nversion = { workspace = true }\n",
        )
        .unwrap();
        assert_eq!(holder(&inline, dir.path()).unwrap().1, Held::Workspace);
    }

    #[test]
    fn a_package_naming_no_version_is_given_one() {
        let original = "[package]\nname = \"fam\"\n";
        let edited = with_version(
            Path::new("Cargo.toml"),
            original.as_bytes(),
            Held::Package,
            &Version::new("1.4.0"),
        )
        .unwrap();
        assert_eq!(edited, "[package]\nname = \"fam\"\nversion = \"1.4.0\"\n");
    }

    #[test]
    fn a_workspace_naming_no_version_is_unreadable_naming_the_value() {
        match with_version(
            Path::new("Cargo.toml"),
            b"[workspace]\nmembers = []\n",
            Held::Workspace,
            &Version::new("1.4.0"),
        ) {
            Err(Not::Unreadable(u)) => assert_eq!(
                u.said.as_deref(),
                Some("the manifest names no workspace.package.version")
            ),
            other => panic!("answered {other:?}"),
        }
    }

    #[test]
    fn a_crate_naming_another_version_is_refused_naming_both() {
        let manifest = Path::new("/checkout/fam/Cargo.toml");
        let bytes = super::super::fixture::crate_of(
            "fam",
            "1.4.1",
            "[package]\nname = \"fam\"\nversion = \"1.4.1\"\n",
        );
        assert_eq!(
            names_package(&bytes, manifest, "fam", &Version::new("1.4.1")),
            Ok(())
        );
        match names_package(&bytes, manifest, "fam", &Version::new("1.4.0")) {
            Err(Not::Refused(r)) => {
                assert_eq!(r.source, CargoProjection::source());
                assert_eq!(
                    r.said.as_deref(),
                    Some(
                        "the crate cargo packed of /checkout/fam/Cargo.toml names fam 1.4.1, \
                         and the version given is fam 1.4.0"
                    )
                );
            }
            other => panic!("answered {other:?}"),
        }
        match names_package(b"no crate", manifest, "fam", &Version::new("1.4.0")) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.what, "the crate cargo packed of /checkout/fam/Cargo.toml")
            }
            other => panic!("answered {other:?}"),
        }
    }
}
