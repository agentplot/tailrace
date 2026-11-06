//! Specs environment-apply and environment-reports: the environment
//! `cache` of `lib` converged on its channel's head over the fakes, two
//! versions of `lib` published in the test, each carrying the gate's
//! acceptance and a second signer's rebuild, `stable` promoted to the
//! second, and the report the deployer signs once it converged.

mod common;
mod world;

use std::cell::Cell;
use std::rc::Rc;

use serde_json::{json, Value};
use timber_core::{
    answer_json, apply, promote, Applied, ApplyAsk, PromoteAsk, Published, Reported, To,
};
use timber_fakes::{
    FakeStore, FakeTarget, Fault, KeeperOp, SignerOp, StoreOp, TargetOp, Times, NO_MARKER,
};
use timber_seams::{
    channel_name, Answer, AskedOf, Audience, Branch, ChannelId, ChannelName, Claim, ClaimKind,
    ComponentName, DeclaredEnvironment, Digest, Envelope, Environment, EnvironmentName, Head,
    Keeper, NamedSigner, Not, Outcome, OutputName, Report, Required, RequiredClaim, Running, Seam,
    Seams, Signer, SignerId, Store, Target, TargetProfile, TargetTable, Verified, Version, Why,
};
use world::{build_of, version_of, World};

/// The identity the host signs promotions and its reports as, which the
/// policy names for acceptance and promotion (108), and lets claim a
/// report.
const GATE: &str = "gate";

/// The second signer, whose rebuild the policy for `stable` requires.
const REBUILD: &str = "rebuild";

/// The moment the promotion is asked at.
const AT: u64 = 1_767_225_600;

/// The moment the apply is asked at, an hour after.
const APPLIED_AT: u64 = AT + 3_600;

/// When the fake target's apply converges.
const CONVERGED: &str = "2026-11-05T09:00:00Z";

/// A target's own words, carried verbatim (59).
const WORDS: &str = "push refused: cache \"fleet\" is read-only\n\t\u{fc} \u{1f6ab} ";

fn named(id: &str, kinds: &[ClaimKind]) -> NamedSigner {
    NamedSigner {
        id: SignerId::new(id),
        verified_by: format!("{id}'s key"),
        kinds: kinds.to_vec(),
        checks: vec![],
    }
}

fn wanted(kind: ClaimKind, signers: Vec<NamedSigner>) -> RequiredClaim {
    RequiredClaim {
        kind,
        check: None,
        environment: None,
        signers,
        asked_of: AskedOf::Every,
    }
}

fn gate() -> NamedSigner {
    named(GATE, &[ClaimKind::Acceptance, ClaimKind::Promotion])
}

/// `stable` requires the gate's acceptance and a rebuild by
/// `rebuilt_by`, and is promoted by the gate.
fn stable_policy(rebuilt_by: &str) -> Required {
    Required::Channel {
        claims: vec![
            wanted(ClaimKind::Acceptance, vec![gate()]),
            wanted(
                ClaimKind::Rebuild,
                vec![named(rebuilt_by, &[ClaimKind::Rebuild])],
            ),
        ],
        bound_to: None,
        promoted_by: vec![gate()],
        approvers: vec![],
    }
}

/// The channel `name` of `component`, as the world declares components.
fn of(component: &str, name: &str) -> ChannelId {
    ChannelId::new(Some(ComponentName::new(component)), ChannelName::new(name))
}

fn stable() -> ChannelId {
    of("lib", "stable")
}

fn fake() -> TargetProfile {
    TargetProfile::new("fake")
}

fn cache() -> EnvironmentName {
    EnvironmentName::new("cache")
}

/// The target's own settings, as the environment's table writes them
/// (45, C.10).
fn settings() -> Value {
    json!({ "at": "a place of its own", "outputs": ["lib-pack"] })
}

/// `cache` following `channel`, its target the profile `profile` names
/// with the settings, or none.
fn cache_on(channel: ChannelId, profile: Option<&str>) -> DeclaredEnvironment {
    DeclaredEnvironment {
        name: cache(),
        channel,
        waits: false,
        approvers: vec![],
        target: profile.map(|p| TargetTable {
            profile: TargetProfile::new(p),
            settings: settings(),
        }),
    }
}

