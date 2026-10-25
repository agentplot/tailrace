//! Spec acceptance: the gate's judgment of a version published in the
//! test, under the policy for acceptance at the branch's landed head, and
//! the channels that follow each publish.

mod common;
mod world;

use serde_json::json;
use tailrace_core::{
    accept, answer_json, AcceptAsk, Followed, Judged, Moved, Published, ToldBy, WhyWanting,
};
use tailrace_fakes::{FakeSigner, KeeperOp, PolicyOp, SignerOp, StoreOp};
use tailrace_seams::{
    Answer, Audience, Branch, ChannelName, Check, ClaimKind, ComponentName, Digest, NamedSigner,
    Not, Outcome, Required, RequiredClaim, Revision, Seam, Seams, SignerId, Source, Version, Why,
};
use world::{build_of, version_of, World, HOST, LANDING};

/// The identity the gate signs as, held by a signer of its own beside
/// the host's.
const GATE: &str = "gate";

/// The moment every judgment here is made at.
const WHEN: u64 = 1_767_225_600;

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

/// Acceptance requires the host's provenance alone.
fn provenance_only() -> Required {
    Required::Acceptance {
        claims: vec![provenance_from(named(HOST, &[ClaimKind::Provenance], &[]))],
    }
}

/// A world whose `lib` was published at its head, each output's build
/// carrying the landing's passing `test` and `lint`, under `required`.
fn published_under(required: Required) -> (World, Published) {
    let world = World::new();
    for o in ["lib-pack", "lib-docs"] {
        let build = build_of(o, &world.head);
        world.verdict("test", &build, Outcome::Pass, LANDING);
        world.verdict("lint", &build, Outcome::Pass, LANDING);
    }
    world.fakes.policy.require(Audience::Acceptance, required);
    let published = world.publish_one(&world.ask_for("lib")).unwrap();
    (world, published)
}

fn ask(version: &str, told: ToldBy) -> AcceptAsk {
    AcceptAsk {
        branch: Branch::new("main"),
        component: Some(ComponentName::new("lib")),
        version: Version::new(version),
        told,
        gate: SignerId::new(GATE),
        when: WHEN,
    }
}

/// The world's fakes bound, with a signer of its own holding each of
/// `holding` after the host's.
fn holding(world: &World, holding: &[&str]) -> Seams {
    let mut seams = world.fakes.seams();
    for id in holding {
        let signer = FakeSigner::new();
        signer.hold(&SignerId::new(*id));
        seams.signers.push(Box::new(signer));
    }
    seams
}

/// The gate's judgment of `ask`, the gate's identity at hand.
fn judged(world: &World, ask: &AcceptAsk) -> Answer<Judged> {
    accept(&holding(world, &[GATE]), &world.history(), ask)
}

/// How many writes the store, the keeper and the signer were asked from
/// the counts `before`.
fn writes_since(world: &World, before: (usize, usize, usize)) -> Vec<String> {
    let store = world.fakes.store.calls()[before.0..]
        .iter()
        .filter(|c| {
            matches!(
                c.op,
                StoreOp::PutBytes | StoreOp::PutArtifact | StoreOp::Attach | StoreOp::Name
            )
        })
        .map(|c| format!("store {:?}", c.op))
        .collect::<Vec<_>>();
    let keeper = world.fakes.keeper.calls()[before.1..]
        .iter()
        .filter(|c| c.op == KeeperOp::Append)
        .map(|c| format!("keeper {:?}", c.op))
        .collect::<Vec<_>>();
    let signer = world.fakes.signer.calls()[before.2..]
        .iter()
        .filter(|c| c.op == SignerOp::Sign)
        .map(|c| format!("signer {:?}", c.op))
        .collect::<Vec<_>>();
    [store, keeper, signer].concat()
}

fn counts(world: &World) -> (usize, usize, usize) {
    (
        world.fakes.store.calls().len(),
        world.fakes.keeper.calls().len(),
        world.fakes.signer.calls().len(),
    )
}

// Requirement: The gate judges a version by its index's digest

