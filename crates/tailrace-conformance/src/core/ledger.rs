//! The ledger's group: a race lost on a channel's name repaired from the
//! ledger (109; design D11), and below the marker bolt 4's unit of the
//! keeper and promotion adds promotion, the ledger and bound channels.

use std::cell::Cell;
use std::rc::Rc;

use serde_json::json;
use tailrace_core::{
    channel, follow, mirror, Channel, Followed, Mirrored, PromoteAsk, Promoted, Published, To,
};
use tailrace_fakes::{FakeKeeper, FakeStore, Fault, KeeperOp, SignerOp, StoreOp, Times};
use tailrace_seams::{
    channel_name, Answer, Chain, ChannelId, ChannelName, Claim, ClaimKind, ComponentName, Digest,
    Envelope, IndexMeta, Keeper, Manifest, Moved, Name, Not, Promotion, Signature, Signer,
    SignerId, Store, Version, Why, IN_TOTO,
};

use crate::core::publish::{one, passing};
use crate::world::{FakeWorld, Write, GATE, REBUILDER, STABLE, WHEN};
use crate::{ok, Scenario};

pub const GROUP: &str = "ledger";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
        Scenario {
            name: "a race lost on a channel's name",
            plays: None,
            cites: &[109],
            run: a_race_lost_on_a_channels_name,
        },
        Scenario {
            name: "S6 a version promoted to stable",
            plays: Some("S6"),
            cites: &[33, 39, 101],
            run: s6_a_version_promoted_to_stable,
        },
        Scenario {
            name: "S7 a promotion from a version the channel has left",
            plays: Some("S7"),
            cites: &[35, 105],
            run: s7_a_promotion_from_a_version_the_channel_has_left,
        },
        Scenario {
            name: "S8 stable moved back",
            plays: Some("S8"),
            cites: &[34, 38],
            run: s8_stable_moved_back,
        },
        Scenario {
            name: "a bound channel takes what its bound channel held",
            plays: None,
            cites: &[37, 83],
            run: a_bound_channel_takes_what_its_bound_channel_held,
        },
        Scenario {
            name: "a promotion found wanting",
            plays: None,
            cites: &[9, 31, 32, 58],
            run: a_promotion_found_wanting,
        },
        Scenario {
            name: "a forked chain refuses a promotion",
            plays: None,
            cites: &[105],
            run: a_forked_chain_refuses_a_promotion,
        },
        Scenario {
            name: "the store rewritten from the ledger",
            plays: None,
            cites: &[106, 109],
            run: the_store_rewritten_from_the_ledger,
        },
        Scenario {
            name: "a channel read from the ledger alone",
            plays: None,
            cites: &[60, 101],
            run: a_channel_read_from_the_ledger_alone,
        },
        Scenario {
            name: "stable promoted by the gate's signer",
            plays: None,
            cites: &[32, 9, 125],
            run: stable_promoted_by_the_gates_signer,
        },
    ]
}

/// The head published in the world, its index; and an index of another
/// version, put beside it.
fn two_indexes(world: &FakeWorld) -> (Digest, Digest) {
    passing(world, &world.head);
    let published = ok(one(world.publish(&world.ask(&world.head))), "the publish");
    let Published::Version { index, .. } = published else {
        panic!("the head was not published: {published:?}")
    };
    let other = IndexMeta::new(
        ComponentName::new("repository"),
        Version::new("0.3.0"),
        vec![],
    )
    .expect("an index of no artifact");
    let other = ok(
        world.fakes.store.put_artifact(&Manifest::Index(other)),
        "the other index's put",
    )
    .digest;
    (index, other)
}

