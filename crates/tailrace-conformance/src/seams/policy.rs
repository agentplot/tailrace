//! The policy's group: 82 and 83, and 116 as a policy keeps it (B.5).

use tailrace_seams::{
    Audience, ChannelName, Check, ClaimKind, NamedSigner, Not, Policy, Required, RequiredClaim,
    Revision, Seam, SignerId,
};

use crate::seams::{ok, unreadable};
use crate::{Rig, Scenario};

pub const GROUP: &str = "policy";

/// A policy, made new for each scenario.
pub trait PolicyRig: Rig {
    fn policy(&self) -> &dyn Policy;

    /// A landed revision at which the policy declares what each audience
    /// of `policies` requires, and nothing for any other audience.
    fn at(&self, policies: &[(Audience, Required)]) -> Revision;

    /// The policy made unreachable, every operation answering unreadable,
    /// and the words it answers with.
    fn unreachable(&self) -> String;
}

pub fn scenarios() -> Vec<Scenario<dyn PolicyRig>> {
    vec![
        Scenario {
            name: "the claims an audience requires",
            plays: None,
            cites: &[82, 116],
            run: the_claims_an_audience_requires,
        },
        Scenario {
            name: "acceptance names no channel",
            plays: None,
            cites: &[83],
            run: acceptance_names_no_channel,
        },
        Scenario {
            name: "no policy for an audience",
            plays: None,
            cites: &[82],
            run: no_policy_for_an_audience,
        },
        Scenario {
            name: "a policy that cannot be reached",
            plays: None,
            cites: &[58, 59],
            run: a_policy_that_cannot_be_reached,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}

/// `id` named for `kinds`, and for a verdict the `checks`, by the names
/// the landing gives them (116).
fn named(id: &str, kinds: &[ClaimKind], checks: &[&str]) -> NamedSigner {
    NamedSigner {
        id: SignerId::new(id),
        verified_by: format!("{id}'s key"),
        kinds: kinds.to_vec(),
        checks: checks.iter().map(|c| Check::new(*c)).collect(),
    }
}

/// A provenance from the host, and a verdict of `test` from the landing.
fn claims() -> Vec<RequiredClaim> {
    vec![
        RequiredClaim {
            kind: ClaimKind::Provenance,
            check: None,
            signers: vec![named("host", &[ClaimKind::Provenance], &[])],
        },
        RequiredClaim {
            kind: ClaimKind::Verdict,
            check: Some(Check::new("test")),
            signers: vec![named("landing", &[ClaimKind::Verdict], &["test"])],
        },
    ]
}

fn the_claims_an_audience_requires(rig: &dyn PolicyRig) {
    let required = Required::Acceptance { claims: claims() };
    let at = rig.at(&[(Audience::Acceptance, required.clone())]);
    let read = ok(
        rig.policy().policy_for(&at, &Audience::Acceptance),
        "policy-for acceptance",
    );
    assert_eq!(
        read, required,
        "the policy answered other claims, signers or reaches than it declares"
    );
    let verdict = &read.claims()[1];
    assert_eq!(
        (
            verdict.signers[0].kinds.as_slice(),
            verdict.signers[0].checks.as_slice()
        ),
        (
            [ClaimKind::Verdict].as_slice(),
            [Check::new("test")].as_slice()
        ),
        "the verdict's signer is bound to other kinds or checks"
    );
}

fn acceptance_names_no_channel(rig: &dyn PolicyRig) {
    let (stable, beta) = (ChannelName::new("stable"), ChannelName::new("beta"));
    let at = rig.at(&[
        (
            Audience::Acceptance,
            Required::Acceptance { claims: claims() },
        ),
        (
            Audience::Channel(stable.clone()),
            Required::Channel {
                claims: claims(),
                bound_to: Some(beta.clone()),
            },
        ),
    ]);
    let policy = rig.policy();
    let acceptance = ok(
        policy.policy_for(&at, &Audience::Acceptance),
        "policy-for acceptance",
    );
    assert!(
        matches!(acceptance, Required::Acceptance { .. }),
        "the policy for acceptance answered {acceptance:?}"
    );
    assert_eq!(acceptance.bound_to(), None, "acceptance names a channel");
    let bound = ok(
        policy.policy_for(&at, &Audience::Channel(stable)),
        "policy-for a bound channel",
    );
    assert_eq!(
        bound.bound_to(),
        Some(&beta),
        "the bound channel's policy answered {bound:?}"
    );
}

fn no_policy_for_an_audience(rig: &dyn PolicyRig) {
    let at = rig.at(&[(
        Audience::Acceptance,
        Required::Acceptance { claims: claims() },
    )]);
    match rig.policy().policy_for(&at, &Audience::Layout) {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source.seam, Seam::Policy, "{a:?}");
            assert!(
                a.what.contains("layout"),
                "the absence names no audience: {a:?}"
            );
            assert!(
                a.what.contains(at.as_str()),
                "the absence names no revision: {a:?}"
            );
        }
        other => panic!("policy-for an audience with no policy answered {other:?}"),
    }
}

fn a_policy_that_cannot_be_reached(rig: &dyn PolicyRig) {
    let at = rig.at(&[(
        Audience::Acceptance,
        Required::Acceptance { claims: claims() },
    )]);
    let words = rig.unreachable();
    unreadable(
        rig.policy().policy_for(&at, &Audience::Acceptance),
        Seam::Policy,
        &words,
        "policy-for",
    );
}
