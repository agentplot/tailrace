//! npm's pack (C.11, 84, 87; design D6): the package whose `package.json`
//! is the shallowest an output holds, copied with the files the build
//! left under it into a scratch directory, its version set there with
//! `npm pkg set` and packed with `npm pack --ignore-scripts`, so the
//! build's own files are never edited and no script of the package runs
//! (15). The tarball is the artifact's bytes as npm wrote them (rule
//! core.5), named in npm by the package's own name (4).

use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;
use tailrace_seams::{
    Answer, ArtifactMeta, Ecosystem, EcosystemName, Not, Output, Packed, Version, Why,
};

use super::{NpmProjection, ECOSYSTEM};

/// What `npm pack --json` says of the one tarball it made.
#[derive(Deserialize)]
struct PackedJson {
    name: String,
    version: String,
    filename: String,
}

/// `npm` run in `at` with `args`, under `path` where given, its notices
/// of updates and funding off: its standard output, refused with its
/// standard error verbatim where it exits other than 0 (59), and
/// unreadable naming `npm` with the system's words where it cannot be
/// run.
fn npm(path: Option<&OsStr>, at: &Path, args: &[&OsStr]) -> Answer<Vec<u8>> {
    let source = NpmProjection::source();
    let mut command = Command::new("npm");
    command
        .args(args)
        .current_dir(at)
        .env("npm_config_update_notifier", "false")
        .env("npm_config_fund", "false")
        .env("npm_config_audit", "false");
    if let Some(path) = path {
        command.env("PATH", path);
    }
    let ran = command
        .output()
        .map_err(|e| Not::unreadable(&source, "npm", Some(e.to_string())))?;
    if !ran.status.success() {
        return Err(Not::refused(
            &source,
            Why::Said,
            Some(String::from_utf8_lossy(&ran.stderr).into_owned()),
        ));
    }
    Ok(ran.stdout)
}

/// The shallowest `package.json` among `output`'s files; refused naming
/// the output where it holds none.
fn manifest(output: &Output) -> Answer<&Path> {
    output
        .files
        .iter()
        .map(Path::new)
        .filter(|file| file.file_name() == Some(OsStr::new("package.json")))
        .min_by_key(|file| file.components().count())
        .ok_or_else(|| {
            Not::refused(
                &NpmProjection::source(),
                Why::Said,
                Some(format!(
                    "the output {} holds no package.json, and the npm projection packs the \
                     package one names",
                    output.name
                )),
            )
        })
}

/// 84, 87: `output` packed at `version` by the npm `path` finds, or the
/// process's own where it names none.
pub(super) fn pack(path: Option<&OsStr>, output: &Output, version: &Version) -> Answer<Packed> {
    let source = NpmProjection::source();
    let package_json = manifest(output)?;
    let dir = package_json.parent().unwrap_or(Path::new(""));
    let scratch = tempfile::tempdir().map_err(|e| {
        Not::unreadable(
            &source,
            "a scratch directory to pack in",
            Some(e.to_string()),
        )
    })?;
    let package = scratch.path().join("package");
    let packed = scratch.path().join("packed");
    let unwritten = |at: &Path, e: std::io::Error| {
        Not::unreadable(&source, at.display().to_string(), Some(e.to_string()))
    };
    fs::create_dir_all(&packed).map_err(|e| unwritten(&packed, e))?;
    for file in output.files.iter().map(Path::new) {
        let Ok(relative) = file.strip_prefix(dir) else {
            continue;
        };
        let to = package.join(relative);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(|e| unwritten(parent, e))?;
        }
        fs::copy(file, &to).map_err(|e| unwritten(file, e))?;
    }

    let set = format!("version={version}");
    npm(
        path,
        &package,
        &["pkg".as_ref(), "set".as_ref(), set.as_ref()],
    )?;
    let listed = npm(
        path,
        &package,
        &[
            "pack".as_ref(),
            "--json".as_ref(),
            "--ignore-scripts".as_ref(),
            "--pack-destination".as_ref(),
            packed.as_os_str(),
        ],
    )?;
    let unread =
        |said: String| Not::unreadable(&source, "what npm pack --json printed", Some(said));
    let listed: Vec<PackedJson> =
        serde_json::from_slice(&listed).map_err(|e| unread(e.to_string()))?;
    let [made] = <[PackedJson; 1]>::try_from(listed)
        .map_err(|listed| unread(format!("{} tarballs, and a pack makes one", listed.len())))?;
    if made.version != version.as_str() {
        return Err(Not::refused(
            &source,
            Why::Said,
            Some(format!(
                "npm packed {} at {}, and the version given is {version}",
                made.name, made.version
            )),
        ));
    }
    let tarball = packed.join(&made.filename);
    let bytes = fs::read(&tarball).map_err(|e| unwritten(&tarball, e))?;
    let meta = ArtifactMeta::new(
        Ecosystem::new(ECOSYSTEM),
        EcosystemName::new(made.name),
        version.clone(),
    )
    .map_err(|bad| Not::refused(&source, Why::Said, Some(bad.to_string())))?;
    Ok(Packed {
        bytes,
        meta: Some(meta),
        same_again: false,
    })
}

