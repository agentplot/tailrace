//! Spec landed-policy: Nothing is read at a revision the branch has not
//! landed; The policy for an audience is asked only at a landed revision.

use tailrace_core::{policy_at, Landed};
use tailrace_fakes::{FakeLanding, FakePolicy, Fault, LandingOp, PolicyOp, Times};
use tailrace_seams::{
    Audience, Branch, Check, ClaimKind, Not, Required, RequiredClaim, Revision, Seam, Why,
};

fn main_at(rev: &str) -> (Branch, Revision) {
    (Branch::new("main"), Revision::new(rev))
}

fn acceptance() -> Required {
    Required::Acceptance {
        claims: vec![RequiredClaim {
            kind: ClaimKind::Verdict,
            check: Some(Check::new("test")),
            signers: vec![],
        }],
    }
}

#[test]
fn a_revision_the_branch_has_not_landed_is_refused_and_nothing_is_read() {
    let (landing, policy) = (FakeLanding::new(), FakePolicy::new());
    policy.require(Audience::Acceptance, acceptance());
    let (main, rev) = main_at("4f2a9c1");
    landing.land(&main, &Revision::new("77b0e3d"));

    let answer = Landed::check(&landing, &main, &rev)
        .and_then(|landed| policy_at(&policy, &landed, &Audience::Acceptance));

    let Err(Not::Refused(refusal)) = answer else {
        panic!("not refused: {answer:?}")
    };
    assert_eq!(
        refusal.why,
        Why::NotLanded {
            branch: main.clone(),
            revision: rev.clone()
        }
    );
    assert_eq!(refusal.source.seam, Seam::Landing);
    assert!(Not::Refused(refusal)
        .to_string()
        .contains("main has not landed 4f2a9c1"));
    assert!(policy.calls().is_empty(), "the policy was asked");
}

#[test]
fn a_landing_that_cannot_be_read_answers_its_own_words() {
    let landing = FakeLanding::new();
    landing.fault(
        LandingOp::Landed,
        Fault::Unreadable("connection refused".into()),
        Times::Once,
    );
    let (main, rev) = main_at("4f2a9c1");
    let Err(Not::Unreadable(unreadable)) = Landed::check(&landing, &main, &rev) else {
        panic!("not unreadable")
    };
    assert_eq!(unreadable.said.as_deref(), Some("connection refused"));
    assert_eq!(unreadable.source.seam, Seam::Landing);
    assert_eq!(unreadable.source.profile, "fake");
}

#[test]
fn the_policy_is_asked_at_the_landed_revision_and_answered_as_given() {
    let (landing, policy) = (FakeLanding::new(), FakePolicy::new());
    let (main, rev) = main_at("4f2a9c1");
    landing.land(&main, &rev);
    policy.require_at(&rev, Audience::Acceptance, acceptance());

    let landed = Landed::check(&landing, &main, &rev).unwrap();
    assert_eq!((landed.branch(), landed.revision()), (&main, &rev));
    assert_eq!(landing.calls().len(), 1);
    assert!(
        policy.calls().is_empty(),
        "the policy was asked before the landing"
    );

    assert_eq!(
        policy_at(&policy, &landed, &Audience::Acceptance),
        Ok(acceptance())
    );
    let calls = policy.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].op, PolicyOp::PolicyFor);
    assert_eq!(calls[0].given["revision"], "4f2a9c1");
    assert_eq!(calls[0].given["audience"], "acceptance");
}

#[test]
fn an_absent_policy_is_answered_absent() {
    let (landing, policy) = (FakeLanding::new(), FakePolicy::new());
    let (main, rev) = main_at("4f2a9c1");
    landing.land(&main, &rev);
    let landed = Landed::check(&landing, &main, &rev).unwrap();
    assert!(matches!(
        policy_at(&policy, &landed, &Audience::Layout),
        Err(Not::Absent(_))
    ));
}
