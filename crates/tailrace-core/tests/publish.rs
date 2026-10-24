//! Spec publishing: one landed revision turned into a version of each
//! component it publishes, over the fakes and a shared history.

mod common;
mod world;

use std::cell::RefCell;
use std::rc::Rc;

use tailrace_core::{answer_json, publish, Ask, Kept, Provenance, Published, BOUND};
use tailrace_fakes::{BuildOp, FakeStore, Fault, LandingOp, SignerOp, StoreOp, Times};
use tailrace_seams::{
    Answer, Branch, Claim, ClaimKind, ComponentName, Digest, EcosystemName, Envelope,
    EnvironmentName, Held, Manifest, Name, Not, Outcome, Put, Seam, SignerId, Source, Store,
    Version, Why,
};
use world::{build_of, build_ops, by_host, eco, output, version_of, Watched, World, HOST, LANDING};

fn refused(answer: Answer<Published>) -> tailrace_seams::Refusal {
    match answer {
        Err(Not::Refused(r)) => r,
        other => panic!("not refused: {other:?}"),
    }
}

fn store_ops(world: &World) -> Vec<StoreOp> {
    world
        .fakes
        .store
        .calls()
        .into_iter()
        .map(|c| c.op)
        .collect()
}

fn puts(world: &World) -> Vec<StoreOp> {
    store_ops(world)
        .into_iter()
        .filter(|op| {
            matches!(
                op,
                StoreOp::PutBytes | StoreOp::PutArtifact | StoreOp::Attach | StoreOp::Name
            )
        })
        .collect()
}

fn indexes(world: &World) -> Vec<tailrace_seams::IndexMeta> {
    world
        .fakes
        .store
        .contents()
        .held
        .into_values()
        .filter_map(|h| match h {
            Held::Manifest(Manifest::Index(meta)) => Some(meta),
            _ => None,
        })
        .collect()
}

fn nothing_offered(world: &World) {
    assert_eq!(puts(world), [], "the store was offered something");
    assert!(world.fakes.store.contents().held.is_empty());
    assert!(world.fakes.store.contents().names.is_empty());
}

fn no_pack(world: &World) {
    for (e, p) in &world.fakes.projections {
        assert_eq!(p.calls(), [], "the projection for {e} was asked");
    }
}

// Requirement: A publish refuses a revision not landed, before anything else

#[test]
fn a_revision_the_branch_has_not_landed_is_refused_and_nothing_is_read() {
    let world = World::new();
    let ask = Ask {
        branch: Branch::new("release"),
        ..world.ask()
    };
    let Err(Not::Refused(r)) = world.publish(&ask) else {
        panic!("a revision not landed was published")
    };
    assert_eq!(
        r.why,
        Why::NotLanded {
            branch: Branch::new("release"),
            revision: world.head.clone(),
        }
    );
    assert_eq!(r.source.seam, Seam::Landing);
    assert_eq!(world.fakes.build.calls(), []);
    assert_eq!(world.fakes.signer.calls(), []);
    assert_eq!(world.fakes.store.calls(), []);
    no_pack(&world);
}

#[test]
fn a_landing_that_cannot_be_read_answers_its_own_words() {
    let world = World::new();
    world.fakes.landing.fault(
        LandingOp::Landed,
        Fault::Unreadable("connection refused".into()),
        Times::UntilHealed,
    );
    let Err(Not::Unreadable(u)) = world.publish(&world.ask()) else {
        panic!("an unreadable landing was published past")
    };
    assert_eq!(u.said.as_deref(), Some("connection refused"));
    assert_eq!(u.source, Source::new(Seam::Landing, "fake"));
    assert_eq!(world.fakes.build.calls(), []);
    nothing_offered(&world);
}

// Requirement: A publish names its components

