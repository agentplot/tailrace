//! The gate's group: an offered version judged under the policy for
//! acceptance at the branch's landed head, accepted whole with a claim of
//! the gate's own or refused whole, and the channels that follow each
//! publish moved by the gate alone; S3, S5 and S9 as far as the gate
//! (record 0012).
//!
//! Each artifact is judged against the claims the policy asks of
//! artifacts made its way (134): the policies here ask the landing's
//! verdict, a review and a rebuild of built artifacts alone, so a
//! version's notes, rendered with no build, are asked the publisher's
//! provenance and the claims about the version. The scenarios of the
//! notes cite 25 and 28, and a following channel's 95 too, in place of
//! 134, which the statement's clause list, read at 157c828, does not
//! hold; they cite 134 once the list adopts it.

use std::cell::RefCell;
use std::rc::Rc;

use serde_json::json;
use tailrace_core::{accept, Judged, Moved, Published, ToldBy, WhyWanting};
use tailrace_fakes::{FakeKeeper, FakeLanding, FakeSigner, FakeStore, KeeperOp, StoreOp};
use tailrace_seams::{
    channel_name, Answer, AskedOf, Audience, ChannelId, ChannelName, Check, Claim, ClaimKind,
    ComponentName, Digest, Envelope, Held, Keeper, Manifest, Name, NamedSigner, Not, Outcome, Put,
    Required, RequiredClaim, Revision, Seam, Signer, SignerId, Store, Why,
};

use crate::core::publish::{one, version_of, TAG, VERSION};
use crate::world::{build_of, FakeWorld, Write, GATE, HOST, LANDING, POLICY};
use crate::{ok, Scenario};

pub const GROUP: &str = "gate";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
        Scenario {
            name: "a version accepted and beta moved",
            plays: Some("S3"),
            cites: &[25, 26, 95],
            run: a_version_accepted_and_beta_moved,
        },
        Scenario {
            name: "a verdict from a signer the policy does not name",
            plays: Some("S5"),
            cites: &[25, 28],
            run: a_verdict_from_a_signer_the_policy_does_not_name,
        },
        Scenario {
            name: "a thief's acceptance",
            plays: Some("S9"),
            cites: &[9, 27],
            run: a_thiefs_acceptance,
        },
        Scenario {
            name: "a build host running the gate",
            plays: Some("S9"),
            cites: &[27, 96],
            run: a_build_host_running_the_gate,
        },
        Scenario {
            name: "a host with no gate key",
            plays: Some("S9"),
            cites: &[95, 96],
            run: a_host_with_no_gate_key,
        },
        Scenario {
            name: "a publish alone",
            plays: Some("S9"),
            cites: &[20, 27],
            run: a_publish_alone,
        },
        Scenario {
            name: "acceptance changes no bytes",
            plays: None,
            cites: &[26],
            run: acceptance_changes_no_bytes,
        },
        Scenario {
            name: "one artifact wanting refuses the version",
            plays: None,
            cites: &[28],
            run: one_artifact_wanting_refuses_the_version,
        },
        Scenario {
            name: "judged by the claims, the landing and the policy alone",
            plays: None,
            cites: &[29],
            run: judged_by_the_claims_the_landing_and_the_policy_alone,
        },
        Scenario {
            name: "an independent rebuild",
            plays: None,
            cites: &[30],
            run: an_independent_rebuild,
        },
        Scenario {
            name: "nothing accepted is un-accepted",
            plays: None,
            cites: &[94],
            run: nothing_accepted_is_un_accepted,
        },
        Scenario {
            name: "asked rather than told",
            plays: None,
            cites: &[95],
            run: asked_rather_than_told,
        },
        Scenario {
            name: "the policy at the landed head",
            plays: None,
            cites: &[107],
            run: the_policy_at_the_landed_head,
        },
        Scenario {
            name: "one host offers and accepts",
            plays: None,
            cites: &[108],
            run: one_host_offers_and_accepts,
        },
        Scenario {
            name: "a separated gate names its offerer",
            plays: None,
            cites: &[27, 108],
            run: a_separated_gate_names_its_offerer,
        },
        Scenario {
            name: "the core starts nothing after an offer",
            plays: None,
            cites: &[95, 130],
            run: the_core_starts_nothing_after_an_offer,
        },
        Scenario {
            name: "a told index the name does not resolve to",
            plays: None,
            cites: &[95, 130],
            run: a_told_index_the_name_does_not_resolve_to,
        },
        Scenario {
            name: "beta of one component",
            plays: None,
            cites: &[95, 125, 131],
            run: beta_of_one_component,
        },
        Scenario {
            name: "a prerelease and the channels that follow",
            plays: None,
            cites: &[95, 131],
            run: a_prerelease_and_the_channels_that_follow,
        },
        Scenario {
            name: "a version whose notes carry no build verdict",
            plays: None,
            cites: &[25, 28],
            run: a_version_whose_notes_carry_no_build_verdict,
        },
        Scenario {
            name: "a rendered artifact wanting the publisher's provenance",
            plays: None,
            cites: &[25, 28],
            run: a_rendered_artifact_wanting_the_publishers_provenance,
        },
        Scenario {
            name: "the outputs' claims wanting beside the notes",
            plays: None,
            cites: &[25, 28],
            run: the_outputs_claims_wanting_beside_the_notes,
        },
        Scenario {
            name: "a following channel and the notes",
            plays: None,
            cites: &[25, 28, 95],
            run: a_following_channel_and_the_notes,
        },
    ]
}

/// The channel that follows each publish.
const BETA: &str = "beta";

fn beta() -> ChannelId {
    ChannelId::new(None, ChannelName::new(BETA))
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
        asked_of: AskedOf::Every,
    }
}

