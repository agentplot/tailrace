//! The cache's group: cache entries written by one host and rendered on
//! another, held once, never an artifact, and removable at any time with
//! nothing accepted or released the worse for it, S14 among them (52 to
//! 54, 100; design D8); and the push after a landing, which writes the
//! entries of a landing's passing builds and offers and keeps nothing
//! (52 to 54, 77).

use tailrace_core::{
    cache_name, push, sync, write, Entry, Filled, Published, Push, PushAsk, SyncAsk, Synced,
    Verified,
};
use tailrace_fakes::{PolicyOp, ProjectionOp};
use tailrace_seams::{
    Answer, Artifact, Audience, Branch, BuildId, CacheEntry, ClaimKind, ComponentName, Contents,
    Digest, Ecosystem, Held, Layout, Name, NamedSigner, Not, Outcome, Output, OutputName, Packed,
    Pinned, Projection, Range, Refusal, Rendered, Required, RequiredClaim, Requirement, Revision,
    Serving, SignerId, Store, Verdict, Version, Why,
};

use crate::core::publish::{one, passing, version_of};
use crate::world::{build_of, FakeWorld, Write, FILE, HOST, LANDING};
use crate::{ok, Scenario};

pub const GROUP: &str = "cache";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
        Scenario {
            name: "written on one host, read on another",
            plays: None,
            cites: &[52],
            run: written_on_one_host_read_on_another,
        },
        Scenario {
            name: "an entry held already",
            plays: None,
            cites: &[52],
            run: an_entry_held_already,
        },
        Scenario {
            name: "one store object for an artifact and an entry",
            plays: None,
            cites: &[100],
            run: one_store_object_for_an_artifact_and_an_entry,
        },
        Scenario {
            name: "an entry is never an artifact",
            plays: None,
            cites: &[53],
            run: an_entry_is_never_an_artifact,
        },
        Scenario {
            name: "an entry removed",
            plays: None,
            cites: &[54],
            run: an_entry_removed,
        },
        Scenario {
            name: "a removed entry builds again",
            plays: Some("S14"),
            cites: &[54],
            run: a_removed_entry_builds_again,
        },
        Scenario {
            name: "a landing's archives pushed and synced on another host",
            plays: None,
            cites: &[52],
            run: a_landings_archives_pushed_and_synced_on_another_host,
        },
        Scenario {
            name: "a push of a revision not landed",
            plays: None,
            cites: &[77],
            run: a_push_of_a_revision_not_landed,
        },
        Scenario {
            name: "a push repeated writes nothing",
            plays: None,
            cites: &[52],
            run: a_push_repeated_writes_nothing,
        },
        Scenario {
            name: "an archive not on the pushing host",
            plays: None,
            cites: &[54],
            run: an_archive_not_on_the_pushing_host,
        },
        Scenario {
            name: "a push offers and keeps nothing",
            plays: None,
            cites: &[53],
            run: a_push_offers_and_keeps_nothing,
        },
    ]
}

/// The world's two outputs, as the policy file declares them.
const OUTPUTS: [&str; 2] = ["site", "server"];

fn output(name: &str, revision: &Revision) -> Output {
    Output {
        name: OutputName::new(name),
        files: vec![format!("dist/{name}@{revision}")],
    }
}

/// `revision` landed with passing verdicts on both outputs, and each
/// output's cache entry written at its build identity; the entries.
fn written(world: &FakeWorld, revision: &Revision) -> Vec<CacheEntry> {
    passing(world, revision);
    OUTPUTS
        .iter()
        .map(|o| {
            let build = build_of(o, revision);
            let answer = ok(
                write(&world.seams(), &output(o, revision), &build),
                &format!("the write of {build}"),
            );
            assert_eq!(answer.held_at, None, "the write of {build} found it held");
            answer.entry
        })
        .collect()
}

/// `revision` synced in `world`, rendered into its clone.
fn synced(world: &FakeWorld, revision: &Revision) -> Synced {
    let ask = SyncAsk {
        branch: world.main.clone(),
        revision: revision.clone(),
        at: world.repo().path.display().to_string(),
    };
    ok(sync(&world.seams(), &ask), "the sync")
}

