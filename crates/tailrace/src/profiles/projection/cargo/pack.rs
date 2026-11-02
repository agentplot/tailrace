//! pack (84, 87, 110, 123; design D1 to D4): the package the output's one
//! crate file names, at the place in the repository it was packed from,
//! checked with `cargo metadata`; the component's version written into
//! the package and every sibling it reaches by path, each dependency on a
//! sibling given that version in the registry `tailrace`, and each on a
//! crate of another component given the registry and checked against the
//! store's listing; the package packed alone by `cargo package` into a
//! temporary target directory with no lockfile of its own; every edited
//! file's bytes written back whatever cargo answered; and the crate
//! refused where its own manifest names another version, since every pack
//! of a revision edits one checkout.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tailrace_seams::{Answer, Ecosystem, EcosystemName, Not, Output, PackAt, Version, Why};
use toml_edit::{DocumentMut, Item, TableLike};

use super::{crate_file, CargoProjection, ECOSYSTEM};
use crate::profiles::run::GIT_REPOSITORY_ENV;

/// What the manifest of a package is called.
const MANIFEST: &str = "Cargo.toml";

/// The registry a family crate's dependencies are packed in, as the
/// checkout's cargo configuration names it (C.11).
pub(super) const REGISTRY: &str = "tailrace";

/// The dependency tables a crate's build reads, beside each target's own
/// of them; dev dependencies are the package's own and left as written.
const TABLES: [&str; 2] = ["dependencies", "build-dependencies"];

/// The package an output's crate file names, and the manifest it was
/// packed from.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Found {
    pub name: String,
    pub manifest: PathBuf,
}

/// What packing a package at a version edits, and what the store must
/// hold before cargo runs (design D3).
#[derive(Debug, Default)]
pub(super) struct Plan {
    /// Each file edited, by its canonical path: its own bytes and its
    /// text as edited.
    files: BTreeMap<PathBuf, Edited>,
    /// Each crate of another component depended on, with the requirement
    /// its dependency writes, once each.
    others: Vec<Other>,
}

#[derive(Debug)]
struct Edited {
    original: Vec<u8>,
    doc: DocumentMut,
}

/// A crate of another component a packed crate depends on, and the
/// requirement written for it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Other {
    pub name: String,
    pub requirement: String,
}

/// A dependency written with a path: the file and the table it is
/// written in, the directory it names, and its `version` and `registry`.
struct PathDependency {
    file: PathBuf,
    keys: Vec<String>,
    dir: PathBuf,
    version: Option<String>,
    registry: Option<String>,
}