/// `cache` as the target is handed it.
fn handed() -> Environment {
    Environment {
        name: cache(),
        channel: stable(),
        waits: false,
        settings: settings(),
    }
}

/// The world, the fake target bound under `fake`, and `lib`'s two
/// indexes.
struct Applying {
    world: World,
    target: FakeTarget,
    first: Digest,
    second: Digest,
}

/// `lib` published at `start` as 0.1.0 and at the head as 0.2.0, each
/// index carrying the gate's acceptance and the second signer's rebuild;
/// `stable` promoted to 0.2.0 by the gate under its policy; `cache`
/// declared at the head following `stable` on the fake target, which
/// runs 0.1.0.
fn applying() -> Applying {
    let mut world = World::new();
    world.fakes.landing.land(&world.main, &world.start);
    for o in ["lib-pack", "lib-docs"] {
        world.fakes.build.made(
            &world.start,
            &OutputName::new(o),
            Report {
                build: build_of(o, &world.start),
                outcome: Outcome::Pass,
                files: vec![format!("dist/{o}.tar")],
            },
        );
    }
    let at_start = timber_core::Ask {
        revision: world.start.clone(),
        ..world.ask_for("lib")
    };
    let first = index_of(&world.publish_one(&at_start).unwrap(), "0.1.0");
    let second = index_of(&world.publish_one(&world.ask_for("lib")).unwrap(), "0.2.0");
    for index in [&first, &second] {
        attach(&world, index, GATE, ClaimKind::Acceptance, json!({}));
        attach(
            &world,
            index,
            REBUILD,
            ClaimKind::Rebuild,
            json!({ "matched": true }),
        );
    }
    world.fakes.signer.hold(&SignerId::new(GATE));
    world
        .fakes
        .policy
        .require(Audience::Channel(stable()), stable_policy(REBUILD));
    world.fakes.policy.require(
        Audience::Layout,
        Required::Layout {
            claims: vec![],
            given: false,
            signers: vec![
                named(REBUILD, &[ClaimKind::Rebuild]),
                named(
                    GATE,
                    &[
                        ClaimKind::Acceptance,
                        ClaimKind::Promotion,
                        ClaimKind::Report,
                    ],
                ),
            ],
        },
    );
    let promoted = promote(
        &world.fakes.seams(),
        &world.history(),
        &PromoteAsk {
            branch: world.main.clone(),
            component: Some(ComponentName::new("lib")),
            channel: stable().name,
            to: To::Version(Version::new("0.2.0")),
            from: None,
            signer: SignerId::new(GATE),
            at: AT,
        },
    );
    assert!(promoted.is_ok(), "{promoted:?}");

    world
        .fakes
        .policy
        .declares_at(&world.head, cache_on(stable(), Some("fake")));
    let target = world.fakes.target(&fake());
    target.runs(
        &cache(),
        Running {
            digest: first.clone(),
            converged_at: "2026-01-01T00:00:00Z".into(),
        },
    );
    target.at(CONVERGED);
    Applying {
        world,
        target,
        first,
        second,
    }
}

fn index_of(published: &Published, version: &str) -> Digest {
    let (v, index) = version_of(published);
    assert_eq!(v.as_str(), version);
    index.clone()
}

/// A claim of `kind` about `index` by `signer`, attached to it.
fn attach(world: &World, index: &Digest, signer: &str, kind: ClaimKind, body: Value) {
    let theirs = timber_fakes::FakeSigner::new();
    let id = SignerId::new(signer);
    theirs.hold(&id);
    let claim = Claim {
        subject: index.clone(),
        kind,
        revision: world.head.clone(),
        build: None,
        body,
    };
    let envelope = theirs.sign(&claim, &id).unwrap();
    world.fakes.store.attach(index, &envelope).unwrap();
}

fn ask() -> ApplyAsk {
    ApplyAsk {
        branch: Branch::new("main"),
        component: Some(ComponentName::new("lib")),
        environment: cache(),
        signer: SignerId::new(GATE),
        at: APPLIED_AT,
    }
}

fn applied(world: &World) -> Answer<Applied> {
    apply(&world.fakes.seams(), &world.history(), &ask())
}

