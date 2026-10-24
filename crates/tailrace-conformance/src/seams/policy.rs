//! The policy's group: 82 and 83, and 116 as a policy keeps it (B.5).

use tailrace_seams::{Audience, Policy, Required, Revision};

use crate::{Rig, Scenario};

pub const GROUP: &str = "policy";

/// A policy, made new for each scenario.
pub trait PolicyRig: Rig {
    fn policy(&self) -> &dyn Policy;

    /// A landed revision at which the policy declares what each audience
    /// of `policies` requires, and nothing for any other audience.
    fn at(&self, policies: &[(Audience, Required)]) -> Revision;

    /// The policy made unreachable, every operation answering unreadable,
    /// and the words it answers with.
    fn unreachable(&self) -> String;
}

pub fn scenarios() -> Vec<Scenario<dyn PolicyRig>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}