fn filled<'a>(synced: &'a Synced, build: &BuildId) -> &'a Answer<Filled> {
    &synced
        .entries
        .iter()
        .find(|(b, _)| b == build)
        .unwrap_or_else(|| panic!("the sync named no entry for {build}: {synced:?}"))
        .1
}

/// The entries each render of `world`'s `ecosystem` projection was
/// given, by build identity and digest, one list per render.
fn renders(world: &FakeWorld, ecosystem: &str) -> Vec<Vec<(String, String)>> {
    world.fakes.projections[&Ecosystem::new(ecosystem)]
        .calls()
        .into_iter()
        .filter(|c| c.op == ProjectionOp::Render)
        .map(|c| {
            c.given["contents"]["cache_entries"]
                .as_array()
                .unwrap_or_else(|| panic!("a render of no cache entries: {}", c.given))
                .iter()
                .map(|e| {
                    (
                        e["build"].as_str().unwrap_or_default().to_string(),
                        e["digest"].as_str().unwrap_or_default().to_string(),
                    )
                })
                .collect()
        })
        .collect()
}

/// A second host: a world of its own, sharing only `world`'s store.
fn second_host(world: &FakeWorld) -> FakeWorld {
    let mut second = FakeWorld::new();
    second.fakes.store = world.fakes.store.clone();
    second
}

fn written_on_one_host_read_on_another(world: &FakeWorld) {
    let entries = written(world, &world.start);
    let second = second_host(world);
    passing(&second, &world.start);
    let answer = synced(&second, &world.start);

    let expected: Vec<Vec<(String, String)>> = entries
        .iter()
        .map(|e| vec![(e.build.to_string(), e.digest.to_string())])
        .collect();
    for ecosystem in second.fakes.projections.keys() {
        assert_eq!(
            renders(&second, ecosystem.as_str()),
            expected,
            "the second host's {ecosystem} projection was not asked to render exactly \
             the first host's entries"
        );
    }
    for entry in &entries {
        let filled = ok(filled(&answer, &entry.build).clone(), "the entry's render");
        assert_eq!(
            filled.digest, entry.digest,
            "{} rendered other bytes",
            entry.build
        );
    }
    for (ecosystem, projection) in &world.fakes.projections {
        assert!(
            projection
                .calls()
                .iter()
                .all(|c| c.op != ProjectionOp::Render),
            "the first host's {ecosystem} projection rendered for the second"
        );
    }
}

fn an_entry_held_already(world: &FakeWorld) {
    let entries = written(world, &world.start);
    let names = world.fakes.store.contents().names;
    let second = second_host(world);
    passing(&second, &world.start);
    for projection in second.fakes.projections.values() {
        projection.unsteady();
    }
    for (o, entry) in OUTPUTS.iter().zip(&entries) {
        let again = ok(
            write(&second.seams(), &output(o, &world.start), &entry.build),
            "the second host's write",
        );
        assert_eq!(
            again.held_at.as_ref(),
            Some(&entry.digest),
            "the second host's write of {} was not answered held at the first's digest",
            entry.build
        );
        assert_ne!(
            again.entry.digest, entry.digest,
            "the second host packed the same bytes; the scenario needs other bytes"
        );
    }
    assert_eq!(
        world.fakes.store.contents().names,
        names,
        "a name moved when the second host wrote"
    );
}

/// A projection that packs, at no version, the bytes it is given.
struct Carrying(Vec<u8>);

impl Projection for Carrying {
    fn pack(&self, _: &Output, _: &BuildId, version: Option<&Version>) -> Answer<Packed> {
        assert!(version.is_none(), "an entry was packed at a version");
        Ok(Packed {
            bytes: self.0.clone(),
            meta: None,
            same_again: true,
        })
    }

    fn render(&self, _: Contents<'_>, _: &str) -> Answer<Rendered> {
        unreachable!("a write renders nothing")
    }

    fn serve(&self, _: &Layout) -> Answer<Serving> {
        unreachable!("a write serves nothing")
    }

    fn pinned(&self, _: &str) -> Answer<Vec<Pinned>> {
        unreachable!("a write reads no lockfile")
    }

    fn requires(&self, _: &Artifact) -> Answer<Vec<Requirement>> {
        Ok(Vec::new())
    }

