//! B.7 A target: where an environment runs, converged by its own
//! reconciler on the version its channel names (88 to 91).

use serde::{Deserialize, Serialize};

use crate::{Answer, ChannelName, Digest, Version};

text! {
    /// An environment, by name.
    EnvironmentName
}

/// One target, the channel it follows, and whether it waits for an
/// approver rather than moving on its own (40, 41).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Environment {
    pub name: EnvironmentName,
    pub channel: ChannelName,
    pub waits: bool,
}

/// 42: an index and its version, which the core made once it verified
/// the claims it was told to require (rule core.29); a profile is handed
/// one and never makes one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verified {
    index: Digest,
    version: Version,
}

impl Verified {
    /// Made by the core alone, after its verify (42, rule core.29).
    pub fn new(index: Digest, version: Version) -> Self {
        Verified { index, version }
    }

    pub fn index(&self) -> &Digest {
        &self.index
    }

    pub fn version(&self) -> &Version {
        &self.version
    }
}

/// 91: what an environment runs, by digest, and when it last converged,
/// as an RFC 3339 time.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Running {
    pub digest: Digest,
    pub converged_at: String,
}

pub trait Target {
    /// 88: the digest `environment`'s channel names; absent when the
    /// channel is empty (31).
    fn desired(&self, environment: &Environment) -> Answer<Digest>;

    /// 89: the digest `environment` runs; unreadable with the reason
    /// when its state cannot be read, never a guess (119).
    fn current(&self, environment: &Environment) -> Answer<Digest>;

    /// 90: `environment` converged on `version`, which the reconciler
    /// has verified (42), or what stopped it.
    fn apply(&self, environment: &Environment, version: &Verified) -> Answer<()>;

    /// 91: what `environment` runs and when it last converged, recorded
    /// where every asker reads it (43).
    fn report(&self, environment: &Environment) -> Answer<Running>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::round_trip;

    #[test]
    fn every_type_round_trips_through_json() {
        round_trip(EnvironmentName::new("production"));
        round_trip(Environment {
            name: EnvironmentName::new("production"),
            channel: ChannelName::new("stable"),
            waits: true,
        });
        let verified = Verified::new(Digest::of(b"index"), Version::new("1.0.0"));
        assert_eq!(verified.index(), &Digest::of(b"index"));
        assert_eq!(verified.version(), &Version::new("1.0.0"));
        round_trip(verified);
        round_trip(Running {
            digest: Digest::of(b"index"),
            converged_at: "2026-10-23T09:00:00Z".into(),
        });
    }
}
