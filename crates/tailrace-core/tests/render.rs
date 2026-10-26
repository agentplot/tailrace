//! Consumer layouts rendered under a policy, over the fakes (design D2
//! to D7).

mod common;

use std::fs;
use std::path::PathBuf;

use serde_json::json;
use tailrace_core::render::Kept;
use tailrace_core::{
    artifact_name, judge, layout_policy_body, locate, own_policy, place, policy_for, refresh,
    render, sync_checkout, Carried, CheckoutAsk, HeldEntry, HeldPin, Landed, Layouts, Located, Own,
    PinAnswer, PolicyOf, Provenance, Refresh, RenderAsk, Under, ARTIFACT, CACHE,
};
use tailrace_fakes::{FakeProjection, Fakes, Fault, PolicyOp, ProjectionOp, StoreOp, Times};
use tailrace_seams::{
    Artifact, ArtifactMeta, Audience, Branch, BuildId, Check, Claim, ClaimKind, ComponentName,
    Digest, Ecosystem, EcosystemName, Envelope, IndexMeta, Manifest, Name, NamedSigner, Not,
    Outcome, OutputName, Pinned, Required, RequiredClaim, Revision, Seam, Signer, SignerId, Store,
    Verdict, Version, Why,
};
use tempfile::TempDir;

const ECO: &str = "eco";

fn eco() -> Ecosystem {
    Ecosystem::new(ECO)
}

fn pin(name: &str, version: &str) -> Pinned {
    Pinned {
        name: EcosystemName::new(name),
        version: Version::new(version),
    }
}

fn meta(ecosystem: &str, name: &str, version: &str) -> ArtifactMeta {
    ArtifactMeta::new(
        Ecosystem::new(ecosystem),
        EcosystemName::new(name),
        Version::new(version),
    )
    .unwrap()
}

/// An artifact of `meta` put in `store`, its bytes and its manifest.
fn artifact(store: &dyn Store, meta: ArtifactMeta) -> Artifact {
    let bytes = format!("{} {} {}", meta.ecosystem(), meta.name(), meta.version());
    let layer = store.put_bytes(bytes.as_bytes()).unwrap().digest;
    let digest = store
        .put_artifact(&Manifest::Artifact {
            meta: meta.clone(),
            layers: vec![layer],
        })
        .unwrap()
        .digest;
    Artifact { digest, meta }
}

/// The index of `component` at `version` naming `artifacts`, put in
/// `store`.
fn index(store: &dyn Store, component: &str, version: &str, artifacts: Vec<Artifact>) -> Digest {
    let meta = IndexMeta::new(
        ComponentName::new(component),
        Version::new(version),
        artifacts,
    )
    .unwrap();
    store.put_artifact(&Manifest::Index(meta)).unwrap().digest
}

/// `version` of `name` in the ecosystem published to `store`, with a
/// second artifact of another ecosystem beside it in its index, and the
/// name a pin resolves through moved to the index.
fn published(store: &dyn Store, name: &str, version: &str) -> Located {
    let ours = artifact(store, meta(ECO, name, version));
    let other = artifact(store, meta("other", name, version));
    let index = index(store, name, version, vec![other, ours.clone()]);
    store
        .name(
            &artifact_name(&eco(), &EcosystemName::new(name), &Version::new(version)),
            &index,
            None,
        )
        .unwrap();
    Located {
        index,
        artifact: ours,
    }
}

#[test]
fn a_pins_name_is_its_ecosystem_name_and_version_under_artifact() {
    assert_eq!(ARTIFACT, "artifact/");
    assert_eq!(
        artifact_name(
            &eco(),
            &EcosystemName::new("@scope/pkg"),
            &Version::new("1.2.0-rc.1")
        ),
        Name::new("artifact/eco/@scope/pkg@1.2.0-rc.1")
    );
}

#[test]
fn a_pin_resolves_through_its_index_to_the_artifact_of_its_ecosystem_and_name() {
    let fakes = Fakes::new();
    let found = published(&fakes.store, "pkg", "1.1.0");
    assert_eq!(
        locate(&fakes.store, &eco(), &pin("pkg", "1.1.0")),
        Ok(Ok(found.clone()))
    );
    assert_eq!(found.artifact.meta, meta(ECO, "pkg", "1.1.0"));
    let ops: Vec<StoreOp> = fakes.store.calls().into_iter().map(|c| c.op).collect();
    assert_eq!(
        ops[ops.len() - 2..],
        [StoreOp::Resolve, StoreOp::Read],
        "a pin was found by other means"
    );
}

#[test]
fn a_name_the_store_does_not_hold_is_absent_naming_it() {
    let fakes = Fakes::new();
    published(&fakes.store, "pkg", "1.1.0");
    match locate(&fakes.store, &eco(), &pin("left-pad", "1.3.0")) {
        Ok(Err(Not::Absent(a))) => {
            assert_eq!(a.source.seam, Seam::Store);
            assert_eq!(a.what, "name artifact/eco/left-pad@1.3.0");
        }
        other => panic!("a third-party pin answered {other:?}"),
    }
}

#[test]
fn a_name_at_no_such_index_is_unreadable_naming_the_name_and_the_digest() {
    let fakes = Fakes::new();
    let store = &fakes.store;
    let unreadable = |pin: &Pinned, at: &Digest, says: &str| match locate(store, &eco(), pin) {
        Ok(Err(Not::Unreadable(u))) => {
            let name = artifact_name(&eco(), &pin.name, &pin.version);
            assert_eq!(u.what, format!("{name} at {at}"));
            let said = u.said.unwrap();
            assert!(said.contains(says), "{said}");
        }
        other => panic!("{pin:?} answered {other:?}"),
    };

    let bytes = store.put_bytes(b"no index").unwrap().digest;
    let at_bytes = pin("bytes", "1.0.0");
    let name = artifact_name(&eco(), &at_bytes.name, &at_bytes.version);
    store.name(&name, &bytes, None).unwrap();
    unreadable(&at_bytes, &bytes, "holds no index");

    let elsewhere = index(
        store,
        "lib",
        "2.0.0",
        vec![artifact(store, meta("other", "lib", "2.0.0"))],
    );
    let other_ecosystem = pin("lib", "2.0.0");
    let name = artifact_name(&eco(), &other_ecosystem.name, &other_ecosystem.version);
    store.name(&name, &elsewhere, None).unwrap();
    unreadable(
        &other_ecosystem,
        &elsewhere,
        "names 0 artifacts of eco lib 2.0.0",
    );

    let gone = Digest::of(b"never held");
    let at_nothing = pin("gone", "1.0.0");
    let name = artifact_name(&eco(), &at_nothing.name, &at_nothing.version);
    store.name(&name, &gone, None).unwrap();
    unreadable(&at_nothing, &gone, "is not held");
}

