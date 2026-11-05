//! pinned (137; design D5): the versions a checkout's root `Cargo.lock`
//! pins from a registry, as the lockfile spells them, and the range each
//! workspace member's manifest declares in the registry `timber` for a
//! name no lockfile pins, as the manifest writes it, so a checkout with a
//! manifest and no lockfile answers its ranges and a first install needs
//! no lockfile written by hand. It reads the files alone: no cargo runs,
//! since cargo may write the lockfile, and nothing is written.

use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use timber_seams::{Answer, EcosystemName, Not, Pin, Pinned, Range, Requirement, Version, Why};
use toml_edit::{DocumentMut, Item, Table};

use super::CargoProjection;

/// What Cargo calls the lockfile at a workspace's root.
const LOCKFILE: &str = "Cargo.lock";

/// What Cargo calls a package's or a workspace's manifest.
const MANIFEST: &str = "Cargo.toml";

/// The name a checkout's cargo configuration gives the registry the
/// host's serving process answers (C.11).
const REGISTRY_NAME: &str = "timber";

/// The prefixes of a lockfile package's `source` that name a registry,
/// one whose index is a git repository or a sparse one (Cargo's
/// lockfile).
const REGISTRY: [&str; 2] = ["registry+", "sparse+"];

/// The tables of a member's manifest that declare dependencies, each
/// also under each `target.<cfg>`.
const DEPENDENCIES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

impl CargoProjection {
    /// 137: the pins of the lockfile at the root of the checkout at
    /// `checkout`, under the projection's own where it is relative, then
    /// each range its members declare in `timber` for a name the
    /// lockfile pins at no version from a registry, each name and range
    /// once; absent naming the lockfile where the checkout holds none and
    /// declares nothing in `timber`, or holds neither file.
    pub(super) fn pins_and_ranges(&self, checkout: &str) -> Answer<Vec<Pin>> {
        let root = self.under_root(checkout);
        let lockfile = root.join(LOCKFILE);
        let pinned = match read(&lockfile)? {
            Some(text) => Some(pins(&lockfile, &text)?),
            None => None,
        };
        let manifest = root.join(MANIFEST);
        let declared = match read(&manifest)? {
            Some(text) => declared(&root, &manifest, &text)?,
            None => Vec::new(),
        };
        let held: BTreeSet<&EcosystemName> = pinned.iter().flatten().map(|p| &p.name).collect();
        let ranges: Vec<Requirement> = declared
            .into_iter()
            .filter(|r| !held.contains(&r.name))
            .collect();
        if pinned.is_none() && ranges.is_empty() {
            return Err(Not::absent(
                &CargoProjection::source(),
                format!("lockfile {}", lockfile.display()),
            ));
        }
        let pins = pinned.into_iter().flatten().map(Pin::Version);
        Ok(pins.chain(ranges.into_iter().map(Pin::Range)).collect())
    }
}

/// The text of the file at `path`, none where there is no file there;
/// unreadable naming it with the operating system's words otherwise.
fn read(path: &Path) -> Answer<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Not::unreadable(
            &CargoProjection::source(),
            path.display().to_string(),
            Some(e.to_string()),
        )),
    }
}

/// Each range the members of the workspace whose root manifest, at
/// `path` under `root`, is `text` declare in `timber`, by the name it
/// is of and its requirement as written, `*` where it writes none, each
/// name and range once, in the members' order: the root's own package,
/// then `workspace.members` expanded as Cargo expands them, less
/// `workspace.exclude`. A dependency with a `path` or a `git` is left
/// out, since Cargo builds it from there.
fn declared(root: &Path, path: &Path, text: &str) -> Answer<Vec<Requirement>> {
    let manifest = parsed(path, text)?;
    let workspace = manifest.get("workspace").and_then(toml::Value::as_table);
    let inherited = workspace
        .and_then(|w| w.get("dependencies"))
        .and_then(toml::Value::as_table);
    let mut members = Vec::new();
    if manifest.get("package").is_some() {
        members.push(manifest.clone());
    }
    if let Some(workspace) = workspace {
        let exclude = strings(path, workspace, "exclude")?;
        for dir in member_dirs(root, &strings(path, workspace, "members")?) {
            let relative = dir.strip_prefix(root).unwrap_or(&dir);
            if dir == root || exclude.iter().any(|ex| relative.starts_with(ex)) {
                continue;
            }
            let at = dir.join(MANIFEST);
            let text = read(&at)?.ok_or_else(|| {
                Not::unreadable(
                    &CargoProjection::source(),
                    at.display().to_string(),
                    Some(format!(
                        "the workspace at {} names it a member",
                        path.display()
                    )),
                )
            })?;
            members.push(parsed(&at, &text)?);
        }
    }
    let mut ranges: Vec<Requirement> = Vec::new();
    for member in &members {
        let mut tables = vec![member];
        if let Some(targets) = member.get("target").and_then(toml::Value::as_table) {
            tables.extend(targets.values().filter_map(toml::Value::as_table));
        }
        for table in tables {
            for kind in DEPENDENCIES {
                let Some(declared) = table.get(kind).and_then(toml::Value::as_table) else {
                    continue;
                };
                for (key, spec) in declared {
                    if let Some(range) = range_of(key, spec, inherited) {
                        if !ranges.contains(&range) {
                            ranges.push(range);
                        }
                    }
                }
            }
        }
    }
    Ok(ranges)
}

