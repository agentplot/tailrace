//! Spec cache-entries: a cache entry written to the store under its
//! build identity, never an artifact, over the fakes, and the revision
//! a checkout syncs by default (design D1 to D4).

mod common;

use common::Fixture;
use serde_json::json;
use tailrace_core::cache::Written;
use tailrace_core::{answer_json, cache_name, sync, write, Filled, SyncAsk, Synced, BOUND};
use tailrace_fakes::{Call, FakeSigner, Fakes, Fault, LandingOp, ProjectionOp, StoreOp, Times};
use tailrace_seams::{
    Answer, ArtifactMeta, Branch, BuildId, Check, Claim, ClaimKind, Contents, Digest, Ecosystem,
    EcosystemName, Held, Layout, Name, Not, Outcome, Output, OutputName, Packed, Projection,
    Rendered, Revision, Seam, Serving, Signer, SignerId, Store, Verdict, Version, Why,
};

fn site() -> Output {
    Output {
        name: OutputName::new("site"),
        files: vec!["dist/site".into()],
    }
}

fn b7e1() -> BuildId {
    BuildId::new("b7e1")
}

/// The fakes with two projections bound, both packing cache entries.
fn fakes() -> Fakes {
    let mut fakes = Fakes::new();
    fakes.projection(&Ecosystem::new("eco-a"));
    fakes.projection(&Ecosystem::new("eco-b"));
    fakes
}

fn store_ops(fakes: &Fakes) -> Vec<StoreOp> {
    fakes.store.calls().into_iter().map(|c| c.op).collect()
}

fn written(answer: Answer<Written>) -> Written {
    answer.unwrap_or_else(|not| panic!("the write answered {not:?}"))
}

// Requirement: A cache entry is written under its build identity

#[test]
fn a_write_puts_the_bytes_and_makes_the_cache_name_from_nothing() {
    let fakes = fakes();
    let answer = written(write(&fakes.seams(), &site(), &b7e1()));

    let packed = fakes.projections[&Ecosystem::new("eco-a")]
        .pack(&site(), &b7e1(), None)
        .unwrap();
    let digest = Digest::of(&packed.bytes);
    assert_eq!(answer.entry.build, b7e1());
    assert_eq!(answer.entry.digest, digest);
    assert_eq!(answer.projection, Ecosystem::new("eco-a"));
    assert_eq!(answer.held_at, None);
    assert!(!answer.bytes_held_already);

    assert_eq!(cache_name(&b7e1()), Name::new("cache/b7e1"));
    let contents = fakes.store.contents();
    assert_eq!(
        contents.names.into_iter().collect::<Vec<_>>(),
        [(Name::new("cache/b7e1"), digest.clone())]
    );
    assert_eq!(contents.held.get(&digest), Some(&Held::Bytes(packed.bytes)));
    assert!(contents.attached.is_empty(), "the write attached a claim");
    assert_eq!(store_ops(&fakes), [StoreOp::PutBytes, StoreOp::Name]);
    let name = &fakes.store.calls()[1].given;
    assert_eq!(name["name"], "cache/b7e1");
    assert_eq!(name["expected"], serde_json::Value::Null);

    let eco_b = &fakes.projections[&Ecosystem::new("eco-b")];
    assert_eq!(eco_b.calls(), [], "a second projection was asked");
    let asked = &fakes.projections[&Ecosystem::new("eco-a")].calls()[0].given;
    assert_eq!(asked["version"], serde_json::Value::Null);
    assert_eq!(asked["build"], "b7e1");
}

#[test]
fn the_first_projection_that_packs_cache_entries_packs_the_entry() {
    let fakes = fakes();
    fakes.projections[&Ecosystem::new("eco-a")].packs_no_cache_entries();
    let answer = written(write(&fakes.seams(), &site(), &b7e1()));
    assert_eq!(answer.projection, Ecosystem::new("eco-b"));
}

#[test]
fn with_no_projection_that_packs_cache_entries_nothing_is_written() {
    let fakes = fakes();
    for projection in fakes.projections.values() {
        projection.packs_no_cache_entries();
    }
    match write(&fakes.seams(), &site(), &b7e1()) {
        Err(Not::Absent(a)) => {
            assert_eq!(a.what, "cache entries");
            assert_eq!(a.source.seam, Seam::Projection);
        }
        other => panic!("answered {other:?}"),
    }
    assert_eq!(store_ops(&fakes), [], "the store was asked something");
    for projection in fakes.projections.values() {
        assert_eq!(projection.calls().len(), 1, "each projection is asked once");
    }
}

