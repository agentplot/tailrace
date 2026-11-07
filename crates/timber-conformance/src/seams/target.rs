//! A target's group: 88 to 91, and 31, 40 and 43 as far as a target keeps
//! them (B.7); below the marker, a target that holds no marker of what it
//! applied, and a table holding a key the target does not read (45, 58,
//! 119; design D9, D10).

use timber_seams::{Answer, Digest, Environment, Not, Seam, Source, Target, Verified};

use crate::{not_applicable, ok, Rig, Scenario};

pub const GROUP: &str = "target";

/// A target, made new for each scenario.
pub trait TargetRig: Rig {
    fn target(&self) -> &dyn Target;

    /// The version at `digest`, verified as the core hands it to a target
    /// after its verify (42).
    fn verified(&self, digest: &Digest) -> Verified;

    /// An environment whose channel names `head`, or nothing. A target
    /// that is pushed to is handed the head at desired (88), and its rig
    /// may leave `head` unread.
    fn environment(&self, head: Option<&Digest>) -> Environment;

    /// An environment whose state cannot be read, and the reason the
    /// target gives.
    fn unreadable_environment(&self) -> (Environment, String);

    /// Whether the target holds a marker of the digest it applied, as the
    /// profile declares; yes by default. A target that can hold none, a
    /// Nix binary cache, answers current and report unread with the
    /// reason, and the group runs "applied where no marker is held" in
    /// place of "converged and reported" (119; design D9).
    fn holds_marker(&self) -> bool {
        true
    }

    /// An environment whose settings hold the target's own table and one
    /// key more, which the target does not read, and that key (45;
    /// design D10).
    fn foreign_settings(&self) -> (Environment, String);
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
        Scenario {
            name: "applied where no marker is held",
            plays: None,
            cites: &[89, 91, 119],
            run: applied_where_no_marker_is_held,
        },
        Scenario {
            name: "a table with a key the target does not read",
            plays: None,
            cites: &[45, 58],
            run: a_table_with_a_key_the_target_does_not_read,
        },
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
            target.desired(&following, &|| Ok(head.clone())),
            "desired of a channel with a head"
        ),
        head,
        "the environment should run another digest than its channel's head"
    );
    let none = || {
        Err(Not::absent(
            &Source::new(Seam::Keeper, "the suite"),
            format!("the head of channel {}", empty.channel),
        ))
    };
    match target.desired(&empty, &none) {
        Err(Not::Absent(_)) => {}
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
    if !rig.holds_marker() {
        not_applicable(
            "the target holds no marker of what it applied, so what an environment runs is \
             unread, and the group runs \"applied where no marker is held\" in its stead (119)",
        );
    }
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

/// The reason `answer` gives for what it could not read, where it is
/// unreadable from the target and names no `applied`; else a panic naming
/// what it answered.
fn unread<T: std::fmt::Debug>(answer: Answer<T>, asked: &str, applied: &Digest) -> String {
    match answer {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source.seam, Seam::Target, "{asked}: {u:?}");
            let said = u
                .said
                .clone()
                .unwrap_or_else(|| panic!("{asked} gave no reason it was unread: {u:?}"));
            assert!(!said.trim().is_empty(), "{asked} gave no reason: {u:?}");
            assert!(
                !u.what.contains(applied.as_str()) && !said.contains(applied.as_str()),
                "{asked} named the digest applied: {u:?}"
            );
            said
        }
        other => panic!("{asked} of a target that holds no marker answered {other:?}"),
    }
}

fn applied_where_no_marker_is_held(rig: &dyn TargetRig) {
    if rig.holds_marker() {
        not_applicable(
            "the target holds a marker of what it applied, so the group runs \"converged and \
             reported\" in its stead (119)",
        );
    }
    let target = rig.target();
    let version = rig.verified(&index(3));
    let applied = version.index().clone();
    let environment = rig.environment(Some(&applied));
    ok(target.apply(&environment, &version), "apply");
    unread(target.current(&environment), "current", &applied);
    unread(target.report(&environment), "report", &applied);
}

fn a_table_with_a_key_the_target_does_not_read(rig: &dyn TargetRig) {
    let target = rig.target();
    let version = rig.verified(&index(4));
    let applied = version.index().clone();
    let (foreign, key) = rig.foreign_settings();
    let names_it = |answer: Answer<()>, asked: &str| match answer {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source.seam, Seam::Target, "{asked}: {u:?}");
            assert!(u.what.contains(&key), "{asked} did not name {key}: {u:?}");
        }
        other => panic!("{asked} of a table holding {key} answered {other:?}, not unreadable"),
    };
    names_it(
        target.desired(&foreign, &|| Ok(applied.clone())).map(drop),
        "desired",
    );
    names_it(target.current(&foreign).map(drop), "current");
    names_it(target.apply(&foreign, &version), "apply");
    if rig.holds_marker() {
        let mut own = foreign.clone();
        own.settings
            .as_object_mut()
            .unwrap_or_else(|| panic!("the settings are no table: {}", foreign.settings))
            .remove(&key);
        assert_ne!(
            target.current(&own),
            Ok(applied),
            "the environment runs what an apply refused for {key}"
        );
    }
}