impl CargoProjection {
    /// The package the one crate file among `output`'s files names, and
    /// its manifest at `<checkout>/<path_in_vcs>/Cargo.toml` (design D1);
    /// refused naming the output and its files where it left none or
    /// several, and naming the crate where it names no place in the
    /// repository; unreadable naming a crate file that cannot be read, with
    /// the reader's words.
    pub(super) fn crate_of(&self, output: &Output) -> Answer<Found> {
        let crates = self.crate_files(output);
        let [file] = crates.as_slice() else {
            let left = match crates.as_slice() {
                [] => "no crate file".to_string(),
                several => format!(
                    "{} crate files, {},",
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
            return Err(Not::refused(
                &CargoProjection::source(),
                Why::Said,
                Some(format!(
                    "the output {} left {left} among its files, {files}, and the cargo \
                     projection packs one crate",
                    output.name
                )),
            ));
        };
        let bytes = fs::read(file).map_err(|e| unreadable(file, e.to_string()))?;
        let (manifest, path) =
            crate_file::packed_from(&bytes).map_err(|said| unreadable(file, said))?;
        let Some(path) = path else {
            return Err(Not::refused(
                &CargoProjection::source(),
                Why::Said,
                Some(format!(
                    "the crate {} names no place in the repository it was packed from, which \
                     cargo package writes as path_in_vcs in a git checkout",
                    file.display()
                )),
            ));
        };
        Ok(Found {
            name: manifest.name,
            manifest: self.root.join(path).join(MANIFEST),
        })
    }

    /// The files among `output`'s that are crate files.
    fn crate_files(&self, output: &Output) -> Vec<PathBuf> {
        output
            .files
            .iter()
            .map(|file| self.under_root(file))
            .filter(|path| path.extension().is_some_and(|e| e == "crate"))
            .collect()
    }

    /// The names of the packages the component's outputs' crate files
    /// name (design D3): an output with no one crate file that reads is no
    /// crate of the component, and its own pack answers why.
    fn published(&self, outputs: &[Output]) -> BTreeSet<String> {
        outputs
            .iter()
            .filter_map(|output| match self.crate_files(output).as_slice() {
                [one] => fs::read(one).ok(),
                _ => None,
            })
            .filter_map(|bytes| crate_file::manifest(&bytes).ok())
            .map(|manifest| manifest.name)
            .collect()
    }

    /// The crate of the package `found` names, packed at `at` (design D3):
    /// its workspace read with `cargo metadata`, the plan of edits made
    /// and checked against the store before cargo packs, every edited file
    /// written, `cargo package` run, and every file's own bytes written
    /// back before this answers, whatever cargo answered.
    pub(super) fn packed(&self, found: &Found, at: &PackAt<'_>) -> Answer<Vec<u8>> {
        let workspace = self.metadata(found)?;
        let plan = self.plan(&found.manifest, &workspace, at)?;
        self.within_store(&plan.others)?;
        let crate_file = match plan.write() {
            Ok(()) => self.package(found, &workspace, at.version),
            Err(not) => Err(not),
        };
        plan.restore()?;
        let bytes = crate_file?;
        names_package(&bytes, &found.manifest, &found.name, at.version)?;
        Ok(bytes)
    }

    /// The root of the workspace of the package `found` names, as `cargo
    /// metadata` answers it; refused where cargo names no package of that
    /// name at its manifest (design D1).
    fn metadata(&self, found: &Found) -> Answer<PathBuf> {
        let manifest = &found.manifest;
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
        let named = metadata
            .packages
            .into_iter()
            .find(|p| canonical(&p.manifest_path) == wanted)
            .map(|p| p.name);
        if named.as_deref() == Some(found.name.as_str()) {
            return Ok(metadata.workspace_root);
        }
        Err(Not::refused(
            &CargoProjection::source(),
            Why::Said,
            Some(format!(
                "the crate file names the package {}, and cargo metadata names {} at {}",
                found.name,
                named.map_or("no package".into(), |n| format!("the package {n}")),
                manifest.display()
            )),
        ))
    }

    /// What packing the package at `manifest` at `at` edits (design D3):
    /// the version on it and on every sibling it reaches by a path
    /// dependency, its own or inherited from the workspace at `workspace`,
    /// a sibling being a crate whose manifest the component's paths hold;
    /// each dependency on a sibling given the version in the registry
    /// `tailrace`, and each on a crate of another component given the
    /// registry where it names none. Refused naming a sibling none of the
    /// component's outputs packs, and the component; and naming a crate of
    /// another component its dependency writes no version of.
    pub(super) fn plan(&self, manifest: &Path, workspace: &Path, at: &PackAt<'_>) -> Answer<Plan> {
        let published = self.published(at.outputs);
        let version = at.version.as_str();
        let mut plan = Plan::default();
        let mut seen = BTreeSet::new();
        let mut reached = vec![manifest.to_path_buf()];
        while let Some(manifest) = reached.pop() {
            if !seen.insert(canonical(&manifest)) {
                continue;
            }
            let (holder, held) = holder(&manifest, workspace)?;
            plan.edit(&holder, &[], |doc_path, doc| {
                set_version(doc, doc_path, held, at.version)
            })?;
            for dependency in path_dependencies(&manifest, workspace)? {
                let theirs = canonical(&dependency.dir.join(MANIFEST));
                let name = package_name(&theirs)?;
                if (at.holds)(&self.relative(&theirs)) {
                    if !published.contains(&name) {
                        return Err(Not::refused(
                            &CargoProjection::source(),
                            Why::Said,
                            Some(format!(
                                "the crate {name}, at {}, is of the component {} by its paths, \
                                 and none of the component's outputs packs it, so no consumer \
                                 could resolve it",
                                theirs.display(),
                                at.component.name
                            )),
                        ));
                    }
                    plan.edit(&dependency.file, &dependency.keys, |_, table| {
                        set(table, "version", version);
                        set(table, "registry", REGISTRY);
                        Ok(())
                    })?;
                    reached.push(theirs);
                } else {
                    let Some(requirement) = dependency.version else {
                        return Err(Not::refused(
                            &CargoProjection::source(),
                            Why::Said,
                            Some(format!(
                                "the dependency on {name}, of another component than {}, in {} \
                                 writes no version, and a crate of another component is packed \
                                 at the requirement its manifest writes",
                                at.component.name,
                                dependency.file.display()
                            )),
                        ));
                    };
                    if dependency.registry.is_none() {
                        plan.edit(&dependency.file, &dependency.keys, |_, table| {
                            set(table, "registry", REGISTRY);
                            Ok(())
                        })?;
                    }
                    let other = Other { name, requirement };
                    if !plan.others.contains(&other) {
                        plan.others.push(other);
                    }
                }
            }
        }
        Ok(plan)
    }

    /// Refused where the store holds no version of a crate of another
    /// component within the requirement written for it, naming the crate,
    /// the requirement and the versions it holds, or that it holds none;
    /// a listing the store answers unreadable or refused is this answer as
    /// the store gave it (design D3; C.11, 138, rule core.19).
    pub(super) fn within_store(&self, others: &[Other]) -> Answer<()> {
        let ecosystem = Ecosystem::new(ECOSYSTEM);
        for other in others {
            let refused = |said: String| {
                Not::refused(
                    &CargoProjection::source(),
                    Why::Said,
                    Some(format!(
                        "{} of another component is required at {}, and {said}",
                        other.name, other.requirement
                    )),
                )
            };
            let requirement = semver::VersionReq::parse(&other.requirement)
                .map_err(|e| refused(format!("Cargo cannot read the requirement: {e}")))?;
            let listed = self
                .store
                .versions(&ecosystem, &EcosystemName::new(&other.name))?;
            if listed.is_empty() {
                return Err(refused("the store holds no version of it".into()));
            }
            let within = listed
                .iter()
                .any(|v| semver::Version::parse(v.as_str()).is_ok_and(|v| requirement.matches(&v)));
            if !within {
                let held: Vec<&str> = listed.iter().map(Version::as_str).collect();
                return Err(refused(format!(
                    "the store holds it only at {}",
                    held.join(", ")
                )));
            }
        }
        Ok(())
    }

    /// The bytes of the crate `cargo package` makes of the package `found`
    /// names alone, offline and with no lockfile of its own, in the
    /// registry `tailrace`, into a temporary target directory removed
    /// after, so nothing is left in the checkout's own (design D3).
    fn package(&self, found: &Found, workspace: &Path, version: &Version) -> Answer<Vec<u8>> {
        let target = tempfile::TempDir::new()
            .map_err(|e| unreadable(&std::env::temp_dir(), e.to_string()))?;
        let root = workspace.join(MANIFEST);
        let args = [
            OsStr::new("package"),
            OsStr::new("--no-verify"),
            OsStr::new("--allow-dirty"),
            OsStr::new("--offline"),
            OsStr::new("--exclude-lockfile"),
            OsStr::new("--registry"),
            OsStr::new(REGISTRY),
            OsStr::new("--manifest-path"),
            root.as_os_str(),
            OsStr::new("-p"),
            OsStr::new(&found.name),
            OsStr::new("--target-dir"),
            target.path().as_os_str(),
        ];
        self.cargo
            .output(found.manifest.parent(), &args, GIT_REPOSITORY_ENV, &[0])?;
        let packed = target
            .path()
            .join("package")
            .join(format!("{}-{version}.crate", found.name));
        fs::read(&packed).map_err(|e| unreadable(&packed, e.to_string()))
    }

    /// `path` relative to the checkout, as the component's paths are
    /// matched; itself where it is not under it.
    fn relative(&self, path: &Path) -> String {
        let path = canonical(path);
        match path.strip_prefix(canonical(&self.root)) {
            Ok(relative) => relative.to_string_lossy().into_owned(),
            Err(_) => path.display().to_string(),
        }
    }
}

impl Plan {
    /// `change` made to the table `keys` name in the file at `path`, the
    /// file read the first time it is edited.
    fn edit(
        &mut self,
        path: &Path,
        keys: &[String],
        change: impl FnOnce(&Path, &mut dyn TableLike) -> Answer<()>,
    ) -> Answer<()> {
        let at = canonical(path);
        if !self.files.contains_key(&at) {
            let original = fs::read(&at).map_err(|e| unreadable(&at, e.to_string()))?;
            let doc = std::str::from_utf8(&original)
                .map_err(|e| e.to_string())
                .and_then(|text| {
                    text.parse::<DocumentMut>()
                        .map_err(|e: toml_edit::TomlError| e.to_string())
                })
                .map_err(|said| unreadable(&at, said))?;
            self.files.insert(at.clone(), Edited { original, doc });
        }
        let edited = self.files.get_mut(&at).expect("the file was just read");
        let mut table = edited.doc.as_item_mut();
        for key in keys {
            table = table
                .get_mut(key)
                .ok_or_else(|| names_none(&at, &keys.join(".")))?;
        }
        let table = table
            .as_table_like_mut()
            .ok_or_else(|| names_none(&at, &keys.join(".")))?;
        change(&at, table)
    }

    /// The text of each file as edited, by its path.
    #[cfg(test)]
    fn texts(&self) -> BTreeMap<PathBuf, String> {
        self.files
            .iter()
            .map(|(path, edited)| (path.clone(), edited.doc.to_string()))
            .collect()
    }

    /// Every edited file written as edited.
    fn write(&self) -> Answer<()> {
        for (path, edited) in &self.files {
            fs::write(path, edited.doc.to_string()).map_err(|e| unreadable(path, e.to_string()))?;
        }
        Ok(())
    }

    /// Every edited file's own bytes written back, each tried whatever
    /// another answered; the first that could not be written is the
    /// answer.
    fn restore(&self) -> Answer<()> {
        let mut first = Ok(());
        for (path, edited) in &self.files {
            if let Err(e) = fs::write(path, &edited.original) {
                if first.is_ok() {
                    first = Err(unreadable(path, e.to_string()));
                }
            }
        }
        first
    }
}

/// The dependencies with a path the manifest at `manifest` writes in its
/// build's tables, its own or inherited from the workspace at
/// `workspace`'s `workspace.dependencies` (design D3).
fn path_dependencies(manifest: &Path, workspace: &Path) -> Answer<Vec<PathDependency>> {
    let doc = read_doc(manifest)?;
    let root = workspace.join(MANIFEST);
    let root_doc = read_doc(&root)?;
    let inherited = root_doc
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .and_then(Item::as_table_like);
    let dir = manifest.parent().unwrap_or(Path::new("."));
    let mut tables: Vec<Vec<String>> = TABLES.iter().map(|t| vec![t.to_string()]).collect();
    if let Some(targets) = doc.get("target").and_then(Item::as_table_like) {
        for (cfg, _) in targets.iter() {
            for table in TABLES {
                tables.push(vec!["target".into(), cfg.into(), table.into()]);
            }
        }
    }
    let mut found = Vec::new();
    for keys in tables {
        let Some(table) = table_at(&doc, &keys) else {
            continue;
        };
        for (key, entry) in table.iter() {
            let Some(entry) = entry.as_table_like() else {
                continue;
            };
            let (file, keys, base, entry) =
                if entry.get("workspace").and_then(Item::as_bool) == Some(true) {
                    let Some(entry) = inherited
                        .and_then(|deps| deps.get(key))
                        .and_then(Item::as_table_like)
                    else {
                        continue;
                    };
                    let keys = vec!["workspace".into(), "dependencies".into(), key.into()];
                    (root.clone(), keys, workspace, entry)
                } else {
                    let mut keys = keys.clone();
                    keys.push(key.into());
                    (manifest.to_path_buf(), keys, dir, entry)
                };
            let Some(path) = entry.get("path").and_then(Item::as_str) else {
                continue;
            };
            let text = |key: &str| entry.get(key).and_then(Item::as_str).map(str::to_string);
            found.push(PathDependency {
                file,
                keys,
                dir: base.join(path),
                version: text("version"),
                registry: text("registry"),
            });
        }
    }
    Ok(found)
}

/// The table `keys` name in `doc`, none where one of them names nothing
/// or no table.
fn table_at<'a>(doc: &'a DocumentMut, keys: &[String]) -> Option<&'a dyn TableLike> {
    let mut item = doc.as_item();
    for key in keys {
        item = item.get(key)?;
    }
    item.as_table_like()
}

