//! The build's group: 74 to 76 (B.2), and what a checkout's run left
//! (139, cited as 74 and 75 until the suite's statement list holds it).

use tailrace_seams::{Build, Not, Outcome, OutputName, Revision, Seam, Why};

use crate::seams::unreadable;
use crate::{ok, Rig, Scenario};

pub const GROUP: &str = "build";

/// What a rig's build left of an output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Made {
    /// Made, by a build that passed.
    Pass,
    /// Made, by a build that failed.
    Fail,
    /// Not made.
    Unmade,
}

/// A build, made new for each scenario.
pub trait BuildRig: Rig {
    fn build(&self) -> &dyn Build;

    /// The same build as a second host asks it.
    fn second_host(&self) -> &dyn Build;

    /// A revision at which the build leaves each of `outputs` as its
    /// [`Made`] says.
    fn revision(&self, outputs: &[(OutputName, Made)]) -> Revision;

    /// The build made unreachable, every operation answering unreadable,
    /// and the words it answers with.
    fn unreachable(&self) -> String;

    /// The checkout an ensure of `revision` ran in, as left is given it.
    fn ran_in(&self, revision: &Revision) -> String;

    /// A checkout in which nothing has run.
    fn fresh_checkout(&self) -> String;
}

pub fn scenarios() -> Vec<Scenario<dyn BuildRig>> {
    vec![
        Scenario {
            name: "outputs ensured and reported",
            plays: None,
            cites: &[74, 75],
            run: outputs_ensured_and_reported,
        },
        Scenario {
            name: "outputs the build could not make",
            plays: None,
            cites: &[74],
            run: outputs_the_build_could_not_make,
        },
        Scenario {
            name: "the same revision on a second host",
            plays: None,
            cites: &[76],
            run: the_same_revision_on_a_second_host,
        },
        Scenario {
            name: "a build that cannot be reached",
            plays: None,
            cites: &[58, 59],
            run: a_build_that_cannot_be_reached,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
        Scenario {
            name: "what an ensure left",
            plays: None,
            cites: &[74, 75],
            run: what_an_ensure_left,
        },
        Scenario {
            name: "a checkout that ran nothing",
            plays: None,
            cites: &[74, 75],
            run: a_checkout_that_ran_nothing,
        },
    ]
}

fn output(name: &str) -> OutputName {
    OutputName::new(name)
}

fn outputs_ensured_and_reported(rig: &dyn BuildRig) {
    let build = rig.build();
    let outputs = [output("site"), output("server"), output("checks")];
    let revision = rig.revision(&[
        (outputs[0].clone(), Made::Pass),
        (outputs[1].clone(), Made::Pass),
        (outputs[2].clone(), Made::Fail),
    ]);
    ok(
        build.ensure(&revision, &outputs),
        "ensure of three outputs made",
    );
    for (output, outcome) in outputs
        .iter()
        .zip([Outcome::Pass, Outcome::Pass, Outcome::Fail])
    {
        let report = ok(
            build.report(&revision, output),
            &format!("the report of {output}"),
        );
        assert!(
            !report.build.as_str().is_empty(),
            "the report of {output} names no build identity: {report:?}"
        );
        assert_eq!(
            report.outcome, outcome,
            "the report of {output} answered {report:?}"
        );
        assert!(
            !report.files.is_empty(),
            "the report of {output} says nowhere its files are: {report:?}"
        );
    }
}

fn outputs_the_build_could_not_make(rig: &dyn BuildRig) {
    let outputs = [output("site"), output("server"), output("checks")];
    let revision = rig.revision(&[
        (outputs[0].clone(), Made::Pass),
        (outputs[1].clone(), Made::Unmade),
        (outputs[2].clone(), Made::Unmade),
    ]);
    match rig.build().ensure(&revision, &outputs) {
        Err(Not::Refused(r)) => match r.why {
            Why::Unmade {
                outputs: mut unmade,
            } => {
                unmade.sort();
                assert_eq!(
                    unmade,
                    [output("checks"), output("server")],
                    "ensure named other outputs unmade"
                );
            }
            why => panic!("ensure was refused for {why:?}, not the outputs unmade"),
        },
        other => panic!("ensure of two outputs unmade answered {other:?}"),
    }
}

fn the_same_revision_on_a_second_host(rig: &dyn BuildRig) {
    let site = output("site");
    let revision = rig.revision(&[(site.clone(), Made::Pass)]);
    let here = ok(rig.build().report(&revision, &site), "the report here");
    let there = ok(
        rig.second_host().report(&revision, &site),
        "the report on the second host",
    );
    assert_eq!(
        here.build, there.build,
        "one revision's output came from two build identities"
    );
}

fn a_build_that_cannot_be_reached(rig: &dyn BuildRig) {
    let site = output("site");
    let revision = rig.revision(&[(site.clone(), Made::Pass)]);
    let words = rig.unreachable();
    let build = rig.build();
    let ensured = build.ensure(&revision, std::slice::from_ref(&site));
    unreadable(ensured, Seam::Build, &words, "ensure");
    unreadable(
        build.report(&revision, &site),
        Seam::Build,
        &words,
        "report",
    );
}

fn what_an_ensure_left(rig: &dyn BuildRig) {
    let build = rig.build();
    let outputs = [output("site"), output("server")];
    let revision = rig.revision(&[
        (outputs[0].clone(), Made::Pass),
        (outputs[1].clone(), Made::Pass),
    ]);
    ok(
        build.ensure(&revision, &outputs),
        "ensure of two outputs made",
    );
    let left = ok(
        build.left(&rig.ran_in(&revision)),
        "what the ensure's checkout left",
    );
    for output in &outputs {
        let report = ok(
            build.report(&revision, output),
            &format!("the report of {output}"),
        );
        assert!(
            left.iter()
                .any(|l| l.build == report.build && l.outcome == Outcome::Pass),
            "left names no pass of {output}'s build identity {}: {left:?}",
            report.build
        );
    }
}

fn a_checkout_that_ran_nothing(rig: &dyn BuildRig) {
    let left = ok(
        rig.build().left(&rig.fresh_checkout()),
        "what a checkout that ran nothing left",
    );
    assert!(left.is_empty(), "a checkout that ran nothing left {left:?}");
}
