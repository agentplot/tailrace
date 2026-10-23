//! The fake projection (B.6): its ecosystem and the name each output
//! takes. A pack is the canonical JSON of what it was given, read from no
//! disk; a render writes nothing (design D7).

use std::collections::BTreeMap;

use serde_json::json;
use tailrace_seams::{
    canonical, Answer, Artifact, ArtifactMeta, Ecosystem, EcosystemName, Layout, Not, Output,
    OutputName, Packed, Projection, Rendered, Seam, Serving, Version, Why,
};

use crate::Fake;

/// The fake projection's operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionOp {
    Pack,
    Render,
    Serve,
}

/// The projection's ecosystem, the names outputs take in it, and whether
/// its packing is the same twice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionState {
    ecosystem: Ecosystem,
    names: BTreeMap<OutputName, EcosystemName>,
    unsteady: bool,
    packs: u64,
}

/// A projection in memory.
pub type FakeProjection = Fake<ProjectionState, ProjectionOp>;

impl FakeProjection {
    /// A projection of `ecosystem`, in which each output takes its own
    /// name until [`FakeProjection::named`] gives it another.
    pub fn new(ecosystem: Ecosystem) -> Self {
        Fake::with(
            Seam::Projection,
            ProjectionState {
                ecosystem,
                names: BTreeMap::new(),
                unsteady: false,
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
}

impl Projection for FakeProjection {
    fn pack(&self, output: &Output, version: &Version) -> Answer<Packed> {
        let given = json!({ "output": output, "version": version });
        self.ask(
            ProjectionOp::Pack,
            given,
            format!("output {}", output.name),
            |s, source| {
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
                    meta,
                    same_again: !s.unsteady,
                })
            },
        )
    }

    fn render(&self, artifacts: &[Artifact], at: &str) -> Answer<Rendered> {
        let given = json!({ "artifacts": artifacts, "at": at });
        self.ask(
            ProjectionOp::Render,
            given,
            format!("the layout at {at}"),
            |_, _| {
                Ok(Rendered {
                    layout: Layout { at: at.to_string() },
                    holds: artifacts.iter().map(|a| a.meta.clone()).collect(),
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
    use tailrace_seams::Digest;

    use super::*;
    use crate::{Fault, Times};

    fn output() -> Output {
        Output {
            name: OutputName::new("site"),
            files: vec!["dist/index.html".into()],
        }
    }

    #[test]
    fn a_pack_is_its_ecosystems_bytes_and_metadata() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        projection.named(&OutputName::new("site"), EcosystemName::new("web-site"));
        let packed = projection.pack(&output(), &Version::new("1.0.0")).unwrap();
        assert_eq!(packed.meta.ecosystem(), &Ecosystem::new("eco"));
        assert_eq!(packed.meta.name(), &EcosystemName::new("web-site"));
        assert_eq!(packed.meta.version(), &Version::new("1.0.0"));
        assert!(packed.same_again);
        assert_eq!(
            projection.pack(&output(), &Version::new("1.0.0")).unwrap(),
            packed
        );
        assert_ne!(
            projection
                .pack(&output(), &Version::new("1.0.1"))
                .unwrap()
                .bytes,
            packed.bytes
        );
    }

    #[test]
    fn an_output_takes_its_own_name_until_named() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        let packed = projection.pack(&output(), &Version::new("1.0.0")).unwrap();
        assert_eq!(packed.meta.name(), &EcosystemName::new("site"));
    }

    #[test]
    fn a_pack_that_is_not_the_same_again_says_so() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        projection.unsteady();
        let first = projection.pack(&output(), &Version::new("1.0.0")).unwrap();
        let second = projection.pack(&output(), &Version::new("1.0.0")).unwrap();
        assert!(!first.same_again);
        assert!(!second.same_again);
        assert_ne!(first.bytes, second.bytes);
    }

    #[test]
    fn a_pack_with_no_version_is_refused_naming_it() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        assert!(matches!(
            projection.pack(&output(), &Version::new("")),
            Err(Not::Refused(r)) if r.said.as_deref() == Some("the metadata names no version")
        ));
    }

    #[test]
    fn a_render_holds_what_it_was_given_and_is_served() {
        let projection = FakeProjection::new(Ecosystem::new("eco"));
        let meta = projection
            .pack(&output(), &Version::new("1.0.0"))
            .unwrap()
            .meta;
        let artifact = Artifact {
            digest: Digest::of(b"site"),
            meta: meta.clone(),
        };
        let rendered = projection.render(&[artifact], "layouts/eco").unwrap();
        assert_eq!(rendered.layout.at, "layouts/eco");
        assert_eq!(rendered.holds, vec![meta]);
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
        assert!(projection.pack(&output(), &Version::new("1.0.0")).is_err());
        assert!(projection.render(&[], "at").is_err());
        assert!(projection.serve(&Layout { at: "at".into() }).is_err());
        assert_eq!(
            projection.calls().iter().map(|c| c.op).collect::<Vec<_>>(),
            [
                ProjectionOp::Pack,
                ProjectionOp::Render,
                ProjectionOp::Serve
            ]
        );
    }
}
