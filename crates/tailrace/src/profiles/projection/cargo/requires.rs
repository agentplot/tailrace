//! requires (140; design D8): what a held crate requires of the family,
//! read from its own normalized manifest: each dependency, normal or
//! build, a target's among them, optional or not, whose registry is a
//! family repository's served Cargo address, by the crate it names and
//! the requirement it writes, each name and range once. A dev dependency
//! is the package's own and never resolved for a consumer of it, and a
//! dependency of crates.io or another registry is Cargo's to resolve
//! there.

use tailrace_seams::{Answer, Artifact, EcosystemName, Not, Range, Requirement};

use super::render::names_artifact;
use super::{crate_file, index, CargoProjection};

/// The tables of a manifest that hold the dependencies a consumer of the
/// crate resolves.
const RESOLVED: [&str; 2] = ["dependencies", "build-dependencies"];

impl CargoProjection {
    /// 140: the family's names and ranges `held`'s crate requires, read
    /// through the store and checked as render checks it; unreadable
    /// naming its digest where its manifest cannot be read.
    pub(super) fn required(&self, held: &Artifact) -> Answer<Vec<Requirement>> {
        let (_, bytes) = self.crate_bytes(held)?;
        let unreadable = |said: String| {
            Not::unreadable(
                &CargoProjection::source(),
                held.digest.to_string(),
                Some(said),
            )
        };
        let manifest = crate_file::manifest(&bytes).map_err(unreadable)?;
        names_artifact(&manifest, held)?;
        let table = &manifest.table;
        let mut tables = vec![table];
        if let Some(targets) = table.get("target") {
            let targets = targets
                .as_table()
                .ok_or_else(|| unreadable("target is not a table".into()))?;
            for (cfg, target) in targets {
                tables.push(
                    target
                        .as_table()
                        .ok_or_else(|| unreadable(format!("target.{cfg} is not a table")))?,
                );
            }
        }
        let mut required: Vec<Requirement> = Vec::new();
        for table in tables {
            for kind in RESOLVED {
                let Some(declared) = table.get(kind) else {
                    continue;
                };
                let declared = declared
                    .as_table()
                    .ok_or_else(|| unreadable(format!("{kind} is not a table")))?;
                for (key, spec) in declared {
                    let Some(spec) = spec.as_table() else {
                        continue;
                    };
                    let text = |key: &str| spec.get(key).and_then(toml::Value::as_str);
                    if !text("registry-index").is_some_and(index::served) {
                        continue;
                    }
                    let range = text("version").ok_or_else(|| {
                        unreadable(format!("the dependency {key} names no version"))
                    })?;
                    let requirement = Requirement {
                        name: EcosystemName::new(text("package").unwrap_or(key)),
                        range: Range::new(range),
                    };
                    if !required.contains(&requirement) {
                        required.push(requirement);
                    }
                }
            }
        }
        Ok(required)
    }
}

#[cfg(test)]
mod tests {
    use tailrace_fakes::{FakeStore, StoreOp};
    use tailrace_seams::{
        ArtifactMeta, Digest, Ecosystem, Manifest, Projection, Store, Version, Why,
    };

    use super::super::fixture::crate_of;
    use super::super::ECOSYSTEM;
    use super::*;

    const SERVED: &str = "sparse+http://127.0.0.1:7420/agentplot/family/cargo/";
    const ANOTHER: &str = "sparse+http://127.0.0.1:7420/another/repo/cargo/";
    const ELSEWHERE: &str = "sparse+https://registry.example/index/";

    fn meta(name: &str, version: &str) -> ArtifactMeta {
        ArtifactMeta::new(
            Ecosystem::new(ECOSYSTEM),
            EcosystemName::new(name),
            Version::new(version),
        )
        .unwrap()
    }

    /// The crate `fam-a` 1.1.0 whose normalized manifest is `manifest`,
    /// put in `store` as a publish puts an artifact.
    fn held(store: &dyn Store, manifest: &str) -> Artifact {
        let bytes = crate_of("fam-a", "1.1.0", manifest);
        let layer = store.put_bytes(&bytes).unwrap().digest;
        let meta = meta("fam-a", "1.1.0");
        let digest = store
            .put_artifact(&Manifest::Artifact {
                meta: meta.clone(),
                layers: vec![layer],
            })
            .unwrap()
            .digest;
        Artifact { digest, meta }
    }

