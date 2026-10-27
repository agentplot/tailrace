//! The verify group: which named signer made each attestation on a
//! version, and whether it counts, S5 and S13 as far as verify's answer
//! (record 0012).

use std::cell::RefCell;
use std::rc::Rc;

use serde_json::json;
use tailrace_core::{verify, Published, Told, Verified, POLICY_FILE};
use tailrace_fakes::{FakeLanding, FakePolicy, Fault, LandingOp, PolicyOp, StoreOp, Times};
use tailrace_seams::{
    Answer, Audience, Check, Claim, ClaimKind, Envelope, Held, Manifest, NamedSigner, Not, Outcome,
    Policy, PolicyChannels, Required, RequiredClaim, Revision, Seam, SignerId, Store, Verdict,
};

use crate::core::publish::{one, passing, version_of, TAG, VERSION};
use crate::world::{build_of, FakeWorld, Write, HOST, LANDING, POLICY};
use crate::{ok, Scenario};

pub const GROUP: &str = "verify";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        Scenario {
            name: "a verdict from a signer the policy does not name",
            plays: Some("S5"),
            cites: &[3, 9, 80],
            run: a_verdict_from_a_signer_the_policy_does_not_name,
        },
        Scenario {
            name: "no landing gate",
            plays: Some("S13"),
            cites: &[9, 15, 111],
            run: no_landing_gate,
        },
        Scenario {
            name: "the policy read at the landed revision",
            plays: None,
            cites: &[65],
            run: the_policy_read_at_the_landed_revision,
        },
        Scenario {
            name: "an expired signer",
            plays: None,
            cites: &[81, 117],
            run: an_expired_signer,
        },
        Scenario {
            name: "a signer beyond its reach",
            plays: None,
            cites: &[9, 116],
            run: a_signer_beyond_its_reach,
        },
        Scenario {
            name: "a changed attestation",
            plays: None,
            cites: &[10],
            run: a_changed_attestation,
        },
        Scenario {
            name: "claims that cannot be read",
            plays: None,
            cites: &[58, 59],
            run: claims_that_cannot_be_read,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}

fn named(id: &str, kinds: &[ClaimKind], checks: &[&str]) -> NamedSigner {
    NamedSigner {
        id: SignerId::new(id),
        verified_by: format!("{id}'s key"),
        kinds: kinds.to_vec(),
        checks: checks.iter().map(|c| Check::new(*c)).collect(),
    }
}

fn provenance_from(signer: NamedSigner) -> RequiredClaim {
    RequiredClaim {
        kind: ClaimKind::Provenance,
        check: None,
        signers: vec![signer],
    }
}

fn verdict_from(signer: NamedSigner, check: &str) -> RequiredClaim {
    RequiredClaim {
        kind: ClaimKind::Verdict,
        check: Some(Check::new(check)),
        signers: vec![signer],
    }
}

/// Acceptance requires the host's provenance and the landing's verdict of
/// `test`.
fn acceptance() -> Required {
    Required::Acceptance {
        claims: vec![
            provenance_from(named(HOST, &[ClaimKind::Provenance], &[])),
            verdict_from(named(LANDING, &[ClaimKind::Verdict], &["test"]), "test"),
        ],
    }
}

/// The head published with the landing's passing verdicts, under the
/// acceptance policy.
fn published(world: &FakeWorld) -> Published {
    passing(world, &world.head);
    world
        .fakes
        .policy
        .require(Audience::Acceptance, acceptance());
    ok(one(world.publish(&world.ask(&world.head))), "the publish")
}

fn verified(world: &FakeWorld) -> Verified {
    ok(
        world.verify(&world.verify_ask(VERSION, Audience::Acceptance)),
        "verify",
    )
}

fn told<'a>(verified: &'a Verified, envelope: &Envelope) -> &'a Told {
    verified
        .claims
        .iter()
        .find(|t| t.envelope == envelope.digest())
        .unwrap_or_else(|| panic!("verify did not list {}", envelope.digest()))
}

