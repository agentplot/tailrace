//! npm's layout (85, 47, 49; design D8): under the layout's place, each
//! package name's packument and its tarballs,
//!
//! ```text
//! <name>/index.json                    the packument
//! <name>/-/<base>-<version>.tgz        each tarball
//! ```
//!
//! `@scope/pkg` two directories deep and `<base>` its part after `/`.
//! Every tarball is read through the store and checked against its
//! layer's digest before anything is written, so a read that fails
//! leaves the layout as it was; then each tarball is written where one of
//! that digest is not there already, each packument written whole, and
//! every name and tarball not given removed, so the layout holds exactly
//! the versions given (47). A packument lists a version with the
//! `package.json` its tarball holds, never written back (C.11), its
//! tarball's path within the layout, its `integrity` and its `shasum`,
//! and names no address, so one layout serves under any base and names
//! no host, store or place on disk (49). A version not given, one the
//! policy refused among them, is in no packument (99).

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use serde_json::{json, Map, Value};
use sha1::Digest as _;
use tailrace_seams::{
    Answer, Artifact, ArtifactMeta, Digest, Holds, Layout, Not, Rendered, Store, Why,
};

use super::{tarball, NpmProjection, ECOSYSTEM};

/// Where a package's packument is kept under its name.
pub(super) const PACKUMENT: &str = "index.json";

/// The directory npm's registry keeps a package's tarballs in, under its
/// name.
pub(super) const TARBALLS: &str = "-";

/// The part of `name` after its scope: the name a tarball's file begins
/// with.
pub(super) fn base(name: &str) -> &str {
    name.rsplit_once('/').map_or(name, |(_, base)| base)
}

/// Whether `name` is an npm package's name a layout can hold: one part,
/// or `@scope/pkg`, neither naming a directory above it.
pub(super) fn is_name(name: &str) -> bool {
    let part = |p: &str| !p.is_empty() && !p.starts_with('.') && !p.contains(['/', '\\']);
    match name.split_once('/') {
        Some((scope, pkg)) => {
            scope.len() > 1 && scope.starts_with('@') && part(&scope[1..]) && part(pkg)
        }
        None => part(name) && !name.starts_with('@'),
    }
}

/// The path within a layout of `name`'s tarball at `version`.
pub(super) fn tarball_path(name: &str, version: &str) -> String {
    format!("{name}/{TARBALLS}/{}-{version}.tgz", base(name))
}

/// One version given, read and checked: its metadata, its tarball's bytes
/// and layer digest, and its `package.json`.
struct Given {
    meta: ArtifactMeta,
    layer: Digest,
    bytes: Vec<u8>,
    manifest: Map<String, Value>,
}

/// `artifact` read through `store`, checked against its layer's digest,
/// and its `package.json` read from its tarball; refused naming it where
/// it is of another ecosystem or a name no layout can hold.
fn read(store: &dyn Store, artifact: &Artifact) -> Answer<Given> {
    let source = NpmProjection::source();
    let meta = &artifact.meta;
    if meta.ecosystem().as_str() != ECOSYSTEM || !is_name(meta.name().as_str()) {
        return Err(Not::refused(
            &source,
            Why::Said,
            Some(format!(
                "{} {} {} is no npm package a layout holds",
                meta.ecosystem(),
                meta.name(),
                meta.version()
            )),
        ));
    }
    let (layer, bytes) = tarball::read(store, artifact)?;
    let manifest = tarball::package_json(artifact, &bytes)?;
    let manifest: Map<String, Value> = serde_json::from_slice(&manifest).map_err(|e| {
        Not::unreadable(
            &source,
            format!(
                "the package.json of {} {} at {}",
                meta.name(),
                meta.version(),
                artifact.digest
            ),
            Some(e.to_string()),
        )
    })?;
    Ok(Given {
        meta: meta.clone(),
        layer,
        bytes,
        manifest,
    })
}

/// The newest of `versions` by npm's order: the highest release, or the
/// highest prerelease where no release is given; the last by spelling
/// where none reads as a version.
fn latest<'a>(versions: impl Iterator<Item = &'a String> + Clone) -> Option<&'a String> {
    let parsed = versions
        .clone()
        .filter_map(|v| nodejs_semver::Version::parse(v).ok().map(|p| (p, v)));
    let releases = parsed.clone().filter(|(p, _)| !p.is_prerelease());
    releases
        .max_by(|(a, _), (b, _)| a.cmp(b))
        .or_else(|| parsed.max_by(|(a, _), (b, _)| a.cmp(b)))
        .map(|(_, v)| v)
        .or_else(|| versions.max())
}

