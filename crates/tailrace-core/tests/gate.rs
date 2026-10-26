//! Spec acceptance: the gate's judgment of a version published in the
//! test, under the policy for acceptance at the branch's landed head, and
//! the channels that follow each publish.

mod common;
mod world;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use serde_json::json;
use tailrace_core::{
    accept, answer_json, channel, following, promote, AcceptAsk, Channel, Declared, Followed,
    Judged, Moved, PromoteAsk, Promoted, Published, To, ToldBy, WhyWanting,
};
use tailrace_fakes::{FakeKeeper, FakeSigner, KeeperOp, PolicyOp, SignerOp, StoreOp};
use tailrace_seams::{
    channel_name, Answer, Audience, Branch, Chain, ChannelName, Check, Claim, ClaimKind,
    ComponentName, Digest, Envelope, Held, Keeper, Manifest, Name, NamedSigner, Not, Outcome,
    Promotion, Put, Required, RequiredClaim, Revision, Satisfied, Seam, Seams, Signer, SignerId,
    Source, Store, Version, Why,
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

fn verdict_from(signer: NamedSigner, check: &str) -> RequiredClaim {
    RequiredClaim {
        kind: ClaimKind::Verdict,
        check: Some(Check::new(check)),
        signers: vec![signer],
    }
}

/// Acceptance requires the host's provenance and the landing's verdict
/// of `test`.
fn acceptance() -> Required {
    Required::Acceptance {
        claims: vec![
            provenance_from(named(HOST, &[ClaimKind::Provenance], &[])),
            verdict_from(named(LANDING, &[ClaimKind::Verdict], &["test"]), "test"),
        ],
    }
}

/// Acceptance requires the host's provenance alone.
fn provenance_only() -> Required {
    Required::Acceptance {
        claims: vec![provenance_from(named(HOST, &[ClaimKind::Provenance], &[]))],
    }
}

/// The policy file of the world, with `lib`'s channel `beta` following
/// each publish.
fn with_beta() -> String {
    format!(
        "{}\n[channels.beta]\non_each_publish = true\ncomponent = \"lib\"\n",
        world::POLICY
    )
}

fn beta() -> ChannelName {
    ChannelName::new("beta")
}

/// The gate's acceptance, which `beta` requires.
fn from_gate() -> RequiredClaim {
    RequiredClaim {
        kind: ClaimKind::Acceptance,
        check: None,
        signers: vec![named(GATE, &[ClaimKind::Acceptance], &[])],
    }
}

/// `beta` requires `claims`.
fn beta_requires(world: &World, claims: Vec<RequiredClaim>) {
    world.fakes.policy.require(
        Audience::Channel(beta()),
        Required::Channel {
            claims,
            bound_to: None,
        },
    );
}

/// A world whose `lib` was published at its head, each output's build
/// carrying the landing's passing `test` and `lint`, under `required`.
fn published_under(required: Required) -> (World, Published) {
    published_in(World::new(), required)
}

fn published_in(world: World, required: Required) -> (World, Published) {
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

/// Told by `published`, a publish's answer naming the index it offered.
fn told(published: &Published) -> ToldBy {
    let (_, index) = version_of(published);
    ToldBy::Publish {
        index: index.clone(),
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

/// Every write the store and the keeper were asked, and every signature
/// the host's signer was asked, since the counts `before`.
fn acts_since(world: &World, before: (usize, usize, usize)) -> Vec<String> {
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
        check: Some(Check::new("test")),
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
fn told_the_index_the_name_resolves_to_the_gate_judges_it() {
    let (world, published) = beta_world();
    let (_, index) = version_of(&published);
    let answer = judged(&world, &ask("0.2.0", told(&published)));
    let Ok(Judged::Accepted {
        index: judged,
        channels,
        ..
    }) = answer
    else {
        panic!("the version a publish told of was not accepted: {answer:?}")
    };
    assert_eq!(&judged, index);
    assert_eq!(channels.len(), 1, "{channels:?}");
    acceptance_by(&world, index, GATE);
}

#[test]
fn told_an_index_the_name_does_not_resolve_to_is_unreadable_naming_both_and_writes_nothing() {
    let (world, published) = beta_world();
    let (_, index) = version_of(&published);
    let other = Digest::of(b"an index the publish did not offer");
    let claims_before = world.fakes.store.claims_of(index).unwrap().len();
    let before = counts(&world);

    let answer = judged(
        &world,
        &ask(
            "0.2.0",
            ToldBy::Publish {
                index: other.clone(),
            },
        ),
    );
    let Err(Not::Unreadable(u)) = answer else {
        panic!("a told index the name does not resolve to was answered {answer:?}")
    };
    assert_eq!(u.source.seam, Seam::Store);
    assert!(u.what.contains("lib-v0.2.0"), "{}", u.what);
    let said = u.said.unwrap_or_default();
    assert!(said.contains(index.as_str()), "{said}");
    assert!(said.contains(other.as_str()), "{said}");

    let read: Vec<StoreOp> = world.fakes.store.calls()[before.0..]
        .iter()
        .map(|c| c.op)
        .collect();
    assert_eq!(read, [StoreOp::Resolve], "more than the name was read");
    assert_eq!(acts_since(&world, before), Vec::<String>::new());
    assert!(world.fakes.keeper.calls().is_empty());
    assert_eq!(
        world.fakes.store.claims_of(index).unwrap().len(),
        claims_before
    );
    assert_eq!(named_at(&world, "beta"), None);
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
    assert_eq!(acts_since(&world, before), Vec::<String>::new());
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
    assert_eq!(acts_since(&world, before), Vec::<String>::new());
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
    assert_eq!(acts_since(&world, before.0), Vec::<String>::new());
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
    assert_eq!(acts_since(&world, before), Vec::<String>::new());
}

// Requirement: Each artifact is judged against each required claim in order

/// The artifacts a published answer names, in the index's order.
fn artifacts(published: &Published) -> Vec<Digest> {
    match published {
        Published::Version { artifacts, .. } => {
            artifacts.iter().map(|a| a.digest.clone()).collect()
        }
        Published::Unchanged { .. } => panic!("unchanged: {published:?}"),
    }
}

/// The refusal a judgment answered: the artifact, the claim wanting and
/// why.
fn refusal(answer: Answer<Judged>) -> (Digest, RequiredClaim, WhyWanting) {
    match answer {
        Ok(Judged::Refused {
            artifact,
            wanting,
            why,
            ..
        }) => (artifact, wanting, why),
        other => panic!("the version was not refused: {other:?}"),
    }
}

#[test]
fn every_required_claim_found_accepts_the_version() {
    let (world, _) = published_under(acceptance());
    let answer = judged(&world, &ask("0.2.0", ToldBy::Asked));
    assert!(matches!(answer, Ok(Judged::Accepted { .. })), "{answer:?}");
}

#[test]
fn a_verdict_from_a_signer_the_policy_does_not_name_refuses_and_stays_attached() {
    let world = World::new();
    for o in ["lib-pack", "lib-docs"] {
        world.verdict("test", &build_of(o, &world.head), Outcome::Pass, "stranger");
    }
    world
        .fakes
        .policy
        .require(Audience::Acceptance, acceptance());
    let published = world.publish_one(&world.ask_for("lib")).unwrap();
    let first = &artifacts(&published)[0];
    let held = world.fakes.store.claims_of(first).unwrap();

    let (artifact, wanting, why) = refusal(judged(&world, &ask("0.2.0", ToldBy::Asked)));
    assert_eq!(&artifact, first);
    assert_eq!(wanting.kind, ClaimKind::Verdict);
    assert_eq!(why, WhyWanting::Missing);
    assert_eq!(world.fakes.store.claims_of(first).unwrap(), held);
}

#[test]
fn a_verdict_from_a_signer_beyond_its_reach_does_not_count() {
    let world = World::new();
    for o in ["lib-pack", "lib-docs"] {
        world.verdict("test", &build_of(o, &world.head), Outcome::Pass, HOST);
    }
    let host = named(HOST, &[ClaimKind::Provenance], &[]);
    world.fakes.policy.require(
        Audience::Acceptance,
        Required::Acceptance {
            claims: vec![provenance_from(host.clone()), verdict_from(host, "test")],
        },
    );
    world.publish_one(&world.ask_for("lib")).unwrap();
    let (_, wanting, why) = refusal(judged(&world, &ask("0.2.0", ToldBy::Asked)));
    assert_eq!(wanting.kind, ClaimKind::Verdict);
    assert_eq!(why, WhyWanting::Missing);
}

#[test]
fn a_provenance_naming_another_revision_does_not_count() {
    let (world, published) = published_under(provenance_only());
    let builder = FakeSigner::new();
    builder.hold(&SignerId::new("builder"));
    for artifact in artifacts(&published) {
        let held = world.fakes.store.claims_of(&artifact).unwrap();
        let claim = Claim {
            revision: Revision::new("0".repeat(40)),
            ..Claim::from_envelope(&held[0]).unwrap()
        };
        let other = builder.sign(&claim, &SignerId::new("builder")).unwrap();
        world.fakes.store.attach(&artifact, &other).unwrap();
    }
    let required = provenance_from(named("builder", &[ClaimKind::Provenance], &[]));
    world.fakes.policy.require(
        Audience::Acceptance,
        Required::Acceptance {
            claims: vec![required.clone()],
        },
    );
    let (artifact, wanting, why) = refusal(judged(&world, &ask("0.2.0", ToldBy::Asked)));
    assert_eq!(artifact, artifacts(&published)[0]);
    assert_eq!(wanting, required);
    assert_eq!(why, WhyWanting::Missing);

    for artifact in artifacts(&published) {
        let held = world.fakes.store.claims_of(&artifact).unwrap();
        let claim = Claim::from_envelope(&held[0]).unwrap();
        let ours = builder.sign(&claim, &SignerId::new("builder")).unwrap();
        world.fakes.store.attach(&artifact, &ours).unwrap();
    }
    let answer = judged(&world, &ask("0.2.0", ToldBy::Asked));
    assert!(
        matches!(answer, Ok(Judged::Accepted { .. })),
        "the builder's provenance of the version's revision did not count: {answer:?}"
    );
}

// Requirement: An expired signer's claim does not count at acceptance

#[test]
fn a_provenance_from_an_expired_signer_refuses_naming_it() {
    let (world, published) = published_under(acceptance());
    world.fakes.signer.expire(&SignerId::new(HOST));
    let (artifact, wanting, why) = refusal(judged(&world, &ask("0.2.0", ToldBy::Asked)));
    assert_eq!(artifact, artifacts(&published)[0]);
    assert_eq!(wanting.kind, ClaimKind::Provenance);
    assert_eq!(
        why,
        WhyWanting::Expired {
            signer: SignerId::new(HOST)
        }
    );
}

// Requirement: A version is accepted or refused whole

#[test]
fn the_second_of_two_artifacts_wanting_refuses_the_version_whole_and_attaches_nothing() {
    let world = World::new();
    let pack = build_of("lib-pack", &world.head);
    world.verdict("test", &pack, Outcome::Pass, LANDING);
    world.verdict(
        "lint",
        &build_of("lib-docs", &world.head),
        Outcome::Pass,
        LANDING,
    );
    world
        .fakes
        .policy
        .require(Audience::Acceptance, acceptance());
    let published = world.publish_one(&world.ask_for("lib")).unwrap();
    let (_, index) = version_of(&published);
    let digests = artifacts(&published);
    let held: Vec<_> = std::iter::once(index)
        .chain(&digests)
        .map(|d| world.fakes.store.claims_of(d).unwrap())
        .collect();
    let before = counts(&world);

    let (artifact, wanting, why) = refusal(judged(&world, &ask("0.2.0", ToldBy::Asked)));
    assert_eq!(artifact, digests[1]);
    assert_eq!(wanting.kind, ClaimKind::Verdict);
    assert_eq!(wanting.check, Some(Check::new("test")));
    assert_eq!(why, WhyWanting::Missing);
    assert_eq!(acts_since(&world, before), Vec::<String>::new());
    let now: Vec<_> = std::iter::once(index)
        .chain(&digests)
        .map(|d| world.fakes.store.claims_of(d).unwrap())
        .collect();
    assert_eq!(now, held);
}

// Requirement: An independent rebuild is counted, never made

/// Acceptance requires the host's provenance and a rebuild, which the
/// host and a verifier may each claim.
fn rebuilt() -> (RequiredClaim, Required) {
    let rebuild = RequiredClaim {
        kind: ClaimKind::Rebuild,
        check: None,
        signers: vec![
            named(HOST, &[ClaimKind::Provenance, ClaimKind::Rebuild], &[]),
            named("verifier", &[ClaimKind::Rebuild], &[]),
        ],
    };
    let required = Required::Acceptance {
        claims: vec![
            provenance_from(named(HOST, &[ClaimKind::Provenance], &[])),
            rebuild.clone(),
        ],
    };
    (rebuild, required)
}

/// A rebuild of each artifact by `signer`, saying the digests `matched`
/// or did not, attached beside it.
fn rebuild_by(world: &World, published: &Published, signer: &str, matched: bool) {
    let theirs = FakeSigner::new();
    theirs.hold(&SignerId::new(signer));
    for artifact in artifacts(published) {
        let claim = Claim {
            subject: artifact.clone(),
            kind: ClaimKind::Rebuild,
            revision: world.head.clone(),
            build: None,
            body: json!({ "matched": matched }),
        };
        let envelope = theirs.sign(&claim, &SignerId::new(signer)).unwrap();
        world.fakes.store.attach(&artifact, &envelope).unwrap();
    }
}

#[test]
fn a_rebuild_by_the_offering_host_does_not_count() {
    let (rebuild, required) = rebuilt();
    let (world, published) = published_under(required);
    rebuild_by(&world, &published, HOST, true);
    let (_, wanting, why) = refusal(judged(&world, &ask("0.2.0", ToldBy::Asked)));
    assert_eq!(wanting, rebuild);
    assert_eq!(
        why,
        WhyWanting::Unmatched {
            signer: SignerId::new(HOST)
        }
    );
}

#[test]
fn a_rebuild_that_did_not_match_does_not_count_and_one_that_did_does() {
    let (rebuild, required) = rebuilt();
    let (world, published) = published_under(required);
    rebuild_by(&world, &published, "verifier", false);
    let (_, wanting, why) = refusal(judged(&world, &ask("0.2.0", ToldBy::Asked)));
    assert_eq!(wanting, rebuild);
    assert_eq!(
        why,
        WhyWanting::Unmatched {
            signer: SignerId::new("verifier")
        }
    );

    rebuild_by(&world, &published, "verifier", true);
    let answer = judged(&world, &ask("0.2.0", ToldBy::Asked));
    assert!(matches!(answer, Ok(Judged::Accepted { .. })), "{answer:?}");
}

/// A store that hands every operation to the fake, first answering
/// `claims_of` where the hook says so and telling `on_name` of each name
/// move asked.
struct Hooked {
    inner: tailrace_fakes::FakeStore,
    claims_of: Box<dyn Fn(&Digest) -> Option<Not>>,
    on_name: Box<dyn Fn(&Name)>,
}

impl Hooked {
    fn over(inner: &tailrace_fakes::FakeStore) -> Hooked {
        Hooked {
            inner: inner.clone(),
            claims_of: Box::new(|_| None),
            on_name: Box::new(|_| ()),
        }
    }
}

impl Store for Hooked {
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        self.inner.put_bytes(bytes)
    }
    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        self.inner.put_artifact(manifest)
    }
    fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
        self.inner.attach(subject, attestation)
    }
    fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>> {
        match (self.claims_of)(digest) {
            Some(not) => Err(not),
            None => self.inner.claims_of(digest),
        }
    }
    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
        (self.on_name)(name);
        self.inner.name(name, to, expected)
    }
    fn resolve(&self, name: &Name) -> Answer<Digest> {
        self.inner.resolve(name)
    }
    fn read(&self, digest: &Digest) -> Answer<Held> {
        self.inner.read(digest)
    }
}

#[test]
fn claims_that_cannot_be_read_are_unreadable_naming_the_digest_with_the_stores_words() {
    let (world, published) = published_under(acceptance());
    let second = artifacts(&published)[1].clone();
    let mut seams = holding(&world, &[GATE]);
    let failing = second.clone();
    seams.store = Box::new(Hooked {
        claims_of: Box::new(move |digest| {
            (*digest == failing).then(|| {
                Not::unreadable(
                    &Source::new(Seam::Store, "failing"),
                    format!("the claims of {digest}"),
                    Some("connection reset by peer".into()),
                )
            })
        }),
        ..Hooked::over(&world.fakes.store)
    });
    let before = counts(&world);
    let Err(Not::Unreadable(u)) = accept(&seams, &world.history(), &ask("0.2.0", ToldBy::Asked))
    else {
        panic!("claims that cannot be read were answered")
    };
    assert!(u.what.contains(second.as_str()), "{}", u.what);
    assert_eq!(u.said.as_deref(), Some("connection reset by peer"));
    assert_eq!(u.source.seam, Seam::Store);
    assert_eq!(acts_since(&world, before), Vec::<String>::new());
}

#[test]
fn a_name_that_resolves_to_no_index_is_unreadable_and_no_refusal() {
    let (world, published) = published_under(acceptance());
    let (_, index) = version_of(&published);
    let artifact = artifacts(&published)[0].clone();
    let tag = Name::new("lib-v0.2.0");
    world
        .fakes
        .store
        .name(&tag, &artifact, Some(index))
        .unwrap();
    let Err(Not::Unreadable(u)) = judged(&world, &ask("0.2.0", ToldBy::Asked)) else {
        panic!("a name naming no index was judged")
    };
    assert!(u.what.contains(artifact.as_str()), "{}", u.what);
    assert!(u.what.contains("index"), "{}", u.what);
}

// Requirement: Acceptance attaches one claim and changes no bytes

#[test]
fn an_accepted_version_holds_one_more_claim_on_its_index_and_no_byte_changes() {
    let (world, published) = published_under(acceptance());
    let (_, index) = version_of(&published);
    let digests = artifacts(&published);
    let on_index = world.fakes.store.claims_of(index).unwrap();
    let on_artifacts: Vec<_> = digests
        .iter()
        .map(|d| world.fakes.store.claims_of(d).unwrap())
        .collect();
    let manifest = world.fakes.store.read(index).unwrap();
    let before = world.fakes.store.calls().len();

    let Ok(Judged::Accepted {
        acceptance,
        already,
        one_host,
        channels,
        ..
    }) = judged(&world, &ask("0.2.0", ToldBy::Asked))
    else {
        panic!("the version was not accepted")
    };
    assert!(!already && !one_host && channels.is_empty());

    let now = world.fakes.store.claims_of(index).unwrap();
    assert_eq!(now.len(), on_index.len() + 1);
    assert_eq!(now[..on_index.len()], on_index[..]);
    let added = now.last().unwrap();
    assert_eq!(added.digest(), acceptance);
    let made = world
        .fakes
        .signer
        .verify(added, &[named(GATE, &[ClaimKind::Acceptance], &[])])
        .unwrap();
    assert_eq!(made.signer, SignerId::new(GATE));
    let claim = Claim::from_envelope(added).unwrap();
    assert_eq!(claim.subject, *index);
    assert_eq!(claim.kind, ClaimKind::Acceptance);
    assert_eq!(claim.revision, world.head);
    assert_eq!(claim.build, None);
    assert_eq!(claim.body["component"], "lib");
    assert_eq!(claim.body["version"], "0.2.0");
    assert_eq!(
        claim.body["policy"],
        json!({ "audience": "acceptance", "revision": world.head.as_str() })
    );
    assert_eq!(claim.body["claims"].as_array().unwrap().len(), 4);

    for (digest, held) in digests.iter().zip(&on_artifacts) {
        assert_eq!(&world.fakes.store.claims_of(digest).unwrap(), held);
    }
    assert_eq!(world.fakes.store.read(index).unwrap(), manifest);
    assert_eq!(
        world.fakes.store.resolve(&Name::new("lib-v0.2.0")),
        Ok(index.clone())
    );
    let asked: Vec<StoreOp> = world.fakes.store.calls()[before..]
        .iter()
        .map(|c| c.op)
        .filter(|op| !matches!(op, StoreOp::Resolve | StoreOp::Read | StoreOp::ClaimsOf))
        .collect();
    assert_eq!(asked, [StoreOp::Attach], "the store was asked {asked:?}");
}

// Requirement: The gate signs only under the accepting credential

#[test]
fn a_host_with_no_gate_key_answers_absent_naming_the_identity_and_writes_nothing() {
    let (world, published) = published_under(acceptance());
    let before = counts(&world);
    let Err(Not::Absent(a)) = accept(
        &world.fakes.seams(),
        &world.history(),
        &ask("0.2.0", told(&published)),
    ) else {
        panic!("a host with no gate key accepted")
    };
    assert_eq!(a.source.seam, Seam::Signer);
    assert!(a.what.contains(GATE), "{}", a.what);
    assert_eq!(
        acts_since(&world, before),
        ["signer Sign"],
        "more than the one signature asked"
    );
    assert!(world.fakes.keeper.calls().is_empty());
}

// Requirement: The gate never accepts what it offered, but on the one host

/// The acceptance claim on `index` signed by `signer`.
fn acceptance_by(world: &World, index: &Digest, signer: &str) -> Claim {
    let by = [named(signer, &[ClaimKind::Acceptance], &[])];
    world
        .fakes
        .store
        .claims_of(index)
        .unwrap()
        .iter()
        .filter(|e| world.fakes.signer.verify(e, &by).is_ok())
        .filter_map(Claim::from_envelope)
        .find(|c| c.kind == ClaimKind::Acceptance)
        .unwrap_or_else(|| panic!("no acceptance by {signer} on {index}"))
}

#[test]
fn a_build_host_running_the_gate_is_refused_as_offered_by_the_gate() {
    let (world, published) = published_under(acceptance());
    let before = counts(&world);
    let as_host = AcceptAsk {
        gate: SignerId::new(HOST),
        ..ask("0.2.0", told(&published))
    };
    let (artifact, wanting, why) =
        refusal(accept(&world.fakes.seams(), &world.history(), &as_host));
    assert_eq!(artifact, artifacts(&published)[0]);
    assert_eq!(wanting.kind, ClaimKind::Provenance);
    assert_eq!(
        why,
        WhyWanting::OfferedByGate {
            signer: SignerId::new(HOST)
        }
    );
    assert_eq!(acts_since(&world, before), Vec::<String>::new());
}

// Requirement: A one-host acceptance says so

#[test]
fn the_solo_laptops_acceptance_says_one_host_offered_and_accepted() {
    let laptop = named(HOST, &[ClaimKind::Provenance, ClaimKind::Acceptance], &[]);
    let (world, published) = published_under(Required::Acceptance {
        claims: vec![
            provenance_from(laptop),
            verdict_from(named(LANDING, &[ClaimKind::Verdict], &["test"]), "test"),
        ],
    });
    let (_, index) = version_of(&published);
    let as_laptop = AcceptAsk {
        gate: SignerId::new(HOST),
        ..ask("0.2.0", ToldBy::Asked)
    };
    let answer = accept(&world.fakes.seams(), &world.history(), &as_laptop);
    let Ok(Judged::Accepted { one_host, .. }) = answer else {
        panic!("the laptop did not accept what it offered: {answer:?}")
    };
    assert!(one_host);
    let claim = acceptance_by(&world, index, HOST);
    assert_eq!(
        claim.body["offered"],
        json!({ "by": [HOST], "one_host": true })
    );
}

#[test]
fn a_separated_gates_acceptance_names_the_offerer() {
    let (world, published) = published_under(acceptance());
    let (_, index) = version_of(&published);
    let Ok(Judged::Accepted { one_host, .. }) = judged(&world, &ask("0.2.0", ToldBy::Asked)) else {
        panic!("the gate did not accept")
    };
    assert!(!one_host);
    let claim = acceptance_by(&world, index, GATE);
    assert_eq!(
        claim.body["offered"],
        json!({ "by": [HOST], "one_host": false })
    );
}

// Requirement: A channel that follows each publish is declared in the policy file

/// What `following` answers for `component` at the world's head: each
/// channel, and the words of an unreadable one.
fn followers(world: &World, component: &str) -> Vec<(String, Result<(), String>)> {
    let landed = common::landed(&world.head);
    let history = world.history();
    let declared = Declared::read(&history, &landed).unwrap();
    following(&history, &landed, &declared, &ComponentName::new(component))
        .unwrap()
        .into_iter()
        .map(|(name, answer)| {
            let answer = answer.map_err(|not| match not {
                Not::Unreadable(u) => {
                    assert!(
                        u.what.starts_with(&format!("[channels.{name}]")),
                        "{}",
                        u.what
                    );
                    u.said.unwrap_or_default()
                }
                other => panic!("{name} answered {other:?}"),
            });
            (name.to_string(), answer)
        })
        .collect()
}

#[test]
fn beta_follows_each_publish_of_the_one_component() {
    let world = World::with_policy(
        r#"
outputs = ["site"]
projections = { site = "eco-a" }

[channels.beta]
on_each_publish = true
require = [{ kind = "acceptance", signers = ["gate"] }]

[channels.stable]
bound_to = "beta"
"#,
    );
    assert_eq!(
        followers(&world, "repository"),
        [("beta".to_string(), Ok(()))]
    );
}

#[test]
fn a_publish_of_one_component_follows_only_its_own_channels_in_the_files_order() {
    let world = World::with_policy(&format!(
        r#"{}
[channels.lib-beta]
on_each_publish = true
component = "lib"

[channels.web-beta]
on_each_publish = true
component = "web"

[channels.stable]
on_each_publish = true
component = "lib"
bound_to = "lib-beta"

[channels.edge]
on_each_publish = true

[channels.other]
on_each_publish = true
component = "app"

[channels.nightly]
component = "lib"
"#,
        world::POLICY
    ));
    let lib = followers(&world, "lib");
    let names: Vec<&str> = lib.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["lib-beta", "stable", "edge", "other"]);
    assert_eq!(lib[0].1, Ok(()));
    let said = |i: usize| lib[i].1.clone().unwrap_err();
    assert!(said(1).contains("bound to lib-beta"), "{}", said(1));
    assert!(said(2).contains("names no component"), "{}", said(2));
    assert!(said(2).contains("lib, web"), "{}", said(2));
    assert!(said(3).contains("app"), "{}", said(3));

    let web = followers(&world, "web");
    let names: Vec<&str> = web.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["web-beta", "edge", "other"]);
    assert_eq!(web[0].1, Ok(()));
}

