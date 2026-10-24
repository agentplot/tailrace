//! Spec moon-build: the build seam bound to moon, each test on a scratch
//! repository of its own, and the suite's build group over it (design
//! D9). Needs moon, so it is in the tools layer.

#![cfg(feature = "tools")]

mod scratch;

use std::fs;
use std::path::PathBuf;

use scratch::{moon_hash, moon_workspace, policy_file, Repo, Scratch, Task};
use tailrace::profiles::build::moon::Moon;
use tailrace_conformance::seams::build::{self, BuildRig, Made};
use tailrace_conformance::{run, Rig};
use tailrace_seams::{Build, Not, Outcome, OutputName, Revision, Seam, Why};

/// A scratch repository, a second clone of it, and moon over each, run
/// through a link the rig can take away.
struct MoonRig {
    scratch: Scratch,
    second: Repo,
    link: PathBuf,
    build: Moon,
    second_host: Moon,
}

impl MoonRig {
    fn new() -> MoonRig {
        let scratch = Scratch::new();
        let second = scratch.clone_as("second");
        let link = scratch.link("moon", "moon");
        let build = Moon::new(&scratch.repo.path).moon_program(&link);
        let second_host = Moon::new(&second.path).moon_program(&link);
        MoonRig {
            scratch,
            second,
            link,
            build,
            second_host,
        }
    }

    /// `tasks` and the policy file naming each output's task committed,
    /// pushed, and pulled into the second clone; answers the revision.
    fn commit(&self, tasks: &[Task], outputs: &[(&str, &str)]) -> Revision {
        let repo = &self.scratch.repo;
        moon_workspace(repo, tasks);
        policy_file(repo, outputs, "");
        let revision = repo.commit("feat: the outputs and their tasks");
        repo.push();
        self.second
            .git(&["pull", "-q", "--ff-only", "origin", "main"]);
        Revision::new(revision)
    }

    /// Where the checkout of `revision` is.
    fn checkout(&self, revision: &Revision) -> PathBuf {
        self.scratch
            .repo
            .path
            .join(".git/tailrace/builds")
            .join(revision.as_str())
    }

    /// The run report moon left in the checkout of `revision`.
    fn run_report(&self, revision: &Revision) -> serde_json::Value {
        serde_json::from_slice(
            &fs::read(self.checkout(revision).join(".moon/cache/runReport.json")).unwrap(),
        )
        .unwrap()
    }
}

impl Rig for MoonRig {
    fn profile(&self) -> String {
        "moon".into()
    }
}

impl BuildRig for MoonRig {
    fn build(&self) -> &dyn Build {
        &self.build
    }

    fn second_host(&self) -> &dyn Build {
        &self.second_host
    }

    /// One project per output, its task `make` leaving the output as
    /// `made` says, ensured here and in the second clone, so the build
    /// has left them.
    fn revision(&self, outputs: &[(OutputName, Made)]) -> Revision {
        let tasks: Vec<Task> = outputs
            .iter()
            .map(|(output, made)| {
                let (writes, exit) = match made {
                    Made::Pass => (true, 0),
                    Made::Fail => (true, 1),
                    Made::Unmade => (false, 0),
                };
                let name = output.as_str();
                Task::new(name, "make", &format!("{name}.txt"), writes, exit)
            })
            .collect();
        let targets: Vec<(String, String)> = tasks
            .iter()
            .map(|t| (t.project.clone(), t.target()))
            .collect();
        let named: Vec<(&str, &str)> = targets
            .iter()
            .map(|(o, t)| (o.as_str(), t.as_str()))
            .collect();
        let revision = self.commit(&tasks, &named);
        let asked: Vec<OutputName> = outputs.iter().map(|(o, _)| o.clone()).collect();
        let _ = self.build.ensure(&revision, &asked);
        let _ = self.second_host.ensure(&revision, &asked);
        revision
    }

    /// The link to moon removed, and the operating system's words for
    /// running it.
    fn unreachable(&self) -> String {
        fs::remove_file(&self.link).unwrap();
        std::process::Command::new(&self.link)
            .spawn()
            .unwrap_err()
            .to_string()
    }
}

