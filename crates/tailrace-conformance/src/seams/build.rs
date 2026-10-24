//! The build's group: 74 to 76 (B.2).

use tailrace_seams::{Build, OutputName, Revision};

use crate::{Rig, Scenario};

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
}

pub fn scenarios() -> Vec<Scenario<dyn BuildRig>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