/// The landing's verdict of `test`, asked of built artifacts alone (134).
fn test_from_the_landing() -> RequiredClaim {
    RequiredClaim {
        kind: ClaimKind::Verdict,
        check: Some(Check::new("test")),
        signers: vec![named(LANDING, &[ClaimKind::Verdict], &["test"])],
        asked_of: AskedOf::Built,
    }
}

/// The reviewer's review, asked of built artifacts alone (134).
fn review() -> RequiredClaim {
    RequiredClaim {
        kind: ClaimKind::Review,
        check: None,
        signers: vec![named("reviewer", &[ClaimKind::Review], &[])],
        asked_of: AskedOf::Built,
    }
}

/// Acceptance requires the host's provenance and the landing's verdict of
/// `test`, as S3's policy does of the fleet's signer and the host's.
fn acceptance() -> Required {
    Required::Acceptance {
        claims: vec![
            provenance_from(named(HOST, &[ClaimKind::Provenance], &[])),
            test_from_the_landing(),
        ],
    }
}

/// Acceptance requires `required`, and `beta`, which the policy lists
/// moving on each publish, the gate's acceptance, the gate its promoter
/// (95, 131).
fn require(world: &FakeWorld, required: Required) {
    let policy = &world.fakes.policy;
    policy.require(Audience::Acceptance, required);
    let gate = named(GATE, &[ClaimKind::Acceptance], &[]);
    policy.require(
        Audience::Channel(beta()),
        Required::Channel {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Acceptance,
                check: None,
                signers: vec![gate.clone()],
                asked_of: AskedOf::Every,
            }],
            bound_to: None,
            promoted_by: vec![gate],
            approvers: vec![],
        },
    );
    policy.follows(&beta(), false);
}

/// The policy file landed, each output built and
/// given `verdicts` at that revision, `required` for acceptance, and the
/// revision published.
fn offered_with(
    world: &FakeWorld,
    required: Required,
    verdicts: impl Fn(&Revision) -> Vec<tailrace_seams::Verdict>,
) -> Published {
    let revision = world.land_policy(POLICY);
    let verdicts = verdicts(&revision);
    world.land(&revision, &verdicts);
    world.outputs(
        &revision,
        &[("site", Outcome::Pass), ("server", Outcome::Pass)],
    );
    require(world, required);
    ok(one(world.publish(&world.ask(&revision))), "the publish")
}

/// The version offered under `required`, the landing's passing verdicts
/// of `test` on each output's build.
fn offered_under(world: &FakeWorld, required: Required) -> Published {
    offered_with(world, required, |revision| {
        ["site", "server"]
            .iter()
            .map(|o| {
                world.verdict(
                    revision,
                    "test",
                    &build_of(o, revision),
                    Outcome::Pass,
                    LANDING,
                )
            })
            .collect()
    })
}

/// The version offered under acceptance.
fn offered(world: &FakeWorld) -> Published {
    offered_under(world, acceptance())
}

/// Told by `published`, a publish's answer naming the index it offered
/// (95).
fn told(published: &Published) -> ToldBy {
    let (_, index) = version_of(published);
    ToldBy::Publish {
        index: index.clone(),
    }
}

fn revision_of(published: &Published) -> Revision {
    match published {
        Published::Version { revision, .. } => revision.clone(),
        Published::Unchanged { .. } => panic!("unchanged: {published:?}"),
    }
}

/// Each artifact a published answer names, by digest and output.
fn artifacts(published: &Published) -> Vec<(Digest, String)> {
    match published {
        Published::Version { artifacts, .. } => artifacts
            .iter()
            .map(|a| (a.digest.clone(), a.output.to_string()))
            .collect(),
        Published::Unchanged { .. } => panic!("unchanged: {published:?}"),
    }
}

/// The notes a published answer names: their artifact's digest and the
/// envelope of the provenance the publisher attached to them (134).
fn notes_of(published: &Published) -> (Digest, Digest) {
    match published {
        Published::Version {
            notes: Some(notes), ..
        } => (notes.digest.clone(), notes.claims[0].envelope.clone()),
        other => panic!("no notes: {other:?}"),
    }
}

/// The claims each of `digests` holds, as the store answers them.
fn held(world: &FakeWorld, digests: &[&Digest]) -> Vec<Vec<Envelope>> {
    digests
        .iter()
        .map(|d| ok(world.fakes.store.claims_of(d), "claims-of"))
        .collect()
}

/// The index and every artifact of a published answer.
fn digests(published: &Published) -> Vec<Digest> {
    let (_, index) = version_of(published);
    std::iter::once(index.clone())
        .chain(artifacts(published).into_iter().map(|(d, _)| d))
        .collect()
}

/// What a judgment that accepted answered: the acceptance's envelope,
/// whether it was there already, whether one host offered and accepted,
/// and each channel's answer.
type Accepted = (Digest, bool, bool, Vec<(ChannelId, Answer<Moved>)>);

fn accepted(answer: Answer<Judged>) -> Accepted {
    match answer {
        Ok(Judged::Accepted {
            acceptance,
            already,
            one_host,
            channels,
            ..
        }) => (acceptance, already, one_host, channels),
        other => panic!("the version was not accepted: {other:?}"),
    }
}