fn provenances(verified: &Verified) -> Vec<&Told> {
    verified
        .claims
        .iter()
        .filter(|t| t.kind == Some(ClaimKind::Provenance))
        .collect()
}

fn a_verdict_from_a_signer_the_policy_does_not_name(world: &FakeWorld) {
    let head = &world.head;
    let (site, server) = (build_of("site", head), build_of("server", head));
    let stranger = world.verdict(head, "test", &site, Outcome::Pass, "stranger");
    let landing = [
        world.verdict(head, "test", &site, Outcome::Pass, LANDING),
        world.verdict(head, "test", &server, Outcome::Pass, LANDING),
    ];
    let mut verdicts: Vec<Verdict> = landing.to_vec();
    verdicts.push(stranger.clone());
    world.land(head, &verdicts);
    world.outputs(head, &[("site", Outcome::Pass), ("server", Outcome::Pass)]);
    world
        .fakes
        .policy
        .require(Audience::Acceptance, acceptance());
    let published = ok(one(world.publish(&world.ask(head))), "the publish");
    let (_, index) = version_of(&published);
    let Published::Version { artifacts, .. } = &published else {
        unreachable!()
    };
    let writes = world.writes();
    let asked = world.fakes.store.calls().len();

    let verified = verified(world);

    // 3: the name resolved to its index first, and every act on it.
    assert_eq!(&verified.index, index, "verify named another index");
    let calls = &world.fakes.store.calls()[asked..];
    assert_eq!(calls[0].op, StoreOp::Resolve, "{calls:?}");
    assert_eq!(calls[0].given["name"], TAG);

    // 9, 80: the stranger's verdict listed, made by no signer the policy
    // names, checked against those it names, and not counted.
    let theirs = told(&verified, &stranger.envelope);
    assert_eq!(theirs.kind, Some(ClaimKind::Verdict));
    assert_eq!(theirs.made_by, None, "{theirs:?}");
    assert_eq!(
        theirs.checked,
        [SignerId::new(HOST), SignerId::new(LANDING)],
        "verify checked other signers"
    );
    assert!(!theirs.counts, "a verdict from no named signer counted");
    for verdict in &landing {
        let told = told(&verified, &verdict.envelope);
        assert_eq!(told.made_by, Some(SignerId::new(LANDING)));
        assert!(told.counts, "the landing's verdict did not count: {told:?}");
    }

    // Kept attached, inside the provenance that carried it.
    let site_artifact = artifacts
        .iter()
        .find(|a| a.build == site)
        .expect("the site was published");
    let held = ok(
        world.fakes.store.claims_of(&site_artifact.digest),
        "claims-of",
    );
    let carried = tailrace_core::Provenance::of(
        &Claim::from_envelope(&held[0]).expect("a claim of tailrace's form"),
    )
    .expect("a provenance")
    .carried;
    assert!(
        carried.iter().any(|c| c.envelope == stranger.envelope),
        "the stranger's verdict is no longer kept"
    );

    // Nothing moves: no write since the publish, one name, no append.
    assert_eq!(world.writes(), writes, "verify wrote to the store");
    let names: Vec<&Write> = writes
        .iter()
        .filter(|w| matches!(w, Write::Name { .. }))
        .collect();
    assert_eq!(names.len(), 1, "a name but the version's moved: {names:?}");
    assert_eq!(world.keeper_calls(), [], "the keeper was asked");
    assert_eq!(world.target_calls(), [], "a target was asked");

    // The version reads back by digest as it was offered.
    let offered = writes
        .iter()
        .find_map(|w| match w {
            Write::Manifest(m @ Manifest::Index(_)) => Some(m.clone()),
            _ => None,
        })
        .expect("an index was offered");
    assert_eq!(
        world.fakes.store.read(index),
        Ok(Held::Manifest(offered)),
        "the index read back is not the index offered"
    );
}