// Requirement: Nothing accepted is un-accepted

/// A later revision landed whose policy for acceptance requires a review
/// no version carries.
fn stricter(world: &World) -> Revision {
    let later = world.repo.touch("chore: stricter", &["docs"]);
    world.fakes.landing.land(&world.main, &later);
    let review = RequiredClaim {
        kind: ClaimKind::Review,
        check: None,
        signers: vec![named("reviewer", &[ClaimKind::Review], &[])],
    };
    let Required::Acceptance { mut claims } = acceptance() else {
        unreachable!()
    };
    claims.push(review);
    world.fakes.policy.require_at(
        &later,
        Audience::Acceptance,
        Required::Acceptance { claims },
    );
    later
}

#[test]
fn a_version_the_gate_accepted_is_accepted_again_after_a_stricter_policy_lands() {
    let (world, _) = published_in(World::with_policy(&with_beta()), acceptance());
    beta_requires(&world, vec![from_gate()]);
    let Ok(Judged::Accepted {
        acceptance: first, ..
    }) = judged(&world, &ask("0.2.0", ToldBy::Asked))
    else {
        panic!("the gate did not accept")
    };
    let later = stricter(&world);
    let before = counts(&world);

    let answer = judged(&world, &ask("0.2.0", ToldBy::Asked));
    let Ok(Judged::Accepted {
        acceptance,
        already,
        policy_at,
        ..
    }) = answer
    else {
        panic!("an accepted version was un-accepted: {answer:?}")
    };
    assert!(already);
    assert_eq!(acceptance, first);
    assert_eq!(policy_at, later);
    assert_eq!(acts_since(&world, before), Vec::<String>::new());
}

