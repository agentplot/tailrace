//! Spec branch-landing: the landing seam bound to a plain git branch,
//! each test on a scratch repository of its own, and the suite's landing
//! group over it (design D9). Needs git alone, so it is in the fast
//! layer; S13 over a plain branch and moon is in the tools layer
//! (`landings_end_to_end.rs`).

mod scratch;

use std::fs;

use scratch::{git_fails, Scratch, MAIN};
use tailrace::profiles::landing::branch::PlainBranch;
use tailrace_conformance::seams::landing::{self, LandingRig, Recorded};
use tailrace_conformance::{run, Rig};
use tailrace_seams::{Branch, Landing, Not, Revision};

/// A scratch repository and the plain branch landing over it.
struct BranchRig {
    scratch: Scratch,
    landing: PlainBranch,
}

impl BranchRig {
    fn new() -> BranchRig {
        let scratch = Scratch::new();
        let landing = PlainBranch::new(&scratch.repo.path, None);
        BranchRig { scratch, landing }
    }
}

impl Rig for BranchRig {
    fn profile(&self) -> String {
        "branch".into()
    }
}

impl LandingRig for BranchRig {
    fn landing(&self) -> &dyn Landing {
        &self.landing
    }

    fn branch(&self) -> Branch {
        Branch::new(MAIN)
    }

    fn records_verdicts(&self) -> bool {
        false
    }

    /// A commit on `main`; a plain branch records no verdicts, so the
    /// suite hands it none.
    fn landed(&self, verdicts: &[Recorded]) -> Revision {
        assert!(verdicts.is_empty(), "a plain branch records no verdicts");
        let repo = &self.scratch.repo;
        repo.write("landed.txt", &repo.head());
        Revision::new(repo.commit("feat: landed"))
    }

    /// A commit on another branch only.
    fn not_landed(&self) -> Revision {
        let repo = &self.scratch.repo;
        repo.git(&["checkout", "-q", "-b", "elsewhere"]);
        repo.write("elsewhere.txt", "elsewhere");
        let commit = repo.commit("feat: elsewhere");
        repo.git(&["checkout", "-q", MAIN]);
        Revision::new(commit)
    }

    /// The repository removed, and git's words for reading it.
    fn unreachable(&self) -> String {
        let repo = &self.scratch.repo.path;
        fs::remove_dir_all(repo).unwrap();
        git_fails(repo, &["rev-parse", "--git-common-dir"])
    }
}

fn main_branch() -> Branch {
    Branch::new(MAIN)
}

#[test]
fn the_landing_group_over_a_branch() {
    let report = run(landing::GROUP, &landing::scenarios(), &|| {
        Box::new(BranchRig::new())
    });
    assert_eq!(report.profile, "branch");
    assert!(
        report
            .not_applicable
            .iter()
            .any(|n| n.scenario == "verdicts as the landing recorded them"),
        "{report:?}"
    );
    for skipped in &report.not_applicable {
        assert_eq!(skipped.why, "the landing records no verdicts", "{report:?}");
    }
    report.assert();
}

#[test]
fn landed_and_not() {
    let rig = BranchRig::new();
    let ancestor = Revision::new(rig.scratch.repo.head());
    let head = rig.landed(&[]);
    let elsewhere = rig.not_landed();
    let landed = |revision: &Revision| {
        rig.landing
            .landed(&main_branch(), revision)
            .map(|l| l.landed)
    };
    assert_eq!(landed(&head), Ok(true));
    assert_eq!(landed(&ancestor), Ok(true));
    assert_eq!(landed(&elsewhere), Ok(false));
    assert_eq!(
        rig.landing
            .landed(&Branch::new("no-such-branch"), &head)
            .map(|l| l.landed),
        Ok(false)
    );
}

#[test]
fn the_remotes_branch() {
    let scratch = Scratch::new();
    let other = scratch.clone_as("other");
    other.write("pushed.txt", "pushed");
    let pushed = Revision::new(other.commit("feat: pushed from another clone"));
    other.push();
    let refs = scratch.repo.git(&["for-each-ref"]);
    let local = PlainBranch::new(&scratch.repo.path, None);
    assert_eq!(
        local.landed(&main_branch(), &pushed).map(|l| l.landed),
        Ok(false)
    );
    let remote = PlainBranch::new(&scratch.repo.path, Some("origin".into()));
    assert_eq!(
        remote.landed(&main_branch(), &pushed).map(|l| l.landed),
        Ok(true)
    );
    assert_eq!(
        scratch.repo.git(&["for-each-ref"]),
        refs,
        "the landing moved a ref"
    );
}

#[test]
fn a_landed_revisions_verdicts() {
    let rig = BranchRig::new();
    let landed = rig.landed(&[]);
    assert_eq!(
        rig.landing.verdicts_of(&landed).map(|v| v.verdicts),
        Ok(vec![])
    );
    let missing = Revision::new("0000000000000000000000000000000000000001");
    assert!(matches!(
        rig.landing.verdicts_of(&missing).map(|v| v.verdicts),
        Err(Not::Absent(a)) if a.what.contains(missing.as_str())
    ));
}

#[test]
fn the_repository_is_gone() {
    let rig = BranchRig::new();
    let landed = rig.landed(&[]);
    let words = rig.unreachable();
    assert!(words.starts_with("fatal: cannot change to"), "{words}");
    for answer in [
        rig.landing.landed(&main_branch(), &landed).map(|_| ()),
        rig.landing.verdicts_of(&landed).map(|_| ()),
    ] {
        match answer {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, *rig.landing.source());
                assert_eq!(u.said.as_deref(), Some(words.as_str()));
            }
            other => panic!("answered {other:?} with the repository gone"),
        }
    }
}

#[test]
fn the_bound_remote_is_gone() {
    let scratch = Scratch::new();
    let landing = PlainBranch::new(&scratch.repo.path, Some("origin".into()));
    let landed = Revision::new(scratch.repo.head());
    assert_eq!(
        landing.landed(&main_branch(), &landed).map(|l| l.landed),
        Ok(true)
    );
    fs::remove_dir_all(&scratch.remote).unwrap();
    let words = git_fails(&scratch.repo.path, &["ls-remote", "origin", "refs/heads/*"]);
    assert!(
        words.contains("does not appear to be a git repository"),
        "{words}"
    );
    for answer in [
        landing.landed(&main_branch(), &landed).map(|_| ()),
        landing.verdicts_of(&landed).map(|_| ()),
    ] {
        match answer {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, *landing.source());
                assert_eq!(u.said.as_deref(), Some(words.as_str()));
            }
            other => panic!("answered {other:?} with the remote gone"),
        }
    }
}
