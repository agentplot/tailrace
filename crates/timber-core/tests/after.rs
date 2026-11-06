//! Spec environment-reports: an environment's newest report read across
//! its channel's chain over the fakes, two versions of `lib` published in
//! the test, each carrying the gate's acceptance and a second signer's
//! rebuild, and `stable` promoted between them (record 0028's design D8),
//! its reports counted newest first (design D3).

mod common;
mod world;

use std::cell::Cell;
use std::rc::Rc;

use serde_json::{json, Value};
use timber_core::{newest, promote, Newest, PromoteAsk, Published, Reported, To};
use timber_fakes::{FakeSigner, Fault, StoreOp, Times};
use timber_seams::{
    Answer, AskedOf, Audience, Branch, ChannelId, ChannelName, Claim, ClaimKind, ComponentName,
    Digest, Envelope, EnvironmentName, Made, NamedSigner, Not, Outcome, OutputName, Report,
    Required, RequiredClaim, Seam, Seams, Signer, SignerId, Source, Store, Version, Why,
};
use world::{build_of, version_of, World};

/// The identity the host signs promotions as, which the policy names
/// for acceptance and promotion (108).
const GATE: &str = "gate";

/// The second signer, whose rebuild the policy for `stable` requires.
const REBUILD: &str = "rebuild";

/// The signer the policy lets claim a report.
const DEPLOYER: &str = "deployer";

/// A signer the policy lets claim a report, listed by a key only a key
/// service could tell its signature by.
const KMS: &str = "kms-deployer";

/// The key the policy lists [`KMS`] by.
const KEY: &str = "arn:aws:kms:us-east-1:111122223333:key/0b7c1f9e";

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