fn applied_over(world: &World, seams: &Seams, ask: &ApplyAsk) -> Answer<Applied> {
    apply(seams, &world.history(), ask)
}

/// The envelopes of report claims attached to `index`, in the order
/// attached.
fn reports_on(world: &World, index: &Digest) -> Vec<Envelope> {
    world
        .fakes
        .store
        .claims_of(index)
        .unwrap()
        .into_iter()
        .filter(|e| Claim::from_envelope(e).is_some_and(|c| c.kind == ClaimKind::Report))
        .collect()
}

/// The report claims the host's signer was asked to sign, in order.
fn signed(world: &World) -> Vec<Value> {
    world
        .fakes
        .signer
        .calls()
        .into_iter()
        .filter(|c| c.op == SignerOp::Sign && c.given["claim"]["kind"] == "report")
        .map(|c| c.given["claim"].clone())
        .collect()
}

/// The operations the fake target was asked, in order.
fn ops(target: &FakeTarget) -> Vec<TargetOp> {
    target.calls().into_iter().map(|c| c.op).collect()
}

fn absent(answer: Answer<Applied>) -> timber_seams::Absent {
    match answer {
        Err(Not::Absent(a)) => a,
        other => panic!("answered {other:?}, not absent"),
    }
}

fn refused(answer: Answer<Applied>) -> Why {
    match answer {
        Err(Not::Refused(r)) => r.why,
        other => panic!("answered {other:?}, not a refusal"),
    }
}

/// The unread answer the fake target gives of `cache` once it holds no
/// marker (119).
fn unmarked() -> Not {
    Not::unreadable(
        &timber_seams::Source::new(Seam::Target, "fake"),
        "what cache runs",
        Some(NO_MARKER.into()),
    )
}

/// A target that answers desired with what it reads itself, as a target
/// that pulls answers it (44), and asks the fake target everything else.
struct ReadsItself {
    inner: FakeTarget,
    reads: Digest,
}

impl Target for ReadsItself {
    fn desired(&self, _environment: &Environment, _head: Head<'_>) -> Answer<Digest> {
        Ok(self.reads.clone())
    }

    fn current(&self, environment: &Environment) -> Answer<Digest> {
        self.inner.current(environment)
    }

    fn apply(&self, environment: &Environment, version: &Verified) -> Answer<()> {
        self.inner.apply(environment, version)
    }

    fn report(&self, environment: &Environment) -> Answer<Running> {
        self.inner.report(environment)
    }
}

/// A target whose report answers `reports`, whatever it applied, and
/// asks the fake target everything else.
struct ReportsOther {
    inner: FakeTarget,
    reports: Digest,
}

impl Target for ReportsOther {
    fn desired(&self, environment: &Environment, head: Head<'_>) -> Answer<Digest> {
        self.inner.desired(environment, head)
    }

    fn current(&self, environment: &Environment) -> Answer<Digest> {
        self.inner.current(environment)
    }

    fn apply(&self, environment: &Environment, version: &Verified) -> Answer<()> {
        self.inner.apply(environment, version)
    }

    fn report(&self, environment: &Environment) -> Answer<Running> {
        let mut running = self.inner.report(environment)?;
        running.digest = self.reports.clone();
        Ok(running)
    }
}

/// A target that, at an apply, records how many operations the store
/// had been asked, then asks the fake target.
struct Watched {
    inner: FakeTarget,
    store: FakeStore,
    at: Rc<Cell<Option<usize>>>,
}

impl Target for Watched {
    fn desired(&self, environment: &Environment, head: Head<'_>) -> Answer<Digest> {
        self.inner.desired(environment, head)
    }

    fn current(&self, environment: &Environment) -> Answer<Digest> {
        self.inner.current(environment)
    }

    fn apply(&self, environment: &Environment, version: &Verified) -> Answer<()> {
        self.at.set(Some(self.store.calls().len()));
        self.inner.apply(environment, version)
    }

    fn report(&self, environment: &Environment) -> Answer<Running> {
        self.inner.report(environment)
    }
}

// Requirement: Apply finds the environment at the landed head