#[test]
fn a_store_that_cannot_be_read_is_the_outer_answer() {
    let fakes = Fakes::new();
    published(&fakes.store, "pkg", "1.1.0");
    fakes.store.fault(
        StoreOp::Resolve,
        Fault::Unreadable("connection refused".into()),
        Times::Once,
    );
    match locate(&fakes.store, &eco(), &pin("pkg", "1.1.0")) {
        Err(Not::Unreadable(u)) => assert_eq!(u.said.as_deref(), Some("connection refused")),
        other => panic!("an unreachable store answered {other:?}"),
    }
    fakes
        .store
        .fault(StoreOp::Read, Fault::Refused("denied".into()), Times::Once);
    assert!(matches!(
        locate(&fakes.store, &eco(), &pin("pkg", "1.1.0")),
        Err(Not::Refused(r)) if r.said.as_deref() == Some("denied")
    ));
}

const GATE: &str = "gate";
const HOST: &str = "host";

fn named(id: &str, kinds: &[ClaimKind], checks: &[&str]) -> NamedSigner {
    NamedSigner {
        id: SignerId::new(id),
        verified_by: format!("{id}'s key"),
        kinds: kinds.to_vec(),
        checks: checks.iter().map(|c| Check::new(*c)).collect(),
    }
}

/// Acceptance by the gate's signer.
fn acceptance() -> RequiredClaim {
    RequiredClaim {
        kind: ClaimKind::Acceptance,
        check: None,
        signers: vec![named(GATE, &[ClaimKind::Acceptance], &[])],
    }
}

/// Provenance by the build host's signer.
fn provenance() -> RequiredClaim {
    RequiredClaim {
        kind: ClaimKind::Provenance,
        check: None,
        signers: vec![named(HOST, &[ClaimKind::Provenance], &[])],
    }
}

/// `revision` landed on `main` in `fakes`.
fn landed(fakes: &Fakes, revision: &str) -> Landed {
    let (main, revision) = (Branch::new("main"), Revision::new(revision));
    fakes.landing.land(&main, &revision);
    Landed::check(&fakes.landing, &main, &revision).unwrap()
}

/// A claim of `kind` about `subject` with `body`, signed as `signer`.
fn signed(subject: &Digest, kind: ClaimKind, body: serde_json::Value, signer: &str) -> Envelope {
    let id = SignerId::new(signer);
    let theirs = tailrace_fakes::FakeSigner::new();
    theirs.hold(&id);
    let claim = Claim {
        subject: subject.clone(),
        kind,
        revision: Revision::new("p1b11s4"),
        build: None,
        body,
    };
    theirs.sign(&claim, &id).unwrap()
}

/// The publisher's layout policy requiring `claims`, about `index`.
fn publishers(index: &Digest, claims: &[RequiredClaim], signer: &str) -> Envelope {
    signed(
        index,
        ClaimKind::LayoutPolicy,
        layout_policy_body(claims),
        signer,
    )
}

#[test]
fn the_consumers_policy_is_read_at_the_landed_revision() {
    let fakes = Fakes::new();
    let at = landed(&fakes, "4f2a9c1");
    fakes.policy.require(
        Audience::Layout,
        Required::Layout {
            claims: vec![provenance()],
            given: false,
        },
    );
    fakes.policy.require_at(
        at.revision(),
        Audience::Layout,
        Required::Layout {
            claims: vec![acceptance(), provenance()],
            given: false,
        },
    );
    assert_eq!(
        own_policy(&fakes.policy, &at),
        Ok(Some(Own {
            revision: Revision::new("4f2a9c1"),
            claims: vec![acceptance(), provenance()],
            given: false,
        }))
    );
    let [asked] = <[_; 1]>::try_from(fakes.policy.calls()).unwrap();
    assert_eq!(asked.op, PolicyOp::PolicyFor);
    assert_eq!(asked.given["revision"], "4f2a9c1");
    assert_eq!(asked.given["audience"], "layout");

    fakes.policy.require_at(
        at.revision(),
        Audience::Layout,
        Required::Layout {
            claims: vec![acceptance()],
            given: true,
        },
    );
    assert_eq!(
        own_policy(&fakes.policy, &at),
        Ok(Some(Own {
            revision: Revision::new("4f2a9c1"),
            claims: vec![acceptance()],
            given: true,
        })),
        "the file's given was read as a layout line it writes"
    );
}

#[test]
fn a_consumer_with_no_policy_file_has_none_and_one_unread_answers_its_words() {
    let fakes = Fakes::new();
    let at = landed(&fakes, "4f2a9c1");
    assert_eq!(own_policy(&fakes.policy, &at), Ok(None));
    fakes.policy.fault(
        PolicyOp::PolicyFor,
        Fault::Unreadable("the policy file does not parse".into()),
        Times::Once,
    );
    assert!(matches!(
        own_policy(&fakes.policy, &at),
        Err(Not::Unreadable(u)) if u.said.as_deref() == Some("the policy file does not parse")
    ));
}

/// The signers the host trusts, each by the host's own key for it,
/// allowed to claim a layout policy alone.
fn trusts(ids: &[&str]) -> Vec<NamedSigner> {
    ids.iter()
        .map(|id| NamedSigner {
            id: SignerId::new(*id),
            verified_by: format!("the host's key for {id}"),
            kinds: vec![ClaimKind::LayoutPolicy],
            checks: vec![],
        })
        .collect()
}

/// `claim` as the host counts it: its signers that the host trusts, each
/// verified by the host's key for it.
fn as_trusted(claim: RequiredClaim, ids: &[&str]) -> RequiredClaim {
    RequiredClaim {
        signers: claim
            .signers
            .into_iter()
            .filter(|s| ids.contains(&s.id.as_str()))
            .map(|s| NamedSigner {
                verified_by: format!("the host's key for {}", s.id),
                ..s
            })
            .collect(),
        ..claim
    }
}

