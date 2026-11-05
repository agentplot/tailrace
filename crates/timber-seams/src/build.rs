//! B.2 The build: the repository's own commands over one revision, which
//! timber asks for outputs and never runs itself (74 to 76), and what a
//! checkout's own run of them left (139).

use serde::{Deserialize, Serialize};

use crate::{Answer, BuildId, Outcome, OutputName, Revision};

/// 75: where an output came from: the build identity, the build's
/// outcome, and where the output's files are.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub build: BuildId,
    pub outcome: Outcome,
    pub files: Vec<String>,
}

/// 139: one build identity a checkout's run left: the tasks that came
/// from it, its outcome, and where their files are.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Left {
    pub build: BuildId,
    pub tasks: Vec<String>,
    pub outcome: Outcome,
    pub files: Vec<String>,
}

/// 76: the same revision and the same inputs give the same build
/// identity on every host.
pub trait Build {
    /// 74: every output in `outputs`, all the authors declare and asked
    /// in one asking, present for `revision`; refused
    /// [`Why::Unmade`](crate::Why::Unmade) naming each output the build
    /// could not make, which refuses the version whole (16).
    fn ensure(&self, revision: &Revision, outputs: &[OutputName]) -> Answer<()>;

    /// 75: the build identity `output` came from at `revision`, the
    /// build's outcome, and where its files are; absent when the build
    /// left no such output.
    fn report(&self, revision: &Revision, output: &OutputName) -> Answer<Report>;

    /// 139: the build identities whose outputs the checkout at `at`
    /// holds, each with its tasks, its outcome and where its files are,
    /// read from the build's own records and nothing of timber's; an
    /// empty list where the checkout ran nothing.
    fn left(&self, at: &str) -> Answer<Vec<Left>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::round_trip;

    #[test]
    fn a_report_round_trips_through_json() {
        round_trip(Report {
            build: BuildId::new("b7e1"),
            outcome: Outcome::Fail,
            files: vec!["dist/site.tar".into()],
        });
    }

    #[test]
    fn what_a_checkout_left_round_trips_through_json() {
        round_trip(vec![
            Left {
                build: BuildId::new("b7e1"),
                tasks: vec!["web:build".into(), "web:check".into()],
                outcome: Outcome::Pass,
                files: vec!["/work/web/dist/site.tar".into()],
            },
            Left {
                build: BuildId::new("c001"),
                tasks: vec!["app:test".into()],
                outcome: Outcome::Fail,
                files: vec![],
            },
        ]);
    }
}