/// A projection that answers metadata to a pack with no version.
struct Offering;

impl Projection for Offering {
    fn pack(&self, output: &Output, _: &BuildId, _: Option<&Version>) -> Answer<Packed> {
        Ok(Packed {
            bytes: output.name.as_str().as_bytes().to_vec(),
            meta: Some(
                ArtifactMeta::new(
                    Ecosystem::new("offering"),
                    EcosystemName::new("site"),
                    Version::new("1.0.0"),
                )
                .unwrap(),
            ),
            same_again: true,
        })
    }

    fn render(&self, _: Contents<'_>, _: &str) -> Answer<Rendered> {
        unreachable!("a write renders nothing")
    }

    fn serve(&self, _: &Layout) -> Answer<Serving> {
        unreachable!("a write serves nothing")
    }
}

#[test]
fn a_pack_with_no_version_that_answers_metadata_is_refused_before_any_put() {
    let fakes = Fakes::new();
    let mut seams = fakes.seams();
    seams
        .projections
        .insert(Ecosystem::new("offering"), Box::new(Offering));
    match write(&seams, &site(), &b7e1()) {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source.seam, Seam::Projection);
            assert_eq!(r.source.profile, "offering");
            assert_eq!(r.why, Why::Said);
            let said = r.said.unwrap();
            assert!(said.contains("names metadata"), "{said}");
        }
        other => panic!("answered {other:?}"),
    }
    assert_eq!(store_ops(&fakes), [], "the store was asked something");
}

// Requirement: An entry held already is answered, never written over

#[test]
fn the_same_entry_written_twice_is_held_already_and_held_once() {
    let fakes = fakes();
    let first = written(write(&fakes.seams(), &site(), &b7e1()));
    let held = fakes.store.contents();
    let second = written(write(&fakes.seams(), &site(), &b7e1()));
    assert_eq!(second.held_at, Some(first.entry.digest.clone()));
    assert_eq!(second.entry, first.entry);
    assert!(second.bytes_held_already);
    assert_eq!(fakes.store.contents(), held, "the second write wrote");
    assert_eq!(fakes.store.contents().held.len(), 1);
    assert_eq!(
        store_ops(&fakes),
        [
            StoreOp::PutBytes,
            StoreOp::Name,
            StoreOp::PutBytes,
            StoreOp::Name
        ],
        "a held name was retried"
    );
}

#[test]
fn another_hosts_bytes_held_first_are_answered_and_the_name_stays() {
    let fakes = fakes();
    let theirs = Digest::of(b"another host's archive");
    fakes.store.put_bytes(b"another host's archive").unwrap();
    fakes
        .store
        .name(&cache_name(&b7e1()), &theirs, None)
        .unwrap();

    let answer = written(write(&fakes.seams(), &site(), &b7e1()));
    assert_eq!(answer.held_at, Some(theirs.clone()));
    assert_ne!(answer.entry.digest, theirs);
    assert_eq!(
        fakes.store.contents().names.get(&cache_name(&b7e1())),
        Some(&theirs),
        "the name moved"
    );
    let names = fakes
        .store
        .calls()
        .into_iter()
        .filter(|c| c.op == StoreOp::Name)
        .count();
    assert_eq!(names, 2, "a held name was retried");
}

// Requirement: The same bytes are one store object

#[test]
fn an_entry_whose_bytes_the_store_holds_is_put_once() {
    let fakes = fakes();
    let packed = fakes.projections[&Ecosystem::new("eco-a")]
        .pack(&site(), &b7e1(), None)
        .unwrap();
    fakes.store.put_bytes(&packed.bytes).unwrap();
    let held = fakes.store.contents().held;

    let answer = written(write(&fakes.seams(), &site(), &b7e1()));
    assert!(answer.bytes_held_already);
    assert_eq!(answer.held_at, None, "the name was made by this write");
    assert_eq!(
        fakes.store.contents().held,
        held,
        "the bytes were put again"
    );
}