/// The consumer's policy for layouts at the landed revision requiring
/// `claims`: a layout line its file writes, or its file's given.
fn consumers(claims: Vec<RequiredClaim>, given: bool) -> Own {
    Own {
        revision: Revision::new("4f2a9c1"),
        claims,
        given,
    }
}

/// The given for a consumer with no policy file: acceptance by the
/// signers the host trusts, `ids`, each by the host's key for it and
/// allowed acceptance alone.
fn given_by_trusted(ids: &[&str]) -> (PolicyOf, Vec<RequiredClaim>) {
    let signers = ids
        .iter()
        .map(|id| NamedSigner {
            id: SignerId::new(*id),
            verified_by: format!("the host's key for {id}"),
            kinds: vec![ClaimKind::Acceptance],
            checks: vec![],
        })
        .collect();
    let acceptance = RequiredClaim {
        kind: ClaimKind::Acceptance,
        check: None,
        signers,
    };
    (PolicyOf::Given { revision: None }, vec![acceptance])
}

#[test]
fn the_consumers_written_line_judges_over_a_publishers_claim() {
    let fakes = Fakes::new();
    let index = Digest::of(b"index");
    let own = consumers(vec![provenance()], false);
    let on_index = [publishers(&index, &[acceptance()], GATE)];
    assert_eq!(
        policy_for(
            &fakes.seams(),
            Some(&own),
            &trusts(&[GATE]),
            &index,
            &on_index
        ),
        Ok(Ok((
            PolicyOf::Own {
                revision: Revision::new("4f2a9c1")
            },
            vec![provenance()]
        )))
    );
    assert_eq!(fakes.signer.calls(), [], "the publisher's claim was told");
}

#[test]
fn with_no_line_of_its_own_a_version_is_judged_under_its_publishers_through_a_trusted_signer() {
    let index = Digest::of(b"index");
    let on_index = [
        signed(&index, ClaimKind::Acceptance, json!(null), GATE),
        publishers(&index, &[acceptance(), provenance()], HOST),
        publishers(&index, &[acceptance(), provenance()], GATE),
        publishers(&Digest::of(b"another index"), &[provenance()], GATE),
    ];
    let files_given = consumers(vec![acceptance()], true);
    for own in [None, Some(&files_given)] {
        let fakes = Fakes::new();
        let (policy, claims) = policy_for(
            &fakes.seams(),
            own,
            &trusts(&[GATE, HOST]),
            &index,
            &on_index,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            policy,
            PolicyOf::Publisher {
                revision: Revision::new("p1b11s4")
            },
            "under {own:?}"
        );
        assert_eq!(
            claims,
            [
                as_trusted(acceptance(), &[GATE]),
                as_trusted(provenance(), &[HOST])
            ]
        );
        assert_eq!(
            claims[0].signers[0].verified_by, "the host's key for gate",
            "the publisher's signer is verified by a key the host does not hold"
        );
        assert_eq!(
            serde_json::to_value(&policy).unwrap(),
            json!({ "publisher": { "revision": "p1b11s4" } })
        );
    }
}

#[test]
fn a_publishers_policy_by_a_signer_the_host_does_not_trust_falls_to_the_given() {
    let fakes = Fakes::new();
    let index = Digest::of(b"index");
    let strangers = publishers(&index, &[provenance()], "stranger");
    let answer = policy_for(
        &fakes.seams(),
        None,
        &trusts(&[GATE]),
        &index,
        std::slice::from_ref(&strangers),
    );
    assert_eq!(answer, Ok(Ok(given_by_trusted(&[GATE]))));

    let gates = publishers(&index, &[acceptance()], GATE);
    let (policy, claims) = policy_for(
        &fakes.seams(),
        None,
        &trusts(&[GATE]),
        &index,
        &[strangers, gates],
    )
    .unwrap()
    .unwrap();
    assert!(
        matches!(policy, PolicyOf::Publisher { .. }),
        "the untrusted claim was counted beside the trusted one: {policy:?}"
    );
    assert_eq!(claims, [as_trusted(acceptance(), &[GATE])]);
}

#[test]
fn a_required_claim_through_a_signer_the_host_does_not_trust_is_found_wanting() {
    let (fakes, located, _) = accepted();
    let gate_or_box = RequiredClaim {
        kind: ClaimKind::Acceptance,
        check: None,
        signers: vec![
            named(GATE, &[ClaimKind::Acceptance], &[]),
            named("box", &[ClaimKind::Acceptance], &[]),
        ],
    };
    let on_index = [
        publishers(&located.index, std::slice::from_ref(&gate_or_box), GATE),
        signed(&located.index, ClaimKind::Acceptance, json!(null), "box"),
    ];
    let seams = fakes.seams();
    let (_, claims) = policy_for(&seams, None, &trusts(&[GATE]), &located.index, &on_index)
        .unwrap()
        .unwrap();
    assert_eq!(claims, [as_trusted(gate_or_box, &[GATE])]);
    let (wanted, checked) = wanting(judge(&seams, &claims, &located, &on_index, &[]).unwrap());
    assert_eq!(wanted.kind, ClaimKind::Acceptance);
    assert_eq!(checked, [SignerId::new(GATE)]);
}

#[test]
fn the_files_given_judges_where_no_publishers_policy_counts_its_gate_not_narrowed() {
    let (fakes, located, gates) = accepted();
    let seams = fakes.seams();
    let own = consumers(vec![acceptance()], true);
    let on_index = [
        gates,
        publishers(&located.index, &[provenance()], "stranger"),
    ];
    let (policy, claims) = policy_for(
        &seams,
        Some(&own),
        &trusts(&[HOST]),
        &located.index,
        &on_index,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        policy,
        PolicyOf::Given {
            revision: Some(Revision::new("4f2a9c1"))
        }
    );
    assert_eq!(
        claims,
        [acceptance()],
        "the file's gate was narrowed to the host's trusted signers"
    );
    assert_eq!(
        serde_json::to_value(&policy).unwrap(),
        json!({ "given": { "revision": "4f2a9c1" } })
    );
    assert_eq!(
        judge(&seams, &claims, &located, &on_index, &[]),
        Ok(Ok(vec![]))
    );
    assert_eq!(
        wanting(judge(&seams, &claims, &located, &on_index[1..], &[]).unwrap()).0,
        acceptance(),
        "a version the file's gate did not accept was held"
    );
}