    fn satisfying(&self, _: &Range, _: &[Version]) -> Answer<Vec<Version>> {
        Ok(Vec::new())
    }
}

/// Acceptance requires the host's provenance on each artifact.
fn acceptance() -> Required {
    Required::Acceptance {
        claims: vec![RequiredClaim {
            kind: ClaimKind::Provenance,
            check: None,
            signers: vec![NamedSigner {
                id: SignerId::new(HOST),
                verified_by: format!("{HOST}'s key"),
                kinds: vec![ClaimKind::Provenance],
                checks: vec![],
            }],
        }],
    }
}

fn verified(world: &FakeWorld, version: &Version) -> Verified {
    ok(
        world.verify(&world.verify_ask(version.as_str(), Audience::Acceptance)),
        "verify",
    )
}

fn one_store_object_for_an_artifact_and_an_entry(world: &FakeWorld) {
    passing(world, &world.head);
    world
        .fakes
        .policy
        .require(Audience::Acceptance, acceptance());
    let published = ok(one(world.publish(&world.ask(&world.head))), "the publish");
    let (version, _) = version_of(&published);
    let version = version.clone();
    let Published::Version { artifacts, .. } = &published else {
        unreachable!("a version")
    };
    let artifact = &artifacts[0];
    let Held::Bytes(bytes) = ok(
        world.fakes.store.read(&artifact.bytes),
        "the artifact's bytes",
    ) else {
        panic!("{} holds no bytes", artifact.bytes)
    };
    let before = verified(world, &version);
    let held = world.fakes.store.contents().held;

    let mut seams = world.seams();
    seams.projections.clear();
    seams
        .projections
        .insert(Ecosystem::new("carrying"), Box::new(Carrying(bytes)));
    let entry = ok(
        write(&seams, &output("site", &world.head), &artifact.build),
        "the entry's write",
    );
    assert!(
        entry.bytes_held_already,
        "the put of bytes an artifact holds was not answered held already"
    );
    assert_eq!(entry.entry.digest, artifact.bytes);
    assert_eq!(
        world.fakes.store.contents().held,
        held,
        "the store holds other objects after the entry's write"
    );
    assert_eq!(
        verified(world, &version),
        before,
        "the version verifies otherwise after the entry"
    );
}

fn an_entry_is_never_an_artifact(world: &FakeWorld) {
    let entries = written(world, &world.start);
    synced(world, &world.start);

    let writes = world.writes();
    let expected: Vec<Write> = entries
        .iter()
        .flat_map(|e| {
            [
                Write::Bytes(e.digest.clone()),
                Write::Name {
                    name: cache_name(&e.build),
                    to: e.digest.clone(),
                    expected: None,
                },
            ]
        })
        .collect();
    assert_eq!(
        writes, expected,
        "writing and syncing entries asked the store for more than each entry's bytes and name"
    );
    let names: Vec<Name> = world.fakes.store.contents().names.into_keys().collect();
    assert!(
        names.iter().all(|n| n.as_str().starts_with("cache/")),
        "a name beside the entries': {names:?}"
    );
    assert!(
        world
            .fakes
            .policy
            .calls()
            .iter()
            .all(|c| c.op != PolicyOp::PolicyFor),
        "the policy was asked"
    );
}

fn an_entry_removed(world: &FakeWorld) {
    let entries = written(world, &world.start);
    let (gone, kept) = (&entries[0], &entries[1]);
    world.fakes.store.forget(&cache_name(&gone.build));
    let answer = synced(world, &world.start);
    assert!(
        matches!(filled(&answer, &gone.build), Err(Not::Absent(_))),
        "the removed entry answered {:?}",
        filled(&answer, &gone.build)
    );
    let filled = ok(filled(&answer, &kept.build).clone(), "the kept entry");
    assert_eq!(filled.digest, kept.digest);
    assert_eq!(
        renders(world, FILE),
        [vec![(kept.build.to_string(), kept.digest.to_string())]],
        "the removed entry was rendered"
    );
}