fn no_landing_gate(world: &FakeWorld) {
    world.land(&world.head, &[]);
    world.outputs(
        &world.head,
        &[("site", Outcome::Pass), ("server", Outcome::Pass)],
    );
    let published = ok(one(world.publish(&world.ask(&world.head))), "the publish");
    let Published::Version { artifacts, .. } = &published else {
        panic!("a revision with no verdicts was not offered: {published:?}")
    };
    assert_eq!(
        artifacts.len(),
        2,
        "not every output was offered on its pass"
    );

    let policy = &world.fakes.policy;
    policy.require(
        Audience::Acceptance,
        Required::Acceptance {
            claims: vec![verdict_from(
                named(LANDING, &[ClaimKind::Verdict], &["test"]),
                "test",
            )],
        },
    );
    let requiring_a_verdict = verified(world);
    assert!(
        !requiring_a_verdict
            .claims
            .iter()
            .any(|t| t.kind == Some(ClaimKind::Verdict)),
        "a verdict was found where the landing carried none: {:?}",
        requiring_a_verdict.claims
    );
    assert!(
        !requiring_a_verdict.claims.iter().any(|t| t.counts),
        "a claim counted under a policy naming none of its signers"
    );

    policy.require(
        Audience::Acceptance,
        Required::Acceptance {
            claims: vec![provenance_from(named(HOST, &[ClaimKind::Provenance], &[]))],
        },
    );
    let requiring_provenance = verified(world);
    // The two outputs' provenances and the notes'.
    let counted = provenances(&requiring_provenance);
    assert_eq!(counted.len(), 3, "{counted:?}");
    for told in counted {
        assert!(told.counts, "the host's provenance did not count: {told:?}");
    }
}

/// A policy that hands every asking to its fake and keeps, at each, what
/// the landing had answered of each revision by then.
struct Watched {
    inner: FakePolicy,
    landing: FakeLanding,
    seen: Rc<RefCell<Vec<Vec<(Revision, bool)>>>>,
}

impl Policy for Watched {
    fn policy_for(&self, revision: &Revision, audience: &Audience) -> Answer<Required> {
        let answered = self
            .landing
            .calls()
            .into_iter()
            .filter(|c| c.op == LandingOp::Landed)
            .map(|c| {
                (
                    serde_json::from_value(c.given["revision"].clone()).expect("a revision"),
                    c.answer.as_ref().is_ok_and(|a| a["landed"] == json!(true)),
                )
            })
            .collect();
        self.seen.borrow_mut().push(answered);
        self.inner.policy_for(revision, audience)
    }

    fn channels(&self, revision: &Revision) -> Answer<PolicyChannels> {
        self.inner.channels(revision)
    }
}

fn the_policy_read_at_the_landed_revision(world: &FakeWorld) {
    published(world);
    let repo = world.repo();
    let later = repo.commit(
        &[(POLICY_FILE, &format!("{POLICY}\n# A later rule.\n"))],
        "chore: a later policy",
    );
    world.land(&later, &[]);
    repo.write(POLICY_FILE, "this checkout's policy is [not a policy");

    let seen = Rc::new(RefCell::new(Vec::new()));
    let mut seams = world.seams();
    seams.policy = Box::new(Watched {
        inner: world.fakes.policy.clone(),
        landing: world.fakes.landing.clone(),
        seen: seen.clone(),
    });
    let before = world.fakes.policy.calls().len();
    let verified = ok(
        verify(
            &seams,
            &world.history(),
            &world.verify_ask(VERSION, Audience::Acceptance),
        ),
        "verify",
    );
    assert_eq!(
        verified.revision, world.head,
        "verify read another revision"
    );

    let asked = &world.fakes.policy.calls()[before..];
    assert_eq!(asked.len(), 1, "the policy was asked {} times", asked.len());
    assert_eq!(asked[0].op, PolicyOp::PolicyFor);
    assert_eq!(
        asked[0].given["revision"],
        world.head.as_str(),
        "the policy was read at another revision than the version's"
    );
    assert_eq!(asked[0].given["audience"], json!("acceptance"));
    let seen = seen.borrow();
    assert!(
        seen[0].contains(&(world.head.clone(), true)),
        "the policy was asked before the landing said the revision is landed: {seen:?}"
    );
}

