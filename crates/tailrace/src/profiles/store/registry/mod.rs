//! The store seam (B.1) over an OCI registry, spoken to over the
//! distribution protocol (C.1, ADR 0003): bytes as blobs, an artifact or
//! an index as a manifest put by its own digest in the bytes the layout
//! writes, an attestation as a referrer of its subject, and a name as a
//! tag, at bytes through a cache manifest (design D2 to D8, D14).
//!
//! Everything the store holds is held in the one repository the host
//! names, `<host>[:<port>]/<path>`, addressed by digest, on every
//! registry (design D2); the store never makes a repository. A loopback
//! host is spoken to over plain HTTP, as Docker speaks to one, and every
//! other over HTTPS (design D3). A digest or name the repository does not
//! hold is absent; a registry that refuses is refused in its own words;
//! one that cannot be reached, or answers what cannot be read, or fails
//! with a 5xx, is unreadable in the client's or the registry's words
//! (58, 59, rule core.19).

mod client;
mod tag;

#[cfg(test)]
pub(crate) mod stub;

use std::cell::RefCell;
use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;
use tailrace_seams::{
    Answer, Digest, Ecosystem, EcosystemName, Envelope, Held, Manifest, Name, Not, Put, Seam,
    Source, Store, Version, Why, REPOSITORY_COMPONENT,
};
use ureq::http::{Method, StatusCode};

use super::oci::{
    self, Descriptor, Referrers, EMPTY_JSON, ENVELOPE, IMAGE_INDEX, IMAGE_MANIFEST, OCTETS,
};
use client::{encoded, Answered, Client};

/// What a cache entry's name begins with, and the space of repositories
/// under the declared one its entries are held in, one per build profile
/// (52, C.1; design D9).
const CACHE_SPACE: &str = "cache";

/// What a manifest request accepts: tailrace's two manifest types.
const MANIFESTS: &str = "application/vnd.oci.image.manifest.v1+json, \
                         application/vnd.oci.image.index.v1+json";

/// How long name reads a tag again while it answers the place it was
/// moved from: a registry may answer a tag's former digest for seconds
/// after the put, ECR for over ten (C.1, ADR 0015).
const BOUND: Duration = Duration::from_secs(30);

/// The wait before a tag is read again the first time; each wait after
/// is twice the one before.
const FIRST_WAIT: Duration = Duration::from_millis(250);

/// A store held in one repository of an OCI registry. Every answer names
/// the store seam and the repository.
pub struct Registry {
    repository: String,
    host: String,
    path: String,
    origin: String,
    base: String,
    source: Source,
    client: Client,
    hook: Option<Box<dyn Fn(&str)>>,
    bound: Duration,
    build: Option<String>,
    push: bool,
    mounts: RefCell<Vec<(String, bool)>>,
}

/// The space of repositories under the declared one an artifact's names
/// are held in, one per ecosystem and name (138, C.1; design D8).
const ARTIFACT_SPACE: &str = "artifact";

/// Where a name is held: the repository path it is a tag of, the tag,
/// and the space the repository is in (design D8 to D10).
struct Place {
    repository: String,
    tag: String,
    space: Space,
}

/// The spaces of the store's names on a registry (C.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Space {
    Declared,
    Artifact,
    Cache,
    Component,
}

/// The few fields of a manifest's JSON the store reads before its form.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Typed {
    #[serde(default)]
    media_type: Option<String>,
    #[serde(default)]
    artifact_type: Option<String>,
}

/// Whether `host`, with its port, is the loopback interface's.
fn loopback(host: &str) -> bool {
    let name = match host.strip_prefix('[') {
        Some(rest) => rest.split(']').next().map(|h| format!("[{h}]")),
        None => host.split(':').next().map(str::to_string),
    };
    matches!(
        name.as_deref(),
        Some("localhost") | Some("127.0.0.1") | Some("[::1]")
    )
}

impl Registry {
    /// The repository `repository` names, `<host>[:<port>]/<path>`;
    /// refused naming the text when it names no host or no path.
    pub fn new(repository: &str) -> Answer<Registry> {
        let source = Source::new(Seam::Store, format!("OCI registry {repository}"));
        let refused = |why: &str| {
            Not::refused(
                &source,
                Why::Said,
                Some(format!(
                    "{repository:?} is not a registry host followed by a repository path: {why}"
                )),
            )
        };
        let Some((host, path)) = repository.split_once('/') else {
            return Err(refused("it holds no /"));
        };
        if host.is_empty() || path.is_empty() {
            return Err(refused("its host or its path is empty"));
        }
        if host.contains("://") || repository.contains(char::is_whitespace) {
            return Err(refused("it names a scheme or holds a space"));
        }
        if path.split('/').any(str::is_empty) {
            return Err(refused("its path holds an empty part"));
        }
        let scheme = if loopback(host) { "http" } else { "https" };
        let origin = format!("{scheme}://{host}");
        Ok(Registry {
            repository: repository.to_string(),
            host: host.to_string(),
            path: path.to_string(),
            base: format!("{origin}/v2/{path}/"),
            origin,
            client: Client::new(host, &source),
            source,
            hook: None,
            bound: BOUND,
            build: None,
            push: false,
            mounts: RefCell::new(Vec::new()),
        })
    }

    /// The registry bound for the build profile `build`, whose cache
    /// repository, `cache/<build>` under the declared one, holds each
    /// cache entry; and, where `push`, putting bytes there too, since a
    /// push writes cache entries and nothing more, so a credential that
    /// writes `cache/` alone pushes them (52, 135, C.1; design D9).
    pub fn bound_for(mut self, build: &str, push: bool) -> Registry {
        self.build = Some(build.to_string());
        self.push = push;
        self
    }

    /// The store seam and this profile, naming the repository.
    pub fn source(&self) -> &Source {
        &self.source
    }

    /// The repository, as the host names it.
    pub fn repository(&self) -> &str {
        &self.repository
    }

    /// Where the repository's API is: `<scheme>://<host>/v2/<path>/`.
    pub fn base(&self) -> &str {
        &self.base
    }

    /// `hook` run between a tag's put and its read back, with the tag,
    /// so a test can lose a race on purpose (design D7).
    #[doc(hidden)]
    pub fn between_put_and_read_back(&mut self, hook: impl Fn(&str) + 'static) {
        self.hook = Some(Box::new(hook));
    }

    /// Each mount this store asked, by the repository it was into, and
    /// whether the registry took it, so a run against a registry reports
    /// whether it takes a cross-repository mount (design D8, ADR 0023).
    #[doc(hidden)]
    pub fn mounts(&self) -> Vec<(String, bool)> {
        self.mounts.borrow().clone()
    }

    /// How long a tag may answer a place it was at after a move, before
    /// or after one read has shown the move: the profile's bound, 30 s,
    /// or the one a test set (C.1, 36, ADR 0015).
    pub fn bound(&self) -> Duration {
        self.bound
    }

    /// Whether a name moves only from the digest it is expected at by the
    /// registry's own condition on the put: never, since the distribution
    /// specification defines no conditional tag put and no registry this
    /// profile binds honours one. name's check before the put is a read a
    /// lagging replica defeats, so its refusal of a move from a place the
    /// name has left is best effort (C.1, ADR 0015).
    pub fn names_conditional(&self) -> bool {
        false
    }