#[test]
fn an_environment_the_policy_does_not_declare_in_the_component_is_absent_naming_both() {
    let Applying { world, target, .. } = applying();
    let web = ApplyAsk {
        component: Some(ComponentName::new("web")),
        ..ask()
    };
    let a = absent(applied_over(&world, &world.fakes.seams(), &web));
    assert_eq!(a.source.seam, Seam::Policy);
    assert!(a.what.contains("environment cache of web"), "{}", a.what);
    assert!(target.calls().is_empty(), "the target was asked");
}

#[test]
fn the_environments_are_asked_of_the_policy_at_the_branchs_landed_head() {
    let Applying { world, .. } = applying();
    let before = world.fakes.policy.calls().len();
    assert!(applied(&world).is_ok());
    let asked: Vec<Value> = world.fakes.policy.calls()[before..]
        .iter()
        .filter(|c| c.op == timber_fakes::PolicyOp::Environments)
        .map(|c| c.given["revision"].clone())
        .collect();
    assert_eq!(asked, [json!(world.head)]);
}

// Requirement: Apply reaches the target the environment names

#[test]
fn a_target_profile_no_binding_holds_is_absent_naming_it_and_no_target_is_asked() {
    let Applying { world, target, .. } = applying();
    world
        .fakes
        .policy
        .declares_at(&world.head, cache_on(stable(), Some("bucket")));
    let a = absent(applied(&world));
    assert_eq!(a.source.seam, Seam::Target);
    assert!(a.what.contains("bucket"), "{}", a.what);
    assert!(target.calls().is_empty(), "the target was asked");
}

#[test]
fn an_environment_with_no_target_is_absent_naming_its_target_and_no_target_is_asked() {
    let Applying { world, target, .. } = applying();
    world
        .fakes
        .policy
        .declares_at(&world.head, cache_on(stable(), None));
    let a = absent(applied(&world));
    assert_eq!(a.source.seam, Seam::Policy);
    assert!(
        a.what.contains("target of environment cache of lib"),
        "{}",
        a.what
    );
    assert!(target.calls().is_empty(), "the target was asked");
}

// Requirement: Apply reads what the environment should run from its
// channel's chain

#[test]
fn an_empty_channel_is_absent_naming_it_and_the_target_is_asked_nothing() {
    let Applying { world, target, .. } = applying();
    world
        .fakes
        .policy
        .declares_at(&world.head, cache_on(of("lib", "beta"), Some("fake")));
    let a = absent(applied(&world));
    assert_eq!(a.source.seam, Seam::Keeper);
    assert!(a.what.contains("lib/beta"), "{}", a.what);
    assert!(target.calls().is_empty(), "the target was asked");
}

#[test]
fn desired_is_handed_the_chains_head_never_the_channels_name_in_the_store() {
    let Applying {
        world,
        target,
        first,
        second,
    } = applying();
    let name = channel_name(&stable());
    world
        .fakes
        .store
        .name(&name, &first, Some(&second))
        .unwrap();
    let before = world.fakes.store.calls().len();
    let answer = applied(&world).unwrap();
    assert_eq!(answer.index, second, "applied the name's digest");
    let desired = &target.calls()[0];
    assert_eq!(desired.op, TargetOp::Desired);
    assert_eq!(desired.answer, Ok(json!(second)));
    assert!(
        !world.fakes.store.calls()[before..]
            .iter()
            .any(|c| c.op == StoreOp::Resolve),
        "a name was resolved"
    );
}

#[test]
fn a_chain_the_ledger_answers_as_last_read_is_unreadable_and_the_target_is_asked_nothing() {
    let Applying { world, target, .. } = applying();
    world.fakes.keeper.unreachable(WORDS);
    let answer = applied(&world);
    let Err(Not::Unreadable(u)) = answer else {
        panic!("answered {answer:?}, not unreadable")
    };
    assert_eq!(u.source.seam, Seam::Keeper);
    assert!(u.what.contains("lib/stable"), "{}", u.what);
    assert_eq!(u.said.unwrap().as_bytes(), WORDS.as_bytes());
    assert!(target.calls().is_empty(), "the target was asked");
}

// Requirement: A target's desired must be the chain's head

