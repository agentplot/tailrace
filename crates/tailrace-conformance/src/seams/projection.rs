//! A projection's group: 84 to 87, and 48 and 50 as far as a projection
//! keeps them (B.6), for artifacts and, where the projection packs them,
//! cache entries; what a checkout's lockfile pins (137); what a held
//! version's manifest requires and which versions satisfy a range, newest
//! first (140); and a served layout's answer to its client's request for
//! a version, held or asked of the core (86, 46, 115, 50, 99, 49).

use std::cell::RefCell;

use tailrace_core::paths_hold;
use tailrace_seams::{
    Answer, Artifact, ArtifactMeta, BuildId, CacheEntry, Component, ComponentName, Contents,
    Digest, Holds, Layout, Not, Output, PackAt, Packed, Pin, Pinned, Projection, Range, Request,
    Requirement, Response, Seam, Source, Version, Why,
};

use crate::seams::{absent, unreadable};
use crate::{not_applicable, ok, Rig, Scenario};

pub const GROUP: &str = "projection";

/// A projection, made new for each scenario.
pub trait ProjectionRig: Rig {
    fn projection(&self) -> &dyn Projection;

    /// An output a build left, which the projection packs.
    fn output(&self) -> Output;

    /// The component the output belongs to and its declared outputs, as
    /// the build left them, the output among them (84, 123): by default a
    /// component of the output alone, whose paths hold every path.
    fn component(&self) -> (Component, Vec<Output>) {
        let output = self.output();
        let component = Component {
            name: ComponentName::new(output.name.as_str()),
            outputs: vec![output.name.clone()],
            paths: vec!["**".into()],
            prefix: format!("{}-v", output.name),
        };
        (component, vec![output])
    }

    /// `n` artifacts in the projection's ecosystem, each of another
    /// version, held where the projection renders from.
    fn artifacts(&self, n: usize) -> Vec<Artifact>;

    /// Whether the ecosystem's clients read no files, so the projection
    /// serves its layouts (86).
    fn serves(&self) -> bool;

    /// The path below a served layout's prefix of the ecosystem's own
    /// request for the bytes of the version `meta` names, since a
    /// request's form is the ecosystem's (86): by default
    /// `<name>/<version>`, which a projection that serves none answers
    /// absent as it answers any path.
    fn request(&self, meta: &ArtifactMeta) -> String {
        format!("{}/{}", meta.name(), meta.version())
    }

    /// Whether the projection packs cache entries; one that packs none
    /// answers absent for one (84, 85).
    fn caches(&self) -> bool;

    /// Whether the ecosystem has a lockfile a checkout pins its versions
    /// in (137).
    fn has_lockfile(&self) -> bool;

    /// A checkout the rig stages, holding the ecosystem's lockfile where
    /// `lockfile` and none of the ecosystem's otherwise: where it is, and
    /// the pins its lockfile holds, as the lockfile spells them, none
    /// where it holds none.
    fn checkout(&self, lockfile: bool) -> (String, Vec<Pinned>);

    /// Whether the ecosystem has a manifest a held version requires
    /// other names of the ecosystem in (140).
    fn has_manifest(&self) -> bool;

    /// An artifact of the ecosystem held where the projection reads it,
    /// and what its manifest requires, as the manifest spells it, in the
    /// order it gives it; none where the ecosystem has no manifest.
    fn held(&self) -> (Artifact, Vec<Requirement>);

