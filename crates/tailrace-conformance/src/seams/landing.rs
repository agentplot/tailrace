//! The landing's group: 77 and 78 (B.3).

use tailrace_seams::{Branch, Landing, Revision, Verdict};

use crate::{Rig, Scenario};

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
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
