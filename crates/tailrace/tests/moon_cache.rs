//! Specs moon-cache-projection and cache-entries: a task moon ran in one
//! checkout replays from the store in a fresh clone of the same
//! revision, its archive packed by the moon cache, written as a cache
//! entry to an OCI layout store, and rendered by sync from a landing
//! whose verdict names its task hash (design D9). Needs moon, so it is
//! in the tools layer.

#![cfg(feature = "tools")]

mod scratch;

use std::fs;
use std::path::{Path, PathBuf};

use scratch::{moon_hash, moon_workspace, Repo, Scratch, Task, MAIN};
use serde_json::json;
use tailrace::profiles::projection::moon_cache::{MoonCache, ECOSYSTEM};
use tailrace::profiles::store::layout::Layout;
use tailrace_core::{sync, write, SyncAsk};
use tailrace_fakes::{FakeSigner, Fakes};
use tailrace_seams::{
    Branch, BuildId, Check, Claim, ClaimKind, Digest, Ecosystem, Outcome, Output, OutputName,
    Revision, Seams, Signer, SignerId, Verdict,
};

/// The task the test runs, and the file it leaves.
const TARGET: &str = "site:make";
const OUTPUT: &str = "site/dist/site.txt";

/// A scratch repository whose one task moon has run, the task's hash,
/// the revision it ran at, and a layout store beside them.
struct Ran {
    scratch: Scratch,
    hash: BuildId,
    revision: Revision,
    layout: PathBuf,
}

fn ran() -> Ran {
    let scratch = Scratch::new();
    moon_workspace(
        &scratch.repo,
        &[Task::new("site", "make", "site.txt", true, 0)],
    );
    let revision = Revision::new(scratch.repo.commit("feat: the site and its task"));
    scratch.repo.push();
    let hash = BuildId::new(moon_hash(&scratch.repo.path, TARGET));
    let layout = scratch.path().join("layout");
    Layout::create(&layout).unwrap();
    Ran {
        scratch,
        hash,
        revision,
        layout,
    }
}

/// The fakes bound with the layout store and the moon cache of
/// `checkout` alone, `revision` landed on `main` with a passing verdict
/// naming `hash`.
fn seams(layout: &Path, checkout: &Path, revision: &Revision, hash: &BuildId) -> Seams {
    let fakes = Fakes::new();
    let main = Branch::new(MAIN);
    fakes.landing.land(&main, revision);
    let id = SignerId::new("landing");
    let theirs = FakeSigner::new();
    theirs.hold(&id);
    let claim = Claim {
        subject: Digest::of(hash.as_str().as_bytes()),
        kind: ClaimKind::Verdict,
        revision: revision.clone(),
        build: Some(hash.clone()),
        body: json!({ "check": "test", "outcome": Outcome::Pass }),
    };
    fakes.landing.verdict(
        revision,
        Verdict {
            check: Check::new("test"),
            build: hash.clone(),
            outcome: Outcome::Pass,
            signer: id.clone(),
            envelope: theirs.sign(&claim, &id).unwrap(),
            found: format!("the fake landing's record of test on {hash}"),
        },
    );
    let mut seams = fakes.seams();
    seams.store = Box::new(Layout::open(layout).unwrap());
    seams.projections.insert(
        Ecosystem::new(ECOSYSTEM),
        Box::new(MoonCache::new(
            checkout,
            Box::new(Layout::open(layout).unwrap()),
        )),
    );
    seams
}

/// The status moon gave `target` in the run report it left in `dir`.
fn status(dir: &Path, target: &str) -> String {
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join(".moon/cache/runReport.json")).unwrap()).unwrap();
    report["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["label"] == format!("RunTask({target})"))
        .and_then(|a| a["status"].as_str())
        .unwrap_or_else(|| panic!("moon's run report holds no run of {target}: {report}"))
        .to_string()
}

