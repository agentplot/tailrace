//! Spec sluice-landing: the landing seam bound to sluice's records, each
//! test on a scratch repository of its own with a bare remote, the
//! records written as sluice v0.37.0 writes them and signed as its signers
//! sign; the suite's landing group over it; and a real landing by sluice
//! itself (design D9). Needs ssh-keygen, moon and sluice, so it is in the
//! tools layer.

#![cfg(feature = "tools")]

mod scratch;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use scratch::gate::{Gate, SIGNER};
use scratch::recorder::{remote_file, Ran, Recorder};
use scratch::{git_fails, home, moon_hash, moon_workspace, Scratch, Task, HOME_ENV, MAIN};
use tailrace::profiles::landing::sluice::{as_written, Sluice};
use tailrace_conformance::seams::landing::{self, LandingRig, Recorded};
use tailrace_conformance::{run, Rig};
use tailrace_seams::{Branch, Landing, Not, Outcome, Revision, Verdict};

const SYSTEM: &str = "linux-x64";

/// A scratch repository whose `main` sluice keeps on the remote, the
/// recorder of its records, and the sluice landing over it.
struct SluiceRig {
    scratch: Scratch,
    recorder: Recorder,
    landing: Sluice,
}

impl SluiceRig {
    fn new() -> SluiceRig {
        let scratch = Scratch::new();
        let recorder = Recorder::new(&scratch.repo, true, &scratch.path().join("keys"));
        let landing = Sluice::new(&scratch.repo.path, Some("origin".into()));
        SluiceRig {
            scratch,
            recorder,
            landing,
        }
    }

    /// The same records, read by a sluice landing that keeps nothing of
    /// this one's.
    fn fresh(&self) -> Sluice {
        Sluice::new(&self.scratch.repo.path, Some("origin".into()))
    }
}

fn outcome(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Pass => "pass",
        Outcome::Fail => "fail",
    }
}

impl Rig for SluiceRig {
    fn profile(&self) -> String {
        "sluice".into()
    }
}

impl LandingRig for SluiceRig {
    fn landing(&self) -> &dyn Landing {
        &self.landing
    }

    fn branch(&self) -> Branch {
        Branch::new(MAIN)
    }

    fn records_verdicts(&self) -> bool {
        true
    }

    /// One submission landed, and each signer's results recorded in one
    /// run record of the combination that landed it.
    fn landed(&self, verdicts: &[Recorded]) -> Revision {
        let landed = self.recorder.land(1, true);
        let mut signers: Vec<&str> = verdicts.iter().map(|v| v.signer.as_str()).collect();
        signers.sort_unstable();
        signers.dedup();
        for signer in signers {
            let results: Vec<Ran> = verdicts
                .iter()
                .filter(|v| v.signer.as_str() == signer)
                .map(|v| Ran::new(v.check.as_str(), v.build.as_str(), outcome(v.outcome)))
                .collect();
            self.recorder
                .record(signer, &landed.combination, SYSTEM, &results);
        }
        Revision::new(&landed.commits[0])
    }

    /// A commit no landing made.
    fn not_landed(&self) -> Revision {
        Revision::new(self.scratch.repo.git(&[
            "commit-tree",
            "HEAD^{tree}",
            "-p",
            "HEAD",
            "-m",
            "never landed",
        ]))
    }

    /// The remote removed, and git's words for listing it.
    fn unreachable(&self) -> String {
        fs::remove_dir_all(&self.scratch.remote).unwrap();
        git_fails(
            &self.scratch.repo.path,
            &["ls-remote", "origin", "refs/sluice/*"],
        )
    }
}

fn main_branch() -> Branch {
    Branch::new(MAIN)
}

/// Where a verdict was found: its path in the signer's ref.
fn path_of(verdict: &Verdict) -> &str {
    verdict.found.split_once(':').unwrap().1
}

#[test]
fn the_landing_group_over_sluice() {
    let report = run(landing::GROUP, &landing::scenarios(), &|| {
        Box::new(SluiceRig::new())
    });
    assert_eq!(report.profile, "sluice");
    assert!(report.not_applicable.is_empty(), "{report:?}");
    report.assert();
}