#[test]
fn each_answer_renders_in_the_one_form_naming_the_index_and_both_revisions() {
    let (index, revision, head) = (
        Digest::of(b"index"),
        Revision::new("4f2a9c1"),
        Revision::new("7b3e0d2"),
    );
    let wanting = RequiredClaim {
        kind: ClaimKind::Verdict,
        check: Some(tailrace_seams::Check::new("test")),
        signers: vec![],
    };
    let promotion = Digest::of(b"promotion");
    let accepted: Answer<Judged> = Ok(Judged::Accepted {
        component: ComponentName::new("lib"),
        version: Version::new("0.2.0"),
        index: index.clone(),
        revision: revision.clone(),
        policy_at: head.clone(),
        acceptance: Digest::of(b"acceptance"),
        already: false,
        one_host: false,
        channels: vec![
            (
                ChannelName::new("beta"),
                Ok(Moved::Promoted {
                    from: None,
                    to: index.clone(),
                    promotion: promotion.clone(),
                    name: Followed::Moved { to: index.clone() },
                }),
            ),
            (
                ChannelName::new("edge"),
                Ok(Moved::Held {
                    artifact: Digest::of(b"artifact"),
                    wanting: wanting.clone(),
                    why: WhyWanting::Missing,
                }),
            ),
            (
                ChannelName::new("next"),
                Err(Not::refused(
                    &Source::new(Seam::Keeper, "bound"),
                    Why::NotHead {
                        head: Some(promotion.clone()),
                    },
                    None,
                )),
            ),
        ],
    });
    let json = answer_json(&accepted);
    assert_eq!(json["answer"], "ok");
    let value = &json["value"];
    assert_eq!(value["judged"], "accepted");
    assert_eq!(value["index"], index.as_str());
    assert_eq!(value["revision"], revision.as_str());
    assert_eq!(value["policy_at"], head.as_str());
    let channels = value["channels"].as_array().unwrap();
    assert_eq!(channels[0]["channel"], "beta");
    assert_eq!(channels[0]["answer"], "ok");
    assert_eq!(channels[0]["value"]["moved"], "promoted");
    assert_eq!(channels[0]["value"]["promotion"], promotion.as_str());
    assert_eq!(channels[1]["value"]["moved"], "held");
    assert_eq!(channels[1]["value"]["why"], "missing");
    assert_eq!(channels[2]["answer"], "refused");
    assert_eq!(
        channels[2]["refused"]["why"],
        json!({ "not_head": { "head": promotion.as_str() } })
    );

    let refused: Answer<Judged> = Ok(Judged::Refused {
        component: ComponentName::new("lib"),
        version: Version::new("0.2.0"),
        index: index.clone(),
        revision: revision.clone(),
        policy_at: head.clone(),
        artifact: Digest::of(b"artifact"),
        wanting,
        why: WhyWanting::Expired {
            signer: SignerId::new("landing"),
        },
    });
    let value = &answer_json(&refused)["value"];
    assert_eq!(value["judged"], "refused");
    assert_eq!(value["index"], index.as_str());
    assert_eq!(value["revision"], revision.as_str());
    assert_eq!(value["policy_at"], head.as_str());
    assert_eq!(value["wanting"]["kind"], "verdict");
    assert_eq!(value["why"], json!({ "expired": { "signer": "landing" } }));
}

#[test]
fn the_answer_names_the_index_the_name_resolves_to_and_reads_nothing_before() {
    let (world, published) = published_under(provenance_only());
    let (_, index) = version_of(&published);
    let before = world.fakes.store.calls().len();
    let Ok(Judged::Accepted {
        index: judged,
        revision,
        policy_at,
        ..
    }) = judged(&world, &ask("0.2.0", ToldBy::Asked))
    else {
        panic!("the version was not accepted")
    };
    assert_eq!(&judged, index);
    assert_eq!(revision, world.head);
    assert_eq!(policy_at, world.head);
    let ops: Vec<StoreOp> = world.fakes.store.calls()[before..]
        .iter()
        .map(|c| c.op)
        .collect();
    assert_eq!(ops[0], StoreOp::Resolve, "{ops:?}");
}