fn refused(answer: Answer<Judged>) -> (Digest, RequiredClaim, WhyWanting) {
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

/// The acceptance on `index` that `signer` made, read as a claim.
fn acceptance_by(world: &FakeWorld, index: &Digest, signer: &str) -> Claim {
    let by = [named(signer, &[ClaimKind::Acceptance], &[])];
    ok(world.fakes.store.claims_of(index), "claims-of")
        .iter()
        .filter(|e| world.fakes.signer.verify(e, &by).is_ok())
        .filter_map(Claim::from_envelope)
        .find(|c| c.kind == ClaimKind::Acceptance)
        .unwrap_or_else(|| panic!("{index} holds no acceptance {signer} made"))
}

/// Every claim of `kind` on `digest`, whoever made it.
fn of_kind(world: &FakeWorld, digest: &Digest, kind: ClaimKind) -> usize {
    ok(world.fakes.store.claims_of(digest), "claims-of")
        .iter()
        .filter_map(Claim::from_envelope)
        .filter(|c| c.kind == kind)
        .count()
}

/// `beta` names nothing and its chain holds nothing.
fn beta_unmoved(world: &FakeWorld) {
    assert!(
        world.fakes.store.resolve(&channel_name(&beta())).is_err(),
        "beta names a version"
    );
    assert!(
        !world.keeper_calls().contains(&KeeperOp::Append),
        "beta's chain was appended to"
    );
}

/// A policy landed after the publish whose acceptance also requires a
/// review the version lacks; its revision.
fn stricter(world: &FakeWorld) -> Revision {
    let later = world.land_policy(POLICY);
    let Required::Acceptance { mut claims } = acceptance() else {
        unreachable!()
    };
    claims.push(review());
    world.fakes.policy.require_at(
        &later,
        Audience::Acceptance,
        Required::Acceptance { claims },
    );
    later
}

/// A store that hands every operation to the world's and, at each move
/// of `beta`, keeps the claim at the head of `beta`'s chain.
struct LedgerWatched {
    inner: FakeStore,
    keeper: FakeKeeper,
    heads: Rc<RefCell<Vec<Option<Digest>>>>,
}

impl Store for LedgerWatched {
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
        self.inner.claims_of(digest)
    }
    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
        if *name == channel_name(&beta()) {
            let head = self.keeper.chain(&beta()).ok();
            self.heads
                .borrow_mut()
                .push(head.and_then(|c| c.claims.last().map(Envelope::digest)));
        }
        self.inner.name(name, to, expected)
    }
    fn resolve(&self, name: &Name) -> Answer<Digest> {
        self.inner.resolve(name)
    }
    fn read(&self, digest: &Digest) -> Answer<Held> {
        self.inner.read(digest)
    }
}

fn a_version_accepted_and_beta_moved(world: &FakeWorld) {
    let published = offered(world);
    let (_, index) = version_of(&published);
    let all = digests(&published);
    let before = held(world, &all.iter().skip(1).collect::<Vec<_>>());
    let on_index = ok(world.fakes.store.claims_of(index), "claims-of").len();
    let written = world.writes().len();
    let heads = Rc::new(RefCell::new(Vec::new()));
    let mut seams = world.seams_holding(&[GATE]);
    seams.store = Box::new(LedgerWatched {
        inner: world.fakes.store.clone(),
        keeper: world.fakes.keeper.clone(),
        heads: heads.clone(),
    });

    let (acceptance, already, _, channels) = accepted(accept(
        &seams,
        &world.history(),
        &world.gate_ask(VERSION, told(&published), GATE),
    ));
    assert!(!already, "a version never judged was accepted already");

    // The acceptance is attached, signed in the gate's identity.
    let now = ok(world.fakes.store.claims_of(index), "claims-of");
    assert_eq!(
        now[on_index].digest(),
        acceptance,
        "the index's next claim is not the acceptance"
    );
    acceptance_by(world, index, GATE);

    // The bytes are the ones offered.
    assert_eq!(
        ok(world.fakes.store.resolve(&Name::new(TAG)), "resolve"),
        *index,
        "the version's name moved"
    );
    assert_eq!(
        held(world, &all.iter().skip(1).collect::<Vec<_>>()),
        before,
        "an artifact's claims changed"
    );
    let writes = world.writes().split_off(written);
    assert!(
        writes.iter().all(
            |w| matches!(w, Write::Attach { subject, .. } if subject == index)
                || matches!(w, Write::Name { name, .. } if *name == channel_name(&beta()))
        ),
        "the gate wrote other than claims on the index and beta's name: {writes:?}"
    );

    // beta moved, and the ledger held the move first.
    let [(channel, Ok(Moved::Promoted { to, promotion, .. }))] = channels.as_slice() else {
        panic!("beta was not promoted: {channels:?}")
    };
    assert_eq!(channel, &beta());
    assert_eq!(to, index);
    assert_eq!(
        ok(
            world.fakes.store.resolve(&channel_name(&beta())),
            "resolve beta"
        ),
        *index,
        "beta does not name the version"
    );
    assert_eq!(
        *heads.borrow(),
        [Some(promotion.clone())],
        "beta's name moved before its chain held the promotion"
    );
}

fn a_verdict_from_a_signer_the_policy_does_not_name(world: &FakeWorld) {
    let published = offered_with(world, acceptance(), |revision| {
        ["site", "server"]
            .iter()
            .map(|o| {
                world.verdict(
                    revision,
                    "test",
                    &build_of(o, revision),
                    Outcome::Pass,
                    "stranger",
                )
            })
            .collect()
    });
    let (_, index) = version_of(&published);
    let all = digests(&published);
    let before = held(world, &all.iter().collect::<Vec<_>>());
    let manifest = ok(world.fakes.store.read(index), "read");
    let written = world.writes().len();

    let (artifact, wanting, why) =
        refused(world.judge(&world.gate_ask(VERSION, told(&published), GATE), &[GATE]));
    assert_eq!(
        artifact,
        artifacts(&published)[0].0,
        "the refusal names another artifact"
    );
    assert_eq!(
        wanting,
        test_from_the_landing(),
        "the refusal names another claim"
    );
    assert_eq!(why, WhyWanting::Missing);

    assert_eq!(world.writes().len(), written, "a refusal wrote");
    beta_unmoved(world);
    assert_eq!(
        ok(world.fakes.store.resolve(&Name::new(TAG)), "resolve"),
        *index
    );
    assert_eq!(ok(world.fakes.store.read(index), "read"), manifest);
    assert_eq!(
        held(world, &all.iter().collect::<Vec<_>>()),
        before,
        "the version is not readable as offered"
    );
}

