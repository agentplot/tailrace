//! Spec moon-cache-projection, the projection's contract: the
//! conformance suite's projection group over the moon cache, its
//! checkout and its layout store in a temporary directory (design D9).
//!
//! Only the group's two scenarios of a projection that packs cache
//! entries run, the pinned scenarios, whose lockfile one is not
//! applicable, since moon's cache has no lockfile, and the manifest
//! scenarios, whose two of a manifest are not applicable beside it, since
//! moon's cache has no manifest either. The group's other
//! scenarios each pack or render at a
//! version, which the moon cache answers absent naming artifacts, since
//! it packs no artifact (53), so they are not run over it: a departure
//! from core.33, which asks every profile to pass the one suite, as
//! `tests/projections.rs` states its own. The unreadable answers of its
//! pack and render are its unit tests'.

use std::fs;
use std::path::PathBuf;

use tailrace::profiles::projection::moon_cache::MoonCache;
use tailrace::profiles::store::layout::Layout;
use tailrace_conformance::seams::projection::{self, ProjectionRig};
use tailrace_conformance::{run, Rig, Scenario};
use tailrace_seams::{
    Artifact, ArtifactMeta, Digest, Ecosystem, EcosystemName, Output, OutputName, Pinned,
    Projection, Range, Requirement, Store, Version,
};
use tempfile::TempDir;

/// The scenarios of a projection that packs cache entries, of pinned,
/// and of requires and satisfying.
const CACHE: [&str; 7] = [
    "a cache entry packed with no version",
    "a layout of the build identities given",
    "pinned as the lockfile spells it",
    "no lockfile, absent",
    "required as the manifest spells it",
    "satisfied by the ecosystem's own ordering",
    "no manifest, none",
];

/// The task hashes the group packs at.
const HASHES: [&str; 2] = ["b7e1", "c3d4"];

fn cache_scenarios() -> Vec<Scenario<dyn ProjectionRig>> {
    projection::scenarios()
        .into_iter()
        .filter(|s| CACHE.contains(&s.name))
        .collect()
}

/// A checkout holding moon's archive at each hash the group packs at,
/// and a layout store holding their bytes.
struct MoonCacheRig {
    _dir: TempDir,
    checkout: PathBuf,
    projection: MoonCache,
}

impl MoonCacheRig {
    fn new() -> MoonCacheRig {
        let dir = TempDir::new().unwrap();
        let checkout = dir.path().join("checkout");
        let outputs = checkout.join(".moon/cache/outputs");
        fs::create_dir_all(&outputs).unwrap();
        let layout = Layout::create(dir.path().join("layout")).unwrap();
        for hash in HASHES {
            let bytes = format!("moon's archive of {hash}").into_bytes();
            fs::write(outputs.join(format!("{hash}.tar.gz")), &bytes).unwrap();
            layout.put_bytes(&bytes).unwrap();
        }
        MoonCacheRig {
            projection: MoonCache::new(&checkout, Box::new(layout)),
            checkout,
            _dir: dir,
        }
    }
}

impl Rig for MoonCacheRig {
    fn profile(&self) -> String {
        tailrace::profiles::projection::moon_cache::ECOSYSTEM.into()
    }
}

impl ProjectionRig for MoonCacheRig {
    fn projection(&self) -> &dyn Projection {
        &self.projection
    }

    fn output(&self) -> Output {
        Output {
            name: OutputName::new("site"),
            files: vec![],
        }
    }

    /// None: the moon cache renders no artifacts.
    fn artifacts(&self, _: usize) -> Vec<Artifact> {
        Vec::new()
    }

    fn serves(&self) -> bool {
        false
    }

    fn caches(&self) -> bool {
        true
    }

    fn has_lockfile(&self) -> bool {
        false
    }

    /// The checkout, which holds moon's outputs and no lockfile.
    fn checkout(&self, _: bool) -> (String, Vec<Pinned>) {
        (self.checkout.display().to_string(), Vec::new())
    }

    fn has_manifest(&self) -> bool {
        false
    }

    /// An artifact named in the moon cache's ecosystem, which requires
    /// nothing, since moon's cache has no manifest.
    fn held(&self) -> (Artifact, Vec<Requirement>) {
        let held = Artifact {
            digest: Digest::of(b"moon's archive of b7e1"),
            meta: ArtifactMeta::new(
                Ecosystem::new(tailrace::profiles::projection::moon_cache::ECOSYSTEM),
                EcosystemName::new("site"),
                Version::new("1.0.0"),
            )
            .unwrap(),
        };
        (held, Vec::new())
    }

    /// None: moon's cache has no manifest to spell a range in.
    fn ranged(&self) -> (Range, Vec<Version>, Vec<Version>) {
        (Range::new(""), Vec::new(), Vec::new())
    }

    /// The outputs directory replaced by a file, and the operating
    /// system's words for reading an archive through it.
    fn unreachable(&self) -> String {
        let outputs = self.checkout.join(".moon/cache/outputs");
        fs::remove_dir_all(&outputs).unwrap();
        fs::write(&outputs, b"not a directory").unwrap();
        fs::read(outputs.join("b7e1.tar.gz"))
            .unwrap_err()
            .to_string()
    }
}

#[test]
fn the_cache_entry_scenarios_over_the_moon_cache() {
    let report = run(projection::GROUP, &cache_scenarios(), &|| {
        Box::new(MoonCacheRig::new())
    });
    assert_eq!(
        (report.profile.as_str(), report.ran),
        ("moon-cache", CACHE.len())
    );
    report.assert();
    let inapplicable: Vec<(&str, &str)> = report
        .not_applicable
        .iter()
        .map(|n| (n.scenario, n.why.as_str()))
        .collect();
    assert_eq!(
        inapplicable,
        [
            (
                "pinned as the lockfile spells it",
                "the ecosystem has no lockfile"
            ),
            (
                "required as the manifest spells it",
                "the ecosystem has no manifest"
            ),
            (
                "satisfied by the ecosystem's own ordering",
                "the ecosystem has no manifest"
            )
        ],
        "{report}"
    );
}
