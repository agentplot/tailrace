//! Spec registry-store, its run against ECR: the conformance suite's
//! store group and the cache manifest's scenarios over the repository the
//! host names in `env.TIMBER_TOOLS_ECR` of its `devenv.local.nix`,
//! reached with the host's Docker credentials, ECR's helper among them,
//! every name they pass prefixed by the run so no run's names meet an
//! earlier one's (design D12, D14). It refuses to run, naming the
//! setting, on a host that names no repository, rather than pass without
//! running.

#![cfg(feature = "tools")]

mod registry;

use std::cell::Cell;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use registry::{cache_entries, unanswered};
use timber::profiles::store::registry::Registry;
use timber_conformance::seams::store::{self, StoreRig};
use timber_conformance::{run, Rig};
use timber_seams::{Answer, Digest, Envelope, Held, Manifest, Name, Put, Store};

/// The setting a host names its ECR repository by.
const SETTING: &str = "TIMBER_TOOLS_ECR";

/// The repository the host names; a panic naming the setting where it
/// names none.
fn repository() -> String {
    match std::env::var(SETTING) {
        Ok(repository) if !repository.trim().is_empty() => repository.trim().to_string(),
        _ => panic!(
            "the ECR run names no repository: set env.{SETTING} in devenv.local.nix to the ECR \
             repository the ECR check pushes to, with the AWS_PROFILE its credential helper \
             reads and the registry host mapped to ecr-login in the Docker configuration's \
             credHelpers"
        ),
    }
}

/// A registry store whose every name carries the run's prefix in the
/// space it is in: an artifact's on its name, so each run lists a
/// repository of its own, a cache entry's on its build identity, and
/// every other name before it.
struct Prefixed {
    registry: Registry,
    prefix: String,
}

impl Prefixed {
    fn named(&self, name: &Name) -> Name {
        if let Some((ecosystem, artifact, version)) = name.as_artifact() {
            return timber_seams::artifact_name(&ecosystem, &self.artifact(&artifact), &version);
        }
        match name.as_str().strip_prefix("cache/") {
            Some(identity) => Name::new(format!("cache/{}{identity}", self.prefix)),
            None => Name::new(format!("{}{name}", self.prefix)),
        }
    }

    fn artifact(&self, name: &timber_seams::EcosystemName) -> timber_seams::EcosystemName {
        timber_seams::EcosystemName::new(format!("{}{name}", self.prefix))
    }
}

impl Store for Prefixed {
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        self.registry.put_bytes(bytes)
    }
    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        self.registry.put_artifact(manifest)
    }
    fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
        self.registry.attach(subject, attestation)
    }
    fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>> {
        self.registry.claims_of(digest)
    }
    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
        self.registry.name(&self.named(name), to, expected)
    }
    fn resolve(&self, name: &Name) -> Answer<Digest> {
        self.registry.resolve(&self.named(name))
    }
    fn read(&self, digest: &Digest) -> Answer<Held> {
        self.registry.read(digest)
    }
    fn versions(
        &self,
        ecosystem: &timber_seams::Ecosystem,
        name: &timber_seams::EcosystemName,
    ) -> Answer<Vec<timber_seams::Version>> {
        self.registry.versions(ecosystem, &self.artifact(name))
    }
}

/// The store group's rig over the host's ECR repository, a scenario's
/// names apart from every other's, and a store at a loopback port
/// nothing listens on, which [`StoreRig::unreachable`] makes the rig's.
struct EcrRig {
    live: Prefixed,
    dead: Registry,
    down: Cell<bool>,
}

impl EcrRig {
    fn new(repository: &str, run: &str, scenario: usize) -> EcrRig {
        EcrRig {
            live: Prefixed {
                registry: Registry::new(repository).expect("the repository the host names"),
                prefix: format!("{run}-{scenario}-"),
            },
            dead: Registry::new(&format!("127.0.0.1:{}/store", unanswered())).unwrap(),
            down: Cell::new(false),
        }
    }
}

impl Rig for EcrRig {
    fn profile(&self) -> String {
        self.live.registry.source().profile.clone()
    }
}

impl StoreRig for EcrRig {
    fn store(&self) -> &dyn Store {
        if self.down.get() {
            &self.dead
        } else {
            &self.live
        }
    }

    fn unreachable(&self) -> String {
        self.down.set(true);
        ureq::get(&format!("{}tags/list", self.dead.base()))
            .call()
            .expect_err("nothing listens on the port")
            .to_string()
    }