#[test]
fn of_two_components_only_the_touched_one_is_published() {
    let world = World::new();
    let publish = world.publish(&world.ask()).unwrap();
    let names: Vec<&str> = publish.components.iter().map(|(c, _)| c.as_str()).collect();
    assert_eq!(names, ["lib"]);
    let (version, _) = version_of(publish.components[0].1.as_ref().unwrap());
    assert_eq!(version, &Version::new("0.2.0"));
    assert!(indexes(&world)
        .iter()
        .all(|i| i.component() == &ComponentName::new("lib")));
    assert!(!world
        .fakes
        .store
        .contents()
        .names
        .keys()
        .any(|n| n.as_str().starts_with("web-v")));
}

#[test]
fn a_component_nothing_touched_is_unchanged_and_nothing_is_offered() {
    let world = World::new();
    let answer = world.publish_one(&world.ask_for("web")).unwrap();
    assert_eq!(
        answer,
        Published::Unchanged {
            component: ComponentName::new("web"),
            last: Some("web-v0.1.0".into()),
        }
    );
    assert_eq!(world.fakes.build.calls(), []);
    no_pack(&world);
    nothing_offered(&world);
}

// Requirement: Each output names the projection that packs it

#[test]
fn an_output_with_no_projection_is_refused_naming_it_before_the_build() {
    let world = World::with_policy(&world::POLICY.replace(", lib-docs = \"eco-b\"", ""));
    let r = refused(world.publish_one(&world.ask_for("lib")));
    assert_eq!(
        r.why,
        Why::NoProjection {
            component: ComponentName::new("lib"),
            output: output("lib-docs"),
        }
    );
    assert_eq!(world.fakes.build.calls(), []);
    nothing_offered(&world);
}

#[test]
fn a_projection_not_bound_is_absent_naming_the_ecosystem() {
    let world =
        World::with_policy(&world::POLICY.replace("lib-docs = \"eco-b\"", "lib-docs = \"eco-c\""));
    let Err(Not::Absent(a)) = world.publish_one(&world.ask_for("lib")) else {
        panic!("an unbound projection was packed")
    };
    assert!(a.what.contains("eco-c"), "{}", a.what);
    assert_eq!(a.source, Source::new(Seam::Host, BOUND));
    assert_eq!(world.fakes.build.calls(), []);
    nothing_offered(&world);
}

// Requirement: Every output is ensured in one asking and its outcome read

#[test]
fn every_output_is_ensured_in_one_asking_then_reported_before_any_pack() {
    let mut world = World::new();
    let build = world.fakes.build.clone();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let inner = world.fakes.projection(&eco("eco-a"));
    let history = world.history();
    let mut seams = world.fakes.seams();
    seams.projections.insert(
        eco("eco-a"),
        Box::new(Watched {
            inner,
            watch: move || build_ops(&build),
            seen: seen.clone(),
        }),
    );
    publish(&seams, &history, &Kept::at(&history), &world.ask())
        .unwrap()
        .components[0]
        .1
        .as_ref()
        .unwrap();

    let calls = world.fakes.build.calls();
    assert_eq!(
        calls.iter().map(|c| c.op).collect::<Vec<_>>(),
        [BuildOp::Ensure, BuildOp::Report, BuildOp::Report]
    );
    assert_eq!(
        calls[0].given["outputs"],
        serde_json::json!(["lib-pack", "lib-docs"])
    );
    assert_eq!(
        seen.borrow()[0],
        [BuildOp::Ensure, BuildOp::Report, BuildOp::Report],
        "the first pack came before every output was reported"
    );
}

#[test]
fn an_output_the_build_could_not_make_refuses_the_version() {
    let world = World::with_policy(
        &world::POLICY
            .replace(
                "outputs = [\"lib-pack\", \"lib-docs\"]",
                "outputs = [\"lib-pack\", \"lib-docs\", \"lib-cli\"]",
            )
            .replace(
                "lib-docs = \"eco-b\"",
                "lib-docs = \"eco-b\", lib-cli = \"eco-b\"",
            ),
    );
    let r = refused(world.publish_one(&world.ask_for("lib")));
    assert_eq!(
        r.why,
        Why::Unmade {
            outputs: vec![output("lib-cli")]
        }
    );
    no_pack(&world);
    nothing_offered(&world);
}

// Requirement: An output is a pass only by the build and the landing together

