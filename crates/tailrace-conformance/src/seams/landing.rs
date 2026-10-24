//! The landing's group: 77 and 78 (B.3).

use serde_json::json;
use tailrace_seams::{
    Branch, BuildId, Check, Claim, ClaimKind, Digest, Envelope, Landing, Outcome, Revision, Seam,
    Signature, SignerId, Verdict, IN_TOTO,
};

use crate::seams::unreadable;
use crate::{ok, Rig, Scenario};

pub const GROUP: &str = "landing";

/// A landing, made new for each scenario.
pub trait LandingRig: Rig {
    fn landing(&self) -> &dyn Landing;

    /// The branch the landing lands on.
    fn branch(&self) -> Branch;

    /// A new revision the branch has landed, with `verdicts` recorded for
    /// it as given.
    fn landed(&self, verdicts: &[Verdict]) -> Revision;

    /// A revision the branch has not landed.
    fn not_landed(&self) -> Revision;

    /// The landing made unreachable, every operation answering
    /// unreadable, and the words it answers with.
    fn unreachable(&self) -> String;
}

pub fn scenarios() -> Vec<Scenario<dyn LandingRig>> {
    vec![
        Scenario {
            name: "landed and not landed",
            plays: None,
            cites: &[77],
            run: landed_and_not_landed,
        },
        Scenario {
            name: "verdicts as the landing recorded them",
            plays: None,
            cites: &[78],
            run: verdicts_as_the_landing_recorded_them,
        },
        Scenario {
            name: "a landing that cannot be reached",
            plays: None,
            cites: &[58, 59, 78],
            run: a_landing_that_cannot_be_reached,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}

/// A verdict of `check` on `build`, signed as its signer signs it.
fn verdict(check: &str, build: &str, outcome: Outcome, signer: &str) -> Verdict {
    let build = BuildId::new(build);
    let claim = Claim {
        subject: Digest::of(build.as_str().as_bytes()),
        kind: ClaimKind::Verdict,
        revision: Revision::new("4f2a9c1"),
        build: Some(build.clone()),
        body: json!({ "check": check, "outcome": outcome }),
    };
    let payload = claim.payload();
    Verdict {
        check: Check::new(check),
        build,
        outcome,
        signer: SignerId::new(signer),
        envelope: Envelope {
            payload_type: IN_TOTO.into(),
            signature: Signature {
                signer: SignerId::new(signer),
                sig: Digest::of(&payload).as_str().as_bytes().to_vec(),
            },
            payload,
        },
    }
}

fn landed_and_not_landed(rig: &dyn LandingRig) {
    let landing = rig.landing();
    let branch = rig.branch();
    let (landed, not) = (rig.landed(&[]), rig.not_landed());
    assert!(
        ok(
            landing.landed(&branch, &landed),
            "landed of a landed revision"
        ),
        "{branch} has not landed {landed}, which it landed"
    );
    assert!(
        !ok(
            landing.landed(&branch, &not),
            "landed of a revision not landed"
        ),
        "{branch} has landed {not}, which it did not"
    );
}

fn verdicts_as_the_landing_recorded_them(rig: &dyn LandingRig) {
    let landing = rig.landing();
    let recorded = [
        verdict("test", "site@4f2a9c1", Outcome::Pass, "landing"),
        verdict("lint", "site@4f2a9c1", Outcome::Fail, "landing"),
    ];
    let with = rig.landed(&recorded);
    let without = rig.landed(&[]);
    let read = ok(landing.verdicts_of(&with), "verdicts-of");
    assert_eq!(read, recorded, "the verdicts read are not those recorded");
    for verdict in &read {
        assert!(
            !verdict.check.as_str().is_empty()
                && !verdict.build.as_str().is_empty()
                && !verdict.signer.as_str().is_empty(),
            "a verdict names no check, build or signer: {verdict:?}"
        );
    }
    assert_eq!(
        ok(
            landing.verdicts_of(&without),
            "verdicts-of a revision with none"
        ),
        [],
        "a revision with no verdict answered some"
    );
}

fn a_landing_that_cannot_be_reached(rig: &dyn LandingRig) {
    let branch = rig.branch();
    let revision = rig.landed(&[verdict("test", "site@4f2a9c1", Outcome::Pass, "landing")]);
    let words = rig.unreachable();
    let landing = rig.landing();
    unreadable(
        landing.verdicts_of(&revision),
        Seam::Landing,
        &words,
        "verdicts-of",
    );
    unreadable(
        landing.landed(&branch, &revision),
        Seam::Landing,
        &words,
        "landed",
    );
}