fn a_race_lost_on_a_channels_name(world: &FakeWorld) {
    let (index, other) = two_indexes(world);
    let channel = stable();
    let promotion = Claim {
        subject: index.clone(),
        kind: ClaimKind::Promotion,
        revision: world.head.clone(),
        build: None,
        body: json!({ "channel": STABLE }),
    };
    let promotion = ok(world.fakes.signer.sign(&promotion, &world.host), "a sign");
    ok(
        world.fakes.keeper.append(&channel, &promotion, None),
        "the promotion's append",
    );

    let name = Name::new(STABLE);
    let store: &dyn Store = &world.fakes.store;
    ok(store.name(&name, &other, None), "another writer's move");
    let written = world.writes().len();
    let appended = world.fakes.keeper.calls().len();

    let answer = ok(
        follow(store, &world.fakes.keeper, &channel, &name, None),
        "the move from the ledger",
    );
    assert_eq!(
        answer,
        Followed::LostRace {
            found: Some(other.clone()),
            to: index.clone()
        },
        "the lost race was not reported naming the other writer's index"
    );
    assert_eq!(
        ok(store.resolve(&name), "resolve"),
        index,
        "the name is not where the ledger's head says"
    );
    let moves: Vec<Write> = world.writes().split_off(written);
    assert_eq!(
        moves,
        [
            Write::Name {
                name: name.clone(),
                to: index.clone(),
                expected: None,
            },
            Write::Name {
                name,
                to: index,
                expected: Some(other),
            },
        ],
        "the store was not asked the caller's move and one repair"
    );
    assert_eq!(
        world.fakes.keeper.calls().len() - appended,
        1,
        "the ledger was asked more than its chain"
    );
}

const BETA: &str = "beta";

fn stable() -> ChannelId {
    ChannelId::new(None, ChannelName::new(STABLE))
}

fn to(version: &str) -> To {
    To::Version(Version::new(version))
}

/// The two versions of the world's one component, `start` published as
/// 0.1.0 and `head` as 0.2.0, each index carrying the gate's acceptance
/// and the rebuild signer's matching rebuild, and `stable`'s policy
/// requiring both; their indexes.
fn two_versions(world: &FakeWorld) -> (Digest, Digest) {
    let mut indexes = Vec::new();
    for revision in [&world.start, &world.head] {
        passing(world, revision);
        let published = ok(one(world.publish(&world.ask(revision))), "the publish");
        let Published::Version { index, .. } = published else {
            panic!("{revision} was not published: {published:?}")
        };
        world.attach_as(GATE, &index, ClaimKind::Acceptance, revision, json!({}));
        world.attach_as(
            REBUILDER,
            &index,
            ClaimKind::Rebuild,
            revision,
            json!({ "matched": true }),
        );
        indexes.push(index);
    }
    world.channel_policy(STABLE, true, None);
    (indexes[0].clone(), indexes[1].clone())
}

/// The claim's digest of a promotion that moved its channel.
fn moved(answer: Answer<Promoted>, asked: &str) -> (Digest, Answer<Mirrored>) {
    match answer {
        Ok(Promoted::Moved { claim, store, .. }) => (claim, store),
        other => panic!("{asked} answered {other:?}, not a move"),
    }
}

/// The claim's digest of a promotion that moved its channel.
fn claim_of(answer: Answer<Promoted>, asked: &str) -> Digest {
    moved(answer, asked).0
}

fn refused<T: std::fmt::Debug>(answer: Answer<T>, asked: &str) -> Why {
    match answer {
        Err(Not::Refused(r)) => r.why,
        other => panic!("{asked} answered {other:?}, not a refusal"),
    }
}

/// How many writes the store, appends the keeper and signatures every
/// signer of the world's own were asked.
fn written(world: &FakeWorld) -> (usize, usize, usize) {
    (
        world.writes().len(),
        world
            .keeper_calls()
            .iter()
            .filter(|op| **op == KeeperOp::Append)
            .count(),
        world
            .fakes
            .signer
            .calls()
            .iter()
            .filter(|c| c.op == SignerOp::Sign)
            .count(),
    )
}

/// The promotions of `channel`'s chain, in order.
fn chain_of(world: &FakeWorld, channel: &ChannelId) -> Vec<Promotion> {
    ok(world.fakes.keeper.chain(channel), "the chain")
        .claims
        .iter()
        .map(|e| {
            Claim::from_envelope(e)
                .as_ref()
                .and_then(Promotion::of)
                .expect("a promotion claim of tailrace's form")
        })
        .collect()
}

