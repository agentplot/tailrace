//! Spec attestation-verification: which signer the policy names made
//! each attestation on a version, over a version published in the test.

mod common;
mod world;

use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use serde_json::json;
use tailrace_core::{verify, Published, Told, Verified, VerifyAsk};
use tailrace_fakes::{Fault, LandingOp, PolicyOp, SignerOp, StoreOp, Times};
use tailrace_seams::{
    Answer, Audience, ChannelId, ChannelName, Check, Claim, ClaimKind, ComponentName,
    DeclaredChannel, Envelope, NamedSigner, Not, Outcome, Policy, Required, RequiredClaim,
    Revision, Seam, SignerId, Store, Version, Why,
};
use world::{build_of, World, HOST, LANDING};

fn named(id: &str, kinds: &[ClaimKind], checks: &[&str]) -> NamedSigner {
    NamedSigner {
        id: SignerId::new(id),
        verified_by: format!("{id}'s key"),
        kinds: kinds.to_vec(),
        checks: checks.iter().map(|c| Check::new(*c)).collect(),
    }
}

/// Acceptance requires the host's provenance and the landing's verdict
/// of `test`.
fn acceptance() -> Required {
    Required::Acceptance {
        claims: vec![
            RequiredClaim {
                kind: ClaimKind::Provenance,
                check: None,
                signers: vec![named(HOST, &[ClaimKind::Provenance], &[])],
            },
            RequiredClaim {
                kind: ClaimKind::Verdict,
                check: Some(Check::new("test")),
                signers: vec![named(LANDING, &[ClaimKind::Verdict], &["test"])],
            },
        ],
    }
}

/// A world whose `lib` was published at its head with two verdicts of
/// the landing's on each output's build, under the acceptance policy.
fn published() -> (World, Published) {
    let world = World::new();
    for o in ["lib-pack", "lib-docs"] {
        let build = build_of(o, &world.head);
        world.verdict("test", &build, Outcome::Pass, LANDING);
        world.verdict("lint", &build, Outcome::Pass, LANDING);
    }
    world
        .fakes
        .policy
        .require(Audience::Acceptance, acceptance());
    let published = world.publish_one(&world.ask_for("lib")).unwrap();
    (world, published)
}

fn ask(version: &str) -> VerifyAsk {
    VerifyAsk {
        branch: tailrace_seams::Branch::new("main"),
        component: Some(ComponentName::new("lib")),
        version: Version::new(version),
        audience: Audience::Acceptance,
    }
}

fn verified(world: &World, ask: &VerifyAsk) -> Answer<Verified> {
    verify(&world.fakes.seams(), &world.history(), ask)
}

fn provenances(verified: &Verified) -> Vec<&Told> {
    verified
        .claims
        .iter()
        .filter(|t| t.kind == Some(ClaimKind::Provenance))
        .collect()
}

fn carried(verified: &Verified) -> Vec<&Told> {
    verified
        .claims
        .iter()
        .filter(|t| t.carried_in.is_some())
        .collect()
}

// Requirement: Verify resolves the version to its index first

#[test]
fn a_published_version_names_its_index_and_every_artifact() {
    let (world, published) = published();
    let Published::Version {
        index, artifacts, ..
    } = &published
    else {
        unreachable!()
    };
    let verified = verified(&world, &ask("0.2.0")).unwrap();
    assert_eq!(&verified.index, index);
    assert_eq!(verified.tag, "lib-v0.2.0");
    assert_eq!(verified.revision, world.head);
    assert_eq!(
        verified
            .artifacts
            .iter()
            .map(|a| &a.digest)
            .collect::<Vec<_>>(),
        artifacts.iter().map(|a| &a.digest).collect::<Vec<_>>()
    );
}

#[test]
fn a_version_never_published_is_absent_naming_its_name() {
    let (world, _) = published();
    let Err(Not::Absent(a)) = verified(&world, &ask("9.9.9")) else {
        panic!("a version never published was verified")
    };
    assert!(a.what.contains("lib-v9.9.9"), "{}", a.what);
    assert_eq!(a.source.seam, Seam::Store);
}

