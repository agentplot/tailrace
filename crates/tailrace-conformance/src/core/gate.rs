//! The gate's group, holding no scenario yet: bolt 4's gate unit adds
//! acceptance and refusal below the marker, and the gate's half of S5 and
//! S13, and takes 25 to 30 out of the register (record 0012).

use crate::world::FakeWorld;
use crate::Scenario;

pub const GROUP: &str = "gate";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