fn a_thiefs_acceptance(world: &FakeWorld) {
    let published = offered(world);
    let (_, index) = version_of(&published);
    let forged = Claim {
        subject: index.clone(),
        kind: ClaimKind::Acceptance,
        revision: revision_of(&published),
        build: None,
        body: json!({ "component": "repository", "version": VERSION }),
    };
    let forged = ok(
        world.fakes.signer.sign(&forged, &world.host),
        "the thief's sign",
    );
    ok(
        world.fakes.store.attach(index, &forged),
        "the thief's attach",
    );

    // The thief holds the build host's key alone.
    let (_, _, why) = refused(world.judge(&world.gate_ask(VERSION, told(&published), HOST), &[]));
    assert_eq!(
        why,
        WhyWanting::OfferedByGate {
            signer: world.host.clone()
        }
    );
    beta_unmoved(world);

    // beta's policy counts no acceptance, and the forged claim is the
    // build host's, as its provenance is.
    let for_beta = ok(
        world.verify(&world.verify_ask(VERSION, Audience::Channel(beta()))),
        "verify for beta",
    );
    let forged_told = |claims: &[tailrace_core::Told]| {
        claims
            .iter()
            .find(|t| t.envelope == forged.digest())
            .cloned()
            .expect("verify lists the forged claim")
    };
    assert!(
        !forged_told(&for_beta.claims).counts,
        "beta's policy counts the thief's acceptance"
    );
    let for_acceptance = ok(
        world.verify(&world.verify_ask(VERSION, Audience::Acceptance)),
        "verify for acceptance",
    );
    let told = forged_told(&for_acceptance.claims);
    assert_eq!(told.made_by.as_ref(), Some(&world.host));
    assert!(!told.counts, "the build host's acceptance counts");
    assert!(for_acceptance
        .claims
        .iter()
        .filter(|t| t.kind == Some(ClaimKind::Provenance))
        .all(|t| t.made_by.as_ref() == Some(&world.host)));
}

fn a_build_host_running_the_gate(world: &FakeWorld) {
    let published = offered(world);
    let written = world.writes().len();
    let (artifact, wanting, why) =
        refused(world.judge(&world.gate_ask(VERSION, told(&published), HOST), &[]));
    assert_eq!(artifact, artifacts(&published)[0].0);
    assert_eq!(wanting.kind, ClaimKind::Provenance);
    assert_eq!(
        why,
        WhyWanting::OfferedByGate {
            signer: world.host.clone()
        },
        "the gate accepted what its own identity offered"
    );
    assert_eq!(world.writes().len(), written, "a refusal wrote");
    beta_unmoved(world);
}

fn a_host_with_no_gate_key(world: &FakeWorld) {
    let published = offered(world);
    let (_, index) = version_of(&published);
    let written = world.writes().len();
    let answer = world.judge(&world.gate_ask(VERSION, told(&published), GATE), &[]);
    let Err(Not::Absent(a)) = &answer else {
        panic!("a host with no gate key answered {answer:?}")
    };
    assert_eq!(a.source.seam, Seam::Signer);
    assert!(a.what.contains(GATE), "{}", a.what);
    assert_eq!(
        world.writes().len(),
        written,
        "a host with no gate key wrote"
    );
    assert_eq!(of_kind(world, index, ClaimKind::Acceptance), 0);
    beta_unmoved(world);
}

fn a_publish_alone(world: &FakeWorld) {
    let published = offered(world);
    let (_, index) = version_of(&published);
    beta_unmoved(world);
    assert!(
        world.keeper_calls().is_empty(),
        "a publish asked the keeper"
    );
    assert_eq!(of_kind(world, index, ClaimKind::Acceptance), 0);
    assert_eq!(of_kind(world, index, ClaimKind::Promotion), 0);
}

fn acceptance_changes_no_bytes(world: &FakeWorld) {
    let published = offered(world);
    let (_, index) = version_of(&published);
    let all = digests(&published);
    let reads: Vec<Held> = all
        .iter()
        .map(|d| ok(world.fakes.store.read(d), "read"))
        .collect();
    let artifacts_held = held(world, &all.iter().skip(1).collect::<Vec<_>>());
    let written = world.writes().len();

    let (acceptance, ..) =
        accepted(world.judge(&world.gate_ask(VERSION, ToldBy::Asked, GATE), &[GATE]));
    let writes = world.writes().split_off(written);
    let [Write::Attach { subject, envelope }] = writes.as_slice() else {
        panic!("acceptance wrote {writes:?}, not one claim")
    };
    assert_eq!(subject, index, "the acceptance is not on the index");
    assert_eq!(envelope.digest(), acceptance);
    let now: Vec<Held> = all
        .iter()
        .map(|d| ok(world.fakes.store.read(d), "read"))
        .collect();
    assert_eq!(now, reads, "a digest holds other bytes");
    assert_eq!(
        held(world, &all.iter().skip(1).collect::<Vec<_>>()),
        artifacts_held
    );
    assert_eq!(
        ok(world.fakes.store.resolve(&Name::new(TAG)), "resolve"),
        *index
    );
}