#[test]
fn an_acceptance_another_signer_made_is_not_the_gates() {
    let (world, published) = published_in(World::with_policy(&with_beta()), acceptance());
    beta_requires(&world, vec![from_gate()]);
    let (_, index) = version_of(&published);
    let forged = Claim {
        subject: index.clone(),
        kind: ClaimKind::Acceptance,
        revision: world.head.clone(),
        build: None,
        body: json!({ "offered": { "by": [HOST], "one_host": false } }),
    };
    let forged = world
        .fakes
        .signer
        .sign(&forged, &SignerId::new(HOST))
        .unwrap();
    world.fakes.store.attach(index, &forged).unwrap();
    stricter(&world);
    let (_, wanting, _) = refusal(judged(&world, &ask("0.2.0", ToldBy::Asked)));
    assert_eq!(wanting.kind, ClaimKind::Review);
}

#[test]
fn a_build_host_running_the_gate_over_its_own_acceptance_is_refused_not_accepted_already() {
    let (world, published) = published_in(World::with_policy(&with_beta()), acceptance());
    beta_requires(&world, vec![from_gate()]);
    let (_, index) = version_of(&published);
    let forged = Claim {
        subject: index.clone(),
        kind: ClaimKind::Acceptance,
        revision: world.head.clone(),
        build: None,
        body: json!({ "offered": { "by": [HOST], "one_host": true } }),
    };
    let forged = world
        .fakes
        .signer
        .sign(&forged, &SignerId::new(HOST))
        .unwrap();
    world.fakes.store.attach(index, &forged).unwrap();
    let as_host = AcceptAsk {
        gate: SignerId::new(HOST),
        ..ask("0.2.0", told(&published))
    };
    let (_, _, why) = refusal(accept(&world.fakes.seams(), &world.history(), &as_host));
    assert_eq!(
        why,
        WhyWanting::OfferedByGate {
            signer: SignerId::new(HOST)
        }
    );
}