#[test]
fn with_no_policy_file_the_given_is_acceptance_by_a_signer_the_host_trusts() {
    let (fakes, located, gates) = accepted();
    let seams = fakes.seams();
    let on_index = [gates];
    let (policy, claims) = policy_for(&seams, None, &trusts(&[GATE]), &located.index, &on_index)
        .unwrap()
        .unwrap();
    assert_eq!((policy.clone(), claims.clone()), given_by_trusted(&[GATE]));
    assert_eq!(
        serde_json::to_value(&policy).unwrap(),
        json!({ "given": { "revision": null } })
    );
    assert_eq!(
        judge(&seams, &claims, &located, &on_index, &[]),
        Ok(Ok(vec![]))
    );
}

#[test]
fn a_host_trusting_none_refuses_naming_acceptance_with_no_signer_checked() {
    let (fakes, located, gates) = accepted();
    let seams = fakes.seams();
    let on_index = [gates, publishers(&located.index, &[acceptance()], GATE)];
    let (policy, claims) = policy_for(&seams, None, &[], &located.index, &on_index)
        .unwrap()
        .unwrap();
    assert_eq!((policy, claims.clone()), given_by_trusted(&[]));
    match judge(&seams, &claims, &located, &on_index, &[]) {
        Ok(Err(Not::Refused(r))) => assert_eq!(
            r.why,
            Why::Wanting {
                version: located.artifact.meta.clone(),
                index: located.index.clone(),
                wanting: RequiredClaim {
                    kind: ClaimKind::Acceptance,
                    check: None,
                    signers: vec![],
                },
                checked: vec![],
            }
        ),
        other => panic!("a host trusting none judged {other:?}"),
    }
}

#[test]
fn two_publishers_policies_of_different_content_are_unreadable_naming_both() {
    let fakes = Fakes::new();
    let index = Digest::of(b"index");
    let first = publishers(&index, &[acceptance()], HOST);
    let second = publishers(&index, &[provenance()], GATE);
    match policy_for(
        &fakes.seams(),
        None,
        &trusts(&[GATE, HOST]),
        &index,
        &[first.clone(), second.clone()],
    ) {
        Ok(Err(Not::Unreadable(u))) => {
            assert!(u.what.contains(&first.digest().to_string()), "{}", u.what);
            assert!(u.what.contains(&second.digest().to_string()), "{}", u.what);
            assert!(u.what.contains(&index.to_string()), "{}", u.what);
        }
        other => panic!("two policies answered {other:?}"),
    }
}

#[test]
fn a_signer_that_cannot_tell_the_publishers_policy_is_the_outer_answer() {
    let fakes = Fakes::new();
    let index = Digest::of(b"index");
    fakes.signer.fault(
        tailrace_fakes::SignerOp::Verify,
        Fault::Unreadable("key service down".into()),
        Times::Once,
    );
    assert!(matches!(
        policy_for(
            &fakes.seams(),
            None,
            &trusts(&[GATE]),
            &index,
            &[publishers(&index, &[acceptance()], GATE)],
        ),
        Err(Not::Unreadable(u)) if u.said.as_deref() == Some("key service down")
    ));
}

const LANDING: &str = "landing";

/// The landing's verdict of `check` passing, by the landing's signer.
fn verdict(check: &str) -> RequiredClaim {
    RequiredClaim {
        kind: ClaimKind::Verdict,
        check: Some(Check::new(check)),
        signers: vec![named(LANDING, &[ClaimKind::Verdict], &[check])],
    }
}

/// The host's provenance of the artifact at `located`, carrying the
/// landing's verdicts of `checks` as the landing made them.
fn provenance_of(located: &Located, checks: &[&str]) -> Envelope {
    let build = BuildId::new("b7e1");
    let carried = checks
        .iter()
        .map(|check| {
            let envelope = signed(
                &Digest::of(build.as_str().as_bytes()),
                ClaimKind::Verdict,
                json!({ "check": check, "outcome": "pass" }),
                LANDING,
            );
            Carried::from_landing(
                &Verdict {
                    check: Check::new(*check),
                    build: build.clone(),
                    outcome: Outcome::Pass,
                    signer: SignerId::new(LANDING),
                    envelope,
                    found: format!("the landing's record of {check}"),
                },
                &Revision::new("p1b11s4"),
            )
        })
        .collect();
    let meta = &located.artifact.meta;
    let body = Provenance {
        output: OutputName::new(meta.name().as_str()),
        ecosystem: meta.ecosystem().clone(),
        name: meta.name().clone(),
        version: meta.version().clone(),
        outcome: Outcome::Pass,
        carried,
    };
    signed(
        &located.artifact.digest,
        ClaimKind::Provenance,
        serde_json::to_value(body).unwrap(),
        HOST,
    )
}

/// The version `pkg 1.1.0` published to a fresh world's store, and the
/// gate's acceptance of its index.
fn accepted() -> (Fakes, Located, Envelope) {
    let fakes = Fakes::new();
    let located = published(&fakes.store, "pkg", "1.1.0");
    let acceptance = signed(&located.index, ClaimKind::Acceptance, json!(null), GATE);
    (fakes, located, acceptance)
}

fn wanting(answer: Result<Vec<SignerId>, Not>) -> (RequiredClaim, Vec<SignerId>) {
    match answer {
        Err(Not::Refused(r)) => match r.why {
            Why::Wanting {
                wanting, checked, ..
            } => (wanting, checked),
            other => panic!("refused for {other:?}"),
        },
        other => panic!("judged {other:?}"),
    }
}

#[test]
fn every_required_claim_satisfied_on_the_index_or_the_artifact_holds_the_version() {
    let (fakes, located, gates) = accepted();
    let hosts = provenance_of(&located, &["test"]);
    let required = [acceptance(), provenance(), verdict("test")];
    assert_eq!(
        judge(&fakes.seams(), &required, &located, &[gates], &[hosts]),
        Ok(Ok(vec![]))
    );
}

