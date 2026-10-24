//! The publish group: a landed revision offered as a version, S1 and S2,
//! and what publish refuses (2 to 20, 102, 110, 111).

use crate::world::FakeWorld;
use crate::Scenario;

pub const GROUP: &str = "publish";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
