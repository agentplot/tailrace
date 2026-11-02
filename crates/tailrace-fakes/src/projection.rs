//! The fake projection (B.6): its ecosystem, the name each output takes,
//! whether it packs cache entries, whether it serves, the build
//! identities it holds no archive for, the pins of each checkout's
//! lockfile and the ranges its manifest declares, what the manifest of
//! each artifact, by digest, requires, and the versions a test says
//! satisfy a range. A pack is the canonical JSON of what it was given,
//! read from no disk; a render writes nothing and keeps the place and the
//! artifacts it was given; serve answers a request `<name>@<version>`
//! from the last render at the layout's place, asking the core for a
//! version it lacks (86, 115); pinned reads no checkout and requires no
//! bytes. satisfying answers, for a range a test set, those of the
//! versions asked that the test named, in the order it named them,
//! newest first; for any other range it keeps the versions the range
//! matches and orders them newest first by Semantic Versioning, so a test
//! can list versions out of order and get the newest that fits (140, 23;
//! design D7).

use std::collections::{BTreeMap, BTreeSet};
use std::slice;

use serde_json::json;
use tailrace_seams::{
    canonical, no_cache_entries, no_lockfile, no_serving, Answer, Artifact, ArtifactMeta, BuildId,
    Component, ComponentName, Contents, Digest, Ecosystem, EcosystemName, Holds, Layout, Miss, Not,
    Output, OutputName, PackAt, Packed, Pin, Pinned, Projection, Range, Rendered, Request,
    Requirement, Response, Seam, Version, Why,
};

use crate::Fake;

/// The fake projection's operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionOp {
    Pack,
    Render,
    Serve,
    Pinned,
    Requires,
    Satisfying,
}

/// The projection's ecosystem, the names outputs take in it, whether its
/// packing is the same twice, whether it packs cache entries, whether it
/// serves, the build identities it holds no archive for, the pins of each
/// checkout holding a lockfile and the ranges of each checkout's
/// manifest, what each artifact's manifest requires, by its digest, the
/// versions satisfying each range a test set, newest first, each render
/// of artifacts, each request served and each version asked of the core.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionState {
    ecosystem: Ecosystem,
    names: BTreeMap<OutputName, EcosystemName>,
    unsteady: bool,
    caches: bool,
    serves: bool,
    no_archive: BTreeSet<BuildId>,
    packs: u64,
    lockfiles: BTreeMap<String, Vec<Pinned>>,
    ranges: BTreeMap<String, Vec<Requirement>>,
    manifests: BTreeMap<Digest, Vec<Requirement>>,
    satisfied: BTreeMap<Range, Vec<Version>>,
    probes: Vec<String>,
    renders: Vec<(String, Vec<Artifact>)>,
    requests: Vec<Request>,
    misses: Vec<Pinned>,
}

impl ProjectionState {
    /// The metadata of `pin`'s artifact in the last render at `at`, none
    /// where that render holds no such version or nothing was rendered
    /// there.
    fn held(&self, at: &str, pin: &Pinned) -> Option<ArtifactMeta> {
        let (_, artifacts) = self.renders.iter().rev().find(|(place, _)| place == at)?;
        artifacts
            .iter()
            .find(|a| a.meta.name() == &pin.name && a.meta.version() == &pin.version)
            .map(|a| a.meta.clone())
    }
}

/// The version a served request's path names, `<name>/<version>`, split
/// at its last `/`, so a scoped name keeps its own; none where it names
/// none.
fn named_in(path: &str) -> Option<Pinned> {
    let (name, version) = path.rsplit_once('/')?;
    (!name.is_empty() && !version.is_empty()).then(|| Pinned {
        name: EcosystemName::new(name),
        version: Version::new(version),
    })
}

/// A response whose body is `words`, with `status`.
fn words(status: u16, words: String) -> Response {
    Response {
        status,
        content_type: "text/plain; charset=utf-8".into(),
        body: words.into_bytes(),
    }
}

/// A projection in memory.
pub type FakeProjection = Fake<ProjectionState, ProjectionOp>;

