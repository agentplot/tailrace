//! Spec promotion: a channel moved by a promotion over the fakes, two
//! versions of `lib` published in the test, each carrying the gate's
//! acceptance and a second signer's rebuild.

mod common;
mod world;

use std::cell::Cell;
use std::rc::Rc;

use serde_json::json;
use tailrace_core::{
    answer_json, channel, mirror, promote, Channel, Followed, Mirrored, PromoteAsk, Promoted,
    Published, To,
};
use tailrace_fakes::{
    FakeKeeper, FakeSigner, FakeStore, Fault, KeeperOp, SignerOp, StoreOp, Times,
};
use tailrace_seams::{
    channel_name, Answer, Audience, Branch, Chain, ChannelId, ChannelName, Claim, ClaimKind,
    ComponentName, Digest, Envelope, Keeper, Moved, NamedSigner, Not, Outcome, OutputName,
    Promotion, Read, Report, Required, RequiredClaim, Satisfied, Seam, Seams, Signature, Signer,
    SignerId, Store, Version, Why,
};
use world::{build_of, version_of, World};

/// The identity the host signs promotions as, which the policy names
/// for acceptance and promotion (108).
const GATE: &str = "gate";

/// The second signer, whose rebuild the policy for `stable` requires.
const REBUILD: &str = "rebuild";

/// The moment every promotion here is asked at.
const AT: u64 = 1_767_225_600;

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
        signers,
    }
}

fn gate() -> NamedSigner {
    named(GATE, &[ClaimKind::Acceptance, ClaimKind::Promotion])
}