    /// A tag read again for at most `bound` while it answers the place
    /// it was moved from, in place of the profile's, so a test sees the
    /// bound pass (ADR 0015).
    #[doc(hidden)]
    pub fn read_again_within(&mut self, bound: Duration) {
        self.bound = bound;
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    /// Where `path` of `repository`'s API is: `<scheme>://<host>/v2/<repository>/<path>`.
    fn url_in(&self, repository: &str, path: &str) -> String {
        format!("{}/v2/{repository}/{path}", self.origin)
    }

    /// `location`, as a registry answers it, as a URL.
    fn absolute(&self, location: &str) -> String {
        if location.starts_with("http://") || location.starts_with("https://") {
            location.to_string()
        } else if location.starts_with('/') {
            format!("{}{location}", self.origin)
        } else {
            format!("{}/{location}", self.origin)
        }
    }

    /// `method` of `url`, a request for `repository`, as the client
    /// sends it (design D8).
    fn send(
        &self,
        repository: &str,
        method: Method,
        url: &str,
        headers: &[(&str, &str)],
        body: Option<&[u8]>,
    ) -> Answer<Answered> {
        self.client.send(repository, method, url, headers, body)
    }

    /// What a `HEAD` of `repository` the operation does not take is: what
    /// a `GET` of the same URL answers, which carries the registry's words
    /// where the `HEAD` carried none.
    fn other_head(
        &self,
        repository: &str,
        url: &str,
        accept: &[(&str, &str)],
        head: &Answered,
    ) -> Not {
        match self.send(repository, Method::GET, url, accept, None) {
            Ok(got) if !got.status.is_success() => self.other(url, &got),
            Ok(_) => self.other(url, head),
            Err(not) => not,
        }
    }

    /// What an answer the operation does not take is: refused in the
    /// registry's words, or unreadable naming `url` for a 5xx.
    fn other(&self, url: &str, answered: &Answered) -> Not {
        if answered.status.is_server_error() {
            Not::unreadable(&self.source, url, Some(answered.words()))
        } else {
            Not::refused(&self.source, Why::Said, Some(answered.words()))
        }
    }

    /// What a write to `repository` the registry did not take is: as
    /// [`Registry::other`] answers it, and, where the registry answers
    /// it 404, holding no such repository, as one that makes repositories
    /// only by a creation template does where none applies, refused
    /// naming the repository the write was for beside the registry's own
    /// words, so an operator reads which one to make; nothing is asked
    /// again (58, 59, C.1; design D13).
    fn other_put(&self, repository: &str, url: &str, answered: &Answered) -> Not {
        if answered.status != StatusCode::NOT_FOUND {
            return self.other(url, answered);
        }
        Not::refused(
            &self.source,
            Why::Said,
            Some(format!(
                "the registry holds no repository {}/{repository}, which the write was for: {}",
                self.host,
                answered.words()
            )),
        )
    }

    /// That `bytes`, answered for `digest`, are its bytes; refused naming
    /// both otherwise (2).
    fn same(&self, digest: &Digest, bytes: &[u8]) -> Answer<()> {
        let answered = Digest::of(bytes);
        if &answered != digest {
            return Err(Not::refused(
                &self.source,
                Why::OtherBytes {
                    expected: digest.clone(),
                    answered,
                },
                None,
            ));
        }
        Ok(())
    }

    /// That the digest a put's answer names, where it names one, is the
    /// digest of what was put; refused naming both otherwise (1).
    fn named(&self, digest: &Digest, answered: &Answered) -> Answer<()> {
        match answered
            .header("docker-content-digest")
            .and_then(|d| Digest::parse(d).ok())
        {
            Some(named) if &named != digest => Err(Not::refused(
                &self.source,
                Why::OtherBytes {
                    expected: digest.clone(),
                    answered: named,
                },
                None,
            )),
            _ => Ok(()),
        }
    }

    /// The size of the blob held under `digest`; none when it is not.
    fn blob_size(&self, repository: &str, digest: &Digest) -> Answer<Option<u64>> {
        let url = self.url_in(repository, &format!("blobs/{digest}"));
        let answered = self.send(repository, Method::HEAD, &url, &[], None)?;
        match answered.status {
            StatusCode::OK => self.length(&url, &answered).map(Some),
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(self.other_head(repository, &url, &[], &answered)),
        }
    }

    /// The media type and size of the manifest `reference`, a digest or a
    /// tag, names; none when it names none.
    fn manifest_head(
        &self,
        repository: &str,
        reference: &str,
    ) -> Answer<Option<(String, u64, Option<Digest>)>> {
        let url = self.url_in(repository, &format!("manifests/{reference}"));
        let accept = [("accept", MANIFESTS)];
        let answered = self.send(repository, Method::HEAD, &url, &accept, None)?;
        match answered.status {
            StatusCode::OK => {
                let size = self.length(&url, &answered)?;
                let media_type = answered
                    .header("content-type")
                    .map(|t| t.split(';').next().unwrap_or(t).trim().to_string())
                    .unwrap_or_else(|| IMAGE_MANIFEST.into());
                let digest = answered
                    .header("docker-content-digest")
                    .and_then(|d| Digest::parse(d).ok());
                Ok(Some((media_type, size, digest)))
            }
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(self.other_head(repository, &url, &accept, &answered)),
        }
    }

    fn length(&self, url: &str, answered: &Answered) -> Answer<u64> {
        answered
            .header("content-length")
            .and_then(|l| l.parse().ok())
            .ok_or_else(|| {
                Not::unreadable(
                    &self.source,
                    url,
                    Some("the registry answered no Content-Length".into()),
                )
            })
    }

    /// `bytes` uploaded as the blob `digest` in one request after the
    /// upload is opened (design D5).
    fn upload(&self, repository: &str, digest: &Digest, bytes: &[u8]) -> Answer<()> {
        let url = self.url_in(repository, "blobs/uploads/");
        let opened = self.send(repository, Method::POST, &url, &[], Some(&[]))?;
        if opened.status != StatusCode::ACCEPTED {
            return Err(self.other_put(repository, &url, &opened));
        }
        let location = opened.header("location").ok_or_else(|| {
            Not::unreadable(
                &self.source,
                &url,
                Some("the registry opened an upload and answered no Location".into()),
            )
        })?;
        self.upload_to(repository, location, digest, bytes)
    }

    /// `bytes` put as the blob `digest` of `repository` through the
    /// upload the registry opened at `location`, in one request (design
    /// D5).
    fn upload_to(
        &self,
        repository: &str,
        location: &str,
        digest: &Digest,
        bytes: &[u8],
    ) -> Answer<()> {
        let location = self.absolute(location);
        let sep = if location.contains('?') { '&' } else { '?' };
        let url = format!("{location}{sep}digest={}", encoded(digest.as_str()));
        let put = self.send(
            repository,
            Method::PUT,
            &url,
            &[("content-type", OCTETS)],
            Some(bytes),
        )?;
        if !put.status.is_success() {
            return Err(self.other_put(repository, &url, &put));
        }
        self.named(digest, &put)
    }

    /// `bytes` of `media_type` put as the manifest `reference`, a digest
    /// or a tag, answering what the registry answered.
    fn put_manifest(
        &self,
        repository: &str,
        reference: &str,
        media_type: &str,
        bytes: &[u8],
    ) -> Answer<Answered> {
        let url = self.url_in(repository, &format!("manifests/{reference}"));
        let put = self.send(
            repository,
            Method::PUT,
            &url,
            &[("content-type", media_type)],
            Some(bytes),
        )?;
        if !put.status.is_success() {
            return Err(self.other_put(repository, &url, &put));
        }
        self.named(&Digest::of(bytes), &put)?;
        Ok(put)
    }

    /// The bytes of the blob `digest`, once they are found to be its;
    /// none when it is not held. A redirect, as ECR's to its bucket, is
    /// followed, and the digest checked on what finally arrives (design
    /// D3).
    fn blob(&self, repository: &str, digest: &Digest) -> Answer<Option<Vec<u8>>> {
        let url = self.url_in(repository, &format!("blobs/{digest}"));
        let answered = self.send(repository, Method::GET, &url, &[], None)?;
        match answered.status {
            StatusCode::OK => {
                self.same(digest, &answered.body)?;
                Ok(Some(answered.body))
            }
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(self.other(&url, &answered)),
        }
    }

    /// The bytes of the manifest `digest` and the media type they name,
    /// once they are found to be its; none when it is not held.
    fn manifest(&self, repository: &str, digest: &Digest) -> Answer<Option<(Vec<u8>, Typed)>> {
        let url = self.url_in(repository, &format!("manifests/{digest}"));
        let answered = self.send(
            repository,
            Method::GET,
            &url,
            &[("accept", MANIFESTS)],
            None,
        )?;
        match answered.status {
            StatusCode::OK => {
                self.same(digest, &answered.body)?;
                let mut typed: Typed = serde_json::from_slice(&answered.body)
                    .map_err(|e| Not::unreadable(&self.source, &url, Some(e.to_string())))?;
                if typed.media_type.is_none() {
                    typed.media_type = answered
                        .header("content-type")
                        .map(|t| t.split(';').next().unwrap_or(t).trim().to_string());
                }
                Ok(Some((answered.body, typed)))
            }
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(self.other(&url, &answered)),
        }
    }

    /// The tag a name is held under; refused naming the name when no
    /// tag can hold it (design D8).
    fn tag(&self, name: &Name) -> Answer<String> {
        tag::of(name).map_err(|words| Not::refused(&self.source, Why::Said, Some(words)))
    }

    /// The digest `tag` names; none when it names nothing. At a cache
    /// manifest it is the digest of the bytes the manifest names (design
    /// D14); at any other manifest the one the registry names it by, or
    /// the digest of the manifest's bytes where it names none.
    fn tag_at(&self, repository: &str, tag: &str) -> Answer<Option<Digest>> {
        let url = self.url_in(repository, &format!("manifests/{tag}"));
        let answered = self.send(
            repository,
            Method::GET,
            &url,
            &[("accept", MANIFESTS)],
            None,
        )?;
        match answered.status {
            StatusCode::OK => {}
            StatusCode::NOT_FOUND => return Ok(None),
            _ => return Err(self.other(&url, &answered)),
        }
        let cached = oci::cached(&answered.body)
            .map_err(|words| Not::unreadable(&self.source, &url, Some(words)))?;
        Ok(Some(cached.unwrap_or_else(|| {
            answered
                .header("docker-content-digest")
                .and_then(|d| Digest::parse(d).ok())
                .unwrap_or_else(|| Digest::of(&answered.body))
        })))
    }

    /// The cache manifest of the bytes held under `digest`, `size` bytes
    /// long, held in the repository: the empty config put, and the
    /// manifest put by its digest unless it is held already (design
    /// D14).
    fn cache(&self, repository: &str, digest: &Digest, size: u64) -> Answer<Vec<u8>> {
        let bytes = oci::cache(digest, size);
        let at = Digest::of(&bytes);
        if self.manifest_head(repository, at.as_str())?.is_none() {
            self.put_bytes_in(repository, EMPTY_JSON)?;
            self.put_manifest(repository, at.as_str(), IMAGE_MANIFEST, &bytes)?;
        }
        Ok(bytes)
    }

    /// `bytes` held as a blob of `repository`: asked after first, and
    /// uploaded only when it is not held (5, 66).
    fn put_bytes_in(&self, repository: &str, bytes: &[u8]) -> Answer<Put> {
        let digest = Digest::of(bytes);
        if self.blob_size(repository, &digest)?.is_some() {
            return Ok(Put {
                digest,
                held_already: true,
            });
        }
        self.upload(repository, &digest, bytes)?;
        Ok(Put {
            digest,
            held_already: false,
        })
    }

    /// The repository path `parts` make under the declared one, each
    /// written as one path component; refused naming it where it runs
    /// past 255 characters (design D12).
    fn under(&self, parts: &[&str]) -> Answer<String> {
        tag::repository(&self.host, &self.path, parts)
            .map_err(|words| Not::refused(&self.source, Why::Said, Some(words)))
    }

    /// Where `name` is held (design D8 to D10): the repository it is a
    /// tag of, and the tag, read as an artifact's name first, then a
    /// cache entry's, a channel's and a version's, so a `:` in a build
    /// identity or a version is never read as a component's. An
    /// artifact's name is the tag of its version in
    /// `artifact/<ecosystem>/<name>` under the declared repository; a
    /// cache entry's the tag of its build identity in the bound build
    /// profile's cache repository, refused naming the entry where the
    /// store is bound for none; a declared component's channel and
    /// version the tags of the channel's name and the version's tag in
    /// the component's own repository; every other name, the undeclared
    /// component's versions and channels among them, a tag of the
    /// declared repository (125).
    fn place(&self, name: &Name) -> Answer<Place> {
        if let Some((ecosystem, artifact, version)) = name.as_artifact() {
            return Ok(Place {
                repository: self.under(&[ARTIFACT_SPACE, ecosystem.as_str(), artifact.as_str()])?,
                tag: self.tag(&Name::new(version.as_str()))?,
                space: Space::Artifact,
            });
        }
        if let Some(identity) = name
            .as_str()
            .strip_prefix(CACHE_SPACE)
            .and_then(|rest| rest.strip_prefix('/'))
        {
            let repository = self.cache_repository()?.ok_or_else(|| {
                Not::refused(
                    &self.source,
                    Why::Said,
                    Some(format!(
                        "{name} is a cache entry's name, and this store is bound for no build \
                         profile, whose cache repository holds its entries"
                    )),
                )
            })?;
            return Ok(Place {
                repository,
                tag: self.tag(&Name::new(identity))?,
                space: Space::Cache,
            });
        }
        let component = match (name.as_channel(), name.as_version()) {
            (Some(channel), _) => channel
                .component
                .map(|component| (component, channel.name.to_string())),
            (None, Some((component, tag))) => Some((component, tag)),
            (None, None) => None,
        };
        if let Some((component, tag)) = component {
            return Ok(Place {
                repository: self.under(&[component.as_str()])?,
                tag: self.tag(&Name::new(tag))?,
                space: Space::Component,
            });
        }
        Ok(Place {
            repository: self.path.clone(),
            tag: self.tag(name)?,
            space: Space::Declared,
        })
    }

    /// The bound build profile's cache repository, `cache/<build>` under
    /// the declared one; none where the store is bound for none (design
    /// D9).
    fn cache_repository(&self) -> Answer<Option<String>> {
        self.build
            .as_deref()
            .map(|build| self.under(&[CACHE_SPACE, build]))
            .transpose()
    }

    /// The manifest `digest` held in `repository` with everything it
    /// names, each asked after by a `HEAD` first and only what the
    /// repository lacks written: each blob mounted from the declared
    /// repository, each manifest an index names held the same way, then
    /// the manifest put by its digest in its own bytes read from the
    /// declared repository, since a registry takes a manifest only where
    /// what it names is held. Attestations are not held there: the seam
    /// reads a digest's claims in the declared repository (8, rule
    /// core.5; design D8).
    fn hold(&self, repository: &str, digest: &Digest) -> Answer<()> {
        if repository == self.path || self.manifest_head(repository, digest.as_str())?.is_some() {
            return Ok(());
        }
        let Some((bytes, typed)) = self.manifest(&self.path, digest)? else {
            return Err(Not::absent(&self.source, digest.to_string()));
        };
        let unreadable = |words: String| {
            Not::unreadable(
                &self.source,
                self.url(&format!("manifests/{digest}")),
                Some(words),
            )
        };
        for (manifest, named) in oci::named(&bytes).map_err(unreadable)? {
            if manifest {
                self.hold(repository, &named)?;
            } else {
                self.mount(repository, &named, &self.path)?;
            }
        }
        let media_type = typed.media_type.unwrap_or_else(|| IMAGE_MANIFEST.into());
        self.put_manifest(repository, digest.as_str(), &media_type, &bytes)?;
        Ok(())
    }

    /// The blob `digest` held in `repository`: nothing where it is held
    /// in `from` and `from` is `repository`, or a `HEAD` finds it there;
    /// mounted from `from` otherwise, and, where the registry declines the
    /// mount, its bytes read from `from` and uploaded under their digest
    /// through the upload the declined mount opened (rule core.5; design
    /// D8, D9).
    fn mount(&self, repository: &str, digest: &Digest, from: &str) -> Answer<()> {
        if repository == from || self.blob_size(repository, digest)?.is_some() {
            return Ok(());
        }
        let url = self.url_in(
            repository,
            &format!(
                "blobs/uploads/?mount={}&from={}",
                encoded(digest.as_str()),
                encoded(from)
            ),
        );
        let answered = self.send(repository, Method::POST, &url, &[], Some(&[]))?;
        if matches!(answered.status, StatusCode::CREATED | StatusCode::ACCEPTED) {
            self.mounts.borrow_mut().push((
                repository.to_string(),
                answered.status == StatusCode::CREATED,
            ));
        }
        match answered.status {
            StatusCode::CREATED => Ok(()),
            StatusCode::ACCEPTED => {
                let Some(bytes) = self.blob(from, digest)? else {
                    return Err(Not::absent(&self.source, digest.to_string()));
                };
                let location = answered.header("location").ok_or_else(|| {
                    Not::unreadable(
                        &self.source,
                        &url,
                        Some("the registry declined the mount and answered no Location".into()),
                    )
                })?;
                self.upload_to(repository, location, digest, &bytes)
            }
            _ => Err(self.other_put(repository, &url, &answered)),
        }
    }

    /// The manifest `place`'s tag is put at for `to`, with what it names
    /// held in `place`'s repository: `to`'s own where `to` is a manifest,
    /// held as [`Registry::hold`] holds it; and where `to` is bytes, the
    /// cache manifest of them, the bytes held as [`Registry::mount`]
    /// holds them (design D8, D14). `to` is asked of the declared
    /// repository and then the cache repository, as a read is, and of the
    /// cache repository alone by a push's store, which asks nothing of the
    /// declared one its credential may not read (135; design D9). Absent
    /// where `to` is neither.
    fn named_at(&self, place: &Place, to: &Digest) -> Answer<(String, Vec<u8>)> {
        let asked = if self.push && place.space == Space::Cache {
            vec![place.repository.clone()]
        } else {
            let mut asked = vec![self.path.clone()];
            asked.extend(self.cache_repository()?);
            asked
        };
        for repository in &asked {
            let Some((bytes, typed)) = self.manifest(repository, to)? else {
                continue;
            };
            if let Ok(Some(layer)) = oci::cached(&bytes) {
                return Err(Not::refused(
                    &self.source,
                    Why::Said,
                    Some(format!(
                        "{to} is the cache manifest of {layer}, and a name names the bytes, \
                         {layer}"
                    )),
                ));
            }
            self.hold(&place.repository, to)?;
            let media_type = typed.media_type.unwrap_or_else(|| IMAGE_MANIFEST.into());
            return Ok((media_type, bytes));
        }
        for repository in &asked {
            if let Some(size) = self.blob_size(repository, to)? {
                self.mount(&place.repository, to, repository)?;
                return Ok((
                    IMAGE_MANIFEST.to_string(),
                    self.cache(&place.repository, to, size)?,
                ));
            }
        }
        Err(Not::absent(&self.source, to.to_string()))
    }

    /// What `digest` holds in `repository`, read as [`Store::read`]
    /// answers it; none where the repository holds no such digest.
    fn read_in(&self, repository: &str, digest: &Digest) -> Answer<Option<Held>> {
        let Some((bytes, typed)) = self.manifest(repository, digest)? else {
            return Ok(self.blob(repository, digest)?.map(Held::Bytes));
        };
        if !typed
            .artifact_type
            .as_deref()
            .is_some_and(oci::is_tailraces)
        {
            return Ok(Some(Held::Bytes(bytes)));
        }
        let none = |digest: &Digest, words: String| {
            Not::unreadable(
                &self.source,
                self.url_in(repository, &format!("manifests/{digest}")),
                Some(words),
            )
        };
        let media_type = typed.media_type.unwrap_or_default();
        let manifest = match oci::read(&media_type, &bytes).map_err(|w| none(digest, w))? {
            oci::Read::Index(index) => Manifest::Index(index),
            oci::Read::Artifact { config, layers } => {
                let Some(meta) = self.blob(repository, &config)? else {
                    return Err(none(digest, format!("its config {config} is not held")));
                };
                Manifest::Artifact {
                    meta: oci::meta(&meta).map_err(|w| none(&config, w))?,
                    layers,
                }
            }
        };
        Ok(Some(Held::Manifest(manifest)))
    }

    fn read_back(&self, tag: &str) {
        if let Some(hook) = &self.hook {
            hook(tag);
        }
    }

    /// Where `tag`, put at `to` from `former`, is found once the registry
    /// shows the put: read back, and read again at growing intervals
    /// while it answers the former place, for at most the bound. A third
    /// digest is answered at once, and the former place once the bound
    /// has passed; the put is never repeated (C.1, 109, ADR 0015, rule
    /// core.6).
    fn shown(
        &self,
        repository: &str,
        tag: &str,
        former: Option<&Digest>,
        to: &Digest,
    ) -> Answer<Option<Digest>> {
        self.read_back(tag);
        let deadline = Instant::now() + self.bound;
        let mut wait = FIRST_WAIT;
        loop {
            let found = self.tag_at(repository, tag)?;
            let left = deadline.saturating_duration_since(Instant::now());
            if found.as_ref() == Some(to) || found.as_ref() != former || left.is_zero() {
                return Ok(found);
            }
            thread::sleep(wait.min(left));
            wait *= 2;
        }
    }

    /// The referrers index the referrers tag `tag` holds; none when the
    /// tag names nothing, and refused naming it when it names no image
    /// index (design D6).
    fn referrers_tag(&self, tag: &str) -> Answer<Option<Referrers>> {
        let url = self.url(&format!("manifests/{tag}"));
        let answered = self.send(
            &self.path,
            Method::GET,
            &url,
            &[("accept", IMAGE_INDEX)],
            None,
        )?;
        match answered.status {
            StatusCode::OK => Referrers::parse(&answered.body).map(Some).map_err(|words| {
                Not::refused(
                    &self.source,
                    Why::Said,
                    Some(format!(
                        "the referrers tag {tag} holds no image index: {words}"
                    )),
                )
            }),
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(self.other(&url, &answered)),
        }
    }

    /// The referrers of `subject` of the envelope's type, as the
    /// referrers API answers them, every page of it; none where the
    /// registry answers no referrers API (design D5, D6).
    fn referrers_api(&self, subject: &Digest) -> Answer<Option<Vec<Descriptor>>> {
        let mut url = self.url(&format!(
            "referrers/{subject}?artifactType={}",
            encoded(ENVELOPE)
        ));
        let mut listed: Vec<Descriptor> = Vec::new();
        let mut first = true;
        loop {
            let answered = self.send(
                &self.path,
                Method::GET,
                &url,
                &[("accept", IMAGE_INDEX)],
                None,
            )?;
            match answered.status {
                StatusCode::OK => {}
                StatusCode::NOT_FOUND if first => return Ok(None),
                _ => return Err(self.other(&url, &answered)),
            }
            let page = Referrers::parse(&answered.body)
                .map_err(|words| Not::unreadable(&self.source, &url, Some(words)))?;
            for descriptor in page.envelopes() {
                if !listed.iter().any(|d| d.digest == descriptor.digest) {
                    listed.push(descriptor);
                }
            }
            match answered.header("link").and_then(next) {
                Some(link) => url = self.absolute(&link),
                None => return Ok(Some(listed)),
            }
            first = false;
        }
    }

    /// `referrer` listed in `subject`'s referrers tag, once, every entry
    /// already there kept, and the tag read back: true when it was added
    /// now. An entry missing on the read back is another writer's push
    /// over it, answered unreadable naming the tag and the referrer, and
    /// nothing is pushed again (design D6, rule core.6).
    fn tag_referrer(&self, subject: &Digest, referrer: &Descriptor) -> Answer<bool> {
        let tag = tag::referrers(subject.hex());
        let mut referrers = self.referrers_tag(&tag)?.unwrap_or_else(Referrers::empty);
        if !referrers.add(referrer) {
            return Ok(false);
        }
        self.put_manifest(&self.path, &tag, IMAGE_INDEX, &referrers.bytes())?;
        self.read_back(&tag);
        let back = self.referrers_tag(&tag)?;
        if !back.is_some_and(|r| r.lists(&referrer.digest)) {
            return Err(Not::unreadable(
                &self.source,
                format!("the referrers tag {tag}"),
                Some(format!(
                    "the referrers tag {tag} no longer lists {} once it was pushed: another \
                     writer pushed over it",
                    referrer.digest
                )),
            ));
        }
        Ok(true)
    }

    /// The envelope the referrer `referrer` holds, byte for byte (69).
    fn envelope(&self, referrer: &Descriptor) -> Answer<Envelope> {
        let at = |digest: &Digest, words: String| {
            Not::unreadable(
                &self.source,
                self.url(&format!("manifests/{digest}")),
                Some(words),
            )
        };
        let Some((bytes, _)) = self.manifest(&self.path, &referrer.digest)? else {
            return Err(at(
                &referrer.digest,
                format!(
                    "the referrers list names {}, which the registry does not hold",
                    referrer.digest
                ),
            ));
        };
        let layer = oci::referred(&bytes).map_err(|words| at(&referrer.digest, words))?;
        let Some(bytes) = self.blob(&self.path, &layer.digest)? else {
            return Err(Not::unreadable(
                &self.source,
                self.url(&format!("blobs/{}", layer.digest)),
                Some(format!(
                    "the referrer {} names {}, which the registry does not hold",
                    referrer.digest, layer.digest
                )),
            ));
        };
        serde_json::from_slice(&bytes).map_err(|e| {
            Not::unreadable(
                &self.source,
                self.url(&format!("blobs/{}", layer.digest)),
                Some(e.to_string()),
            )
        })
    }
}

/// The URL a `Link` header names as the next page.
fn next(link: &str) -> Option<String> {
    link.split(',').find_map(|part| {
        let (target, params) = part.split_once(';')?;
        params
            .split(';')
            .any(|p| p.trim().replace(' ', "") == "rel=\"next\"")
            .then(|| {
                target
                    .trim()
                    .trim_start_matches('<')
                    .trim_end_matches('>')
                    .to_string()
            })
    })
}

impl Store for Registry {
    /// The blob asked after first, and uploaded only when it is not held,
    /// in the declared repository, or, for a store bound for a push, its
    /// cache repository (5, 66, 135, design D5, D9).
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        match self.cache_repository()? {
            Some(cache) if self.push => self.put_bytes_in(&cache, bytes),
            _ => self.put_bytes_in(&self.path, bytes),
        }
    }