#[test]
fn a_claim_by_a_signer_the_policy_does_not_name_does_not_count() {
    let (fakes, located, _) = accepted();
    let before = fakes.store.calls().len();
    let strangers = signed(
        &located.index,
        ClaimKind::Acceptance,
        json!(null),
        "stranger",
    );
    let answer = judge(
        &fakes.seams(),
        &[acceptance()],
        &located,
        std::slice::from_ref(&strangers),
        &[],
    );
    let (wanted, checked) = wanting(answer.unwrap());
    assert_eq!(wanted, acceptance());
    assert_eq!(checked, [SignerId::new(GATE)]);
    assert_eq!(
        fakes.store.calls().len(),
        before,
        "the judgment asked the store something"
    );
}

#[test]
fn a_claim_of_a_kind_its_signer_may_not_make_does_not_count() {
    let (fakes, located, _) = accepted();
    let by_the_gate = signed(
        &located.artifact.digest,
        ClaimKind::Provenance,
        json!({}),
        GATE,
    );
    let gate_for_provenance = RequiredClaim {
        kind: ClaimKind::Provenance,
        check: None,
        signers: vec![named(GATE, &[ClaimKind::Acceptance], &[])],
    };
    let answer = judge(
        &fakes.seams(),
        std::slice::from_ref(&gate_for_provenance),
        &located,
        &[],
        &[by_the_gate],
    );
    assert_eq!(wanting(answer.unwrap()).0, gate_for_provenance);

    let other_check = provenance_of(&located, &["lint"]);
    let answer = judge(
        &fakes.seams(),
        &[verdict("test")],
        &located,
        &[],
        &[other_check],
    );
    assert_eq!(wanting(answer.unwrap()).0, verdict("test"));
}

#[test]
fn a_claim_by_an_expired_signer_counts_and_is_reported() {
    let (fakes, located, gates) = accepted();
    fakes.signer.expire(&SignerId::new(GATE));
    assert_eq!(
        judge(&fakes.seams(), &[acceptance()], &located, &[gates], &[]),
        Ok(Ok(vec![SignerId::new(GATE)]))
    );
}

#[test]
fn of_two_claims_wanting_the_first_in_the_policys_order_is_named() {
    let (fakes, located, _) = accepted();
    let answer = judge(
        &fakes.seams(),
        &[acceptance(), provenance()],
        &located,
        &[],
        &[],
    );
    match answer {
        Ok(Err(Not::Refused(r))) => {
            assert_eq!(r.source.seam, Seam::Policy);
            assert_eq!(
                r.why,
                Why::Wanting {
                    version: located.artifact.meta.clone(),
                    index: located.index.clone(),
                    wanting: acceptance(),
                    checked: vec![SignerId::new(GATE)],
                }
            );
        }
        other => panic!("judged {other:?}"),
    }
    let hosts = provenance_of(&located, &[]);
    let answer = judge(
        &fakes.seams(),
        &[provenance(), acceptance()],
        &located,
        &[],
        &[hosts],
    );
    assert_eq!(wanting(answer.unwrap()).0, acceptance());
}

#[test]
fn a_signer_that_cannot_say_is_the_outer_answer() {
    let (fakes, located, gates) = accepted();
    fakes.signer.fault(
        tailrace_fakes::SignerOp::Verify,
        Fault::Unreadable("key service down".into()),
        Times::Once,
    );
    assert!(matches!(
        judge(&fakes.seams(), &[acceptance()], &located, &[gates], &[]),
        Err(Not::Unreadable(u)) if u.said.as_deref() == Some("key service down")
    ));
}

/// A checkout's working tree and its record, each in a temporary
/// directory, over fakes binding the ecosystem's projection, with
/// `pkg 1.1.0` published and accepted by the gate.
struct Checkout {
    fakes: Fakes,
    projection: FakeProjection,
    _dir: TempDir,
    tree: PathBuf,
    layouts: Layouts,
    pkg: Located,
}

impl Checkout {
    fn new() -> Checkout {
        let mut fakes = Fakes::new();
        let projection = fakes.projection(&eco());
        let dir = TempDir::new().unwrap();
        let tree = dir.path().join("checkout");
        fs::create_dir_all(&tree).unwrap();
        let layouts = Layouts::open(dir.path().join("git").join("tailrace").join("layouts"));
        let pkg = published(&fakes.store, "pkg", "1.1.0");
        let gates = signed(&pkg.index, ClaimKind::Acceptance, json!(null), GATE);
        fakes.store.attach(&pkg.index, &gates).unwrap();
        Checkout {
            fakes,
            projection,
            _dir: dir,
            tree,
            layouts,
            pkg,
        }
    }

    /// The layout of `pins` asked under the consumer's own policy
    /// requiring `claims`, at `when`.
    fn ask(&self, pins: &[Pinned], claims: Vec<RequiredClaim>, when: u64) -> RenderAsk {
        RenderAsk {
            ecosystem: eco(),
            pins: pins.to_vec(),
            own: Some(consumers(claims, false)),
            trusted: vec![],
            tree: self.tree.clone(),
            when,
        }
    }

    fn at(&self) -> String {
        place(&self.tree, &eco()).display().to_string()
    }

    /// The artifacts each render of the projection was given, by
    /// version, one list per render.
    fn renders(&self) -> Vec<Vec<String>> {
        self.projection
            .renders()
            .into_iter()
            .map(|(at, artifacts)| {
                assert_eq!(at, self.at(), "a render at another place");
                artifacts
                    .iter()
                    .map(|a| format!("{} {}", a.meta.name(), a.meta.version()))
                    .collect()
            })
            .collect()
    }
}

fn own() -> Option<PolicyOf> {
    Some(PolicyOf::Own {
        revision: Revision::new("4f2a9c1"),
    })
}

