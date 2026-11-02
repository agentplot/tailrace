//! npm's registry protocol served from a layout (86, 46, 115, 99; design
//! D9). The request's path, its `%2f` and `%40` unescaped, names
//!
//! - `<name>`: the packument, from the layout alone, each version's
//!   `dist.tarball` made absolute under the base it is served at;
//! - `<name>/-/<base>-<version>.tgz`: the tarball's bytes, a version the
//!   layout lacks asked of the core once and then answered from the
//!   layout (50).
//!
//! Every answer that is not the bytes asked for is a JSON object whose
//! `error` npm prints after its status line: `404` absent, `403` refused,
//! `502` unreadable, the words the answer's own rendering, the source's
//! words verbatim (58, 59, 99). A method other than `GET` or `HEAD` is
//! `405`.

use std::fs;
use std::io;
use std::path::Path;

use serde_json::{json, Value};
use tailrace_seams::{
    Answer, EcosystemName, Layout, Miss, Not, Pinned, Request, Response, Version,
};

use super::layout::{base, is_name, tarball_path, PACKUMENT, TARBALLS};

const JSON: &str = "application/json";
const TARBALL: &str = "application/octet-stream";

/// A response whose body is `{"error": words}`, with `status`.
fn error(status: u16, words: impl Into<String>) -> Response {
    Response {
        status,
        content_type: JSON.into(),
        body: serde_json::to_vec(&json!({ "error": words.into() }))
            .expect("an error is plain data"),
    }
}

/// The response npm reads for `not`, an answer that is not the bytes it
/// asked for: `404` absent, `403` refused, `502` unreadable, the words
/// the answer's own.
fn not_answered(not: &Not) -> Response {
    let status = match not {
        Not::Absent(_) => 404,
        Not::Refused(_) => 403,
        Not::Unreadable(_) => 502,
    };
    error(status, not.to_string())
}

/// `502` naming what at `path` could not be read, with the system's
/// words.
fn unreadable(path: &Path, e: &dyn std::fmt::Display) -> Response {
    error(502, format!("could not read {}: {e}", path.display()))
}

/// `path` with npm's escapes of a scoped name, `%2f` and `%40`, read back
/// as `/` and `@`.
fn unescaped(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut rest = path;
    while let Some(at) = rest.find('%') {
        out.push_str(&rest[..at]);
        let escape = rest.get(at..at + 3).unwrap_or(&rest[at..]);
        match escape.to_ascii_lowercase().as_str() {
            "%2f" => out.push('/'),
            "%40" => out.push('@'),
            _ => {
                out.push('%');
                rest = &rest[at + 1..];
                continue;
            }
        }
        rest = &rest[at + 3..];
    }
    out.push_str(rest);
    out
}

/// What a request's path names in npm's registry protocol.
enum Asked {
    Packument(String),
    Tarball(Pinned),
    Nothing,
}

/// The path, unescaped, read as a packument's or a tarball's.
fn asked(path: &str) -> Asked {
    let path = unescaped(path);
    let path = path.trim_matches('/');
    if is_name(path) {
        return Asked::Packument(path.to_string());
    }
    let Some((name, file)) = path.split_once(&format!("/{TARBALLS}/")) else {
        return Asked::Nothing;
    };
    let version = file
        .strip_prefix(base(name))
        .and_then(|f| f.strip_prefix('-'))
        .and_then(|f| f.strip_suffix(".tgz"))
        .filter(|v| !v.is_empty() && !v.contains(['/', '\\']));
    match version {
        Some(version) if is_name(name) => Asked::Tarball(Pinned {
            name: EcosystemName::new(name),
            version: Version::new(version),
        }),
        _ => Asked::Nothing,
    }
}

/// `name`'s packument from the layout at `root`, each tarball's address
/// made absolute under `base`; `404` where the layout holds no version of
/// it, saying that `tailrace sync` holds a name `package.json` declares.
fn packument(root: &Path, name: &str, base: &str) -> Response {
    let path = root.join(name).join(PACKUMENT);
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return error(
                404,
                format!(
                    "the layout holds no version of {name}; tailrace sync holds a name \
                     package.json declares, so declare it there and run tailrace sync"
                ),
            )
        }
        Err(e) => return unreadable(&path, &e),
    };
    let mut packument: Value = match serde_json::from_slice(&bytes) {
        Ok(packument) => packument,
        Err(e) => return unreadable(&path, &e),
    };
    if let Some(versions) = packument["versions"].as_object_mut() {
        for entry in versions.values_mut() {
            if let Some(tarball) = entry["dist"]["tarball"].as_str() {
                entry["dist"]["tarball"] = Value::String(format!("{base}/{tarball}"));
            }
        }
    }
    Response {
        status: 200,
        content_type: JSON.into(),
        body: serde_json::to_vec(&packument).expect("a packument is plain data"),
    }
}

/// The tarball of `pin` from the layout at `root`; one the layout lacks
/// asked of the core once through `miss`, and answered from the layout
/// once the core holds it (115, 50).
fn tarball(root: &Path, pin: &Pinned, miss: Miss<'_>) -> Response {
    let path = root.join(tarball_path(pin.name.as_str(), pin.version.as_str()));
    let read = |path: &Path| match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(unreadable(path, &e)),
    };
    let bytes = match read(&path) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => {
            if let Err(not) = miss(pin) {
                return not_answered(&not);
            }
            match read(&path) {
                Ok(Some(bytes)) => bytes,
                Ok(None) => {
                    return error(
                        502,
                        format!(
                            "the layout holds no tarball of {} {} after the core held it",
                            pin.name, pin.version
                        ),
                    )
                }
                Err(response) => return response,
            }
        }
        Err(response) => return response,
    };
    Response {
        status: 200,
        content_type: TARBALL.into(),
        body: bytes,
    }
}

/// 86, 46, 115, 99: `request` of the layout at `layout`, answered in
/// npm's registry protocol.
pub(super) fn serve(layout: &Layout, request: &Request, miss: Miss<'_>) -> Answer<Response> {
    if request.method != "GET" && request.method != "HEAD" {
        return Ok(error(
            405,
            format!("{} is not a method npm's registry answers", request.method),
        ));
    }
    let root = Path::new(&layout.at);
    Ok(match asked(&request.path) {
        Asked::Packument(name) => packument(root, &name, &request.base),
        Asked::Tarball(pin) => tarball(root, &pin, miss),
        Asked::Nothing => error(404, format!("{} names no package or tarball", request.path)),
    })
}