    /// A range as the ecosystem spells one, versions listed in neither
    /// the ecosystem's order nor spelling order, and those of them that
    /// satisfy the range, newest first by the ecosystem's own ordering;
    /// none of any where the ecosystem has no manifest.
    fn ranged(&self) -> (Range, Vec<Version>, Vec<Version>);

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
        Scenario {
            name: "pinned as the lockfile spells it",
            plays: None,
            cites: &[137],
            run: pinned_as_the_lockfile_spells_it,
        },
        Scenario {
            name: "no lockfile, absent",
            plays: None,
            cites: &[58, 137],
            run: no_lockfile_absent,
        },
        Scenario {
            name: "required as the manifest spells it",
            plays: None,
            cites: &[140],
            run: required_as_the_manifest_spells_it,
        },
        Scenario {
            name: "satisfied by the ecosystem's own ordering",
            plays: None,
            cites: &[23, 140],
            run: satisfied_by_the_ecosystems_own_ordering,
        },
        Scenario {
            name: "no manifest, none",
            plays: None,
            cites: &[140],
            run: no_manifest_none,
        },
        Scenario {
            name: "a version held, served",
            plays: None,
            cites: &[86, 46],
            run: a_version_held_served,
        },
        Scenario {
            name: "a version lacked, asked of the core",
            plays: None,
            cites: &[115, 50],
            run: a_version_lacked_asked_of_the_core,
        },
        Scenario {
            name: "a refusal in the response",
            plays: None,
            cites: &[99, 59],
            run: a_refusal_in_the_response,
        },
        Scenario {
            name: "one base, two places",
            plays: None,
            cites: &[49],
            run: one_base_two_places,
        },
    ]
}

/// Where the group renders its layouts.
const AT: &str = "layouts/conformance";

/// The build identity the group packs at.
const B7E1: &str = "b7e1";

/// Another build identity, for a second cache entry.
const C3D4: &str = "c3d4";

/// `output` packed by the rig's projection at `build` as an artifact at
/// `version`, given the rig's component, its outputs and its paths'
/// match as a publish hands them (84, 123).
fn packed_at(
    rig: &dyn ProjectionRig,
    output: &Output,
    build: &BuildId,
    version: &Version,
) -> Answer<Packed> {
    let (component, outputs) = rig.component();
    let holds = paths_hold(&component.paths).expect("the rig's component's paths are globs");
    rig.projection().pack(
        output,
        build,
        Some(&PackAt {
            version,
            component: &component,
            outputs: &outputs,
            holds: &holds,
        }),
    )
}

/// Where the group renders a second layout of the same artifacts.
const ELSEWHERE: &str = "layouts/conformance-elsewhere";

/// The base the group serves its layouts under, as a client reached it.
const BASE: &str = "http://127.0.0.1:7420/o/r/conformance";

/// The words of the refusal the core answers a miss with.
const WANTING: &str = "acceptance wanting";

/// The client's `GET` of `path` below a layout served under [`BASE`].
fn get(path: String) -> Request {
    Request {
        method: "GET".into(),
        path,
        base: BASE.into(),
    }
}

/// The pin naming `meta`'s ecosystem name and version.
fn pin_of(meta: &ArtifactMeta) -> Pinned {
    Pinned {
        name: meta.name().clone(),
        version: meta.version().clone(),
    }
}

/// That `response` answers the request it was asked: `200` with bytes.
fn answers(response: &Response, asked: &str) {
    assert!(
        response.status == 200 && !response.body.is_empty(),
        "{asked} answered {} {} with {} bytes",
        response.status,
        response.content_type,
        response.body.len()
    );
}

/// Whether `bytes` hold `words` as they are.
fn holds(bytes: &[u8], words: &str) -> bool {
    bytes
        .windows(words.len())
        .any(|window| window == words.as_bytes())
}

/// The versions a miss asked the core for, recorded, the core answering
/// each with `answer`.
struct Asked<F: Fn() -> Answer<()>> {
    pins: RefCell<Vec<Pinned>>,
    answer: F,
}

impl<F: Fn() -> Answer<()>> Asked<F> {
    fn new(answer: F) -> Self {
        Asked {
            pins: RefCell::new(Vec::new()),
            answer,
        }
    }

    fn miss(&self, pin: &Pinned) -> Answer<()> {
        self.pins.borrow_mut().push(pin.clone());
        (self.answer)()
    }

    fn pins(&self) -> Vec<Pinned> {
        self.pins.borrow().clone()
    }
}

/// Not applicable to a projection whose ecosystem's clients read files,
/// once it answers the rig's request absent naming serving and asks the
/// core nothing (86).
fn not_served(rig: &dyn ProjectionRig) {
    if rig.serves() {
        return;
    }
    let (held, _) = rig.held();
    let asked = Asked::new(|| Ok(()));
    let layout = Layout { at: AT.into() };
    absent(
        rig.projection()
            .serve(&layout, &get(rig.request(&held.meta)), &|pin| {
                asked.miss(pin)
            }),
        Seam::Projection,
        "serving",
        "serve of a projection whose clients read files",
    );
    assert_eq!(
        asked.pins(),
        [],
        "a projection that serves none asked the core"
    );
    not_applicable("the ecosystem's clients read files");
}