/// A keeper that records, when it is asked to append, how many writes the
/// store had been asked by then.
struct Watching {
    inner: FakeKeeper,
    store: FakeStore,
    at: Rc<Cell<Option<usize>>>,
}

impl Keeper for Watching {
    fn append(&self, c: &ChannelId, p: &Envelope, d: Option<&Digest>) -> Answer<Digest> {
        let writes = self
            .store
            .calls()
            .iter()
            .filter(|c| {
                matches!(
                    c.op,
                    StoreOp::PutBytes | StoreOp::PutArtifact | StoreOp::Attach | StoreOp::Name
                )
            })
            .count();
        self.at.set(Some(writes));
        self.inner.append(c, p, d)
    }
    fn chain(&self, c: &ChannelId) -> Answer<Chain> {
        self.inner.chain(c)
    }
}

fn s6_a_version_promoted_to_stable(world: &FakeWorld) {
    let (_, index) = two_versions(world);
    let before = world.writes().len();
    let at = Rc::new(Cell::new(None));
    let mut seams = world.seams_as_gate();
    seams.keeper = Box::new(Watching {
        inner: world.fakes.keeper.clone(),
        store: world.fakes.store.clone(),
        at: at.clone(),
    });
    let answer = tailrace_core::promote(
        &seams,
        &world.history(),
        &world.promote_ask(STABLE, to("0.2.0")),
    );
    let Ok(Promoted::Moved {
        claim,
        policy,
        store,
        ..
    }) = answer
    else {
        panic!("an approver's promotion answered {answer:?}")
    };
    let at = at.get().expect("the ledger was asked to append");
    assert_eq!(
        at, before,
        "the store was written before the ledger held the move"
    );
    let writes = world.writes().split_off(before);
    let Some(Write::Attach { subject, envelope }) = writes.first() else {
        panic!("the store's first write after the append was not the claim's attach: {writes:?}")
    };
    assert_eq!((subject, envelope.digest()), (&index, claim.clone()));
    assert!(
        matches!(writes.last(), Some(Write::Name { name, to, .. }) if *name == channel_name(&stable()) && *to == index),
        "stable's name was not moved last, to the index: {writes:?}"
    );
    assert_eq!(
        policy.claims.len(),
        2,
        "the acceptance and the rebuild did not satisfy it"
    );
    assert_eq!(
        store,
        Ok(Mirrored {
            attached: vec![claim.clone()],
            name: Followed::Moved { to: index.clone() },
        })
    );
    assert_eq!(
        ok(
            world.fakes.store.resolve(&channel_name(&stable())),
            "resolve stable"
        ),
        index,
        "stable's name does not resolve to the version promoted"
    );
    let head = chain_of(world, &stable())
        .pop()
        .expect("the chain holds the promotion");
    assert_eq!(
        (
            head.to.version.as_str(),
            &head.to.index,
            &head.asked_by,
            head.at
        ),
        ("0.2.0", &index, &SignerId::new(GATE), WHEN)
    );
}

fn s7_a_promotion_from_a_version_the_channel_has_left(world: &FakeWorld) {
    let (_, second) = two_versions(world);
    claim_of(
        world.promote(&world.promote_ask(STABLE, to("0.1.0"))),
        "the first promotion",
    );
    claim_of(
        world.promote(&world.promote_ask(STABLE, to("0.2.0"))),
        "the second promotion",
    );
    let before = written(world);
    let stale = PromoteAsk {
        from: Some(Version::new("0.1.0")),
        ..world.promote_ask(STABLE, to("0.1.0"))
    };
    assert_eq!(
        refused(world.promote(&stale), "a promotion from 0.1.0"),
        Why::ChannelAt {
            channel: stable(),
            at: Some(Moved {
                version: Version::new("0.2.0"),
                index: second,
            }),
        },
        "the refusal does not name where stable is"
    );
    assert_eq!(written(world), before, "a stale promotion wrote or signed");
}