#[test]
fn several_components_and_none_named_is_refused_naming_them() {
    let (world, _) = published();
    let Err(Not::Refused(r)) = verified(
        &world,
        &VerifyAsk {
            component: None,
            ..ask("0.2.0")
        },
    ) else {
        panic!("a version of no component was verified")
    };
    assert_eq!(
        r.why,
        Why::WhichComponent {
            declared: vec![ComponentName::new("lib"), ComponentName::new("web")]
        }
    );
}

// Requirement: Verify reads the policy at the version's landed revision

/// A policy that delegates to the fake and records, at each asking, how
/// many times the landing had said yes.
struct Watched {
    inner: tailrace_fakes::FakePolicy,
    landing: tailrace_fakes::FakeLanding,
    seen: Rc<RefCell<Vec<Vec<(Revision, bool)>>>>,
}

impl Policy for Watched {
    fn policy_for(&self, revision: &Revision, audience: &Audience) -> Answer<Required> {
        let yeses = self
            .landing
            .calls()
            .into_iter()
            .filter(|c| c.op == LandingOp::Landed)
            .map(|c| {
                (
                    serde_json::from_value(c.given["revision"].clone()).unwrap(),
                    c.answer == Ok(json!(true)),
                )
            })
            .collect();
        self.seen.borrow_mut().push(yeses);
        self.inner.policy_for(revision, audience)
    }

    fn channels(&self, revision: &Revision) -> Answer<Vec<DeclaredChannel>> {
        self.inner.channels(revision)
    }
}

#[test]
fn the_policy_is_read_once_for_acceptance_at_the_tags_revision_after_the_landing() {
    let (world, _) = published();
    let later = world.repo.touch("feat: lib again", &["crates/lib"]);
    world.fakes.landing.land(&world.main, &later);
    let seen = Rc::new(RefCell::new(Vec::new()));
    let mut seams = world.fakes.seams();
    seams.policy = Box::new(Watched {
        inner: world.fakes.policy.clone(),
        landing: world.fakes.landing.clone(),
        seen: seen.clone(),
    });
    let before = world.fakes.policy.calls().len();
    let verified = verify(&seams, &world.history(), &ask("0.2.0")).unwrap();
    assert_eq!(verified.revision, world.head);

    let asked = world.fakes.policy.calls()[before..].to_vec();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].op, PolicyOp::PolicyFor);
    assert_eq!(asked[0].given["revision"], world.head.as_str());
    assert_eq!(asked[0].given["audience"], json!("acceptance"));
    let seen = seen.borrow();
    assert!(
        seen[0].contains(&(world.head.clone(), true)),
        "the policy was asked before the landing said yes: {seen:?}"
    );
}

#[test]
fn no_policy_for_the_audience_is_absent_naming_it_and_the_revision() {
    let (world, _) = published();
    let Err(Not::Absent(a)) = verified(
        &world,
        &VerifyAsk {
            audience: Audience::Channel(ChannelId::new(None, ChannelName::new("beta"))),
            ..ask("0.2.0")
        },
    ) else {
        panic!("a version was verified under no policy")
    };
    assert!(a.what.contains("beta"), "{}", a.what);
    assert!(a.what.contains(world.head.as_str()), "{}", a.what);
    assert_eq!(a.source.seam, Seam::Policy);
}

