//! The layouts' group, holding no scenario yet: bolt 5's unit that
//! renders under a policy adds consumer layouts below the marker.

use crate::world::FakeWorld;
use crate::Scenario;

pub const GROUP: &str = "layouts";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
