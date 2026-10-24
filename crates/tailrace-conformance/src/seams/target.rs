//! A target's group: 88 to 91, and 31, 40 and 43 as far as a target keeps
//! them (B.7).

use tailrace_seams::{Digest, Environment, Not, Seam, Target, Verified};

use crate::{ok, Rig, Scenario};

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
        Scenario {
            name: "what an environment should run",
            plays: None,
            cites: &[31, 40, 88],
            run: what_an_environment_should_run,
        },
        Scenario {
            name: "an environment that cannot be read",
            plays: None,
            cites: &[58, 89],
            run: an_environment_that_cannot_be_read,
        },
        Scenario {
            name: "converged and reported",
            plays: None,
            cites: &[43, 90, 91],
            run: converged_and_reported,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}

fn index(n: u8) -> Digest {
    Digest::of(&[b'i', n])
}

fn what_an_environment_should_run(rig: &dyn TargetRig) {
    let target = rig.target();
    let head = index(1);
    let following = rig.environment(Some(&head));
    let empty = rig.environment(None);
    assert_eq!(
        ok(
            target.desired(&following),
            "desired of a channel with a head"
        ),
        head,
        "the environment should run another digest than its channel's head"
    );
    match target.desired(&empty) {
        Err(Not::Absent(a)) => assert_eq!(a.source.seam, Seam::Target, "{a:?}"),
        other => panic!("desired of an empty channel answered {other:?}, not empty"),
    }
}

fn an_environment_that_cannot_be_read(rig: &dyn TargetRig) {
    let (environment, reason) = rig.unreadable_environment();
    match rig.target().current(&environment) {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source.seam, Seam::Target, "{u:?}");
            assert_eq!(
                u.said.as_deref(),
                Some(reason.as_str()),
                "current did not carry the reason"
            );
        }
        other => panic!("current of an environment that cannot be read answered {other:?}"),
    }
}

fn converged_and_reported(rig: &dyn TargetRig) {
    let target = rig.target();
    let head = index(2);
    let environment = rig.environment(Some(&head));
    ok(target.apply(&environment, &rig.verified(&head)), "apply");
    assert_eq!(
        ok(target.current(&environment), "current"),
        head,
        "the environment runs another digest than it converged on"
    );
    let running = ok(target.report(&environment), "report");
    assert_eq!(
        running.digest, head,
        "the report names another digest: {running:?}"
    );
    assert!(
        !running.converged_at.is_empty(),
        "the report says no time it converged: {running:?}"
    );
}