fn one_artifact_wanting_refuses_the_version(world: &FakeWorld) {
    let published = offered_with(world, acceptance(), |revision| {
        [("site", "test"), ("server", "lint")]
            .iter()
            .map(|(o, check)| {
                world.verdict(
                    revision,
                    check,
                    &build_of(o, revision),
                    Outcome::Pass,
                    LANDING,
                )
            })
            .collect()
    });
    let all = digests(&published);
    let before = held(world, &all.iter().collect::<Vec<_>>());
    let written = world.writes().len();
    let server = artifacts(&published)
        .into_iter()
        .find(|(_, output)| output == "server")
        .map(|(d, _)| d)
        .expect("the server was offered");

    let (artifact, wanting, why) =
        refused(world.judge(&world.gate_ask(VERSION, told(&published), GATE), &[GATE]));
    assert_eq!(artifact, server, "the refusal names another artifact");
    assert_eq!(wanting, test_from_the_landing());
    assert_eq!(why, WhyWanting::Missing);
    assert_eq!(
        world.writes().len(),
        written,
        "part of the version was accepted"
    );
    assert_eq!(held(world, &all.iter().collect::<Vec<_>>()), before);
}

fn judged_by_the_claims_the_landing_and_the_policy_alone(world: &FakeWorld) {
    let published = offered(world);
    let (_, index) = version_of(&published);
    let revision = revision_of(&published);

    // A version whose revision the landing has not landed.
    let later = world.land_policy(POLICY);
    let landing = FakeLanding::new();
    landing.land(&world.main, &later);
    let mut seams = world.seams_holding(&[GATE]);
    seams.landing = Box::new(landing);
    let written = world.writes().len();
    let answer = accept(
        &seams,
        &world.history(),
        &world.gate_ask(VERSION, told(&published), GATE),
    );
    let Err(Not::Refused(r)) = &answer else {
        panic!("a version whose revision did not land answered {answer:?}")
    };
    assert_eq!(
        r.why,
        Why::NotLanded {
            branch: world.main.clone(),
            revision,
        }
    );
    assert_eq!(world.writes().len(), written);

    // An accepted version: the index read, the claims of each digest, and
    // nothing of any artifact's bytes; the build asked nothing.
    let (store, build) = (world.fakes.store.calls().len(), world.build_calls().len());
    accepted(world.judge(&world.gate_ask(VERSION, ToldBy::Asked, GATE), &[GATE]));
    for call in &world.fakes.store.calls()[store..] {
        if call.op == StoreOp::Read {
            assert_eq!(
                call.given["digest"],
                index.as_str(),
                "the gate read the bytes of {}",
                call.given["digest"]
            );
        }
    }
    assert_eq!(world.build_calls().len(), build, "the gate asked the build");
}

/// A rebuild of each artifact by `signer`, saying the digests `matched`
/// or did not.
fn rebuild_by(world: &FakeWorld, published: &Published, signer: &str, matched: bool) {
    let theirs = FakeSigner::new();
    let id = SignerId::new(signer);
    theirs.hold(&id);
    for (artifact, _) in artifacts(published) {
        let claim = Claim {
            subject: artifact.clone(),
            kind: ClaimKind::Rebuild,
            revision: revision_of(published),
            build: None,
            body: json!({ "matched": matched }),
        };
        let envelope = ok(theirs.sign(&claim, &id), "the rebuild's sign");
        ok(world.fakes.store.attach(&artifact, &envelope), "attach");
    }
}

fn an_independent_rebuild(world: &FakeWorld) {
    let rebuild = RequiredClaim {
        kind: ClaimKind::Rebuild,
        check: None,
        signers: vec![
            named(HOST, &[ClaimKind::Provenance, ClaimKind::Rebuild], &[]),
            named("verifier", &[ClaimKind::Rebuild], &[]),
        ],
        asked_of: AskedOf::Built,
    };
    let published = offered_under(
        world,
        Required::Acceptance {
            claims: vec![
                provenance_from(named(HOST, &[ClaimKind::Provenance], &[])),
                rebuild.clone(),
            ],
        },
    );
    let ask = world.gate_ask(VERSION, ToldBy::Asked, GATE);

    rebuild_by(world, &published, "verifier", false);
    let (_, wanting, why) = refused(world.judge(&ask, &[GATE]));
    assert_eq!(wanting, rebuild);
    assert_eq!(
        why,
        WhyWanting::Unmatched {
            signer: SignerId::new("verifier")
        },
        "a rebuild whose digests did not match counted"
    );

    rebuild_by(world, &published, HOST, true);
    let (_, wanting, _) = refused(world.judge(&ask, &[GATE]));
    assert_eq!(wanting, rebuild, "the offering host's rebuild counted");

    rebuild_by(world, &published, "verifier", true);
    accepted(world.judge(&ask, &[GATE]));
}

fn nothing_accepted_is_un_accepted(world: &FakeWorld) {
    offered(world);
    let ask = world.gate_ask(VERSION, ToldBy::Asked, GATE);
    let (first, ..) = accepted(world.judge(&ask, &[GATE]));
    stricter(world);
    let written = world.writes().len();

    let (acceptance, already, ..) = accepted(world.judge(&ask, &[GATE]));
    assert!(already, "the acceptance was not found");
    assert_eq!(acceptance, first, "another acceptance was answered");
    assert_eq!(
        world.writes().len(),
        written,
        "an accepted version was written again"
    );
}

fn asked_rather_than_told(world: &FakeWorld) {
    offered(world);
    let (_, _, _, channels) =
        accepted(world.judge(&world.gate_ask(VERSION, ToldBy::Asked, GATE), &[GATE]));
    assert!(
        channels.is_empty(),
        "an asked judgment followed {channels:?}"
    );
    beta_unmoved(world);
}

fn the_policy_at_the_landed_head(world: &FakeWorld) {
    let published = offered(world);
    let later = stricter(world);
    let policy = world.fakes.policy.calls().len();
    let answer = world.judge(&world.gate_ask(VERSION, ToldBy::Asked, GATE), &[GATE]);
    let Ok(Judged::Refused {
        revision,
        policy_at,
        wanting,
        ..
    }) = &answer
    else {
        panic!("the version was not refused under the later policy: {answer:?}")
    };
    assert_eq!(revision, &revision_of(&published));
    assert_eq!(policy_at, &later, "the policy was read elsewhere");
    assert_eq!(wanting, &review());
    for call in &world.fakes.policy.calls()[policy..] {
        assert_eq!(
            call.given["revision"],
            later.as_str(),
            "the policy was asked at {}",
            call.given["revision"]
        );
    }
}