#[test]
fn each_commit_of_a_landing_of_two_entries() {
    let rig = SluiceRig::new();
    let landed = rig.recorder.land(2, true);
    assert_eq!(landed.commits.len(), 2);
    for commit in &landed.commits {
        assert_eq!(
            rig.landing
                .landed(&main_branch(), &Revision::new(commit))
                .map(|l| l.landed),
            Ok(true),
            "{commit}"
        );
    }
}

#[test]
fn a_commit_sluice_did_not_land() {
    let rig = SluiceRig::new();
    let before = Revision::new(rig.scratch.repo.head());
    rig.recorder.land(1, true);
    assert_eq!(
        rig.landing
            .landed(&main_branch(), &before)
            .map(|l| l.landed),
        Ok(false)
    );
}

#[test]
fn a_landing_begun_and_not_finished() {
    let rig = SluiceRig::new();
    let done = rig.recorder.land(1, true);
    let begun = rig.recorder.land(1, false);
    assert_eq!(
        rig.landing
            .landed(&main_branch(), &Revision::new(&begun.commits[0]))
            .map(|l| l.landed),
        Ok(false)
    );
    assert_eq!(
        rig.landing
            .landed(&main_branch(), &Revision::new(&done.commits[0]))
            .map(|l| l.landed),
        Ok(true)
    );
}

#[test]
fn a_branch_on_one_host() {
    let scratch = Scratch::new();
    let recorder = Recorder::new(&scratch.repo, false, &scratch.path().join("keys"));
    let landed = recorder.land(1, true);
    let landing = Sluice::new(&scratch.repo.path, None);
    let revision = Revision::new(&landed.commits[0]);
    assert_eq!(
        landing.landed(&main_branch(), &revision).map(|l| l.landed),
        Ok(true)
    );
    assert_eq!(
        landing.verdicts_of(&revision).map(|v| v.verdicts),
        Ok(vec![])
    );
}

#[test]
fn two_checks_on_one_record() {
    let rig = SluiceRig::new();
    let landed = rig.recorder.land(1, true);
    let (path, bytes) = rig.recorder.record(
        "box",
        &landed.combination,
        SYSTEM,
        &[
            Ran::new("app:test", "4863a9af", "pass"),
            Ran::new("app:lint", "6de6b709", "fail"),
        ],
    );
    let refs = rig.scratch.repo.git(&["for-each-ref"]);
    let verdicts = rig
        .landing
        .verdicts_of(&Revision::new(&landed.commits[0]))
        .unwrap()
        .verdicts;
    assert_eq!(
        rig.landing
            .landed(&main_branch(), &Revision::new(&landed.commits[0]))
            .map(|l| l.landed),
        Ok(true)
    );
    assert_eq!(
        rig.scratch.repo.git(&["for-each-ref"]),
        refs,
        "the landing moved a ref"
    );
    let read: Vec<_> = verdicts
        .iter()
        .map(|v| {
            (
                v.check.as_str(),
                v.build.as_str(),
                v.outcome,
                v.signer.as_str(),
            )
        })
        .collect();
    assert_eq!(
        read,
        [
            ("app:test", "4863a9af", Outcome::Pass, "box"),
            ("app:lint", "6de6b709", Outcome::Fail, "box"),
        ]
    );
    for verdict in &verdicts {
        assert_eq!(as_written(&verdict.envelope), bytes);
        assert_eq!(path_of(verdict), path);
    }
}

#[test]
fn two_signers() {
    let rig = SluiceRig::new();
    let landed = rig.recorder.land(1, true);
    for signer in ["box", "forge"] {
        rig.recorder.record(
            signer,
            &landed.combination,
            SYSTEM,
            &[Ran::new("app:test", "4863a9af", "pass")],
        );
    }
    let verdicts = rig
        .landing
        .verdicts_of(&Revision::new(&landed.commits[0]))
        .unwrap()
        .verdicts;
    let mut by: Vec<&str> = verdicts.iter().map(|v| v.signer.as_str()).collect();
    by.sort_unstable();
    assert_eq!(by, ["box", "forge"]);
    for verdict in &verdicts {
        assert!(verdict
            .found
            .starts_with(&format!("origin refs/sluice/runs/{}:", verdict.signer)));
        assert!(verdict
            .envelope
            .signatures
            .iter()
            .all(|s| s.signer == verdict.signer));
        assert_eq!(
            verdict.envelope.signatures[0].keyid,
            rig.recorder.key(verdict.signer.as_str()).1
        );
    }
}

