//! The target seam's profiles (B.7), each by the profile name an
//! environment's `target` table names it by, and the binding of every one
//! built. The repository's environments pick among them (92, C.10,
//! C.12): a profile bound once serves every environment that names it,
//! each handed its own settings as the policy answers them (design D3).
//! The Nix profiles read a closure's artifact through [`nix_closure`],
//! which they share, so none calls another (rule core.2; design D6).

pub mod nix_cache;
pub mod nix_closure;

use std::collections::BTreeMap;
use std::path::Path;

use timber_seams::{Store, Target, TargetProfile};

use nix_cache::NixCache;

/// Every target profile built, by its profile name (92, C.10, C.12;
/// design D3): the Nix cache, reading a version's index and its closures
/// through `store`, a store of its own, since a store is not shared. A
/// profile that runs the repository's own tools is handed `checkout`, the
/// checkout's path, as a projection is; no profile built yet runs one.
pub fn bound(store: Box<dyn Store>, _checkout: &Path) -> BTreeMap<TargetProfile, Box<dyn Target>> {
    BTreeMap::from([(
        TargetProfile::new(nix_cache::PROFILE),
        Box::new(NixCache::new(store)) as Box<dyn Target>,
    )])
}
