//! A registry in memory on a loopback port, for the registry store's unit
//! tests (design D17): repositories of blobs, manifests and tags spoken
//! to over the distribution protocol as far as the store speaks it, every
//! request kept, a mount it may decline, repositories it may make on
//! push or hold only where made, and a tag list it pages.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use timber_seams::Digest;

/// One request the stub was asked: its method and its path with query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Asked {
    pub method: String,
    pub path: String,
}

impl Asked {
    /// The repository the request is for, the path between `/v2/` and
    /// the operation, the operation read from the path's end, as a
    /// registry's routes match it: what follows the operation's word holds
    /// no `/` but an upload's, so a repository whose own path holds the
    /// word is read whole.
    pub(crate) fn repository(&self) -> Option<&str> {
        let rest = self.path.strip_prefix("/v2/")?;
        let bare = rest.split('?').next().unwrap_or(rest);
        ["/blobs/", "/manifests/", "/tags/", "/referrers/"]
            .iter()
            .filter_map(|op| bare.rfind(op))
            .max()
            .map(|at| &rest[..at])
    }
}

/// One repository the stub holds.
#[derive(Clone, Debug, Default)]
pub(crate) struct Held {
    pub blobs: BTreeMap<Digest, Vec<u8>>,
    pub manifests: BTreeMap<Digest, (String, Vec<u8>)>,
    pub tags: BTreeMap<String, Digest>,
}

#[derive(Default)]
struct State {
    repositories: BTreeMap<String, Held>,
    asked: Vec<Asked>,
    uploads: u64,
}

/// What the stub does beside holding what it is given.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Behaviour {
    /// It makes a repository at its first write, as zot does; otherwise
    /// a write to one it does not hold is `NAME_UNKNOWN`, as ECR answers
    /// where no creation template applies.
    pub makes_on_push: bool,
    /// It answers a mount 202, opening an upload, as a registry that
    /// declines cross-repository mounts does.
    pub declines_mounts: bool,
    /// The most tags one page of a tag list names.
    pub page: usize,
    /// It answers a tag list's every page after the first
    /// `NAME_UNKNOWN`, as a registry whose repository was deleted between
    /// pages does.
    pub loses_later_pages: bool,
}

impl Default for Behaviour {
    fn default() -> Self {
        Behaviour {
            makes_on_push: true,
            declines_mounts: false,
            page: 100,
            loses_later_pages: false,
        }
    }
}

/// The stub, serving until the test process ends.
#[derive(Clone)]
pub(crate) struct Stub {
    pub port: u16,
    state: Arc<Mutex<State>>,
}

/// The registry's words for a repository it does not hold.
pub(crate) const NAME_UNKNOWN: &str =
    r#"{"errors":[{"code":"NAME_UNKNOWN","message":"repository name not known to registry"}]}"#;

impl Stub {
    /// A stub behaving as `behaviour`, holding the repositories `made`.
    pub(crate) fn start(behaviour: Behaviour, made: &[&str]) -> Stub {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
        let port = listener.local_addr().unwrap().port();
        let mut state = State::default();
        for repository in made {
            state
                .repositories
                .insert(repository.to_string(), Held::default());
        }
        let state = Arc::new(Mutex::new(state));
        let served = state.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                answer(stream, &served, behaviour, port);
            }
        });
        Stub { port, state }
    }

    /// The repository text a registry store binds for `path` here.
    pub(crate) fn repository(&self, path: &str) -> String {
        format!("127.0.0.1:{}/{path}", self.port)
    }

    /// Every request asked so far, in order.
    pub(crate) fn asked(&self) -> Vec<Asked> {
        self.state.lock().unwrap().asked.clone()
    }

    /// What `repository` holds; none where the stub holds no such
    /// repository.
    pub(crate) fn held(&self, repository: &str) -> Option<Held> {
        self.state
            .lock()
            .unwrap()
            .repositories
            .get(repository)
            .cloned()
    }

    /// `tag` of `repository` put at `digest`, as another tool would.
    pub(crate) fn tag(&self, repository: &str, tag: &str, digest: &Digest) {
        self.state
            .lock()
            .unwrap()
            .repositories
            .entry(repository.to_string())
            .or_default()
            .tags
            .insert(tag.to_string(), digest.clone());
    }

    /// Every repository the stub holds.
    pub(crate) fn repositories(&self) -> Vec<String> {
        self.state
            .lock()
            .unwrap()
            .repositories
            .keys()
            .cloned()
            .collect()
    }
}

