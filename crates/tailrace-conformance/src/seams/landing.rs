//! The landing's group: 77 and 78 (B.3). The rig stages what a landing
//! records of each verdict, and the landing makes the rest: the signed
//! claim is the landing's to make and tailrace's only to carry, so the
//! suite makes none (12, rule core.8; design D3a).

use tailrace_seams::{
    Branch, BuildId, Check, Landing, Outcome, Revision, Seam, SignerId, Verdict, IN_TOTO,
};

use crate::seams::unreadable;
use crate::{not_applicable, ok, Rig, Scenario};

pub const GROUP: &str = "landing";

/// What a landing records of one verdict: its check, the build identity
/// it was reached on, its outcome and its signer (78).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recorded {
    pub check: Check,
    pub build: BuildId,
    pub outcome: Outcome,
    pub signer: SignerId,
}

/// A landing, made new for each scenario.
pub trait LandingRig: Rig {
    fn landing(&self) -> &dyn Landing;

    /// The branch the landing lands on.
    fn branch(&self) -> Branch;

    /// Whether the landing records verdicts; a plain branch records none
    /// (C.6).
    fn records_verdicts(&self) -> bool;

    /// A new revision the branch has landed, the landing recording
    /// `verdicts` for it as it records any verdict. The suite hands it
    /// verdicts only where the rig records them.
    fn landed(&self, verdicts: &[Recorded]) -> Revision;

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
            name: "a revision with no verdict",
            plays: None,
            cites: &[78],
            run: a_revision_with_no_verdict,
        },
        Scenario {
            name: "a landing that cannot be reached",
            plays: None,
            cites: &[58, 59, 78],
            run: a_landing_that_cannot_be_reached,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
        Scenario {
            name: "two signers, each by its own name",
            plays: None,
            cites: &[104],
            run: two_signers_each_by_its_own_name,
        },
    ]
}

/// A verdict of `check` on the build `site@4f2a9c1`, by the signer
/// `landing`.
fn recorded(check: &str, outcome: Outcome) -> Recorded {
    Recorded {
        check: Check::new(check),
        build: BuildId::new("site@4f2a9c1"),
        outcome,
        signer: SignerId::new("landing"),
    }
}

/// Whether `verdict` carries the four `recorded` names.
fn carries(verdict: &Verdict, recorded: &Recorded) -> bool {
    verdict.check == recorded.check
        && verdict.build == recorded.build
        && verdict.outcome == recorded.outcome
        && verdict.signer == recorded.signer
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
    if !rig.records_verdicts() {
        not_applicable("the landing records no verdicts");
    }
    let recorded = [
        recorded("test", Outcome::Pass),
        recorded("lint", Outcome::Fail),
    ];
    let at = rig.landed(&recorded);
    let read = ok(rig.landing().verdicts_of(&at), "verdicts-of");
    assert_eq!(
        read.len(),
        recorded.len(),
        "the landing recorded {recorded:?} and answered {read:?}"
    );
    for want in &recorded {
        let held = read.iter().filter(|v| carries(v, want)).count();
        assert_eq!(
            held, 1,
            "the verdicts read hold {want:?} {held} times, not once: {read:?}"
        );
    }
    // The body of the payload is the landing's, and the suite asks
    // nothing of it.
    for verdict in &read {
        let signed = &verdict.envelope;
        assert_eq!(
            signed.payload_type, IN_TOTO,
            "the verdict of {} is signed in another form: {verdict:?}",
            verdict.check
        );
        assert!(
            !signed.payload.is_empty(),
            "the verdict of {} carries no payload: {verdict:?}",
            verdict.check
        );
        assert!(
            !signed.signatures.is_empty()
                && signed.signatures.iter().all(|s| s.signer == verdict.signer),
            "the signature on the verdict of {} names another signer than the verdict's",
            verdict.check
        );
    }
}

fn a_revision_with_no_verdict(rig: &dyn LandingRig) {
    let at = rig.landed(&[]);
    assert_eq!(
        ok(
            rig.landing().verdicts_of(&at),
            "verdicts-of a revision with none"
        ),
        [],
        "a revision with no verdict answered some"
    );
}

fn a_landing_that_cannot_be_reached(rig: &dyn LandingRig) {
    let branch = rig.branch();
    let revision = rig.landed(&[]);
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

/// 104: a landing names each verdict's signer by the name it gives that
/// signer, the one name the policy names it by, and never by a second.
fn two_signers_each_by_its_own_name(rig: &dyn LandingRig) {
    if !rig.records_verdicts() {
        not_applicable("the landing records no verdicts");
    }
    let by = |signer: &str| Recorded {
        check: Check::new("test"),
        build: BuildId::new("site@4f2a9c1"),
        outcome: Outcome::Pass,
        signer: SignerId::new(signer),
    };
    let recorded = [by("box"), by("forge")];
    let at = rig.landed(&recorded);
    let read = ok(rig.landing().verdicts_of(&at), "verdicts-of");
    assert_eq!(
        read.len(),
        recorded.len(),
        "the landing recorded {recorded:?} and answered {read:?}"
    );
    for want in &recorded {
        let named: Vec<&Verdict> = read.iter().filter(|v| v.signer == want.signer).collect();
        assert_eq!(
            named.len(),
            1,
            "{} recorded one verdict, and {} name it: {read:?}",
            want.signer,
            named.len()
        );
        let verdict = named[0];
        assert!(
            carries(verdict, want),
            "the verdict named {} is not the one it recorded: {verdict:?}",
            want.signer
        );
        let signatures = &verdict.envelope.signatures;
        assert!(
            !signatures.is_empty() && signatures.iter().all(|s| s.signer == want.signer),
            "the envelope of the verdict {} recorded holds a signature by another: {verdict:?}",
            want.signer
        );
    }
}