#[test]
fn a_version_never_offered_is_absent_naming_its_tag_and_nothing_is_written() {
    let (world, _) = published_under(provenance_only());
    let before = counts(&world);
    let Err(Not::Absent(a)) = judged(&world, &ask("9.9.9", ToldBy::Asked)) else {
        panic!("a version never offered was judged")
    };
    assert!(a.what.contains("lib-v9.9.9"), "{}", a.what);
    assert_eq!(a.source.seam, Seam::Store);
    assert_eq!(writes_since(&world, before), Vec::<String>::new());
}

// Requirement: The policy is read at the governed branch's landed head

#[test]
fn a_branch_head_not_landed_is_refused_and_nothing_is_read_of_the_policy_or_written() {
    let (world, _) = published_under(provenance_only());
    let later = world.repo.touch("feat: lib again", &["crates/lib"]);
    let (before, policy) = (counts(&world), world.fakes.policy.calls().len());
    let Err(Not::Refused(r)) = judged(&world, &ask("0.2.0", ToldBy::Asked)) else {
        panic!("a head the landing has not landed was judged")
    };
    assert_eq!(
        r.why,
        Why::NotLanded {
            branch: Branch::new("main"),
            revision: later,
        }
    );
    assert_eq!(world.fakes.policy.calls().len(), policy);
    assert_eq!(writes_since(&world, before), Vec::<String>::new());
}

#[test]
fn a_policy_landed_after_the_publish_is_the_one_read() {
    let (world, _) = published_under(provenance_only());
    let later = world.repo.touch("chore: stricter", &["docs"]);
    world.fakes.landing.land(&world.main, &later);
    let review = RequiredClaim {
        kind: ClaimKind::Review,
        check: None,
        signers: vec![named("reviewer", &[ClaimKind::Review], &[])],
    };
    world.fakes.policy.require_at(
        &later,
        Audience::Acceptance,
        Required::Acceptance {
            claims: vec![
                provenance_from(named(HOST, &[ClaimKind::Provenance], &[])),
                review.clone(),
            ],
        },
    );
    let before = (counts(&world), world.fakes.policy.calls().len());
    let Ok(Judged::Refused {
        revision,
        policy_at,
        wanting,
        why,
        ..
    }) = judged(&world, &ask("0.2.0", ToldBy::Asked))
    else {
        panic!("the version was not refused under the later policy")
    };
    assert_eq!(revision, world.head);
    assert_eq!(policy_at, later);
    assert_eq!(wanting, review);
    assert_eq!(why, WhyWanting::Missing);
    let asked = world.fakes.policy.calls()[before.1..].to_vec();
    assert!(!asked.is_empty());
    for call in asked {
        assert_eq!(call.op, PolicyOp::PolicyFor);
        assert_eq!(call.given["revision"], later.as_str(), "{:?}", call.given);
    }
    assert_eq!(writes_since(&world, before.0), Vec::<String>::new());
}

// Requirement: The gate judges only a landed revision's version

#[test]
fn a_version_whose_revision_did_not_land_is_refused_naming_it_and_nothing_is_attached() {
    let world = World::new();
    world
        .fakes
        .policy
        .require(Audience::Acceptance, provenance_only());
    let published = world.publish_one(&world.ask_for("lib")).unwrap();
    version_of(&published);
    let later = world.repo.touch("feat: lib again", &["crates/lib"]);
    let landing = tailrace_fakes::FakeLanding::new();
    landing.land(&world.main, &later);
    let mut seams = holding(&world, &[GATE]);
    seams.landing = Box::new(landing);
    let before = counts(&world);
    let Err(Not::Refused(r)) = accept(&seams, &world.history(), &ask("0.2.0", ToldBy::Asked))
    else {
        panic!("a version whose revision did not land was judged")
    };
    assert_eq!(
        r.why,
        Why::NotLanded {
            branch: Branch::new("main"),
            revision: world.head.clone(),
        }
    );
    assert_eq!(writes_since(&world, before), Vec::<String>::new());
}
