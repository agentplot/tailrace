//! The verify group: which named signer made each attestation on a
//! version, and whether it counts, S5 and S13 as far as verify's answer
//! (record 0012).

use crate::world::FakeWorld;
use crate::Scenario;

pub const GROUP: &str = "verify";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
