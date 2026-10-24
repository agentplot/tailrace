//! A target's group: 88 to 91, and 31, 40 and 43 as far as a target keeps
//! them (B.7).

use tailrace_seams::{Digest, Environment, Target, Verified};

use crate::{Rig, Scenario};

pub const GROUP: &str = "target";

/// A target, made new for each scenario.
pub trait TargetRig: Rig {
    fn target(&self) -> &dyn Target;

    /// The version at `digest`, verified as the core hands it to a target
    /// after its verify (42).
    fn verified(&self, digest: &Digest) -> Verified;

    /// An environment whose channel names `head`, or nothing.
    fn environment(&self, head: Option<&Digest>) -> Environment;

    /// An environment whose state cannot be read, and the reason the
    /// target gives.
    fn unreadable_environment(&self) -> (Environment, String);
}

pub fn scenarios() -> Vec<Scenario<dyn TargetRig>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