/// The name the package of the manifest at `path` declares; unreadable
/// naming the manifest where it declares none.
fn package_name(path: &Path) -> Answer<String> {
    read_doc(path)?
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(Item::as_str)
        .map(str::to_string)
        .ok_or_else(|| names_none(path, "package.name"))
}

fn read_doc(path: &Path) -> Answer<DocumentMut> {
    let text = fs::read_to_string(path).map_err(|e| unreadable(path, e.to_string()))?;
    text.parse()
        .map_err(|e: toml_edit::TomlError| unreadable(path, e.to_string()))
}

/// `key` set to `value` in `table`, the value's own comments and spacing
/// kept where it was written.
fn set(table: &mut dyn TableLike, key: &str, value: &str) {
    let mut value = toml_edit::Value::from(value);
    match table.get_mut(key) {
        Some(item) => {
            if let Some(old) = item.as_value() {
                *value.decor_mut() = old.decor().clone();
            }
            *item = Item::Value(value);
        }
        None => {
            // An inline table's last value carries the space before its
            // closing brace, which the value added after it takes over.
            let last = table.iter_mut().last().and_then(|(_, item)| {
                let last = item.as_value_mut()?;
                let suffix = last.decor().suffix().cloned();
                last.decor_mut().set_suffix("");
                suffix
            });
            if let Some(suffix) = last {
                value.decor_mut().set_suffix(suffix);
            }
            table.insert(key, Item::Value(value));
        }
    }
}

