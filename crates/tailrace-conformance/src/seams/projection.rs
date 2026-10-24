//! A projection's group: 84 to 87, and 48 and 50 as far as a projection
//! keeps them (B.6).

use tailrace_seams::{Artifact, ArtifactMeta, Output, Projection, Seam, Version};

use crate::seams::unreadable;
use crate::{ok, Rig, Scenario};

pub const GROUP: &str = "projection";

/// A projection, made new for each scenario.
pub trait ProjectionRig: Rig {
    fn projection(&self) -> &dyn Projection;

    /// An output a build left, which the projection packs.
    fn output(&self) -> Output;

    /// `n` artifacts in the projection's ecosystem, each of another
    /// version, held where the projection renders from.
    fn artifacts(&self, n: usize) -> Vec<Artifact>;

    /// Whether the ecosystem's clients read no files, so the projection
    /// serves its layouts (86).
    fn serves(&self) -> bool;

    /// The projection made unreachable, every operation answering
    /// unreadable, and the words it answers with.
    fn unreachable(&self) -> String;
}

pub fn scenarios() -> Vec<Scenario<dyn ProjectionRig>> {
    vec![
        Scenario {
            name: "packed the same again",
            plays: None,
            cites: &[84, 87],
            run: packed_the_same_again,
        },
        Scenario {
            name: "a layout of the versions given",
            plays: None,
            cites: &[48, 50, 85],
            run: a_layout_of_the_versions_given,
        },
        Scenario {
            name: "served where the clients read no files",
            plays: None,
            cites: &[86],
            run: served_where_the_clients_read_no_files,
        },
        Scenario {
            name: "a projection that cannot be reached",
            plays: None,
            cites: &[58, 59],
            run: a_projection_that_cannot_be_reached,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}

/// Where the group renders its layouts.
const AT: &str = "layouts/conformance";

fn packed_the_same_again(rig: &dyn ProjectionRig) {
    let (projection, output, version) = (rig.projection(), rig.output(), Version::new("1.4.0"));
    let first = ok(projection.pack(&output, &version), "the first pack");
    let second = ok(projection.pack(&output, &version), "the second pack");
    for packed in [&first, &second] {
        let meta = &packed.meta;
        assert_eq!(
            meta.version(),
            &version,
            "a pack's metadata names another version: {meta:?}"
        );
        assert!(
            !meta.ecosystem().as_str().is_empty() && !meta.name().as_str().is_empty(),
            "a pack's metadata names no ecosystem or name: {meta:?}"
        );
    }
    if first.same_again && second.same_again {
        assert_eq!(
            first.bytes, second.bytes,
            "one output and version packed to other bytes, and the projection promised the same"
        );
        assert_eq!(
            first.meta, second.meta,
            "one output packed to other metadata"
        );
    }
}

fn metas(artifacts: &[Artifact]) -> Vec<ArtifactMeta> {
    artifacts.iter().map(|a| a.meta.clone()).collect()
}

fn a_layout_of_the_versions_given(rig: &dyn ProjectionRig) {
    let artifacts = rig.artifacts(2);
    assert_eq!(artifacts.len(), 2, "the rig gave {artifacts:?}");
    let projection = rig.projection();
    let first = ok(projection.render(&artifacts, AT), "the first render");
    let again = ok(projection.render(&artifacts, AT), "the second render");
    let mut held = first.holds.clone();
    let mut given = metas(&artifacts);
    held.sort_by(|a, b| a.version().cmp(b.version()));
    given.sort_by(|a, b| a.version().cmp(b.version()));
    assert_eq!(
        held, given,
        "the layout holds other versions than it was given"
    );
    assert_eq!(
        first, again,
        "the same artifacts rendered again gave another layout"
    );
}

fn served_where_the_clients_read_no_files(rig: &dyn ProjectionRig) {
    if !rig.serves() {
        return;
    }
    let projection = rig.projection();
    let rendered = ok(projection.render(&rig.artifacts(1), AT), "render");
    let serving = ok(projection.serve(&rendered.layout), "serve");
    assert!(
        !serving.address.is_empty(),
        "a served layout answered no address: {serving:?}"
    );
}

fn a_projection_that_cannot_be_reached(rig: &dyn ProjectionRig) {
    let output = rig.output();
    let words = rig.unreachable();
    unreadable(
        rig.projection().pack(&output, &Version::new("1.4.0")),
        Seam::Projection,
        &words,
        "pack",
    );
}