#[cfg(test)]
mod tests {
    use tailrace_seams::{BuildId, OutputName, Projection, Seam};
    use tempfile::TempDir;

    use super::*;
    use crate::profiles::projection::npm::NpmProjection;

    /// A projection whose `PATH` holds no npm, so a pack that ran it
    /// would answer unreadable.
    fn projection(dir: &TempDir) -> NpmProjection {
        let store = tailrace_fakes::FakeStore::new();
        let empty = dir.path().join("bin");
        fs::create_dir_all(&empty).unwrap();
        NpmProjection::new(dir.path(), Box::new(store), "http://127.0.0.1:7420/").with_path(empty)
    }

    #[test]
    fn an_output_with_no_package_json_is_refused_naming_it_and_runs_nothing() {
        let dir = TempDir::new().unwrap();
        let readme = dir.path().join("dist/README.md");
        fs::create_dir_all(readme.parent().unwrap()).unwrap();
        fs::write(&readme, "# widget\n").unwrap();
        let output = Output {
            name: OutputName::new("widget"),
            files: vec![readme.display().to_string()],
        };
        let packed =
            projection(&dir).pack(&output, &BuildId::new("b7e1"), Some(&Version::new("1.4.0")));
        match packed {
            Err(Not::Refused(r)) => {
                assert_eq!(r.source.seam, Seam::Projection);
                let said = r.said.unwrap();
                assert!(said.contains("widget"), "{said}");
                assert!(said.contains("no package.json"), "{said}");
            }
            other => panic!("an output with no package.json answered {other:?}"),
        }
    }

    #[test]
    fn a_pack_with_no_version_is_absent_naming_cache_entries_and_runs_nothing() {
        let dir = TempDir::new().unwrap();
        let output = Output {
            name: OutputName::new("widget"),
            files: vec![dir.path().join("package.json").display().to_string()],
        };
        match projection(&dir).pack(&output, &BuildId::new("b7e1"), None) {
            Err(Not::Absent(a)) => {
                assert_eq!(a.source, NpmProjection::source());
                assert_eq!(a.what, "cache entries");
            }
            other => panic!("a pack with no version answered {other:?}"),
        }
    }

    #[test]
    fn an_npm_that_cannot_be_run_is_unreadable_naming_npm() {
        let dir = TempDir::new().unwrap();
        let manifest = dir.path().join("dist/package.json");
        fs::create_dir_all(manifest.parent().unwrap()).unwrap();
        fs::write(&manifest, r#"{"name":"@family/widget","version":"0.0.0"}"#).unwrap();
        let output = Output {
            name: OutputName::new("widget"),
            files: vec![manifest.display().to_string()],
        };
        let packed =
            projection(&dir).pack(&output, &BuildId::new("b7e1"), Some(&Version::new("1.4.0")));
        match packed {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.what, "npm");
                assert!(u.said.is_some(), "{u:?}");
            }
            other => panic!("an npm that cannot be run answered {other:?}"),
        }
        assert_eq!(
            fs::read_to_string(&manifest).unwrap(),
            r#"{"name":"@family/widget","version":"0.0.0"}"#,
            "the build's own package.json was edited"
        );
    }
}