#[test]
fn a_build_that_reports_a_failure_refuses_naming_the_output_and_outcome() {
    let world = World::new();
    world.made("lib-docs", Outcome::Fail);
    let r = refused(world.publish_one(&world.ask_for("lib")));
    assert_eq!(
        r.why,
        Why::NotPassed {
            output: output("lib-docs"),
            build: build_of("lib-docs", &world.head),
            outcome: Outcome::Fail,
        }
    );
    assert_eq!(r.source.seam, Seam::Build);
    nothing_offered(&world);
}

#[test]
fn only_a_failing_verdict_naming_the_build_refuses_it() {
    let world = World::new();
    world.verdict(
        "test",
        &build_of("lib-pack", &world.head),
        Outcome::Pass,
        LANDING,
    );
    world.verdict(
        "test",
        &build_of("lib-docs", &world.head),
        Outcome::Fail,
        LANDING,
    );
    let r = refused(world.publish_one(&world.ask_for("lib")));
    assert_eq!(
        r.why,
        Why::NotPassed {
            output: output("lib-docs"),
            build: build_of("lib-docs", &world.head),
            outcome: Outcome::Fail,
        }
    );
    assert_eq!(r.source.seam, Seam::Landing);
    no_pack(&world);
    nothing_offered(&world);
}

#[test]
fn a_landing_with_no_verdicts_offers_on_the_builds_pass_alone() {
    let world = World::new();
    let published = world.publish_one(&world.ask_for("lib")).unwrap();
    let Published::Version { artifacts, .. } = &published else {
        panic!("{published:?}")
    };
    assert_eq!(artifacts.len(), 2);
    for artifact in artifacts {
        let envelopes = world.fakes.store.contents().attached[&artifact.digest].clone();
        let claim = Claim::from_envelope(&envelopes[0]).unwrap();
        assert!(Provenance::of(&claim).unwrap().carried.is_empty());
    }
}

// Requirement: An output no verdict names is refused naming both

#[test]
fn a_build_the_landing_never_checked_is_refused_naming_every_identity() {
    let world = World::new();
    let other = tailrace_seams::BuildId::new("elsewhere");
    world.verdict(
        "test",
        &build_of("lib-pack", &world.head),
        Outcome::Pass,
        LANDING,
    );
    world.verdict("lint", &other, Outcome::Pass, LANDING);
    world.verdict("test", &other, Outcome::Pass, LANDING);
    let r = refused(world.publish_one(&world.ask_for("lib")));
    assert_eq!(
        r.why,
        Why::Unnamed {
            output: output("lib-docs"),
            build: build_of("lib-docs", &world.head),
            carried: vec![build_of("lib-pack", &world.head), other],
        }
    );
    nothing_offered(&world);
}

// Requirement: The name is claimed after the outputs are judged and before anything is packed

#[test]
fn a_refused_version_claims_no_name() {
    let world = World::new();
    world.made("lib-pack", Outcome::Fail);
    refused(world.publish_one(&world.ask_for("lib")));
    assert_eq!(world.repo.tag_at("lib-v0.2.0"), None);
    assert_eq!(world.shared.tag_at("lib-v0.2.0"), None);
}

#[test]
fn the_name_is_claimed_before_the_first_pack_and_the_build_never_sees_it() {
    let mut world = World::new();
    let repo = world.repo.clone();
    let shared = world.shared.clone();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let inner = world.fakes.projection(&eco("eco-a"));
    let history = world.history();
    let mut seams = world.fakes.seams();
    seams.projections.insert(
        eco("eco-a"),
        Box::new(Watched {
            inner,
            watch: move || (repo.tag_at("lib-v0.2.0"), shared.tag_at("lib-v0.2.0")),
            seen: seen.clone(),
        }),
    );
    publish(&seams, &history, &Kept::at(&history), &world.ask())
        .unwrap()
        .components[0]
        .1
        .as_ref()
        .unwrap();
    let head = Some(world.head.to_string());
    assert_eq!(seen.borrow()[0], (head.clone(), head));
    for call in world.fakes.build.calls() {
        let given = call.given.to_string();
        assert!(
            !given.contains("0.2.0"),
            "the build was given the version: {given}"
        );
    }
}

