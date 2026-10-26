//! Spec branch-landing: the landing seam bound to a plain git branch,
//! each test on a scratch repository of its own, and the suite's landing
//! group over it, with no remote bound and with one (design D9). Needs git alone, so it is in the fast
//! layer; S13 over a plain branch and moon is in the tools layer
//! (`landings_end_to_end.rs`).

mod scratch;

use std::fs;

use scratch::{git_fails, Scratch, MAIN};
use tailrace::profiles::landing::branch::PlainBranch;
use tailrace_conformance::seams::landing::{self, LandingRig, Recorded};
use tailrace_conformance::{run, Rig};
use tailrace_seams::{Branch, IsLanded, Landing, Not, Read, Revision, Verdicts};

/// A scratch repository and the plain branch landing over it, reading
/// the repository's own branches or, bound to it, the remote's.
struct BranchRig {
    scratch: Scratch,
    landing: PlainBranch,
    remote: bool,
}

impl BranchRig {
    fn new() -> BranchRig {
        let scratch = Scratch::new();
        let landing = PlainBranch::new(&scratch.repo.path, None);
        BranchRig {
            scratch,
            landing,
            remote: false,
        }
    }

    /// The landing bound to the scratch repository's remote, `origin`.
    fn on_a_remote() -> BranchRig {
        let scratch = Scratch::new();
        let landing = PlainBranch::new(&scratch.repo.path, Some("origin".into()));
        BranchRig {
            scratch,
            landing,
            remote: true,
        }
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

    fn reads_a_remote(&self) -> bool {
        self.remote
    }

    /// A commit on `main`, pushed to the remote where the landing reads
    /// one; a plain branch records no verdicts, so the suite hands it
    /// none.
    fn landed(&self, verdicts: &[Recorded]) -> Revision {
        assert!(verdicts.is_empty(), "a plain branch records no verdicts");
        let repo = &self.scratch.repo;
        repo.write("landed.txt", &repo.head());
        let landed = Revision::new(repo.commit("feat: landed"));
        if self.remote {
            repo.push();
        }
        landed
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

    /// The remote removed, and git's words for listing it, where the
    /// landing reads one; otherwise the repository removed, and git's
    /// words for reading it.
    fn unreachable(&self) -> String {
        let repo = &self.scratch.repo.path;
        if self.remote {
            fs::remove_dir_all(&self.scratch.remote).unwrap();
            return git_fails(repo, &["ls-remote", "origin", "refs/heads/*"]);
        }
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
    let skipped: Vec<(&str, &str)> = report
        .not_applicable
        .iter()
        .map(|n| (n.scenario, n.why.as_str()))
        .collect();
    assert_eq!(
        skipped,
        [
            (
                "verdicts as the landing recorded them",
                "the landing records no verdicts"
            ),
            (
                "two signers, each by its own name",
                "the landing records no verdicts"
            ),
            (
                "a landing read as last fetched",
                "the landing reads no remote: the repository is the landing itself"
            ),
        ],
        "{report:?}"
    );
    report.assert();
}

#[test]
fn the_landing_group_over_a_branch_on_a_remote() {
    let report = run(landing::GROUP, &landing::scenarios(), &|| {
        Box::new(BranchRig::on_a_remote())
    });
    assert_eq!(report.profile, "branch");
    let skipped: Vec<(&str, &str)> = report
        .not_applicable
        .iter()
        .map(|n| (n.scenario, n.why.as_str()))
        .collect();
    assert_eq!(
        skipped,
        [
            (
                "verdicts as the landing recorded them",
                "the landing records no verdicts"
            ),
            (
                "two signers, each by its own name",
                "the landing records no verdicts"
            ),
        ],
        "{report:?}"
    );
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

/// The repository's refs but the landing's copy, each by the object it
/// names.
fn refs_but_the_copy(repo: &scratch::Repo) -> Vec<String> {
    repo.git(&["for-each-ref"])
        .lines()
        .filter(|line| !line.contains("refs/tailrace/landing/"))
        .map(str::to_string)
        .collect()
}

/// The copy the landing keeps of the remote's `main`.
const COPY_OF_MAIN: &str = "refs/tailrace/landing/heads/main";

/// When the copy of `main` was fetched, from its reflog, and how many
/// entries the reflog holds.
fn copied_at(repo: &scratch::Repo) -> (u64, usize) {
    let log = repo.git(&[
        "reflog",
        "show",
        "--date=unix",
        "--format=%gd",
        COPY_OF_MAIN,
    ]);
    let at = log
        .lines()
        .next()
        .and_then(|l| l.rsplit_once("@{"))
        .and_then(|(_, rest)| rest.strip_suffix('}'))
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("the copy's reflog says no time: {log}"));
    (at, log.lines().count())
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[test]
fn the_remotes_branch() {
    let scratch = Scratch::new();
    let other = scratch.clone_as("other");
    other.write("pushed.txt", "pushed");
    let pushed = Revision::new(other.commit("feat: pushed from another clone"));
    other.push();
    let refs = refs_but_the_copy(&scratch.repo);
    let local = PlainBranch::new(&scratch.repo.path, None);
    assert_eq!(
        local.landed(&main_branch(), &pushed),
        Ok(IsLanded {
            landed: false,
            read: Read::Source
        })
    );
    let remote = PlainBranch::new(&scratch.repo.path, Some("origin".into()));
    assert_eq!(
        remote.landed(&main_branch(), &pushed),
        Ok(IsLanded {
            landed: true,
            read: Read::Source
        })
    );
    assert_eq!(
        refs_but_the_copy(&scratch.repo),
        refs,
        "the landing moved a ref"
    );
}

#[test]
fn each_read_remakes_the_copy_and_writes_no_other_ref() {
    let scratch = Scratch::new();
    let landing = PlainBranch::new(&scratch.repo.path, Some("origin".into()));
    let refs = refs_but_the_copy(&scratch.repo);
    let head = Revision::new(scratch.repo.head());
    assert!(landing.landed(&main_branch(), &head).unwrap().landed);
    assert_eq!(
        scratch.repo.git(&["rev-parse", COPY_OF_MAIN]),
        head.as_str()
    );
    let (first, entries) = copied_at(&scratch.repo);
    assert_eq!(entries, 1);

    std::thread::sleep(std::time::Duration::from_millis(1_100));
    let other = scratch.clone_as("other");
    other.write("pushed.txt", "pushed");
    let pushed = other.commit("feat: pushed from another clone");
    other.push();
    assert!(
        landing
            .landed(&main_branch(), &Revision::new(&pushed))
            .unwrap()
            .landed
    );
    assert_eq!(scratch.repo.git(&["rev-parse", COPY_OF_MAIN]), pushed);
    let (second, entries) = copied_at(&scratch.repo);
    assert!(second > first, "the copy's time stayed at {first}");
    assert_eq!(
        entries, 1,
        "the copy's reflog holds more than its last fetch"
    );
    assert_eq!(
        refs_but_the_copy(&scratch.repo),
        refs,
        "the read moved a ref beside the copy"
    );
    assert!(
        !scratch.repo.path.join(".git/FETCH_HEAD").exists(),
        "the read wrote FETCH_HEAD"
    );
}

#[test]
fn a_branch_the_remote_no_longer_lists_leaves_the_copy() {
    let scratch = Scratch::new();
    scratch.repo.git(&["push", "-q", "origin", "main:next"]);
    let landing = PlainBranch::new(&scratch.repo.path, Some("origin".into()));
    let head = Revision::new(scratch.repo.head());
    let next = Branch::new("next");
    assert!(landing.landed(&next, &head).unwrap().landed);
    scratch.repo.git(&["push", "-q", "origin", ":next"]);
    assert_eq!(
        landing.landed(&next, &head),
        Ok(IsLanded {
            landed: false,
            read: Read::Source
        })
    );
    assert_eq!(
        scratch
            .repo
            .git(&["for-each-ref", "refs/tailrace/landing/heads/next"]),
        ""
    );
    fs::remove_dir_all(&scratch.remote).unwrap();
    assert!(
        matches!(landing.landed(&next, &head), Err(Not::Unreadable(_))),
        "a branch the copy does not hold was answered"
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

/// git's words for listing `origin`'s branches from `scratch`'s
/// repository, the remote gone.
fn remote_gone(scratch: &Scratch) -> String {
    fs::remove_dir_all(&scratch.remote).unwrap();
    let words = git_fails(&scratch.repo.path, &["ls-remote", "origin", "refs/heads/*"]);
    assert!(
        words.contains("does not appear to be a git repository"),
        "{words}"
    );
    words
}

#[test]
fn a_remote_never_fetched() {
    let scratch = Scratch::new();
    let landing = PlainBranch::new(&scratch.repo.path, Some("origin".into()));
    let landed = Revision::new(scratch.repo.head());
    let words = remote_gone(&scratch);
    for answer in [
        landing.landed(&main_branch(), &landed).map(|_| ()),
        landing.verdicts_of(&landed).map(|_| ()),
    ] {
        match answer {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, *landing.source());
                assert_eq!(u.said.as_deref(), Some(words.as_str()));
            }
            other => panic!("answered {other:?} with the remote never fetched"),
        }
    }
}

#[test]
fn landed_as_last_fetched() {
    let scratch = Scratch::new();
    let landing = PlainBranch::new(&scratch.repo.path, Some("origin".into()));
    let landed = Revision::new(scratch.repo.head());
    let before = now();
    assert!(landing.landed(&main_branch(), &landed).unwrap().landed);
    let words = remote_gone(&scratch);

    let Ok(IsLanded {
        landed: true,
        read: Read::Kept { at, said },
    }) = landing.landed(&main_branch(), &landed)
    else {
        panic!("a landed revision was not answered landed from the copy")
    };
    assert!(at >= before, "fetched at {at}, first read at {before}");
    assert_eq!(said, words);
    assert_eq!(
        landing.verdicts_of(&landed),
        Ok(Verdicts {
            verdicts: vec![],
            read: Read::Kept {
                at,
                said: words.clone()
            }
        })
    );
    let missing = Revision::new("0000000000000000000000000000000000000001");
    assert!(matches!(
        landing.verdicts_of(&missing),
        Err(Not::Absent(a)) if a.what.contains(missing.as_str())
    ));
    match landing.landed(&Branch::new("never-fetched"), &landed) {
        Err(Not::Unreadable(u)) => assert_eq!(u.said, Some(words)),
        other => panic!("a branch never fetched answered {other:?}"),
    }
}

#[test]
fn landed_on_the_remote_after_the_fetch() {
    let scratch = Scratch::new();
    let landing = PlainBranch::new(&scratch.repo.path, Some("origin".into()));
    let before = now();
    assert!(
        landing
            .landed(&main_branch(), &Revision::new(scratch.repo.head()))
            .unwrap()
            .landed
    );
    let after = now();
    scratch.repo.write("later.txt", "later");
    let later = Revision::new(scratch.repo.commit("feat: landed after the fetch"));
    scratch.repo.push();
    remote_gone(&scratch);

    let Ok(IsLanded {
        landed,
        read: Read::Kept { at, .. },
    }) = landing.landed(&main_branch(), &later)
    else {
        panic!("a revision landed after the fetch was not answered from the copy")
    };
    assert!(
        !landed,
        "a revision landed after the fetch was answered landed"
    );
    assert!((before..=after).contains(&at), "fetched at {at}");
}

#[test]
fn no_remote_bound() {
    let rig = BranchRig::new();
    let landed = rig.landed(&[]);
    assert_eq!(
        rig.landing.landed(&main_branch(), &landed),
        Ok(IsLanded {
            landed: true,
            read: Read::Source
        })
    );
    assert_eq!(
        rig.landing.verdicts_of(&landed),
        Ok(Verdicts {
            verdicts: vec![],
            read: Read::Source
        })
    );
}