    /// The registry's bound: its tag reads may answer a moved name's
    /// former place for that long (C.1, ADR 0015).
    fn settles_within(&self) -> Duration {
        self.live.registry.bound()
    }

    /// The registry's word: its names take no condition, ECR's tag put
    /// among them (C.1, ADR 0015).
    fn names_conditional(&self) -> bool {
        self.live.registry.names_conditional()
    }
}

/// A prefix no earlier run's names carry.
fn run_prefix() -> String {
    let started = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("a clock after 1970")
        .as_secs();
    format!("run{started}-{}", std::process::id())
}

#[test]
fn ecr_keeps_the_stores_contract() {
    let repository = repository();
    let run_prefix = run_prefix();
    let scenario = Cell::new(0);
    let report = run(store::GROUP, &store::scenarios(), &|| {
        scenario.set(scenario.get() + 1);
        Box::new(EcrRig::new(&repository, &run_prefix, scenario.get()))
    });
    report.assert();
    assert_eq!(
        report.profile,
        format!("OCI registry {repository}"),
        "the report names {}",
        report.profile
    );
    assert_eq!(report.ran, store::scenarios().len());
    let not: Vec<&str> = report.not_applicable.iter().map(|n| n.scenario).collect();
    assert_eq!(not, ["a name moved from where it is not"], "{report}");
}

/// What the run records in ADR 0023 (design D8, D13): an artifact's name
/// made at an index in the repository the templates make for it, with
/// whether ECR took each cross-repository mount, and a put into a
/// repository no template covers, a sibling of the host's, refused in
/// ECR's words naming it. Run with `-- --nocapture` to read both.
#[test]
fn ecr_mounts_into_an_artifacts_repository_and_refuses_one_nobody_made() {
    let repository = repository();
    let prefix = run_prefix();
    let registry = Registry::new(&repository).expect("the repository the host names");
    let meta = timber_seams::ArtifactMeta::new(
        timber_seams::Ecosystem::new("file"),
        timber_seams::EcosystemName::new(format!("{prefix}-site")),
        timber_seams::Version::new("1.0.0"),
    )
    .unwrap();
    let layer = registry
        .put_bytes(format!("{prefix} the site").as_bytes())
        .unwrap()
        .digest;
    let artifact = registry
        .put_artifact(&Manifest::Artifact {
            meta: meta.clone(),
            layers: vec![layer],
        })
        .unwrap()
        .digest;
    let index = timber_seams::IndexMeta::new(
        timber_seams::ComponentName::new(timber_seams::REPOSITORY_COMPONENT),
        meta.version().clone(),
        vec![timber_seams::Artifact {
            digest: artifact,
            meta: meta.clone(),
        }],
    )
    .unwrap();
    let index = registry
        .put_artifact(&Manifest::Index(index))
        .unwrap()
        .digest;
    let name = timber_seams::artifact_name(meta.ecosystem(), meta.name(), meta.version());
    registry
        .name(&name, &index, None)
        .expect("the artifact name's repository, which a template covers");
    assert_eq!(registry.resolve(&name), Ok(index));
    let mounts = registry.mounts();
    let taken = mounts.iter().filter(|(_, taken)| *taken).count();
    println!(
        "ecr: {taken} of {} cross-repository mounts were taken, into {:?}",
        mounts.len(),
        mounts
            .iter()
            .map(|(r, _)| r)
            .collect::<std::collections::BTreeSet<_>>()
    );

    let (host, path) = repository.split_once('/').unwrap();
    let uncovered = format!("{host}/{path}-{prefix}-uncovered");
    let nobodys = Registry::new(&uncovered).unwrap();
    match nobodys.put_bytes(b"bytes for a repository nobody made") {
        Err(timber_seams::Not::Refused(r)) => {
            let said = r.said.unwrap_or_default();
            println!("ecr: a repository no template covers answered: {said}");
            assert!(
                said.contains(&uncovered),
                "the refusal names no repository: {said}"
            );
        }
        other => panic!(
            "{uncovered} was written: a template covers it, and none may for this run: {other:?}"
        ),
    }
}

#[test]
fn ecr_names_a_cache_entry_through_a_cache_manifest() {
    let repository = repository();
    let registry = Registry::new(&repository)
        .expect("the repository the host names")
        .bound_for("moon", false);
    let within = registry.bound();
    cache_entries(
        &Prefixed {
            registry,
            prefix: format!("{}-cache-", run_prefix()),
        },
        within,
    );
}
