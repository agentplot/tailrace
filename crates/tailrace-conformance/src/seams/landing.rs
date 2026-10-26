//! The landing's group: 77 and 78 (B.3). The rig stages what a landing
//! records of each verdict, and the landing makes the rest: the signed
//! claim is the landing's to make and tailrace's only to carry, so the
//! suite makes none (12, rule core.8; design D3a). A landing that reads
//! a remote is held to answering as last fetched where it does not
//! answer (77).

use std::time::{SystemTime, UNIX_EPOCH};

use tailrace_seams::{
    Branch, BuildId, Check, Landing, Outcome, Read, Revision, Seam, SignerId, Verdict, IN_TOTO,
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

    /// Whether the landing reads a remote, and so keeps a copy of it to
    /// answer from where the remote does not; a landing with no remote
    /// bound reads the repository, which is the landing itself (C.5,
    /// C.6).
    fn reads_a_remote(&self) -> bool;

    /// A new revision the branch has landed, the landing recording
    /// `verdicts` for it as it records any verdict. The suite hands it
    /// verdicts only where the rig records them.
    fn landed(&self, verdicts: &[Recorded]) -> Revision;

    /// A revision the branch has not landed.
    fn not_landed(&self) -> Revision;

    /// The landing made unreachable, and the words it answers with: a
    /// landing never read answers every operation unreadable with them,
    /// and one read before answers from its copy as last fetched, saying
    /// so with them (77).
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
        Scenario {
            name: "a landing read as last fetched",
            plays: None,
            cites: &[59, 61, 77, 78],
            run: a_landing_read_as_last_fetched,
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
        )
        .landed,
        "{branch} has not landed {landed}, which it landed"
    );
    assert!(
        !ok(
            landing.landed(&branch, &not),
            "landed of a revision not landed"
        )
        .landed,
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
    let read = ok(rig.landing().verdicts_of(&at), "verdicts-of").verdicts;
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
        )
        .verdicts,
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
    let read = ok(rig.landing().verdicts_of(&at), "verdicts-of").verdicts;
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

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// That `read` is from the copy as last fetched, at a time no earlier
/// than `since`, with `words` unchanged.
fn as_fetched(read: &Read, since: u64, words: &str, asked: &str) {
    match read {
        Read::Kept { at, said } => {
            assert!(
                *at >= since,
                "{asked} was fetched at {at}, before it was first read at {since}"
            );
            assert_eq!(
                said, words,
                "{asked} did not carry the remote's words unchanged"
            );
        }
        Read::Source => panic!("{asked} said it was read from the landing, which was unreachable"),
    }
}

/// 77: a landing read once and then made unreachable answers from its
/// copy as last fetched, as it answered then, saying when and with the
/// remote's words; a revision landed after the copy was fetched is not
/// landed from it, never landed and never refused (58, 59, 61, 78).
fn a_landing_read_as_last_fetched(rig: &dyn LandingRig) {
    if !rig.reads_a_remote() {
        not_applicable("the landing reads no remote: the repository is the landing itself");
    }
    let branch = rig.branch();
    let recorded = if rig.records_verdicts() {
        vec![recorded("test", Outcome::Pass)]
    } else {
        vec![]
    };
    let first = rig.landed(&recorded);
    let since = now();
    let landed = ok(rig.landing().landed(&branch, &first), "landed");
    let verdicts = ok(rig.landing().verdicts_of(&first), "verdicts-of");
    assert!(
        landed.landed,
        "{branch} has not landed {first}, which it landed"
    );
    assert_eq!(
        (&landed.read, &verdicts.read),
        (&Read::Source, &Read::Source),
        "a landing whose remote answers said it read a copy"
    );
    let later = rig.landed(&[]);
    let words = rig.unreachable();

    let landing = rig.landing();
    let again = ok(
        landing.landed(&branch, &first),
        "landed of a landed revision, the landing unreachable",
    );
    assert!(
        again.landed,
        "{branch} had landed {first} when last fetched, and was answered not landed"
    );
    as_fetched(&again.read, since, &words, "landed");
    let read_again = ok(
        landing.verdicts_of(&first),
        "verdicts-of a landed revision, the landing unreachable",
    );
    assert_eq!(
        read_again.verdicts, verdicts.verdicts,
        "verdicts-of answered otherwise from the copy"
    );
    as_fetched(&read_again.read, since, &words, "verdicts-of");

    let after = ok(
        landing.landed(&branch, &later),
        "landed of a revision landed after the copy",
    );
    assert!(
        !after.landed,
        "{later}, landed after the copy was fetched, was answered landed from it"
    );
    as_fetched(
        &after.read,
        since,
        &words,
        "landed of a revision landed after the copy",
    );
}