impl FakeProjection {
    /// A projection of `ecosystem` that packs cache entries, in which each
    /// output takes its own name until [`FakeProjection::named`] gives it
    /// another.
    pub fn new(ecosystem: Ecosystem) -> Self {
        Fake::with(
            Seam::Projection,
            ProjectionState {
                ecosystem,
                names: BTreeMap::new(),
                unsteady: false,
                caches: true,
                serves: true,
                no_archive: BTreeSet::new(),
                packs: 0,
                lockfiles: BTreeMap::new(),
                ranges: BTreeMap::new(),
                manifests: BTreeMap::new(),
                satisfied: BTreeMap::new(),
                probes: Vec::new(),
                renders: Vec::new(),
                requests: Vec::new(),
                misses: Vec::new(),
            },
        )
    }

    /// `output` takes `name` in the ecosystem.
    pub fn named(&self, output: &OutputName, name: EcosystemName) {
        self.state().names.insert(output.clone(), name);
    }

    /// 87: every pack from now differs from the last, and says it cannot
    /// promise the same bytes.
    pub fn unsteady(&self) {
        self.state().unsteady = true;
    }

    /// 84, 85: from now a projection that packs no cache entries, whose
    /// pack with no version and render of cache entries answer absent.
    pub fn packs_no_cache_entries(&self) {
        self.state().caches = false;
    }

    /// 86: from now a projection whose ecosystem's clients read files,
    /// whose serve answers absent naming serving for every request and
    /// asks the core nothing.
    pub fn serves_none(&self) {
        self.state().serves = false;
    }

    /// 86: every request asked of serve, in order.
    pub fn requests(&self) -> Vec<Request> {
        self.state().requests.clone()
    }

    /// 115: every version serve asked of the core, in order.
    pub fn misses(&self) -> Vec<Pinned> {
        self.state().misses.clone()
    }

    /// Whether the projection packs cache entries.
    pub fn caches(&self) -> bool {
        self.state().caches
    }

    /// 54: from now a pack with no version at `build` answers absent
    /// naming the archive it looked for, as a projection does on a host
    /// the build left no archive on.
    pub fn no_archive(&self, build: &BuildId) {
        self.state().no_archive.insert(build.clone());
    }

    /// 137: the checkout at `checkout` holds the ecosystem's lockfile,
    /// pinning `pins` in the order given; every other checkout holds
    /// none.
    pub fn lockfile(&self, checkout: &str, pins: &[Pinned]) {
        self.state()
            .lockfiles
            .insert(checkout.to_string(), pins.to_vec());
    }

    /// 137: the manifest of the checkout at `checkout` declares
    /// `ranges` for names no lockfile pins yet, answered after its pins in
    /// the order given; a checkout with ranges and no lockfile answers
    /// its ranges alone.
    pub fn ranges(&self, checkout: &str, ranges: &[Requirement]) {
        self.state()
            .ranges
            .insert(checkout.to_string(), ranges.to_vec());
    }

    /// 140, 23: `newest_first` are the versions that satisfy `range`, in
    /// the ecosystem's order newest first, whatever their spelling; a
    /// version asked and not among them satisfies it not.
    pub fn satisfied(&self, range: &Range, newest_first: &[Version]) {
        self.state()
            .satisfied
            .insert(range.clone(), newest_first.to_vec());
    }

    /// 140: the manifest of the artifact at `digest` requires
    /// `requirements`, answered in the order given; every other
    /// artifact's manifest is one the fake cannot find.
    pub fn manifest(&self, digest: &Digest, requirements: &[Requirement]) {
        self.state()
            .manifests
            .insert(digest.clone(), requirements.to_vec());
    }

    /// 84, 123: from now each pack given what it is packed at records,
    /// beside the component and the outputs it was given, what `holds`
    /// answered for each of `paths`.
    pub fn probe(&self, paths: &[&str]) {
        self.state().probes = paths.iter().map(|p| p.to_string()).collect();
    }

    /// 85: each render of artifacts answered, in order: the place it was
    /// asked at and the artifacts it was given.
    pub fn renders(&self) -> Vec<(String, Vec<Artifact>)> {
        self.state().renders.clone()
    }
}

/// 84: what a test packs one output at: a version, and a component of
/// that output alone, named by it, whose paths hold every path of the
/// repository.
pub struct Alone {
    version: Version,
    component: Component,
    output: Output,
}

impl Alone {
    pub fn new(output: &Output, version: &str) -> Alone {
        Alone {
            version: Version::new(version),
            component: Component {
                name: ComponentName::new(output.name.as_str()),
                outputs: vec![output.name.clone()],
                paths: vec!["**".into()],
                prefix: format!("{}-v", output.name),
            },
            output: output.clone(),
        }
    }

