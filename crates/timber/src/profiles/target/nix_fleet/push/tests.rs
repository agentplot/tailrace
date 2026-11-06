use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;

use serde_json::{json, Value};
use tempfile::TempDir;
use timber_conformance::seams::target::{self, TargetRig};
use timber_conformance::Rig;
use timber_fakes::FakeStore;
use timber_seams::{
    Artifact, ArtifactMeta, ChannelId, ChannelName, ComponentName, Ecosystem, EcosystemName,
    Envelope, EnvironmentName, Held, Manifest, Name, Put, Version,
};

use super::scripted::{Asked, Scripted, BEFORE, CLOCK};
use super::*;

const VERSION: &str = "0.4.0";
const COMPONENT: &str = "swancloud-srv";
const MACHINE: &str = "swancloud-srv";

/// The machine's closure root, under this host's store directory.
fn root() -> String {
    format!(
        "{}/0c7yzjm6jy3dxfyb8zd6x0fv1r6l1sbm-nixos-system-swancloud-srv-25.05",
        nix_closure::store_dir()
    )
}

/// Another closure's root.
fn other_root() -> String {
    format!(
        "{}/9kn2wd8q1ycx0l6hzm4fjrb3gv5sap7i-nixos-system-swancloud-srv-25.05",
        nix_closure::store_dir()
    )
}

fn closure_of(roots: &[String]) -> String {
    let mut text = roots.join("\n");
    text.push('\n');
    text
}

fn repository() -> RepositoryName {
    RepositoryName {
        owner: "afterthought".into(),
        name: "swancloud".into(),
    }
}

fn environment(name: &str, settings: Value) -> Environment {
    Environment {
        name: EnvironmentName::new(name),
        channel: ChannelId::new(
            Some(ComponentName::new(COMPONENT)),
            ChannelName::new("stable"),
        ),
        waits: true,
        settings,
    }
}

fn srv() -> Environment {
    environment("srv", json!({ "machine": MACHINE, "closure": COMPONENT }))
}

fn marker_of(environment: &Environment) -> MarkerPath {
    MarkerPath::of(&repository(), environment, &ClanPush::source()).unwrap()
}

/// git run in `dir`, with no configuration of the host's, answering what
/// it printed.
fn git(dir: &Path, args: &[&str]) -> String {
    let mut command = Command::new("git");
    for name in GIT_REPOSITORY_ENV {
        command.env_remove(name);
    }
    let out = command
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "an author")
        .env("GIT_AUTHOR_EMAIL", "author@example.org")
        .env("GIT_COMMITTER_NAME", "an author")
        .env("GIT_COMMITTER_EMAIL", "author@example.org")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

/// A store that answers a read of a digest it was told stands for another
/// with what that other holds, so the suite can hand the target a
/// version at a digest of its own choosing, as the conformance rig of a
/// fake target does.
#[derive(Clone, Default)]
struct Aliased {
    store: FakeStore,
    aliases: Rc<RefCell<BTreeMap<Digest, Digest>>>,
}

impl Store for Aliased {
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        self.store.put_bytes(bytes)
    }
    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        self.store.put_artifact(manifest)
    }
    fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
        self.store.attach(subject, attestation)
    }
    fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>> {
        self.store.claims_of(digest)
    }
    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
        self.store.name(name, to, expected)
    }
    fn resolve(&self, name: &Name) -> Answer<Digest> {
        self.store.resolve(name)
    }
    fn read(&self, digest: &Digest) -> Answer<Held> {
        let to = self.aliases.borrow().get(digest).cloned();
        self.store.read(to.as_ref().unwrap_or(digest))
    }
    fn versions(&self, ecosystem: &Ecosystem, name: &EcosystemName) -> Answer<Vec<Version>> {
        self.store.versions(ecosystem, name)
    }
}

/// A checkout of a fleet with one commit, a store, a scripted fleet whose
/// machine's toplevel evaluates to [`root`], and the target over them.
struct World {
    _dir: TempDir,
    checkout: PathBuf,
    commit: String,
    store: Aliased,
    fleet: Scripted,
    target: ClanPush,
}