// Requirement: The gate moves each following channel after accepting what a publish told it of

/// What `channel`'s name in the store names, if anything.
fn named_at(world: &World, channel: &str) -> Option<Digest> {
    world
        .fakes
        .store
        .resolve(&channel_name(&ChannelName::new(channel)))
        .ok()
}

/// The appends the keeper was asked.
fn appends(world: &World) -> usize {
    world
        .fakes
        .keeper
        .calls()
        .iter()
        .filter(|c| c.op == KeeperOp::Append)
        .count()
}

#[test]
fn a_version_asked_of_rather_than_told_by_a_publish_moves_no_channel() {
    let (world, _) = published_in(World::with_policy(&with_beta()), acceptance());
    beta_requires(&world, vec![from_gate()]);
    let Ok(Judged::Accepted { channels, .. }) = judged(&world, &ask("0.2.0", ToldBy::Asked)) else {
        panic!("the gate did not accept")
    };
    assert!(channels.is_empty(), "{channels:?}");
    assert!(world.fakes.keeper.calls().is_empty());
    assert_eq!(named_at(&world, "beta"), None);
}

#[test]
fn a_channel_requiring_a_review_the_version_lacks_stays_while_the_version_is_accepted() {
    let (world, published) = published_in(World::with_policy(&with_beta()), acceptance());
    let review = RequiredClaim {
        kind: ClaimKind::Review,
        check: None,
        signers: vec![named("reviewer", &[ClaimKind::Review], &[])],
    };
    beta_requires(&world, vec![from_gate(), review.clone()]);
    let answer = judged(&world, &ask("0.2.0", told(&published)));
    let Ok(Judged::Accepted {
        already: false,
        channels,
        ..
    }) = answer
    else {
        panic!("the version was not accepted: {answer:?}")
    };
    assert_eq!(
        channels,
        [(
            beta(),
            Ok(Moved::Held {
                artifact: artifacts(&published)[0].clone(),
                wanting: review,
                why: WhyWanting::Missing,
            })
        )]
    );
    assert_eq!(appends(&world), 0);
    assert_eq!(named_at(&world, "beta"), None);
}