/// `stable` requires the gate's acceptance and a rebuild by the second
/// signer, is bound to `bound_to` where it names one, and is promoted by
/// the gate.
fn stable_policy(bound_to: Option<&str>) -> Required {
    Required::Channel {
        claims: vec![
            wanted(ClaimKind::Acceptance, vec![gate()]),
            wanted(
                ClaimKind::Rebuild,
                vec![named(REBUILD, &[ClaimKind::Rebuild])],
            ),
        ],
        bound_to: bound_to.map(ChannelName::new),
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

fn beta() -> ChannelId {
    of("lib", "beta")
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
    let at_start = tailrace_core::Ask {
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
        .require(Audience::Channel(stable()), stable_policy(None));
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

/// A claim of `kind` about `index` by `signer`, attached to it, and its
/// envelope's digest.
fn attach(world: &World, index: &Digest, signer: &str, kind: ClaimKind, body: serde_json::Value) {
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

/// The envelopes on `index` of `kind`, by their digests.
fn claims_of(world: &World, index: &Digest, kind: ClaimKind) -> Vec<Digest> {
    world
        .fakes
        .store
        .claims_of(index)
        .unwrap()
        .iter()
        .filter(|e| Claim::from_envelope(e).is_some_and(|c| c.kind == kind))
        .map(Envelope::digest)
        .collect()
}

fn ask(to: To) -> PromoteAsk {
    PromoteAsk {
        branch: Branch::new("main"),
        component: Some(ComponentName::new("lib")),
        channel: stable().name,
        to,
        from: None,
        signer: SignerId::new(GATE),
        at: AT,
    }
}

fn to(version: &str) -> To {
    To::Version(Version::new(version))
}

fn promoted(world: &World, ask: &PromoteAsk) -> Answer<Promoted> {
    promote(&world.fakes.seams(), &world.history(), ask)
}

fn promoted_over(world: &World, seams: &Seams, ask: &PromoteAsk) -> Answer<Promoted> {
    promote(seams, &world.history(), ask)
}

/// Every write the store, the keeper and the signer were asked since the
/// counts `before`.
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

fn moved(answer: Answer<Promoted>) -> (Digest, Answer<Mirrored>) {
    match answer {
        Ok(Promoted::Moved { claim, store, .. }) => (claim, store),
        other => panic!("the promotion answered {other:?}"),
    }
}

/// The claim's digest of a promotion that moved its channel.
fn claim_of(answer: Answer<Promoted>) -> Digest {
    moved(answer).0
}

fn refused(answer: Answer<impl std::fmt::Debug>) -> Why {
    match answer {
        Err(Not::Refused(r)) => r.why,
        other => panic!("answered {other:?}, not a refusal"),
    }
}

/// The promotions of `stable`'s chain, in order.
fn chain_of(world: &World, channel: &ChannelId) -> Vec<Promotion> {
    world
        .fakes
        .keeper
        .chain(channel)
        .unwrap()
        .claims
        .iter()
        .map(|e| Promotion::of(&Claim::from_envelope(e).unwrap()).unwrap())
        .collect()
}

// Requirement: A promotion moves a channel to an accepted version

#[test]
fn a_version_with_every_claim_the_channel_requires_moves_it_naming_the_claims_that_satisfied_it() {
    let Two { world, second, .. } = two();
    let answer = promoted(&world, &ask(to("0.2.0")));
    let Ok(Promoted::Moved {
        claim,
        policy,
        store,
        from,
        predecessor,
        ..
    }) = answer
    else {
        panic!("the promotion answered {answer:?}")
    };
    let mut satisfied = claims_of(&world, &second, ClaimKind::Acceptance);
    satisfied.extend(claims_of(&world, &second, ClaimKind::Rebuild));
    assert_eq!(policy.claims, satisfied, "the claims that satisfied it");
    assert_eq!(policy.revision, world.head);
    assert_eq!((from, predecessor), (None, None), "stable's first move");
    assert_eq!(
        store,
        Ok(Mirrored {
            attached: vec![claim.clone()],
            name: Followed::Moved { to: second.clone() },
        })
    );
    assert_eq!(chain_of(&world, &stable()).len(), 1);
    assert_eq!(
        world.fakes.store.resolve(&channel_name(&stable())),
        Ok(second.clone())
    );
    assert_eq!(claims_of(&world, &second, ClaimKind::Promotion), [claim]);
}

#[test]
fn the_ledgers_append_comes_before_every_store_write() {
    let Two { world, .. } = two();
    let before = world.fakes.store.calls().len();
    let at = Rc::new(Cell::new(None));
    struct Seen(FakeKeeper, FakeStore, Rc<Cell<Option<usize>>>);
    impl Keeper for Seen {
        fn append(&self, c: &ChannelId, p: &Envelope, d: Option<&Digest>) -> Answer<Digest> {
            self.2.set(Some(self.1.calls().len()));
            self.0.append(c, p, d)
        }
        fn chain(&self, c: &ChannelId) -> Answer<Chain> {
            self.0.chain(c)
        }
    }
    let mut seams = world.fakes.seams();
    seams.keeper = Box::new(Seen(
        world.fakes.keeper.clone(),
        world.fakes.store.clone(),
        at.clone(),
    ));
    claim_of(promoted_over(&world, &seams, &ask(to("0.2.0"))));
    let at = at.get().expect("the keeper was asked to append");
    let calls = world.fakes.store.calls();
    let writes = |range: &[tailrace_fakes::Call<StoreOp>]| {
        range
            .iter()
            .filter(|c| matches!(c.op, StoreOp::Attach | StoreOp::Name | StoreOp::PutBytes))
            .map(|c| c.op)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        writes(&calls[before..at]),
        [],
        "the store was written first"
    );
    assert_eq!(writes(&calls[at..]), [StoreOp::Attach, StoreOp::Name]);
}

// Requirement: The promotion claim records the move

#[test]
fn the_claim_read_back_names_every_field_of_the_move() {
    let Two {
        world,
        first,
        second,
    } = two();
    let (one, _) = moved(promoted(&world, &ask(to("0.1.0"))));
    let (two, _) = moved(promoted(
        &world,
        &PromoteAsk {
            from: Some(Version::new("0.1.0")),
            ..ask(to("0.2.0"))
        },
    ));
    let chain = world.fakes.keeper.chain(&stable()).unwrap();
    assert_eq!(chain.read, Read::Source);
    let head = chain.claims.last().unwrap();
    assert_eq!(head.digest(), two);
    let claim = Claim::from_envelope(head).unwrap();
    assert_eq!(claim.subject, second);
    assert_eq!(claim.kind, ClaimKind::Promotion);
    assert_eq!(
        claim.build, None,
        "a promotion names no build (rule core.9)"
    );
    assert_eq!(claim.revision, world.head, "the revision lib-v0.2.0 names");
    let mut satisfied = claims_of(&world, &second, ClaimKind::Acceptance);
    satisfied.extend(claims_of(&world, &second, ClaimKind::Rebuild));
    assert_eq!(
        Promotion::of(&claim),
        Some(Promotion {
            channel: ChannelName::new("stable"),
            component: ComponentName::new("lib"),
            to: Moved {
                version: Version::new("0.2.0"),
                index: second.clone(),
            },
            from: Some(Moved {
                version: Version::new("0.1.0"),
                index: first.clone(),
            }),
            predecessor: Some(one),
            asked_by: SignerId::new(GATE),
            policy: Satisfied {
                revision: world.head.clone(),
                claims: satisfied,
            },
            at: AT,
        })
    );
    assert_eq!(head.sole().unwrap().signer, SignerId::new(GATE));
}

#[test]
fn a_version_not_accepted_is_refused_naming_acceptance_and_nothing_is_written() {
    let world = World::new();
    let index = index_of(&world.publish_one(&world.ask_for("lib")).unwrap(), "0.2.0");
    attach(
        &world,
        &index,
        REBUILD,
        ClaimKind::Rebuild,
        json!({ "matched": true }),
    );
    world.fakes.signer.hold(&SignerId::new(GATE));
    world
        .fakes
        .policy
        .require(Audience::Channel(stable()), stable_policy(None));
    let before = counts(&world);
    assert_eq!(
        refused(promoted(&world, &ask(to("0.2.0")))),
        Why::Wanting {
            kind: ClaimKind::Acceptance,
            check: None,
            signers: vec![SignerId::new(GATE)],
        }
    );
    assert_eq!(writes_since(&world, before), Vec::<String>::new());
}

#[test]
fn an_acceptance_by_a_signer_the_channel_does_not_name_is_not_counted() {
    let world = World::new();
    let index = index_of(&world.publish_one(&world.ask_for("lib")).unwrap(), "0.2.0");
    attach(&world, &index, "thief", ClaimKind::Acceptance, json!({}));
    attach(
        &world,
        &index,
        REBUILD,
        ClaimKind::Rebuild,
        json!({ "matched": true }),
    );
    world.fakes.signer.hold(&SignerId::new(GATE));
    world
        .fakes
        .policy
        .require(Audience::Channel(stable()), stable_policy(None));
    assert!(matches!(
        refused(promoted(&world, &ask(to("0.2.0")))),
        Why::Wanting {
            kind: ClaimKind::Acceptance,
            ..
        }
    ));
}

#[test]
fn a_claim_the_channel_requires_wanting_is_refused_naming_it_and_its_signers() {
    let world = World::new();
    let index = index_of(&world.publish_one(&world.ask_for("lib")).unwrap(), "0.2.0");
    attach(&world, &index, GATE, ClaimKind::Acceptance, json!({}));
    world.fakes.signer.hold(&SignerId::new(GATE));
    world
        .fakes
        .policy
        .require(Audience::Channel(stable()), stable_policy(None));
    let before = counts(&world);
    let why = refused(promoted(&world, &ask(to("0.2.0"))));
    assert_eq!(
        why,
        Why::Wanting {
            kind: ClaimKind::Rebuild,
            check: None,
            signers: vec![SignerId::new(REBUILD)],
        }
    );
    assert!(why.to_string().contains("no rebuild"), "{why}");
    assert_eq!(writes_since(&world, before), Vec::<String>::new());
}

#[test]
fn a_version_never_published_is_absent_and_nothing_is_written() {
    let Two { world, .. } = two();
    let before = counts(&world);
    match promoted(&world, &ask(to("0.9.0"))) {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source.seam, Seam::Store);
            assert!(a.what.contains("lib-v0.9.0"), "{a:?}");
        }
        other => panic!("a version never published answered {other:?}"),
    }
    assert_eq!(writes_since(&world, before), Vec::<String>::new());
}

// Requirement: Only the gate's signer makes a promotion

#[test]
fn a_host_the_policy_does_not_let_promote_is_refused_before_anything_is_signed() {
    let Two { world, .. } = two();
    world.fakes.policy.require(
        Audience::Channel(stable()),
        Required::Channel {
            claims: vec![wanted(
                ClaimKind::Acceptance,
                vec![named(GATE, &[ClaimKind::Acceptance])],
            )],
            bound_to: None,
            promoted_by: vec![named("approver", &[ClaimKind::Promotion])],
            approvers: vec![],
        },
    );
    let before = counts(&world);
    assert_eq!(
        refused(promoted(&world, &ask(to("0.2.0")))),
        Why::Wanting {
            kind: ClaimKind::Promotion,
            check: None,
            signers: vec![SignerId::new("approver")],
        }
    );
    assert_eq!(writes_since(&world, before), Vec::<String>::new());
}

/// `stable` as the policy gives it with no line written: the gate's
/// acceptance required, and the gate, named for acceptance alone, its one
/// promoter (32, C.10); with `line` after the acceptance where one is.
fn given_stable(line: Vec<RequiredClaim>) -> Required {
    let gate = named(GATE, &[ClaimKind::Acceptance]);
    let mut claims = vec![wanted(ClaimKind::Acceptance, vec![gate.clone()])];
    claims.extend(line);
    Required::Channel {
        claims,
        bound_to: None,
        promoted_by: vec![gate],
        approvers: vec![],
    }
}

#[test]
fn a_given_stable_is_promoted_by_a_signer_named_only_for_acceptance() {
    let Two { world, second, .. } = two();
    world
        .fakes
        .policy
        .require(Audience::Channel(stable()), given_stable(vec![]));
    let answer = promoted(&world, &ask(to("0.2.0")));
    let Ok(Promoted::Moved { to, .. }) = &answer else {
        panic!("the gate did not promote a given stable: {answer:?}")
    };
    assert_eq!(to.index, second);
    let chain = chain_of(&world, &stable());
    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0].asked_by, SignerId::new(GATE));
}

#[test]
fn a_lines_promotion_signer_is_refused_and_nothing_is_appended() {
    let Two { world, .. } = two();
    let laptop = named("laptop", &[ClaimKind::Promotion]);
    world.fakes.policy.require(
        Audience::Channel(stable()),
        given_stable(vec![wanted(
            ClaimKind::Acceptance,
            vec![named(GATE, &[ClaimKind::Acceptance]), laptop],
        )]),
    );
    world.fakes.signer.hold(&SignerId::new("laptop"));
    let before = counts(&world);
    let answer = promoted(
        &world,
        &PromoteAsk {
            signer: SignerId::new("laptop"),
            ..ask(to("0.2.0"))
        },
    );
    assert_eq!(
        refused(answer),
        Why::Wanting {
            kind: ClaimKind::Promotion,
            check: None,
            signers: vec![SignerId::new(GATE)],
        }
    );
    assert_eq!(writes_since(&world, before), Vec::<String>::new());
    assert!(matches!(
        world.fakes.keeper.chain(&stable()),
        Err(Not::Absent(_))
    ));
}

#[test]
fn a_channel_of_another_component_is_absent_naming_both_and_nothing_is_read_of_the_version() {
    let Two { world, .. } = two();
    world
        .fakes
        .policy
        .require(Audience::Channel(of("web", "edge")), given_stable(vec![]));
    let before = counts(&world);
    let resolved = world
        .fakes
        .store
        .calls()
        .iter()
        .filter(|c| c.op == StoreOp::Resolve)
        .count();
    let answer = promoted(
        &world,
        &PromoteAsk {
            channel: ChannelName::new("edge"),
            ..ask(to("0.2.0"))
        },
    );
    let Err(Not::Absent(a)) = answer else {
        panic!("a channel of another component answered {answer:?}")
    };
    assert_eq!(a.source.seam, Seam::Policy);
    assert!(
        a.what.contains("edge") && a.what.contains("lib"),
        "{}",
        a.what
    );
    assert_eq!(writes_since(&world, before), Vec::<String>::new());
    assert_eq!(
        world
            .fakes
            .store
            .calls()
            .iter()
            .filter(|c| c.op == StoreOp::Resolve)
            .count(),
        resolved,
        "the version's name was resolved"
    );
}

// Requirement: A channel moves only from where it is

#[test]
fn s7_a_promotion_from_a_version_the_channel_has_left_is_refused_naming_where_it_is() {
    let Two { world, second, .. } = two();
    claim_of(promoted(&world, &ask(to("0.1.0"))));
    claim_of(promoted(&world, &ask(to("0.2.0"))));
    let before = counts(&world);
    let why = refused(promoted(
        &world,
        &PromoteAsk {
            from: Some(Version::new("0.1.0")),
            ..ask(to("0.1.0"))
        },
    ));
    assert_eq!(
        why,
        Why::ChannelAt {
            channel: stable(),
            at: Some(Moved {
                version: Version::new("0.2.0"),
                index: second,
            }),
        }
    );
    assert!(why.to_string().contains("lib/stable is at 0.2.0"), "{why}");
    assert_eq!(writes_since(&world, before), Vec::<String>::new());

    let fresh = two();
    assert_eq!(
        refused(promoted(
            &fresh.world,
            &PromoteAsk {
                from: Some(Version::new("0.1.0")),
                ..ask(to("0.2.0"))
            },
        )),
        Why::ChannelAt {
            channel: stable(),
            at: None
        }
    );
}

/// A keeper that, the first time the chain is read, has another
/// promotion appended at its head, as a second gate's would be, and
/// counts the appends the core asks.
struct Raced {
    inner: FakeKeeper,
    rival: Envelope,
    read: Cell<bool>,
    appends: Rc<Cell<usize>>,
}

impl Keeper for Raced {
    fn append(&self, c: &ChannelId, p: &Envelope, d: Option<&Digest>) -> Answer<Digest> {
        self.appends.set(self.appends.get() + 1);
        self.inner.append(c, p, d)
    }
    fn chain(&self, c: &ChannelId) -> Answer<Chain> {
        let answer = self.inner.chain(c);
        if !self.read.replace(true) {
            let head = answer.as_ref().ok().and_then(|ch| ch.claims.last());
            self.inner
                .append(c, &self.rival, head.map(Envelope::digest).as_ref())
                .unwrap();
        }
        answer
    }
}

/// An envelope in another form than a claim's, holding `words`.
fn sealed(words: &[u8]) -> Envelope {
    Envelope::signed(
        "application/vnd.their-own+json",
        words.to_vec(),
        Signature {
            signer: SignerId::new(GATE),
            keyid: String::new(),
            sig: vec![1],
        },
    )
}

#[test]
fn another_promotion_got_there_first_and_the_ledgers_refusal_is_the_answer() {
    let Two { world, .. } = two();
    claim_of(promoted(&world, &ask(to("0.1.0"))));
    let rival = sealed(b"a second gate's promotion");
    let appends = Rc::new(Cell::new(0));
    let mut seams = world.fakes.seams();
    seams.keeper = Box::new(Raced {
        inner: world.fakes.keeper.clone(),
        rival: rival.clone(),
        read: Cell::new(false),
        appends: appends.clone(),
    });
    let before = world.fakes.store.calls().len();
    assert_eq!(
        refused(promoted_over(&world, &seams, &ask(to("0.2.0")))),
        Why::NotHead {
            head: Some(rival.digest())
        }
    );
    assert_eq!(appends.get(), 1, "the append was retried");
    assert!(
        !world.fakes.store.calls()[before..]
            .iter()
            .any(|c| matches!(c.op, StoreOp::Attach | StoreOp::Name)),
        "the store was written"
    );
}

#[test]
fn a_forked_chain_refuses_the_promotion_naming_both_claims_and_nothing_is_written() {
    let Two { world, .. } = two();
    claim_of(promoted(&world, &ask(to("0.1.0"))));
    let root = world.fakes.keeper.chain(&stable()).unwrap().claims[0].clone();
    let (a, b) = (sealed(b"one side"), sealed(b"the other"));
    world.fakes.keeper.fork(&stable(), &a, &b);
    let before = counts(&world);
    assert_eq!(
        refused(promoted(&world, &ask(to("0.2.0")))),
        Why::Fork {
            predecessor: Some(root.digest()),
            claims: vec![a.digest(), b.digest()],
        }
    );
    assert_eq!(writes_since(&world, before), Vec::<String>::new());
}

#[test]
fn a_promotion_to_where_the_channel_is_answers_there_and_appends_nothing() {
    let Two { world, second, .. } = two();
    claim_of(promoted(&world, &ask(to("0.2.0"))));
    let appends = |world: &World| {
        world
            .fakes
            .keeper
            .calls()
            .iter()
            .filter(|c| c.op == KeeperOp::Append)
            .count()
    };
    let before = appends(&world);
    match promoted(&world, &ask(to("0.2.0"))) {
        Ok(Promoted::There { at, store, .. }) => {
            assert_eq!(
                at,
                Moved {
                    version: Version::new("0.2.0"),
                    index: second.clone()
                }
            );
            assert_eq!(
                store,
                Ok(Mirrored {
                    attached: vec![],
                    name: Followed::There { at: second },
                })
            );
        }
        other => panic!("a promotion to where the channel is answered {other:?}"),
    }
    assert_eq!(appends(&world), before, "a promotion was appended");
}

// Requirement: A wrong move is undone by a promotion back

#[test]
fn s8_back_moves_to_the_version_the_head_moved_from_and_the_version_left_stays_readable() {
    let Two {
        world,
        first,
        second,
    } = two();
    claim_of(promoted(&world, &ask(to("0.1.0"))));
    let (left, _) = moved(promoted(&world, &ask(to("0.2.0"))));
    match promoted(&world, &ask(To::Back)) {
        Ok(Promoted::Moved { to, from, .. }) => {
            assert_eq!(to.version, Version::new("0.1.0"));
            assert_eq!(from.map(|f| f.version), Some(Version::new("0.2.0")));
        }
        other => panic!("back answered {other:?}"),
    }
    let chain = chain_of(&world, &stable());
    assert_eq!(
        chain.iter().map(|p| p.to.index.clone()).collect::<Vec<_>>(),
        [first.clone(), second.clone(), first.clone()]
    );
    assert_eq!(
        world.fakes.store.resolve(&channel_name(&stable())),
        Ok(first)
    );
    assert!(world.fakes.store.read(&second).is_ok());
    assert!(world
        .fakes
        .store
        .claims_of(&second)
        .unwrap()
        .iter()
        .any(|e| e.digest() == left));
    assert_eq!(claims_of(&world, &second, ClaimKind::Acceptance).len(), 1);
}

#[test]
fn back_from_an_empty_chain_or_a_first_move_is_absent_and_nothing_is_written() {
    let Two { world, .. } = two();
    let before = counts(&world);
    let absent = |answer: Answer<Promoted>| match answer {
        Err(Not::Absent(a)) => assert_eq!(a.what, "a version lib/stable moved from"),
        other => panic!("back answered {other:?}"),
    };
    absent(promoted(&world, &ask(To::Back)));
    assert_eq!(writes_since(&world, before), Vec::<String>::new());
    claim_of(promoted(&world, &ask(to("0.1.0"))));
    let before = counts(&world);
    absent(promoted(&world, &ask(To::Back)));
    assert_eq!(writes_since(&world, before), Vec::<String>::new());
}

// Requirement: A bound channel takes only versions its bound channel held

/// The world with `beta` requiring the gate's acceptance alone and
/// `stable` bound to it.
fn bound() -> Two {
    let two = two();
    two.world.fakes.policy.require(
        Audience::Channel(beta()),
        Required::Channel {
            claims: vec![wanted(ClaimKind::Acceptance, vec![gate()])],
            bound_to: None,
            promoted_by: vec![gate()],
            approvers: vec![],
        },
    );
    two.world
        .fakes
        .policy
        .require(Audience::Channel(stable()), stable_policy(Some("beta")));
    two
}

fn to_beta(version: &str) -> PromoteAsk {
    PromoteAsk {
        channel: beta().name,
        ..ask(to(version))
    }
}

#[test]
fn a_version_the_bound_channel_never_held_is_refused_naming_it() {
    let Two { world, .. } = bound();
    claim_of(promoted(&world, &to_beta("0.2.0")));
    let before = counts(&world);
    let why = refused(promoted(&world, &ask(to("0.1.0"))));
    assert_eq!(
        why,
        Why::NeverHeld {
            channel: stable(),
            bound_to: beta(),
        }
    );
    assert_eq!(writes_since(&world, before), Vec::<String>::new());
    assert!(matches!(
        world.fakes.keeper.chain(&stable()),
        Err(Not::Absent(_))
    ));
}

/// A keeper that answers `beta`'s chain as last read, with words.
struct BetaKept(FakeKeeper);

impl Keeper for BetaKept {
    fn append(&self, c: &ChannelId, p: &Envelope, d: Option<&Digest>) -> Answer<Digest> {
        self.0.append(c, p, d)
    }
    fn chain(&self, c: &ChannelId) -> Answer<Chain> {
        let chain = self.0.chain(c)?;
        Ok(if *c == beta() {
            Chain {
                read: Read::Kept {
                    at: AT,
                    said: "beta's source does not answer".into(),
                },
                ..chain
            }
        } else {
            chain
        })
    }
}

#[test]
fn a_version_the_bound_channel_held_and_left_moves_the_channel_from_a_chain_read_as_last_read() {
    let Two { world, first, .. } = bound();
    claim_of(promoted(&world, &to_beta("0.1.0")));
    claim_of(promoted(&world, &to_beta("0.2.0")));
    let mut seams = world.fakes.seams();
    seams.keeper = Box::new(BetaKept(world.fakes.keeper.clone()));
    match promoted_over(&world, &seams, &ask(to("0.1.0"))) {
        Ok(Promoted::Moved { to, .. }) => assert_eq!(to.index, first),
        other => panic!("a version beta held answered {other:?}"),
    }
}

// Requirement: The store is rewritten from the ledger

#[test]
fn the_channels_own_chain_read_as_last_read_is_unreadable_with_its_words() {
    let Two { world, .. } = two();
    claim_of(promoted(&world, &ask(to("0.1.0"))));
    world.fakes.keeper.chain(&stable()).unwrap();
    world.fakes.keeper.unreachable("ref unreadable");
    let before = counts(&world);
    match promoted(&world, &ask(to("0.2.0"))) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.what, "the chain of lib/stable");
            assert_eq!(u.said.as_deref(), Some("ref unreadable"));
        }
        other => panic!("a chain read as last read answered {other:?}"),
    }
    assert_eq!(writes_since(&world, before), Vec::<String>::new());
}

