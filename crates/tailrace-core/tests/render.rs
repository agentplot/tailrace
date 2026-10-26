//! Consumer layouts rendered under a policy, over the fakes (design D2
//! to D7).

use serde_json::json;
use tailrace_core::{
    artifact_name, judge, layout_policy_body, locate, own_policy, policy_for, Carried, Landed,
    Located, Own, PolicyOf, Provenance, ARTIFACT,
};
use tailrace_fakes::{Fakes, Fault, PolicyOp, StoreOp, Times};
use tailrace_seams::{
    Artifact, ArtifactMeta, Audience, Branch, BuildId, Check, Claim, ClaimKind, ComponentName,
    Digest, Ecosystem, EcosystemName, Envelope, IndexMeta, Manifest, Name, NamedSigner, Not,
    Outcome, OutputName, Pinned, Required, RequiredClaim, Revision, Seam, Signer, SignerId, Store,
    Verdict, Version, Why,
};

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
        },
    );
    fakes.policy.require_at(
        at.revision(),
        Audience::Layout,
        Required::Layout {
            claims: vec![acceptance(), provenance()],
        },
    );
    assert_eq!(
        own_policy(&fakes.policy, &at),
        Ok(Some(Own {
            revision: Revision::new("4f2a9c1"),
            claims: vec![acceptance(), provenance()],
        }))
    );
    let [asked] = <[_; 1]>::try_from(fakes.policy.calls()).unwrap();
    assert_eq!(asked.op, PolicyOp::PolicyFor);
    assert_eq!(asked.given["revision"], "4f2a9c1");
    assert_eq!(asked.given["audience"], "layout");
}

#[test]
fn a_consumer_declaring_no_policy_for_layouts_has_none_and_one_unread_answers_its_words() {
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

#[test]
fn the_consumers_own_policy_judges_whatever_the_index_carries() {
    let index = Digest::of(b"index");
    let own = Own {
        revision: Revision::new("4f2a9c1"),
        claims: vec![acceptance()],
    };
    let on_index = [publishers(&index, &[provenance()], HOST)];
    assert_eq!(
        policy_for(Some(&own), &meta(ECO, "pkg", "1.1.0"), &index, &on_index),
        Ok((
            PolicyOf::Own {
                revision: Revision::new("4f2a9c1")
            },
            vec![acceptance()]
        ))
    );
}

#[test]
fn with_no_policy_of_its_own_a_version_is_judged_under_its_publishers() {
    let index = Digest::of(b"index");
    let on_index = [
        signed(&index, ClaimKind::Acceptance, json!(null), GATE),
        publishers(&index, &[acceptance(), provenance()], HOST),
        publishers(&index, &[acceptance(), provenance()], GATE),
        publishers(&Digest::of(b"another index"), &[provenance()], HOST),
    ];
    let (policy, claims) = policy_for(None, &meta(ECO, "pkg", "1.1.0"), &index, &on_index).unwrap();
    assert_eq!(
        policy,
        PolicyOf::Publisher {
            revision: Revision::new("p1b11s4")
        }
    );
    assert_eq!(claims, [acceptance(), provenance()]);
    assert_eq!(
        serde_json::to_value(&policy).unwrap(),
        json!({ "publisher": { "revision": "p1b11s4" } })
    );
}

#[test]
fn no_policy_anywhere_is_refused_naming_the_publishers_layout_policy() {
    let index = Digest::of(b"index");
    let version = meta(ECO, "pkg", "1.1.0");
    let on_index = [signed(&index, ClaimKind::Acceptance, json!(null), GATE)];
    match policy_for(None, &version, &index, &on_index) {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::Wanting {
                version,
                index,
                wanting: RequiredClaim {
                    kind: ClaimKind::LayoutPolicy,
                    check: None,
                    signers: vec![],
                },
                checked: vec![],
            }
        ),
        other => panic!("no policy anywhere answered {other:?}"),
    }
}

#[test]
fn two_publishers_policies_of_different_content_are_unreadable_naming_both() {
    let index = Digest::of(b"index");
    let first = publishers(&index, &[acceptance()], HOST);
    let second = publishers(&index, &[provenance()], HOST);
    match policy_for(
        None,
        &meta(ECO, "pkg", "1.1.0"),
        &index,
        &[first.clone(), second.clone()],
    ) {
        Err(Not::Unreadable(u)) => {
            assert!(u.what.contains(&first.digest().to_string()), "{}", u.what);
            assert!(u.what.contains(&second.digest().to_string()), "{}", u.what);
            assert!(u.what.contains(&index.to_string()), "{}", u.what);
        }
        other => panic!("two policies answered {other:?}"),
    }
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