    /// What the output is packed at.
    pub fn at(&self) -> PackAt<'_> {
        PackAt {
            version: &self.version,
            component: &self.component,
            outputs: slice::from_ref(&self.output),
            holds: &every,
        }
    }
}

fn every(_: &str) -> bool {
    true
}

impl Projection for FakeProjection {
    fn pack(&self, output: &Output, at: &BuildId, version: Option<&PackAt<'_>>) -> Answer<Packed> {
        let probes = self.state().probes.clone();
        let given = json!({
            "output": output,
            "build": at,
            "version": version.map(|v| v.version),
            "component": version.map(|v| v.component),
            "outputs": version.map(|v| v.outputs),
            "holds": version.map(|v| {
                probes
                    .iter()
                    .map(|p| (p.clone(), (v.holds)(p)))
                    .collect::<BTreeMap<_, _>>()
            }),
        });
        let version = version.map(|v| v.version);
        self.ask(
            ProjectionOp::Pack,
            given,
            format!("output {}", output.name),
            |s, source| {
                let Some(version) = version else {
                    if !s.caches {
                        return Err(no_cache_entries(source));
                    }
                    if s.no_archive.contains(at) {
                        return Err(Not::absent(source, format!("the archive of {at}")));
                    }
                    s.packs += 1;
                    let pack = s.unsteady.then_some(s.packs);
                    let bytes = canonical(&json!({
                        "build": at,
                        "output": output.name,
                        "files": output.files,
                        "pack": pack,
                    }));
                    return Ok(Packed {
                        bytes,
                        meta: None,
                        same_again: !s.unsteady,
                    });
                };
                let name = s
                    .names
                    .get(&output.name)
                    .cloned()
                    .unwrap_or_else(|| EcosystemName::new(output.name.as_str()));
                let meta = ArtifactMeta::new(s.ecosystem.clone(), name, version.clone())
                    .map_err(|bad| Not::refused(source, Why::Said, Some(bad.to_string())))?;
                s.packs += 1;
                let pack = s.unsteady.then_some(s.packs);
                let bytes = canonical(&json!({
                    "ecosystem": meta.ecosystem(),
                    "name": meta.name(),
                    "version": meta.version(),
                    "output": output.name,
                    "files": output.files,
                    "pack": pack,
                }));
                Ok(Packed {
                    bytes,
                    meta: Some(meta),
                    same_again: !s.unsteady,
                })
            },
        )
    }

    fn render(&self, contents: Contents<'_>, at: &str) -> Answer<Rendered> {
        let given = json!({ "contents": contents, "at": at });
        self.ask(
            ProjectionOp::Render,
            given,
            format!("the layout at {at}"),
            |s, source| {
                let holds = match contents {
                    Contents::Artifacts(artifacts) => {
                        s.renders.push((at.to_string(), artifacts.to_vec()));
                        Holds::Versions(artifacts.iter().map(|a| a.meta.clone()).collect())
                    }
                    Contents::CacheEntries(_) if !s.caches => return Err(no_cache_entries(source)),
                    Contents::CacheEntries(entries) => {
                        Holds::BuildIds(entries.iter().map(|e| e.build.clone()).collect())
                    }
                };
                Ok(Rendered {
                    layout: Layout { at: at.to_string() },
                    holds,
                })
            },
        )
    }

    fn serve(&self, layout: &Layout, request: &Request, miss: Miss<'_>) -> Answer<Response> {
        let given = json!({ "layout": layout, "request": request });
        let asked = named_in(&request.path);
        // The core is asked with the fake let go, since the core renders
        // the version it holds into this same fake.
        let lacked = {
            let mut s = self.state();
            s.requests.push(request.clone());
            let lacked = asked
                .as_ref()
                .filter(|pin| s.serves && s.held(&layout.at, pin).is_none())
                .cloned();
            s.misses.extend(lacked.clone());
            lacked
        };
        let missed = lacked.map(|pin| miss(&pin));
        self.ask(
            ProjectionOp::Serve,
            given,
            format!("the layout at {}", layout.at),
            |s, source| {
                if !s.serves {
                    return Err(no_serving(source));
                }
                let Some(pin) = asked else {
                    return Ok(words(404, format!("{} names no version", request.path)));
                };
                match missed {
                    Some(Err(not @ Not::Absent(_))) => return Ok(words(404, not.to_string())),
                    Some(Err(not @ Not::Refused(_))) => return Ok(words(403, not.to_string())),
                    Some(Err(not)) => return Ok(words(502, not.to_string())),
                    Some(Ok(())) | None => {}
                }
                let meta = match s.held(&layout.at, &pin) {
                    Some(meta) => meta,
                    None => ArtifactMeta::new(s.ecosystem.clone(), pin.name, pin.version)
                        .map_err(|bad| Not::refused(source, Why::Said, Some(bad.to_string())))?,
                };
                Ok(Response {
                    status: 200,
                    content_type: "application/json".into(),
                    body: canonical(&meta),
                })
            },
        )
    }

