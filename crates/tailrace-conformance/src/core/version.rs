//! The version's group: a version per component from the history alone,
//! claimed as its name in the shared history (21 to 24, 65, 123, 124).

use crate::world::FakeWorld;
use crate::Scenario;

pub const GROUP: &str = "version";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
