//! The keeper's group: 120 to 122, and 34, 35 and 105 as a keeper keeps
//! them (B.8).

use tailrace_seams::{ChannelName, Envelope, Keeper};

use crate::{Rig, Scenario};

pub const GROUP: &str = "keeper";

/// A keeper, made new for each scenario.
pub trait KeeperRig: Rig {
    fn keeper(&self) -> &dyn Keeper;

    /// The `n`th promotion claim of `channel`, signed, each `n` another
    /// claim.
    fn promotion(&self, channel: &ChannelName, n: u32) -> Envelope;

    /// Two promotions held in `channel`'s chain naming one predecessor, a
    /// fork, as the keeper came to hold them.
    fn fork(&self, channel: &ChannelName) -> [Envelope; 2];

    /// The keeper made unreachable, every operation answering unreadable,
    /// and the words it answers with.
    fn unreachable(&self) -> String;
}

pub fn scenarios() -> Vec<Scenario<dyn KeeperRig>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