impl World {
    fn new() -> World {
        World::of(Ok(repository()))
    }

    fn of(repository: Answer<RepositoryName>) -> World {
        let dir = TempDir::new().unwrap();
        let checkout = dir.path().join("fleet");
        std::fs::create_dir_all(&checkout).unwrap();
        git(&checkout, &["init", "-q", "-b", "main"]);
        std::fs::write(checkout.join("flake.nix"), "{ outputs = _: { }; }\n").unwrap();
        git(&checkout, &["add", "flake.nix"]);
        git(&checkout, &["commit", "-q", "-m", "feat: the fleet"]);
        let commit = git(&checkout, &["rev-parse", "HEAD"]);
        let store = Aliased::default();
        let fleet = Scripted::new(&ClanPush::source());
        fleet.evaluates(MACHINE, &root());
        let target = ClanPush::with_fleet(
            Box::new(store.clone()),
            &checkout,
            repository,
            Box::new(fleet.clone()),
        );
        World {
            _dir: dir,
            checkout,
            commit,
            store,
            fleet,
            target,
        }
    }

    /// Where the revision's worktree is kept.
    fn worktree(&self) -> PathBuf {
        let common = git(
            &self.checkout,
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        );
        PathBuf::from(common)
            .join("timber/fleet")
            .join(&self.commit)
    }

    /// A version of `COMPONENT` published from the checkout's commit,
    /// each of `files` an artifact of the ecosystem it names, packed as
    /// the file projection packs one; and the version verified.
    fn publish(&self, files: &[(&str, &str, &str)]) -> Verified {
        self.publish_from(files, Some(Revision::new(&self.commit)), Some(repository()))
    }

    fn publish_from(
        &self,
        files: &[(&str, &str, &str)],
        revision: Option<Revision>,
        repository: Option<RepositoryName>,
    ) -> Verified {
        let store = &self.store.store;
        let mut artifacts = Vec::new();
        for (ecosystem, name, text) in files {
            let layer = store.put_bytes(text.as_bytes()).unwrap().digest;
            let meta = ArtifactMeta::new(
                Ecosystem::new(*ecosystem),
                EcosystemName::new(*name),
                Version::new(VERSION),
            )
            .unwrap();
            let manifest = Manifest::Artifact {
                meta: meta.clone(),
                layers: vec![layer],
            };
            let digest = store.put_artifact(&manifest).unwrap().digest;
            artifacts.push(Artifact { digest, meta });
        }
        let index = IndexMeta::new(
            ComponentName::new(COMPONENT),
            Version::new(VERSION),
            artifacts,
        )
        .unwrap()
        .with(revision, repository);
        let digest = store.put_artifact(&Manifest::Index(index)).unwrap().digest;
        Verified::new(digest, Version::new(VERSION))
    }

    /// The machine's closure published as its one root.
    fn version(&self) -> Verified {
        self.publish(&[("file", COMPONENT, &closure_of(&[root()]))])
    }
}

fn refused_said<T: std::fmt::Debug>(answer: Answer<T>) -> String {
    match answer {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source, ClanPush::source());
            assert_eq!(r.why, Why::Said);
            r.said.expect("the target's words")
        }
        other => panic!("answered {other:?}, not refused"),
    }
}

fn unreadable<T: std::fmt::Debug>(answer: Answer<T>) -> (String, String) {
    match answer {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, ClanPush::source());
            (u.what, u.said.expect("the words"))
        }
        other => panic!("answered {other:?}, not unreadable"),
    }
}

fn index(n: u8) -> Digest {
    Digest::of(&[b'i', n])
}

// Requirement: The target's table names the machine and its closure, and
// nothing else