/// The task's archive written from the first checkout into the layout,
/// and the fresh clone `name` synced from it, its outputs directory made
/// by `prepare` first; answers the clone.
fn synced_into(ran: &Ran, name: &str, prepare: impl Fn(&Repo)) -> Repo {
    let first = &ran.scratch.repo.path;
    let output = Output {
        name: OutputName::new("site"),
        files: vec![],
    };
    let written = write(
        &seams(&ran.layout, first, &ran.revision, &ran.hash),
        &output,
        &ran.hash,
    )
    .unwrap();
    assert_eq!(written.projection, Ecosystem::new(ECOSYSTEM));

    let fresh = ran.scratch.clone_as(name);
    assert!(
        !fresh.path.join(".moon/cache").exists(),
        "the fresh clone holds moon's cache"
    );
    prepare(&fresh);
    let ask = SyncAsk {
        branch: Branch::new(MAIN),
        revision: ran.revision.clone(),
        at: fresh.path.display().to_string(),
    };
    let synced = sync(
        &seams(&ran.layout, &fresh.path, &ran.revision, &ran.hash),
        &ask,
    )
    .unwrap();
    let [(build, filled)] = synced.entries.as_slice() else {
        panic!("the sync named {} entries", synced.entries.len())
    };
    assert_eq!(build, &ran.hash);
    let filled = filled.as_ref().unwrap();
    assert_eq!(filled.digest, written.entry.digest);
    assert_eq!(filled.rendered, [Ecosystem::new(ECOSYSTEM)]);
    fresh
}

/// moon run in `fresh` replays the task: reported cached, at the hash it
/// ran at in the first checkout, its output the first checkout's.
fn replays(ran: &Ran, fresh: &Repo) {
    assert!(
        !fresh.path.join(OUTPUT).exists(),
        "the clone holds the output"
    );
    assert_eq!(moon_hash(&fresh.path, TARGET), ran.hash.as_str());
    assert_eq!(status(&fresh.path, TARGET), "cached");
    assert_eq!(
        fs::read(fresh.path.join(OUTPUT)).unwrap(),
        fs::read(ran.scratch.repo.path.join(OUTPUT)).unwrap()
    );
}

#[test]
fn a_task_that_ran_in_one_checkout_replays_in_a_fresh_clone() {
    let ran = ran();
    let fresh = synced_into(&ran, "fresh", |_| {});
    let archive = fresh
        .path
        .join(".moon/cache/outputs")
        .join(format!("{}.tar.gz", ran.hash));
    assert_eq!(
        fs::read(&archive).unwrap(),
        fs::read(
            ran.scratch
                .repo
                .path
                .join(".moon/cache/outputs")
                .join(format!("{}.tar.gz", ran.hash))
        )
        .unwrap()
    );
    replays(&ran, &fresh);
}

#[test]
fn a_task_replays_in_a_clone_whose_outputs_directory_is_a_link() {
    let ran = ran();
    let shared = ran.scratch.path().join("shared-outputs");
    fs::create_dir_all(&shared).unwrap();
    let fresh = synced_into(&ran, "linked", |clone| {
        fs::create_dir_all(clone.path.join(".moon/cache")).unwrap();
        std::os::unix::fs::symlink(&shared, clone.path.join(".moon/cache/outputs")).unwrap();
    });
    assert!(
        shared.join(format!("{}.tar.gz", ran.hash)).is_file(),
        "the archive is not in the shared directory"
    );
    replays(&ran, &fresh);
    let link = fresh.path.join(".moon/cache/outputs");
    assert!(fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(fs::read_link(&link).unwrap(), shared);
}

#[test]
fn a_clone_with_no_entry_runs_the_task() {
    let ran = ran();
    let fresh = ran.scratch.clone_as("unsynced");
    assert_eq!(moon_hash(&fresh.path, TARGET), ran.hash.as_str());
    assert_eq!(status(&fresh.path, TARGET), "passed");
}