/// The range the dependency `key`, written `spec`, declares in
/// `timber`, written so or inherited from the workspace's
/// `inherited` dependencies; none for one of another registry, of
/// crates.io, or with a `path` or a `git`.
fn range_of(key: &str, spec: &toml::Value, inherited: Option<&toml::Table>) -> Option<Requirement> {
    let spec = spec.as_table()?;
    let from = if spec.get("workspace").and_then(toml::Value::as_bool) == Some(true) {
        inherited?.get(key)?.as_table()?
    } else {
        spec
    };
    let text = |key: &str| from.get(key).and_then(toml::Value::as_str);
    if text("registry") != Some(REGISTRY_NAME)
        || from.contains_key("path")
        || from.contains_key("git")
    {
        return None;
    }
    Some(Requirement {
        name: EcosystemName::new(text("package").unwrap_or(key)),
        range: Range::new(text("version").unwrap_or("*")),
    })
}

/// The directories `patterns`, relative to `root`, name, each expanded
/// as a glob, a pattern matching nothing naming its own path, as Cargo
/// reads `workspace.members`.
fn member_dirs(root: &Path, patterns: &[String]) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for pattern in patterns {
        let at = root.join(pattern);
        let matched: Vec<PathBuf> = glob::glob(&at.to_string_lossy())
            .map(|paths| {
                paths
                    .filter_map(Result::ok)
                    .filter(|p| p.is_dir())
                    .collect()
            })
            .unwrap_or_default();
        if matched.is_empty() {
            dirs.push(at);
        } else {
            dirs.extend(matched);
        }
    }
    dirs
}

/// The manifest `text`, at `path`, read as TOML; refused naming it with
/// the parser's words where it is not.
fn parsed(path: &Path, text: &str) -> Answer<toml::Table> {
    toml::from_str(text).map_err(|e| {
        Not::refused(
            &CargoProjection::source(),
            Why::Said,
            Some(format!(
                "{} is no manifest Cargo reads: {e}",
                path.display()
            )),
        )
    })
}

/// The list of strings `table` writes at `key`, none where it writes
/// none; refused naming the manifest at `path` where it is another
/// value.
fn strings(path: &Path, table: &toml::Table, key: &str) -> Answer<Vec<String>> {
    let Some(value) = table.get(key) else {
        return Ok(Vec::new());
    };
    value
        .as_array()
        .and_then(|values| {
            values
                .iter()
                .map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .ok_or_else(|| {
            Not::refused(
                &CargoProjection::source(),
                Why::Said,
                Some(format!(
                    "{} is no manifest Cargo reads: workspace.{key} is no list of strings",
                    path.display()
                )),
            )
        })
}

/// Each package the lockfile `text`, at `path`, pins from a registry, by
/// its name and version as written, in the order it gives them; a
/// package with no source, the workspace's own or a path's, and one from
/// git, left out. Refused naming the file with the parser's words where
/// it is not TOML, and naming the file and the package where one names
/// no name or version.
fn pins(path: &Path, text: &str) -> Answer<Vec<Pinned>> {
    let doc: DocumentMut = text
        .parse()
        .map_err(|e: toml_edit::TomlError| refused(path, e.to_string()))?;
    let packages = match doc.get("package") {
        None => return Ok(Vec::new()),
        Some(item) => item
            .as_array_of_tables()
            .ok_or_else(|| refused(path, "its package is no array of tables".into()))?,
    };
    let mut pinned = Vec::new();
    for (i, package) in packages.iter().enumerate() {
        let from_registry = text_of(package, "source")
            .is_some_and(|source| REGISTRY.iter().any(|p| source.starts_with(p)));
        if !from_registry {
            continue;
        }
        let named = |key: &str| {
            text_of(package, key).ok_or_else(|| {
                refused(
                    path,
                    format!("its package {} names no {key} as a string", i + 1),
                )
            })
        };
        pinned.push(Pinned {
            name: EcosystemName::new(named("name")?),
            version: Version::new(named("version")?),
        });
    }
    Ok(pinned)
}

fn text_of<'a>(table: &'a Table, key: &str) -> Option<&'a str> {
    table.get(key).and_then(Item::as_str)
}