#[test]
fn a_host_in_the_table_is_unreadable_at_every_operation_and_nothing_runs() {
    let world = World::new();
    let version = world.version();
    let foreign = environment(
        "srv",
        json!({ "machine": MACHINE, "closure": COMPONENT, "host": "root@192.168.1.172" }),
    );
    let named = "setting host of environment srv of swancloud-srv";
    let (what, said) = unreadable(world.target.apply(&foreign, &version));
    assert_eq!(what, named);
    assert!(said.contains("reads machine and closure"), "{said}");
    assert_eq!(
        unreadable(world.target.desired(&foreign, &|| Ok(index(3)))).0,
        named
    );
    assert_eq!(unreadable(world.target.current(&foreign)).0, named);
    assert_eq!(unreadable(world.target.report(&foreign)).0, named);
    assert!(world.fleet.ran().is_empty(), "{:?}", world.fleet.ran());
}

#[test]
fn no_closure_named_is_unreadable_naming_it_as_required_and_nothing_runs() {
    let world = World::new();
    let version = world.version();
    let (what, said) = unreadable(
        world
            .target
            .apply(&environment("srv", json!({ "machine": MACHINE })), &version),
    );
    assert_eq!(what, "setting closure of environment srv of swancloud-srv");
    assert!(said.contains("required"), "{said}");
    assert!(world.fleet.ran().is_empty());
}

// Requirement: desired is the channel's head

#[test]
fn desired_answers_the_head_it_is_handed_and_an_empty_channels_absence() {
    let world = World::new();
    assert_eq!(world.target.desired(&srv(), &|| Ok(index(3))), Ok(index(3)));
    let empty = || -> Answer<Digest> {
        Err(Not::absent(
            &Source::new(Seam::Keeper, "fake"),
            "the head of the chain of swancloud-srv/stable",
        ))
    };
    assert_eq!(world.target.desired(&srv(), &empty), empty());
    assert!(world.fleet.ran().is_empty());
}

// Requirement: Apply switches the machine to the one root its closure
// artifact names

#[test]
fn a_closure_of_two_roots_is_refused_naming_the_artifact_and_both_and_clan_is_not_run() {
    let world = World::new();
    let version = world.publish(&[("file", COMPONENT, &closure_of(&[root(), other_root()]))]);
    let said = refused_said(world.target.apply(&srv(), &version));
    assert!(
        said.starts_with("the closure file swancloud-srv@0.4.0"),
        "{said}"
    );
    assert!(
        said.contains(&format!("names 2 roots, {}, {}", root(), other_root())),
        "{said}"
    );
    assert!(world.fleet.ran().is_empty(), "{:?}", world.fleet.ran());
}

#[test]
fn no_artifact_of_the_output_is_absent_naming_it_and_the_index_and_nothing_runs() {
    let world = World::new();
    let version = world.publish(&[("file", "notes", &closure_of(&[root()]))]);
    match world.target.apply(&srv(), &version) {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source, ClanPush::source());
            assert_eq!(
                a.what,
                format!(
                    "the file artifact of the output swancloud-srv in the index {} of version \
                     0.4.0",
                    version.index()
                )
            );
        }
        other => panic!("answered {other:?}, not absent"),
    }
    assert!(world.fleet.ran().is_empty(), "{:?}", world.fleet.ran());
}

#[test]
fn an_index_of_another_repository_or_of_no_revision_is_refused_and_nothing_runs() {
    let world = World::new();
    let text = closure_of(&[root()]);
    let files = [("file", COMPONENT, text.as_str())];
    let other = RepositoryName {
        owner: "someone".into(),
        name: "else".into(),
    };
    let elsewhere = world.publish_from(&files, Some(Revision::new(&world.commit)), Some(other));
    let said = refused_said(world.target.apply(&srv(), &elsewhere));
    assert!(
        said.contains(
            "names the publishing repository someone/else, and this checkout is of \
                       afterthought/swancloud"
        ),
        "{said}"
    );
    let unrevised = world.publish_from(&files, None, None);
    let said = refused_said(world.target.apply(&srv(), &unrevised));
    assert!(said.contains("names no revision"), "{said}");
    assert!(world.fleet.ran().is_empty(), "{:?}", world.fleet.ran());
}

