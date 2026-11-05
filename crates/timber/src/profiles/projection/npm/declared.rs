//! npm's ranges (137; design D7): each family package a checkout's root
//! `package.json`, and each of its workspaces', declares, by its name and
//! its range as the manifest spells it, so a checkout with a manifest and
//! no lockfile answers its ranges and a first install needs no lockfile
//! written by hand. It reads the files alone: no npm runs, since npm may
//! write the lockfile, and nothing is written.

use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use timber_seams::{Answer, Not, Requirement};

use super::family::Manifest;
use super::NpmProjection;

/// What npm calls a package's manifest.
const MANIFEST: &str = "package.json";

/// 137: each family name of `scopes` the root manifest of the checkout at
/// `checkout`, or a workspace's its `workspaces` names, declares, with
/// its range as written, each name and range once in the manifests'
/// order, the root's first; a name a workspace is passed over, since npm
/// links it from the checkout. None where the checkout holds no
/// `package.json`; a manifest that cannot be read or does not parse is
/// unreadable naming it with the system's or the parser's words (58).
pub(super) fn declared(checkout: &Path, scopes: &BTreeSet<String>) -> Answer<Vec<Requirement>> {
    let Some(root) = read(&checkout.join(MANIFEST))? else {
        return Ok(Vec::new());
    };
    let mut members = Vec::new();
    for dir in workspaces(checkout, root.workspaces.globs()) {
        if let Some(member) = read(&dir.join(MANIFEST))? {
            members.push(member);
        }
    }
    let own: BTreeSet<&str> = members.iter().filter_map(|m| m.name.as_deref()).collect();
    let mut ranges: Vec<Requirement> = Vec::new();
    for manifest in std::iter::once(&root).chain(&members) {
        for range in manifest.declares(scopes) {
            if !own.contains(range.name.as_str()) && !ranges.contains(&range) {
                ranges.push(range);
            }
        }
    }
    Ok(ranges)
}

/// The manifest at `path`, none where there is no file there; unreadable
/// naming it with the system's words where it cannot be read, and with
/// the parser's where it is no manifest.
fn read(path: &Path) -> Answer<Option<Manifest>> {
    let unreadable = |said: String| {
        Not::unreadable(
            &NpmProjection::source(),
            path.display().to_string(),
            Some(said),
        )
    };
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(unreadable(e.to_string())),
    };
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|e| unreadable(e.to_string()))
}

