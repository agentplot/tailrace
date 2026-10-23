//! B.2 The build: the repository's own commands over one revision, which
//! tailrace asks for outputs and never runs itself (74 to 76).

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
}
