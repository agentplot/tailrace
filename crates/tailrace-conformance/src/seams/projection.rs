//! A projection's group: 84 to 87, and 48 and 50 as far as a projection
//! keeps them (B.6).

use tailrace_seams::{Artifact, Output, Projection};

use crate::{Rig, Scenario};

pub const GROUP: &str = "projection";

/// A projection, made new for each scenario.
pub trait ProjectionRig: Rig {
    fn projection(&self) -> &dyn Projection;

    /// An output a build left, which the projection packs.
    fn output(&self) -> Output;

    /// `n` artifacts in the projection's ecosystem, each of another
    /// version, held where the projection renders from.
    fn artifacts(&self, n: usize) -> Vec<Artifact>;

    /// Whether the ecosystem's clients read no files, so the projection
    /// serves its layouts (86).
    fn serves(&self) -> bool;

    /// The projection made unreachable, every operation answering
    /// unreadable, and the words it answers with.
    fn unreachable(&self) -> String;
}

pub fn scenarios() -> Vec<Scenario<dyn ProjectionRig>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