fn a_removed_entry_builds_again(world: &FakeWorld) {
    let entries = written(world, &world.start);
    let site = entries
        .iter()
        .find(|e| e.build == build_of("site", &world.start))
        .expect("the site's entry");
    world.fakes.store.forget(&cache_name(&site.build));
    passing(world, &world.head);
    let names = world.fakes.store.contents().names;

    let answer = synced(world, &world.start);
    for (build, entry) in &answer.entries {
        assert!(
            !matches!(entry, Err(Not::Refused(_) | Not::Unreadable(_))),
            "the sync refused {build}: {entry:?}"
        );
    }
    assert!(
        matches!(filled(&answer, &site.build), Err(Not::Absent(_))),
        "the removed entry was not answered absent"
    );
    ok(
        filled(&answer, &build_of("server", &world.start)).clone(),
        "the server's entry",
    );

    let published = ok(one(world.publish(&world.ask(&world.head))), "the publish");
    let Published::Version { artifacts, .. } = &published else {
        panic!("the head was not published: {published:?}")
    };
    let mut built: Vec<BuildId> = artifacts.iter().map(|a| a.build.clone()).collect();
    built.sort();
    let mut expected: Vec<BuildId> = OUTPUTS.iter().map(|o| build_of(o, &world.head)).collect();
    expected.sort();
    assert_eq!(built, expected, "the publish did not offer the new builds");

    let now = world.fakes.store.contents();
    for (name, digest) in &names {
        assert_eq!(
            now.names.get(name),
            Some(digest),
            "{name} names otherwise after the sync and the publish"
        );
        assert!(now.held.contains_key(digest), "{digest} is no longer held");
    }
}

/// `revision` landed with the verdicts `passing` records, both passing,
/// and a third failing on a build identity of its own; every verdict.
fn landed_with_a_failure(world: &FakeWorld, revision: &Revision) -> Vec<Verdict> {
    let mut verdicts = passing(world, revision);
    let failed = world.verdict(
        revision,
        "lint",
        &failed_build(revision),
        Outcome::Fail,
        LANDING,
    );
    world.land(revision, std::slice::from_ref(&failed));
    verdicts.push(failed);
    verdicts
}

/// The build identity the failing verdict names.
fn failed_build(revision: &Revision) -> BuildId {
    build_of("lint", revision)
}

fn push_ask(world: &FakeWorld, revision: &Revision) -> PushAsk {
    PushAsk {
        branch: world.main.clone(),
        revision: revision.clone(),
    }
}

/// `revision` pushed from `world`.
fn pushed(world: &FakeWorld, revision: &Revision) -> Push {
    ok(push(&world.seams(), &push_ask(world, revision)), "the push")
}

/// The entry a push wrote for `build`, or a panic naming its answer.
fn pushed_entry(push: &Push, build: &BuildId) -> CacheEntry {
    let (_, answer) = push
        .entries
        .iter()
        .find(|(b, _)| b == build)
        .unwrap_or_else(|| panic!("the push named no entry for {build}: {push:?}"));
    ok(answer.clone(), &format!("the push of {build}")).entry
}

fn a_landings_archives_pushed_and_synced_on_another_host(world: &FakeWorld) {
    let verdicts = landed_with_a_failure(world, &world.start);
    let answer = pushed(world, &world.start);
    let builds: Vec<BuildId> = answer.entries.iter().map(|(b, _)| b.clone()).collect();
    let passing: Vec<BuildId> = OUTPUTS.iter().map(|o| build_of(o, &world.start)).collect();
    assert_eq!(
        builds, passing,
        "the push did not answer exactly the passing build identities"
    );
    let entries: Vec<CacheEntry> = passing.iter().map(|b| pushed_entry(&answer, b)).collect();

    let second = second_host(world);
    second.land(&world.start, &verdicts);
    let synced = synced(&second, &world.start);
    let expected: Vec<Vec<(String, String)>> = entries
        .iter()
        .map(|e| vec![(e.build.to_string(), e.digest.to_string())])
        .collect();
    for ecosystem in second.fakes.projections.keys() {
        assert_eq!(
            renders(&second, ecosystem.as_str()),
            expected,
            "the second host's {ecosystem} projection was not asked to render exactly \
             the pushed entries"
        );
    }
    for entry in &entries {
        let filled = ok(filled(&synced, &entry.build).clone(), "the entry's render");
        assert_eq!(
            filled.digest, entry.digest,
            "{} synced other bytes",
            entry.build
        );
    }
    assert!(
        !world
            .fakes
            .store
            .contents()
            .names
            .contains_key(&cache_name(&failed_build(&world.start))),
        "the failed build's entry was written"
    );
}