// Requirement: A following channel moves through the ledger first

/// A world whose `lib` was published under acceptance, `beta` following
/// each publish and requiring the gate's acceptance.
fn beta_world() -> (World, Published) {
    let (world, published) = published_in(World::with_policy(&with_beta()), acceptance());
    beta_requires(&world, vec![from_gate()]);
    (world, published)
}

/// `beta`'s answer of a judgment that accepted.
fn beta_moved(answer: Answer<Judged>) -> Answer<Moved> {
    let Ok(Judged::Accepted { mut channels, .. }) = answer else {
        panic!("the version was not accepted: {answer:?}")
    };
    assert_eq!(channels.len(), 1, "{channels:?}");
    let (channel, moved) = channels.remove(0);
    assert_eq!(channel, beta());
    moved
}

/// `lib`'s `version` at `index`.
fn at(version: &str, index: &Digest) -> tailrace_seams::Moved {
    tailrace_seams::Moved {
        version: Version::new(version),
        index: index.clone(),
    }
}

/// `beta`'s first promotion, to 0.1.0 at `subject`, signed as the gate.
fn promotion_to(world: &World, subject: &Digest) -> Envelope {
    let gate = FakeSigner::new();
    gate.hold(&SignerId::new(GATE));
    let promotion = Promotion {
        channel: beta(),
        component: ComponentName::new("lib"),
        to: at("0.1.0", subject),
        from: None,
        predecessor: None,
        asked_by: SignerId::new("each publish"),
        policy: Satisfied {
            revision: world.head.clone(),
            claims: vec![],
        },
        at: WHEN,
    };
    gate.sign(&promotion.claim(world.head.clone()), &SignerId::new(GATE))
        .unwrap()
}

