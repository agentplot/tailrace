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

use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;
use tailrace_seams::{
    Answer, Digest, Envelope, Held, Manifest, Name, Not, Put, Seam, Source, Store, Why,
};
use ureq::http::{Method, StatusCode};

use super::oci::{
    self, Descriptor, Referrers, EMPTY_JSON, ENVELOPE, IMAGE_INDEX, IMAGE_MANIFEST, OCTETS,
};
use client::{encoded, Answered, Client};

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
    origin: String,
    base: String,
    source: Source,
    client: Client,
    hook: Option<Box<dyn Fn(&str)>>,
    bound: Duration,
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
            base: format!("{origin}/v2/{path}/"),
            origin,
            client: Client::new(host, &source),
            source,
            hook: None,
            bound: BOUND,
        })
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

    fn send(
        &self,
        method: Method,
        url: &str,
        headers: &[(&str, &str)],
        body: Option<&[u8]>,
    ) -> Answer<Answered> {
        self.client.send(method, url, headers, body)
    }

    /// What a `HEAD` the operation does not take is: what a `GET` of
    /// the same URL answers, which carries the registry's words where the
    /// `HEAD` carried none.
    fn other_head(&self, url: &str, accept: &[(&str, &str)], head: &Answered) -> Not {
        match self.send(Method::GET, url, accept, None) {
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
    fn blob_size(&self, digest: &Digest) -> Answer<Option<u64>> {
        let url = self.url(&format!("blobs/{digest}"));
        let answered = self.send(Method::HEAD, &url, &[], None)?;
        match answered.status {
            StatusCode::OK => self.length(&url, &answered).map(Some),
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(self.other_head(&url, &[], &answered)),
        }
    }

    /// The media type and size of the manifest `reference`, a digest or a
    /// tag, names; none when it names none.
    fn manifest_head(&self, reference: &str) -> Answer<Option<(String, u64, Option<Digest>)>> {
        let url = self.url(&format!("manifests/{reference}"));
        let accept = [("accept", MANIFESTS)];
        let answered = self.send(Method::HEAD, &url, &accept, None)?;
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
            _ => Err(self.other_head(&url, &accept, &answered)),
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
    fn upload(&self, digest: &Digest, bytes: &[u8]) -> Answer<()> {
        let url = self.url("blobs/uploads/");
        let opened = self.send(Method::POST, &url, &[], Some(&[]))?;
        if opened.status != StatusCode::ACCEPTED {
            return Err(self.other(&url, &opened));
        }
        let location = opened.header("location").ok_or_else(|| {
            Not::unreadable(
                &self.source,
                &url,
                Some("the registry opened an upload and answered no Location".into()),
            )
        })?;
        let location = self.absolute(location);
        let sep = if location.contains('?') { '&' } else { '?' };
        let url = format!("{location}{sep}digest={}", encoded(digest.as_str()));
        let put = self.send(Method::PUT, &url, &[("content-type", OCTETS)], Some(bytes))?;
        if !put.status.is_success() {
            return Err(self.other(&url, &put));
        }
        self.named(digest, &put)
    }

    /// `bytes` of `media_type` put as the manifest `reference`, a digest
    /// or a tag, answering what the registry answered.
    fn put_manifest(&self, reference: &str, media_type: &str, bytes: &[u8]) -> Answer<Answered> {
        let url = self.url(&format!("manifests/{reference}"));
        let put = self.send(
            Method::PUT,
            &url,
            &[("content-type", media_type)],
            Some(bytes),
        )?;
        if !put.status.is_success() {
            return Err(self.other(&url, &put));
        }
        self.named(&Digest::of(bytes), &put)?;
        Ok(put)
    }

    /// The bytes of the blob `digest`, once they are found to be its;
    /// none when it is not held. A redirect, as ECR's to its bucket, is
    /// followed, and the digest checked on what finally arrives (design
    /// D3).
    fn blob(&self, digest: &Digest) -> Answer<Option<Vec<u8>>> {
        let url = self.url(&format!("blobs/{digest}"));
        let answered = self.send(Method::GET, &url, &[], None)?;
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
    fn manifest(&self, digest: &Digest) -> Answer<Option<(Vec<u8>, Typed)>> {
        let url = self.url(&format!("manifests/{digest}"));
        let answered = self.send(Method::GET, &url, &[("accept", MANIFESTS)], None)?;
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
    fn tag_at(&self, tag: &str) -> Answer<Option<Digest>> {
        let url = self.url(&format!("manifests/{tag}"));
        let answered = self.send(Method::GET, &url, &[("accept", MANIFESTS)], None)?;
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
    fn cache(&self, digest: &Digest, size: u64) -> Answer<Vec<u8>> {
        let bytes = oci::cache(digest, size);
        let at = Digest::of(&bytes);
        if self.manifest_head(at.as_str())?.is_none() {
            self.put_bytes(EMPTY_JSON)?;
            self.put_manifest(at.as_str(), IMAGE_MANIFEST, &bytes)?;
        }
        Ok(bytes)
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
    fn shown(&self, tag: &str, former: Option<&Digest>, to: &Digest) -> Answer<Option<Digest>> {
        self.read_back(tag);
        let deadline = Instant::now() + self.bound;
        let mut wait = FIRST_WAIT;
        loop {
            let found = self.tag_at(tag)?;
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
        let answered = self.send(Method::GET, &url, &[("accept", IMAGE_INDEX)], None)?;
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
            let answered = self.send(Method::GET, &url, &[("accept", IMAGE_INDEX)], None)?;
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
        self.put_manifest(&tag, IMAGE_INDEX, &referrers.bytes())?;
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
        let Some((bytes, _)) = self.manifest(&referrer.digest)? else {
            return Err(at(
                &referrer.digest,
                format!(
                    "the referrers list names {}, which the registry does not hold",
                    referrer.digest
                ),
            ));
        };
        let layer = oci::referred(&bytes).map_err(|words| at(&referrer.digest, words))?;
        let Some(bytes) = self.blob(&layer.digest)? else {
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
    /// The blob asked after first, and uploaded only when it is not held
    /// (5, 66, design D5).
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        let digest = Digest::of(bytes);
        if self.blob_size(&digest)?.is_some() {
            return Ok(Put {
                digest,
                held_already: true,
            });
        }
        self.upload(&digest, bytes)?;
        Ok(Put {
            digest,
            held_already: false,
        })
    }

    /// The manifest the layout writes, each digest it names asked after
    /// first, absent naming the first not held; the config put, and the
    /// manifest put by its digest alone unless it is held (1, 5, 67).
    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        let size = |digest: &Digest| -> Answer<u64> {
            let held = match manifest {
                Manifest::Artifact { .. } => self.blob_size(digest)?,
                Manifest::Index(_) => self.manifest_head(digest.as_str())?.map(|(_, s, _)| s),
            };
            held.ok_or_else(|| Not::absent(&self.source, digest.to_string()))
        };
        let written = oci::write(manifest, &self.source, &size)?;
        let digest = Digest::of(&written.bytes);
        if self.manifest_head(digest.as_str())?.is_some() {
            return Ok(Put {
                digest,
                held_already: true,
            });
        }
        if let Some(config) = &written.config {
            self.put_bytes(config)?;
        }
        self.put_manifest(digest.as_str(), written.media_type, &written.bytes)?;
        Ok(Put {
            digest,
            held_already: false,
        })
    }

    /// The envelope pushed as a referrer of `subject`, and listed in its
    /// referrers tag where the registry answers the push with no
    /// `OCI-Subject`; absent when `subject` is no manifest held, with
    /// nothing written (8, 68, design D5, D6).
    fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
        let Some((media_type, size, _)) = self.manifest_head(subject.as_str())? else {
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
        if self.manifest_head(descriptor.digest.as_str())?.is_some() {
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
        let put = self.put_manifest(descriptor.digest.as_str(), IMAGE_MANIFEST, &referrer.bytes)?;
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
        if self.manifest_head(digest.as_str())?.is_none() {
            return match self.blob_size(digest)? {
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
        let tag = self.tag(name)?;
        let at = self.tag_at(&tag)?;
        if at.as_ref() != expected {
            return Err(Not::refused(&self.source, Why::NameAt { at }, None));
        }
        let (media_type, bytes) = match self.manifest(to)? {
            Some((bytes, typed)) => {
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
                let media_type = typed.media_type.unwrap_or_else(|| IMAGE_MANIFEST.into());
                (media_type, bytes)
            }
            None => match self.blob_size(to)? {
                Some(size) => (IMAGE_MANIFEST.to_string(), self.cache(to, size)?),
                None => return Err(Not::absent(&self.source, to.to_string())),
            },
        };
        self.put_manifest(&tag, &media_type, &bytes)?;
        let found = self.shown(&tag, at.as_ref(), to)?;
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
        let tag = self.tag(name)?;
        self.tag_at(&tag)?
            .ok_or_else(|| Not::absent(&self.source, format!("name {name}")))
    }

    /// The manifest held under `digest`, read in its form when it is
    /// tailrace's, an artifact's metadata from its config; any other
    /// manifest's bytes, or a blob's (72, design D5).
    fn read(&self, digest: &Digest) -> Answer<Held> {
        let Some((bytes, typed)) = self.manifest(digest)? else {
            return match self.blob(digest)? {
                Some(bytes) => Ok(Held::Bytes(bytes)),
                None => Err(Not::absent(&self.source, digest.to_string())),
            };
        };
        if !typed
            .artifact_type
            .as_deref()
            .is_some_and(oci::is_tailraces)
        {
            return Ok(Held::Bytes(bytes));
        }
        let none = |digest: &Digest, words: String| {
            Not::unreadable(
                &self.source,
                self.url(&format!("manifests/{digest}")),
                Some(words),
            )
        };
        let media_type = typed.media_type.unwrap_or_default();
        let manifest = match oci::read(&media_type, &bytes).map_err(|w| none(digest, w))? {
            oci::Read::Index(index) => Manifest::Index(index),
            oci::Read::Artifact { config, layers } => {
                let Some(meta) = self.blob(&config)? else {
                    return Err(none(digest, format!("its config {config} is not held")));
                };
                Manifest::Artifact {
                    meta: oci::meta(&meta).map_err(|w| none(&config, w))?,
                    layers,
                }
            }
        };
        Ok(Held::Manifest(manifest))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