#[test]
fn a_channel_named_alone_is_the_versions_components() {
    let (world, _) = published();
    let libs = ChannelId::new(Some(ComponentName::new("lib")), ChannelName::new("beta"));
    world.fakes.policy.require(
        Audience::Channel(libs.clone()),
        Required::Channel {
            claims: vec![],
            bound_to: None,
            promoted_by: vec![],
        },
    );
    let verified = verified(
        &world,
        &VerifyAsk {
            audience: Audience::Channel(ChannelId::new(None, ChannelName::new("beta"))),
            ..ask("0.2.0")
        },
    )
    .unwrap();
    assert_eq!(verified.audience, Audience::Channel(libs.clone()));
    let asked = world.fakes.policy.calls();
    let policy_for = asked
        .iter()
        .find(|c| c.op == PolicyOp::PolicyFor)
        .expect("the policy for the channel was asked");
    assert_eq!(
        policy_for.given["audience"],
        serde_json::to_value(Audience::Channel(libs)).unwrap()
    );
}

// Requirement: Each attestation is told by the signer that made it

#[test]
fn a_provenance_and_its_carried_verdicts_are_told_by_their_signers() {
    let (world, published) = published();
    let Published::Version { artifacts, .. } = &published else {
        unreachable!()
    };
    let verified = verified(&world, &ask("0.2.0")).unwrap();
    let provenances = provenances(&verified);
    assert_eq!(provenances.len(), 2);
    for (told, artifact) in provenances.iter().zip(artifacts) {
        assert_eq!(told.on, artifact.digest);
        assert_eq!(told.envelope, artifact.claims[0].envelope);
        assert_eq!(told.made_by, Some(SignerId::new(HOST)));
        assert!(told.counts && !told.expired);
    }
    let carried = carried(&verified);
    assert_eq!(carried.len(), 4);
    for told in &carried {
        assert_eq!(told.made_by, Some(SignerId::new(LANDING)));
        assert_eq!(told.kind, Some(ClaimKind::Verdict));
        let check = told.check.as_ref().unwrap().as_str();
        assert_eq!(told.counts, check == "test", "{told:?}");
    }
    assert_eq!(carried[0].carried_in, Some(provenances[0].envelope.clone()));
}

// Requirement: An expired signer is reported as such

#[test]
fn a_signer_whose_identity_expired_is_named_and_marked_expired() {
    let (world, _) = published();
    world.fakes.signer.expire(&SignerId::new(LANDING));
    let verified = verified(&world, &ask("0.2.0")).unwrap();
    let carried = carried(&verified);
    assert!(!carried.is_empty());
    for told in carried {
        assert_eq!(told.made_by, Some(SignerId::new(LANDING)));
        assert!(told.expired);
        if told.check == Some(Check::new("test")) {
            assert!(told.counts, "expiry alone stopped it counting");
        }
    }
    for told in provenances(&verified) {
        assert!(!told.expired);
    }
}

// Requirement: A claim from no named signer is kept and never counted

#[test]
fn a_verdict_from_a_signer_the_policy_names_for_nothing_is_kept_and_not_counted() {
    let world = World::new();
    let pack = build_of("lib-pack", &world.head);
    world.verdict("test", &pack, Outcome::Pass, LANDING);
    let stranger = world.verdict("test", &pack, Outcome::Pass, "stranger");
    world.verdict(
        "test",
        &build_of("lib-docs", &world.head),
        Outcome::Pass,
        LANDING,
    );
    world
        .fakes
        .policy
        .require(Audience::Acceptance, acceptance());
    let published = world.publish_one(&world.ask_for("lib")).unwrap();
    let Published::Version { artifacts, .. } = &published else {
        unreachable!()
    };

    let verified = verified(&world, &ask("0.2.0")).unwrap();
    let theirs: Vec<&Told> = verified
        .claims
        .iter()
        .filter(|t| t.envelope == stranger.envelope.digest())
        .collect();
    assert_eq!(theirs.len(), 1);
    assert_eq!(theirs[0].made_by, None);
    assert!(!theirs[0].counts);
    assert_eq!(
        theirs[0].checked,
        [SignerId::new(HOST), SignerId::new(LANDING)]
    );
    let held = world.fakes.store.claims_of(&artifacts[0].digest).unwrap();
    let provenance =
        tailrace_core::Provenance::of(&Claim::from_envelope(&held[0]).unwrap()).unwrap();
    assert!(provenance
        .carried
        .iter()
        .any(|c| c.envelope == stranger.envelope));
}