// Requirement: Each artifact is packed, put and checked by its digest

#[test]
fn two_artifacts_are_put_each_under_its_own_digest() {
    let world = World::new();
    let published = world.publish_one(&world.ask_for("lib")).unwrap();
    let Published::Version { artifacts, .. } = published else {
        unreachable!()
    };
    let held = world.fakes.store.contents().held;
    let mut ecosystems = Vec::new();
    for artifact in &artifacts {
        let Held::Bytes(bytes) = &held[&artifact.bytes] else {
            panic!("no bytes under {}", artifact.bytes)
        };
        assert_eq!(Digest::of(bytes), artifact.bytes);
        let Held::Manifest(Manifest::Artifact { meta, layers }) = &held[&artifact.digest] else {
            panic!("no artifact under {}", artifact.digest)
        };
        assert_eq!(layers, std::slice::from_ref(&artifact.bytes));
        assert_eq!(meta.version(), &Version::new("0.2.0"));
        assert_eq!(meta.name(), &EcosystemName::new(artifact.output.as_str()));
        ecosystems.push(meta.ecosystem().clone());
    }
    assert_eq!(ecosystems, [eco("eco-a"), eco("eco-b")]);
}

/// A store that names every bytes it is given by another digest.
struct Misnaming(FakeStore);

impl Store for Misnaming {
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        let mut put = self.0.put_bytes(bytes)?;
        put.digest = Digest::of(b"not these bytes");
        Ok(put)
    }
    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        self.0.put_artifact(manifest)
    }
    fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
        self.0.attach(subject, attestation)
    }
    fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>> {
        self.0.claims_of(digest)
    }
    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
        self.0.name(name, to, expected)
    }
    fn resolve(&self, name: &Name) -> Answer<Digest> {
        self.0.resolve(name)
    }
    fn read(&self, digest: &Digest) -> Answer<Held> {
        self.0.read(digest)
    }
}

#[test]
fn a_store_that_names_bytes_wrongly_is_refused_naming_both() {
    let world = World::new();
    let history = world.history();
    let mut seams = world.fakes.seams();
    seams.store = Box::new(Misnaming(world.fakes.store.clone()));
    let publish = publish(&seams, &history, &Kept::at(&history), &world.ask_for("lib")).unwrap();
    let r = refused(publish.components.into_iter().next().unwrap().1);
    let packed = world.fakes.store.calls()[1].given["bytes"].clone();
    let bytes: Vec<u8> = serde_json::from_value(packed).unwrap();
    assert_eq!(
        r.why,
        Why::OtherBytes {
            expected: Digest::of(&bytes),
            answered: Digest::of(b"not these bytes"),
        }
    );
    assert_eq!(r.source.seam, Seam::Store);
    assert!(indexes(&world).is_empty());
    assert!(world.fakes.store.contents().names.is_empty());
}

// Requirement: Each artifact carries one provenance claim in the host's identity

#[test]
fn each_artifact_carries_one_provenance_signed_by_the_host() {
    let world = World::new();
    let published = world.publish_one(&world.ask_for("lib")).unwrap();
    let Published::Version { artifacts, .. } = published else {
        unreachable!()
    };
    for artifact in &artifacts {
        let envelopes = world.fakes.store.claims_of(&artifact.digest).unwrap();
        assert_eq!(envelopes.len(), 1);
        assert_eq!(envelopes[0].sole().unwrap().signer.as_str(), HOST);
        let claim = Claim::from_envelope(&envelopes[0]).unwrap();
        assert_eq!(claim.kind, ClaimKind::Provenance);
        assert_eq!(claim.subject, artifact.digest);
        assert_eq!(claim.revision, world.head);
        assert_eq!(
            claim.build,
            Some(build_of(artifact.output.as_str(), &world.head))
        );
        let provenance = Provenance::of(&claim).unwrap();
        assert_eq!(provenance.output, artifact.output);
        assert_eq!(provenance.outcome, Outcome::Pass);
    }
}