#[test]
fn a_replay_of_an_earlier_landings_pass() {
    let rig = SluiceRig::new();
    let earlier = rig.recorder.land(1, true);
    let (ran, ran_bytes) = rig.recorder.record(
        "box",
        &earlier.combination,
        SYSTEM,
        &[Ran::new("app:test", "4863a9af", "pass")],
    );
    let later = rig.recorder.land(1, true);
    let (_, replay_bytes) = rig.recorder.record(
        "box",
        &later.combination,
        SYSTEM,
        &[Ran::replayed("app:test", "4863a9af")],
    );
    let verdicts = rig
        .landing
        .verdicts_of(&Revision::new(&later.commits[0]))
        .unwrap()
        .verdicts;
    assert_eq!(verdicts.len(), 1, "{verdicts:?}");
    let carried = &verdicts[0];
    assert_eq!(
        (
            carried.check.as_str(),
            carried.build.as_str(),
            carried.signer.as_str()
        ),
        ("app:test", "4863a9af", "box")
    );
    assert_eq!(carried.outcome, Outcome::Pass);
    assert_eq!(as_written(&carried.envelope), ran_bytes);
    assert_ne!(as_written(&carried.envelope), replay_bytes);
    assert_eq!(carried.found, format!("origin refs/sluice/runs/box:{ran}"));
}

#[test]
fn a_pass_on_another_system_does_not_stand_in() {
    let rig = SluiceRig::new();
    let earlier = rig.recorder.land(1, true);
    rig.recorder.record(
        "box",
        &earlier.combination,
        "macos-arm64",
        &[Ran::new("app:test", "4863a9af", "pass")],
    );
    let later = rig.recorder.land(1, true);
    rig.recorder.record(
        "box",
        &later.combination,
        SYSTEM,
        &[Ran::replayed("app:test", "4863a9af")],
    );
    assert_eq!(
        rig.landing
            .verdicts_of(&Revision::new(&later.commits[0]))
            .map(|v| v.verdicts),
        Ok(vec![])
    );
}

#[test]
fn a_malformed_record() {
    let rig = SluiceRig::new();
    let landed = rig.recorder.land(1, true);
    let path = format!("{}/{SYSTEM}/0123456789abcdef.json", landed.combination);
    rig.recorder.add("box", &path, b"a record changed by hand");
    match rig
        .landing
        .verdicts_of(&Revision::new(&landed.commits[0]))
        .map(|v| v.verdicts)
    {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, *rig.landing.source());
            assert_eq!(u.what, format!("origin refs/sluice/runs/box:{path}"));
            assert!(u
                .said
                .is_some_and(|s| s.starts_with("the record is not a DSSE envelope")));
        }
        other => panic!("a malformed record answered {other:?}"),
    }
}

#[test]
fn the_envelope_as_sluice_wrote_it() {
    let rig = SluiceRig::new();
    let landed = rig.recorder.land(1, true);
    let (path, bytes) = rig.recorder.record(
        "box",
        &landed.combination,
        SYSTEM,
        &[Ran::new("app:test", "4863a9af", "pass")],
    );
    let [verdict] = <[Verdict; 1]>::try_from(
        rig.fresh()
            .verdicts_of(&Revision::new(&landed.commits[0]))
            .unwrap()
            .verdicts,
    )
    .unwrap();
    let held = remote_file(&rig.scratch.remote, "refs/sluice/runs/box", &path);
    assert_eq!(held, bytes);
    assert_eq!(as_written(&verdict.envelope), held);
}

