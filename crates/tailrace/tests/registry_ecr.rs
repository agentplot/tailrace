//! Spec registry-store, its run against ECR: the conformance suite's
//! store group over the repository the host names in
//! `env.TAILRACE_TOOLS_ECR` of its `devenv.local.nix`, reached with the
//! host's Docker credentials, ECR's helper among them, every name the
//! group passes prefixed by the run so no run's names meet an earlier
//! one's (design D12). It fails naming the setting on a host that names
//! no repository, rather than pass without running.

#![cfg(feature = "tools")]

mod registry;

use std::cell::{Cell, RefCell};
use std::net::TcpListener;
use std::time::{SystemTime, UNIX_EPOCH};

use registry::free_port;
use tailrace::profiles::store::registry::Registry;
use tailrace_conformance::seams::store::{self, StoreRig};
use tailrace_conformance::{run, Rig};
use tailrace_seams::{Answer, Digest, Envelope, Held, Manifest, Name, Put, Store};

/// The setting a host names its ECR repository by.
const SETTING: &str = "TAILRACE_TOOLS_ECR";

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

/// A registry store whose every name is the run's prefix and the name.
struct Prefixed {
    registry: Registry,
    prefix: String,
}

impl Prefixed {
    fn named(&self, name: &Name) -> Name {
        Name::new(format!("{}{name}", self.prefix))
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
}

/// The store group's rig over the host's ECR repository, a scenario's
/// names apart from every other's, and a store at a loopback port
/// nothing listens on, which [`StoreRig::unreachable`] makes the rig's.
struct EcrRig {
    live: Prefixed,
    dead: Registry,
    held: RefCell<Option<TcpListener>>,
    down: Cell<bool>,
}

impl EcrRig {
    fn new(repository: &str, run: &str, scenario: usize) -> EcrRig {
        let (port, held) = free_port();
        EcrRig {
            live: Prefixed {
                registry: Registry::new(repository).expect("the repository the host names"),
                prefix: format!("{run}-{scenario}-"),
            },
            dead: Registry::new(&format!("127.0.0.1:{port}/store")).unwrap(),
            held: RefCell::new(Some(held)),
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
        self.held.borrow_mut().take();
        self.down.set(true);
        ureq::get(&format!("{}tags/list", self.dead.base()))
            .call()
            .expect_err("nothing listens on the port")
            .to_string()
    }
}

#[test]
fn ecr_keeps_the_stores_contract() {
    let repository = repository();
    let started = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("a clock after 1970")
        .as_secs();
    let run_prefix = format!("run{started}-{}", std::process::id());
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
    assert_eq!(report.not_applicable, [], "{report}");
}