#[test]
fn a_projection_answering_unreadable_is_the_writes_answer() {
    let fakes = fakes();
    fakes.projections[&Ecosystem::new("eco-a")].fault(
        ProjectionOp::Pack,
        Fault::Unreadable("no archive".into()),
        Times::Once,
    );
    match write(&fakes.seams(), &site(), &b7e1()) {
        Err(Not::Unreadable(u)) => assert_eq!(u.said.as_deref(), Some("no archive")),
        other => panic!("answered {other:?}"),
    }
    assert_eq!(store_ops(&fakes), []);
}

// Requirement: Sync defaults to the newest landed revision the checkout descends from

#[test]
fn a_checkout_ahead_of_the_branch_answers_the_revision_it_branched_from() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit("chore: start", &[("a", "a")]);
    let landed = repo.commit("feat: landed", &[("b", "b")]);
    repo.git(&["checkout", "-q", "-b", "work"]);
    repo.commit("feat: mine", &[("c", "c")]);
    repo.commit("feat: mine again", &[("d", "d")]);
    repo.git(&["checkout", "-q", "main"]);
    let after = repo.commit("feat: landed after", &[("e", "e")]);
    repo.git(&["checkout", "-q", "work"]);

    let history = repo.history();
    let main = Branch::new("main");
    assert_eq!(history.base(&main).unwrap(), landed);
    assert_ne!(history.head(&main).unwrap(), landed);
    assert_eq!(history.head(&main).unwrap(), after);

    repo.git(&["checkout", "-q", "main"]);
    assert_eq!(repo.history().base(&main).unwrap(), after);
}

#[test]
fn a_branch_the_repository_lacks_is_absent_naming_it() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit("chore: start", &[("a", "a")]);
    match repo.history().base(&Branch::new("elsewhere")) {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source.seam, Seam::History);
            assert!(a.what.contains("elsewhere"), "{}", a.what);
        }
        other => panic!("answered {other:?}"),
    }
}

#[test]
fn a_branch_that_shares_no_history_with_the_checkout_is_absent_naming_it() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit("chore: start", &[("a", "a")]);
    repo.git(&["checkout", "-q", "--orphan", "apart"]);
    repo.commit("chore: apart", &[("z", "z")]);
    match repo.history().base(&Branch::new("main")) {
        Err(Not::Absent(a)) => assert!(a.what.contains("branch main"), "{}", a.what),
        other => panic!("answered {other:?}"),
    }
}

// Requirement: Sync renders the entries the landing's verdicts name

const LANDED: &str = "4f2a9c1";

fn landed() -> Revision {
    Revision::new(LANDED)
}

fn main() -> Branch {
    Branch::new("main")
}

fn ask() -> SyncAsk {
    SyncAsk {
        branch: main(),
        revision: landed(),
        at: "/srv/checkout".into(),
    }
}

/// A verdict of `test` on `build` at the landed revision, signed as the
/// landing signs.
fn verdict(build: &str, outcome: Outcome) -> Verdict {
    let id = SignerId::new("landing");
    let theirs = FakeSigner::new();
    theirs.hold(&id);
    let claim = Claim {
        subject: Digest::of(build.as_bytes()),
        kind: ClaimKind::Verdict,
        revision: landed(),
        build: Some(BuildId::new(build)),
        body: json!({ "check": "test", "outcome": outcome }),
    };
    Verdict {
        check: Check::new("test"),
        build: BuildId::new(build),
        outcome,
        signer: id.clone(),
        envelope: theirs.sign(&claim, &id).unwrap(),
        found: format!("the fake landing's record of test on {build}"),
    }
}

/// The fakes with the revision landed, the landing recording `verdicts`
/// for it, and an entry written for each of `entries`.
fn landed_with(verdicts: &[(&str, Outcome)], entries: &[&str]) -> Fakes {
    let fakes = fakes();
    fakes.landing.land(&main(), &landed());
    for (build, outcome) in verdicts {
        fakes.landing.verdict(&landed(), verdict(build, *outcome));
    }
    for build in entries {
        written(write(&fakes.seams(), &site(), &BuildId::new(*build)));
    }
    fakes
}

/// The store's calls after the first `since`.
fn store_since(fakes: &Fakes, since: usize) -> Vec<Call<StoreOp>> {
    fakes.store.calls().split_off(since)
}