#[test]
fn no_identity_at_hand_is_absent_and_offers_no_index() {
    let world = World::new();
    let history = world.history();
    let ask = Ask {
        signer: SignerId::new("nobody"),
        ..world.ask_for("lib")
    };
    let publish = publish(&world.fakes.seams(), &history, &Kept::at(&history), &ask).unwrap();
    let Err(Not::Absent(a)) = &publish.components[0].1 else {
        panic!("signed with no identity: {publish:?}")
    };
    assert!(a.what.contains("nobody"), "{}", a.what);
    assert_eq!(a.source.seam, Seam::Signer);
    assert!(indexes(&world).is_empty());
    assert_eq!(
        world
            .fakes
            .store
            .resolve(&Name::new("lib-v0.2.0"))
            .unwrap_err(),
        Not::absent(&Source::new(Seam::Store, "fake"), "name lib-v0.2.0")
    );
}

// Requirement: The landing's verdicts are carried as made

#[test]
fn the_verdicts_naming_a_build_are_carried_byte_for_byte() {
    let world = World::new();
    let pack = build_of("lib-pack", &world.head);
    let test = world.verdict("test", &pack, Outcome::Pass, LANDING);
    let lint = world.verdict("lint", &pack, Outcome::Pass, LANDING);
    world.verdict(
        "test",
        &build_of("lib-docs", &world.head),
        Outcome::Pass,
        LANDING,
    );
    let published = world.publish_one(&world.ask_for("lib")).unwrap();
    let Published::Version { artifacts, .. } = published else {
        unreachable!()
    };
    let envelope = &world.fakes.store.claims_of(&artifacts[0].digest).unwrap()[0];
    let provenance = Provenance::of(&Claim::from_envelope(envelope).unwrap()).unwrap();
    assert_eq!(provenance.carried.len(), 2);
    for (carried, verdict) in provenance.carried.iter().zip([&test, &lint]) {
        assert_eq!(carried.envelope, verdict.envelope);
        assert_eq!(carried.envelope.payload, verdict.envelope.payload);
        assert_eq!(carried.envelope.signatures, verdict.envelope.signatures);
        assert_eq!(carried.found.seam, Seam::Landing);
        assert_eq!(carried.found.revision, world.head);
        assert_eq!(carried.check, verdict.check);
    }
    let verdict_payloads = [&test.envelope.payload, &lint.envelope.payload];
    for envelopes in world.fakes.store.contents().attached.values() {
        for held in by_host(envelopes) {
            assert!(
                !verdict_payloads.contains(&&held.payload),
                "the host signed a verdict"
            );
        }
    }
    for call in world.fakes.signer.calls() {
        let kind = &call.given["claim"]["kind"];
        assert_eq!(
            kind, "provenance",
            "the host signed a claim of another kind"
        );
    }
}

// Requirement: Attestations are held beside their digest

#[test]
fn every_envelope_signed_is_held_beside_its_subject_and_nowhere_else() {
    let world = World::new();
    world.verdicts_for_every_output("test");
    world.publish_one(&world.ask_for("lib")).unwrap();
    let signed: Vec<Envelope> = world
        .fakes
        .signer
        .calls()
        .into_iter()
        .filter(|c| c.op == SignerOp::Sign)
        .map(|c| serde_json::from_value(c.answer.unwrap()).unwrap())
        .collect();
    assert_eq!(signed.len(), 2);
    for envelope in &signed {
        let subject = Claim::from_envelope(envelope).unwrap().subject;
        assert!(world
            .fakes
            .store
            .claims_of(&subject)
            .unwrap()
            .contains(envelope));
    }
    let record = world.kept();
    for entry in walk(record.dir()) {
        let text = std::fs::read_to_string(&entry).unwrap();
        assert!(!text.contains("payload"), "{text}");
        assert!(!text.contains("signature"), "{text}");
        for envelope in &signed {
            assert!(!text.contains(envelope.digest().as_str()), "{text}");
        }
    }
}

fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(walk(&path));
        } else {
            files.push(path);
        }
    }
    files
}

