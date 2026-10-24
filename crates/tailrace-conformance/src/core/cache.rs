//! The cache's group, holding no scenario yet: bolt 3's cache unit adds
//! the cache entries, never artifacts, below the marker.

use crate::world::FakeWorld;
use crate::Scenario;

pub const GROUP: &str = "cache";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