fn a_push_of_a_revision_not_landed(world: &FakeWorld) {
    passing(world, &world.start);
    let asked = world.asked();
    let answer = push(&world.seams(), &push_ask(world, &world.head));
    match answer {
        Err(Not::Refused(Refusal {
            why: Why::NotLanded { branch, revision },
            ..
        })) => {
            assert_eq!(branch, world.main, "the refusal names another branch");
            assert_eq!(revision, world.head, "the refusal names another revision");
        }
        other => panic!("the push of a revision not landed answered {other:?}"),
    }
    assert_eq!(
        world.asked(),
        asked + 1,
        "the push asked more than whether the branch landed the revision"
    );
    assert_eq!(world.writes(), [], "the store was written");
}

fn a_push_repeated_writes_nothing(world: &FakeWorld) {
    passing(world, &world.start);
    let first = pushed(world, &world.start);
    let held = world.fakes.store.contents();
    let second = pushed(world, &world.start);
    assert_eq!(first.entries.len(), second.entries.len());
    for (build, _) in &first.entries {
        let entry = pushed_entry(&first, build);
        let (_, again) = second
            .entries
            .iter()
            .find(|(b, _)| b == build)
            .expect("the second push names the same build identities");
        let again = ok(again.clone(), "the second push");
        assert_eq!(
            again.held_at.as_ref(),
            Some(&entry.digest),
            "the second push of {build} was not answered held at the first's digest"
        );
    }
    assert_eq!(
        world.fakes.store.contents(),
        held,
        "the store holds otherwise after the second push"
    );
}

fn an_archive_not_on_the_pushing_host(world: &FakeWorld) {
    passing(world, &world.start);
    let (gone, kept) = (
        build_of("site", &world.start),
        build_of("server", &world.start),
    );
    for projection in world.fakes.projections.values() {
        projection.no_archive(&gone);
    }
    let answer = pushed(world, &world.start);
    let (_, missing) = answer
        .entries
        .iter()
        .find(|(b, _)| *b == gone)
        .expect("the push names the build with no archive");
    assert!(
        matches!(missing, Err(Not::Absent(_))),
        "the build with no archive answered {missing:?}"
    );
    for (build, entry) in &answer.entries {
        assert!(
            !matches!(entry, Err(Not::Refused(_) | Not::Unreadable(_))),
            "the push refused {build}: {entry:?}"
        );
    }
    let entry = pushed_entry(&answer, &kept);
    let names = world.fakes.store.contents().names;
    assert_eq!(names.get(&cache_name(&kept)), Some(&entry.digest));
    assert!(
        !names.contains_key(&cache_name(&gone)),
        "a name was made for the build with no archive"
    );
}

fn a_push_offers_and_keeps_nothing(world: &FakeWorld) {
    passing(world, &world.start);
    let kept = world.kept();
    ok(
        kept.write(&Entry {
            component: ComponentName::new("repository"),
            version: Version::new("0.1.0"),
            tag: "v0.1.0".into(),
            revision: world.start.clone(),
            branch: Branch::new(crate::fixture::MAIN),
            index: Digest::of(b"an index published before"),
            store: world.store(),
            notes: None,
        }),
        "the record's entry",
    );
    let record = world.record();
    let answer = pushed(world, &world.start);

    let expected: Vec<Write> = answer
        .entries
        .iter()
        .flat_map(|(build, _)| {
            let entry = pushed_entry(&answer, build);
            [
                Write::Bytes(entry.digest.clone()),
                Write::Name {
                    name: cache_name(build),
                    to: entry.digest,
                    expected: None,
                },
            ]
        })
        .collect();
    assert_eq!(expected.len(), 2 * OUTPUTS.len());
    assert_eq!(
        world.writes(),
        expected,
        "the push asked the store for more than each entry's bytes and name"
    );
    assert_eq!(world.fakes.signer.calls(), [], "the signer was asked");
    assert!(
        world
            .fakes
            .policy
            .calls()
            .iter()
            .all(|c| c.op != PolicyOp::PolicyFor),
        "the policy was asked"
    );
    assert_eq!(world.record(), record, "the push changed the host's record");
}
