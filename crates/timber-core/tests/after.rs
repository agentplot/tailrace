//! Spec environment-reports: an environment's newest report read across
//! its channel's chain over the fakes, two versions of `lib` published in
//! the test, each carrying the gate's acceptance and a second signer's
//! rebuild, and `stable` promoted between them (design D8).

mod common;
mod world;

use serde_json::{json, Value};
use timber_core::{newest, promote, Newest, PromoteAsk, Published, Reported, To};
use timber_fakes::{FakeSigner, Fault, StoreOp, Times};
use timber_seams::{
    Answer, AskedOf, Audience, Branch, ChannelId, ChannelName, Claim, ClaimKind, ComponentName,
    Digest, EnvironmentName, NamedSigner, Not, Outcome, OutputName, Report, Required,
    RequiredClaim, Seam, Signer, SignerId, Store, Version,
};
use world::{build_of, version_of, World};

/// The identity the host signs promotions as, which the policy names
/// for acceptance and promotion (108).
const GATE: &str = "gate";

/// The second signer, whose rebuild the policy for `stable` requires.
const REBUILD: &str = "rebuild";

/// The signer the requirement lets claim a report.
const DEPLOYER: &str = "deployer";

/// 2026-01-01T00:00:00Z, the moment the first promotion is asked at.
const AT: u64 = 1_767_225_600;

const HOUR: u64 = 3_600;

/// A store's own words, carried verbatim (59).
const WORDS: &str = "referrers of the index: 503 Service Unavailable\n\t\u{fc} ";

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

fn stable() -> ChannelId {
    ChannelId::new(Some(ComponentName::new("lib")), ChannelName::new("stable"))
}

fn lib() -> ComponentName {
    ComponentName::new("lib")
}

fn srv() -> EnvironmentName {
    EnvironmentName::new("srv")
}

/// A report of `srv` by the deployer.
fn by_deployer() -> RequiredClaim {
    RequiredClaim {
        environment: Some(srv()),
        ..wanted(
            ClaimKind::Report,
            vec![named(DEPLOYER, &[ClaimKind::Report])],
        )
    }
}

/// Two published versions of `lib` and their indexes.
struct Two {
    world: World,
    first: Digest,
    second: Digest,
}