/// The folders under `checkout` that `globs` match, as npm reads a root
/// manifest's `workspaces`: each glob's folders, less those a glob led by
/// `!` matches, in order of path, the checkout itself never among them.
fn workspaces(checkout: &Path, globs: &[String]) -> Vec<PathBuf> {
    let matched = |glob: &str| -> Vec<PathBuf> {
        let at = checkout.join(glob.trim_end_matches('/'));
        glob::glob(&at.to_string_lossy())
            .map(|paths| {
                paths
                    .filter_map(Result::ok)
                    .filter(|p| p.is_dir())
                    .collect()
            })
            .unwrap_or_default()
    };
    let excluded: BTreeSet<PathBuf> = globs
        .iter()
        .filter_map(|g| g.strip_prefix('!'))
        .flat_map(matched)
        .collect();
    let dirs: BTreeSet<PathBuf> = globs
        .iter()
        .filter(|g| !g.starts_with('!'))
        .flat_map(|g| matched(g))
        .filter(|dir| dir != checkout && !excluded.contains(dir))
        .collect();
    dirs.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};
    use tempfile::TempDir;
    use timber_fakes::FakeStore;
    use timber_seams::{EcosystemName, Pin, Pinned, Projection, Range, Version};

    use super::*;

    const ORIGIN: &str = "http://127.0.0.1:7420/";

    const FAMILY: &str = "@family:registry=http://127.0.0.1:7420/agentplot/consumer/npm/\n";

    fn range(name: &str, range: &str) -> Pin {
        Pin::Range(Requirement {
            name: EcosystemName::new(name),
            range: Range::new(range),
        })
    }

    fn pin(name: &str, version: &str) -> Pin {
        Pin::Version(Pinned {
            name: EcosystemName::new(name),
            version: Version::new(version),
        })
    }

    /// A checkout holding `files`, each JSON value written as its text and
    /// each string as itself, and what the projection of it answers for
    /// its pins.
    fn pinned(files: &[(&str, Value)]) -> (TempDir, Answer<Vec<Pin>>) {
        let dir = TempDir::new().unwrap();
        for (path, content) in files {
            let at = dir.path().join(path);
            fs::create_dir_all(at.parent().unwrap()).unwrap();
            let text = match content {
                Value::String(text) => text.clone(),
                json => json.to_string(),
            };
            fs::write(at, text).unwrap();
        }
        let projection = NpmProjection::new(dir.path(), Box::new(FakeStore::new()), ORIGIN);
        let answer = projection.pinned(&dir.path().display().to_string());
        (dir, answer)
    }

    fn lockfile(packages: Value) -> Value {
        json!({ "name": "consumer", "lockfileVersion": 3, "packages": packages })
    }

    // Requirement: npm answers what a lockfile pins and what a manifest
    // asks for

    #[test]
    fn a_first_installs_ranges_are_the_familys_as_the_manifest_spells_them() {
        let (_, answer) = pinned(&[
            (".npmrc", json!(FAMILY)),
            (
                "package.json",
                json!({
                    "name": "consumer",
                    "dependencies": {
                        "@family/widget": "^1.2.0",
                        "@family/local": "file:../local",
                        "react": "^18.0.0",
                    },
                    "devDependencies": { "@family/tool": "latest" },
                }),
            ),
        ]);
        assert_eq!(
            answer.unwrap(),
            [
                range("@family/widget", "^1.2.0"),
                range("@family/tool", "*")
            ]
        );
    }

    #[test]
    fn a_name_the_lockfile_already_pins_is_answered_as_its_pin_and_no_range() {
        let (_, answer) = pinned(&[
            (".npmrc", json!(FAMILY)),
            (
                "package.json",
                json!({
                    "dependencies": {
                        "@family/widget": "^1.2.0",
                        "@family/base": "^2.0.0",
                        "w": "npm:@family/aliased@^3.0.0",
                    },
                }),
            ),
            (
                "package-lock.json",
                lockfile(json!({
                    "": { "name": "consumer" },
                    "node_modules/@family/widget": { "version": "1.4.0" },
                    "node_modules/w": { "name": "@family/aliased", "version": "3.1.0" },
                })),
            ),
        ]);
        assert_eq!(
            answer.unwrap(),
            [
                pin("@family/aliased", "3.1.0"),
                pin("@family/widget", "1.4.0"),
                range("@family/base", "^2.0.0"),
            ]
        );
    }

    #[test]
    fn no_family_without_the_npmrc_line_is_absent_naming_the_lockfile() {
        let manifest = json!({ "dependencies": { "@family/widget": "^1.2.0" } });
        for npmrc in [None, Some("@family:registry=https://registry.npmjs.org/\n")] {
            let mut files = vec![("package.json", manifest.clone())];
            if let Some(npmrc) = npmrc {
                files.push((".npmrc", json!(npmrc)));
            }
            match pinned(&files) {
                (_, Err(Not::Absent(a))) => {
                    assert_eq!(a.source, NpmProjection::source());
                    assert_eq!(a.what, "lockfile");
                }
                (_, other) => panic!("{npmrc:?} answered {other:?}"),
            }
        }
    }

    #[test]
    fn each_workspace_declares_its_ranges_and_a_workspaces_own_name_is_given_none() {
        let (_, answer) = pinned(&[
            (".npmrc", json!(FAMILY)),
            (
                "package.json",
                json!({
                    "name": "consumer",
                    "workspaces": ["packages/*", "!packages/skipped"],
                    "dependencies": { "@family/app": "*", "@family/base": "^2.0.0" },
                }),
            ),
            (
                "packages/app/package.json",
                json!({
                    "name": "@family/app",
                    "dependencies": {
                        "@family/base": "^2.1.0",
                        "@family/lib": "workspace:*",
                    },
                    "optionalDependencies": { "@family/fast": "~0.4.0" },
                    "peerDependencies": { "@family/ui": "^3.0.0", "@family/theme": "^1.0.0" },
                    "peerDependenciesMeta": { "@family/theme": { "optional": true } },
                }),
            ),
            (
                "packages/lib/package.json",
                json!({ "name": "@family/lib", "devDependencies": { "@family/base": "^2.0.0" } }),
            ),
            (
                "packages/skipped/package.json",
                json!({ "name": "@family/skipped", "dependencies": { "@family/gone": "1" } }),
            ),
        ]);
        assert_eq!(
            answer.unwrap(),
            [
                range("@family/base", "^2.0.0"),
                range("@family/base", "^2.1.0"),
                range("@family/fast", "~0.4.0"),
                range("@family/ui", "^3.0.0"),
            ]
        );
    }

    #[test]
    fn workspaces_named_by_an_objects_packages_are_read() {
        let (_, answer) = pinned(&[
            (".npmrc", json!(FAMILY)),
            (
                "package.json",
                json!({ "workspaces": { "packages": ["tools/cli"] } }),
            ),
            (
                "tools/cli/package.json",
                json!({ "name": "cli", "dependencies": { "@family/base": "2.x" } }),
            ),
        ]);
        assert_eq!(answer.unwrap(), [range("@family/base", "2.x")]);
    }

    #[test]
    fn a_package_json_that_does_not_parse_is_unreadable_with_the_parsers_words() {
        let text = "{ \"dependencies\": { \"@family/widget\": ";
        let words = serde_json::from_str::<Value>(text).unwrap_err().to_string();
        let (dir, answer) = pinned(&[
            (".npmrc", json!(FAMILY)),
            ("package.json", json!({ "workspaces": ["packages/*"] })),
            ("packages/a/package.json", json!(text)),
        ]);
        match answer {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, NpmProjection::source());
                assert_eq!(
                    u.what,
                    dir.path()
                        .join("packages/a/package.json")
                        .display()
                        .to_string()
                );
                assert_eq!(u.said, Some(words));
            }
            other => panic!("a package.json that does not parse answered {other:?}"),
        }
    }
}