#[test]
fn a_signer_beyond_its_reach_does_not_count() {
    let (world, published) = published();
    let Published::Version { artifacts, .. } = &published else {
        unreachable!()
    };
    let claim = Claim {
        subject: artifacts[0].digest.clone(),
        kind: ClaimKind::Test,
        revision: world.head.clone(),
        build: Some(build_of("lib-pack", &world.head)),
        body: json!({ "suite": "theirs" }),
    };
    let envelope =
        tailrace_seams::Signer::sign(&world.fakes.signer, &claim, &SignerId::new(HOST)).unwrap();
    world
        .fakes
        .store
        .attach(&artifacts[0].digest, &envelope)
        .unwrap();

    let verified = verified(&world, &ask("0.2.0")).unwrap();
    let told = verified
        .claims
        .iter()
        .find(|t| t.envelope == envelope.digest())
        .unwrap();
    assert_eq!(told.kind, Some(ClaimKind::Test));
    assert_eq!(told.made_by, Some(SignerId::new(HOST)));
    assert!(!told.counts);
}

#[test]
fn a_changed_attestation_is_made_by_no_named_signer() {
    let (world, published) = published();
    let Published::Version { artifacts, .. } = &published else {
        unreachable!()
    };
    let held = world.fakes.store.claims_of(&artifacts[0].digest).unwrap();
    let mut claim = Claim::from_envelope(&held[0]).unwrap();
    claim.revision = Revision::new("0000000");
    let changed = Envelope {
        payload: claim.payload(),
        ..held[0].clone()
    };
    world
        .fakes
        .store
        .attach(&artifacts[0].digest, &changed)
        .unwrap();

    let verified = verified(&world, &ask("0.2.0")).unwrap();
    let told = verified
        .claims
        .iter()
        .find(|t| t.envelope == changed.digest())
        .unwrap();
    assert_eq!(told.made_by, None);
    assert!(!told.counts);
}

// Requirement: A failed read is not an empty answer

#[test]
fn claims_that_cannot_be_read_are_unreadable_with_the_stores_words() {
    let (world, _) = published();
    world.fakes.store.fault(
        StoreOp::ClaimsOf,
        Fault::Unreadable("connection refused".into()),
        Times::UntilHealed,
    );
    let Err(Not::Unreadable(u)) = verified(&world, &ask("0.2.0")) else {
        panic!("claims that cannot be read were answered")
    };
    assert_eq!(u.said.as_deref(), Some("connection refused"));
    assert_eq!(u.source.seam, Seam::Store);
}

// Requirement: Verify writes nothing

#[test]
fn a_verify_puts_attaches_names_signs_and_records_nothing() {
    let (world, _) = published();
    let store = world.fakes.store.calls().len();
    let signer = world.fakes.signer.calls().len();
    let record: Vec<(std::path::PathBuf, Vec<u8>)> = fs::read_dir(world.kept().dir().join("lib"))
        .unwrap()
        .map(|e| {
            let path = e.unwrap().path();
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect();
    let tags = world.repo.git(&["tag", "-l"]);

    verified(&world, &ask("0.2.0")).unwrap();
    for call in &world.fakes.store.calls()[store..] {
        assert!(
            matches!(
                call.op,
                StoreOp::Resolve | StoreOp::Read | StoreOp::ClaimsOf
            ),
            "verify asked the store {:?}",
            call.op
        );
    }
    for call in &world.fakes.signer.calls()[signer..] {
        assert_eq!(call.op, SignerOp::Verify);
    }
    let now: Vec<(std::path::PathBuf, Vec<u8>)> = fs::read_dir(world.kept().dir().join("lib"))
        .unwrap()
        .map(|e| {
            let path = e.unwrap().path();
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect();
    assert_eq!(now, record);
    assert_eq!(world.repo.git(&["tag", "-l"]), tags);
}