#[test]
fn a_target_that_answers_another_digest_than_the_head_is_refused_naming_both() {
    let Applying {
        world,
        target,
        first,
        second,
    } = applying();
    let mut seams = world.fakes.seams();
    seams.targets.insert(
        fake(),
        Box::new(ReadsItself {
            inner: target.clone(),
            reads: first.clone(),
        }),
    );
    let answer = applied_over(&world, &seams, &ask());
    let Err(Not::Refused(r)) = &answer else {
        panic!("answered {answer:?}, not a refusal")
    };
    assert_eq!(r.source.seam, Seam::Target);
    assert_eq!(
        r.why,
        Why::Desired {
            channel: stable(),
            head: second,
            desired: first,
        }
    );
    assert!(!ops(&target).contains(&TargetOp::Apply), "applied");
}

// Requirement: Apply judges the head's claims before it hands over the
// version

#[test]
fn a_head_whose_claim_is_wanting_is_refused_naming_it_and_nothing_is_applied() {
    let Applying { world, target, .. } = applying();
    world
        .fakes
        .policy
        .require(Audience::Channel(stable()), stable_policy("box"));
    assert_eq!(
        refused(applied(&world)),
        Why::Wanting {
            kind: ClaimKind::Rebuild,
            check: None,
            environment: None,
            signers: vec![SignerId::new("box")],
        }
    );
    assert_eq!(ops(&target), [TargetOp::Desired, TargetOp::Current]);
    assert!(signed(&world).is_empty(), "a report was signed");
}

#[test]
fn the_heads_claims_are_read_before_the_target_is_handed_the_version() {
    let Applying {
        world,
        target,
        second,
        ..
    } = applying();
    let at = Rc::new(Cell::new(None));
    let mut seams = world.fakes.seams();
    seams.targets.insert(
        fake(),
        Box::new(Watched {
            inner: target.clone(),
            store: world.fakes.store.clone(),
            at: at.clone(),
        }),
    );
    let before = world.fakes.store.calls().len();
    assert!(applied_over(&world, &seams, &ask()).is_ok());
    let at = at.get().expect("the target was asked to apply");
    let read = |range: &[timber_fakes::Call<StoreOp>]| {
        range
            .iter()
            .any(|c| c.op == StoreOp::ClaimsOf && c.given["digest"] == json!(second))
    };
    let calls = world.fakes.store.calls();
    assert!(
        read(&calls[before..at]),
        "applied before the claims were read"
    );
    assert!(!read(&calls[at..]), "the claims were read after the apply");
}

// Requirement: Apply hands the target the verified version

#[test]
fn the_channels_head_is_applied_and_the_answer_names_it_with_the_targets_report() {
    let Applying {
        world,
        target,
        first,
        second,
    } = applying();
    let answer = applied(&world);
    let report = reports_on(&world, &second);
    assert_eq!(report.len(), 1, "{report:?}");
    assert_eq!(
        answer,
        Ok(Applied {
            environment: cache(),
            component: ComponentName::new("lib"),
            channel: ChannelName::new("stable"),
            version: Version::new("0.2.0"),
            index: second.clone(),
            applied: true,
            current: Ok(first),
            running: Ok(Running {
                digest: second.clone(),
                converged_at: CONVERGED.into(),
            }),
            report: Ok(report[0].digest()),
        })
    );
    assert_eq!(
        ops(&target),
        [
            TargetOp::Desired,
            TargetOp::Current,
            TargetOp::Apply,
            TargetOp::Report
        ]
    );
    let apply = &target.calls()[2];
    assert_eq!(
        apply.given,
        json!({
            "environment": handed(),
            "version": Verified::new(second, Version::new("0.2.0")),
        }),
        "the target was handed another environment or version"
    );
}

// Requirement: Apply does not apply what the environment already runs

#[test]
fn an_environment_that_runs_the_head_already_is_not_applied_and_the_answer_says_so() {
    let Applying {
        world,
        target,
        second,
        ..
    } = applying();
    target.runs(
        &cache(),
        Running {
            digest: second.clone(),
            converged_at: "2026-02-01T00:00:00Z".into(),
        },
    );
    let answer = applied(&world).unwrap();
    assert!(!answer.applied, "{answer:?}");
    assert_eq!(answer.current, Ok(second.clone()));
    assert_eq!(answer.version, Version::new("0.2.0"));
    assert!(!ops(&target).contains(&TargetOp::Apply), "applied");
}