/// A request read whole: its method, path, headers and body.
fn read(stream: &TcpStream) -> Option<(String, String, Vec<(String, String)>, Vec<u8>)> {
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let mut parts = line.split_whitespace();
    let (method, path) = (parts.next()?.to_string(), parts.next()?.to_string());
    let mut headers = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).ok()?;
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        let (name, value) = line.split_once(':')?;
        headers.push((name.trim().to_lowercase(), value.trim().to_string()));
    }
    let length = headers
        .iter()
        .find(|(n, _)| n == "content-length")
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(0);
    let mut body = vec![0; length];
    reader.read_exact(&mut body).ok()?;
    Some((method, path, headers, body))
}

/// An answer: its status, headers and body.
struct Reply {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Reply {
    fn new(status: u16) -> Reply {
        Reply {
            status,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    fn header(mut self, name: &str, value: impl Into<String>) -> Reply {
        self.headers.push((name.into(), value.into()));
        self
    }

    fn body(mut self, body: impl Into<Vec<u8>>) -> Reply {
        self.body = body.into();
        self
    }

    fn error(status: u16, code: &str, message: &str) -> Reply {
        let body = if code == "NAME_UNKNOWN" {
            NAME_UNKNOWN.to_string()
        } else {
            json!({ "errors": [{ "code": code, "message": message }] }).to_string()
        };
        Reply::new(status)
            .header("content-type", "application/json")
            .body(body)
    }
}

fn answer(mut stream: TcpStream, state: &Mutex<State>, behaviour: Behaviour, port: u16) {
    let Some((method, path, headers, body)) = read(&stream) else {
        return;
    };
    let reply = {
        let mut state = state.lock().unwrap();
        state.asked.push(Asked {
            method: method.clone(),
            path: path.clone(),
        });
        handle(&mut state, behaviour, port, &method, &path, &headers, body)
    };
    let head = method == "HEAD";
    let mut text = format!("HTTP/1.1 {} Stub\r\n", reply.status);
    for (name, value) in &reply.headers {
        text.push_str(&format!("{name}: {value}\r\n"));
    }
    if !reply.headers.iter().any(|(n, _)| n == "content-length") {
        text.push_str(&format!("content-length: {}\r\n", reply.body.len()));
    }
    text.push_str("connection: close\r\n\r\n");
    let mut bytes = text.into_bytes();
    if !head {
        bytes.extend_from_slice(&reply.body);
    }
    let _ = stream.write_all(&bytes);
}

/// The query's value of `key`, percent-decoded.
fn query(path: &str, key: &str) -> Option<String> {
    let (_, query) = path.split_once('?')?;
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k == key).then(|| decoded(v))
    })
}

fn decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&text[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap()
}

/// The digests a manifest's bytes name: an index's manifests, or an image
/// manifest's config and layers.
fn named(bytes: &[u8]) -> Vec<(bool, Digest)> {
    let Ok(value) = serde_json::from_slice::<Value>(bytes) else {
        return Vec::new();
    };
    let digest = |v: &Value| v["digest"].as_str().and_then(|d| Digest::parse(d).ok());
    if let Some(manifests) = value["manifests"].as_array() {
        return manifests
            .iter()
            .filter_map(digest)
            .map(|d| (true, d))
            .collect();
    }
    let mut blobs: Vec<(bool, Digest)> = value["config"]
        .as_object()
        .and_then(|_| digest(&value["config"]))
        .map(|d| (false, d))
        .into_iter()
        .collect();
    if let Some(layers) = value["layers"].as_array() {
        blobs.extend(layers.iter().filter_map(digest).map(|d| (false, d)));
    }
    blobs
}