fn one_host_offers_and_accepts(world: &FakeWorld) {
    let published = offered_under(
        world,
        Required::Acceptance {
            claims: vec![
                provenance_from(named(
                    HOST,
                    &[ClaimKind::Provenance, ClaimKind::Acceptance],
                    &[],
                )),
                test_from_the_landing(),
            ],
        },
    );
    let (_, index) = version_of(&published);
    let (_, _, one_host, _) =
        accepted(world.judge(&world.gate_ask(VERSION, ToldBy::Asked, HOST), &[]));
    assert!(one_host, "the one host's acceptance does not say so");
    assert_eq!(
        acceptance_by(world, index, HOST).body["offered"],
        json!({ "by": [HOST], "one_host": true })
    );
}

fn a_separated_gate_names_its_offerer(world: &FakeWorld) {
    let published = offered(world);
    let (_, index) = version_of(&published);
    let (_, _, one_host, _) =
        accepted(world.judge(&world.gate_ask(VERSION, ToldBy::Asked, GATE), &[GATE]));
    assert!(!one_host);
    assert_eq!(
        acceptance_by(world, index, GATE).body["offered"],
        json!({ "by": [HOST], "one_host": false })
    );
    assert!(
        matches!(
            world.judge(&world.gate_ask(VERSION, ToldBy::Asked, GATE), &[GATE]),
            Ok(Judged::Accepted { already: true, .. })
        ),
        "the separated acceptance was not the gate's"
    );
}

fn the_core_starts_nothing_after_an_offer(world: &FakeWorld) {
    let published = offered(world);
    let (_, index) = version_of(&published);

    // The publish answered, and nothing is accepted, promoted or asked of
    // the ledger.
    assert!(
        world.keeper_calls().is_empty(),
        "a publish asked the keeper"
    );
    for digest in digests(&published) {
        for kind in [ClaimKind::Acceptance, ClaimKind::Promotion] {
            assert_eq!(
                of_kind(world, &digest, kind),
                0,
                "a publish attached {kind:?} to {digest}"
            );
        }
    }
    beta_unmoved(world);

    // The gate, told of the publish's answer, accepts the index it names
    // and moves beta.
    let (acceptance, _, _, channels) =
        accepted(world.judge(&world.gate_ask(VERSION, told(&published), GATE), &[GATE]));
    assert!(
        ok(world.fakes.store.claims_of(index), "claims-of")
            .iter()
            .any(|e| e.digest() == acceptance),
        "the acceptance is not on the index the publish named"
    );
    acceptance_by(world, index, GATE);
    let [(channel, Ok(Moved::Promoted { to, promotion, .. }))] = channels.as_slice() else {
        panic!("beta was not promoted: {channels:?}")
    };
    assert_eq!(channel, &beta());
    assert_eq!(to, index);
    let chain = ok(world.fakes.keeper.chain(&beta()), "beta's chain").claims;
    assert_eq!(
        chain.iter().map(Envelope::digest).collect::<Vec<_>>(),
        std::slice::from_ref(promotion),
        "beta's chain does not hold the promotion"
    );
}

fn a_told_index_the_name_does_not_resolve_to(world: &FakeWorld) {
    let published = offered(world);
    let (_, index) = version_of(&published);
    let (other, _) = artifacts(&published).remove(0);
    let written = world.writes().len();
    let signed = world.fakes.signer.calls().len();

    let told = ToldBy::Publish {
        index: other.clone(),
    };
    let answer = world.judge(&world.gate_ask(VERSION, told, GATE), &[GATE]);
    let Err(Not::Unreadable(u)) = &answer else {
        panic!("a told index the name does not resolve to answered {answer:?}")
    };
    let said = u.said.clone().unwrap_or_default();
    assert!(
        said.contains(index.as_str()) && said.contains(other.as_str()),
        "the answer does not name both digests: {said}"
    );
    assert_eq!(world.writes().len(), written, "the store was written");
    assert!(
        world.fakes.keeper.calls().is_empty(),
        "the keeper was asked"
    );
    assert_eq!(
        world.fakes.signer.calls().len(),
        signed,
        "the host's signer was asked"
    );
    assert_eq!(of_kind(world, index, ClaimKind::Acceptance), 0);
    beta_unmoved(world);
}

