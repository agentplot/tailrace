//! The family in npm (140; design D13, D14): the scopes a checkout's own
//! `.npmrc` sends to the host's serving process, whose names npm reads
//! from a served layout and from nowhere else, and what a manifest asks
//! of them. Every other name is the public registry's and no concern of
//! the layout's (section 8). Of the `.npmrc` only a scope's registry line
//! is read; no credential in it is read or kept (rule core.21).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use serde::de::{Deserializer, MapAccess, Visitor};
use serde::Deserialize;
use serde_json::Value;
use timber_seams::{Answer, EcosystemName, Not, Range, Requirement};

use super::NpmProjection;

/// The scopes the `.npmrc` beside the checkout's root `package.json` at
/// `checkout` sends to `origin`, the host's serving process: each
/// `@<scope>:registry=<url>` line whose URL, the last such line for its
/// scope, begins with `origin`. None where the checkout holds no
/// `.npmrc`; unreadable naming it with the system's words where it
/// cannot be read.
pub(super) fn scopes(checkout: &Path, origin: &str) -> Answer<BTreeSet<String>> {
    let path = checkout.join(".npmrc");
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(BTreeSet::new()),
        Err(e) => {
            return Err(Not::unreadable(
                &NpmProjection::source(),
                path.display().to_string(),
                Some(e.to_string()),
            ))
        }
    };
    let mut registries: BTreeMap<&str, &str> = BTreeMap::new();
    for line in text.lines().map(str::trim) {
        if line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        let Some((key, url)) = line.split_once('=') else {
            continue;
        };
        let Some(scope) = key.trim().strip_suffix(":registry") else {
            continue;
        };
        if scope.len() > 1 && scope.starts_with('@') {
            registries.insert(scope, url.trim().trim_matches(['"', '\'']));
        }
    }
    Ok(registries
        .into_iter()
        .filter(|(_, url)| url.starts_with(origin))
        .map(|(scope, _)| scope.to_string())
        .collect())
}

/// Whether `name` is the family's: a scoped name whose scope is among
/// `scopes`.
pub(super) fn is_family(name: &str, scopes: &BTreeSet<String>) -> bool {
    name.split_once('/')
        .is_some_and(|(scope, _)| scopes.contains(scope))
}

/// The name and range the dependency `name` at `spec` asks of a registry,
/// as npm reads it: an alias `npm:<target>@<range>` asks its target,
/// `latest` is any version, read as `*`, since the packument's `latest`
/// names the newest version held; a spec npm does not read as a range,
/// a path, `git`, a URL or another tag, names no version a layout holds,
/// and is none (design D7).
pub(super) fn ranged(name: &str, spec: &str) -> Option<(String, String)> {
    let (name, spec) = match spec.strip_prefix("npm:") {
        Some(aliased) => match aliased.get(1..).and_then(|rest| rest.rfind('@')) {
            Some(at) => (&aliased[..at + 1], &aliased[at + 2..]),
            None => (aliased, "*"),
        },
        None => (name, spec),
    };
    let spec = spec.trim();
    if spec == "latest" {
        return Some((name.to_string(), "*".to_string()));
    }
    nodejs_semver::Range::parse(spec)
        .ok()
        .map(|_| (name.to_string(), spec.to_string()))
}

/// A dependency field of a manifest, its names in the order the manifest
/// gives them.
#[derive(Default)]
pub(super) struct Ordered(pub Vec<(String, Value)>);

impl<'de> Deserialize<'de> for Ordered {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Ordered, D::Error> {
        struct InOrder;
        impl<'de> Visitor<'de> for InOrder {
            type Value = Ordered;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("an object of dependencies")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Ordered, A::Error> {
                let mut fields = Vec::new();
                while let Some(field) = map.next_entry::<String, Value>()? {
                    fields.push(field);
                }
                Ok(Ordered(fields))
            }
        }
        deserializer.deserialize_map(InOrder)
    }
}

/// A peer dependency's entry in `peerDependenciesMeta`.
#[derive(Default, Deserialize)]
struct PeerMeta {
    #[serde(default)]
    optional: bool,
}

/// What a manifest bundles: its names, or every dependency where it is
/// `true`.
#[derive(Default, Deserialize)]
#[serde(untagged)]
enum Bundled {
    #[default]
    None,
    All(bool),
    Names(Vec<String>),
}