fn handle(
    state: &mut State,
    behaviour: Behaviour,
    port: u16,
    method: &str,
    path: &str,
    headers: &[(String, String)],
    body: Vec<u8>,
) -> Reply {
    if path == "/v2/" || path == "/v2" {
        return Reply::new(200).body("{}");
    }
    let asked = Asked {
        method: method.into(),
        path: path.into(),
    };
    let Some(repository) = asked.repository().map(str::to_string) else {
        return Reply::error(404, "NOT_FOUND", "no such route");
    };
    let rest = &path["/v2/".len() + repository.len()..];
    let bare = rest.split('?').next().unwrap_or(rest);
    let write = matches!(method, "PUT" | "POST");
    if !state.repositories.contains_key(&repository) {
        if write && behaviour.makes_on_push {
            state
                .repositories
                .insert(repository.clone(), Held::default());
        } else {
            return Reply::error(404, "NAME_UNKNOWN", "");
        }
    }

    if let Some(digest) = bare
        .strip_prefix("/blobs/")
        .filter(|d| !d.starts_with("uploads"))
    {
        let held = &state.repositories[&repository];
        return match Digest::parse(digest).ok().and_then(|d| held.blobs.get(&d)) {
            Some(bytes) => Reply::new(200)
                .header("content-length", bytes.len().to_string())
                .header("docker-content-digest", digest)
                .body(bytes.clone()),
            None => Reply::error(404, "BLOB_UNKNOWN", "blob unknown to registry"),
        };
    }
    if bare == "/blobs/uploads/" && method == "POST" {
        if let (Some(mount), Some(from)) = (query(path, "mount"), query(path, "from")) {
            let digest = Digest::parse(&mount).unwrap();
            let found = state
                .repositories
                .get(&from)
                .and_then(|r| r.blobs.get(&digest))
                .cloned();
            if let (Some(bytes), false) = (found, behaviour.declines_mounts) {
                state
                    .repositories
                    .get_mut(&repository)
                    .unwrap()
                    .blobs
                    .insert(digest.clone(), bytes);
                return Reply::new(201)
                    .header("location", format!("/v2/{repository}/blobs/{digest}"))
                    .header("docker-content-digest", digest.as_str());
            }
        }
        state.uploads += 1;
        return Reply::new(202).header(
            "location",
            format!(
                "http://127.0.0.1:{port}/v2/{repository}/blobs/uploads/{}",
                state.uploads
            ),
        );
    }
    if bare.starts_with("/blobs/uploads/") && method == "PUT" {
        let Some(digest) = query(path, "digest").and_then(|d| Digest::parse(&d).ok()) else {
            return Reply::error(400, "DIGEST_INVALID", "no digest");
        };
        if Digest::of(&body) != digest {
            return Reply::error(400, "DIGEST_INVALID", "the bytes are not the digest's");
        }
        state
            .repositories
            .get_mut(&repository)
            .unwrap()
            .blobs
            .insert(digest.clone(), body);
        return Reply::new(201).header("docker-content-digest", digest.as_str());
    }
    if let Some(reference) = bare.strip_prefix("/manifests/") {
        let held = state.repositories.get_mut(&repository).unwrap();
        if method == "PUT" {
            let media_type = headers
                .iter()
                .find(|(n, _)| n == "content-type")
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            for (manifest, digest) in named(&body) {
                let there = if manifest {
                    held.manifests.contains_key(&digest)
                } else {
                    held.blobs.contains_key(&digest)
                };
                if !there {
                    let code = if manifest {
                        "MANIFEST_UNKNOWN"
                    } else {
                        "MANIFEST_BLOB_UNKNOWN"
                    };
                    return Reply::error(400, code, &format!("{digest} is not in {repository}"));
                }
            }
            let digest = Digest::of(&body);
            held.manifests.insert(digest.clone(), (media_type, body));
            if Digest::parse(reference).is_err() {
                held.tags.insert(reference.to_string(), digest.clone());
            }
            return Reply::new(201).header("docker-content-digest", digest.as_str());
        }
        let digest = match Digest::parse(reference) {
            Ok(digest) => Some(digest),
            Err(_) => held.tags.get(reference).cloned(),
        };
        return match digest.and_then(|d| held.manifests.get(&d).map(|m| (d, m.clone()))) {
            Some((digest, (media_type, bytes))) => Reply::new(200)
                .header("content-type", media_type)
                .header("content-length", bytes.len().to_string())
                .header("docker-content-digest", digest.as_str())
                .body(bytes),
            None => Reply::error(404, "MANIFEST_UNKNOWN", "manifest unknown"),
        };
    }
    if bare == "/tags/list" {
        let held = &state.repositories[&repository];
        let last = query(path, "last");
        if last.is_some() && behaviour.loses_later_pages {
            return Reply::error(404, "NAME_UNKNOWN", "");
        }
        let tags: Vec<&String> = held
            .tags
            .keys()
            .filter(|t| last.as_ref().is_none_or(|l| t.as_str() > l.as_str()))
            .collect();
        let page: Vec<&String> = tags.iter().take(behaviour.page).copied().collect();
        let mut reply = Reply::new(200)
            .header("content-type", "application/json")
            .body(json!({ "name": repository, "tags": page }).to_string());
        if tags.len() > page.len() {
            let last = page.last().unwrap();
            reply = reply.header(
                "link",
                format!(
                    "</v2/{repository}/tags/list?n={}&last={last}>; rel=\"next\"",
                    behaviour.page
                ),
            );
        }
        return reply;
    }
    Reply::error(404, "NOT_FOUND", "no such route")
}
