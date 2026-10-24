//! The status group: what the host published, answered from its own
//! record with no seam asked and nothing written (58, 60 to 62).

use crate::world::FakeWorld;
use crate::Scenario;

pub const GROUP: &str = "status";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
