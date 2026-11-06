//! Spec clan-push-target, its run against one machine (design D11): the
//! machine the host names in `env.TIMBER_TOOLS_FLEET` of its
//! `devenv.local.nix`, as `<checkout>#<machine>`, where `<checkout>` is a
//! clan fleet's git checkout whose `HEAD~1` and `HEAD` configure
//! `<machine>` differently, and `<machine>` a throwaway machine the host
//! holds the key to and may switch at will: never one in use. The host's
//! Nix builds both revisions' toplevels into its store; each is
//! published as a version into a layout in the test's directory, its
//! closure file packed by the file projection, the index naming its
//! revision; the target group runs over the profile with the installed
//! clan; then A, B and A again are applied, current, report, the marker
//! and what the machine runs checked after each, and the marker removed
//! leaves current unread. It reaches the machine with the host's own key
//! file, so it runs by hand under `check-fleet` and in no gate task (rule
//! core.38), and it refuses to run, naming the setting, on a host that
//! names no machine, rather than pass without running.

#![cfg(feature = "tools")]

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::rc::Rc;

use serde_json::{json, Value};
use tempfile::TempDir;
use timber::profiles::projection::file::FileProjection;
use timber::profiles::store::layout::Layout;
use timber::profiles::target::nix_fleet::push::{self, ClanPush};
use timber_conformance::seams::target::{self, TargetRig};
use timber_conformance::Rig;
use timber_core::paths_hold;
use timber_seams::{
    Answer, Artifact, BuildId, ChannelId, ChannelName, Component, ComponentName, Digest, Ecosystem,
    EcosystemName, Envelope, Environment, EnvironmentName, Held, IndexMeta, Manifest, Name, Not,
    Output, OutputName, PackAt, Projection, Put, RepositoryName, Revision, Store, Target, Verified,
    Version,
};

/// The setting a host names its fleet's checkout and throwaway machine by.
const SETTING: &str = "TIMBER_TOOLS_FLEET";

/// The machine's component and its one output, a closure file.
const COMPONENT: &str = "timber-tools-machine";

/// The repository the versions are published from, as the marker's path
/// names it, so no marker of the run meets one of the fleet's own.
fn repository() -> RepositoryName {
    RepositoryName {
        owner: "timber-tools".into(),
        name: "fleet".into(),
    }
}

/// The checkout and the machine the host names; a panic naming the
/// setting where it names none.
fn setting() -> (PathBuf, String) {
    let named = std::env::var(SETTING).unwrap_or_default();
    match named.trim().rsplit_once('#') {
        Some((checkout, machine)) if !checkout.is_empty() && !machine.is_empty() => {
            (PathBuf::from(checkout), machine.to_string())
        }
        _ => panic!(
            "the fleet run names no machine: set env.{SETTING} in devenv.local.nix to \
             `<checkout>#<machine>`, a clan fleet's checkout whose HEAD~1 and HEAD configure \
             <machine> differently, and a throwaway machine this host holds the key to and may \
             switch at will"
        ),
    }
}

/// `program`'s stdout where it exits 0, else a panic with its words.
fn succeeds(program: &mut Command) -> String {
    let ran = program
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("{program:?} could not be started: {e}"));
    assert!(
        ran.status.success(),
        "{program:?} exited {}: {}{}",
        ran.status,
        String::from_utf8_lossy(&ran.stdout),
        String::from_utf8_lossy(&ran.stderr)
    );
    String::from_utf8(ran.stdout).expect("the output is UTF-8")
}

/// `script` run on `machine` by `sh` through `clan ssh`, as the profile
/// reaches it; its stdout and stderr, whatever clan exits with, since
/// clan's ssh answers no exit of the command it ran.
fn on_machine(checkout: &Path, machine: &str, script: &str) -> (String, String) {
    let ran = Command::new("clan")
        .args(["ssh", machine, "--flake"])
        .arg(checkout)
        .args(["--host-key-check", "strict", "-c", "sh", "-c", script])
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("clan could not be started: {e}"));
    (
        String::from_utf8_lossy(&ran.stdout).into_owned(),
        String::from_utf8_lossy(&ran.stderr).trim().to_string(),
    )
}

/// The marker of `environment` on its machine, as the shell there names
/// it.
fn marker_path(environment: &str) -> String {
    let repository = repository();
    format!(
        "\"${{XDG_STATE_HOME:-$HOME/.local/state}}\"/timber/markers/{}/{}/{COMPONENT}/{environment}.json",
        repository.owner, repository.name
    )
}

/// The toplevel of `machine` at `revision` of the fleet at `checkout`,
/// built by the host's Nix into its store, under the first configuration
/// that names it.
fn toplevel(checkout: &Path, revision: &str, machine: &str) -> String {
    let mut said = Vec::new();
    for configurations in ["nixosConfigurations", "darwinConfigurations"] {
        let installable = format!(
            "git+file://{}?rev={revision}#{configurations}.{machine}.config.system.build.toplevel",
            checkout.display()
        );
        let ran = Command::new("nix")
            .args(["--extra-experimental-features", "nix-command flakes"])
            .args(["build", "--no-link", "--print-out-paths", &installable])
            .stdin(Stdio::null())
            .output()
            .unwrap();
        if ran.status.success() {
            return String::from_utf8(ran.stdout).unwrap().trim().to_string();
        }
        said.push(String::from_utf8_lossy(&ran.stderr).trim().to_string());
    }
    panic!(
        "no toplevel of {machine} at {revision}: {}",
        said.join("\n")
    )
}