/// `name`'s packument listing exactly `versions`.
fn packument(name: &str, versions: &BTreeMap<String, &Given>) -> Value {
    let listed: Map<String, Value> = versions
        .iter()
        .map(|(version, given)| {
            let mut entry = given.manifest.clone();
            let shasum: String = sha1::Sha1::digest(&given.bytes)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            entry.insert(
                "dist".into(),
                json!({
                    "tarball": tarball_path(name, version),
                    "integrity": format!("sha512-{}", STANDARD.encode(sha2::Sha512::digest(&given.bytes))),
                    "shasum": shasum,
                }),
            );
            (version.clone(), Value::Object(entry))
        })
        .collect();
    json!({
        "name": name,
        "dist-tags": { "latest": latest(versions.keys()) },
        "versions": listed,
    })
}

fn unwritten(path: &Path, e: io::Error) -> Not {
    Not::unreadable(
        &NpmProjection::source(),
        path.display().to_string(),
        Some(e.to_string()),
    )
}

/// `bytes` written at `path`, beside it first and renamed over it, so
/// npm never reads half a file.
fn write(path: &Path, bytes: &[u8]) -> Answer<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(dir).map_err(|e| unwritten(dir, e))?;
    let file = path
        .file_name()
        .map_or_else(Default::default, |f| f.to_string_lossy());
    let beside = dir.join(format!(".{file}.{}.partial", std::process::id()));
    let written = fs::write(&beside, bytes).and_then(|()| fs::rename(&beside, path));
    written.map_err(|e| {
        let _ = fs::remove_file(&beside);
        unwritten(path, e)
    })
}

/// The names of the packages the layout at `at` holds a directory for.
fn names_held(at: &Path) -> io::Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(at)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !entry.file_type()?.is_dir() || name.starts_with('.') {
            continue;
        }
        if !name.starts_with('@') {
            names.push(name);
            continue;
        }
        for scoped in fs::read_dir(entry.path())? {
            let scoped = scoped?;
            if scoped.file_type()?.is_dir() {
                names.push(format!("{name}/{}", scoped.file_name().to_string_lossy()));
            }
        }
    }
    Ok(names)
}

/// 85, 47, 49: the layout at `at` of exactly `artifacts`, every tarball
/// read through `store` before anything is written.
pub(super) fn render(store: &dyn Store, artifacts: &[Artifact], at: &str) -> Answer<Rendered> {
    let given = artifacts
        .iter()
        .map(|artifact| read(store, artifact))
        .collect::<Answer<Vec<Given>>>()?;
    let mut by_name: BTreeMap<String, BTreeMap<String, &Given>> = BTreeMap::new();
    for one in &given {
        by_name
            .entry(one.meta.name().to_string())
            .or_default()
            .insert(one.meta.version().to_string(), one);
    }

    let root = Path::new(at);
    fs::create_dir_all(root).map_err(|e| unwritten(root, e))?;
    for (name, versions) in &by_name {
        for (version, one) in versions {
            let path = root.join(tarball_path(name, version));
            let held = fs::read(&path).is_ok_and(|bytes| Digest::of(&bytes) == one.layer);
            if !held {
                write(&path, &one.bytes)?;
            }
        }
        let text = serde_json::to_vec_pretty(&packument(name, versions))
            .expect("a packument is plain data");
        write(&root.join(name).join(PACKUMENT), &text)?;
        let tarballs = root.join(name).join(TARBALLS);
        let named: Vec<String> = versions
            .keys()
            .map(|v| format!("{}-{v}.tgz", base(name)))
            .collect();
        for entry in fs::read_dir(&tarballs).map_err(|e| unwritten(&tarballs, e))? {
            let entry = entry.map_err(|e| unwritten(&tarballs, e))?;
            let file = entry.file_name().to_string_lossy().into_owned();
            if !named.contains(&file) {
                let path = entry.path();
                fs::remove_file(&path).map_err(|e| unwritten(&path, e))?;
            }
        }
    }
    for name in names_held(root).map_err(|e| unwritten(root, e))? {
        if !by_name.contains_key(&name) {
            let path = root.join(&name);
            fs::remove_dir_all(&path).map_err(|e| unwritten(&path, e))?;
            if let Some((scope, _)) = name.split_once('/') {
                let scope = root.join(scope);
                if fs::read_dir(&scope).is_ok_and(|mut d| d.next().is_none()) {
                    fs::remove_dir(&scope).map_err(|e| unwritten(&scope, e))?;
                }
            }
        }
    }
    Ok(Rendered {
        layout: Layout { at: at.to_string() },
        holds: Holds::Versions(artifacts.iter().map(|a| a.meta.clone()).collect()),
    })
}
