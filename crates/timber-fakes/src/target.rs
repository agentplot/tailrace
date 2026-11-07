//! The fake target (B.7): a target that is pushed to, answering desired
//! with the head the core hands it (88); per environment, what it runs
//! or that it cannot be read; whether it holds a marker of what it
//! applied (119); and the settings keys it reads (45), as a test scripts
//! it (design D4, D9, D10).

use std::collections::{BTreeMap, BTreeSet};

use serde_json::json;
use timber_seams::{
    Answer, Digest, Environment, EnvironmentName, Head, Not, Running, Seam, Source, Target,
    Verified,
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

/// What the fake target says of current and report once it holds no
/// marker (119).
pub const NO_MARKER: &str = "the fake target holds no marker of what it applied";

/// What each environment runs, whether it can be read, whether the
/// target holds a marker, the settings keys it reads, and the time an
/// apply stamps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetState {
    running: BTreeMap<EnvironmentName, Running>,
    unreadable: BTreeMap<EnvironmentName, String>,
    marker: bool,
    reads: Option<BTreeSet<String>>,
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
    /// A target whose environments run nothing, which holds a marker of
    /// what it applies and reads every settings key.
    pub fn new() -> Self {
        Fake::with(
            Seam::Target,
            TargetState {
                running: BTreeMap::new(),
                unreadable: BTreeMap::new(),
                marker: true,
                reads: None,
                now: "2026-01-01T00:00:00Z".into(),
            },
        )
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

    /// 119: from now the target holds no marker of what it applied:
    /// current and report answer unreadable with [`NO_MARKER`], whatever
    /// was applied.
    pub fn holds_no_marker(&self) {
        self.state().marker = false;
    }

    /// 45: from now the target reads `keys` of an environment's settings
    /// and no other: desired, current and apply answer unreadable naming
    /// the first other key, and nothing is applied.
    pub fn reads<K: Into<String>>(&self, keys: impl IntoIterator<Item = K>) {
        self.state().reads = Some(keys.into_iter().map(Into::into).collect());
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
    /// 45: unreadable naming the first key of `environment`'s settings
    /// the target does not read, where it reads only some.
    fn settings(&self, environment: &Environment, source: &Source) -> Answer<()> {
        let (Some(reads), Some(settings)) = (&self.reads, environment.settings.as_object()) else {
            return Ok(());
        };
        match settings.keys().find(|key| !reads.contains(*key)) {
            None => Ok(()),
            Some(key) => Err(Not::unreadable(
                source,
                format!("setting {key} of {}", environment.name),
                Some(format!("the fake target reads no setting {key}")),
            )),
        }
    }

    fn running(&self, environment: &Environment, source: &Source) -> Answer<&Running> {
        if !self.marker {
            return Err(Not::unreadable(
                source,
                runs(environment),
                Some(NO_MARKER.into()),
            ));
        }
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
    /// 88: the head the core handed, as a target that is pushed to
    /// answers it: its digest, or its absent where the channel is empty
    /// (31).
    fn desired(&self, environment: &Environment, head: Head<'_>) -> Answer<Digest> {
        let given = json!({ "environment": environment });
        let what = format!("the head of channel {}", environment.channel);
        self.ask(TargetOp::Desired, given, what, |s, source| {
            s.settings(environment, source)?;
            head()
        })
    }

    fn current(&self, environment: &Environment) -> Answer<Digest> {
        let given = json!({ "environment": environment });
        self.ask(TargetOp::Current, given, runs(environment), |s, source| {
            s.settings(environment, source)?;
            s.running(environment, source).map(|r| r.digest.clone())
        })
    }

    fn apply(&self, environment: &Environment, version: &Verified) -> Answer<()> {
        let given = json!({ "environment": environment, "version": version });
        self.ask(TargetOp::Apply, given, runs(environment), |s, source| {
            s.settings(environment, source)?;
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
    use timber_seams::{ChannelId, ChannelName, ComponentName, Version};

    use super::*;
    use crate::{Fault, Times};

    fn production() -> Environment {
        Environment {
            name: EnvironmentName::new("production"),
            channel: ChannelId::new(Some(ComponentName::new("web")), ChannelName::new("stable")),
            waits: false,
            settings: json!({}),
        }
    }

    /// The head of a channel the chain holds none of, as the core hands
    /// it.
    fn empty() -> Answer<Digest> {
        Err(Not::absent(
            &Source::new(Seam::Keeper, "fake"),
            "the head of channel web/stable",
        ))
    }

    fn source() -> Source {
        Source::new(Seam::Target, "fake")
    }

    fn index(n: u8) -> Digest {
        Digest::of(&[b'i', n])
    }

    #[test]
    fn desired_answers_the_head_it_is_handed() {
        let target = FakeTarget::new();
        assert_eq!(
            target.desired(&production(), &|| Ok(index(2))),
            Ok(index(2))
        );
        assert_eq!(target.desired(&production(), &empty), empty());
        let calls = target.calls();
        assert_eq!(calls.len(), 2);
        assert!(calls.iter().all(|c| c.op == TargetOp::Desired));
        assert_eq!(calls[1].answer, empty().map(|_| unreachable!()));
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
    fn a_target_that_holds_no_marker_answers_current_and_report_unreadable() {
        let target = FakeTarget::new();
        target.holds_no_marker();
        let verified = Verified::new(index(2), Version::new("1.0.0"));
        assert_eq!(target.apply(&production(), &verified), Ok(()));
        let unreadable = Not::unreadable(&source(), "what production runs", Some(NO_MARKER.into()));
        assert_eq!(target.current(&production()), Err(unreadable.clone()));
        assert_eq!(target.report(&production()), Err(unreadable));
        let ops: Vec<TargetOp> = target.calls().iter().map(|c| c.op).collect();
        assert_eq!(ops, [TargetOp::Apply, TargetOp::Current, TargetOp::Report]);
        assert_eq!(target.calls()[0].given["version"]["index"], json!(index(2)));
    }

    #[test]
    fn a_settings_key_the_target_does_not_read_is_unreadable_and_nothing_is_applied() {
        let target = FakeTarget::new();
        target.reads(["cache"]);
        let environment = Environment {
            settings: json!({ "cache": "a place of its own", "token": "a credential" }),
            ..production()
        };
        let naming_token = |answer: Answer<()>| match answer {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, source(), "{u:?}");
                assert!(u.what.contains("token"), "{u:?}");
                assert!(!u.what.contains("cache"), "{u:?}");
            }
            other => panic!("answered {other:?}, not unreadable naming token"),
        };
        let verified = Verified::new(index(2), Version::new("1.0.0"));
        naming_token(target.apply(&environment, &verified));
        naming_token(target.desired(&environment, &|| Ok(index(2))).map(|_| ()));
        naming_token(target.current(&environment).map(|_| ()));

        let read = Environment {
            settings: json!({ "cache": "a place of its own" }),
            ..production()
        };
        assert!(
            matches!(target.current(&read), Err(Not::Absent(_))),
            "nothing was applied"
        );
        assert_eq!(target.desired(&read, &|| Ok(index(2))), Ok(index(2)));
        assert_eq!(target.apply(&read, &verified), Ok(()));
        assert_eq!(target.current(&read), Ok(index(2)));
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
        assert!(target.desired(&production(), &empty).is_err());
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