// Requirement: Apply's answer

#[test]
fn a_target_that_cannot_apply_is_the_answer_in_its_own_words() {
    let Applying { world, target, .. } = applying();
    target.fault(
        TargetOp::Apply,
        Fault::Unreadable(WORDS.into()),
        Times::Once,
    );
    let answer = applied(&world);
    let Err(Not::Unreadable(u)) = answer else {
        panic!("answered {answer:?}, not unreadable")
    };
    assert_eq!(u.source.seam, Seam::Target);
    assert_eq!(u.said.unwrap().as_bytes(), WORDS.as_bytes());
    assert_eq!(ops(&target).last(), Some(&TargetOp::Apply));
}

#[test]
fn a_target_that_keeps_no_marker_is_applied_and_answers_current_and_report_unread() {
    let Applying {
        world,
        target,
        second,
        ..
    } = applying();
    target.holds_no_marker();
    let answer = applied(&world).unwrap();
    assert!(answer.applied, "{answer:?}");
    assert_eq!(answer.index, second);
    assert_eq!(answer.current, Err(unmarked()));
    assert_eq!(answer.running, Err(unmarked()));
    assert_eq!(answer.report, Err(unmarked()));
    let json = serde_json::to_value(&answer).unwrap();
    assert_eq!(json["current"]["answer"], "unreadable", "{json}");
    assert_eq!(json["running"]["unreadable"]["said"], NO_MARKER, "{json}");
    assert_eq!(json["report"]["unreadable"]["said"], NO_MARKER, "{json}");
    assert!(signed(&world).is_empty(), "a report was signed");
    assert!(
        reports_on(&world, &second).is_empty(),
        "a report was attached"
    );
}

#[test]
fn the_answer_serializes_in_the_one_form_every_surface_renders() {
    let Applying {
        world,
        first,
        second,
        ..
    } = applying();
    let answer = applied(&world);
    let report = reports_on(&world, &second)[0].digest();
    assert_eq!(
        answer_json(&answer),
        json!({
            "answer": "ok",
            "value": {
                "environment": "cache",
                "component": "lib",
                "channel": "stable",
                "version": "0.2.0",
                "index": second,
                "applied": true,
                "current": { "answer": "ok", "value": first },
                "running": {
                    "answer": "ok",
                    "value": { "digest": second, "converged_at": CONVERGED },
                },
                "report": { "answer": "ok", "value": report },
            },
        })
    );
}

// Requirement: Apply moves no name and makes no store

#[test]
fn an_apply_moves_no_name_appends_nothing_and_writes_only_its_report_beside_the_index() {
    let Applying { world, second, .. } = applying();
    let held = world.fakes.store.contents();
    let chain = world.fakes.keeper.chain(&stable()).unwrap();
    let (store, keeper, signer) = (
        world.fakes.store.calls().len(),
        world.fakes.keeper.calls().len(),
        world.fakes.signer.calls().len(),
    );
    assert!(applied(&world).unwrap().applied);
    let writes: Vec<(StoreOp, Value)> = world.fakes.store.calls()[store..]
        .iter()
        .filter(|c| {
            matches!(
                c.op,
                StoreOp::PutBytes | StoreOp::PutArtifact | StoreOp::Attach | StoreOp::Name
            )
        })
        .map(|c| (c.op, c.given["subject"].clone()))
        .collect();
    assert_eq!(writes, [(StoreOp::Attach, json!(second))]);
    let now = world.fakes.store.contents();
    assert_eq!(
        (now.held, now.names),
        (held.held, held.names),
        "bytes or a name were written"
    );
    assert!(
        !world.fakes.keeper.calls()[keeper..]
            .iter()
            .any(|c| c.op == KeeperOp::Append),
        "the ledger was appended"
    );
    let signs = world.fakes.signer.calls()[signer..]
        .iter()
        .filter(|c| c.op == SignerOp::Sign)
        .count();
    assert_eq!(signs, 1, "signed other than the one report");
    assert_eq!(world.fakes.keeper.chain(&stable()).unwrap(), chain);
}