fn synced(fakes: &Fakes) -> Synced {
    sync(&fakes.seams(), &ask()).unwrap_or_else(|not| panic!("the sync answered {not:?}"))
}

fn filled(answer: &Answer<Filled>) -> &Filled {
    answer
        .as_ref()
        .unwrap_or_else(|not| panic!("the entry answered {not:?}"))
}

/// The build identities of the entries each render of `projection` was
/// given, one list per render.
fn rendered_by(fakes: &Fakes, projection: &str) -> Vec<Vec<String>> {
    fakes.projections[&Ecosystem::new(projection)]
        .calls()
        .into_iter()
        .filter(|c| c.op == ProjectionOp::Render)
        .map(|c| {
            c.given["contents"]["cache_entries"]
                .as_array()
                .expect("a render of cache entries")
                .iter()
                .map(|e| e["build"].as_str().unwrap().to_string())
                .collect()
        })
        .collect()
}

#[test]
fn a_revision_the_branch_has_not_landed_is_refused_and_the_store_is_asked_nothing() {
    let fakes = fakes();
    match sync(&fakes.seams(), &ask()) {
        Err(Not::Refused(r)) => {
            assert_eq!(
                r.why,
                Why::NotLanded {
                    branch: main(),
                    revision: landed(),
                }
            );
            assert_eq!(r.source.profile, BOUND);
        }
        other => panic!("answered {other:?}"),
    }
    assert_eq!(store_ops(&fakes), []);
    let landing: Vec<LandingOp> = fakes.landing.calls().into_iter().map(|c| c.op).collect();
    assert_eq!(landing, [LandingOp::Landed], "the verdicts were read");
    for projection in fakes.projections.values() {
        assert_eq!(projection.calls(), []);
    }
}

