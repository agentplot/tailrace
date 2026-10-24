//! The record's group, holding no scenario yet: bolt 3's unit of the
//! push after a landing adds the record rebuilt from the store and git
//! below the marker.

use crate::world::FakeWorld;
use crate::Scenario;

pub const GROUP: &str = "record";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
