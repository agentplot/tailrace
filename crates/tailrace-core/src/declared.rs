//! The components the authors declare and the version rule, read from
//! the policy file at a landed revision, never from a checkout (65, 123,
//! C.10, rule core.22; design D1, D4).

use std::collections::BTreeMap;

use semver::{Prerelease, Version};
use serde::Deserialize;
use tailrace_seams::{Answer, Component, ComponentName, Ecosystem, Not, OutputName, Source, Why};

use crate::{History, Landed, LOCAL, NOTES};

/// The policy file, at the root of the repository's tree.
pub const POLICY_FILE: &str = ".config/tailrace.toml";

/// What the one component of a repository that declares none is named.
pub const REPOSITORY: &str = "repository";

/// The version rule: git-cliff's conventional bump with these settings,
/// and a prerelease where no commit makes a release (21, section 9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    /// The prefix of the one component's tags, when none is declared.
    pub prefix: String,
    /// The version of a component no release tag carries.
    pub initial: Version,
    /// The identifier of a prerelease, before its number.
    pub prerelease: String,
    pub features_always_bump_minor: bool,
    pub breaking_always_bump_major: bool,
}

impl Default for Rule {
    fn default() -> Self {
        Rule {
            prefix: "v".into(),
            initial: Version::new(0, 1, 0),
            prerelease: "pre".into(),
            features_always_bump_minor: true,
            breaking_always_bump_major: true,
        }
    }
}

/// What the policy file at a landed revision declares of versions.
///
/// The file is `.config/tailrace.toml`. This reads three things of it and
/// leaves every other table and key to the reader that owns it:
///
/// - `outputs`, a list of output names: the one component's outputs when
///   no component is declared; none when absent.
/// - `[version]`, the rule: `prefix` (default `"v"`), the one component's
///   tag prefix; `initial` (default `"0.1.0"`), a semantic version;
///   `prerelease` (default `"pre"`), a prerelease identifier;
///   `features_always_bump_minor` and `breaking_always_bump_major`
///   (default `true` each), git-cliff's bump settings.
/// - `[projections]`, a table from output name to ecosystem, written
///   `site = "file"` or `site = { ecosystem = "file" }` with the table's
///   other keys left to their readers: the ecosystem of the projection
///   that packs each of the one component's outputs, when no component
///   is declared (C.10); none when absent.
/// - `[[component]]`, each with `name`, `outputs`, `paths` (glob
///   patterns, at least one) and `prefix`, every key required, and
///   `projections` as the top-level table is for the one component. No
///   two share a name or a prefix.
///
/// With no file, or no `[[component]]`, the repository is one component
/// named `repository` over the paths `["**"]`, with the rule's prefix,
/// the top-level `outputs` and the top-level `projections`. A
/// projections key naming an output its component does not declare is
/// unreadable naming it. A projections key naming the ecosystem `notes`,
/// which a version's notes are carried in, is refused naming the
/// component and the output (design D9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declared {
    pub rule: Rule,
    pub components: Vec<Component>,
    /// C.10: per component, the ecosystem each of its outputs is packed
    /// in.
    pub projections: BTreeMap<ComponentName, BTreeMap<OutputName, Ecosystem>>,
}

#[derive(Deserialize)]
struct File {
    #[serde(default)]
    outputs: Vec<OutputName>,
    #[serde(default)]
    version: RawRule,
    #[serde(default)]
    projections: BTreeMap<OutputName, RawProjection>,
    #[serde(default)]
    component: Vec<RawComponent>,
}

/// An output's projection: its ecosystem, or a table naming it.
#[derive(Deserialize)]
#[serde(untagged)]
enum RawProjection {
    Ecosystem(Ecosystem),
    Table { ecosystem: Ecosystem },
}

fn ecosystems(raw: BTreeMap<OutputName, RawProjection>) -> BTreeMap<OutputName, Ecosystem> {
    raw.into_iter()
        .map(|(output, p)| match p {
            RawProjection::Ecosystem(e) | RawProjection::Table { ecosystem: e } => (output, e),
        })
        .collect()
}

#[derive(Default, Deserialize)]
struct RawRule {
    prefix: Option<String>,
    initial: Option<String>,
    prerelease: Option<String>,
    features_always_bump_minor: Option<bool>,
    breaking_always_bump_major: Option<bool>,
}

#[derive(Deserialize)]
struct RawComponent {
    name: ComponentName,
    outputs: Vec<OutputName>,
    paths: Vec<String>,
    prefix: String,
    #[serde(default)]
    projections: BTreeMap<OutputName, RawProjection>,
}