// Requirement: A converged apply signs one report

#[test]
fn a_converged_apply_signs_one_report_beside_the_index_and_answers_its_digest() {
    let Applying {
        world,
        target,
        second,
        ..
    } = applying();
    let answer = applied(&world).unwrap();
    let [report] = <[Envelope; 1]>::try_from(reports_on(&world, &second)).unwrap();
    assert_eq!(answer.report, Ok(report.digest()));
    assert_eq!(
        report.sole().map(|s| s.signer.as_str()),
        Some(GATE),
        "signed by another identity"
    );
    assert_eq!(ops(&target).last(), Some(&TargetOp::Report));
    assert_eq!(signed(&world).len(), 1);
}

// Requirement: A report claim says what runs where, and when

#[test]
fn the_report_names_the_environment_its_channel_the_version_what_runs_and_when() {
    let Applying { world, second, .. } = applying();
    applied(&world).unwrap();
    let [report] = <[Envelope; 1]>::try_from(reports_on(&world, &second)).unwrap();
    let claim = Claim::from_envelope(&report).unwrap();
    assert_eq!(claim.subject, second);
    assert_eq!(claim.revision, world.head, "names another revision");
    assert_eq!(claim.build, None);
    assert_eq!(
        Reported::of(&claim),
        Some(Reported {
            environment: cache(),
            component: ComponentName::new("lib"),
            channel: ChannelName::new("stable"),
            version: Version::new("0.2.0"),
            index: second.clone(),
            runs: second.clone(),
            converged_at: CONVERGED.into(),
            at: APPLIED_AT,
        })
    );
    let statement: Value = serde_json::from_slice(&report.payload).unwrap();
    assert_eq!(
        statement["predicateType"],
        "https://agentplot.github.io/timber/claim/report/v1"
    );
}

#[test]
fn a_report_that_names_another_digest_signs_nothing_and_is_unread_naming_both() {
    let Applying {
        world,
        target,
        first,
        second,
    } = applying();
    let mut seams = world.fakes.seams();
    seams.targets.insert(
        fake(),
        Box::new(ReportsOther {
            inner: target.clone(),
            reports: first.clone(),
        }),
    );
    let answer = applied_over(&world, &seams, &ask()).unwrap();
    assert!(answer.applied, "{answer:?}");
    let Err(Not::Unreadable(u)) = &answer.report else {
        panic!("the report answered {:?}, not unreadable", answer.report)
    };
    assert_eq!(u.source.seam, Seam::Target);
    assert!(u.what.contains(first.as_str()), "{}", u.what);
    assert!(u.what.contains(second.as_str()), "{}", u.what);
    assert!(signed(&world).is_empty(), "a report was signed");
    assert!(
        reports_on(&world, &second).is_empty(),
        "a report was attached"
    );
}

#[test]
fn an_unread_report_signs_nothing_and_carries_the_targets_answer_in_its_place() {
    let Applying {
        world,
        target,
        second,
        ..
    } = applying();
    target.fault(
        TargetOp::Report,
        Fault::Unreadable(WORDS.into()),
        Times::Once,
    );
    let answer = applied(&world).unwrap();
    assert!(answer.applied, "{answer:?}");
    let Err(Not::Unreadable(u)) = &answer.report else {
        panic!("the report answered {:?}, not unreadable", answer.report)
    };
    assert_eq!(u.source.seam, Seam::Target);
    assert_eq!(u.said.as_deref().map(str::as_bytes), Some(WORDS.as_bytes()));
    assert_eq!(answer.running.as_ref().err(), answer.report.as_ref().err());
    assert!(signed(&world).is_empty(), "a report was signed");
    assert!(
        reports_on(&world, &second).is_empty(),
        "a report was attached"
    );
}

// Requirement: An environment already running the head is reported once

#[test]
fn an_environment_already_reported_running_the_head_is_not_reported_again() {
    let Applying { world, second, .. } = applying();
    let first = applied(&world).unwrap();
    assert!(first.applied);
    let signs = signed(&world).len();
    let again = applied(&world).unwrap();
    assert!(!again.applied, "{again:?}");
    assert_eq!(again.report, first.report, "answered another report");
    assert_eq!(signed(&world).len(), signs, "a report was signed again");
    assert_eq!(reports_on(&world, &second).len(), 1);
}

