//! The binding's group: 92, and 93 held of the suite itself (B.9).

use tailrace_seams::{Ecosystem, EnvironmentName, Seams, SignerId};

use crate::seams::store::StoreRig;
use crate::{Rig, Scenario};

pub const GROUP: &str = "binding";

/// What binds the seams, made new for each scenario.
pub trait BindingRig: Rig {
    /// One store, build, landing, policy and keeper, a signer for each of
    /// `signers`, a projection for each of `ecosystems` and a target for
    /// each of `environments`, bound as one binding.
    fn bind(
        &self,
        ecosystems: &[Ecosystem],
        environments: &[EnvironmentName],
        signers: &[SignerId],
    ) -> Seams;

    /// A rig whose store holds other bytes under a digest, which breaks
    /// the store's contract.
    fn broken_store(&self) -> Box<dyn StoreRig>;
}

pub fn scenarios() -> Vec<Scenario<dyn BindingRig>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
