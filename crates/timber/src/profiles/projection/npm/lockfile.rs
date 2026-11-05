//! npm's pins (137; design D7): every package a checkout's
//! `npm-shrinkwrap.json`, else its `package-lock.json`, pins, by its own
//! name and its version as the lockfile spells them, each once. Only
//! the lockfiles npm 7 and later write, of `lockfileVersion` 2 or 3, are
//! read: their `packages` name every installed package by its path under
//! `node_modules`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use timber_seams::{Answer, EcosystemName, Not, Pinned, Version};

use super::NpmProjection;

/// The lockfiles npm reads, the one it prefers first.
const LOCKFILES: [&str; 2] = ["npm-shrinkwrap.json", "package-lock.json"];

/// The lockfile npm reads in the checkout at `checkout`: its
/// `npm-shrinkwrap.json` where it holds one, else its
/// `package-lock.json`; none where it holds neither.
pub(super) fn find(checkout: &Path) -> Option<PathBuf> {
    LOCKFILES
        .iter()
        .map(|name| checkout.join(name))
        .find(|path| path.is_file())
}

/// The lockfile at `path` unreadable, naming it, with `said`.
fn unreadable(path: &Path, what: Option<&str>, said: Option<String>) -> Not {
    let file = path.display().to_string();
    let what = match what {
        Some(what) => format!("{file}, {what}"),
        None => file,
    };
    Not::unreadable(&NpmProjection::source(), what, said)
}

/// 137: every package the lockfile at `path` pins, by the entry's `name`
/// where it has one, an alias's target, and otherwise by its path's part
/// after its last `node_modules/`, at its `version`, each once and in
/// order of name and version. The root entry, a linked entry and a
/// workspace's own folder, which no registry holds, are passed over. A
/// lockfile of `lockfileVersion` 1, or one that does not parse, is
/// unreadable naming the file (58).
pub(super) fn pins(path: &Path) -> Answer<Vec<Pinned>> {
    let text = fs::read_to_string(path).map_err(|e| unreadable(path, None, Some(e.to_string())))?;
    let lockfile: Value =
        serde_json::from_str(&text).map_err(|e| unreadable(path, None, Some(e.to_string())))?;
    match lockfile.get("lockfileVersion").and_then(Value::as_u64) {
        Some(version) if version >= 2 => {}
        Some(version) => {
            return Err(unreadable(
                path,
                Some(&format!(
                    "of lockfileVersion {version}, which npm 7 and later rewrite"
                )),
                None,
            ))
        }
        None => {
            return Err(unreadable(
                path,
                Some("which names no lockfileVersion"),
                None,
            ))
        }
    }
    let Some(packages) = lockfile.get("packages").and_then(Value::as_object) else {
        return Err(unreadable(path, Some("which holds no packages"), None));
    };
    let mut pinned = BTreeSet::new();
    for (key, entry) in packages {
        let Some((_, installed)) = key.rsplit_once("node_modules/") else {
            continue;
        };
        if entry.get("link").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        let Some(version) = entry.get("version").and_then(Value::as_str) else {
            continue;
        };
        let name = entry
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(installed);
        pinned.insert(Pinned {
            name: EcosystemName::new(name),
            version: Version::new(version),
        });
    }
    Ok(pinned.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tempfile::TempDir;

    use super::*;

    fn pin(name: &str, version: &str) -> Pinned {
        Pinned {
            name: EcosystemName::new(name),
            version: Version::new(version),
        }
    }

    fn write(dir: &Path, name: &str, lockfile: &Value) {
        fs::write(dir.join(name), lockfile.to_string()).unwrap();
    }

    fn lockfile(packages: Value) -> Value {
        json!({
            "name": "consumer",
            "version": "1.0.0",
            "lockfileVersion": 3,
            "requires": true,
            "packages": packages,
        })
    }

    #[test]
    fn a_lockfiles_pins_are_named_as_it_spells_them_each_once() {
        let dir = TempDir::new().unwrap();
        write(
            dir.path(),
            "package-lock.json",
            &lockfile(json!({
                "": { "name": "consumer", "version": "1.0.0" },
                "node_modules/@family/widget": { "version": "1.4.0" },
                "node_modules/tool/node_modules/@family/widget": { "version": "1.4.0" },
                "node_modules/left-pad": { "version": "1.3.0" },
                "node_modules/w": { "name": "@family/widget", "version": "1.2.0" },
                "node_modules/local": { "resolved": "packages/local", "link": true },
                "packages/local": { "name": "@family/local", "version": "0.1.0" },
            })),
        );
        let path = find(dir.path()).unwrap();
        assert_eq!(path, dir.path().join("package-lock.json"));
        assert_eq!(
            pins(&path).unwrap(),
            [
                pin("@family/widget", "1.2.0"),
                pin("@family/widget", "1.4.0"),
                pin("left-pad", "1.3.0"),
            ]
        );
    }

    #[test]
    fn a_shrinkwrap_is_read_before_the_lock() {
        let dir = TempDir::new().unwrap();
        write(
            dir.path(),
            "package-lock.json",
            &lockfile(json!({ "node_modules/a": { "version": "1.0.0" } })),
        );
        write(
            dir.path(),
            "npm-shrinkwrap.json",
            &lockfile(json!({ "node_modules/b": { "version": "2.0.0" } })),
        );
        let path = find(dir.path()).unwrap();
        assert_eq!(path, dir.path().join("npm-shrinkwrap.json"));
        assert_eq!(pins(&path).unwrap(), [pin("b", "2.0.0")]);
    }

    #[test]
    fn a_checkout_with_no_lockfile_is_absent_naming_the_lockfile() {
        use timber_seams::Projection;
        let dir = TempDir::new().unwrap();
        assert_eq!(find(dir.path()), None);
        let projection = NpmProjection::new(
            dir.path(),
            Box::new(timber_fakes::FakeStore::new()),
            "http://127.0.0.1:7420/",
        );
        match projection.pinned(&dir.path().display().to_string()) {
            Err(Not::Absent(a)) => {
                assert_eq!(a.source, NpmProjection::source());
                assert_eq!(a.what, "lockfile");
            }
            other => panic!("a checkout with no lockfile answered {other:?}"),
        }
    }

    #[test]
    fn a_lockfile_npm_6_wrote_is_unreadable_naming_it_and_its_version() {
        let dir = TempDir::new().unwrap();
        write(
            dir.path(),
            "package-lock.json",
            &json!({
                "name": "consumer",
                "lockfileVersion": 1,
                "dependencies": { "left-pad": { "version": "1.3.0" } },
            }),
        );
        let path = find(dir.path()).unwrap();
        match pins(&path) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, NpmProjection::source());
                assert!(u.what.contains(&path.display().to_string()), "{u:?}");
                assert!(u.what.contains("lockfileVersion 1"), "{u:?}");
            }
            other => panic!("a lockfile of version 1 answered {other:?}"),
        }
    }

    #[test]
    fn a_lockfile_that_does_not_parse_is_unreadable_with_the_parsers_words() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("package-lock.json");
        fs::write(&path, "{ \"lockfileVersion\": 3,").unwrap();
        let words = serde_json::from_str::<Value>("{ \"lockfileVersion\": 3,")
            .unwrap_err()
            .to_string();
        match pins(&path) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.what, path.display().to_string());
                assert_eq!(u.said, Some(words));
            }
            other => panic!("a lockfile that does not parse answered {other:?}"),
        }
    }
}