fn s8_stable_moved_back(world: &FakeWorld) {
    let (first, second) = two_versions(world);
    claim_of(
        world.promote(&world.promote_ask(STABLE, to("0.1.0"))),
        "the first promotion",
    );
    let (left, _) = moved(
        world.promote(&world.promote_ask(STABLE, to("0.2.0"))),
        "the second",
    );
    claim_of(
        world.promote(&world.promote_ask(STABLE, To::Back)),
        "the promotion back",
    );
    let chain = chain_of(world, &stable());
    assert_eq!(
        chain.iter().map(|p| &p.to.index).collect::<Vec<_>>(),
        [&first, &second, &first],
        "the ledger does not hold both moves and the move back"
    );
    assert_eq!(
        chain[2].from.as_ref().map(|f| &f.version),
        Some(&Version::new("0.2.0"))
    );
    assert_eq!(
        ok(
            world.fakes.store.resolve(&channel_name(&stable())),
            "resolve stable"
        ),
        first,
        "stable is not back at 0.1.0"
    );
    ok(
        world.fakes.store.read(&second),
        "0.2.0's index read by digest",
    );
    let held = ok(world.fakes.store.claims_of(&second), "0.2.0's claims");
    assert!(
        held.iter().any(|e| e.digest() == left),
        "0.2.0's promotion claim is gone from its index"
    );
    assert!(
        held.iter()
            .filter_map(Claim::from_envelope)
            .any(|c| c.kind == ClaimKind::Acceptance),
        "0.2.0 is no longer accepted"
    );
}

fn a_bound_channel_takes_what_its_bound_channel_held(world: &FakeWorld) {
    let (first, _) = two_versions(world);
    world.channel_policy(BETA, false, None);
    world.channel_policy(STABLE, true, Some(BETA));
    let to_beta = |version| world.promote_ask(BETA, to(version));
    claim_of(world.promote(&to_beta("0.2.0")), "beta to 0.2.0");
    let before = written(world);
    assert_eq!(
        refused(
            world.promote(&world.promote_ask(STABLE, to("0.1.0"))),
            "stable to 0.1.0"
        ),
        Why::NeverHeld {
            channel: stable(),
            bound_to: ChannelId::new(None, ChannelName::new(BETA)),
        },
        "a version beta never held reached stable"
    );
    assert_eq!(
        written(world),
        before,
        "a refused promotion wrote or signed"
    );
    claim_of(world.promote(&to_beta("0.1.0")), "beta to 0.1.0");
    claim_of(world.promote(&to_beta("0.2.0")), "beta back to 0.2.0");
    let Ok(Promoted::Moved { to, .. }) = world.promote(&world.promote_ask(STABLE, to("0.1.0")))
    else {
        panic!("a version beta held and left did not reach stable")
    };
    assert_eq!(to.index, first);
}

fn a_promotion_found_wanting(world: &FakeWorld) {
    two_versions(world);
    let named = |id: &str, kinds: &[ClaimKind]| tailrace_seams::NamedSigner {
        id: SignerId::new(id),
        verified_by: format!("{id}'s key"),
        kinds: kinds.to_vec(),
        checks: vec![],
    };
    let required = |kind, signers| tailrace_seams::RequiredClaim {
        kind,
        check: None,
        signers,
    };
    let both = [ClaimKind::Acceptance, ClaimKind::Promotion];
    let cases = [
        (
            vec![required(
                ClaimKind::Acceptance,
                vec![named("another-gate", &both)],
            )],
            vec![named("another-gate", &both)],
            ClaimKind::Acceptance,
            "another-gate",
        ),
        (
            vec![
                required(ClaimKind::Acceptance, vec![named(GATE, &both)]),
                required(
                    ClaimKind::Rebuild,
                    vec![named("verifier", &[ClaimKind::Rebuild])],
                ),
            ],
            vec![named(GATE, &both)],
            ClaimKind::Rebuild,
            "verifier",
        ),
        (
            vec![required(
                ClaimKind::Acceptance,
                vec![named(GATE, &[ClaimKind::Acceptance])],
            )],
            vec![named("approver", &[ClaimKind::Promotion])],
            ClaimKind::Promotion,
            "approver",
        ),
    ];
    for (claims, promoted_by, kind, signer) in cases {
        world.fakes.policy.require(
            tailrace_seams::Audience::Channel(stable()),
            tailrace_seams::Required::Channel {
                claims,
                bound_to: None,
                promoted_by,
            },
        );
        let before = written(world);
        assert_eq!(
            refused(
                world.promote(&world.promote_ask(STABLE, to("0.2.0"))),
                "a promotion"
            ),
            Why::Wanting {
                kind,
                check: None,
                signers: vec![SignerId::new(signer)],
            },
            "the promotion was not refused naming the {kind} found wanting"
        );
        assert_eq!(
            written(world),
            before,
            "a promotion found wanting wrote or signed"
        );
    }
}

