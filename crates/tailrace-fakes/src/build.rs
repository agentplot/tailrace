//! The fake build (B.2): what each revision left of each output, as a
//! test scripts it; an output it was not told of is unmade.

use std::collections::BTreeMap;

use serde_json::json;
use tailrace_seams::{Answer, Build, Not, OutputName, Report, Revision, Seam, Why};

use crate::Fake;

/// The fake build's operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildOp {
    Ensure,
    Report,
}

/// What each revision left of each output.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildState {
    made: BTreeMap<(Revision, OutputName), Report>,
}

/// A build in memory, which builds nothing.
pub type FakeBuild = Fake<BuildState, BuildOp>;

impl Default for FakeBuild {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeBuild {
    pub fn new() -> Self {
        Fake::with(Seam::Build, BuildState::default())
    }

    /// `output` left at `revision` as `report` says: its build identity,
    /// its outcome, a failed one among them (16), and its files.
    pub fn made(&self, revision: &Revision, output: &OutputName, report: Report) {
        self.state()
            .made
            .insert((revision.clone(), output.clone()), report);
    }
}

impl Build for FakeBuild {
    fn ensure(&self, revision: &Revision, outputs: &[OutputName]) -> Answer<()> {
        let given = json!({ "revision": revision, "outputs": outputs });
        self.ask(
            BuildOp::Ensure,
            given,
            format!("the build of {revision}"),
            |s, source| {
                let unmade: Vec<OutputName> = outputs
                    .iter()
                    .filter(|o| !s.made.contains_key(&(revision.clone(), (*o).clone())))
                    .cloned()
                    .collect();
                if unmade.is_empty() {
                    Ok(())
                } else {
                    Err(Not::refused(source, Why::Unmade { outputs: unmade }, None))
                }
            },
        )
    }

    fn report(&self, revision: &Revision, output: &OutputName) -> Answer<Report> {
        let given = json!({ "revision": revision, "output": output });
        let what = format!("output {output} at {revision}");
        self.ask(BuildOp::Report, given, what.clone(), |s, source| {
            s.made
                .get(&(revision.clone(), output.clone()))
                .cloned()
                .ok_or_else(|| Not::absent(source, what))
        })
    }
}

#[cfg(test)]
mod tests {
    use tailrace_seams::{BuildId, Outcome, Source};

    use super::*;
    use crate::{Fault, Times};

    fn report(outcome: Outcome) -> Report {
        Report {
            build: BuildId::new("b7e1"),
            outcome,
            files: vec!["dist/out".into()],
        }
    }

    fn outputs(names: &[&str]) -> Vec<OutputName> {
        names.iter().map(|n| OutputName::new(*n)).collect()
    }

    #[test]
    fn outputs_that_could_not_be_made_are_named() {
        let build = FakeBuild::new();
        let rev = Revision::new("4f2a9c1");
        build.made(&rev, &OutputName::new("site"), report(Outcome::Pass));
        build.made(&rev, &OutputName::new("cli"), report(Outcome::Pass));
        assert_eq!(
            build.ensure(&rev, &outputs(&["site", "cli", "docs"])),
            Err(Not::refused(
                &Source::new(Seam::Build, "fake"),
                Why::Unmade {
                    outputs: outputs(&["docs"])
                },
                None
            ))
        );
        assert_eq!(build.ensure(&rev, &outputs(&["site", "cli"])), Ok(()));
        assert!(build
            .ensure(&Revision::new("other"), &outputs(&["site"]))
            .is_err());
    }

    #[test]
    fn a_report_names_the_build_its_outcome_and_files() {
        let build = FakeBuild::new();
        let rev = Revision::new("4f2a9c1");
        build.made(&rev, &OutputName::new("site"), report(Outcome::Fail));
        assert_eq!(
            build.report(&rev, &OutputName::new("site")),
            Ok(report(Outcome::Fail))
        );
        assert_eq!(
            build.report(&rev, &OutputName::new("docs")),
            Err(Not::absent(
                &Source::new(Seam::Build, "fake"),
                "output docs at 4f2a9c1"
            ))
        );
    }

    #[test]
    fn every_operation_can_be_faulted_and_is_recorded() {
        let build = FakeBuild::new();
        let rev = Revision::new("4f2a9c1");
        build.fault(
            BuildOp::Ensure,
            Fault::Unreadable("runner gone".into()),
            Times::Once,
        );
        build.fault(
            BuildOp::Report,
            Fault::Unreadable("runner gone".into()),
            Times::Once,
        );
        assert!(
            matches!(build.ensure(&rev, &[]), Err(Not::Unreadable(u)) if u.said.as_deref() == Some("runner gone"))
        );
        assert!(matches!(
            build.report(&rev, &OutputName::new("site")),
            Err(Not::Unreadable(u)) if u.said.as_deref() == Some("runner gone")
        ));
        assert_eq!(
            build.calls().iter().map(|c| c.op).collect::<Vec<_>>(),
            [BuildOp::Ensure, BuildOp::Report]
        );
    }
}