    fn requirement(name: &str, range: &str) -> Requirement {
        Requirement {
            name: EcosystemName::new(name),
            range: Range::new(range),
        }
    }

    // Requirement: What a crate requires of the family

    #[test]
    fn a_crates_family_dependencies_are_its_served_normal_build_optional_and_target_ones() {
        let store = FakeStore::new();
        let manifest = format!(
            "[package]\nname = \"fam-a\"\nversion = \"1.1.0\"\n\n\
             [dependencies.fam-b]\nversion = \"1.1.0\"\nregistry-index = \"{SERVED}\"\n\n\
             [dependencies.fam-c]\nversion = \"0.3\"\noptional = true\n\
             registry-index = \"{SERVED}\"\n\n\
             [dependencies.json]\nversion = \"1\"\npackage = \"serde_json\"\n\n\
             [dependencies.vendored]\nversion = \"2\"\nregistry-index = \"{ELSEWHERE}\"\n\n\
             [dev-dependencies.fam-e]\nversion = \"1\"\nregistry-index = \"{SERVED}\"\n\n\
             [build-dependencies.base]\nversion = \"^0.4\"\npackage = \"base-c\"\n\
             registry-index = \"{ANOTHER}\"\n\n\
             [target.\"cfg(unix)\".dependencies.fam-d]\nversion = \"2\"\n\
             registry-index = \"{SERVED}\"\n\n\
             [target.\"cfg(windows)\".dependencies.fam-d]\nversion = \"2\"\n\
             registry-index = \"{SERVED}\"\n"
        );
        let artifact = held(&store, &manifest);
        let before = store.calls().len();
        let projection = CargoProjection::new("/checkout", Box::new(store.clone()));
        assert_eq!(
            projection.requires(&artifact).unwrap(),
            [
                requirement("fam-b", "1.1.0"),
                requirement("fam-c", "0.3"),
                requirement("base-c", "^0.4"),
                requirement("fam-d", "2"),
            ]
        );
        assert!(
            store.calls()[before..]
                .iter()
                .all(|c| c.op == StoreOp::Read),
            "requires wrote to the store"
        );
    }

    #[test]
    fn a_crate_requiring_only_crates_io_requires_nothing_of_the_family() {
        let store = FakeStore::new();
        let artifact = held(
            &store,
            "[package]\nname = \"fam-a\"\nversion = \"1.1.0\"\n\n\
             [dependencies.serde]\nversion = \"1\"\n",
        );
        let projection = CargoProjection::new("/checkout", Box::new(store));
        assert_eq!(projection.requires(&artifact).unwrap(), []);
    }

    #[test]
    fn a_crate_that_does_not_read_is_unreadable_naming_its_digest() {
        let store = FakeStore::new();
        let artifact = held(&store, "");
        let projection = CargoProjection::new("/checkout", Box::new(store));
        match projection.requires(&artifact) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, CargoProjection::source());
                assert_eq!(u.what, artifact.digest.to_string());
                assert_eq!(
                    u.said.as_deref(),
                    Some("the crate's Cargo.toml declares no [package]")
                );
            }
            other => panic!("answered {other:?}"),
        }
    }

    #[test]
    fn bytes_of_another_digest_are_refused_naming_both() {
        let store = FakeStore::new();
        let artifact = held(&store, "[package]\nname = \"fam-a\"\nversion = \"1.1.0\"\n");
        let Ok(tailrace_seams::Held::Manifest(Manifest::Artifact { layers, .. })) =
            store.read(&artifact.digest)
        else {
            panic!("the artifact's manifest is held");
        };
        store.corrupt(&layers[0], b"other bytes");
        let projection = CargoProjection::new("/checkout", Box::new(store));
        match projection.requires(&artifact) {
            Err(Not::Refused(r)) => assert_eq!(
                r.why,
                Why::OtherBytes {
                    expected: layers[0].clone(),
                    answered: Digest::of(b"other bytes"),
                }
            ),
            other => panic!("answered {other:?}"),
        }
    }
}
