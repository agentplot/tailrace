//! The ledger's group, holding no scenario yet: bolt 4's unit of the
//! keeper and promotion adds promotion, the ledger and bound channels
//! below the marker.

use crate::world::FakeWorld;
use crate::Scenario;

pub const GROUP: &str = "ledger";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
