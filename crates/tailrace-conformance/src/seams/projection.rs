//! A projection's group: 84 to 87, and 48 and 50 as far as a projection
//! keeps them (B.6), for artifacts and, where the projection packs them,
//! cache entries.

use tailrace_seams::{
    Artifact, ArtifactMeta, BuildId, CacheEntry, Contents, Digest, Holds, Output, Projection, Seam,
    Version,
};

use crate::seams::{absent, unreadable};
use crate::{not_applicable, ok, Rig, Scenario};

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

    /// Whether the projection packs cache entries; one that packs none
    /// answers absent for one (84, 85).
    fn caches(&self) -> bool;

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
        Scenario {
            name: "a cache entry packed with no version",
            plays: None,
            cites: &[84, 87],
            run: a_cache_entry_packed_with_no_version,
        },
        Scenario {
            name: "a layout of the build identities given",
            plays: None,
            cites: &[85],
            run: a_layout_of_the_build_identities_given,
        },
        Scenario {
            name: "a projection that packs no cache entries",
            plays: None,
            cites: &[84, 85],
            run: a_projection_that_packs_no_cache_entries,
        },
    ]
}

/// Where the group renders its layouts.
const AT: &str = "layouts/conformance";

/// The build identity the group packs at.
const B7E1: &str = "b7e1";

/// Another build identity, for a second cache entry.
const C3D4: &str = "c3d4";

fn packed_the_same_again(rig: &dyn ProjectionRig) {
    let (projection, output, version) = (rig.projection(), rig.output(), Version::new("1.4.0"));
    let at = BuildId::new(B7E1);
    let first = ok(
        projection.pack(&output, &at, Some(&version)),
        "the first pack",
    );
    let second = ok(
        projection.pack(&output, &at, Some(&version)),
        "the second pack",
    );
    for packed in [&first, &second] {
        let meta = packed
            .meta
            .as_ref()
            .expect("packed the same again: a pack given a version named no metadata");
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
    let first = ok(
        projection.render(Contents::Artifacts(&artifacts), AT),
        "the first render",
    );
    let again = ok(
        projection.render(Contents::Artifacts(&artifacts), AT),
        "the second render",
    );
    let Holds::Versions(mut held) = first.holds.clone() else {
        panic!("a layout of artifacts holds no versions: {:?}", first.holds)
    };
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
    let rendered = ok(
        projection.render(Contents::Artifacts(&rig.artifacts(1)), AT),
        "render",
    );
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
        rig.projection()
            .pack(&output, &BuildId::new(B7E1), Some(&Version::new("1.4.0"))),
        Seam::Projection,
        &words,
        "pack",
    );
}

fn a_cache_entry_packed_with_no_version(rig: &dyn ProjectionRig) {
    if !rig.caches() {
        not_applicable("the projection packs no cache entries");
    }
    let (projection, output, at) = (rig.projection(), rig.output(), BuildId::new(B7E1));
    let first = ok(projection.pack(&output, &at, None), "the first pack");
    let second = ok(projection.pack(&output, &at, None), "the second pack");
    for packed in [&first, &second] {
        assert_eq!(
            packed.meta, None,
            "a cache entry was packed with artifact metadata"
        );
    }
    if first.same_again && second.same_again {
        assert_eq!(
            first.bytes, second.bytes,
            "one output at one build identity packed to other bytes, and the projection promised the same"
        );
    }
}

fn a_layout_of_the_build_identities_given(rig: &dyn ProjectionRig) {
    if !rig.caches() {
        not_applicable("the projection packs no cache entries");
    }
    let (projection, output) = (rig.projection(), rig.output());
    let entries: Vec<CacheEntry> = [B7E1, C3D4]
        .into_iter()
        .map(|build| {
            let build = BuildId::new(build);
            let packed = ok(
                projection.pack(&output, &build, None),
                &format!("the pack at {build}"),
            );
            CacheEntry {
                digest: Digest::of(&packed.bytes),
                build,
            }
        })
        .collect();
    let first = ok(
        projection.render(Contents::CacheEntries(&entries), AT),
        "the first render",
    );
    let again = ok(
        projection.render(Contents::CacheEntries(&entries), AT),
        "the second render",
    );
    let Holds::BuildIds(mut held) = first.holds.clone() else {
        panic!(
            "a layout of cache entries holds no build identities: {:?}",
            first.holds
        )
    };
    let mut given: Vec<BuildId> = entries.iter().map(|e| e.build.clone()).collect();
    held.sort();
    given.sort();
    assert_eq!(
        held, given,
        "the layout holds other build identities than it was given"
    );
    assert_eq!(
        first, again,
        "the same cache entries rendered again gave another layout"
    );
}

fn a_projection_that_packs_no_cache_entries(rig: &dyn ProjectionRig) {
    if rig.caches() {
        not_applicable("the projection packs cache entries");
    }
    let (projection, output, at) = (rig.projection(), rig.output(), BuildId::new(B7E1));
    absent(
        projection.pack(&output, &at, None),
        Seam::Projection,
        "cache entries",
        "pack with no version",
    );
    let entry = CacheEntry {
        build: at,
        digest: Digest::of(output.name.as_str().as_bytes()),
    };
    absent(
        projection.render(Contents::CacheEntries(&[entry]), AT),
        Seam::Projection,
        "cache entries",
        "render of a cache entry",
    );
}