#[test]
fn an_environment_running_the_head_whose_newest_report_names_another_index_is_reported() {
    let Applying {
        world,
        target,
        first,
        second,
    } = applying();
    let old = Reported {
        environment: cache(),
        component: ComponentName::new("lib"),
        channel: ChannelName::new("stable"),
        version: Version::new("0.1.0"),
        index: first.clone(),
        runs: first.clone(),
        converged_at: "2026-01-01T00:00:00Z".into(),
        at: AT - 1,
    };
    // Signed as the host signed it at an earlier apply.
    let earlier = timber_fakes::FakeSigner::new();
    earlier.hold(&SignerId::new(GATE));
    let envelope = earlier
        .sign(&old.claim(world.start.clone()), &SignerId::new(GATE))
        .unwrap();
    world.fakes.store.attach(&first, &envelope).unwrap();
    // The chain has held 0.1.0 too, so the old report is the newest.
    promote_to(&world, To::Version(Version::new("0.1.0")), AT + 1);
    promote_to(&world, To::Version(Version::new("0.2.0")), AT + 2);
    target.runs(
        &cache(),
        Running {
            digest: second.clone(),
            converged_at: CONVERGED.into(),
        },
    );
    let answer = applied(&world).unwrap();
    assert!(!answer.applied, "{answer:?}");
    let [report] = <[Envelope; 1]>::try_from(reports_on(&world, &second)).unwrap();
    assert_eq!(answer.report, Ok(report.digest()));
    assert_eq!(signed(&world).len(), 1);
}

#[test]
fn a_report_the_store_refused_to_hold_is_made_again_by_the_next_apply() {
    let Applying {
        world,
        target,
        second,
        ..
    } = applying();
    world
        .fakes
        .store
        .fault(StoreOp::Attach, Fault::Refused(WORDS.into()), Times::Once);
    let lost = applied(&world).unwrap();
    assert!(lost.applied && lost.report.is_err(), "{lost:?}");
    assert!(reports_on(&world, &second).is_empty());

    let again = applied(&world).unwrap();
    assert!(!again.applied, "{again:?}");
    let [report] = <[Envelope; 1]>::try_from(reports_on(&world, &second)).unwrap();
    assert_eq!(again.report, Ok(report.digest()));
    assert_eq!(
        ops(&target)
            .iter()
            .filter(|op| **op == TargetOp::Report)
            .count(),
        2,
        "the target's report was not asked again"
    );
    assert_eq!(applied(&world).unwrap().report, Ok(report.digest()));
    assert_eq!(reports_on(&world, &second).len(), 1, "reported twice");
}

// Requirement: A report is attached beside its index

#[test]
fn a_refused_attach_is_the_reports_answer_in_the_stores_words_with_the_convergence_made() {
    let Applying { world, second, .. } = applying();
    world
        .fakes
        .store
        .fault(StoreOp::Attach, Fault::Refused(WORDS.into()), Times::Once);
    let answer = applied(&world).unwrap();
    assert!(answer.applied, "{answer:?}");
    assert_eq!(
        answer.running,
        Ok(Running {
            digest: second.clone(),
            converged_at: CONVERGED.into(),
        })
    );
    let Err(Not::Refused(r)) = &answer.report else {
        panic!("the report answered {:?}, not refused", answer.report)
    };
    assert_eq!(r.source.seam, Seam::Store);
    assert_eq!(r.said.as_deref().map(str::as_bytes), Some(WORDS.as_bytes()));
}

/// `stable` moved by the gate to `to` at `at`.
fn promote_to(world: &World, to: To, at: u64) {
    let promoted = promote(
        &world.fakes.seams(),
        &world.history(),
        &PromoteAsk {
            branch: world.main.clone(),
            component: Some(ComponentName::new("lib")),
            channel: stable().name,
            to,
            from: None,
            signer: SignerId::new(GATE),
            at,
        },
    );
    assert!(promoted.is_ok(), "{promoted:?}");
}