#[test]
fn a_layout_holds_what_the_policy_admits_at_its_place_ignored_by_git() {
    let checkout = Checkout::new();
    let pins = [pin("pkg", "1.1.0")];
    let refreshed = render(
        &checkout.fakes.seams(),
        &checkout.layouts,
        &checkout.ask(&pins, vec![acceptance()], 1_767_225_600),
    )
    .unwrap();
    assert_eq!(
        refreshed.at,
        checkout
            .tree
            .join(".tailrace/layouts/eco")
            .display()
            .to_string()
    );
    assert_eq!(refreshed.rendered_at, 1_767_225_600);
    assert_eq!(
        refreshed.policy,
        Under::Own {
            revision: Revision::new("4f2a9c1")
        }
    );
    assert_eq!(
        refreshed.pins,
        [PinAnswer {
            pin: pin("pkg", "1.1.0"),
            policy: own(),
            answer: Ok(HeldPin {
                index: checkout.pkg.index.clone(),
                artifact: checkout.pkg.artifact.digest.clone(),
                expired: vec![],
            }),
        }]
    );
    assert_eq!(checkout.renders(), [vec!["pkg 1.1.0".to_string()]]);
    assert_eq!(
        fs::read_to_string(checkout.tree.join(".tailrace/.gitignore")).unwrap(),
        "*\n"
    );
}

#[test]
fn a_third_party_pin_is_absent_and_the_rest_is_rendered() {
    let checkout = Checkout::new();
    let pins = [pin("left-pad", "1.3.0"), pin("pkg", "1.1.0")];
    let refreshed = render(
        &checkout.fakes.seams(),
        &checkout.layouts,
        &checkout.ask(&pins, vec![acceptance()], 1),
    )
    .unwrap();
    match &refreshed.pins[0].answer {
        Err(Not::Absent(a)) => assert_eq!(a.what, "name artifact/eco/left-pad@1.3.0"),
        other => panic!("the third-party pin answered {other:?}"),
    }
    assert_eq!(refreshed.pins[0].policy, None);
    assert!(refreshed.pins[1].answer.is_ok(), "{refreshed:?}");
    assert_eq!(checkout.renders(), [vec!["pkg 1.1.0".to_string()]]);
    let asked: Vec<String> = checkout
        .fakes
        .store
        .calls()
        .into_iter()
        .filter(|c| c.op == StoreOp::Resolve)
        .map(|c| c.given["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        asked,
        ["artifact/eco/left-pad@1.3.0", "artifact/eco/pkg@1.1.0"]
    );
}

#[test]
fn a_version_refused_now_leaves_the_layout() {
    let checkout = Checkout::new();
    let seams = checkout.fakes.seams();
    let pins = [pin("pkg", "1.1.0")];
    render(
        &seams,
        &checkout.layouts,
        &checkout.ask(&pins, vec![acceptance()], 1),
    )
    .unwrap();
    let again = render(
        &seams,
        &checkout.layouts,
        &checkout.ask(&pins, vec![acceptance(), provenance()], 2),
    )
    .unwrap();
    match &again.pins[0].answer {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::Wanting {
                version: checkout.pkg.artifact.meta.clone(),
                index: checkout.pkg.index.clone(),
                wanting: provenance(),
                checked: vec![SignerId::new(HOST)],
            }
        ),
        other => panic!("the version answered {other:?}"),
    }
    assert_eq!(again.pins[0].policy, own());
    assert_eq!(
        checkout.renders(),
        [vec!["pkg 1.1.0".to_string()], vec![]],
        "the refused version stayed in the layout"
    );
}

#[test]
fn a_store_that_cannot_be_reached_leaves_the_layout_as_it_was() {
    let checkout = Checkout::new();
    let seams = checkout.fakes.seams();
    let pins = [pin("pkg", "1.1.0")];
    let first = refresh(
        &seams,
        &checkout.layouts,
        &checkout.ask(&pins, vec![acceptance()], 1_767_225_600),
    );
    assert!(matches!(first, Ok(Refresh::Refreshed(_))), "{first:?}");
    let record = fs::read(checkout.layouts.path(&eco())).unwrap();

    checkout.fakes.store.fault(
        StoreOp::Resolve,
        Fault::Unreadable("connection refused".into()),
        Times::UntilHealed,
    );
    let second = refresh(
        &seams,
        &checkout.layouts,
        &checkout.ask(&pins, vec![acceptance()], 1_767_312_000),
    )
    .unwrap();
    let Refresh::Kept(Kept {
        at,
        holds,
        rendered_at,
        policy,
        because,
    }) = second
    else {
        panic!("an unreachable store refreshed the layout: {second:?}")
    };
    assert_eq!(at, checkout.at());
    assert_eq!(
        holds,
        [HeldEntry {
            name: EcosystemName::new("pkg"),
            version: Version::new("1.1.0"),
            index: checkout.pkg.index.clone(),
            artifact: checkout.pkg.artifact.digest.clone(),
            policy: own().unwrap(),
        }]
    );
    assert_eq!(rendered_at, Some(1_767_225_600));
    assert_eq!(
        policy,
        Some(Under::Own {
            revision: Revision::new("4f2a9c1")
        })
    );
    assert!(
        matches!(&because, Not::Unreadable(u) if u.said.as_deref() == Some("connection refused")),
        "{because:?}"
    );
    assert_eq!(checkout.renders().len(), 1, "the projection rendered again");
    assert_eq!(
        fs::read(checkout.layouts.path(&eco())).unwrap(),
        record,
        "the record was written again"
    );
}

#[test]
fn a_layout_never_rendered_and_unreachable_holds_nothing() {
    let checkout = Checkout::new();
    checkout.fakes.store.fault(
        StoreOp::Resolve,
        Fault::Unreadable("connection refused".into()),
        Times::UntilHealed,
    );
    let answer = refresh(
        &checkout.fakes.seams(),
        &checkout.layouts,
        &checkout.ask(&[pin("pkg", "1.1.0")], vec![acceptance()], 1),
    )
    .unwrap();
    let Refresh::Kept(kept) = answer else {
        panic!("an unreachable store refreshed the layout: {answer:?}")
    };
    assert_eq!(
        (kept.holds, kept.rendered_at, kept.policy),
        (vec![], None, None)
    );
    assert!(matches!(kept.because, Not::Unreadable(_)));
    assert!(
        checkout
            .projection
            .calls()
            .iter()
            .all(|c| c.op != ProjectionOp::Render),
        "the projection was asked to render"
    );
    assert!(!checkout.layouts.path(&eco()).exists());
    assert!(
        !checkout.tree.join(".tailrace").exists(),
        "a write was made"
    );
}