    /// The manifest the layout writes, each digest it names asked after
    /// first, absent naming the first not held; the config put, and the
    /// manifest put by its digest alone unless it is held (1, 5, 67).
    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        let size = |digest: &Digest| -> Answer<u64> {
            let held = match manifest {
                Manifest::Artifact { .. } => self.blob_size(&self.path, digest)?,
                Manifest::Index(_) => self
                    .manifest_head(&self.path, digest.as_str())?
                    .map(|(_, s, _)| s),
            };
            held.ok_or_else(|| Not::absent(&self.source, digest.to_string()))
        };
        let written = oci::write(manifest, &self.source, &size)?;
        let digest = Digest::of(&written.bytes);
        let held_already = self.manifest_head(&self.path, digest.as_str())?.is_some();
        if !held_already {
            if let Some(config) = &written.config {
                self.put_bytes_in(&self.path, config)?;
            }
            self.put_manifest(
                &self.path,
                digest.as_str(),
                written.media_type,
                &written.bytes,
            )?;
        }
        // C.1: a declared component's index is held in its own
        // repository too, from the publish that puts it (design D10).
        if let Manifest::Index(index) = manifest {
            if index.component().as_str() != REPOSITORY_COMPONENT {
                let repository = self.under(&[index.component().as_str()])?;
                self.hold(&repository, &digest)?;
            }
        }
        Ok(Put {
            digest,
            held_already,
        })
    }

    /// The envelope pushed as a referrer of `subject`, and listed in its
    /// referrers tag where the registry answers the push with no
    /// `OCI-Subject`; absent when `subject` is no manifest held, with
    /// nothing written (8, 68, design D5, D6).
    fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
        let Some((media_type, size, _)) = self.manifest_head(&self.path, subject.as_str())? else {
            return Err(Not::absent(&self.source, subject.to_string()));
        };
        let referrer = oci::referrer(
            Descriptor {
                media_type,
                digest: subject.clone(),
                size,
                artifact_type: None,
                annotations: Default::default(),
            },
            attestation,
        );
        let descriptor = referrer.descriptor();
        let digest = attestation.digest();
        if self
            .manifest_head(&self.path, descriptor.digest.as_str())?
            .is_some()
        {
            let held_already = match self.referrers_api(subject)? {
                Some(_) => true,
                None => !self.tag_referrer(subject, &descriptor)?,
            };
            return Ok(Put {
                digest,
                held_already,
            });
        }
        self.put_bytes(&referrer.config)?;
        self.put_bytes(&referrer.layer)?;
        let put = self.put_manifest(
            &self.path,
            descriptor.digest.as_str(),
            IMAGE_MANIFEST,
            &referrer.bytes,
        )?;
        if put.header("oci-subject").is_none() {
            self.tag_referrer(subject, &descriptor)?;
        }
        Ok(Put {
            digest,
            held_already: false,
        })
    }

    /// Every envelope a referrer of the envelope's type holds for
    /// `digest`, from the referrers API or, where the registry answers
    /// none, the referrers tag; another tool's referrers are never
    /// answered, and bytes, which attach never makes a subject, have none
    /// (69, design D5, D6).
    fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>> {
        if self.manifest_head(&self.path, digest.as_str())?.is_none() {
            return match self.blob_size(&self.path, digest)? {
                Some(_) => Ok(Vec::new()),
                None => Err(Not::absent(&self.source, digest.to_string())),
            };
        }
        let listed = match self.referrers_api(digest)? {
            Some(listed) => listed,
            None => self
                .referrers_tag(&tag::referrers(digest.hex()))?
                .map(|r| r.envelopes())
                .unwrap_or_default(),
        };
        listed.iter().map(|r| self.envelope(r)).collect()
    }

    /// The name's tag read first and moved only from `expected`, by that
    /// read's best effort, the manifest `to` names put under it, or the
    /// cache manifest of the bytes it names, and the tag read back until
    /// the registry shows the put: refused naming where it is otherwise,
    /// before the put or after it, the place it was moved from once the
    /// bound has passed, and nothing is retried (70, 109, C.1, design D7,
    /// D14, ADR 0015, rule core.6).
    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
        let place = self.place(name)?;
        let (repository, tag) = (&place.repository, &place.tag);
        let at = self.tag_at(repository, tag)?;
        if at.as_ref() != expected {
            return Err(Not::refused(&self.source, Why::NameAt { at }, None));
        }
        let (media_type, bytes) = self.named_at(&place, to)?;
        self.put_manifest(repository, tag, &media_type, &bytes)?;
        let found = self.shown(repository, tag, at.as_ref(), to)?;
        if found.as_ref() == Some(to) {
            return Ok(());
        }
        let said =
            (found == at).then(|| {
                format!(
                "the registry still answered {tag} at {}, where it was before {to} was put under \
                 it, {:?} after the put, and it was not put again",
                found.as_ref().map_or_else(|| "nothing".into(), Digest::to_string),
                self.bound
            )
            });
        Err(Not::refused(&self.source, Why::NameAt { at: found }, said))
    }

    /// The digest the name's tag names, the bytes' at a cache manifest
    /// (71, design D14).
    fn resolve(&self, name: &Name) -> Answer<Digest> {
        let place = self.place(name)?;
        self.tag_at(&place.repository, &place.tag)?
            .ok_or_else(|| Not::absent(&self.source, format!("name {name}")))
    }

    /// The manifest held under `digest`, read in its form when it is
    /// tailrace's, an artifact's metadata from its config; any other
    /// manifest's bytes, or a blob's; asked of the declared repository and,
    /// where it holds no such digest, the bound build profile's cache
    /// repository, so bytes a push left there are read with no map of
    /// digests kept (72, 135, rule core.20; design D5, D9).
    fn read(&self, digest: &Digest) -> Answer<Held> {
        let mut repositories = vec![self.path.clone()];
        repositories.extend(self.cache_repository()?);
        for repository in &repositories {
            if let Some(held) = self.read_in(repository, digest)? {
                return Ok(held);
            }
        }
        Err(Not::absent(&self.source, digest.to_string()))
    }

    /// 138: the versions of `name` of `ecosystem`, each tag of the
    /// name's repository read back, every page of its tag list followed;
    /// the empty list where the registry holds no such repository or it
    /// holds no tag, and unreadable naming the repository and a tag that
    /// does not read back as one the store wrote, never leaving it out
    /// (rule core.19; design D11).
    fn versions(&self, ecosystem: &Ecosystem, name: &EcosystemName) -> Answer<Vec<Version>> {
        #[derive(Deserialize)]
        struct Tags {
            #[serde(default)]
            tags: Option<Vec<String>>,
        }
        let repository = self.under(&[ARTIFACT_SPACE, ecosystem.as_str(), name.as_str()])?;
        let mut url = self.url_in(&repository, "tags/list");
        let mut versions = Vec::new();
        loop {
            let answered = self.send(&repository, Method::GET, &url, &[], None)?;
            match answered.status {
                StatusCode::OK => {}
                StatusCode::NOT_FOUND => return Ok(versions),
                _ => return Err(self.other(&url, &answered)),
            }
            let page: Tags = serde_json::from_slice(&answered.body)
                .map_err(|e| Not::unreadable(&self.source, &url, Some(e.to_string())))?;
            for written in page.tags.unwrap_or_default() {
                let Some(version) = tag::read(&written) else {
                    return Err(Not::unreadable(
                        &self.source,
                        format!("the tag {written} of {}/{repository}", self.host),
                        Some(format!(
                            "{}/{repository} holds the tag {written}, which names no version \
                             of {ecosystem} {name} as this store writes one",
                            self.host
                        )),
                    ));
                };
                versions.push(Version::new(version));
            }
            match answered.header("link").and_then(next) {
                Some(link) => url = self.absolute(&link),
                None => return Ok(versions),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tailrace_seams::{
        artifact_name, Artifact, ArtifactMeta, ComponentName, IndexMeta, Signature, SignerId,
        IN_TOTO,
    };

    use super::stub::{Behaviour, Stub};
    use super::*;

    /// What a version a store holds is: its index, its artifacts, and
    /// the bytes and configs they name.
    struct Offered {
        index: Digest,
        artifacts: Vec<Digest>,
        blobs: Vec<Digest>,
    }

    fn meta(ecosystem: &str, name: &str) -> ArtifactMeta {
        ArtifactMeta::new(
            Ecosystem::new(ecosystem),
            EcosystemName::new(name),
            Version::new("1.3.0"),
        )
        .unwrap()
    }

    /// A version of `component` with an artifact of each `(ecosystem,
    /// name)`, put into `store` with an attestation on its index.
    fn published(store: &Registry, component: &str, names: &[(&str, &str)]) -> Offered {
        let mut artifacts = Vec::new();
        let mut blobs = Vec::new();
        for (ecosystem, name) in names {
            let layer = store
                .put_bytes(format!("{ecosystem} {name} {component}").as_bytes())
                .unwrap()
                .digest;
            let manifest = Manifest::Artifact {
                meta: meta(ecosystem, name),
                layers: vec![layer.clone()],
            };
            let digest = store.put_artifact(&manifest).unwrap().digest;
            blobs.push(Digest::of(&tailrace_seams::canonical(&meta(
                ecosystem, name,
            ))));
            blobs.push(layer);
            artifacts.push(Artifact {
                digest,
                meta: meta(ecosystem, name),
            });
        }
        let index = IndexMeta::new(
            ComponentName::new(component),
            Version::new("1.3.0"),
            artifacts.clone(),
        )
        .unwrap();
        let index = store.put_artifact(&Manifest::Index(index)).unwrap().digest;
        let envelope = Envelope::signed(
            IN_TOTO,
            b"a claim".to_vec(),
            Signature {
                signer: SignerId::new("host"),
                keyid: String::new(),
                sig: vec![1],
            },
        );
        store.attach(&index, &envelope).unwrap();
        Offered {
            index,
            artifacts: artifacts.into_iter().map(|a| a.digest).collect(),
            blobs,
        }
    }

    fn named(ecosystem: &str, name: &str, version: &str) -> Name {
        artifact_name(
            &Ecosystem::new(ecosystem),
            &EcosystemName::new(name),
            &Version::new(version),
        )
    }

    #[test]
    fn an_artifacts_name_is_a_tag_of_its_own_repository_holding_its_index() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        let version = published(&store, "web", &[("eco", "site"), ("eco", "cli")]);
        let name = named("eco", "site", "1.3.0");
        store.name(&name, &version.index, None).unwrap();
        assert_eq!(store.resolve(&name), Ok(version.index.clone()));

        let held = stub.held("acme/store/artifact/eco/site").unwrap();
        assert_eq!(held.tags.get("1.3.0"), Some(&version.index));
        for digest in std::iter::once(&version.index).chain(&version.artifacts) {
            assert!(held.manifests.contains_key(digest), "{digest} not held");
        }
        for blob in &version.blobs {
            assert!(held.blobs.contains_key(blob), "{blob} not held");
        }
        assert_eq!(
            stub.repositories(),
            [
                "acme/store",
                "acme/store/artifact/eco/site",
                "acme/store/web"
            ]
        );
        let declared = stub.held("acme/store").unwrap();
        assert!(
            declared.tags.keys().all(|t| t.starts_with("sha256-")),
            "{:?}",
            declared.tags
        );
        assert_eq!(
            held.manifests.len(),
            3,
            "an attestation was held in the artifact's repository"
        );
        assert_eq!(store.claims_of(&version.index).unwrap().len(), 1);
    }

    /// The writes `stub` was asked since its request `since`, as method
    /// and path without the query.
    fn writes(stub: &Stub, since: usize) -> Vec<String> {
        stub.asked()[since..]
            .iter()
            .filter(|a| a.method == "PUT" || a.method == "POST")
            .map(|a| format!("{} {}", a.method, a.path.split('?').next().unwrap()))
            .collect()
    }

    #[test]
    fn an_entry_named_from_bytes_the_declared_repository_holds_is_held_in_the_cache_repository() {
        for declines_mounts in [false, true] {
            let stub = Stub::start(
                Behaviour {
                    declines_mounts,
                    ..Behaviour::default()
                },
                &[],
            );
            let store = Registry::new(&stub.repository("acme/store"))
                .unwrap()
                .bound_for("moon", false);
            let bytes = store.put_bytes(b"a cache entry").unwrap().digest;
            let before = stub.asked().len();
            let entry = Name::new("cache/b7e1");
            store.name(&entry, &bytes, None).unwrap();
            assert_eq!(store.resolve(&entry), Ok(bytes.clone()));

            let cache = "/v2/acme/store/cache/moon";
            let written = writes(&stub, before);
            assert!(
                written.iter().all(|w| w.contains(cache)),
                "{declines_mounts}: {written:#?}"
            );
            let tag = written
                .iter()
                .position(|w| w.ends_with("/manifests/b7e1"))
                .unwrap();
            let manifest = written
                .iter()
                .position(|w| w.contains("/manifests/sha256:"))
                .unwrap();
            let blob = written
                .iter()
                .position(|w| w.contains("/blobs/uploads/"))
                .unwrap();
            assert!(blob < manifest && manifest < tag, "{written:#?}");
            let held = stub.held("acme/store/cache/moon").unwrap();
            assert_eq!(
                held.blobs.get(&bytes).map(|b| Digest::of(b)),
                Some(bytes.clone())
            );
            assert!(held.tags.contains_key("b7e1"));
        }
    }

    #[test]
    fn a_pushs_cache_entry_is_held_in_the_cache_repository_alone() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let push = Registry::new(&stub.repository("acme/store"))
            .unwrap()
            .bound_for("moon", true);
        let bytes = push.put_bytes(b"a cache entry").unwrap().digest;
        let entry = Name::new("cache/b7e1");
        push.name(&entry, &bytes, None).unwrap();
        assert_eq!(push.resolve(&entry), Ok(bytes.clone()));
        assert!(
            stub.asked()
                .iter()
                .all(|a| a.repository() == Some("acme/store/cache/moon")),
            "the declared repository was asked: {:#?}",
            stub.asked()
        );
        assert_eq!(stub.repositories(), ["acme/store/cache/moon"]);
        let held = stub.held("acme/store/cache/moon").unwrap();
        assert!(held.blobs.contains_key(&bytes));
        let at = held.tags["b7e1"].clone();
        assert_eq!(oci::cached(&held.manifests[&at].1), Ok(Some(bytes.clone())));

        let sync = Registry::new(&stub.repository("acme/store"))
            .unwrap()
            .bound_for("moon", false);
        let resolved = sync.resolve(&entry).unwrap();
        assert_eq!(
            sync.read(&resolved),
            Ok(Held::Bytes(b"a cache entry".to_vec()))
        );
    }

    #[test]
    fn a_store_bound_for_no_build_profile_refuses_a_cache_entrys_name() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        let bytes = store.put_bytes(b"a cache entry").unwrap().digest;
        let before = stub.asked().len();
        match store.name(&Name::new("cache/b7e1"), &bytes, None) {
            Err(Not::Refused(r)) => {
                assert!(r.said.unwrap().contains("cache/b7e1"));
            }
            other => panic!("a cache entry with no build profile answered {other:?}"),
        }
        assert_eq!(stub.asked().len(), before);
    }

    #[test]
    fn a_declared_components_version_is_a_tag_of_its_own_repository() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        let version = published(&store, "lib", &[("eco", "lib")]);
        let name = Name::new("lib:lib-v1.2.0");
        store.name(&name, &version.index, None).unwrap();
        assert_eq!(store.resolve(&name), Ok(version.index.clone()));
        let held = stub.held("acme/store/lib").unwrap();
        assert_eq!(held.tags.get("lib-v1.2.0"), Some(&version.index));
        let declared = stub.held("acme/store").unwrap();
        assert!(!declared.tags.contains_key("lib-v1.2.0"));
        assert!(!declared.tags.contains_key("lib_3alib-v1.2.0"));
    }

    #[test]
    fn two_components_channels_of_one_name_are_tags_of_their_own_repositories() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        let lib = published(&store, "lib", &[("eco", "lib")]);
        let app = published(&store, "app", &[("eco", "app")]);
        let (libs, apps) = (Name::new("channel/lib/beta"), Name::new("channel/app/beta"));
        store.name(&libs, &lib.index, None).unwrap();
        store.name(&apps, &app.index, None).unwrap();
        assert_eq!(store.resolve(&libs), Ok(lib.index.clone()));
        assert_eq!(store.resolve(&apps), Ok(app.index.clone()));
        assert_eq!(
            stub.held("acme/store/lib").unwrap().tags.get("beta"),
            Some(&lib.index)
        );
        assert_eq!(
            stub.held("acme/store/app").unwrap().tags.get("beta"),
            Some(&app.index)
        );
        let declared = stub.held("acme/store").unwrap();
        assert!(
            declared.tags.keys().all(|t| t.starts_with("sha256-")),
            "{:?}",
            declared.tags
        );
    }

    #[test]
    fn a_repository_declaring_no_component_keeps_its_version_and_channel_in_the_declared_one() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        let version = published(&store, REPOSITORY_COMPONENT, &[("eco", "site")]);
        for name in ["v1.2.0", "channel/stable"] {
            store.name(&Name::new(name), &version.index, None).unwrap();
        }
        let declared = stub.held("acme/store").unwrap();
        assert_eq!(declared.tags.get("v1.2.0"), Some(&version.index));
        assert_eq!(declared.tags.get("channel_2fstable"), Some(&version.index));
        assert_eq!(stub.repositories(), ["acme/store"]);
    }

    #[test]
    fn a_channel_moved_from_where_it_is_not_is_refused_naming_where_it_is_in_its_repository() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        let [one, two, three] =
            ["one", "two", "three"].map(|n| published(&store, "lib", &[("eco", n)]));
        let beta = Name::new("channel/lib/beta");
        store.name(&beta, &one.index, None).unwrap();
        match store.name(&beta, &two.index, Some(&three.index)) {
            Err(Not::Refused(r)) => assert_eq!(
                r.why,
                Why::NameAt {
                    at: Some(one.index.clone())
                }
            ),
            other => panic!("a move from a third index answered {other:?}"),
        }
        assert_eq!(store.resolve(&beta), Ok(one.index));
    }

    #[test]
    fn a_declared_components_index_is_held_in_its_repository_untagged_from_the_put() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        let version = published(&store, "lib", &[("eco", "lib"), ("eco", "lib-docs")]);
        let held = stub.held("acme/store/lib").unwrap();
        for digest in std::iter::once(&version.index).chain(&version.artifacts) {
            assert!(held.manifests.contains_key(digest), "{digest} not held");
        }
        for blob in &version.blobs {
            assert!(held.blobs.contains_key(blob), "{blob} not held");
        }
        assert!(held.tags.is_empty(), "{:?}", held.tags);
        assert_eq!(held.manifests.len(), 3, "an attestation was held in lib");
        let declared = stub.held("acme/store").unwrap();
        assert!(declared.manifests.contains_key(&version.index));
        assert_eq!(store.claims_of(&version.index).unwrap().len(), 1);
    }

    #[test]
    fn an_index_its_components_repository_lacks_is_held_there_before_a_channels_tag() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        // An index the component's repository does not hold, as one put
        // before indexes were held there is.
        let version = published(&store, REPOSITORY_COMPONENT, &[("eco", "lib")]);
        assert_eq!(stub.held("acme/store/lib").map(|h| h.manifests.len()), None);
        let before = stub.asked().len();
        store
            .name(&Name::new("channel/lib/beta"), &version.index, None)
            .unwrap();
        let written = writes(&stub, before);
        let tag = written
            .iter()
            .position(|w| w.ends_with("/v2/acme/store/lib/manifests/beta"))
            .unwrap();
        let index = written
            .iter()
            .position(|w| w.ends_with(&format!("/v2/acme/store/lib/manifests/{}", version.index)))
            .unwrap();
        assert!(index < tag, "{written:#?}");
        let held = stub.held("acme/store/lib").unwrap();
        assert!(held.manifests.contains_key(&version.artifacts[0]));
    }

    /// A refusal's words, once it carries the registry's words for a
    /// repository it does not hold and names `repository` of `stub`.
    fn refused_naming(stub: &Stub, answer: Answer<()>, repository: &str) {
        match answer {
            Err(Not::Refused(r)) => {
                let said = r.said.unwrap();
                assert!(said.contains(super::stub::NAME_UNKNOWN), "{said}");
                assert!(said.contains(&stub.repository(repository)), "{said}");
            }
            other => panic!("a write into {repository} answered {other:?}"),
        }
    }

    /// The writes `stub` was asked of `repository` since its request
    /// `since`.
    fn writes_of(stub: &Stub, since: usize, repository: &str) -> usize {
        stub.asked()[since..]
            .iter()
            .filter(|a| {
                (a.method == "PUT" || a.method == "POST") && a.repository() == Some(repository)
            })
            .count()
    }

    #[test]
    fn a_declared_repository_that_is_not_there_is_refused_naming_it() {
        let stub = Stub::start(
            Behaviour {
                makes_on_push: false,
                ..Behaviour::default()
            },
            &[],
        );
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        refused_naming(&stub, store.put_bytes(b"site").map(drop), "acme/store");
        assert_eq!(stub.repositories(), Vec::<String>::new());
    }

    #[test]
    fn an_artifact_names_repository_missing_is_refused_naming_it_and_no_tag_put() {
        let stub = Stub::start(
            Behaviour {
                makes_on_push: false,
                ..Behaviour::default()
            },
            &["acme/store"],
        );
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        let version = published(&store, REPOSITORY_COMPONENT, &[("eco", "site")]);
        let before = stub.asked().len();
        let repository = "acme/store/artifact/eco/site";
        refused_naming(
            &stub,
            store.name(&named("eco", "site", "1.3.0"), &version.index, None),
            repository,
        );
        assert_eq!(writes_of(&stub, before, repository), 1, "asked again");
        assert!(!stub
            .asked()
            .iter()
            .any(|a| a.method == "PUT" && a.path.ends_with("/manifests/1.3.0")));
    }

    #[test]
    fn a_components_repository_missing_is_refused_naming_it_and_never_asked_again() {
        let stub = Stub::start(
            Behaviour {
                makes_on_push: false,
                ..Behaviour::default()
            },
            &["acme/store"],
        );
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        let version = published(&store, REPOSITORY_COMPONENT, &[("eco", "lib")]);
        let before = stub.asked().len();
        refused_naming(
            &stub,
            store.name(&Name::new("lib:lib-v1.2.0"), &version.index, None),
            "acme/store/lib",
        );
        assert_eq!(writes_of(&stub, before, "acme/store/lib"), 1, "asked again");
        assert!(!stub
            .asked()
            .iter()
            .any(|a| a.method == "PUT" && a.path.ends_with("/manifests/lib-v1.2.0")));
    }

    #[test]
    fn a_cache_repository_missing_is_refused_naming_it_and_the_declared_one_is_not_written() {
        let stub = Stub::start(
            Behaviour {
                makes_on_push: false,
                ..Behaviour::default()
            },
            &["acme/store"],
        );
        let push = Registry::new(&stub.repository("acme/store"))
            .unwrap()
            .bound_for("moon", true);
        refused_naming(
            &stub,
            push.put_bytes(b"a cache entry").map(drop),
            "acme/store/cache/moon",
        );
        assert!(
            stub.asked()
                .iter()
                .all(|a| a.repository() == Some("acme/store/cache/moon")),
            "{:#?}",
            stub.asked()
        );
        assert_eq!(
            writes_of(&stub, 0, "acme/store/cache/moon"),
            1,
            "asked again"
        );
    }

    #[test]
    fn a_names_versions_are_its_repositorys_tags_every_page_read_back() {
        let stub = Stub::start(
            Behaviour {
                page: 2,
                ..Behaviour::default()
            },
            &[],
        );
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        let version = published(&store, REPOSITORY_COMPONENT, &[("npm", "@acme/site")]);
        for v in ["1.0.0", "1.1.0+b.1", "2.0.0-rc.1"] {
            store
                .name(&named("npm", "@acme/site", v), &version.index, None)
                .unwrap();
        }
        let mut listed = store
            .versions(&Ecosystem::new("npm"), &EcosystemName::new("@acme/site"))
            .unwrap();
        listed.sort();
        assert_eq!(
            listed,
            [
                Version::new("1.0.0"),
                Version::new("1.1.0+b.1"),
                Version::new("2.0.0-rc.1")
            ]
        );
        let pages = stub
            .asked()
            .iter()
            .filter(|a| a.path.contains("/tags/list"))
            .count();
        assert_eq!(pages, 2, "every page was not read");
        assert_eq!(
            store.versions(&Ecosystem::new("npm"), &EcosystemName::new("never-named")),
            Ok(vec![])
        );
    }

    #[test]
    fn a_name_whose_repository_holds_an_operations_word_is_held_and_refused_as_its_own() {
        for word in ["tags", "blobs", "manifests", "referrers", "uploads"] {
            let stub = Stub::start(
                Behaviour {
                    page: 1,
                    ..Behaviour::default()
                },
                &[],
            );
            let store = Registry::new(&stub.repository("acme/store")).unwrap();
            let version = published(&store, REPOSITORY_COMPONENT, &[("npm", word)]);
            let repository = format!("acme/store/artifact/npm/{word}");
            for v in ["1.0.0", "1.1.0"] {
                let name = named("npm", word, v);
                store.name(&name, &version.index, None).unwrap();
                assert_eq!(store.resolve(&name), Ok(version.index.clone()), "{word}");
            }
            let held = stub.held(&repository).unwrap();
            assert_eq!(held.tags.get("1.0.0"), Some(&version.index), "{word}");
            assert!(held.manifests.contains_key(&version.index), "{word}");
            let mut listed = store
                .versions(&Ecosystem::new("npm"), &EcosystemName::new(word))
                .unwrap();
            listed.sort();
            assert_eq!(
                listed,
                [Version::new("1.0.0"), Version::new("1.1.0")],
                "{word}"
            );

            let refusing = Stub::start(
                Behaviour {
                    makes_on_push: false,
                    ..Behaviour::default()
                },
                &["acme/store"],
            );
            let store = Registry::new(&refusing.repository("acme/store")).unwrap();
            let version = published(&store, REPOSITORY_COMPONENT, &[("npm", word)]);
            refused_naming(
                &refusing,
                store.name(&named("npm", word, "1.0.0"), &version.index, None),
                &repository,
            );
        }
    }

    #[test]
    fn a_tag_that_reads_back_as_no_version_is_unreadable_naming_it() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        stub.tag("acme/store/artifact/eco/site", "_zz", &Digest::of(b"x"));
        match store.versions(&Ecosystem::new("eco"), &EcosystemName::new("site")) {
            Err(Not::Unreadable(u)) => {
                assert!(u.what.contains("_zz"), "{}", u.what);
                assert!(
                    u.what.contains("acme/store/artifact/eco/site"),
                    "{}",
                    u.what
                );
            }
            other => panic!("a tag no version writes answered {other:?}"),
        }
    }

    #[test]
    fn a_scoped_packages_name_is_escaped_into_one_path_component() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        let first = published(&store, "web", &[("npm", "@acme/site")]);
        let second = published(&store, "other", &[("npm", "acme-site")]);
        let (scoped, plain) = (
            named("npm", "@acme/site", "1.3.0"),
            named("npm", "acme-site", "1.3.0"),
        );
        store.name(&scoped, &first.index, None).unwrap();
        store.name(&plain, &second.index, None).unwrap();
        assert_eq!(store.resolve(&scoped), Ok(first.index.clone()));
        assert_eq!(store.resolve(&plain), Ok(second.index.clone()));
        let tags = |repository: &str| stub.held(repository).unwrap().tags;
        assert_eq!(
            tags("acme/store/artifact/npm/0_40acme_2fsite").get("1.3.0"),
            Some(&first.index)
        );
        assert_eq!(
            tags("acme/store/artifact/npm/acme-site").get("1.3.0"),
            Some(&second.index)
        );
    }

    #[test]
    fn a_declined_mount_uploads_the_bytes_before_the_manifests_and_the_tag() {
        let stub = Stub::start(
            Behaviour {
                declines_mounts: true,
                ..Behaviour::default()
            },
            &[],
        );
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        let version = published(&store, "web", &[("eco", "site")]);
        let before = stub.asked().len();
        let name = named("eco", "site", "1.3.0");
        store.name(&name, &version.index, None).unwrap();
        assert_eq!(store.resolve(&name), Ok(version.index.clone()));

        let repository = "acme/store/artifact/eco/site";
        let into: Vec<bool> = store
            .mounts()
            .into_iter()
            .filter(|(r, _)| r == repository)
            .map(|(_, taken)| taken)
            .collect();
        assert_eq!(
            into,
            vec![false; version.blobs.len()],
            "a declined mount was taken"
        );
        let writes: Vec<String> = stub.asked()[before..]
            .iter()
            .filter(|a| {
                a.repository() == Some(repository) && a.method != "GET" && a.method != "HEAD"
            })
            .map(|a| format!("{} {}", a.method, a.path.split('?').next().unwrap()))
            .collect();
        let mounts = writes.iter().filter(|w| w.starts_with("POST")).count();
        assert_eq!(mounts, version.blobs.len(), "{writes:#?}");
        let first_manifest = writes
            .iter()
            .position(|w| w.contains("/manifests/"))
            .unwrap();
        assert!(
            writes[..first_manifest]
                .iter()
                .all(|w| w.contains("/blobs/uploads/")),
            "{writes:#?}"
        );
        assert_eq!(
            writes[first_manifest..],
            [
                format!("PUT /v2/{repository}/manifests/{}", version.artifacts[0]),
                format!("PUT /v2/{repository}/manifests/{}", version.index),
                format!("PUT /v2/{repository}/manifests/1.3.0"),
            ]
        );
        let held = stub.held(repository).unwrap();
        for blob in &version.blobs {
            assert_eq!(
                held.blobs.get(blob).map(|b| Digest::of(b)),
                Some(blob.clone()),
                "{blob} was not uploaded under its digest"
            );
        }
    }

    #[test]
    fn a_second_name_in_a_repository_holding_the_index_writes_the_tag_alone() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        let version = published(&store, "web", &[("eco", "site")]);
        store
            .name(&named("eco", "site", "1.3.0"), &version.index, None)
            .unwrap();
        let before = stub.asked().len();
        store
            .name(&named("eco", "site", "1.3.0-again"), &version.index, None)
            .unwrap();
        let writes: Vec<String> = stub.asked()[before..]
            .iter()
            .filter(|a| a.method == "PUT" || a.method == "POST")
            .map(|a| a.path.clone())
            .collect();
        assert_eq!(
            writes,
            ["/v2/acme/store/artifact/eco/site/manifests/1.3.0-again"]
        );
    }

    #[test]
    fn a_name_made_again_from_nothing_in_its_repository_is_refused_naming_where_it_is() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        let first = published(&store, "web", &[("eco", "site")]);
        let second = published(&store, "other", &[("eco", "site")]);
        let name = named("eco", "site", "1.3.0");
        store.name(&name, &first.index, None).unwrap();
        match store.name(&name, &second.index, None) {
            Err(Not::Refused(r)) => assert_eq!(
                r.why,
                Why::NameAt {
                    at: Some(first.index.clone())
                }
            ),
            other => panic!("the second make answered {other:?}"),
        }
        assert_eq!(store.resolve(&name), Ok(first.index));
    }

    #[test]
    fn a_name_whose_repository_runs_past_255_characters_is_refused_and_nothing_written() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = Registry::new(&stub.repository("acme/store")).unwrap();
        let version = published(&store, "web", &[("eco", "site")]);
        let before = stub.asked().len();
        let long = named("eco", &"n".repeat(240), "1.3.0");
        match store.name(&long, &version.index, None) {
            Err(Not::Refused(r)) => {
                let said = r.said.unwrap();
                assert!(said.contains(&"n".repeat(240)), "{said}");
                assert!(said.contains("characters"), "{said}");
            }
            other => panic!("a path past 255 characters answered {other:?}"),
        }
        assert_eq!(stub.asked().len(), before, "a request was sent");
    }

    fn refused(answer: Answer<Registry>) -> String {
        match answer {
            Err(Not::Refused(r)) => {
                assert_eq!(r.why, Why::Said);
                r.said.unwrap()
            }
            Err(other) => panic!("answered {other:?}, not refused"),
            Ok(r) => panic!("answered a registry at {}", r.base()),
        }
    }

    #[test]
    fn a_loopback_registry_is_spoken_to_over_http() {
        let registry = Registry::new("localhost:5000/tailrace").unwrap();
        assert_eq!(registry.base(), "http://localhost:5000/v2/tailrace/");
        assert_eq!(
            registry.source(),
            &Source::new(Seam::Store, "OCI registry localhost:5000/tailrace")
        );
        assert_eq!(registry.repository(), "localhost:5000/tailrace");
        for (text, base) in [
            ("127.0.0.1:41000/a/b", "http://127.0.0.1:41000/v2/a/b/"),
            ("[::1]:5000/tailrace", "http://[::1]:5000/v2/tailrace/"),
            ("localhost/tailrace", "http://localhost/v2/tailrace/"),
        ] {
            assert_eq!(Registry::new(text).unwrap().base(), base);
        }
    }

    #[test]
    fn any_other_registry_is_spoken_to_over_https() {
        let ecr = "123456789012.dkr.ecr.us-east-1.amazonaws.com/tailrace/store";
        assert_eq!(
            Registry::new(ecr).unwrap().base(),
            "https://123456789012.dkr.ecr.us-east-1.amazonaws.com/v2/tailrace/store/"
        );
        assert_eq!(
            Registry::new("localhost.example.com:5000/x")
                .unwrap()
                .base(),
            "https://localhost.example.com:5000/v2/x/"
        );
    }

    #[test]
    fn a_text_naming_no_host_or_no_path_is_refused_naming_it() {
        for text in [
            "tailrace",
            "localhost:5000/",
            "/tailrace",
            "",
            "a//b",
            "https://x/y",
        ] {
            let words = refused(Registry::new(text));
            assert!(words.contains(&format!("{text:?}")), "{text}: {words}");
        }
    }

    #[test]
    fn the_next_page_is_read_from_a_link_header() {
        assert_eq!(
            next(r#"</v2/tailrace/referrers/sha256:00?n=10&last=x>; rel="next""#).as_deref(),
            Some("/v2/tailrace/referrers/sha256:00?n=10&last=x")
        );
        assert_eq!(next(r#"</v2/x>; rel="prev""#), None);
    }
}
