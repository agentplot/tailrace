//! The fake projection (B.6): its ecosystem, the name each output takes,
//! whether it packs cache entries, the build identities it holds no
//! archive for, the pins of each checkout's lockfile, and what the
//! manifest of each artifact, by digest, requires. A pack is the
//! canonical JSON of what it was given, read from no disk; a render
//! writes nothing and keeps the place and the artifacts it was given;
//! pinned reads no checkout and requires no bytes. satisfying keeps the
//! versions a range matches and orders them newest first by Semantic
//! Versioning, so a test can list versions out of order and get the
//! newest that fits (140, 23; design D7).

use std::collections::{BTreeMap, BTreeSet};

use serde_json::json;
use tailrace_seams::{
    canonical, no_cache_entries, no_lockfile, Answer, Artifact, ArtifactMeta, BuildId, Contents,
    Digest, Ecosystem, EcosystemName, Holds, Layout, Not, Output, OutputName, Packed, Pinned,
    Projection, Range, Rendered, Requirement, Seam, Serving, Version, Why,
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
/// packing is the same twice, whether it packs cache entries, the build
/// identities it holds no archive for, the pins of each checkout holding
/// a lockfile, what each artifact's manifest requires, by its digest, and
/// each render of artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionState {
    ecosystem: Ecosystem,
    names: BTreeMap<OutputName, EcosystemName>,
    unsteady: bool,
    caches: bool,
    no_archive: BTreeSet<BuildId>,
    packs: u64,
    lockfiles: BTreeMap<String, Vec<Pinned>>,
    manifests: BTreeMap<Digest, Vec<Requirement>>,
    renders: Vec<(String, Vec<Artifact>)>,
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
                no_archive: BTreeSet::new(),
                packs: 0,
                lockfiles: BTreeMap::new(),
                manifests: BTreeMap::new(),
                renders: Vec::new(),
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

    /// 140: the manifest of the artifact at `digest` requires
    /// `requirements`, answered in the order given; every other
    /// artifact's manifest is one the fake cannot find.
    pub fn manifest(&self, digest: &Digest, requirements: &[Requirement]) {
        self.state()
            .manifests
            .insert(digest.clone(), requirements.to_vec());
    }

    /// 85: each render of artifacts answered, in order: the place it was
    /// asked at and the artifacts it was given.
    pub fn renders(&self) -> Vec<(String, Vec<Artifact>)> {
        self.state().renders.clone()
    }
}

impl Projection for FakeProjection {
    fn pack(&self, output: &Output, at: &BuildId, version: Option<&Version>) -> Answer<Packed> {
        let given = json!({ "output": output, "build": at, "version": version });
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

    fn serve(&self, layout: &Layout) -> Answer<Serving> {
        let given = json!({ "layout": layout });
        self.ask(
            ProjectionOp::Serve,
            given,
            format!("the layout at {}", layout.at),
            |s, _| {
                Ok(Serving {
                    address: format!("fake://{}/{}", s.ecosystem, layout.at),
                })
            },
        )
    }

    fn pinned(&self, checkout: &str) -> Answer<Vec<Pinned>> {
        let given = json!({ "checkout": checkout });
        self.ask(
            ProjectionOp::Pinned,
            given,
            format!("the lockfile of {checkout}"),
            |s, source| {
                s.lockfiles
                    .get(checkout)
                    .cloned()
                    .ok_or_else(|| no_lockfile(source))
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
            |_, source| {
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
            .pack(&output(), &b7e1(), Some(&Version::new(version)))
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
        let version = Version::new("1.0.0");
        let at_b7e1 = projection.pack(&output(), &b7e1(), Some(&version)).unwrap();
        let at_c3d4 = projection
            .pack(&output(), &BuildId::new("c3d4"), Some(&version))
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
            projection.pack(&output(), &b7e1(), Some(&Version::new(""))),
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
    fn a_render_holds_what_it_was_given_and_is_served() {
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
        assert_eq!(
            projection.serve(&rendered.layout).unwrap().address,
            "fake://eco/layouts/eco"
        );
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
        assert_eq!(projection.pinned("/work/checkout").unwrap(), pins);
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
        assert!(projection.serve(&Layout { at: "at".into() }).is_err());
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