#[test]
fn the_ledger_holds_the_promotion_before_the_name_moves() {
    let (world, published) = beta_world();
    let (_, index) = version_of(&published);
    let seen: Rc<RefCell<Vec<usize>>> = Rc::default();
    let mut seams = holding(&world, &[GATE]);
    let (keeper, held) = (world.fakes.keeper.clone(), seen.clone());
    seams.store = Box::new(Hooked {
        on_name: Box::new(move |name| {
            if *name == channel_name(&beta()) {
                held.borrow_mut()
                    .push(keeper.chain(&beta()).map(|c| c.claims.len()).unwrap_or(0));
            }
        }),
        ..Hooked::over(&world.fakes.store)
    });

    let moved = beta_moved(accept(
        &seams,
        &world.history(),
        &ask("0.2.0", told(&published)),
    ));
    let Ok(Moved::Promoted {
        from,
        to,
        promotion,
        name,
    }) = moved
    else {
        panic!("beta was not promoted: {moved:?}")
    };
    assert_eq!((from, &to), (None, index));
    assert_eq!(name, Followed::Moved { to: index.clone() });
    assert_eq!(
        *seen.borrow(),
        [1],
        "the name moved before the ledger held the promotion"
    );
    assert_eq!(named_at(&world, "beta").as_ref(), Some(index));

    let chain = world.fakes.keeper.chain(&beta()).unwrap().claims;
    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0].digest(), promotion);
    assert!(world
        .fakes
        .store
        .claims_of(index)
        .unwrap()
        .contains(&chain[0]));
    world
        .fakes
        .signer
        .verify(&chain[0], &[named(GATE, &[ClaimKind::Promotion], &[])])
        .unwrap();
    let claim = Claim::from_envelope(&chain[0]).unwrap();
    assert_eq!(
        (&claim.subject, claim.kind, &claim.revision),
        (index, ClaimKind::Promotion, &world.head)
    );
    let made = Promotion::of(&claim).expect("the gate's promotion is not of the one body");
    assert_eq!(made.policy.claims.len(), 2);
    assert_eq!(
        made,
        Promotion {
            channel: beta(),
            component: ComponentName::new("lib"),
            to: at("0.2.0", index),
            from: None,
            predecessor: None,
            asked_by: SignerId::new("each publish"),
            policy: Satisfied {
                revision: world.head.clone(),
                claims: made.policy.claims.clone(),
            },
            at: WHEN,
        }
    );
}