#[test]
fn a_store_that_cannot_be_written_after_the_append_is_answered_beside_the_promotion() {
    let Two { world, second, .. } = two();
    world.fakes.store.fault(
        StoreOp::Attach,
        Fault::Unreadable("503 Service Unavailable".into()),
        Times::Once,
    );
    let (claim, store) = moved(promoted(&world, &ask(to("0.2.0"))));
    match &store {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.said.as_deref(), Some("503 Service Unavailable"))
        }
        other => panic!("the store's answer was {other:?}"),
    }
    let chain = world.fakes.keeper.chain(&stable()).unwrap().claims;
    assert_eq!(
        chain.iter().map(Envelope::digest).collect::<Vec<_>>(),
        [claim]
    );
    assert_eq!(claims_of(&world, &second, ClaimKind::Promotion), []);
    let json = answer_json(&promoted(&world, &ask(to("0.2.0"))));
    assert_eq!(json["value"]["promoted"], "there", "{json}");
    assert_eq!(json["value"]["store"]["answer"], "ok", "{json}");
}

#[test]
fn a_claim_missing_from_the_store_is_attached_by_the_next_promotion() {
    let Two {
        world,
        first,
        second,
    } = two();
    world.fakes.store.fault(
        StoreOp::Attach,
        Fault::Unreadable("503 Service Unavailable".into()),
        Times::Once,
    );
    let (one, store) = moved(promoted(&world, &ask(to("0.1.0"))));
    assert!(store.is_err());
    assert_eq!(claims_of(&world, &first, ClaimKind::Promotion), []);
    let appends = world.fakes.keeper.calls().len();
    let (two, store) = moved(promoted(&world, &ask(to("0.2.0"))));
    assert_eq!(
        store,
        Ok(Mirrored {
            attached: vec![one.clone(), two.clone()],
            name: Followed::LostRace {
                found: None,
                to: second.clone()
            },
        })
    );
    assert_eq!(claims_of(&world, &first, ClaimKind::Promotion), [one]);
    assert_eq!(claims_of(&world, &second, ClaimKind::Promotion), [two]);
    assert_eq!(
        world.fakes.keeper.calls()[appends..]
            .iter()
            .filter(|c| c.op == KeeperOp::Append)
            .count(),
        1,
        "the ledger was written but for the new promotion"
    );
}