    fn pinned(&self, checkout: &str) -> Answer<Vec<Pin>> {
        let given = json!({ "checkout": checkout });
        self.ask(
            ProjectionOp::Pinned,
            given,
            format!("the lockfile of {checkout}"),
            |s, source| {
                let pins = s.lockfiles.get(checkout);
                let ranges = s.ranges.get(checkout);
                if pins.is_none() && ranges.is_none() {
                    return Err(no_lockfile(source));
                }
                let pins = pins.into_iter().flatten().cloned().map(Pin::Version);
                let ranges = ranges.into_iter().flatten().cloned().map(Pin::Range);
                Ok(pins.chain(ranges).collect())
            },
        )
    }

    fn requires(&self, held: &Artifact) -> Answer<Vec<Requirement>> {
        let given = json!({ "held": held });
        let manifest = format!(
            "the manifest of {}@{}",
            held.meta.name(),
            held.meta.version()
        );
        self.ask(
            ProjectionOp::Requires,
            given,
            manifest.clone(),
            |s, source| {
                s.manifests
                    .get(&held.digest)
                    .cloned()
                    .ok_or_else(|| Not::absent(source, manifest))
            },
        )
    }

    fn satisfying(&self, range: &Range, versions: &[Version]) -> Answer<Vec<Version>> {
        let given = json!({ "range": range, "versions": versions });
        self.ask(
            ProjectionOp::Satisfying,
            given,
            format!("the versions satisfying {range}"),
            |s, source| {
                if let Some(newest_first) = s.satisfied.get(range) {
                    return Ok(newest_first
                        .iter()
                        .filter(|v| versions.contains(v))
                        .cloned()
                        .collect());
                }
                let req = semver::VersionReq::parse(range.as_str())
                    .map_err(|e| Not::refused(source, Why::Said, Some(e.to_string())))?;
                let mut kept: Vec<(semver::Version, &Version)> = versions
                    .iter()
                    .filter_map(|v| semver::Version::parse(v.as_str()).ok().map(|p| (p, v)))
                    .filter(|(p, _)| req.matches(p))
                    .collect();
                kept.sort_by(|(a, _), (b, _)| b.cmp_precedence(a));
                Ok(kept.into_iter().map(|(_, v)| v.clone()).collect())
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use tailrace_seams::{Artifact, CacheEntry, Digest};

    use super::*;
    use crate::{Fault, Times};

    fn output() -> Output {
        Output {
            name: OutputName::new("site"),
            files: vec!["dist/index.html".into()],
        }
    }

    fn b7e1() -> BuildId {
        BuildId::new("b7e1")
    }

    fn artifact_pack(projection: &FakeProjection, version: &str) -> Packed {
        projection
            .pack(
                &output(),
                &b7e1(),
                Some(&Alone::new(&output(), version).at()),
            )
            .unwrap()
    }

    fn absent_naming_cache_entries<T: std::fmt::Debug>(answer: Answer<T>) {
        match answer {
            Err(Not::Absent(a)) => assert_eq!(a.what, "cache entries"),
            other => panic!("answered {other:?}"),
        }
    }

    #[test]
    fn a_pack_is_its_ecosystems_bytes_and_metadata() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        projection.named(&OutputName::new("site"), EcosystemName::new("web-site"));
        let packed = artifact_pack(&projection, "1.0.0");
        let meta = packed
            .meta
            .clone()
            .expect("a versioned pack names metadata");
        assert_eq!(meta.ecosystem(), &Ecosystem::new("eco"));
        assert_eq!(meta.name(), &EcosystemName::new("web-site"));
        assert_eq!(meta.version(), &Version::new("1.0.0"));
        assert!(packed.same_again);
        assert_eq!(artifact_pack(&projection, "1.0.0"), packed);
        assert_ne!(artifact_pack(&projection, "1.0.1").bytes, packed.bytes);
    }

    #[test]
    fn a_versioned_pack_is_the_same_at_two_build_identities() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        let alone = Alone::new(&output(), "1.0.0");
        let at_b7e1 = projection
            .pack(&output(), &b7e1(), Some(&alone.at()))
            .unwrap();
        let at_c3d4 = projection
            .pack(&output(), &BuildId::new("c3d4"), Some(&alone.at()))
            .unwrap();
        assert_eq!(at_b7e1, at_c3d4);
    }

    #[test]
    fn an_output_takes_its_own_name_until_named() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        let meta = artifact_pack(&projection, "1.0.0").meta.unwrap();
        assert_eq!(meta.name(), &EcosystemName::new("site"));
    }

