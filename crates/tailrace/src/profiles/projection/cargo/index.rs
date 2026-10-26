//! A crate's line in a Cargo registry's index (design D5), written from
//! the crate's own normalized manifest: its name and version, its
//! dependencies of every kind and target, its checksum, its features, and
//! the links and Rust version it declares. Keys are written in one fixed
//! order, so one crate always writes one line.

use std::collections::BTreeMap;

use serde::Serialize;

use super::crate_file::CrateManifest;

/// The index of crates.io, which a dependency naming no registry is
/// from: an index line naming no registry would mean the registry that
/// holds the line, which is not crates.io (Cargo's registry index
/// reference).
pub(super) const CRATES_IO: &str = "https://github.com/rust-lang/crates.io-index";

/// One version's line.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct Line {
    name: String,
    vers: String,
    deps: Vec<Dependency>,
    cksum: String,
    features: BTreeMap<String, Vec<String>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    features2: BTreeMap<String, Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    links: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rust_version: Option<String>,
    yanked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    v: Option<u32>,
}

/// One dependency of a line: `name` as the crate's code names it, and
/// `package` the crate it is where the dependency renames it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct Dependency {
    name: String,
    req: String,
    features: Vec<String>,
    optional: bool,
    default_features: bool,
    target: Option<String>,
    kind: &'static str,
    registry: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    package: Option<String>,
}

/// The tables of a manifest that hold dependencies, and their kind.
const KINDS: [(&str, &str); 3] = [
    ("dependencies", "normal"),
    ("build-dependencies", "build"),
    ("dev-dependencies", "dev"),
];

/// The index line of the crate whose normalized manifest is `manifest`
/// and whose bytes' SHA-256 is `cksum`, in hexadecimal; the words naming
/// what of the manifest a line cannot be written from otherwise.
pub(super) fn line(manifest: &CrateManifest, cksum: &str) -> Result<String, String> {
    let table = &manifest.table;
    let package = table
        .get("package")
        .and_then(toml::Value::as_table)
        .ok_or("the crate's Cargo.toml declares no [package]")?;
    let mut deps = dependencies(table, None)?;
    if let Some(targets) = table.get("target").and_then(toml::Value::as_table) {
        for (cfg, target) in targets {
            let target = target
                .as_table()
                .ok_or(format!("target.{cfg} is not a table"))?;
            deps.extend(dependencies(target, Some(cfg))?);
        }
    }
    deps.sort();
    let (features, features2) = features(table)?;
    let text = |key: &str| -> Result<Option<String>, String> {
        match package.get(key) {
            None => Ok(None),
            Some(value) => value
                .as_str()
                .map(|s| Some(s.to_string()))
                .ok_or(format!("package.{key} is not a string")),
        }
    };
    let line = Line {
        name: manifest.name.clone(),
        vers: manifest.version.clone(),
        deps,
        cksum: cksum.to_string(),
        v: (!features2.is_empty()).then_some(2),
        features,
        features2,
        links: text("links")?,
        rust_version: text("rust-version")?,
        yanked: false,
    };
    Ok(serde_json::to_string(&line).expect("an index line is JSON"))
}

/// Every dependency `table` declares, of every kind, for `target` where
/// it is a target's.
fn dependencies(table: &toml::Table, target: Option<&str>) -> Result<Vec<Dependency>, String> {
    let mut deps = Vec::new();
    for (key, kind) in KINDS {
        let Some(declared) = table.get(key) else {
            continue;
        };
        let declared = declared.as_table().ok_or(format!("{key} is not a table"))?;
        for (name, spec) in declared {
            deps.push(dependency(name, spec, kind, target)?);
        }
    }
    Ok(deps)
}

/// The dependency `name` declares with `spec`, a requirement alone or a
/// table of it.
fn dependency(
    name: &str,
    spec: &toml::Value,
    kind: &'static str,
    target: Option<&str>,
) -> Result<Dependency, String> {
    let mut dep = Dependency {
        name: name.to_string(),
        req: String::new(),
        features: Vec::new(),
        optional: false,
        default_features: true,
        target: target.map(str::to_string),
        kind,
        registry: CRATES_IO.to_string(),
        package: None,
    };
    let spec = match spec {
        toml::Value::String(req) => {
            dep.req = req.clone();
            return Ok(dep);
        }
        toml::Value::Table(spec) => spec,
        _ => {
            return Err(format!(
                "the dependency {name} is neither a requirement nor a table"
            ))
        }
    };
    let text = |key: &str| spec.get(key).and_then(toml::Value::as_str);
    let flag = |key: &str| spec.get(key).and_then(toml::Value::as_bool);
    dep.req = text("version")
        .ok_or(format!("the dependency {name} names no version"))?
        .to_string();
    if let Some(features) = spec.get("features") {
        dep.features = strings(features).ok_or(format!(
            "the dependency {name}'s features are not a list of strings"
        ))?;
    }
    dep.optional = flag("optional").unwrap_or(false);
    dep.default_features = flag("default-features")
        .or(flag("default_features"))
        .unwrap_or(true);
    if let Some(index) = text("registry-index") {
        dep.registry = index.to_string();
    } else if let Some(registry) = text("registry") {
        return Err(format!(
            "the dependency {name} names the registry {registry} by name and not by its index"
        ));
    }
    dep.package = text("package").map(str::to_string);
    Ok(dep)
}