#[test]
fn every_passing_verdicts_entry_is_rendered_and_a_failed_ones_never_asked_for() {
    let fakes = landed_with(
        &[
            ("b7e1", Outcome::Pass),
            ("c3d4", Outcome::Fail),
            ("d5f6", Outcome::Pass),
        ],
        &["b7e1", "c3d4", "d5f6"],
    );
    let since = fakes.store.calls().len();
    let answer = synced(&fakes);

    let builds: Vec<&str> = answer.entries.iter().map(|(b, _)| b.as_str()).collect();
    assert_eq!(builds, ["b7e1", "d5f6"]);
    let names = fakes.store.contents().names;
    for (build, entry) in &answer.entries {
        let entry = filled(entry);
        assert_eq!(Some(&entry.digest), names.get(&cache_name(build)));
        assert_eq!(
            entry.rendered,
            [Ecosystem::new("eco-a"), Ecosystem::new("eco-b")]
        );
    }
    let resolved: Vec<String> = store_since(&fakes, since)
        .into_iter()
        .map(|c| c.given["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        resolved,
        ["cache/b7e1", "cache/d5f6"],
        "a failed build was asked for"
    );
}

#[test]
fn two_passing_verdicts_naming_one_build_identity_resolve_it_once() {
    let fakes = landed_with(
        &[("b7e1", Outcome::Pass), ("b7e1", Outcome::Pass)],
        &["b7e1"],
    );
    let since = fakes.store.calls().len();
    let answer = synced(&fakes);
    assert_eq!(answer.entries.len(), 1);
    assert_eq!(store_since(&fakes, since).len(), 1, "resolved twice");
}

#[test]
fn an_entry_the_store_names_nothing_for_is_absent_and_the_others_render() {
    let fakes = landed_with(
        &[("b7e1", Outcome::Pass), ("c3d4", Outcome::Pass)],
        &["c3d4"],
    );
    let answer = synced(&fakes);
    let (gone, kept) = (&answer.entries[0], &answer.entries[1]);
    assert_eq!(gone.0, BuildId::new("b7e1"));
    match &gone.1 {
        Err(Not::Absent(a)) => {
            assert_eq!(a.source.seam, Seam::Store);
            assert!(a.what.contains("b7e1"), "{}", a.what);
        }
        other => panic!("the gone entry answered {other:?}"),
    }
    assert_eq!(kept.0, BuildId::new("c3d4"));
    filled(&kept.1);
    assert_eq!(rendered_by(&fakes, "eco-a"), [["c3d4"]]);

    let json = answer_json(&Ok::<_, Not>(answer.clone()));
    assert_eq!(json["answer"], "ok");
    let entries = &json["value"]["entries"];
    assert_eq!(entries[0]["answer"], "absent");
    assert_eq!(entries[0]["build"], "b7e1");
    assert_eq!(entries[1]["answer"], "ok");
    assert_eq!(entries[1]["build"], "c3d4");
    assert_eq!(entries[1]["value"]["rendered"], json!(["eco-a", "eco-b"]));
    assert_eq!(json["value"]["revision"], LANDED);
    assert_eq!(json["value"]["branch"], "main");
}

#[test]
fn each_entry_is_rendered_alone_through_every_projection_that_packs_cache_entries() {
    let mut fakes = landed_with(
        &[("b7e1", Outcome::Pass), ("c3d4", Outcome::Pass)],
        &["b7e1", "c3d4"],
    );
    fakes
        .projection(&Ecosystem::new("eco-c"))
        .packs_no_cache_entries();
    let answer = synced(&fakes);
    for (_, entry) in &answer.entries {
        assert_eq!(
            filled(entry).rendered,
            [Ecosystem::new("eco-a"), Ecosystem::new("eco-b")],
            "a projection that packs none was counted"
        );
    }
    for projection in ["eco-a", "eco-b", "eco-c"] {
        assert_eq!(
            rendered_by(&fakes, projection),
            [["b7e1"], ["c3d4"]],
            "{projection} was not given each entry alone"
        );
    }
    let at: Vec<String> = fakes.projections[&Ecosystem::new("eco-a")]
        .calls()
        .into_iter()
        .filter(|c| c.op == ProjectionOp::Render)
        .map(|c| c.given["at"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(at, ["/srv/checkout", "/srv/checkout"]);
}

#[test]
fn no_projection_that_packs_cache_entries_renders_none_and_is_no_refusal() {
    let fakes = landed_with(&[("b7e1", Outcome::Pass)], &["b7e1"]);
    for projection in fakes.projections.values() {
        projection.packs_no_cache_entries();
    }
    let answer = synced(&fakes);
    match &answer.entries[0].1 {
        Err(Not::Absent(a)) => assert_eq!(a.what, "cache entries"),
        other => panic!("answered {other:?}"),
    }
}

#[test]
fn a_renders_refusal_is_that_entrys_answer_alone() {
    let fakes = landed_with(
        &[("b7e1", Outcome::Pass), ("c3d4", Outcome::Pass)],
        &["b7e1", "c3d4"],
    );
    fakes.projections[&Ecosystem::new("eco-b")].fault(
        ProjectionOp::Render,
        Fault::Refused("the archive is not moon's".into()),
        Times::Once,
    );
    let answer = synced(&fakes);
    match &answer.entries[0].1 {
        Err(Not::Refused(r)) => {
            assert_eq!(r.said.as_deref(), Some("the archive is not moon's"))
        }
        other => panic!("the first entry answered {other:?}"),
    }
    assert_eq!(
        filled(&answer.entries[1].1).rendered,
        [Ecosystem::new("eco-a"), Ecosystem::new("eco-b")]
    );
}

#[test]
fn sync_asks_the_store_only_to_resolve_and_never_the_policy() {
    let fakes = landed_with(
        &[("b7e1", Outcome::Pass), ("c3d4", Outcome::Pass)],
        &["b7e1"],
    );
    let since = fakes.store.calls().len();
    synced(&fakes);
    let ops: Vec<StoreOp> = store_since(&fakes, since)
        .into_iter()
        .map(|c| c.op)
        .collect();
    assert_eq!(ops, [StoreOp::Resolve, StoreOp::Resolve]);
    assert_eq!(fakes.policy.calls(), [], "the policy was asked");
    assert_eq!(fakes.build.calls(), [], "the build was asked");
    assert_eq!(fakes.signer.calls(), [], "a signer was asked");
    assert_eq!(fakes.keeper.calls(), [], "the keeper was asked");
    for projection in fakes.projections.values() {
        assert!(
            projection
                .calls()
                .iter()
                .skip(1)
                .all(|c| c.op == ProjectionOp::Render),
            "a projection was asked to pack or serve by the sync"
        );
    }
}