/// The closure file naming `root`, written under `dir`, packed by the
/// file projection at `version` and put in `store` as a publish puts it:
/// its bytes, its artifact, and the version's index naming it, the
/// revision it was published from and the repository; the version
/// verified, as the core makes it after its verify (42).
fn publish(store: &Layout, dir: &Path, version: &str, revision: &str, root: &str) -> Verified {
    let file = dir.join(format!("closure-{version}"));
    fs::write(&file, format!("{root}\n")).unwrap();
    let output = Output {
        name: OutputName::new(COMPONENT),
        files: vec![file.display().to_string()],
    };
    let component = Component {
        name: ComponentName::new(COMPONENT),
        outputs: vec![output.name.clone()],
        paths: vec!["machines/".into()],
        prefix: format!("{COMPONENT}-v"),
    };
    let holds = paths_hold(&component.paths).unwrap();
    let version = Version::new(version);
    let packed = FileProjection::new()
        .pack(
            &output,
            &BuildId::new(revision),
            Some(&PackAt {
                version: &version,
                component: &component,
                outputs: std::slice::from_ref(&output),
                holds: &holds,
            }),
        )
        .unwrap_or_else(|not| panic!("the closure file packed: {not:?}"));
    let meta = packed.meta.expect("the file projection names its artifact");
    let layer = store.put_bytes(&packed.bytes).unwrap().digest;
    let digest = store
        .put_artifact(&Manifest::Artifact {
            meta: meta.clone(),
            layers: vec![layer],
        })
        .unwrap()
        .digest;
    let index = IndexMeta::new(
        component.name.clone(),
        version.clone(),
        vec![Artifact { digest, meta }],
    )
    .unwrap()
    .published_from(Revision::new(revision), Some(repository()));
    let index = store.put_artifact(&Manifest::Index(index)).unwrap().digest;
    Verified::new(index, version)
}

/// A layout that answers a read of a digest it was told stands for
/// another with what that other holds, so the suite can hand the target a
/// version at a digest of its own choosing.
#[derive(Clone)]
struct Aliased {
    layout: PathBuf,
    aliases: Rc<RefCell<BTreeMap<Digest, Digest>>>,
}

impl Aliased {
    fn store(&self) -> Layout {
        Layout::open(&self.layout).expect("the layout the test published into")
    }
}

impl Store for Aliased {
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        self.store().put_bytes(bytes)
    }
    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        self.store().put_artifact(manifest)
    }
    fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
        self.store().attach(subject, attestation)
    }
    fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>> {
        self.store().claims_of(digest)
    }
    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
        self.store().name(name, to, expected)
    }
    fn resolve(&self, name: &Name) -> Answer<Digest> {
        self.store().resolve(name)
    }
    fn read(&self, digest: &Digest) -> Answer<Held> {
        let to = self.aliases.borrow().get(digest).cloned();
        self.store().read(to.as_ref().unwrap_or(digest))
    }
    fn versions(&self, ecosystem: &Ecosystem, name: &EcosystemName) -> Answer<Vec<Version>> {
        self.store().versions(ecosystem, name)
    }
}

/// The fleet the run stands on: the checkout and the machine named, both
/// revisions' roots built, both versions published, and the target.
struct Fleet {
    _dir: TempDir,
    checkout: PathBuf,
    machine: String,
    roots: [String; 2],
    versions: [Verified; 2],
    store: Aliased,
    target: ClanPush,
}

impl Fleet {
    fn new() -> Fleet {
        let (checkout, machine) = setting();
        let revision = |name: &str| {
            succeeds(Command::new("git").arg("-C").arg(&checkout).args([
                "rev-parse",
                "--verify",
                &format!("{name}^{{commit}}"),
            ]))
            .trim()
            .to_string()
        };
        let revisions = [revision("HEAD~1"), revision("HEAD")];
        let roots = revisions.clone().map(|r| toplevel(&checkout, &r, &machine));
        assert_ne!(
            roots[0],
            roots[1],
            "HEAD~1 and HEAD of {} configure {machine} alike",
            checkout.display()
        );
        let dir = TempDir::new().unwrap();
        let layout = dir.path().join("layout");
        let made = Layout::create(&layout).expect("a layout made");
        let versions = [
            publish(&made, dir.path(), "0.1.0", &revisions[0], &roots[0]),
            publish(&made, dir.path(), "0.2.0", &revisions[1], &roots[1]),
        ];
        let store = Aliased {
            layout,
            aliases: Rc::default(),
        };
        let target = ClanPush::new(Box::new(store.clone()), &checkout, Ok(repository()));
        Fleet {
            _dir: dir,
            checkout,
            machine,
            roots,
            versions,
            store,
            target,
        }
    }

