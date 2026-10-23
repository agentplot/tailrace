//! The fake target (B.7): per environment, its channel's head or empty,
//! and what it runs or that it cannot be read (design D7).

use std::collections::BTreeMap;

use serde_json::json;
use tailrace_seams::{
    Answer, Digest, Environment, EnvironmentName, Not, Running, Seam, Target, Verified,
};

use crate::Fake;

/// The fake target's operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetOp {
    Desired,
    Current,
    Apply,
    Report,
}

/// Each environment's channel head, what it runs, whether it can be read,
/// and the time an apply stamps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetState {
    heads: BTreeMap<EnvironmentName, Digest>,
    running: BTreeMap<EnvironmentName, Running>,
    unreadable: BTreeMap<EnvironmentName, String>,
    now: String,
}

/// A target in memory.
pub type FakeTarget = Fake<TargetState, TargetOp>;

impl Default for FakeTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeTarget {
    /// A target whose environments' channels are empty and which run
    /// nothing.
    pub fn new() -> Self {
        Fake::with(
            Seam::Target,
            TargetState {
                heads: BTreeMap::new(),
                running: BTreeMap::new(),
                unreadable: BTreeMap::new(),
                now: "2026-01-01T00:00:00Z".into(),
            },
        )
    }

    /// The channel `environment` follows names `digest`.
    pub fn head(&self, environment: &EnvironmentName, digest: &Digest) {
        self.state()
            .heads
            .insert(environment.clone(), digest.clone());
    }

    /// `environment` runs what `running` says.
    pub fn runs(&self, environment: &EnvironmentName, running: Running) {
        self.state().running.insert(environment.clone(), running);
    }

    /// `environment`'s state cannot be read, for the reason `words` give.
    pub fn unreadable(&self, environment: &EnvironmentName, words: impl Into<String>) {
        self.state()
            .unreadable
            .insert(environment.clone(), words.into());
    }

    /// An apply from now converges at `when`, an RFC 3339 time.
    pub fn at(&self, when: impl Into<String>) {
        self.state().now = when.into();
    }
}

fn runs(environment: &Environment) -> String {
    format!("what {} runs", environment.name)
}

impl TargetState {
    fn running(
        &self,
        environment: &Environment,
        source: &tailrace_seams::Source,
    ) -> Answer<&Running> {
        if let Some(words) = self.unreadable.get(&environment.name) {
            return Err(Not::unreadable(
                source,
                runs(environment),
                Some(words.clone()),
            ));
        }
        self.running
            .get(&environment.name)
            .ok_or_else(|| Not::absent(source, runs(environment)))
    }
}

impl Target for FakeTarget {
    fn desired(&self, environment: &Environment) -> Answer<Digest> {
        let given = json!({ "environment": environment });
        let what = format!("the head of channel {}", environment.channel);
        self.ask(TargetOp::Desired, given, what.clone(), |s, source| {
            s.heads
                .get(&environment.name)
                .cloned()
                .ok_or_else(|| Not::absent(source, what))
        })
    }

    fn current(&self, environment: &Environment) -> Answer<Digest> {
        let given = json!({ "environment": environment });
        self.ask(TargetOp::Current, given, runs(environment), |s, source| {
            s.running(environment, source).map(|r| r.digest.clone())
        })
    }

    fn apply(&self, environment: &Environment, version: &Verified) -> Answer<()> {
        let given = json!({ "environment": environment, "version": version });
        self.ask(TargetOp::Apply, given, runs(environment), |s, _| {
            let running = Running {
                digest: version.index().clone(),
                converged_at: s.now.clone(),
            };
            s.running.insert(environment.name.clone(), running);
            Ok(())
        })
    }

    fn report(&self, environment: &Environment) -> Answer<Running> {
        let given = json!({ "environment": environment });
        self.ask(TargetOp::Report, given, runs(environment), |s, source| {
            s.running(environment, source).cloned()
        })
    }
}

#[cfg(test)]
mod tests {
    use tailrace_seams::{ChannelName, Source, Version};

    use super::*;
    use crate::{Fault, Times};

    fn production() -> Environment {
        Environment {
            name: EnvironmentName::new("production"),
            channel: ChannelName::new("stable"),
            waits: false,
        }
    }

    fn source() -> Source {
        Source::new(Seam::Target, "fake")
    }

    #[test]
    fn an_empty_channel_is_absent_and_a_head_is_desired() {
        let target = FakeTarget::new();
        assert_eq!(
            target.desired(&production()),
            Err(Not::absent(&source(), "the head of channel stable"))
        );
        target.head(&production().name, &Digest::of(b"index"));
        assert_eq!(target.desired(&production()), Ok(Digest::of(b"index")));
    }

    #[test]
    fn an_environment_that_cannot_be_read_is_unreadable_never_a_digest() {
        let target = FakeTarget::new();
        target.runs(
            &production().name,
            Running {
                digest: Digest::of(b"old"),
                converged_at: "2026-01-01T00:00:00Z".into(),
            },
        );
        target.unreadable(&production().name, "no marker on the target");
        let unreadable = Err(Not::unreadable(
            &source(),
            "what production runs",
            Some("no marker on the target".into()),
        ));
        assert_eq!(target.current(&production()), unreadable);
        assert_eq!(
            target.report(&production()),
            unreadable.map(|_: Digest| unreachable!())
        );
    }

    #[test]
    fn an_apply_converges_and_is_reported() {
        let target = FakeTarget::new();
        assert!(matches!(target.current(&production()), Err(Not::Absent(_))));
        target.at("2026-10-23T09:00:00Z");
        let verified = Verified::new(Digest::of(b"index"), Version::new("1.0.0"));
        assert_eq!(target.apply(&production(), &verified), Ok(()));
        assert_eq!(target.current(&production()), Ok(Digest::of(b"index")));
        assert_eq!(
            target.report(&production()),
            Ok(Running {
                digest: Digest::of(b"index"),
                converged_at: "2026-10-23T09:00:00Z".into(),
            })
        );
    }

    #[test]
    fn every_operation_can_be_faulted_and_is_recorded() {
        let target = FakeTarget::new();
        let ops = [
            TargetOp::Desired,
            TargetOp::Current,
            TargetOp::Apply,
            TargetOp::Report,
        ];
        for op in ops {
            target.fault(
                op,
                Fault::Unreadable("cluster unreachable".into()),
                Times::Once,
            );
        }
        let verified = Verified::new(Digest::of(b"index"), Version::new("1.0.0"));
        assert!(target.desired(&production()).is_err());
        assert!(target.current(&production()).is_err());
        assert!(target.apply(&production(), &verified).is_err());
        assert!(target.report(&production()).is_err());
        assert!(target
            .calls()
            .iter()
            .all(|c| matches!(&c.answer, Err(Not::Unreadable(u)) if u.said.as_deref() == Some("cluster unreachable"))));
        assert_eq!(target.calls().iter().map(|c| c.op).collect::<Vec<_>>(), ops);
    }
}