/// 95, 125, 131: the repository of two components, the fake policy
/// listing `beta` of each moving on each publish, a version of `lib` a
/// publish told the gate of: `lib`'s `beta` moves, `app`'s does not, and
/// the ledger holds one chain. The world's policy file names no channel,
/// so every channel the gate moves is one the policy seam lists.
fn beta_of_one_component(_: &FakeWorld) {
    let world = FakeWorld::two_components();
    let head = world.head.clone();
    world.land(&head, &[]);
    world.outputs(
        &head,
        &[("library", Outcome::Pass), ("server", Outcome::Pass)],
    );
    let policy = &world.fakes.policy;
    policy.require(
        Audience::Acceptance,
        Required::Acceptance {
            claims: vec![provenance_from(named(HOST, &[ClaimKind::Provenance], &[]))],
        },
    );
    let gate = named(GATE, &[ClaimKind::Acceptance], &[]);
    let beta = |component: &str| {
        ChannelId::new(Some(ComponentName::new(component)), ChannelName::new(BETA))
    };
    for channel in [beta("lib"), beta("app")] {
        policy.require(
            Audience::Channel(channel.clone()),
            Required::Channel {
                claims: vec![RequiredClaim {
                    kind: ClaimKind::Acceptance,
                    check: None,
                    signers: vec![gate.clone()],
                    asked_of: AskedOf::Every,
                }],
                bound_to: None,
                promoted_by: vec![gate.clone()],
                approvers: vec![],
            },
        );
        policy.follows(&channel, false);
    }
    let publish = ok(world.publish(&world.ask(&head)), "the publish");
    let (_, published) = publish
        .components
        .iter()
        .find(|(c, _)| c.as_str() == "lib")
        .expect("the publish answered lib");
    let published = ok(published.clone(), "lib's publish");
    let (version, index) = version_of(&published);

    let ask = tailrace_core::AcceptAsk {
        component: Some(ComponentName::new("lib")),
        ..world.gate_ask(version.as_str(), told(&published), GATE)
    };
    let (_, _, _, channels) = accepted(world.judge(&ask, &[GATE]));
    let [(channel, Ok(Moved::Promoted { to, .. }))] = channels.as_slice() else {
        panic!("lib's beta alone was not promoted: {channels:?}")
    };
    assert_eq!(
        channel,
        &beta("lib"),
        "another channel than lib's beta moved"
    );
    assert_eq!(to, index);
    assert_eq!(
        world
            .fakes
            .store
            .resolve(&channel_name(&beta("lib")))
            .as_ref(),
        Ok(index),
        "lib's beta does not name the version"
    );
    assert!(
        world
            .fakes
            .store
            .resolve(&channel_name(&beta("app")))
            .is_err(),
        "app's beta names a version"
    );
    assert_eq!(
        ok(world.fakes.keeper.chain(&beta("lib")), "lib's beta's chain")
            .claims
            .len(),
        1
    );
    assert!(
        matches!(world.fakes.keeper.chain(&beta("app")), Err(Not::Absent(_))),
        "app's beta has a chain"
    );
}

/// 95, 131: `alpha`, which takes prereleases, and `beta`, which takes
/// releases alone, follow each publish of the world's one component,
/// each requiring and promoted by the gate's acceptance; then `revision`
/// landed, built, published and told to the gate: each channel moved, by
/// name.
fn followed_on_publish(world: &FakeWorld, revision: &Revision) -> Vec<(String, bool)> {
    world.land(revision, &[]);
    world.outputs(
        revision,
        &[("site", Outcome::Pass), ("server", Outcome::Pass)],
    );
    let policy = &world.fakes.policy;
    policy.require(
        Audience::Acceptance,
        Required::Acceptance {
            claims: vec![provenance_from(named(HOST, &[ClaimKind::Provenance], &[]))],
        },
    );
    let gate = named(GATE, &[ClaimKind::Acceptance], &[]);
    for (name, prereleases) in [("alpha", true), (BETA, false)] {
        let channel = ChannelId::new(None, ChannelName::new(name));
        policy.require(
            Audience::Channel(channel.clone()),
            Required::Channel {
                claims: vec![RequiredClaim {
                    kind: ClaimKind::Acceptance,
                    check: None,
                    signers: vec![gate.clone()],
                    asked_of: AskedOf::Every,
                }],
                bound_to: None,
                promoted_by: vec![gate.clone()],
                approvers: vec![],
            },
        );
        policy.follows(&channel, prereleases);
    }
    let publish = ok(world.publish(&world.ask(revision)), "the publish");
    let published = ok(publish.components[0].1.clone(), "the publish's version");
    let (version, _) = version_of(&published);
    let ask = world.gate_ask(version.as_str(), told(&published), GATE);
    let (_, _, _, channels) = accepted(world.judge(&ask, &[GATE]));
    channels
        .into_iter()
        .map(|(c, moved)| {
            (
                c.name.to_string(),
                matches!(moved, Ok(Moved::Promoted { .. })),
            )
        })
        .collect()
}

fn a_prerelease_and_the_channels_that_follow(world: &FakeWorld) {
    assert_eq!(
        followed_on_publish(world, &world.head),
        [("alpha".to_string(), true), (BETA.to_string(), true)],
        "a release did not move both following channels"
    );

    let other = FakeWorld::new();
    let repo = other.repo();
    repo.tag("v0.2.0", &other.head);
    let tidied = repo.commit(
        &[("server/main.txt", "the server tidied")],
        "chore: the server tidied",
    );
    repo.push();
    assert_eq!(
        followed_on_publish(&other, &tidied),
        [("alpha".to_string(), true)],
        "a prerelease moved another channel than the one that takes prereleases"
    );
    assert!(
        matches!(
            other
                .fakes
                .keeper
                .chain(&ChannelId::new(None, ChannelName::new(BETA))),
            Err(Not::Absent(_))
        ),
        "beta's chain was appended to by a prerelease"
    );
}

fn a_version_whose_notes_carry_no_build_verdict(world: &FakeWorld) {
    let published = offered(world);
    let (_, index) = version_of(&published);
    let (notes, provenance) = notes_of(&published);
    let on_notes = ok(world.fakes.store.claims_of(&notes), "claims-of the notes");
    assert_eq!(
        on_notes.iter().map(Envelope::digest).collect::<Vec<_>>(),
        std::slice::from_ref(&provenance),
        "the notes carry other than the publisher's provenance"
    );
    assert_eq!(
        Claim::from_envelope(&on_notes[0]).map(|c| c.build),
        Some(None),
        "the notes' provenance names a build"
    );

    let (_, _, _, channels) =
        accepted(world.judge(&world.gate_ask(VERSION, told(&published), GATE), &[GATE]));
    let satisfied: Vec<String> = acceptance_by(world, index, GATE).body["claims"]
        .as_array()
        .expect("the acceptance names the claims that satisfied it")
        .iter()
        .map(|c| c.as_str().unwrap_or_default().to_string())
        .collect();
    assert!(
        satisfied.contains(&provenance.to_string()),
        "the acceptance does not name the notes' provenance: {satisfied:?}"
    );
    // Each output's provenance and verdict, and the notes' provenance.
    assert_eq!(
        satisfied.len(),
        5,
        "the acceptance names other claims: {satisfied:?}"
    );
    assert!(
        matches!(channels.as_slice(), [(_, Ok(Moved::Promoted { .. }))]),
        "beta was not promoted: {channels:?}"
    );
}