    #[test]
    fn a_pack_that_is_not_the_same_again_says_so() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        projection.unsteady();
        let first = artifact_pack(&projection, "1.0.0");
        let second = artifact_pack(&projection, "1.0.0");
        assert!(!first.same_again);
        assert!(!second.same_again);
        assert_ne!(first.bytes, second.bytes);
    }

    #[test]
    fn a_pack_with_an_empty_version_is_refused_naming_it() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        assert!(matches!(
            projection.pack(&output(), &b7e1(), Some(&Alone::new(&output(), "").at())),
            Err(Not::Refused(r)) if r.said.as_deref() == Some("the metadata names no version")
        ));
    }

    #[test]
    fn a_cache_entry_is_packed_by_its_build_identity_and_names_no_artifact() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        let first = projection.pack(&output(), &b7e1(), None).unwrap();
        let second = projection.pack(&output(), &b7e1(), None).unwrap();
        let other = projection
            .pack(&output(), &BuildId::new("c3d4"), None)
            .unwrap();
        assert_eq!(first, second);
        assert!(first.same_again);
        assert_ne!(other.bytes, first.bytes);
        for packed in [&first, &second, &other] {
            assert_eq!(packed.meta, None);
            let text = String::from_utf8(packed.bytes.clone()).unwrap();
            for named in ["eco", "version", "1.0.0"] {
                assert!(!text.contains(named), "{text} names {named}");
            }
        }
    }

    #[test]
    fn cache_entries_render_as_a_layout_of_their_build_identities() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        let entries = [
            CacheEntry {
                build: b7e1(),
                digest: Digest::of(b"b7e1"),
            },
            CacheEntry {
                build: BuildId::new("c3d4"),
                digest: Digest::of(b"c3d4"),
            },
        ];
        let rendered = projection
            .render(Contents::CacheEntries(&entries), "layouts/cache")
            .unwrap();
        assert_eq!(rendered.layout.at, "layouts/cache");
        assert_eq!(
            rendered.holds,
            Holds::BuildIds(vec![b7e1(), BuildId::new("c3d4")])
        );
    }

    #[test]
    fn scripted_to_pack_no_cache_entries_it_answers_absent_for_one() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        assert!(projection.caches());
        projection.packs_no_cache_entries();
        assert!(!projection.caches());
        absent_naming_cache_entries(projection.pack(&output(), &b7e1(), None));
        absent_naming_cache_entries(projection.render(
            Contents::CacheEntries(&[CacheEntry {
                build: b7e1(),
                digest: Digest::of(b"b7e1"),
            }]),
            "layouts/cache",
        ));
        let meta = artifact_pack(&projection, "1.0.0").meta;
        assert_eq!(
            meta.map(|m| m.version().clone()),
            Some(Version::new("1.0.0"))
        );
    }

    #[test]
    fn scripted_to_hold_no_archive_for_a_build_it_answers_absent_for_that_one() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        projection.no_archive(&b7e1());
        match projection.pack(&output(), &b7e1(), None) {
            Err(Not::Absent(a)) => {
                assert_eq!(a.source.seam, Seam::Projection);
                assert_eq!(a.what, "the archive of b7e1");
            }
            other => panic!("answered {other:?}"),
        }
        let other = projection.pack(&output(), &BuildId::new("c3d4"), None);
        assert!(other.unwrap().meta.is_none());
        assert!(artifact_pack(&projection, "1.0.0").meta.is_some());
    }

    #[test]
    fn a_render_holds_what_it_was_given() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        let meta = artifact_pack(&projection, "1.0.0").meta.unwrap();
        let artifact = Artifact {
            digest: Digest::of(b"site"),
            meta: meta.clone(),
        };
        let rendered = projection
            .render(Contents::Artifacts(&[artifact]), "layouts/eco")
            .unwrap();
        assert_eq!(rendered.layout.at, "layouts/eco");
        assert_eq!(rendered.holds, Holds::Versions(vec![meta]));
    }

    fn request(path: &str) -> Request {
        Request {
            method: "GET".into(),
            path: path.into(),
            base: "http://127.0.0.1:7420/o/r/eco".into(),
        }
    }

    #[test]
    fn a_version_held_is_served_and_one_lacked_is_asked_of_the_core() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        let meta = artifact_pack(&projection, "1.0.0").meta.unwrap();
        let artifact = Artifact {
            digest: Digest::of(b"site"),
            meta: meta.clone(),
        };
        let layout = projection
            .render(Contents::Artifacts(&[artifact]), "layouts/eco")
            .unwrap()
            .layout;
        let asked = std::cell::RefCell::new(Vec::new());
        let refusal = Not::refused(
            &tailrace_seams::Source::new(Seam::Policy, "core"),
            Why::Said,
            Some("acceptance wanting".into()),
        );
        let miss = |pin: &Pinned| {
            asked.borrow_mut().push(pin.clone());
            Err(refusal.clone())
        };
        let held = projection
            .serve(&layout, &request("site/1.0.0"), &miss)
            .unwrap();
        assert_eq!(held.status, 200);
        assert_eq!(held.body, canonical(&meta));
        assert_eq!(*asked.borrow(), [], "a held version asked the core");
        let lacked = projection
            .serve(&layout, &request("site/1.1.0"), &miss)
            .unwrap();
        assert_eq!(lacked.status, 403);
        assert_eq!(lacked.body, refusal.to_string().into_bytes());
        assert!(String::from_utf8(lacked.body)
            .unwrap()
            .contains("acceptance wanting"));
        assert_eq!(*asked.borrow(), [pin("site", "1.1.0")]);
        assert_eq!(
            projection.requests(),
            [request("site/1.0.0"), request("site/1.1.0")]
        );
        assert_eq!(projection.misses(), [pin("site", "1.1.0")]);
    }

    #[test]
    fn the_cores_answer_for_a_version_lacked_is_its_status_and_words() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        let layout = Layout {
            at: "layouts/eco".into(),
        };
        let source = tailrace_seams::Source::new(Seam::Store, "core");
        for (status, answer) in [
            (
                404,
                Err(Not::absent(&source, "name artifact/eco/@scope/pkg@2.0.0")),
            ),
            (
                502,
                Err(Not::unreadable(
                    &source,
                    "the store",
                    Some("connection refused".into()),
                )),
            ),
        ] {
            let words = answer.clone().unwrap_err().to_string();
            let response = projection
                .serve(&layout, &request("@scope/pkg/2.0.0"), &|_| answer.clone())
                .unwrap();
            assert_eq!(response.status, status);
            assert_eq!(response.body, words.into_bytes());
        }
        let held = projection
            .serve(&layout, &request("@scope/pkg/2.0.0"), &|_| Ok(()))
            .unwrap();
        assert_eq!(held.status, 200);
        let meta = ArtifactMeta::new(
            Ecosystem::new("eco"),
            EcosystemName::new("@scope/pkg"),
            Version::new("2.0.0"),
        )
        .unwrap();
        assert_eq!(held.body, canonical(&meta));
        assert_eq!(projection.misses(), vec![pin("@scope/pkg", "2.0.0"); 3]);
        let none = projection
            .serve(&layout, &request(""), &|_| panic!("asked the core"))
            .unwrap();
        assert_eq!(none.status, 404);
    }

    #[test]
    fn scripted_to_serve_none_it_answers_absent_naming_serving() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        projection.serves_none();
        let layout = Layout {
            at: "layouts/eco".into(),
        };
        for path in ["", "site/1.0.0"] {
            match projection.serve(&layout, &request(path), &|_| panic!("asked the core")) {
                Err(Not::Absent(a)) => {
                    assert_eq!(a.source.seam, Seam::Projection);
                    assert_eq!(a.what, "serving");
                }
                other => panic!("a projection serving none answered {other:?}"),
            }
        }
        assert_eq!(projection.misses(), []);
        assert_eq!(projection.requests().len(), 2);
    }

    fn pin(name: &str, version: &str) -> Pinned {
        Pinned {
            name: EcosystemName::new(name),
            version: Version::new(version),
        }
    }

    #[test]
    fn pins_set_for_a_checkout_are_answered_in_the_order_set() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        let pins = [pin("pkg", "1.1.0"), pin("lib", "0.3.0")];
        projection.lockfile("/work/checkout", &pins);
        assert_eq!(
            projection.pinned("/work/checkout").unwrap(),
            pins.iter().cloned().map(Pin::Version).collect::<Vec<_>>()
        );
        match projection.pinned("/work/another") {
            Err(Not::Absent(a)) => {
                assert_eq!(a.source.seam, Seam::Projection);
                assert_eq!(a.what, "lockfile");
            }
            other => panic!("a checkout with no lockfile answered {other:?}"),
        }
        projection.lockfile("/work/empty", &[]);
        assert_eq!(projection.pinned("/work/empty").unwrap(), []);
    }

    #[test]
    fn ranges_set_for_a_checkout_are_answered_after_its_pins_with_or_without_a_lockfile() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        let ranges = [requirement("fam-a", "1"), requirement("fam-c", "0.3")];
        projection.lockfile("/work/checkout", &[pin("pkg", "1.1.0")]);
        projection.ranges("/work/checkout", &ranges);
        assert_eq!(
            projection.pinned("/work/checkout").unwrap(),
            [
                Pin::Version(pin("pkg", "1.1.0")),
                Pin::Range(ranges[0].clone()),
                Pin::Range(ranges[1].clone()),
            ]
        );
        projection.ranges("/work/no-lockfile", &ranges[..1]);
        assert_eq!(
            projection.pinned("/work/no-lockfile").unwrap(),
            [Pin::Range(ranges[0].clone())]
        );
    }

    fn requirement(name: &str, range: &str) -> Requirement {
        Requirement {
            name: EcosystemName::new(name),
            range: Range::new(range),
        }
    }

    fn held(version: &str) -> Artifact {
        Artifact {
            digest: Digest::of(version.as_bytes()),
            meta: ArtifactMeta::new(
                Ecosystem::new("eco"),
                EcosystemName::new("site"),
                Version::new(version),
            )
            .unwrap(),
        }
    }

    fn versions(spelled: &[&str]) -> Vec<Version> {
        spelled.iter().map(|v| Version::new(*v)).collect()
    }

    #[test]
    fn requirements_set_for_an_artifact_are_answered_in_the_order_set() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        let requirements = [
            requirement("@scope/lib", "^1.2.0"),
            requirement("left-pad", ">=1.2.0-rc.1 <2"),
        ];
        projection.manifest(&held("1.0.0").digest, &requirements);
        assert_eq!(projection.requires(&held("1.0.0")).unwrap(), requirements);
        match projection.requires(&held("1.1.0")) {
            Err(Not::Absent(a)) => {
                assert_eq!(a.source.seam, Seam::Projection);
                assert_eq!(a.what, "the manifest of site@1.1.0");
            }
            other => panic!("an artifact given no manifest answered {other:?}"),
        }
        projection.manifest(&held("1.2.0").digest, &[]);
        assert_eq!(projection.requires(&held("1.2.0")).unwrap(), []);
    }

    #[test]
    fn versions_that_satisfy_a_range_are_answered_newest_first() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        let given = versions(&["1.9.0", "2.0.0", "1.10.0", "1.2.0", "1.1.9"]);
        assert_eq!(
            projection.satisfying(&Range::new("^1.2"), &given).unwrap(),
            versions(&["1.10.0", "1.9.0", "1.2.0"])
        );
        assert_eq!(
            projection.satisfying(&Range::new("^3"), &given).unwrap(),
            []
        );
    }

    #[test]
    fn versions_a_test_says_satisfy_a_range_are_answered_in_its_order() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        let range = Range::new("a range of the test's own");
        projection.satisfied(&range, &versions(&["b", "a", "c"]));
        assert_eq!(
            projection
                .satisfying(&range, &versions(&["a", "c", "d", "b"]))
                .unwrap(),
            versions(&["b", "a", "c"])
        );
        assert_eq!(
            projection
                .satisfying(&range, &versions(&["c", "d"]))
                .unwrap(),
            versions(&["c"])
        );
        assert_eq!(
            projection
                .satisfying(&Range::new("^1"), &versions(&["1.0.0", "1.2.0"]))
                .unwrap(),
            versions(&["1.2.0", "1.0.0"]),
            "a range no test set is read as Semantic Versioning"
        );
        let calls = projection.calls();
        assert_eq!(calls.len(), 3);
        assert_eq!(calls[0].given["range"], "a range of the test's own");
        assert_eq!(calls[0].given["versions"], json!(["a", "c", "d", "b"]));
    }

    #[test]
    fn a_range_it_cannot_read_is_refused_in_the_parsers_words() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        let words = semver::VersionReq::parse("not a range")
            .unwrap_err()
            .to_string();
        match projection.satisfying(&Range::new("not a range"), &versions(&["1.0.0"])) {
            Err(Not::Refused(r)) => {
                assert_eq!(r.source.seam, Seam::Projection);
                assert_eq!(r.why, Why::Said);
                assert_eq!(r.said, Some(words));
            }
            other => panic!("an unreadable range answered {other:?}"),
        }
    }

    #[test]
    fn a_version_it_cannot_read_satisfies_no_range() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        let given = versions(&["1.2", "1.3.0", "latest", "1.2.5"]);
        assert_eq!(
            projection.satisfying(&Range::new("^1.2"), &given).unwrap(),
            versions(&["1.3.0", "1.2.5"])
        );
    }

    #[test]
    fn a_render_of_artifacts_keeps_the_place_and_the_artifacts_given() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        let artifacts: Vec<Artifact> = ["1.0.0", "1.1.0"]
            .into_iter()
            .map(|v| Artifact {
                digest: Digest::of(v.as_bytes()),
                meta: artifact_pack(&projection, v).meta.unwrap(),
            })
            .collect();
        let rendered = projection
            .render(
                Contents::Artifacts(&artifacts),
                "/work/.tailrace/layouts/eco",
            )
            .unwrap();
        assert_eq!(
            rendered.holds,
            Holds::Versions(artifacts.iter().map(|a| a.meta.clone()).collect())
        );
        assert_eq!(
            projection.renders(),
            [("/work/.tailrace/layouts/eco".to_string(), artifacts)]
        );
        projection
            .render(
                Contents::CacheEntries(&[CacheEntry {
                    build: b7e1(),
                    digest: Digest::of(b"b7e1"),
                }]),
                "layouts/cache",
            )
            .unwrap();
        assert_eq!(projection.renders().len(), 1, "a cache entry was kept");
    }

    #[test]
    fn every_operation_can_be_faulted_and_is_recorded() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        for op in [
            ProjectionOp::Pack,
            ProjectionOp::Render,
            ProjectionOp::Serve,
            ProjectionOp::Pinned,
            ProjectionOp::Requires,
            ProjectionOp::Satisfying,
        ] {
            projection.fault(op, Fault::Refused("no".into()), Times::Once);
        }
        projection.manifest(&held("1.0.0").digest, &[requirement("lib", "^1")]);
        assert!(projection.pack(&output(), &b7e1(), None).is_err());
        assert!(projection.render(Contents::Artifacts(&[]), "at").is_err());
        assert!(projection
            .serve(&Layout { at: "at".into() }, &request(""), &|_| Ok(()))
            .is_err());
        assert!(projection.pinned("checkout").is_err());
        assert!(projection.requires(&held("1.0.0")).is_err());
        assert!(projection
            .satisfying(&Range::new("^1"), &versions(&["1.0.0"]))
            .is_err());
        assert_eq!(projection.renders(), [], "a faulted render was kept");
        let calls = projection.calls();
        assert_eq!(
            calls.iter().map(|c| c.op).collect::<Vec<_>>(),
            [
                ProjectionOp::Pack,
                ProjectionOp::Render,
                ProjectionOp::Serve,
                ProjectionOp::Pinned,
                ProjectionOp::Requires,
                ProjectionOp::Satisfying
            ]
        );
        assert_eq!(calls[0].given["build"], "b7e1");
        assert_eq!(calls[0].given["version"], serde_json::Value::Null);
        assert_eq!(calls[1].given["contents"]["artifacts"], json!([]));
        assert_eq!(calls[1].given["at"], "at");
        assert_eq!(calls[3].given["checkout"], "checkout");
        assert_eq!(calls[4].given["held"], json!(held("1.0.0")));
        assert_eq!(calls[5].given["range"], "^1");
        assert_eq!(calls[5].given["versions"], json!(["1.0.0"]));
    }
}