/// `lib` published at `start` as 0.1.0 and at the head as 0.2.0, each
/// index carrying the gate's acceptance and the second signer's rebuild,
/// the policy for `stable` requiring both, and the host's signer holding
/// the gate's identity.
fn two() -> Two {
    let world = World::new();
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
    world.fakes.policy.require(
        Audience::Channel(stable()),
        Required::Channel {
            claims: vec![
                wanted(ClaimKind::Acceptance, vec![gate()]),
                wanted(
                    ClaimKind::Rebuild,
                    vec![named(REBUILD, &[ClaimKind::Rebuild])],
                ),
            ],
            bound_to: None,
            promoted_by: vec![gate()],
            approvers: vec![],
        },
    );
    Two {
        world,
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
    let theirs = FakeSigner::new();
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

/// `stable` moved to `to` at `at`.
fn move_to(world: &World, to: To, at: u64) {
    let promoted = promote(
        &world.fakes.seams(),
        &world.history(),
        &PromoteAsk {
            branch: Branch::new("main"),
            component: Some(lib()),
            channel: stable().name,
            to,
            from: None,
            signer: SignerId::new(GATE),
            at,
        },
    );
    assert!(promoted.is_ok(), "{promoted:?}");
}

fn to(version: &str) -> To {
    To::Version(Version::new(version))
}

/// A report of `environment` of `component` running `version` at
/// `index`, from an apply at `at`, signed by `signer` and attached to the
/// index; the report and its envelope's digest.
fn report(
    world: &World,
    signer: &str,
    (environment, component): (&str, &str),
    (version, index): (&str, &Digest),
    at: u64,
) -> Newest {
    let theirs = FakeSigner::new();
    let id = SignerId::new(signer);
    theirs.hold(&id);
    let report = Reported {
        environment: EnvironmentName::new(environment),
        component: ComponentName::new(component),
        channel: ChannelName::new("stable"),
        version: Version::new(version),
        index: index.clone(),
        runs: index.clone(),
        // The machine's clock, which orders nothing: a day behind.
        converged_at: "2025-12-31T00:00:00Z".into(),
        at,
    };
    let envelope = theirs.sign(&report.claim(world.head.clone()), &id).unwrap();
    world.fakes.store.attach(index, &envelope).unwrap();
    Newest {
        claim: envelope.digest(),
        report,
    }
}

fn newest_of(world: &World) -> Answer<Newest> {
    newest(
        &world.fakes.seams(),
        &srv(),
        &lib(),
        &stable(),
        &by_deployer(),
    )
}

// Requirement: An environment's newest report

#[test]
fn a_rolled_back_environments_newest_report_names_the_version_it_went_back_to() {
    let Two {
        world,
        first,
        second,
    } = two();
    move_to(&world, to("0.1.0"), AT);
    move_to(&world, to("0.2.0"), AT + HOUR);
    move_to(&world, To::Back, AT + 2 * HOUR);
    report(&world, DEPLOYER, ("srv", "lib"), ("0.1.0", &first), AT);
    report(
        &world,
        DEPLOYER,
        ("srv", "lib"),
        ("0.2.0", &second),
        AT + 10 * HOUR,
    );
    let back = report(
        &world,
        DEPLOYER,
        ("srv", "lib"),
        ("0.1.0", &first),
        AT + 11 * HOUR,
    );
    assert_eq!(newest_of(&world), Ok(back));
}

#[test]
fn the_newest_index_of_the_chain_carrying_a_stale_report_does_not_make_it_newest() {
    let Two {
        world,
        first,
        second,
    } = two();
    move_to(&world, to("0.1.0"), AT);
    move_to(&world, to("0.2.0"), AT + HOUR);
    move_to(&world, To::Back, AT + 2 * HOUR);
    report(&world, DEPLOYER, ("srv", "lib"), ("0.1.0", &first), AT);
    let still = report(
        &world,
        DEPLOYER,
        ("srv", "lib"),
        ("0.2.0", &second),
        AT + HOUR,
    );
    assert_eq!(newest_of(&world), Ok(still));
}

#[test]
fn each_index_the_chain_moved_to_is_asked_its_claims_once() {
    let Two {
        world,
        first,
        second,
    } = two();
    move_to(&world, to("0.1.0"), AT);
    move_to(&world, to("0.2.0"), AT + HOUR);
    move_to(&world, To::Back, AT + 2 * HOUR);
    report(&world, DEPLOYER, ("srv", "lib"), ("0.1.0", &first), AT);
    let before = world.fakes.store.calls().len();
    assert!(newest_of(&world).is_ok());
    let asked: Vec<Value> = world.fakes.store.calls()[before..]
        .iter()
        .filter(|c| c.op == StoreOp::ClaimsOf)
        .map(|c| c.given["digest"].clone())
        .collect();
    assert_eq!(asked, [json!(first), json!(second)]);
}

// Requirement: No newest report is absent or unreadable, never a guess

#[test]
fn no_report_is_absent_naming_the_environment() {
    let Two { world, .. } = two();
    move_to(&world, to("0.1.0"), AT);
    match newest_of(&world) {
        Err(Not::Absent(a)) => assert!(a.what.contains("environment srv of lib"), "{}", a.what),
        other => panic!("answered {other:?}, not absent"),
    }
}

#[test]
fn an_empty_channel_holds_no_report_and_is_absent_naming_the_environment() {
    let Two { world, .. } = two();
    match newest_of(&world) {
        Err(Not::Absent(a)) => assert!(a.what.contains("environment srv of lib"), "{}", a.what),
        other => panic!("answered {other:?}, not absent"),
    }
}

#[test]
fn reports_of_another_environment_or_component_are_not_its_own() {
    let Two { world, first, .. } = two();
    move_to(&world, to("0.1.0"), AT);
    report(&world, DEPLOYER, ("eu", "lib"), ("0.1.0", &first), AT);
    report(&world, DEPLOYER, ("srv", "web"), ("0.1.0", &first), AT);
    assert!(matches!(newest_of(&world), Err(Not::Absent(_))));
}

// Requirement: A report counts only through a signer the policy lets
// claim one

#[test]
fn a_report_by_a_signer_that_may_not_report_counts_for_nothing_and_stays_attached() {
    let Two {
        world,
        first,
        second,
    } = two();
    move_to(&world, to("0.1.0"), AT);
    move_to(&world, to("0.2.0"), AT + HOUR);
    let counted = report(&world, DEPLOYER, ("srv", "lib"), ("0.1.0", &first), AT);
    let theirs = report(
        &world,
        "desk",
        ("srv", "lib"),
        ("0.2.0", &second),
        AT + HOUR,
    );
    assert_eq!(newest_of(&world), Ok(counted));
    let listed_without_report = RequiredClaim {
        signers: vec![named("desk", &[ClaimKind::Review])],
        ..by_deployer()
    };
    assert!(matches!(
        newest(
            &world.fakes.seams(),
            &srv(),
            &lib(),
            &stable(),
            &listed_without_report
        ),
        Err(Not::Absent(_))
    ));
    let held = world.fakes.store.claims_of(&second).unwrap();
    assert!(held.iter().any(|e| e.digest() == theirs.claim));
}

#[test]
fn claims_the_store_cannot_read_are_unreadable_naming_the_index_in_its_own_words() {
    let Two { world, first, .. } = two();
    move_to(&world, to("0.1.0"), AT);
    report(&world, DEPLOYER, ("srv", "lib"), ("0.1.0", &first), AT);
    world.fakes.store.fault(
        StoreOp::ClaimsOf,
        Fault::Unreadable(WORDS.into()),
        Times::Once,
    );
    match newest_of(&world) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source.seam, Seam::Store);
            assert!(u.what.contains(first.as_str()), "{}", u.what);
            assert_eq!(u.said.unwrap().as_bytes(), WORDS.as_bytes());
        }
        other => panic!("answered {other:?}, not unreadable"),
    }
}

#[test]
fn a_chain_the_ledger_cannot_read_is_unreadable_naming_it() {
    let Two { world, first, .. } = two();
    move_to(&world, to("0.1.0"), AT);
    report(&world, DEPLOYER, ("srv", "lib"), ("0.1.0", &first), AT);
    world.fakes.keeper.unreachable(WORDS);
    match newest_of(&world) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source.seam, Seam::Keeper);
            assert!(u.what.contains("lib/stable"), "{}", u.what);
            assert_eq!(u.said.unwrap().as_bytes(), WORDS.as_bytes());
        }
        other => panic!("answered {other:?}, not unreadable"),
    }
}
