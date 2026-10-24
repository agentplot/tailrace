//! The store's group: 66 to 73, and the clauses of Part A a store's
//! operations keep (B.1).

use tailrace_seams::Store;

use crate::{Rig, Scenario};

pub const GROUP: &str = "store";

/// A store, made new for each scenario.
pub trait StoreRig: Rig {
    fn store(&self) -> &dyn Store;

    /// The store made unreachable, every operation answering unreadable,
    /// and the words it answers with, which the profile carries
    /// unchanged.
    fn unreachable(&self) -> String;
}

pub fn scenarios() -> Vec<Scenario<dyn StoreRig>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