fn output(name: &str) -> OutputName {
    OutputName::new(name)
}

#[test]
fn the_build_group_over_moon() {
    let report = run(build::GROUP, &build::scenarios(), &|| {
        Box::new(MoonRig::new())
    });
    assert_eq!(report.profile, "moon");
    assert!(report.not_applicable.is_empty(), "{report:?}");
    report.assert();
}

#[test]
fn an_output_and_its_task() {
    let rig = MoonRig::new();
    let revision = rig.commit(
        &[
            Task::new("web", "build", "site.txt", true, 0),
            Task::new("web", "other", "other.txt", true, 0),
        ],
        &[("site", "web:build")],
    );
    assert_eq!(rig.build.ensure(&revision, &[output("site")]), Ok(()));
    let ran: Vec<String> = rig.run_report(&revision)["actions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|a| a["label"].as_str())
        .filter(|l| l.starts_with("RunTask("))
        .map(str::to_string)
        .collect();
    assert_eq!(ran, ["RunTask(web:build)"]);
    let report = rig.build.report(&revision, &output("site")).unwrap();
    assert_eq!(
        report.build.as_str(),
        moon_hash(&rig.second.path, "web:build")
    );
    assert_eq!(report.outcome, Outcome::Pass);
    assert_eq!(
        report.files,
        [rig.checkout(&revision)
            .join("web/dist/site.txt")
            .display()
            .to_string()]
    );
}

#[test]
fn an_output_with_no_task() {
    let rig = MoonRig::new();
    let revision = rig.commit(
        &[Task::new("web", "build", "site.txt", true, 0)],
        &[("site", "web:build")],
    );
    assert_eq!(
        rig.build
            .ensure(&revision, &[output("site"), output("server")]),
        Err(Not::refused(
            rig.build.source(),
            Why::Unmade {
                outputs: vec![output("server")]
            },
            None
        ))
    );
}

#[test]
fn a_checkouts_edit_is_not_read() {
    let rig = MoonRig::new();
    let revision = rig.commit(
        &[
            Task::new("web", "build", "site.txt", true, 0),
            Task::new("web", "other", "other.txt", true, 0),
        ],
        &[("site", "web:build")],
    );
    policy_file(&rig.scratch.repo, &[("site", "web:other")], "");
    assert_eq!(rig.build.ensure(&revision, &[output("site")]), Ok(()));
    let report = rig.build.report(&revision, &output("site")).unwrap();
    assert_eq!(
        report.build.as_str(),
        moon_hash(&rig.second.path, "web:build")
    );
    assert!(!rig.checkout(&revision).join("web/dist/other.txt").exists());
}

#[test]
fn three_outputs_in_one_asking() {
    let rig = MoonRig::new();
    let revision = rig.commit(
        &[
            Task::new("web", "build", "site.txt", true, 0),
            Task::new("api", "build", "server.txt", true, 0),
            Task::new("app", "test", "checks.txt", true, 1),
        ],
        &[
            ("site", "web:build"),
            ("server", "api:build"),
            ("checks", "app:test"),
        ],
    );
    let asked = [output("site"), output("server"), output("checks")];
    assert_eq!(rig.build.ensure(&revision, &asked), Ok(()));
    // One run report, which ensure removes before its run, holds all
    // three: moon ran once over them.
    let mut ran: Vec<String> = rig.run_report(&revision)["actions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|a| a["label"].as_str())
        .filter(|l| l.starts_with("RunTask("))
        .map(str::to_string)
        .collect();
    ran.sort();
    assert_eq!(
        ran,
        [
            "RunTask(api:build)",
            "RunTask(app:test)",
            "RunTask(web:build)"
        ]
    );
    let outcomes: Vec<Outcome> = asked
        .iter()
        .map(|o| rig.build.report(&revision, o).unwrap().outcome)
        .collect();
    assert_eq!(outcomes, [Outcome::Pass, Outcome::Pass, Outcome::Fail]);
}

#[test]
fn the_same_revision_ensured_again() {
    let rig = MoonRig::new();
    let revision = rig.commit(
        &[
            Task::new("web", "build", "site.txt", true, 0),
            Task::new("api", "build", "server.txt", true, 0),
        ],
        &[("site", "web:build"), ("server", "api:build")],
    );
    let asked = [output("site"), output("server")];
    assert_eq!(rig.build.ensure(&revision, &asked), Ok(()));
    let first: Vec<_> = asked
        .iter()
        .map(|o| rig.build.report(&revision, o).unwrap().build)
        .collect();
    assert_eq!(rig.build.ensure(&revision, &asked), Ok(()));
    let statuses: Vec<String> = rig.run_report(&revision)["actions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["label"].as_str().unwrap().starts_with("RunTask("))
        .map(|a| a["status"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(statuses, ["cached", "cached"]);
    let again: Vec<_> = asked
        .iter()
        .map(|o| rig.build.report(&revision, o).unwrap().build)
        .collect();
    assert_eq!(again, first);
}

#[test]
fn a_failed_tasks_report() {
    let rig = MoonRig::new();
    let revision = rig.commit(
        &[Task::new("app", "test", "checks.txt", true, 1)],
        &[("checks", "app:test")],
    );
    assert_eq!(rig.build.ensure(&revision, &[output("checks")]), Ok(()));
    let report = rig.build.report(&revision, &output("checks")).unwrap();
    assert_eq!(
        report.build.as_str(),
        moon_hash(&rig.second.path, "app:test")
    );
    assert_eq!(report.outcome, Outcome::Fail);
    assert_eq!(
        report.files,
        [rig.checkout(&revision)
            .join("app/dist/checks.txt")
            .display()
            .to_string()]
    );
}

#[test]
fn nothing_ensured() {
    let rig = MoonRig::new();
    let revision = rig.commit(
        &[Task::new("web", "build", "site.txt", true, 0)],
        &[("site", "web:build")],
    );
    match rig.build.report(&revision, &output("site")) {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source.seam, Seam::Build);
            assert!(a.what.contains("site"), "{a:?}");
        }
        other => panic!("report of an output never ensured answered {other:?}"),
    }
}

#[test]
fn two_checkouts() {
    let rig = MoonRig::new();
    let revision = rig.commit(
        &[
            Task::new("web", "build", "site.txt", true, 0),
            Task::new("app", "test", "checks.txt", true, 1),
        ],
        &[("site", "web:build"), ("checks", "app:test")],
    );
    let asked = [output("site"), output("checks")];
    assert_eq!(rig.build.ensure(&revision, &asked), Ok(()));
    assert_eq!(rig.second_host.ensure(&revision, &asked), Ok(()));
    for o in &asked {
        let here = rig.build.report(&revision, o).unwrap();
        let there = rig.second_host.report(&revision, o).unwrap();
        assert_eq!(here.build, there.build, "{o} came from two builds");
        assert_ne!(here.files, there.files, "{o} was built in one checkout");
    }
}

#[test]
fn moon_not_there() {
    let rig = MoonRig::new();
    let revision = rig.commit(
        &[Task::new("web", "build", "site.txt", true, 0)],
        &[("site", "web:build")],
    );
    assert_eq!(rig.build.ensure(&revision, &[output("site")]), Ok(()));
    let never_ensured = rig.commit(
        &[Task::new("web", "build", "site.txt", true, 0)],
        &[("site", "web:build")],
    );
    let words = rig.unreachable();
    assert_eq!(words, "No such file or directory (os error 2)");
    for answer in [
        rig.build.ensure(&revision, &[output("site")]).map(|_| ()),
        rig.build.report(&revision, &output("site")).map(|_| ()),
        rig.build
            .report(&never_ensured, &output("site"))
            .map(|_| ()),
        rig.build
            .ensure(&revision, &[output("no-task")])
            .map(|_| ()),
        rig.build.report(&revision, &output("no-task")).map(|_| ()),
    ] {
        match answer {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, *rig.build.source());
                assert_eq!(u.said.as_deref(), Some(words.as_str()));
            }
            other => panic!("answered {other:?} with moon gone"),
        }
    }
}