/// Refused naming both where the crate `bytes`, packed of the package at
/// `manifest`, names another package or version than `name` and
/// `version` (design D3), as when two packs edit one checkout at once;
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

/// A member of the workspace, as `cargo metadata` lists it.
#[derive(Deserialize)]
struct Listed {
    name: String,
    manifest_path: PathBuf,
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
    let doc = read_doc(manifest)?;
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
            Ok((workspace_root.join(MANIFEST), Held::Workspace))
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

/// The value `held` names set to `version` in `doc`, the manifest at
/// `path`, and the rest of the file as it was: the value's own comments
/// and spacing kept, and the value written where a package's own manifest
/// names none.
fn set_version(doc: &mut dyn TableLike, path: &Path, held: Held, version: &Version) -> Answer<()> {
    let keys = held.keys();
    let (last, tables) = keys.split_last().expect("a value is named by keys");
    let mut table = doc;
    for key in tables {
        table = table
            .get_mut(key)
            .and_then(Item::as_table_like_mut)
            .ok_or_else(|| names_none(path, &keys.join(".")))?;
    }
    if table.get(last).is_none() && held == Held::Workspace {
        return Err(names_none(path, &keys.join(".")));
    }
    set(table, last, version.as_str());
    Ok(())
}

fn names_none(path: &Path, keys: &str) -> Not {
    unreadable(path, format!("the manifest names no {keys}"))
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

#[cfg(test)]
mod tests;