fn a_forked_chain_refuses_a_promotion(world: &FakeWorld) {
    two_versions(world);
    claim_of(
        world.promote(&world.promote_ask(STABLE, to("0.1.0"))),
        "the first promotion",
    );
    let root = ok(world.fakes.keeper.chain(&stable()), "the chain").claims[0].digest();
    let sealed = |words: &[u8]| {
        Envelope::signed(
            IN_TOTO,
            words.to_vec(),
            Signature {
                signer: SignerId::new(GATE),
                keyid: String::new(),
                sig: vec![1],
            },
        )
    };
    let (a, b) = (sealed(b"one side"), sealed(b"the other side"));
    world.fakes.keeper.fork(&stable(), &a, &b);
    let before = written(world);
    assert_eq!(
        refused(
            world.promote(&world.promote_ask(STABLE, to("0.2.0"))),
            "a promotion"
        ),
        Why::Fork {
            predecessor: Some(root),
            claims: vec![a.digest(), b.digest()],
        },
        "the fork was not reported naming both claims"
    );
    assert_eq!(
        written(world),
        before,
        "a promotion on a fork wrote or signed"
    );
}

fn the_store_rewritten_from_the_ledger(world: &FakeWorld) {
    let (first, second) = two_versions(world);
    world.fakes.store.fault(
        StoreOp::Attach,
        Fault::Unreadable("the store is away".into()),
        Times::Once,
    );
    let (missed, store) = moved(
        world.promote(&world.promote_ask(STABLE, to("0.1.0"))),
        "the first",
    );
    assert!(
        store.is_err(),
        "the store answered {store:?} with its attach failing"
    );
    let appends = written(world).1;
    let (claim, store) = moved(
        world.promote(&world.promote_ask(STABLE, to("0.2.0"))),
        "the second",
    );
    assert_eq!(
        store,
        Ok(Mirrored {
            attached: vec![missed.clone(), claim.clone()],
            name: Followed::LostRace {
                found: None,
                to: second.clone(),
            },
        }),
        "the claim the store missed was not attached from the ledger"
    );
    assert_eq!(
        written(world).1 - appends,
        1,
        "the ledger was written but for the promotion"
    );
    let on = |index: &Digest| {
        ok(world.fakes.store.claims_of(index), "claims-of")
            .iter()
            .map(Envelope::digest)
            .collect::<Vec<_>>()
    };
    assert!(on(&first).contains(&missed));

    let elsewhere = IndexMeta::new(
        ComponentName::new("repository"),
        Version::new("0.3.0"),
        vec![],
    )
    .expect("an index of no artifact");
    let elsewhere = ok(
        world.fakes.store.put_artifact(&Manifest::Index(elsewhere)),
        "another writer's index",
    )
    .digest;
    ok(
        world
            .fakes
            .store
            .name(&channel_name(&stable()), &elsewhere, Some(&second)),
        "another writer's move",
    );
    let appends = written(world).1;
    assert_eq!(
        ok(
            mirror(&world.fakes.store, &world.fakes.keeper, &stable()),
            "the mirror"
        ),
        Mirrored {
            attached: vec![],
            name: Followed::LostRace {
                found: Some(elsewhere),
                to: second.clone(),
            },
        },
        "the store was not rewritten from the ledger"
    );
    assert_eq!(
        ok(
            world.fakes.store.resolve(&channel_name(&stable())),
            "resolve stable"
        ),
        second
    );
    assert_eq!(written(world).1, appends, "the mirror wrote to the ledger");
}

