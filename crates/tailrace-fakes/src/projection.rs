//! The fake projection (B.6): its ecosystem, the name each output takes,
//! whether it packs cache entries, and the build identities it holds no
//! archive for. A pack is the canonical JSON of
//! what it was given, read from no disk; a render writes nothing (design
//! D7).

use std::collections::{BTreeMap, BTreeSet};

use serde_json::json;
use tailrace_seams::{
    canonical, no_cache_entries, Answer, ArtifactMeta, BuildId, Contents, Ecosystem, EcosystemName,
    Holds, Layout, Not, Output, OutputName, Packed, Projection, Rendered, Seam, Serving, Version,
    Why,
};

use crate::Fake;

/// The fake projection's operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionOp {
    Pack,
    Render,
    Serve,
}

/// The projection's ecosystem, the names outputs take in it, whether its
/// packing is the same twice, whether it packs cache entries, and the
/// build identities it holds no archive for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionState {
    ecosystem: Ecosystem,
    names: BTreeMap<OutputName, EcosystemName>,
    unsteady: bool,
    caches: bool,
    no_archive: BTreeSet<BuildId>,
    packs: u64,
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

    #[test]
    fn every_operation_can_be_faulted_and_is_recorded() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        for op in [
            ProjectionOp::Pack,
            ProjectionOp::Render,
            ProjectionOp::Serve,
        ] {
            projection.fault(op, Fault::Refused("no".into()), Times::Once);
        }
        assert!(projection.pack(&output(), &b7e1(), None).is_err());
        assert!(projection.render(Contents::Artifacts(&[]), "at").is_err());
        assert!(projection.serve(&Layout { at: "at".into() }).is_err());
        let calls = projection.calls();
        assert_eq!(
            calls.iter().map(|c| c.op).collect::<Vec<_>>(),
            [
                ProjectionOp::Pack,
                ProjectionOp::Render,
                ProjectionOp::Serve
            ]
        );
        assert_eq!(calls[0].given["build"], "b7e1");
        assert_eq!(calls[0].given["version"], serde_json::Value::Null);
        assert_eq!(calls[1].given["contents"]["artifacts"], json!([]));
        assert_eq!(calls[1].given["at"], "at");
    }
}