/// `path` refused as no lockfile Cargo reads, for `said`.
fn refused(path: &Path, said: String) -> Not {
    Not::refused(
        &CargoProjection::source(),
        Why::Said,
        Some(format!(
            "{} is no lockfile Cargo reads: {said}",
            path.display()
        )),
    )
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use timber_fakes::FakeStore;
    use timber_seams::{Pin, Projection};

    use super::*;

    const SERVED: &str = "sparse+http://127.0.0.1:7420/agentplot/family/cargo/";

    fn range(name: &str, range: &str) -> Pin {
        Pin::Range(Requirement {
            name: EcosystemName::new(name),
            range: Range::new(range),
        })
    }

    fn pinned_at(projection: &CargoProjection, dir: &TempDir) -> Answer<Vec<Pin>> {
        projection.pinned(&dir.path().display().to_string())
    }

    fn pin(name: &str, version: &str) -> Pin {
        Pin::Version(Pinned {
            name: EcosystemName::new(name),
            version: Version::new(version),
        })
    }

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

    /// Every file under `dir`, by its path, with its bytes.
    fn tree(dir: &Path) -> Vec<(String, Vec<u8>)> {
        let mut found: Vec<(String, Vec<u8>)> = walk(dir)
            .into_iter()
            .map(|p| {
                let bytes = fs::read(&p).unwrap();
                (p.strip_prefix(dir).unwrap().display().to_string(), bytes)
            })
            .collect();
        found.sort();
        found
    }

    fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
        let mut found = Vec::new();
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                found.extend(walk(&path));
            } else {
                found.push(path);
            }
        }
        found
    }

    // Requirement: The pins a Cargo lockfile holds and the ranges its
    // manifests declare

    #[test]
    fn a_lockfiles_registry_pins_are_answered_as_written_and_no_member_or_git_package() {
        let lockfile = format!(
            "# This file is automatically @generated by Cargo.\n\
             version = 4\n\
             \n\
             [[package]]\n\
             name = \"consumer\"\n\
             version = \"0.1.0\"\n\
             dependencies = [\n \"fam-b\",\n \"serde\",\n \"vendored\",\n]\n\
             \n\
             [[package]]\n\
             name = \"fam-b\"\n\
             version = \"1.1.0\"\n\
             source = \"{SERVED}\"\n\
             checksum = \"c0ffee\"\n\
             \n\
             [[package]]\n\
             name = \"serde\"\n\
             version = \"1.0.228\"\n\
             source = \"registry+https://github.com/rust-lang/crates.io-index\"\n\
             checksum = \"beef\"\n\
             \n\
             [[package]]\n\
             name = \"vendored\"\n\
             version = \"0.2.0\"\n\
             source = \"git+https://example.invalid/vendored#0123456789abcdef\"\n"
        );
        let (dir, projection) = checkout(&[("Cargo.lock", &lockfile)]);
        let before = tree(dir.path());
        assert_eq!(
            projection
                .pinned(&dir.path().display().to_string())
                .unwrap(),
            [pin("fam-b", "1.1.0"), pin("serde", "1.0.228")]
        );
        assert_eq!(tree(dir.path()), before, "pinned wrote");
    }

    #[test]
    fn a_relative_checkout_is_under_the_projections_own() {
        let (_dir, projection) = checkout(&[(
            "consumer/Cargo.lock",
            &format!(
                "version = 4\n\n[[package]]\nname = \"fam-b\"\nversion = \"1.1.0\"\n\
                 source = \"{SERVED}\"\n"
            ),
        )]);
        assert_eq!(
            projection.pinned("consumer").unwrap(),
            [pin("fam-b", "1.1.0")]
        );
    }

    #[test]
    fn a_checkout_with_no_lockfile_is_absent_naming_its_lockfile() {
        let (dir, projection) = checkout(&[]);
        match projection.pinned(&dir.path().display().to_string()) {
            Err(Not::Absent(a)) => {
                assert_eq!(a.source, CargoProjection::source());
                assert_eq!(
                    a.what,
                    format!("lockfile {}", dir.path().join(LOCKFILE).display())
                );
            }
            other => panic!("answered {other:?}"),
        }
        assert_eq!(tree(dir.path()), [], "pinned wrote");
    }

    #[test]
    fn a_lockfile_that_is_not_toml_is_refused_naming_it_with_the_parsers_words() {
        let (dir, projection) = checkout(&[("Cargo.lock", "[[package]\n")]);
        let words = "[[package]\n"
            .parse::<DocumentMut>()
            .unwrap_err()
            .to_string();
        match projection.pinned(&dir.path().display().to_string()) {
            Err(Not::Refused(r)) => {
                assert_eq!(r.source, CargoProjection::source());
                assert_eq!(
                    r.said,
                    Some(format!(
                        "{} is no lockfile Cargo reads: {words}",
                        dir.path().join(LOCKFILE).display()
                    ))
                );
            }
            other => panic!("answered {other:?}"),
        }
    }

    #[test]
    fn a_registry_package_with_no_version_is_refused_naming_it() {
        let (dir, projection) = checkout(&[(
            "Cargo.lock",
            &format!("version = 4\n\n[[package]]\nname = \"fam-b\"\nsource = \"{SERVED}\"\n"),
        )]);
        match projection.pinned(&dir.path().display().to_string()) {
            Err(Not::Refused(r)) => assert_eq!(
                r.said,
                Some(format!(
                    "{} is no lockfile Cargo reads: its package 1 names no version as a string",
                    dir.path().join(LOCKFILE).display()
                ))
            ),
            other => panic!("answered {other:?}"),
        }
    }

    #[test]
    fn a_lockfile_pinning_nothing_answers_no_pins() {
        let (dir, projection) = checkout(&[("Cargo.lock", "version = 4\n")]);
        assert_eq!(
            projection
                .pinned(&dir.path().display().to_string())
                .unwrap(),
            []
        );
    }

    /// A workspace of three members and the root's own package: `a`
    /// declares `fam-b` and a dev `fam-dev`, `b` inherits `fam-c` and
    /// renames `fam-d`, `c` declares `fam-e` with a path, a target's
    /// `fam-b` at another range, and crates of another registry and of
    /// crates.io; `excluded` is left out.
    const ROOT: &str = "[package]\nname = \"consumer\"\nversion = \"0.1.0\"\n\n\
        [dependencies]\nfam-root = { registry = \"timber\" }\n\n\
        [workspace]\nmembers = [\"crates/*\", \"tools/c\"]\nexclude = [\"crates/excluded\"]\n\n\
        [workspace.dependencies]\nfam-c = { version = \"0.3\", registry = \"timber\" }\n\
        serde = \"1\"\n";
    const A: &str = "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n\
        [dependencies]\nfam-b = { version = \"1\", registry = \"timber\" }\n\
        serde = { workspace = true }\n\n\
        [dev-dependencies]\nfam-dev = { version = \"2\", registry = \"timber\" }\n";
    const B: &str = "[package]\nname = \"b\"\nversion = \"0.1.0\"\n\n\
        [dependencies]\nfam-c = { workspace = true, features = [\"full\"] }\n\
        renamed = { package = \"fam-d\", version = \"^1.2\", registry = \"timber\" }\n\n\
        [build-dependencies.fam-b]\nversion = \"1\"\nregistry = \"timber\"\n";
    const C: &str = "[package]\nname = \"c\"\nversion = \"0.1.0\"\n\n\
        [dependencies]\nfam-e = { version = \"1\", path = \"../e\", registry = \"timber\" }\n\
        vendored = { version = \"2\", registry = \"elsewhere\" }\n\
        libc = \"0.2\"\n\n\
        [target.'cfg(unix)'.dependencies]\nfam-b = { version = \"1.1\", registry = \"timber\" }\n";
    const EXCLUDED: &str = "[package]\nname = \"excluded\"\nversion = \"0.1.0\"\n\n\
        [dependencies]\nfam-x = { version = \"1\", registry = \"timber\" }\n";

    fn workspace(lockfile: Option<&str>) -> (TempDir, CargoProjection) {
        let mut files = vec![
            ("Cargo.toml", ROOT),
            ("crates/a/Cargo.toml", A),
            ("crates/b/Cargo.toml", B),
            ("tools/c/Cargo.toml", C),
            ("crates/excluded/Cargo.toml", EXCLUDED),
        ];
        if let Some(lockfile) = lockfile {
            files.push(("Cargo.lock", lockfile));
        }
        checkout(&files)
    }

    #[test]
    fn a_manifest_and_no_lockfile_answers_its_members_own_inherited_and_renamed_ranges() {
        let (dir, projection) = workspace(None);
        let before = tree(dir.path());
        assert_eq!(
            pinned_at(&projection, &dir).unwrap(),
            [
                range("fam-root", "*"),
                range("fam-b", "1"),
                range("fam-dev", "2"),
                range("fam-c", "0.3"),
                range("fam-d", "^1.2"),
                range("fam-b", "1.1"),
            ]
        );
        assert_eq!(tree(dir.path()), before, "pinned wrote");
    }

    #[test]
    fn a_name_the_lockfile_pins_from_a_registry_is_answered_as_its_pin_and_no_range() {
        let lockfile = format!(
            "version = 4\n\n\
             [[package]]\nname = \"fam-b\"\nversion = \"1.1.0\"\nsource = \"{SERVED}\"\n\n\
             [[package]]\nname = \"fam-c\"\nversion = \"0.3.0\"\n"
        );
        let (dir, projection) = workspace(Some(&lockfile));
        assert_eq!(
            pinned_at(&projection, &dir).unwrap(),
            [
                pin("fam-b", "1.1.0"),
                range("fam-root", "*"),
                range("fam-dev", "2"),
                range("fam-c", "0.3"),
                range("fam-d", "^1.2"),
            ]
        );
    }

    #[test]
    fn a_manifest_declaring_nothing_in_timber_and_no_lockfile_is_absent_naming_the_lockfile() {
        let (dir, projection) = checkout(&[(
            "Cargo.toml",
            "[package]\nname = \"consumer\"\nversion = \"0.1.0\"\n\n\
             [dependencies]\nserde = \"1\"\n",
        )]);
        let before = tree(dir.path());
        match pinned_at(&projection, &dir) {
            Err(Not::Absent(a)) => assert_eq!(
                a.what,
                format!("lockfile {}", dir.path().join(LOCKFILE).display())
            ),
            other => panic!("answered {other:?}"),
        }
        assert_eq!(tree(dir.path()), before, "pinned wrote");
    }

    #[test]
    fn a_lockfile_and_a_manifest_declaring_nothing_answers_the_pins_alone() {
        let (dir, projection) = checkout(&[
            (
                "Cargo.toml",
                "[package]\nname = \"consumer\"\nversion = \"0.1.0\"\n",
            ),
            ("Cargo.lock", "version = 4\n"),
        ]);
        assert_eq!(pinned_at(&projection, &dir).unwrap(), []);
    }

    #[test]
    fn a_manifest_that_is_not_toml_is_refused_naming_it_with_the_parsers_words() {
        let words = toml::from_str::<toml::Table>("[package\n")
            .unwrap_err()
            .to_string();
        let (dir, projection) = checkout(&[
            ("Cargo.toml", "[workspace]\nmembers = [\"a\"]\n"),
            ("a/Cargo.toml", "[package\n"),
        ]);
        match pinned_at(&projection, &dir) {
            Err(Not::Refused(r)) => {
                assert_eq!(r.source, CargoProjection::source());
                assert_eq!(
                    r.said,
                    Some(format!(
                        "{} is no manifest Cargo reads: {words}",
                        dir.path().join("a").join(MANIFEST).display()
                    ))
                );
            }
            other => panic!("answered {other:?}"),
        }
    }

    #[test]
    fn a_member_with_no_manifest_is_unreadable_naming_it() {
        let (dir, projection) = checkout(&[("Cargo.toml", "[workspace]\nmembers = [\"gone\"]\n")]);
        match pinned_at(&projection, &dir) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(
                    u.what,
                    dir.path().join("gone").join(MANIFEST).display().to_string()
                )
            }
            other => panic!("answered {other:?}"),
        }
    }
}
