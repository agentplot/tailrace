//! Specs file-projection and image-projection: the conformance suite's
//! projection group over the file and image projections, each output a
//! file in a temporary directory of its own (design D4, D5, D7).
//!
//! Only the group's pack scenarios run, the scenario of a projection
//! that packs no cache entries, which neither does, and the pinned
//! scenarios, whose lockfile one is not applicable, since neither
//! ecosystem has a lockfile. Neither projection renders or serves yet, so
//! the render and serve scenarios are not run over them:
//! a departure from core.33, which asks every profile to pass the one
//! suite, stated in design D7. It ends with the unit that builds their
//! render (bolt 5's render under a policy, phase 2's layouts).

mod fixtures;

use std::fs;
use std::path::PathBuf;

use tailrace::profiles::projection::file::FileProjection;
use tailrace::profiles::projection::image::ImageProjection;
use tailrace_conformance::seams::projection::{self, ProjectionRig};
use tailrace_conformance::{run, Rig, Scenario};
use tailrace_seams::{
    Artifact, ArtifactMeta, Digest, Ecosystem, EcosystemName, Output, OutputName, Pinned,
    Projection, Version,
};
use tempfile::TempDir;

/// The scenarios of pack, the only operation these projections build,
/// of a projection that packs no cache entries, and of pinned.
const PACK: [&str; 5] = [
    "packed the same again",
    "a projection that cannot be reached",
    "a projection that packs no cache entries",
    "pinned as the lockfile spells it",
    "no lockfile, absent",
];

/// What the group's report names not applicable over a projection whose
/// ecosystem has no lockfile.
const NO_LOCKFILE: (&str, &str) = (
    "pinned as the lockfile spells it",
    "the ecosystem has no lockfile",
);

fn pack_scenarios() -> Vec<Scenario<dyn ProjectionRig>> {
    projection::scenarios()
        .into_iter()
        .filter(|s| PACK.contains(&s.name))
        .collect()
}

/// A projection, and the one file of the output it packs.
struct FileRig {
    dir: TempDir,
    path: PathBuf,
    ecosystem: &'static str,
    projection: Box<dyn Projection>,
}

impl FileRig {
    fn new(ecosystem: &'static str, bytes: &[u8], projection: Box<dyn Projection>) -> FileRig {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("output");
        fs::write(&path, bytes).unwrap();
        FileRig {
            dir,
            path,
            ecosystem,
            projection,
        }
    }

    fn file() -> FileRig {
        FileRig::new("file", b"<p>hello</p>", Box::new(FileProjection))
    }

    fn image() -> FileRig {
        FileRig::new(
            "image",
            &fixtures::image::archive(),
            Box::new(ImageProjection),
        )
    }
}

impl Rig for FileRig {
    fn profile(&self) -> String {
        self.ecosystem.into()
    }
}

impl ProjectionRig for FileRig {
    fn projection(&self) -> &dyn Projection {
        self.projection.as_ref()
    }

    fn output(&self) -> Output {
        Output {
            name: OutputName::new("site"),
            files: vec![self.path.display().to_string()],
        }
    }

    fn artifacts(&self, n: usize) -> Vec<Artifact> {
        (0..n)
            .map(|i| Artifact {
                digest: Digest::of(format!("site 1.{i}.0").as_bytes()),
                meta: ArtifactMeta::new(
                    Ecosystem::new(self.ecosystem),
                    EcosystemName::new("site"),
                    Version::new(format!("1.{i}.0")),
                )
                .unwrap(),
            })
            .collect()
    }

    fn serves(&self) -> bool {
        false
    }

    fn caches(&self) -> bool {
        false
    }

    fn has_lockfile(&self) -> bool {
        false
    }

    /// The output's directory, which holds no lockfile of any ecosystem.
    fn checkout(&self, _: bool) -> (String, Vec<Pinned>) {
        (self.dir.path().display().to_string(), Vec::new())
    }

    /// The output's file removed, and the operating system's words for
    /// reading it.
    fn unreachable(&self) -> String {
        fs::remove_file(&self.path).unwrap();
        fs::read(&self.path).unwrap_err().to_string()
    }
}

#[test]
fn the_pack_scenarios_over_the_file_projection() {
    let report = run(projection::GROUP, &pack_scenarios(), &|| {
        Box::new(FileRig::file())
    });
    assert_eq!((report.profile.as_str(), report.ran), ("file", PACK.len()));
    report.assert();
    let inapplicable: Vec<(&str, &str)> = report
        .not_applicable
        .iter()
        .map(|n| (n.scenario, n.why.as_str()))
        .collect();
    assert_eq!(inapplicable, [NO_LOCKFILE], "{report}");
}

#[test]
fn the_pack_scenarios_over_the_image_projection() {
    let report = run(projection::GROUP, &pack_scenarios(), &|| {
        Box::new(FileRig::image())
    });
    assert_eq!((report.profile.as_str(), report.ran), ("image", PACK.len()));
    report.assert();
    let inapplicable: Vec<(&str, &str)> = report
        .not_applicable
        .iter()
        .map(|n| (n.scenario, n.why.as_str()))
        .collect();
    assert_eq!(inapplicable, [NO_LOCKFILE], "{report}");
}