// Requirement: The index is offered last, then the version is named

#[test]
fn the_artifacts_and_their_attachments_come_before_the_index_and_the_name_after() {
    let world = World::new();
    world.publish_one(&world.ask_for("lib")).unwrap();
    let calls = world.fakes.store.calls();
    let order: Vec<(StoreOp, bool)> = calls
        .iter()
        .filter(|c| c.op != StoreOp::Resolve)
        .map(|c| (c.op, c.given["manifest"].get("index").is_some()))
        .collect();
    assert_eq!(
        order,
        [
            (StoreOp::PutBytes, false),
            (StoreOp::PutArtifact, false),
            (StoreOp::PutBytes, false),
            (StoreOp::PutArtifact, false),
            (StoreOp::Attach, false),
            (StoreOp::Attach, false),
            (StoreOp::PutArtifact, true),
            (StoreOp::Name, false),
        ]
    );
}

#[test]
fn a_publish_stopped_after_one_artifact_has_published_nothing() {
    let world = World::new();
    let history = world.history();
    let mut seams = world.fakes.seams();
    // The first output's bytes and artifact are put, then the second's
    // put fails.
    struct SecondFails(FakeStore, RefCell<u32>);
    impl Store for SecondFails {
        fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
            *self.1.borrow_mut() += 1;
            if *self.1.borrow() == 2 {
                return Err(Not::unreadable(
                    &Source::new(Seam::Store, "fake"),
                    "the put",
                    Some("connection refused".into()),
                ));
            }
            self.0.put_bytes(bytes)
        }
        fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
            self.0.put_artifact(manifest)
        }
        fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
            self.0.attach(subject, attestation)
        }
        fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>> {
            self.0.claims_of(digest)
        }
        fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
            self.0.name(name, to, expected)
        }
        fn resolve(&self, name: &Name) -> Answer<Digest> {
            self.0.resolve(name)
        }
        fn read(&self, digest: &Digest) -> Answer<Held> {
            self.0.read(digest)
        }
    }
    seams.store = Box::new(SecondFails(world.fakes.store.clone(), RefCell::new(0)));
    let publish = publish(&seams, &history, &Kept::at(&history), &world.ask_for("lib")).unwrap();
    let Err(Not::Unreadable(u)) = &publish.components[0].1 else {
        panic!("{publish:?}")
    };
    assert_eq!(u.said.as_deref(), Some("connection refused"));
    assert_eq!(
        world.fakes.store.contents().held.len(),
        2,
        "one artifact was put"
    );
    assert!(indexes(&world).is_empty());
    assert!(world.fakes.store.resolve(&Name::new("lib-v0.2.0")).is_err());
    assert!(matches!(
        world.kept().read(&Default::default()),
        Err(Not::Absent(_))
    ));
}

// Requirement: A second publish of a revision ends at the same index

#[test]
fn the_same_revision_published_again_ends_at_the_same_index_and_writes_nothing() {
    let world = World::new();
    world.verdicts_for_every_output("test");
    let first = world.publish(&world.ask()).unwrap();
    let store = world.fakes.store.contents();
    let tags = (
        world.repo.git(&["tag", "-l"]),
        world.shared.git(&["tag", "-l"]),
    );
    let record: Vec<(std::path::PathBuf, Vec<u8>)> = walk(world.kept().dir())
        .into_iter()
        .map(|p| {
            let bytes = std::fs::read(&p).unwrap();
            (p, bytes)
        })
        .collect();
    let calls_before = world.fakes.store.calls().len();

    let second = world.publish(&world.ask()).unwrap();
    assert_eq!(second, first);
    assert_eq!(world.fakes.store.contents(), store);
    let again = world.fakes.store.calls()[calls_before..].to_vec();
    assert!(again
        .iter()
        .all(|c| matches!(c.op, StoreOp::Resolve | StoreOp::Read | StoreOp::ClaimsOf)));
    assert_eq!(
        (
            world.repo.git(&["tag", "-l"]),
            world.shared.git(&["tag", "-l"])
        ),
        tags
    );
    let now: Vec<(std::path::PathBuf, Vec<u8>)> = walk(world.kept().dir())
        .into_iter()
        .map(|p| {
            let bytes = std::fs::read(&p).unwrap();
            (p, bytes)
        })
        .collect();
    assert_eq!(now, record);
}