#[test]
fn a_promotion_names_the_chains_head_and_moves_the_name_from_where_it_left_it() {
    let (world, published) = beta_world();
    let (_, index) = version_of(&published);
    let before = Digest::of(b"the version before");
    world.fakes.store.put_bytes(b"the version before").unwrap();
    let first = promotion_to(&world, &before);
    world.fakes.keeper.append(&beta(), &first, None).unwrap();
    world
        .fakes
        .store
        .name(&channel_name(&beta()), &before, None)
        .unwrap();

    let moved = beta_moved(judged(&world, &ask("0.2.0", told(&published))));
    let Ok(Moved::Promoted { from, name, .. }) = moved else {
        panic!("beta was not promoted: {moved:?}")
    };
    assert_eq!(from, Some(before.clone()));
    assert_eq!(name, Followed::Moved { to: index.clone() });
    let chain = world.fakes.keeper.chain(&beta()).unwrap().claims;
    assert_eq!(chain.len(), 2);
    let made = Promotion::of(&Claim::from_envelope(&chain[1]).unwrap()).unwrap();
    assert_eq!(made.predecessor, Some(first.digest()));
    assert_eq!(made.from, Some(at("0.1.0", &before)));
    assert_eq!(made.to, at("0.2.0", index));
    assert_eq!(named_at(&world, "beta").as_ref(), Some(index));
}

/// A keeper another writer appends `then` to right after the first
/// chain it answers, counting the appends it is asked.
struct Racing {
    inner: FakeKeeper,
    then: Envelope,
    raced: Cell<bool>,
    appended: Rc<Cell<usize>>,
}