#[test]
fn a_render_that_fails_leaves_the_record_as_it_was() {
    let checkout = Checkout::new();
    let seams = checkout.fakes.seams();
    let pins = [pin("pkg", "1.1.0")];
    refresh(
        &seams,
        &checkout.layouts,
        &checkout.ask(&pins, vec![acceptance()], 1),
    )
    .unwrap();
    checkout.projection.fault(
        ProjectionOp::Render,
        Fault::Unreadable("disk full".into()),
        Times::Once,
    );
    let answer = refresh(&seams, &checkout.layouts, &checkout.ask(&pins, vec![], 2)).unwrap();
    let Refresh::Kept(kept) = answer else {
        panic!("a failed render refreshed the layout: {answer:?}")
    };
    assert_eq!(kept.rendered_at, Some(1));
    assert!(matches!(&kept.because, Not::Unreadable(u) if u.said.as_deref() == Some("disk full")));
}

/// `pkg 1.1.0`, which the gate accepted, `lib 2.0.0`, which it did not,
/// and `left-pad 1.3.0`, which the store does not name, rendered under
/// acceptance at `when`; the record read back as JSON.
fn rendered_with_one_of_each(checkout: &Checkout, when: u64) -> serde_json::Value {
    let pins = [
        pin("pkg", "1.1.0"),
        pin("lib", "2.0.0"),
        pin("left-pad", "1.3.0"),
    ];
    render(
        &checkout.fakes.seams(),
        &checkout.layouts,
        &checkout.ask(&pins, vec![acceptance()], when),
    )
    .unwrap();
    let text = fs::read_to_string(checkout.layouts.path(&eco())).unwrap();
    serde_json::from_str(&text).unwrap()
}

#[test]
fn a_layouts_record_is_json_naming_every_version_held_and_refused() {
    let checkout = Checkout::new();
    let lib = published(&checkout.fakes.store, "lib", "2.0.0");
    let record = rendered_with_one_of_each(&checkout, 1_767_225_600);
    assert_eq!(
        checkout.layouts.path(&eco()),
        checkout.layouts.dir().join("eco.json")
    );
    let own = json!({ "own": { "revision": "4f2a9c1" } });
    assert_eq!(
        record,
        json!({
            "ecosystem": "eco",
            "at": checkout.at(),
            "rendered_at": 1_767_225_600,
            "policy": own,
            "held": [{
                "name": "pkg",
                "version": "1.1.0",
                "index": checkout.pkg.index,
                "artifact": checkout.pkg.artifact.digest,
                "policy": own,
            }],
            "refused": [{
                "name": "lib",
                "version": "2.0.0",
                "index": lib.index,
                "wanting": acceptance(),
                "policy": own,
            }],
        })
    );
    let entries: Vec<_> = fs::read_dir(checkout.layouts.dir())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(entries, ["eco.json"], "the record left a file beside it");
}

#[test]
fn a_deleted_record_is_written_again_the_same_but_for_when() {
    let checkout = Checkout::new();
    published(&checkout.fakes.store, "lib", "2.0.0");
    let mut first = rendered_with_one_of_each(&checkout, 1);
    fs::remove_file(checkout.layouts.path(&eco())).unwrap();
    let mut again = rendered_with_one_of_each(&checkout, 2);
    assert_eq!(
        (first["rendered_at"].take(), again["rendered_at"].take()),
        (json!(1), json!(2))
    );
    assert_eq!(again, first);
    let renders = checkout.renders();
    assert_eq!(renders[0], renders[1], "the layout holds other versions");
}

#[test]
fn a_record_that_is_no_record_is_unreadable_naming_the_file_with_the_parsers_words() {
    let checkout = Checkout::new();
    let path = checkout.layouts.path(&eco());
    fs::create_dir_all(checkout.layouts.dir()).unwrap();
    fs::write(&path, "{ not json").unwrap();
    let words = serde_json::from_str::<serde_json::Value>("{ not json")
        .unwrap_err()
        .to_string();
    match checkout.layouts.read(&eco()) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source.seam, Seam::Host);
            assert_eq!(u.what, path.display().to_string());
            assert_eq!(u.said, Some(words));
        }
        other => panic!("a broken record answered {other:?}"),
    }
    match Layouts::open(checkout.layouts.dir().join("none")).read(&eco()) {
        Err(Not::Absent(a)) => assert!(a.what.ends_with("eco.json"), "{a:?}"),
        other => panic!("no record answered {other:?}"),
    }
}

#[test]
fn the_record_is_kept_under_the_checkouts_own_git_directory() {
    let fixture = common::Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit("chore: start", &[("README", "a repository")]);
    let history = repo.history();
    assert_eq!(
        Layouts::at(&history).dir(),
        history.git_dir().join("tailrace").join("layouts")
    );
}

const LANDED: &str = "4f2a9c1";

impl Checkout {
    /// What sync asks of this checkout for the revision landed on `main`,
    /// at `when`, on a host trusting `trusted`.
    fn checkout_ask(&self, when: u64, trusted: Vec<NamedSigner>) -> CheckoutAsk {
        CheckoutAsk {
            branch: Branch::new("main"),
            revision: Revision::new(LANDED),
            tree: self.tree.clone(),
            when,
            trusted,
        }
    }

    /// The revision landed, its policy for layouts requiring `claims`,
    /// and the lockfile pinning `pins` where given.
    fn landed(&self, claims: Vec<RequiredClaim>, pins: Option<&[Pinned]>) {
        let revision = Revision::new(LANDED);
        self.fakes.landing.land(&Branch::new("main"), &revision);
        self.fakes.policy.require_at(
            &revision,
            Audience::Layout,
            Required::Layout {
                claims,
                given: false,
            },
        );
        if let Some(pins) = pins {
            self.projection
                .lockfile(&self.tree.display().to_string(), pins);
        }
    }
}

/// A passing verdict of `test` on `build` at the landed revision, as the
/// landing signs it.
fn passing(build: &str) -> Verdict {
    let claim = signed(
        &Digest::of(build.as_bytes()),
        ClaimKind::Verdict,
        json!({ "check": "test", "outcome": "pass" }),
        LANDING,
    );
    Verdict {
        check: Check::new("test"),
        build: BuildId::new(build),
        outcome: Outcome::Pass,
        signer: SignerId::new(LANDING),
        envelope: claim,
        found: format!("the fake landing's record of test on {build}"),
    }
}