#[test]
fn a_projection_that_packs_differently_is_refused_and_the_name_stays() {
    let mut world = World::new();
    let first = world.publish_one(&world.ask_for("lib")).unwrap();
    let (_, index) = version_of(&first);
    let Published::Version { artifacts, .. } = &first else {
        unreachable!()
    };
    world.fakes.projection(&eco("eco-a")).unsteady();
    let calls_before = world.fakes.store.calls().len();

    let r = refused(world.publish_one(&world.ask_for("lib")));
    let Why::Differed {
        artifact,
        first: held,
        now,
    } = &r.why
    else {
        panic!("{r:?}")
    };
    assert_eq!(artifact.ecosystem(), &eco("eco-a"));
    assert_eq!(artifact.name(), &EcosystemName::new("lib-pack"));
    assert_eq!(held.as_ref(), Some(&artifacts[0].bytes));
    assert_ne!(now, &artifacts[0].bytes);
    let again = world.fakes.store.calls()[calls_before..].to_vec();
    assert!(
        again
            .iter()
            .all(|c| matches!(c.op, StoreOp::Resolve | StoreOp::Read)),
        "{again:?}"
    );
    assert_eq!(
        world.fakes.store.resolve(&Name::new("lib-v0.2.0")).unwrap(),
        *index
    );
}

// Requirement: A publish answers what a program reads

#[test]
fn the_answer_names_every_digest_the_store_holds() {
    let world = World::new();
    let publish = world.publish(&world.ask_for("lib"));
    let json = answer_json(&publish);
    assert_eq!(json["answer"], "ok");
    let lib = &json["value"]["components"][0];
    assert_eq!(lib["component"], "lib");
    assert_eq!(lib["answer"], "ok");
    let value = &lib["value"];
    assert_eq!(value["published"], "version");
    assert_eq!(value["version"], "0.2.0");
    assert_eq!(value["tag"], "lib-v0.2.0");
    assert_eq!(value["revision"], world.head.as_str());

    let contents = world.fakes.store.contents();
    let index: Digest = serde_json::from_value(value["index"].clone()).unwrap();
    assert_eq!(contents.names[&Name::new("lib-v0.2.0")], index);
    let Held::Manifest(Manifest::Index(meta)) = &contents.held[&index] else {
        panic!("no index under {index}")
    };
    let artifacts = value["artifacts"].as_array().unwrap();
    assert_eq!(artifacts.len(), 2);
    for (artifact, held) in artifacts.iter().zip(meta.artifacts()) {
        assert_eq!(artifact["digest"], held.digest.as_str());
        assert_eq!(artifact["ecosystem"], held.meta.ecosystem().as_str());
        assert_eq!(artifact["name"], held.meta.name().as_str());
        assert_eq!(artifact["same_again"], true);
        let claims = artifact["claims"].as_array().unwrap();
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0]["kind"], "provenance");
        assert_eq!(claims[0]["signer"], HOST);
        assert_eq!(claims[0]["on"], held.digest.as_str());
        let attached = &contents.attached[&held.digest];
        assert_eq!(claims[0]["envelope"], attached[0].digest().as_str());
    }
}

// Requirement: A publish offers and does nothing more

#[test]
fn a_publish_moves_no_channel_and_reaches_no_target() {
    let mut world = World::new();
    let target = world.fakes.target(&EnvironmentName::new("production"));
    world.publish_one(&world.ask_for("lib")).unwrap();
    assert_eq!(world.fakes.keeper.calls(), []);
    assert_eq!(target.calls(), []);
    let named: Vec<_> = world
        .fakes
        .store
        .calls()
        .into_iter()
        .filter(|c| c.op == StoreOp::Name)
        .collect();
    assert_eq!(named.len(), 1);
    assert_eq!(named[0].given["name"], "lib-v0.2.0");
}