impl Bundled {
    fn holds(&self, name: &str) -> bool {
        match self {
            Bundled::None => false,
            Bundled::All(all) => *all,
            Bundled::Names(names) => names.iter().any(|n| n == name),
        }
    }
}

/// The folders a root manifest names its workspaces by, as globs under
/// it: an array of them, or an object's `packages`.
#[derive(Default, Deserialize)]
#[serde(untagged)]
pub(super) enum Workspaces {
    #[default]
    None,
    Globs(Vec<String>),
    Packages {
        #[serde(default)]
        packages: Vec<String>,
    },
    Other(serde::de::IgnoredAny),
}

impl Workspaces {
    /// The globs named, none where the manifest names none.
    pub(super) fn globs(&self) -> &[String] {
        match self {
            Workspaces::Globs(globs) | Workspaces::Packages { packages: globs } => globs,
            Workspaces::None | Workspaces::Other(_) => &[],
        }
    }
}

/// The parts of a `package.json` that name it and ask npm for other
/// packages.
#[derive(Default, Deserialize)]
pub(super) struct Manifest {
    #[serde(default)]
    pub(super) name: Option<String>,
    #[serde(default)]
    pub(super) workspaces: Workspaces,
    #[serde(default)]
    dependencies: Ordered,
    #[serde(default, rename = "devDependencies")]
    dev: Ordered,
    #[serde(default, rename = "optionalDependencies")]
    optional: Ordered,
    #[serde(default, rename = "peerDependencies")]
    peer: Ordered,
    #[serde(default, rename = "peerDependenciesMeta")]
    peer_meta: BTreeMap<String, PeerMeta>,
    #[serde(default, rename = "bundleDependencies")]
    bundle: Bundled,
    #[serde(default, rename = "bundledDependencies")]
    bundled: Bundled,
}

impl Manifest {
    /// The peer dependencies npm 7 and later install: every one
    /// `peerDependenciesMeta` does not mark optional.
    fn installed_peers(&self) -> impl Iterator<Item = &(String, Value)> {
        self.peer
            .0
            .iter()
            .filter(|(name, _)| !self.peer_meta.get(name).is_some_and(|meta| meta.optional))
    }

    /// 140: each family name of `scopes` this manifest, a held package's,
    /// requires, with its range as written: its `dependencies`,
    /// `optionalDependencies` and the `peerDependencies` npm installs, in
    /// that order and each in the manifest's, every name it bundles
    /// passed over, since it comes inside the tarball, and every spec
    /// that is no range (design D14). A dependency's `devDependencies`
    /// are never installed, and are not read.
    pub(super) fn requires(&self, scopes: &BTreeSet<String>) -> Vec<Requirement> {
        let asked = self
            .dependencies
            .0
            .iter()
            .chain(&self.optional.0)
            .chain(self.installed_peers())
            .filter(|(name, _)| !self.bundle.holds(name) && !self.bundled.holds(name));
        family_ranges(asked, scopes)
    }

    /// 137: each family name of `scopes` this manifest, a checkout's own
    /// or one of its workspaces', declares, with its range as written:
    /// its `dependencies`, `devDependencies`, `optionalDependencies` and
    /// the `peerDependencies` npm installs, in that order and each in the
    /// manifest's, every spec that is no range passed over (design D7).
    pub(super) fn declares(&self, scopes: &BTreeSet<String>) -> Vec<Requirement> {
        let asked = self
            .dependencies
            .0
            .iter()
            .chain(&self.dev.0)
            .chain(&self.optional.0)
            .chain(self.installed_peers());
        family_ranges(asked, scopes)
    }
}