#[test]
fn a_revision_the_checkout_lacks_is_absent_naming_it_and_nothing_runs() {
    let world = World::new();
    let text = closure_of(&[root()]);
    let files = [("file", COMPONENT, text.as_str())];
    let missing = "0123456789abcdef0123456789abcdef01234567";
    let version = world.publish_from(&files, Some(Revision::new(missing)), Some(repository()));
    match world.target.apply(&srv(), &version) {
        Err(Not::Absent(a)) => assert!(a.what.starts_with(&format!("revision {missing}")), "{a:?}"),
        other => panic!("answered {other:?}, not absent"),
    }
    assert!(world.fleet.ran().is_empty());
}

#[test]
fn a_checkout_whose_repository_is_unread_answers_so_at_every_operation_and_nothing_runs() {
    let words = Not::absent(
        &Source::history("fleet"),
        "remote of the landing, which neither [landing] nor the repository names",
    );
    let world = World::of(Err(words.clone()));
    let version = world.version();
    assert_eq!(world.target.apply(&srv(), &version), Err(words.clone()));
    assert_eq!(world.target.current(&srv()), Err(words.clone()));
    assert_eq!(world.target.report(&srv()).map(drop), Err(words));
    assert!(world.fleet.ran().is_empty());
}

// Requirement: Apply runs only what the version's closure names

#[test]
fn a_flake_evaluating_to_another_path_is_refused_naming_both_and_no_update_runs() {
    let world = World::new();
    world.fleet.evaluates(MACHINE, &other_root());
    let said = refused_said(world.target.apply(&srv(), &world.version()));
    assert_eq!(
        said,
        format!(
            "the flake at revision {} evaluates the toplevel of swancloud-srv to {}, and \
             version 0.4.0 names {}",
            world.commit,
            other_root(),
            root()
        )
    );
    assert_eq!(world.fleet.ran(), ["eval"]);
}

// Requirement: Apply realises the root from substituters alone

#[test]
fn a_path_no_substituter_holds_is_refused_in_nixs_words_and_clan_is_not_run() {
    let world = World::new();
    let words = format!(
        "error: path '{}' is required, but there is no substituter that can build it",
        root()
    );
    world.fleet.lacks(&root(), &words);
    assert_eq!(
        refused_said(world.target.apply(&srv(), &world.version())),
        words
    );
    assert_eq!(world.fleet.ran(), ["eval", "realise"]);
}

// Requirement: Apply is one update of the machine

#[test]
fn an_update_clan_refuses_answers_its_words_verbatim() {
    let world = World::new();
    let words = "error: switch-to-configuration failed: unit sshd.service failed to start";
    world.fleet.update_fails(words);
    let (what, said) = unreadable(world.target.apply(&srv(), &world.version()));
    assert_eq!(said, words);
    assert_eq!(what, "clan machines update swancloud-srv");
    assert_eq!(world.fleet.ran(), ["eval", "realise", "update"]);
}

#[test]
fn a_converged_update_runs_once_from_the_revisions_worktree_and_reuses_it() {
    let world = World::new();
    let version = world.version();
    assert_eq!(world.target.apply(&srv(), &version), Ok(()));
    assert_eq!(
        world.fleet.ran(),
        ["eval", "realise", "update", "mark", "read"]
    );
    let worktree = world.worktree();
    assert_eq!(git(&worktree, &["rev-parse", "HEAD"]), world.commit);
    let asked = world.fleet.asked();
    assert_eq!(
        asked[..3],
        [
            Asked::Eval {
                flake: worktree.clone(),
                machine: MACHINE.into()
            },
            Asked::Realise(root()),
            Asked::Update {
                flake: worktree.clone(),
                machine: MACHINE.into()
            },
        ]
    );
    assert_eq!(world.target.apply(&srv(), &version), Ok(()));
    assert_eq!(
        world.fleet.ran().iter().filter(|r| **r == "update").count(),
        2
    );
    let worktrees = git(&world.checkout, &["worktree", "list", "--porcelain"]);
    assert_eq!(worktrees.matches("worktree ").count(), 2, "{worktrees}");
}