/// A store that hands every operation to the world's, but answers no
/// claim of `hidden`.
struct Hiding {
    inner: FakeStore,
    hidden: Digest,
}

impl Store for Hiding {
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
        if *digest == self.hidden {
            return Ok(vec![]);
        }
        self.inner.claims_of(digest)
    }
    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
        self.inner.name(name, to, expected)
    }
    fn resolve(&self, name: &Name) -> Answer<Digest> {
        self.inner.resolve(name)
    }
    fn read(&self, digest: &Digest) -> Answer<Held> {
        self.inner.read(digest)
    }
}

fn a_rendered_artifact_wanting_the_publishers_provenance(world: &FakeWorld) {
    let published = offered(world);
    let (notes, _) = notes_of(&published);
    let mut seams = world.seams_holding(&[GATE]);
    seams.store = Box::new(Hiding {
        inner: world.fakes.store.clone(),
        hidden: notes.clone(),
    });
    let written = world.writes().len();

    let (artifact, wanting, why) = refused(accept(
        &seams,
        &world.history(),
        &world.gate_ask(VERSION, told(&published), GATE),
    ));
    assert_eq!(artifact, notes, "the refusal names another artifact");
    assert_eq!(
        wanting,
        provenance_from(named(HOST, &[ClaimKind::Provenance], &[])),
        "the refusal names another claim"
    );
    assert_eq!(why, WhyWanting::Missing);
    assert_eq!(world.writes().len(), written, "a refusal wrote");
    beta_unmoved(world);
}

fn the_outputs_claims_wanting_beside_the_notes(world: &FakeWorld) {
    let published = offered_with(world, acceptance(), |revision| {
        [("site", "test"), ("server", "lint")]
            .iter()
            .map(|(o, check)| {
                world.verdict(
                    revision,
                    check,
                    &build_of(o, revision),
                    Outcome::Pass,
                    LANDING,
                )
            })
            .collect()
    });
    let (notes, _) = notes_of(&published);
    let server = artifacts(&published)
        .into_iter()
        .find(|(_, output)| output == "server")
        .map(|(d, _)| d)
        .expect("the server was offered");
    let written = world.writes().len();

    let (artifact, wanting, why) =
        refused(world.judge(&world.gate_ask(VERSION, told(&published), GATE), &[GATE]));
    assert_ne!(artifact, notes, "the notes were asked a build's verdict");
    assert_eq!(artifact, server, "the refusal names another artifact");
    assert_eq!(wanting, test_from_the_landing());
    assert_eq!(why, WhyWanting::Missing);
    assert_eq!(world.writes().len(), written, "a refusal wrote");
}

fn a_following_channel_and_the_notes(world: &FakeWorld) {
    let published = offered(world);
    let (_, index) = version_of(&published);
    let gate = named(GATE, &[ClaimKind::Acceptance], &[]);
    let rebuild = RequiredClaim {
        kind: ClaimKind::Rebuild,
        check: None,
        signers: vec![named("verifier", &[ClaimKind::Rebuild], &[])],
        asked_of: AskedOf::Built,
    };
    world.fakes.policy.require(
        Audience::Channel(beta()),
        Required::Channel {
            claims: vec![
                RequiredClaim {
                    kind: ClaimKind::Acceptance,
                    check: None,
                    signers: vec![gate.clone()],
                    asked_of: AskedOf::Every,
                },
                rebuild.clone(),
            ],
            bound_to: None,
            promoted_by: vec![gate],
            approvers: vec![],
        },
    );
    let outputs = artifacts(&published);
    let verifier = FakeSigner::new();
    let id = SignerId::new("verifier");
    verifier.hold(&id);
    let rebuilt = |artifact: &Digest| {
        let claim = Claim {
            subject: artifact.clone(),
            kind: ClaimKind::Rebuild,
            revision: revision_of(&published),
            build: None,
            body: json!({ "matched": true }),
        };
        let envelope = ok(verifier.sign(&claim, &id), "the rebuild's sign");
        ok(world.fakes.store.attach(artifact, &envelope), "attach");
    };
    let ask = world.gate_ask(VERSION, told(&published), GATE);

    // The first output carries no rebuild: accepted, and beta held for it.
    rebuilt(&outputs[1].0);
    let (_, already, _, channels) = accepted(world.judge(&ask, &[GATE]));
    assert!(!already, "a version never judged was accepted already");
    let [(
        channel,
        Ok(Moved::Held {
            artifact,
            wanting,
            why,
        }),
    )] = channels.as_slice()
    else {
        panic!("beta was not held: {channels:?}")
    };
    assert_eq!(channel, &beta());
    assert_eq!(
        artifact, &outputs[0].0,
        "beta was held for another artifact"
    );
    assert_eq!(wanting, &rebuild);
    assert_eq!(why, &WhyWanting::Missing);
    beta_unmoved(world);

    // Each output rebuilt and the notes not: beta moves.
    rebuilt(&outputs[0].0);
    let (_, _, _, channels) = accepted(world.judge(&ask, &[GATE]));
    let [(_, Ok(Moved::Promoted { to, .. }))] = channels.as_slice() else {
        panic!("beta was held for the notes: {channels:?}")
    };
    assert_eq!(to, index);
    assert_eq!(
        ok(
            world.fakes.store.resolve(&channel_name(&beta())),
            "resolve beta"
        ),
        *index,
        "beta does not name the version"
    );
}