#[test]
fn a_checkout_is_synced_its_cache_and_then_each_layout_under_the_policy_at_the_landed_revision() {
    let checkout = Checkout::new();
    checkout.landed(
        vec![acceptance()],
        Some(&[pin("pkg", "1.1.0"), pin("left-pad", "1.3.0")]),
    );
    checkout
        .fakes
        .landing
        .verdict(&Revision::new(LANDED), passing("b7e1"));
    let since = checkout.fakes.store.calls().len();
    let synced = sync_checkout(
        &checkout.fakes.seams(),
        &checkout.layouts,
        &checkout.checkout_ask(1_767_225_600, vec![]),
    )
    .unwrap();

    assert_eq!(synced.entries.len(), 1, "{synced:?}");
    let [(ecosystem, Ok(Refresh::Refreshed(refreshed)))] = synced.layouts.as_slice() else {
        panic!("the layouts answered {:?}", synced.layouts)
    };
    assert_eq!(*ecosystem, eco());
    assert_eq!(
        refreshed.policy,
        Under::Own {
            revision: Revision::new(LANDED)
        }
    );
    assert!(refreshed.pins[0].answer.is_ok(), "{refreshed:?}");
    assert!(matches!(refreshed.pins[1].answer, Err(Not::Absent(_))));

    let [asked] = <[_; 1]>::try_from(checkout.fakes.policy.calls()).unwrap();
    assert_eq!(asked.given["revision"], LANDED);
    assert_eq!(asked.given["audience"], "layout");
    let resolved: Vec<String> = checkout
        .fakes
        .store
        .calls()
        .split_off(since)
        .into_iter()
        .filter(|c| c.op == StoreOp::Resolve)
        .map(|c| c.given["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        resolved[0],
        format!("{CACHE}b7e1"),
        "the cache was not filled first"
    );
    assert_eq!(
        resolved[1..],
        ["artifact/eco/pkg@1.1.0", "artifact/eco/left-pad@1.3.0"],
        "the layouts resolved a name beginning {CACHE}"
    );

    let value = serde_json::to_value(&synced).unwrap();
    let keys: Vec<&String> = value.as_object().unwrap().keys().collect();
    assert_eq!(keys, ["branch", "entries", "layouts", "revision"]);
    assert_eq!(value["revision"], LANDED);
    assert_eq!(value["entries"][0]["build"], "b7e1");
    let layout = &value["layouts"][0];
    assert_eq!(
        (&layout["ecosystem"], &layout["answer"]),
        (&json!("eco"), &json!("ok"))
    );
    let pins = &layout["value"]["refreshed"]["pins"];
    assert_eq!(
        (&pins[0]["name"], &pins[0]["answer"]),
        (&json!("pkg"), &json!("ok"))
    );
    assert_eq!(
        (&pins[1]["name"], &pins[1]["answer"]),
        (&json!("left-pad"), &json!("absent"))
    );
}

#[test]
fn with_nothing_pinned_the_policy_is_not_asked() {
    let checkout = Checkout::new();
    checkout.landed(vec![acceptance()], None);
    let synced = sync_checkout(
        &checkout.fakes.seams(),
        &checkout.layouts,
        &checkout.checkout_ask(1, vec![]),
    )
    .unwrap();
    assert_eq!(
        synced.layouts,
        [],
        "a projection with no lockfile has a layout"
    );
    assert_eq!(checkout.fakes.policy.calls(), [], "the policy was asked");
    assert!(
        !checkout.layouts.path(&eco()).exists(),
        "a layout was recorded"
    );
    assert!(
        !checkout.tree.join(".tailrace").exists(),
        "a layout was made"
    );

    checkout
        .projection
        .lockfile(&checkout.tree.display().to_string(), &[]);
    let synced = sync_checkout(
        &checkout.fakes.seams(),
        &checkout.layouts,
        &checkout.checkout_ask(2, vec![]),
    )
    .unwrap();
    let [(_, Ok(Refresh::Refreshed(refreshed)))] = synced.layouts.as_slice() else {
        panic!("an empty lockfile answered {:?}", synced.layouts)
    };
    assert_eq!(refreshed.pins, []);
    assert_eq!(checkout.fakes.policy.calls(), [], "the policy was asked");
}

#[test]
fn a_policy_that_cannot_be_read_leaves_every_layout_as_it_was() {
    let checkout = Checkout::new();
    checkout.landed(vec![acceptance()], Some(&[pin("pkg", "1.1.0")]));
    let seams = checkout.fakes.seams();
    sync_checkout(&seams, &checkout.layouts, &checkout.checkout_ask(1, vec![])).unwrap();
    checkout.fakes.policy.fault(
        PolicyOp::PolicyFor,
        Fault::Unreadable("the policy file does not parse".into()),
        Times::Once,
    );
    let synced =
        sync_checkout(&seams, &checkout.layouts, &checkout.checkout_ask(2, vec![])).unwrap();
    let [(_, Ok(Refresh::Kept(kept)))] = synced.layouts.as_slice() else {
        panic!("an unread policy answered {:?}", synced.layouts)
    };
    assert_eq!(kept.rendered_at, Some(1));
    assert_eq!(kept.holds.len(), 1);
    assert!(
        matches!(&kept.because, Not::Unreadable(u) if u.said.as_deref() == Some("the policy file does not parse")),
        "{kept:?}"
    );
    assert_eq!(checkout.renders().len(), 1, "the layout was rendered again");
}

#[test]
fn a_revision_not_landed_is_refused_before_a_lockfile_is_read() {
    let checkout = Checkout::new();
    checkout
        .projection
        .lockfile(&checkout.tree.display().to_string(), &[pin("pkg", "1.1.0")]);
    let answer = sync_checkout(
        &checkout.fakes.seams(),
        &checkout.layouts,
        &checkout.checkout_ask(1, vec![]),
    );
    assert!(matches!(answer, Err(Not::Refused(_))), "{answer:?}");
    assert!(
        checkout
            .projection
            .calls()
            .iter()
            .all(|c| c.op != ProjectionOp::Pinned),
        "a lockfile was read"
    );
}