// Requirement: Apply leaves a marker of what it applied

#[test]
fn the_marker_names_the_index_the_version_the_root_and_the_time_of_the_switch() {
    let world = World::new();
    let version = world.version();
    world.target.apply(&srv(), &version).unwrap();
    let bytes = world.fleet.marker(MACHINE, &marker_of(&srv())).unwrap();
    let marker: Marker = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        marker,
        Marker {
            index: version.index().clone(),
            version: Version::new(VERSION),
            closure: root(),
            at: CLOCK.into(),
        }
    );
    assert_eq!(
        marker_of(&srv()).to_string(),
        "${XDG_STATE_HOME:-$HOME/.local/state}/timber/markers/afterthought/swancloud/\
         swancloud-srv/srv.json"
    );
}

// Requirement: A marker that cannot be written is said

#[test]
fn a_marker_that_cannot_be_written_is_unreadable_naming_it_and_saying_the_switch_was_made() {
    let world = World::new();
    let words = "mkdir: cannot create directory '/root/.local/state/timber': Read-only file system";
    world.fleet.mark_fails(words);
    let (what, said) = unreadable(world.target.apply(&srv(), &world.version()));
    assert_eq!(
        what,
        format!("the marker {} on swancloud-srv", marker_of(&srv()))
    );
    assert_eq!(
        said,
        format!(
            "the switch to {} was made, and the marker could not be written: {words}",
            root()
        )
    );
    assert_eq!(world.fleet.ran(), ["eval", "realise", "update", "mark"]);
}

#[test]
fn a_read_back_naming_another_closure_is_unread_naming_what_runs() {
    let world = World::new();
    world.fleet.switches_to(MACHINE, &other_root());
    let (what, said) = unreadable(world.target.apply(&srv(), &world.version()));
    assert_eq!(what, "what environment srv of swancloud-srv runs");
    assert!(
        said.starts_with(&format!(
            "swancloud-srv runs {}, and its marker",
            other_root()
        )),
        "{said}"
    );
    assert!(said.contains(&root()), "{said}");
}

// Requirement: current is what the machine runs, read through the marker

#[test]
fn current_and_report_answer_the_marker_where_the_machine_runs_its_closure() {
    let world = World::new();
    let version = world.version();
    world.target.apply(&srv(), &version).unwrap();
    assert_eq!(world.target.current(&srv()), Ok(version.index().clone()));
    assert_eq!(
        world.target.report(&srv()),
        Ok(Running {
            digest: version.index().clone(),
            converged_at: CLOCK.into(),
        })
    );
    let Some(Asked::Read { flake, .. }) = world.fleet.asked().last().cloned() else {
        panic!("{:?}", world.fleet.asked());
    };
    assert_eq!(flake, world.checkout, "current read through another flake");
}

// Requirement: current is unread where the marker does not say what runs

#[test]
fn a_machine_deployed_by_hand_after_timber_is_unread_naming_both() {
    let world = World::new();
    let version = world.version();
    world.target.apply(&srv(), &version).unwrap();
    world.fleet.runs(MACHINE, &other_root());
    for (what, said) in [
        unreadable(world.target.current(&srv())),
        unreadable(world.target.report(&srv())),
    ] {
        assert_eq!(what, "what environment srv of swancloud-srv runs");
        assert_eq!(
            said,
            format!(
                "swancloud-srv runs {}, and its marker at {} names {}, so it was switched since \
                 timber applied it",
                other_root(),
                marker_of(&srv()),
                root()
            )
        );
        assert!(!said.contains(version.index().as_str()), "{said}");
    }
}

#[test]
fn a_machine_timber_never_applied_is_unread_naming_what_it_runs_and_no_marker() {
    let world = World::new();
    for (what, said) in [
        unreadable(world.target.current(&srv())),
        unreadable(world.target.report(&srv())),
    ] {
        assert_eq!(what, "what environment srv of swancloud-srv runs");
        assert_eq!(
            said,
            format!(
                "swancloud-srv runs {BEFORE}, and holds no marker of timber's at {}",
                marker_of(&srv())
            )
        );
    }
    assert_eq!(world.fleet.ran(), ["read", "read"]);
}

