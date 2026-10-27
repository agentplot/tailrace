//! pinned (137; design D5): the versions a checkout's root `Cargo.lock`
//! pins from a registry, as the lockfile spells them, read with
//! `toml_edit` and nothing else: no cargo runs, since cargo may write the
//! lockfile, and nothing is written.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use tailrace_seams::{Answer, EcosystemName, Not, Pinned, Version, Why};
use toml_edit::{DocumentMut, Item, Table};

use super::CargoProjection;

/// What Cargo calls the lockfile at a workspace's root.
const LOCKFILE: &str = "Cargo.lock";

/// The prefixes of a lockfile package's `source` that name a registry,
/// one whose index is a git repository or a sparse one (Cargo's
/// lockfile).
const REGISTRY: [&str; 2] = ["registry+", "sparse+"];

impl CargoProjection {
    /// 137: the pins of the lockfile at the root of the checkout at
    /// `checkout`, under the projection's own where it is relative.
    pub(super) fn lockfile_pins(&self, checkout: &str) -> Answer<Vec<Pinned>> {
        let path = self.under_root(checkout).join(LOCKFILE);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                return Err(Not::absent(
                    &CargoProjection::source(),
                    format!("lockfile {}", path.display()),
                ))
            }
            Err(e) => {
                return Err(Not::unreadable(
                    &CargoProjection::source(),
                    path.display().to_string(),
                    Some(e.to_string()),
                ))
            }
        };
        pins(&path, &text)
    }
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
    use tailrace_fakes::FakeStore;
    use tailrace_seams::{Pin, Projection};
    use tempfile::TempDir;

    use super::*;

    const SERVED: &str = "sparse+http://127.0.0.1:7420/agentplot/family/cargo/";

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
}