/// The signers the policy lets claim a report: the deployer.
fn reporters() -> Vec<NamedSigner> {
    vec![named(DEPLOYER, &[ClaimKind::Report])]
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

fn newest_of_srv(world: &World) -> Answer<Newest> {
    newest(
        &world.fakes.seams(),
        &srv(),
        &lib(),
        &stable(),
        &reporters(),
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
    assert_eq!(newest_of_srv(&world), Ok(back));
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
    assert_eq!(newest_of_srv(&world), Ok(still));
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
    assert!(newest_of_srv(&world).is_ok());
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
    match newest_of_srv(&world) {
        Err(Not::Absent(a)) => assert!(a.what.contains("environment srv of lib"), "{}", a.what),
        other => panic!("answered {other:?}, not absent"),
    }
}

#[test]
fn an_empty_channel_holds_no_report_and_is_absent_naming_the_environment() {
    let Two { world, .. } = two();
    match newest_of_srv(&world) {
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
    assert!(matches!(newest_of_srv(&world), Err(Not::Absent(_))));
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
    assert_eq!(newest_of_srv(&world), Ok(counted));
    let listed_without_report = vec![named("desk", &[ClaimKind::Review])];
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
    match newest_of_srv(&world) {
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
    match newest_of_srv(&world) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source.seam, Seam::Keeper);
            assert!(u.what.contains("lib/stable"), "{}", u.what);
            assert_eq!(u.said.unwrap().as_bytes(), WORDS.as_bytes());
        }
        other => panic!("answered {other:?}, not unreadable"),
    }
}

#[test]
fn a_policy_letting_no_signer_report_is_absent_saying_so() {
    let Two { world, first, .. } = two();
    move_to(&world, to("0.1.0"), AT);
    report(&world, DEPLOYER, ("srv", "lib"), ("0.1.0", &first), AT);
    match newest(&world.fakes.seams(), &srv(), &lib(), &stable(), &[]) {
        Err(Not::Absent(a)) => {
            assert!(a.what.contains("environment srv of lib"), "{}", a.what);
            assert!(
                a.what.contains("the policy lets no signer claim a report"),
                "{}",
                a.what
            );
        }
        other => panic!("answered {other:?}, not absent"),
    }
}

// Requirement: An environment's newest report: told newest first

#[test]
fn two_reports_at_one_moment_answer_the_later_in_chain_order() {
    let Two {
        world,
        first,
        second,
    } = two();
    move_to(&world, to("0.1.0"), AT);
    move_to(&world, to("0.2.0"), AT + HOUR);
    report(
        &world,
        DEPLOYER,
        ("srv", "lib"),
        ("0.1.0", &first),
        AT + HOUR,
    );
    let later = report(
        &world,
        DEPLOYER,
        ("srv", "lib"),
        ("0.2.0", &second),
        AT + HOUR,
    );
    assert_eq!(newest_of_srv(&world), Ok(later));

    // On one index, the one attached later.
    let again = report(
        &world,
        DEPLOYER,
        ("srv", "lib"),
        ("0.2.0", &second),
        AT + HOUR,
    );
    assert_eq!(newest_of_srv(&world), Ok(again));
}

/// A key service the status binding reaches no further than: it answers
/// unreadable, naming [`KEY`], for an envelope signed under it, and
/// refuses any other; counting each envelope it is asked of.
struct KeyService {
    asked: Rc<Cell<usize>>,
}

impl Signer for KeyService {
    fn sign(&self, _: &Claim, identity: &SignerId) -> Answer<Envelope> {
        Err(Not::absent(
            &Source::new(Seam::Signer, "kms"),
            format!("the identity {identity}"),
        ))
    }

    fn verify(&self, attestation: &Envelope, signers: &[NamedSigner]) -> Answer<Made> {
        self.asked.set(self.asked.get() + 1);
        let source = Source::new(Seam::Signer, "kms");
        if attestation.signatures.iter().any(|s| s.keyid == KEY) {
            return Err(Not::unreadable(
                &source,
                format!("the public half of {KEY}"),
                Some("status reaches no key service".into()),
            ));
        }
        Err(Not::refused(
            &source,
            Why::NoNamedSigner {
                checked: signers.iter().map(|n| n.id.clone()).collect(),
            },
            None,
        ))
    }
}

/// The fakes' seams with the key service bound after the fake signer,
/// and how many envelopes it was asked to tell.
fn with_key_service(world: &World) -> (Seams, Rc<Cell<usize>>) {
    let asked = Rc::new(Cell::new(0));
    let mut seams = world.fakes.seams();
    seams.signers.push(Box::new(KeyService {
        asked: asked.clone(),
    }));
    (seams, asked)
}

/// A report of `srv` of `lib` running `version` at `index`, from an
/// apply at `at`, signed under [`KEY`] as [`KMS`], which the fake signer
/// cannot tell, attached to the index.
fn report_under_key(world: &World, (version, index): (&str, &Digest), at: u64) -> Newest {
    let theirs = FakeSigner::new();
    let id = SignerId::new(KMS);
    theirs.hold(&id);
    let report = Reported {
        environment: srv(),
        component: lib(),
        channel: ChannelName::new("stable"),
        version: Version::new(version),
        index: index.clone(),
        runs: index.clone(),
        converged_at: "2025-12-31T00:00:00Z".into(),
        at,
    };
    let mut envelope = theirs.sign(&report.claim(world.head.clone()), &id).unwrap();
    for signature in &mut envelope.signatures {
        signature.keyid = KEY.into();
        signature.sig = b"a P-256 signature".to_vec();
    }
    world.fakes.store.attach(index, &envelope).unwrap();
    Newest {
        claim: envelope.digest(),
        report,
    }
}

/// The deployer and [`KMS`], each letting claim a report.
fn with_kms() -> Vec<NamedSigner> {
    vec![
        named(DEPLOYER, &[ClaimKind::Report]),
        NamedSigner {
            verified_by: KEY.into(),
            ..named(KMS, &[ClaimKind::Report])
        },
    ]
}

#[test]
fn an_older_report_whose_signer_cannot_be_told_is_never_told() {
    let Two {
        world,
        first,
        second,
    } = two();
    move_to(&world, to("0.1.0"), AT);
    move_to(&world, to("0.2.0"), AT + HOUR);
    report_under_key(&world, ("0.1.0", &first), AT);
    let newer = report(
        &world,
        DEPLOYER,
        ("srv", "lib"),
        ("0.2.0", &second),
        AT + HOUR,
    );
    let (seams, asked) = with_key_service(&world);
    assert_eq!(
        newest(&seams, &srv(), &lib(), &stable(), &with_kms()),
        Ok(newer)
    );
    assert_eq!(asked.get(), 0, "the older report was told");
}

#[test]
fn a_newer_report_whose_signer_cannot_be_told_is_unreadable_naming_it_and_the_key() {
    let Two {
        world,
        first,
        second,
    } = two();
    move_to(&world, to("0.1.0"), AT);
    move_to(&world, to("0.2.0"), AT + HOUR);
    report(&world, DEPLOYER, ("srv", "lib"), ("0.1.0", &first), AT);
    let newer = report_under_key(&world, ("0.2.0", &second), AT + HOUR);
    let (seams, asked) = with_key_service(&world);
    match newest(&seams, &srv(), &lib(), &stable(), &with_kms()) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, Source::new(Seam::Signer, "kms"));
            assert!(u.what.contains(newer.claim.as_str()), "{}", u.what);
            assert!(u.what.contains("environment srv of lib"), "{}", u.what);
            assert!(u.what.contains(KEY), "{}", u.what);
            assert_eq!(u.said.as_deref(), Some("status reaches no key service"));
        }
        other => panic!("answered {other:?}, not unreadable"),
    }
    assert_eq!(asked.get(), 1);
}