#[test]
fn mirror_moves_the_channels_name_back_from_another_writers_index() {
    let Two {
        world,
        first,
        second,
    } = two();
    claim_of(promoted(&world, &ask(to("0.2.0"))));
    world
        .fakes
        .store
        .name(&channel_name(&stable()), &first, Some(&second))
        .unwrap();
    let appends = world.fakes.keeper.calls().len();
    assert_eq!(
        mirror(&world.fakes.store, &world.fakes.keeper, &stable()),
        Ok(Mirrored {
            attached: vec![],
            name: Followed::LostRace {
                found: Some(first),
                to: second.clone()
            },
        })
    );
    assert_eq!(
        world.fakes.store.resolve(&channel_name(&stable())),
        Ok(second)
    );
    assert!(!world.fakes.keeper.calls()[appends..]
        .iter()
        .any(|c| c.op == KeeperOp::Append));
}

// Requirement: A channel is read from the ledger alone

#[test]
fn a_channel_never_promoted_is_empty_and_the_store_is_asked_nothing() {
    let store = FakeStore::new();
    let keeper = FakeKeeper::new();
    assert_eq!(channel(&keeper, &stable()), Ok(Channel::Empty));
    assert_eq!(store.calls().len(), 0);
}

#[test]
fn a_channel_is_read_with_its_head_and_chain_from_the_ledger_alone() {
    let Two {
        world,
        first,
        second,
    } = two();
    let (one, _) = moved(promoted(&world, &ask(to("0.1.0"))));
    let (two, _) = moved(promoted(&world, &ask(to("0.2.0"))));
    let before = world.fakes.store.calls().len();
    let Ok(Channel::At { head, chain, read }) = channel(&world.fakes.keeper, &stable()) else {
        panic!("stable is not at a version")
    };
    assert_eq!(read, Read::Source);
    assert_eq!(head.to.index, second);
    assert_eq!(head.from.map(|f| f.index), Some(first.clone()));
    assert_eq!(
        chain
            .iter()
            .map(|(d, p)| (d.clone(), p.to.index.clone()))
            .collect::<Vec<_>>(),
        [(one, first), (two.clone(), second)]
    );
    assert_eq!(
        world.fakes.store.calls().len(),
        before,
        "the store was asked"
    );

    world.fakes.keeper.unreachable("ref unreadable");
    let Ok(Channel::At { read, chain, .. }) = channel(&world.fakes.keeper, &stable()) else {
        panic!("stable read as last read is not at a version")
    };
    assert!(matches!(read, Read::Kept { said, .. } if said == "ref unreadable"));
    assert_eq!(chain.last().map(|(d, _)| d.clone()), Some(two));
    let json = serde_json::to_value(channel(&world.fakes.keeper, &stable()).unwrap()).unwrap();
    assert_eq!(json["state"], "at", "{json}");
    assert_eq!(json["head"]["to"]["version"], "0.2.0", "{json}");
}

#[test]
fn a_chain_whose_claim_is_no_promotion_is_unreadable_naming_it() {
    let keeper = FakeKeeper::new();
    let theirs = sealed(b"not a promotion");
    keeper.append(&stable(), &theirs, None).unwrap();
    match channel(&keeper, &stable()) {
        Err(Not::Unreadable(u)) => assert!(u.what.contains(theirs.digest().as_str()), "{u:?}"),
        other => panic!("answered {other:?}"),
    }
}

#[test]
fn the_answers_render_in_the_one_form() {
    let Two { world, .. } = two();
    let answer = promoted(&world, &ask(to("0.2.0")));
    let json = answer_json(&answer);
    assert_eq!(json["answer"], "ok", "{json}");
    assert_eq!(json["value"]["promoted"], "moved", "{json}");
    assert_eq!(json["value"]["channel"], "stable", "{json}");
    assert_eq!(json["value"]["to"]["version"], "0.2.0", "{json}");
    assert_eq!(json["value"]["from"], serde_json::Value::Null, "{json}");
    assert_eq!(json["value"]["store"]["answer"], "ok", "{json}");
    assert_eq!(
        json["value"]["store"]["value"]["name"]["followed"], "moved",
        "{json}"
    );
}