#[test]
fn the_remote_is_gone() {
    let rig = SluiceRig::new();
    let landed = rig.recorder.land(1, true);
    let revision = Revision::new(&landed.commits[0]);
    let words = rig.unreachable();
    assert!(
        words.contains("does not appear to be a git repository"),
        "{words}"
    );
    for answer in [
        rig.landing.landed(&main_branch(), &revision).map(|_| ()),
        rig.landing.verdicts_of(&revision).map(|_| ()),
    ] {
        match answer {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, *rig.landing.source());
                assert_eq!(u.said.as_deref(), Some(words.as_str()));
            }
            other => panic!("answered {other:?} with the remote gone"),
        }
    }
}

/// A hook the scratch git runs, as a gate's fetch runs sluice's own, and
/// a profile's tools, which run in the test's own process, run under the
/// scratch home: none of the host's, and no sluice configuration in it.
#[test]
fn a_hook_and_the_tests_own_process_run_under_the_scratch_home() {
    let scratch = Scratch::new();
    let seen = scratch.path().join("seen");
    let hooks = scratch.repo.path.join(".git/hooks");
    fs::create_dir_all(&hooks).unwrap();
    let hook = hooks.join("post-commit");
    let printed: Vec<String> = HOME_ENV.iter().map(|(n, _)| format!("\"${n}\"")).collect();
    fs::write(
        &hook,
        format!(
            "#!/bin/sh\nprintf '%s\\n' {} > {:?}\n",
            printed.join(" "),
            seen.display().to_string()
        ),
    )
    .unwrap();
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
    scratch.repo.commit("chore: a commit the hook sees");
    let seen = fs::read_to_string(&seen).expect("the hook ran");
    let wanted: Vec<PathBuf> = HOME_ENV.iter().map(|(_, d)| home().join(d)).collect();
    let hooked: Vec<PathBuf> = seen.lines().map(PathBuf::from).collect();
    assert_eq!(hooked, wanted);
    let own: Vec<PathBuf> = HOME_ENV
        .iter()
        .map(|(n, _)| PathBuf::from(std::env::var_os(n).expect("set")))
        .collect();
    assert_eq!(own, wanted);
    assert!(wanted.iter().all(|d| d.is_dir()), "{wanted:?}");
    assert!(!home().join("config/sluice/config.toml").exists());
}

/// sluice itself lands a submission whose gate requires one moon task,
/// signing as `box`; the landing reads it.
#[test]
fn a_real_landing() {
    let scratch = Scratch::new();
    moon_workspace(
        &scratch.repo,
        &[Task::new("app", "test", "app.txt", true, 0)],
    );
    scratch.repo.commit("feat: the app and its test");
    scratch.repo.push();
    let recorder = Recorder::new(&scratch.repo, true, &scratch.path().join("keys"));
    let gate = Gate::new(&scratch, &recorder, &["app:test"]);
    let landed = Revision::new(gate.land("x", &[("app/x.txt", "x")]));

    let landing = Sluice::new(&scratch.repo.path, Some("origin".into()));
    assert_eq!(
        landing.landed(&main_branch(), &landed).map(|l| l.landed),
        Ok(true)
    );
    let [verdict] =
        <[Verdict; 1]>::try_from(landing.verdicts_of(&landed).unwrap().verdicts).unwrap();
    assert_eq!(verdict.check.as_str(), "app:test");
    assert_eq!(verdict.signer.as_str(), SIGNER);
    assert_eq!(verdict.outcome, Outcome::Pass);
    let at = scratch.clone_as("at-the-landing");
    at.git(&["checkout", "-q", "--detach", landed.as_str()]);
    assert_eq!(verdict.build.as_str(), moon_hash(&at.path, "app:test"));
    assert!(verdict.found.starts_with("origin refs/sluice/runs/box:"));
    let held = remote_file(&scratch.remote, "refs/sluice/runs/box", path_of(&verdict));
    assert_eq!(as_written(&verdict.envelope), held);
    assert_eq!(verdict.envelope.signatures[0].keyid, recorder.key(SIGNER).1);
}
