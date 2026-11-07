//! The target seam's profiles (B.7), each by the profile name an
//! environment's `target` table names it by, and the binding of every one
//! built. The repository's environments pick among them (92, C.10,
//! C.12): a profile bound once serves every environment that names it,
//! each handed its own settings as the policy answers them (design D3).
//! The Nix profiles read a closure's artifact through [`nix_closure`],
//! which they share, so none calls another (rule core.2; design D6).

pub mod nix_cache;
pub mod nix_closure;
pub mod nix_fleet;

use std::collections::BTreeMap;
use std::path::Path;

use timber_seams::{Answer, RepositoryName, Store, Target, TargetProfile};

use nix_cache::NixCache;
use nix_fleet::push::{self, ClanPush};

/// Every target profile built, by its profile name (92, C.10, C.12;
/// design D3), each reading a version's index and its closures through a
/// store of its own, which `store` binds, since a store is not shared:
/// the Nix cache, and the clan push profile, which runs the fleet's own
/// tools against `checkout`, the checkout's path, and keeps its markers
/// under `repository`, the checkout's repository, or answers why it has
/// none at every operation that needs it (119).
pub fn bound(
    store: &dyn Fn() -> Answer<Box<dyn Store>>,
    checkout: &Path,
    repository: Answer<RepositoryName>,
) -> Answer<BTreeMap<TargetProfile, Box<dyn Target>>> {
    Ok(BTreeMap::from([
        (
            TargetProfile::new(nix_cache::PROFILE),
            Box::new(NixCache::new(store()?)) as Box<dyn Target>,
        ),
        (
            TargetProfile::new(push::PROFILE),
            Box::new(ClanPush::new(store()?, checkout, repository)) as Box<dyn Target>,
        ),
    ]))
}
