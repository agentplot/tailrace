//! B.9 Binding the contracts: the one binding the core is handed, and
//! reaches every seam through (92, rule core.2; design D8).

use std::collections::BTreeMap;

use crate::{
    Build, Ecosystem, Keeper, Landing, Policy, Projection, Signer, Store, Target, TargetProfile,
};

/// 92: exactly one store written, one build asked, one landing read, one
/// policy and one keeper appended to; the signers bound for the host; and
/// any number of projections, by ecosystem, and targets, by the profile
/// each implements, which an environment's target names (C.10). How the
/// host's bindings file fills it (118) is a profile's.
pub struct Seams {
    pub store: Box<dyn Store>,
    pub build: Box<dyn Build>,
    pub landing: Box<dyn Landing>,
    pub policy: Box<dyn Policy>,
    pub keeper: Box<dyn Keeper>,
    pub signers: Vec<Box<dyn Signer>>,
    pub projections: BTreeMap<Ecosystem, Box<dyn Projection>>,
    pub targets: BTreeMap<TargetProfile, Box<dyn Target>>,
}

impl Seams {
    /// The projection bound for `ecosystem`, or `None` when none is.
    pub fn projection(&self, ecosystem: &Ecosystem) -> Option<&dyn Projection> {
        self.projections.get(ecosystem).map(|p| p.as_ref())
    }

    /// The target bound for `profile`, or `None` when none is.
    pub fn target(&self, profile: &TargetProfile) -> Option<&dyn Target> {
        self.targets.get(profile).map(|t| t.as_ref())
    }
}