fn packed_the_same_again(rig: &dyn ProjectionRig) {
    let (output, version) = (rig.output(), Version::new("1.4.0"));
    let at = BuildId::new(B7E1);
    let first = ok(packed_at(rig, &output, &at, &version), "the first pack");
    let second = ok(packed_at(rig, &output, &at, &version), "the second pack");
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
    let artifacts = rig.artifacts(1);
    let rendered = ok(
        projection.render(Contents::Artifacts(&artifacts), AT),
        "render",
    );
    let asked = Asked::new(|| Ok(()));
    let served = ok(
        projection.serve(
            &rendered.layout,
            &get(rig.request(&artifacts[0].meta)),
            &|pin| asked.miss(pin),
        ),
        "serve",
    );
    answers(&served, "the request for a version the layout holds");
    assert_eq!(
        asked.pins(),
        [],
        "serving a version the layout holds asked the core"
    );
}

fn a_projection_that_cannot_be_reached(rig: &dyn ProjectionRig) {
    let output = rig.output();
    let words = rig.unreachable();
    unreadable(
        packed_at(rig, &output, &BuildId::new(B7E1), &Version::new("1.4.0")),
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

fn pinned_as_the_lockfile_spells_it(rig: &dyn ProjectionRig) {
    if !rig.has_lockfile() {
        not_applicable("the ecosystem has no lockfile");
    }
    let (at, pins) = rig.checkout(true);
    assert!(!pins.is_empty(), "the rig's lockfile pins nothing");
    let pinned = ok(rig.projection().pinned(&at), "pinned");
    let pins: Vec<Pin> = pins.into_iter().map(Pin::Version).collect();
    assert_eq!(
        pinned, pins,
        "pinned answered other names or versions than the lockfile pins"
    );
}

fn no_lockfile_absent(rig: &dyn ProjectionRig) {
    let (at, _) = rig.checkout(false);
    absent(
        rig.projection().pinned(&at),
        Seam::Projection,
        "lockfile",
        "pinned of a checkout with no lockfile",
    );
}

fn required_as_the_manifest_spells_it(rig: &dyn ProjectionRig) {
    if !rig.has_manifest() {
        not_applicable("the ecosystem has no manifest");
    }
    let (held, requirements) = rig.held();
    assert!(
        !requirements.is_empty(),
        "the rig's manifest requires nothing"
    );
    let required = ok(rig.projection().requires(&held), "requires");
    assert_eq!(
        required, requirements,
        "requires answered other names or ranges than the manifest requires"
    );
}

fn satisfied_by_the_ecosystems_own_ordering(rig: &dyn ProjectionRig) {
    if !rig.has_manifest() {
        not_applicable("the ecosystem has no manifest");
    }
    let (range, versions, satisfying) = rig.ranged();
    assert!(
        satisfying.len() > 1,
        "the rig's range is satisfied by fewer than two of its versions, so no order is read"
    );
    let as_listed: Vec<Version> = versions
        .iter()
        .filter(|v| satisfying.contains(v))
        .cloned()
        .collect();
    let mut by_spelling = satisfying.clone();
    by_spelling.sort();
    let mut by_spelling_down = by_spelling.clone();
    by_spelling_down.reverse();
    for (order, named) in [
        (&as_listed, "the order listed"),
        (&by_spelling, "spelling order"),
        (&by_spelling_down, "spelling order, newest last"),
    ] {
        assert_ne!(
            order, &satisfying,
            "the rig's answer is in {named}, so the scenario cannot tell the ecosystem's ordering from it"
        );
    }
    let answered = ok(rig.projection().satisfying(&range, &versions), "satisfying");
    assert_eq!(
        answered, satisfying,
        "satisfying answered other versions, or another order, than the ecosystem's newest first"
    );
}

fn no_manifest_none(rig: &dyn ProjectionRig) {
    if rig.has_manifest() {
        not_applicable("the ecosystem has a manifest");
    }
    let (held, _) = rig.held();
    let projection = rig.projection();
    let required = ok(projection.requires(&held), "requires");
    assert_eq!(
        required,
        [],
        "requires answered names from an ecosystem with no manifest"
    );
    let (range, mut versions, _) = rig.ranged();
    versions.push(held.meta.version().clone());
    let satisfying = ok(projection.satisfying(&range, &versions), "satisfying");
    assert_eq!(
        satisfying,
        [],
        "satisfying answered versions from an ecosystem with no manifest"
    );
}

fn a_version_held_served(rig: &dyn ProjectionRig) {
    not_served(rig);
    let projection = rig.projection();
    let artifacts = rig.artifacts(2);
    let rendered = ok(
        projection.render(Contents::Artifacts(&artifacts), AT),
        "render",
    );
    let asked = Asked::new(|| Ok(()));
    let served = ok(
        projection.serve(
            &rendered.layout,
            &get(rig.request(&artifacts[1].meta)),
            &|pin| asked.miss(pin),
        ),
        "serve",
    );
    answers(&served, "the request for a version the layout holds");
    assert_eq!(
        asked.pins(),
        [],
        "serving a version the layout holds asked the core"
    );
}

fn a_version_lacked_asked_of_the_core(rig: &dyn ProjectionRig) {
    not_served(rig);
    let projection = rig.projection();
    let artifacts = rig.artifacts(2);
    let rendered = ok(
        projection.render(Contents::Artifacts(&artifacts[..1]), AT),
        "render of the first",
    );
    // The core holds the version asked by rendering both into the layout.
    let asked = Asked::new(|| {
        projection
            .render(Contents::Artifacts(&artifacts), AT)
            .map(|_| ())
    });
    let served = ok(
        projection.serve(
            &rendered.layout,
            &get(rig.request(&artifacts[1].meta)),
            &|pin| asked.miss(pin),
        ),
        "serve",
    );
    assert_eq!(
        asked.pins(),
        [pin_of(&artifacts[1].meta)],
        "the core was not asked once for the version the layout lacks"
    );
    answers(&served, "the request for a version the core rendered");
}

fn a_refusal_in_the_response(rig: &dyn ProjectionRig) {
    not_served(rig);
    let projection = rig.projection();
    let artifacts = rig.artifacts(2);
    let rendered = ok(
        projection.render(Contents::Artifacts(&artifacts[..1]), AT),
        "render of the first",
    );
    let refused = Not::refused(
        &Source::new(Seam::Policy, "conformance"),
        Why::Said,
        Some(WANTING.into()),
    );
    let asked = Asked::new(|| Err(refused.clone()));
    let served = ok(
        projection.serve(
            &rendered.layout,
            &get(rig.request(&artifacts[1].meta)),
            &|pin| asked.miss(pin),
        ),
        "serve",
    );
    assert_eq!(asked.pins(), [pin_of(&artifacts[1].meta)]);
    assert_ne!(
        served.status, 200,
        "a version the core refused was answered as served"
    );
    assert!(
        holds(&served.body, WANTING),
        "the response to a refused version does not carry {WANTING:?} verbatim: {}",
        String::from_utf8_lossy(&served.body)
    );
}

fn one_base_two_places(rig: &dyn ProjectionRig) {
    not_served(rig);
    let projection = rig.projection();
    let artifacts = rig.artifacts(2);
    let asked = Asked::new(|| Ok(()));
    let mut served = Vec::new();
    for at in [AT, ELSEWHERE] {
        let rendered = ok(
            projection.render(Contents::Artifacts(&artifacts), at),
            &format!("render at {at}"),
        );
        served.push(ok(
            projection.serve(
                &rendered.layout,
                &get(rig.request(&artifacts[0].meta)),
                &|pin| asked.miss(pin),
            ),
            &format!("serve at {at}"),
        ));
    }
    answers(&served[0], "the request for a version the layout holds");
    assert_eq!(
        served[0], served[1],
        "one request under one base answered otherwise from two places"
    );
    for at in [AT, ELSEWHERE] {
        assert!(
            !holds(&served[0].body, at),
            "a served layout's response names the place {at}"
        );
    }
    assert_eq!(
        asked.pins(),
        [],
        "serving what the layouts hold asked the core"
    );
}