fn a_channel_read_from_the_ledger_alone(world: &FakeWorld) {
    assert_eq!(
        ok(channel(&world.fakes.keeper, &stable()), "an empty channel"),
        Channel::Empty
    );
    let (first, second) = two_versions(world);
    let (one, _) = moved(
        world.promote(&world.promote_ask(STABLE, to("0.1.0"))),
        "the first",
    );
    let (two, _) = moved(
        world.promote(&world.promote_ask(STABLE, to("0.2.0"))),
        "the second",
    );
    let asked = world.store_calls().len();
    let read = ok(channel(&world.fakes.keeper, &stable()), "stable");
    let Channel::At { head, chain, read } = read else {
        panic!("stable read as {read:?}")
    };
    assert_eq!(read, tailrace_seams::Read::Source);
    assert_eq!(
        (&head.to.index, head.from.as_ref().map(|f| &f.index)),
        (&second, Some(&first))
    );
    assert_eq!(
        chain
            .iter()
            .map(|(d, p)| (d, &p.to.index))
            .collect::<Vec<_>>(),
        [(&one, &first), (&two, &second)]
    );
    assert_eq!(
        world.store_calls().len(),
        asked,
        "reading a channel asked the store"
    );
}

/// 32, 9, 125: `stable` whose policy names the gate, which may claim
/// acceptance alone, as its one promoter, as the policy gives `stable`
/// with no line written, bound to `beta`; and the version `beta` held.
fn given_stable(world: &FakeWorld) {
    two_versions(world);
    world.channel_policy(BETA, false, None);
    let gate = tailrace_seams::NamedSigner {
        id: SignerId::new(GATE),
        verified_by: format!("{GATE}'s key"),
        kinds: vec![ClaimKind::Acceptance],
        checks: vec![],
    };
    world.fakes.policy.require(
        tailrace_seams::Audience::Channel(stable()),
        tailrace_seams::Required::Channel {
            claims: vec![tailrace_seams::RequiredClaim {
                kind: ClaimKind::Acceptance,
                check: None,
                signers: vec![gate.clone()],
            }],
            bound_to: Some(ChannelName::new(BETA)),
            promoted_by: vec![gate],
        },
    );
    claim_of(
        world.promote(&world.promote_ask(BETA, to("0.2.0"))),
        "beta to 0.2.0",
    );
}

fn stable_promoted_by_the_gates_signer(world: &FakeWorld) {
    given_stable(world);
    claim_of(
        world.promote(&world.promote_ask(STABLE, to("0.2.0"))),
        "stable to 0.2.0 by the gate",
    );
    let chain = chain_of(world, &stable());
    assert_eq!(chain.len(), 1, "stable's chain holds other than the move");
    assert_eq!(chain[0].asked_by, SignerId::new(GATE));

    let other = FakeWorld::new();
    given_stable(&other);
    let before = written(&other);
    let by_laptop = PromoteAsk {
        signer: SignerId::new("laptop"),
        ..other.promote_ask(STABLE, to("0.2.0"))
    };
    assert_eq!(
        refused(other.promote(&by_laptop), "stable to 0.2.0 by laptop"),
        Why::Wanting {
            kind: ClaimKind::Promotion,
            check: None,
            signers: vec![SignerId::new(GATE)],
        },
        "a signer the policy does not name promoted stable"
    );
    assert_eq!(
        written(&other),
        before,
        "a refused promotion wrote or signed"
    );
    assert!(
        matches!(other.fakes.keeper.chain(&stable()), Err(Not::Absent(_))),
        "stable's chain was appended to"
    );
}