    fn environment(&self, name: &str) -> Environment {
        Environment {
            name: EnvironmentName::new(name),
            channel: ChannelId::new(
                Some(ComponentName::new(COMPONENT)),
                ChannelName::new("stable"),
            ),
            waits: true,
            settings: json!({ "machine": self.machine, "closure": COMPONENT }),
        }
    }

    /// What the machine runs, and the marker of `environment` as JSON or
    /// `None`, read on it apart from the profile.
    fn machine_says(&self, environment: &str) -> (String, Option<Value>) {
        let script = format!(
            "readlink /run/current-system; m={}; if [ -e \"$m\" ]; then cat \"$m\"; fi; echo end",
            marker_path(environment)
        );
        let (out, err) = on_machine(&self.checkout, &self.machine, &script);
        let lines: Vec<&str> = out.lines().map(|l| l.trim_end_matches('\r')).collect();
        assert_eq!(lines.last(), Some(&"end"), "{out}{err}");
        let runs = lines[0].to_string();
        let marker = (lines.len() > 2).then(|| serde_json::from_str(lines[1]).unwrap());
        (runs, marker)
    }
}

/// The target group's rig over the profile and the machine: each
/// environment is one of its own on the machine, the unreadable one a
/// machine the fleet does not name, whose words the rig reads from clan
/// itself, and the foreign settings add a `host`.
struct FleetRig {
    fleet: Rc<Fleet>,
    environments: Cell<u32>,
}

impl FleetRig {
    const NO_SUCH: &'static str = "timber-tools-no-such-machine";

    fn next(&self) -> Environment {
        self.environments.set(self.environments.get() + 1);
        self.fleet
            .environment(&format!("group-{}", self.environments.get()))
    }
}

impl Rig for FleetRig {
    fn profile(&self) -> String {
        push::PROFILE.into()
    }
}

impl TargetRig for FleetRig {
    fn target(&self) -> &dyn Target {
        &self.fleet.target
    }

    /// Version A, read at `digest`.
    fn verified(&self, digest: &Digest) -> Verified {
        let a = &self.fleet.versions[0];
        let mut aliases = self.fleet.store.aliases.borrow_mut();
        aliases.insert(digest.clone(), a.index().clone());
        Verified::new(digest.clone(), a.version().clone())
    }

    fn environment(&self, _head: Option<&Digest>) -> Environment {
        self.next()
    }

    fn unreadable_environment(&self) -> (Environment, String) {
        let mut environment = self.next();
        environment.settings["machine"] = json!(Self::NO_SUCH);
        let (_, words) = on_machine(&self.fleet.checkout, Self::NO_SUCH, "true");
        assert!(!words.is_empty(), "clan reached {}", Self::NO_SUCH);
        (environment, words)
    }

    fn foreign_settings(&self) -> (Environment, String) {
        let mut environment = self.next();
        environment.settings["host"] = json!("root@192.0.2.1");
        (environment, "host".into())
    }
}

#[test]
fn the_target_group_passes_and_a_machine_runs_a_then_b_then_a_with_its_marker_after_each() {
    let fleet = Rc::new(Fleet::new());

    let report = timber_conformance::run(target::GROUP, &target::scenarios(), &|| {
        Box::new(FleetRig {
            fleet: Rc::clone(&fleet),
            environments: Cell::new(0),
        }) as Box<dyn TargetRig>
    });
    report.assert();
    let inapplicable: Vec<&str> = report.not_applicable.iter().map(|n| n.scenario).collect();
    assert_eq!(
        inapplicable,
        ["applied where no marker is held"],
        "{report}"
    );

    let name = "fleet";
    let environment = fleet.environment(name);
    for n in [0, 1, 0] {
        let (version, root) = (&fleet.versions[n], &fleet.roots[n]);
        fleet
            .target
            .apply(&environment, version)
            .unwrap_or_else(|not| panic!("apply of {}: {not:?}", version.version()));
        assert_eq!(
            fleet.target.current(&environment),
            Ok(version.index().clone())
        );
        let running = fleet.target.report(&environment).unwrap();
        assert_eq!(&running.digest, version.index());
        let (runs, marker) = fleet.machine_says(name);
        assert_eq!(&runs, root, "the machine runs another closure");
        let marker = marker.expect("the machine holds the marker");
        assert_eq!(marker["index"], json!(version.index().as_str()));
        assert_eq!(marker["version"], json!(version.version().as_str()));
        assert_eq!(marker["closure"], json!(root));
        assert_eq!(marker["at"], json!(running.converged_at));
    }

    on_machine(
        &fleet.checkout,
        &fleet.machine,
        &format!("rm -f {}", marker_path(name)),
    );
    match fleet.target.current(&environment) {
        Err(Not::Unreadable(u)) => {
            let said = u.said.unwrap_or_default();
            assert!(said.contains("holds no marker of timber's"), "{said}");
        }
        other => panic!("current with the marker gone answered {other:?}"),
    }
}