impl Keeper for Racing {
    fn append(
        &self,
        channel: &ChannelName,
        promotion: &Envelope,
        predecessor: Option<&Digest>,
    ) -> Answer<Digest> {
        self.appended.set(self.appended.get() + 1);
        self.inner.append(channel, promotion, predecessor)
    }
    fn chain(&self, channel: &ChannelName) -> Answer<Chain> {
        let answer = self.inner.chain(channel);
        if !self.raced.replace(true) {
            let head = answer
                .as_ref()
                .ok()
                .and_then(|c| c.claims.last())
                .map(|e| e.digest());
            self.inner
                .append(channel, &self.then, head.as_ref())
                .unwrap();
        }
        answer
    }
}

#[test]
fn a_chain_moved_between_read_and_append_is_refused_naming_its_head_and_nothing_is_retried() {
    let (world, published) = beta_world();
    let (_, index) = version_of(&published);
    let theirs = promotion_to(&world, &Digest::of(b"another version"));
    let appended = Rc::new(Cell::new(0));
    let mut seams = holding(&world, &[GATE]);
    seams.keeper = Box::new(Racing {
        inner: world.fakes.keeper.clone(),
        then: theirs.clone(),
        raced: Cell::new(false),
        appended: appended.clone(),
    });

    let moved = beta_moved(accept(
        &seams,
        &world.history(),
        &ask("0.2.0", told(&published)),
    ));
    let Err(Not::Refused(r)) = moved else {
        panic!("an append after another was answered {moved:?}")
    };
    assert_eq!(
        r.why,
        Why::NotHead {
            head: Some(theirs.digest())
        }
    );
    assert_eq!(appended.get(), 1, "the append was retried");
    assert_eq!(named_at(&world, "beta"), None);
    let promotions = world
        .fakes
        .store
        .claims_of(index)
        .unwrap()
        .iter()
        .filter_map(Claim::from_envelope)
        .filter(|c| c.kind == ClaimKind::Promotion)
        .count();
    assert_eq!(promotions, 0, "a refused promotion was attached");
}

#[test]
fn a_forked_chain_is_reported_and_nothing_is_appended_or_moved() {
    let (world, published) = beta_world();
    let root = promotion_to(&world, &Digest::of(b"the first"));
    world.fakes.keeper.append(&beta(), &root, None).unwrap();
    let (a, b) = (
        promotion_to(&world, &Digest::of(b"one")),
        promotion_to(&world, &Digest::of(b"two")),
    );
    world.fakes.keeper.fork(&beta(), &a, &b);
    let before = appends(&world);

    let moved = beta_moved(judged(&world, &ask("0.2.0", told(&published))));
    let Err(Not::Refused(r)) = moved else {
        panic!("a forked chain was answered {moved:?}")
    };
    assert_eq!(
        r.why,
        Why::Fork {
            predecessor: Some(root.digest()),
            claims: vec![a.digest(), b.digest()],
        }
    );
    assert_eq!(appends(&world), before);
    assert_eq!(named_at(&world, "beta"), None);
}

#[test]
fn a_channel_already_at_the_version_is_there_and_nothing_is_written() {
    let (world, published) = beta_world();
    let (_, index) = version_of(&published);
    beta_moved(judged(&world, &ask("0.2.0", told(&published)))).unwrap();
    let before = counts(&world);

    let answer = judged(&world, &ask("0.2.0", told(&published)));
    let Ok(Judged::Accepted { already: true, .. }) = &answer else {
        panic!("a version accepted was not answered accepted already: {answer:?}")
    };
    assert_eq!(beta_moved(answer), Ok(Moved::There { at: index.clone() }));
    assert_eq!(acts_since(&world, before), Vec::<String>::new());
}

#[test]
fn a_version_the_gate_moved_beta_to_is_taken_by_stable_bound_to_beta() {
    let (world, published) = beta_world();
    let (_, index) = version_of(&published);
    world.fakes.signer.hold(&SignerId::new(GATE));
    let seams = world.fakes.seams();
    beta_moved(accept(
        &seams,
        &world.history(),
        &ask("0.2.0", told(&published)),
    ))
    .unwrap();
    let Ok(Channel::At { head, .. }) = channel(seams.keeper.as_ref(), &beta()) else {
        panic!("beta's chain does not read from the ledger")
    };
    assert_eq!(head.to, at("0.2.0", index));

    let stable = ChannelName::new("stable");
    world.fakes.policy.require(
        Audience::Channel(stable.clone()),
        Required::Channel {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Acceptance,
                check: None,
                signers: vec![named(
                    GATE,
                    &[ClaimKind::Acceptance, ClaimKind::Promotion],
                    &[],
                )],
            }],
            bound_to: Some(beta()),
        },
    );
    let answer = promote(
        &seams,
        &world.history(),
        &PromoteAsk {
            branch: Branch::new("main"),
            component: Some(ComponentName::new("lib")),
            channel: stable,
            to: To::Version(Version::new("0.2.0")),
            from: None,
            signer: SignerId::new(GATE),
            at: WHEN,
        },
    );
    let Ok(Promoted::Moved { to, store, .. }) = answer else {
        panic!("stable bound to beta did not take what the gate moved beta to: {answer:?}")
    };
    assert_eq!(to, at("0.2.0", index));
    assert!(store.is_ok(), "{store:?}");
    assert_eq!(named_at(&world, "stable").as_ref(), Some(index));
}