/// Each of `asked`, a dependency's name and spec, that is a family name of
/// `scopes` at a range, by the name and range it asks of a registry, each
/// once and in the order asked.
fn family_ranges<'a>(
    asked: impl Iterator<Item = &'a (String, Value)>,
    scopes: &BTreeSet<String>,
) -> Vec<Requirement> {
    let mut ranges: Vec<Requirement> = Vec::new();
    for (name, spec) in asked {
        let Some((name, range)) = spec.as_str().and_then(|spec| ranged(name, spec)) else {
            continue;
        };
        let requirement = Requirement {
            name: EcosystemName::new(name),
            range: Range::new(range),
        };
        if is_family(requirement.name.as_str(), scopes) && !ranges.contains(&requirement) {
            ranges.push(requirement);
        }
    }
    ranges
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tempfile::TempDir;

    use super::*;

    const ORIGIN: &str = "http://127.0.0.1:7420/";

    fn npmrc(text: &str) -> (TempDir, BTreeSet<String>) {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(".npmrc"), text).unwrap();
        let scopes = scopes(dir.path(), ORIGIN).unwrap();
        (dir, scopes)
    }

    #[test]
    fn a_scope_sent_to_the_hosts_process_is_the_familys() {
        let (_, scopes) = npmrc(
            "; the family's packages\n@family:registry=http://127.0.0.1:7420/agentplot/consumer/npm/\n\
             @elsewhere:registry=https://registry.example.com/\n\
             registry=https://registry.npmjs.org/\n",
        );
        assert_eq!(scopes, BTreeSet::from(["@family".to_string()]));
        assert!(is_family("@family/widget", &scopes));
        assert!(!is_family("@elsewhere/widget", &scopes));
        assert!(!is_family("left-pad", &scopes));
    }

    #[test]
    fn a_scope_sent_elsewhere_is_no_family_and_the_last_line_for_a_scope_counts() {
        let (_, scopes) = npmrc(
            "@family:registry=https://registry.npmjs.org/\n\
             @moved:registry=http://127.0.0.1:7420/o/r/npm/\n\
             @moved:registry=https://registry.example.com/\n",
        );
        assert_eq!(scopes, BTreeSet::new());
    }

    #[test]
    fn a_line_carrying_a_token_is_read_for_its_scope_alone() {
        let (dir, scopes) = npmrc(
            "//registry.npmjs.org/:_authToken=npm_s3cr3t\n\
             @family:registry = \"http://127.0.0.1:7420/agentplot/consumer/npm/\"\n\
             //127.0.0.1:7420/:_authToken=t0k3n\n",
        );
        assert_eq!(scopes, BTreeSet::from(["@family".to_string()]));
        let read = format!("{scopes:?}");
        assert!(
            !read.contains("s3cr3t") && !read.contains("t0k3n"),
            "{read}"
        );
        fs::remove_file(dir.path().join(".npmrc")).unwrap();
        assert_eq!(super::scopes(dir.path(), ORIGIN).unwrap(), BTreeSet::new());
    }

    #[test]
    fn a_spec_is_read_as_npm_reads_a_range() {
        let read = |spec: &str| ranged("@family/w", spec);
        let range = |name: &str, range: &str| Some((name.to_string(), range.to_string()));
        assert_eq!(read("^1.2.0"), range("@family/w", "^1.2.0"));
        assert_eq!(read("latest"), range("@family/w", "*"));
        assert_eq!(
            read(">=1.2.0 <2 || 3.x"),
            range("@family/w", ">=1.2.0 <2 || 3.x")
        );
        assert_eq!(
            read("npm:@family/widget@^1.4.0"),
            range("@family/widget", "^1.4.0")
        );
        assert_eq!(read("npm:@family/widget"), range("@family/widget", "*"));
        for none in [
            "file:../local",
            "git+https://example.com/w.git",
            "https://example.com/w.tgz",
            "github:agentplot/w",
            "next",
        ] {
            assert_eq!(read(none), None, "{none}");
        }
    }

    #[test]
    fn a_manifest_requires_its_family_dependencies_and_installed_peers_in_order() {
        let scopes = BTreeSet::from(["@family".to_string()]);
        let manifest: Manifest = serde_json::from_value(json!({
            "name": "@family/widget",
            "version": "1.4.0",
            "dependencies": {
                "@family/zeta": "^1.0.0",
                "@family/base": "^2.1.0",
                "left-pad": "^1.3.0",
                "@family/inside": "^1.0.0",
                "@family/local": "file:../local",
            },
            "peerDependencies": { "@family/ui": "^3.0.0", "@family/theme": "^1.0.0" },
            "peerDependenciesMeta": { "@family/theme": { "optional": true } },
            "optionalDependencies": { "@family/fast": "npm:@family/fast-impl@~0.4.0" },
            "devDependencies": { "@family/test-kit": "^1.0.0" },
            "bundleDependencies": ["@family/inside"],
        }))
        .unwrap();
        let named: Vec<String> = manifest
            .requires(&scopes)
            .iter()
            .map(|r| format!("{} {}", r.name, r.range))
            .collect();
        assert_eq!(
            named,
            [
                "@family/zeta ^1.0.0",
                "@family/base ^2.1.0",
                "@family/fast-impl ~0.4.0",
                "@family/ui ^3.0.0"
            ]
        );
    }
}