/// The manifest's features: those an older Cargo reads, and in the
/// second those naming a dependency by `dep:` or a weak `?/`, which only
/// a Cargo reading `v` 2 lines does.
fn features(
    table: &toml::Table,
) -> Result<(BTreeMap<String, Vec<String>>, BTreeMap<String, Vec<String>>), String> {
    let mut features = BTreeMap::new();
    let mut features2 = BTreeMap::new();
    let Some(declared) = table.get("features") else {
        return Ok((features, features2));
    };
    let declared = declared.as_table().ok_or("features is not a table")?;
    for (feature, enables) in declared {
        let enables =
            strings(enables).ok_or(format!("the feature {feature} is not a list of strings"))?;
        let second = enables
            .iter()
            .any(|e| e.starts_with("dep:") || e.contains("?/"));
        if second {
            features2.insert(feature.clone(), enables);
        } else {
            features.insert(feature.clone(), enables);
        }
    }
    Ok((features, features2))
}

fn strings(value: &toml::Value) -> Option<Vec<String>> {
    value
        .as_array()?
        .iter()
        .map(|v| v.as_str().map(str::to_string))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::crate_file;
    use super::super::fixture::crate_of;
    use super::*;

    const FAMILY: &str = "sparse+https://tailrace.invalid/cargo/";

    fn line_of(manifest: &str) -> Result<String, String> {
        let bytes = crate_of("fam-app", "2.0.0", manifest);
        line(&crate_file::manifest(&bytes).unwrap(), "c0ffee")
    }

    #[test]
    fn every_kind_of_dependency_and_a_dep_feature_are_written_as_cargo_reads_them() {
        let manifest = format!(
            "[package]\n\
             name = \"fam-app\"\n\
             version = \"2.0.0\"\n\
             links = \"fam\"\n\
             rust-version = \"1.80\"\n\
             \n\
             [dependencies.fam-lib]\n\
             version = \"^1.4.0\"\n\
             registry-index = \"{FAMILY}\"\n\
             \n\
             [dependencies.json]\n\
             version = \"1\"\n\
             package = \"serde_json\"\n\
             \n\
             [dependencies.serde]\n\
             version = \"1.0\"\n\
             optional = true\n\
             default-features = false\n\
             features = [\"derive\"]\n\
             \n\
             [dev-dependencies.tempfile]\n\
             version = \"3\"\n\
             \n\
             [target.\"cfg(unix)\".dependencies.libc]\n\
             version = \"0.2\"\n\
             \n\
             [features]\n\
             default = [\"plain\"]\n\
             plain = []\n\
             serde = [\"dep:serde\"]\n"
        );
        let expected = format!(
            "{{\"name\":\"fam-app\",\"vers\":\"2.0.0\",\"deps\":[\
             {{\"name\":\"fam-lib\",\"req\":\"^1.4.0\",\"features\":[],\"optional\":false,\
             \"default_features\":true,\"target\":null,\"kind\":\"normal\",\
             \"registry\":\"{FAMILY}\"}},\
             {{\"name\":\"json\",\"req\":\"1\",\"features\":[],\"optional\":false,\
             \"default_features\":true,\"target\":null,\"kind\":\"normal\",\
             \"registry\":\"{CRATES_IO}\",\"package\":\"serde_json\"}},\
             {{\"name\":\"libc\",\"req\":\"0.2\",\"features\":[],\"optional\":false,\
             \"default_features\":true,\"target\":\"cfg(unix)\",\"kind\":\"normal\",\
             \"registry\":\"{CRATES_IO}\"}},\
             {{\"name\":\"serde\",\"req\":\"1.0\",\"features\":[\"derive\"],\"optional\":true,\
             \"default_features\":false,\"target\":null,\"kind\":\"normal\",\
             \"registry\":\"{CRATES_IO}\"}},\
             {{\"name\":\"tempfile\",\"req\":\"3\",\"features\":[],\"optional\":false,\
             \"default_features\":true,\"target\":null,\"kind\":\"dev\",\
             \"registry\":\"{CRATES_IO}\"}}],\
             \"cksum\":\"c0ffee\",\
             \"features\":{{\"default\":[\"plain\"],\"plain\":[]}},\
             \"features2\":{{\"serde\":[\"dep:serde\"]}},\
             \"links\":\"fam\",\"rust_version\":\"1.80\",\"yanked\":false,\"v\":2}}"
        );
        assert_eq!(line_of(&manifest).unwrap(), expected);
    }

    #[test]
    fn a_crate_with_no_dependencies_or_features_writes_a_line_cargo_reads_as_version_1() {
        assert_eq!(
            line_of("[package]\nname = \"fam-app\"\nversion = \"2.0.0\"\n").unwrap(),
            "{\"name\":\"fam-app\",\"vers\":\"2.0.0\",\"deps\":[],\"cksum\":\"c0ffee\",\
             \"features\":{},\"yanked\":false}"
        );
    }

    #[test]
    fn a_weak_feature_is_a_second_feature() {
        let line = line_of(
            "[package]\nname = \"fam-app\"\nversion = \"2.0.0\"\n\n\
             [features]\nweak = [\"serde?/std\"]\n",
        )
        .unwrap();
        assert!(
            line.contains("\"features\":{},\"features2\":{\"weak\":[\"serde?/std\"]}"),
            "{line}"
        );
        assert!(line.ends_with(",\"v\":2}"), "{line}");
    }

    #[test]
    fn a_dependency_with_no_version_or_a_registry_by_name_is_answered_in_words() {
        assert_eq!(
            line_of(
                "[package]\nname = \"fam-app\"\nversion = \"2.0.0\"\n\n\
                 [dependencies.fam-lib]\npath = \"../fam\"\n"
            ),
            Err("the dependency fam-lib names no version".into())
        );
        assert_eq!(
            line_of(
                "[package]\nname = \"fam-app\"\nversion = \"2.0.0\"\n\n\
                 [dependencies.fam-lib]\nversion = \"1\"\nregistry = \"family\"\n"
            ),
            Err(
                "the dependency fam-lib names the registry family by name and not by its index"
                    .into()
            )
        );
    }
}