#[test]
fn a_marker_timber_did_not_write_is_unreadable_naming_it() {
    let world = World::new();
    world.fleet.holds(MACHINE, &marker_of(&srv()), b"not json");
    let (what, said) = unreadable(world.target.current(&srv()));
    assert_eq!(
        what,
        format!("the marker {} on swancloud-srv", marker_of(&srv()))
    );
    assert!(said.starts_with("it is no marker timber wrote"), "{said}");
}

// Requirement: The machine is reached with the host's own key

#[test]
fn an_unreachable_machine_answers_in_sshs_words_and_timber_asks_no_one() {
    let world = World::new();
    let words = "root@192.168.1.172: Permission denied (publickey).";
    world.fleet.unreachable(MACHINE, words);
    assert_eq!(unreadable(world.target.current(&srv())).1, words);
    assert_eq!(unreadable(world.target.report(&srv())).1, words);
    assert_eq!(
        unreadable(world.target.apply(&srv(), &world.version())).1,
        words
    );
    assert_eq!(
        world.fleet.ran(),
        ["read", "read", "eval", "realise", "update"]
    );
}

// Requirement: The profile passes the target group

/// The target group's rig over the profile and the scripted fleet: the
/// target holds a marker, each environment is one of its own on the one
/// machine, the unreadable one a machine ssh cannot reach, and the
/// foreign settings add a `host` (119, 45; design D5).
struct ClanRig {
    world: World,
    environments: Cell<u32>,
}

impl ClanRig {
    const UNREACHABLE: &'static str = "gone";
    const WORDS: &'static str = "ssh: connect to host 192.0.2.1 port 22: Connection refused";

    fn new() -> ClanRig {
        let world = World::new();
        world.fleet.unreachable(Self::UNREACHABLE, Self::WORDS);
        ClanRig {
            world,
            environments: Cell::new(0),
        }
    }

    fn next(&self, machine: &str) -> Environment {
        self.environments.set(self.environments.get() + 1);
        environment(
            &format!("environment-{}", self.environments.get()),
            json!({ "machine": machine, "closure": COMPONENT }),
        )
    }
}

impl Rig for ClanRig {
    fn profile(&self) -> String {
        PROFILE.into()
    }
}

impl TargetRig for ClanRig {
    fn target(&self) -> &dyn Target {
        &self.world.target
    }

    /// The machine's closure published, and read at `digest` as a store
    /// holds an index at its own: the suite stands in the core's place,
    /// which makes a `Verified` after its verify (42).
    fn verified(&self, digest: &Digest) -> Verified {
        let published = self.world.version();
        let mut aliases = self.world.store.aliases.borrow_mut();
        aliases.insert(digest.clone(), published.index().clone());
        Verified::new(digest.clone(), published.version().clone())
    }

    /// The target is pushed to, handed the head at desired, so `head` is
    /// left unread.
    fn environment(&self, _head: Option<&Digest>) -> Environment {
        self.next(MACHINE)
    }

    fn unreadable_environment(&self) -> (Environment, String) {
        (self.next(Self::UNREACHABLE), Self::WORDS.into())
    }

    fn foreign_settings(&self) -> (Environment, String) {
        let mut environment = self.next(MACHINE);
        environment.settings["host"] = json!("root@192.168.1.172");
        (environment, "host".into())
    }
}

#[test]
fn the_target_group_passes_over_the_scripted_fleet_holding_a_marker() {
    let report = timber_conformance::run(target::GROUP, &target::scenarios(), &|| {
        Box::new(ClanRig::new()) as Box<dyn TargetRig>
    });
    report.assert();
    let inapplicable: Vec<&str> = report.not_applicable.iter().map(|n| n.scenario).collect();
    assert_eq!(
        inapplicable,
        ["applied where no marker is held"],
        "{report}"
    );
}