impl Declared {
    /// 123: the components and the rule the policy file declares at the
    /// landed revision; unreadable naming the file and the revision, with
    /// the parser's words, when it is not well formed (58, 59); refused
    /// [`Why::DeclaredTwice`] when two components share a name or a
    /// prefix, and [`Why::Said`] when an output is packed as `notes`.
    pub fn read(history: &History, landed: &Landed) -> Answer<Declared> {
        let what = format!("{POLICY_FILE} at {}", landed.revision());
        let unreadable = |said: String| Not::unreadable(&Source::history(LOCAL), &what, Some(said));
        let file = match history.blob(landed, POLICY_FILE) {
            Ok(bytes) => {
                let text = String::from_utf8(bytes).map_err(|e| unreadable(e.to_string()))?;
                toml::from_str::<File>(&text).map_err(|e| unreadable(e.to_string()))?
            }
            Err(Not::Absent(_)) => File {
                outputs: vec![],
                version: RawRule::default(),
                projections: BTreeMap::new(),
                component: vec![],
            },
            Err(not) => return Err(not),
        };

        let default = Rule::default();
        let initial = match file.version.initial {
            Some(text) => Version::parse(&text).map_err(|e| unreadable(format!("initial: {e}")))?,
            None => default.initial,
        };
        let prerelease = file.version.prerelease.unwrap_or(default.prerelease);
        Prerelease::new(&format!("{prerelease}.1"))
            .map_err(|e| unreadable(format!("prerelease: {e}")))?;
        let rule = Rule {
            prefix: file.version.prefix.unwrap_or(default.prefix),
            initial,
            prerelease,
            features_always_bump_minor: file
                .version
                .features_always_bump_minor
                .unwrap_or(default.features_always_bump_minor),
            breaking_always_bump_major: file
                .version
                .breaking_always_bump_major
                .unwrap_or(default.breaking_always_bump_major),
        };

        let undeclared = |name: &ComponentName,
                          outputs: &[OutputName],
                          projections: &BTreeMap<OutputName, Ecosystem>| {
            match projections.keys().find(|o| !outputs.contains(o)) {
                Some(output) => Err(unreadable(format!(
                    "component {name}: projections names {output}, which it does not declare as an output"
                ))),
                None => Ok(()),
            }
        };
        let reserved = |name: &ComponentName, projections: &BTreeMap<OutputName, Ecosystem>| {
            match projections.iter().find(|(_, e)| e.as_str() == NOTES) {
                Some((output, _)) => Err(Not::refused(
                    &Source::history(LOCAL),
                    Why::Said,
                    Some(format!(
                        "component {name}: projections packs {output} as {NOTES}, the ecosystem of a version's notes, which no output is packed in"
                    )),
                )),
                None => Ok(()),
            }
        };

        if file.component.is_empty() {
            let name = ComponentName::new(REPOSITORY);
            let packed_by = ecosystems(file.projections);
            undeclared(&name, &file.outputs, &packed_by)?;
            reserved(&name, &packed_by)?;
            return Ok(Declared {
                components: vec![Component {
                    name: name.clone(),
                    outputs: file.outputs,
                    paths: vec!["**".into()],
                    prefix: rule.prefix.clone(),
                }],
                projections: BTreeMap::from([(name, packed_by)]),
                rule,
            });
        }

        let mut components: Vec<Component> = Vec::new();
        let mut projections = BTreeMap::new();
        for raw in file.component {
            let packed_by = ecosystems(raw.projections);
            undeclared(&raw.name, &raw.outputs, &packed_by)?;
            reserved(&raw.name, &packed_by)?;
            if raw.paths.is_empty() {
                return Err(unreadable(format!(
                    "component {} declares no paths",
                    raw.name
                )));
            }
            for path in &raw.paths {
                glob::Pattern::new(path)
                    .map_err(|e| unreadable(format!("component {}: {path}: {e}", raw.name)))?;
            }
            for other in &components {
                let what = if other.name == raw.name {
                    format!("name \"{}\"", raw.name)
                } else if other.prefix == raw.prefix {
                    format!("prefix \"{}\"", raw.prefix)
                } else {
                    continue;
                };
                return Err(Not::refused(
                    &Source::history(LOCAL),
                    Why::DeclaredTwice {
                        first: other.name.clone(),
                        second: raw.name,
                        what,
                    },
                    None,
                ));
            }
            projections.insert(raw.name.clone(), packed_by);
            components.push(Component {
                name: raw.name,
                outputs: raw.outputs,
                paths: raw.paths,
                prefix: raw.prefix,
            });
        }
        Ok(Declared {
            rule,
            components,
            projections,
        })
    }

    /// C.10: the ecosystem of the projection that packs `output` of
    /// `component`; `None` when the policy file names none.
    pub fn projection_of(
        &self,
        component: &ComponentName,
        output: &OutputName,
    ) -> Option<&Ecosystem> {
        self.projections.get(component)?.get(output)
    }
}