fn an_expired_signer(world: &FakeWorld) {
    published(world);
    world.fakes.signer.expire(&SignerId::new(HOST));
    let verified = verified(world);
    // The two outputs' provenances and the notes'.
    let told = provenances(&verified);
    assert_eq!(told.len(), 3, "{told:?}");
    for told in told {
        assert_eq!(told.made_by, Some(SignerId::new(HOST)), "{told:?}");
        assert!(told.expired, "an expired signer was not told as expired");
        assert!(
            told.counts,
            "a claim stopped counting when its signer expired"
        );
    }
}

fn a_signer_beyond_its_reach(world: &FakeWorld) {
    published(world);
    let policy = &world.fakes.policy;
    let carried = |verified: &Verified| -> Vec<Told> {
        verified
            .claims
            .iter()
            .filter(|t| t.carried_in.is_some())
            .cloned()
            .collect()
    };

    // The landing named for provenance alone.
    policy.require(
        Audience::Acceptance,
        Required::Acceptance {
            claims: vec![provenance_from(named(
                LANDING,
                &[ClaimKind::Provenance],
                &[],
            ))],
        },
    );
    let verdicts = carried(&verified(world));
    assert_eq!(verdicts.len(), 2, "{verdicts:?}");
    for told in &verdicts {
        assert_eq!(told.made_by, Some(SignerId::new(LANDING)), "{told:?}");
        assert!(
            !told.counts,
            "a verdict counted from a signer named for provenance"
        );
    }

    // The landing named for verdicts of another check.
    policy.require(
        Audience::Acceptance,
        Required::Acceptance {
            claims: vec![verdict_from(
                named(LANDING, &[ClaimKind::Verdict], &["lint"]),
                "lint",
            )],
        },
    );
    for told in &carried(&verified(world)) {
        assert_eq!(told.check, Some(Check::new("test")));
        assert_eq!(told.made_by, Some(SignerId::new(LANDING)), "{told:?}");
        assert!(
            !told.counts,
            "a verdict counted for a check its signer may not claim"
        );
    }
}

fn a_changed_attestation(world: &FakeWorld) {
    let published = published(world);
    let Published::Version { artifacts, .. } = &published else {
        unreachable!()
    };
    let store = &world.fakes.store;
    let subject = &artifacts[0].digest;
    let held = ok(store.claims_of(subject), "claims-of")[0].clone();
    let mut claim = Claim::from_envelope(&held).expect("a claim of tailrace's form");
    claim.revision = Revision::new("0".repeat(40));
    let changed = Envelope {
        payload: claim.payload(),
        ..held.clone()
    };
    ok(store.attach(subject, &changed), "attach");

    let verified = verified(world);
    let told = told(&verified, &changed);
    assert_eq!(
        told.made_by, None,
        "a changed attestation was told as signed"
    );
    assert!(!told.counts, "a changed attestation counted");
    let after = ok(store.claims_of(subject), "claims-of");
    assert!(
        after.contains(&held) && after.contains(&changed),
        "an attestation left the digest: {after:?}"
    );
}

fn claims_that_cannot_be_read(world: &FakeWorld) {
    published(world);
    world.fakes.store.fault(
        StoreOp::ClaimsOf,
        Fault::Unreadable("connection refused".into()),
        Times::UntilHealed,
    );
    match world.verify(&world.verify_ask(VERSION, Audience::Acceptance)) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source.seam, Seam::Store, "{u:?}");
            assert_eq!(
                u.said.as_deref(),
                Some("connection refused"),
                "the store's words were not carried unchanged"
            );
        }
        other => panic!("verify of claims that cannot be read answered {other:?}"),
    }
}
