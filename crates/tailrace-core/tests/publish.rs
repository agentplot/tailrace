//! Spec publishing: one landed revision turned into a version of each
//! component it publishes, over the fakes and a shared history.

mod common;
mod world;

use std::cell::RefCell;
use std::rc::Rc;

use tailrace_core::{
    answer_json, layout_policy_body, publish, Ask, Kept, Provenance, Published, BOUND, NOTES,
};
use tailrace_fakes::{
    BuildOp, FakeProjection, FakeStore, Fault, LandingOp, PolicyOp, SignerOp, StoreOp, Times,
};
use tailrace_seams::{
    Answer, Artifact, Audience, Branch, BuildId, Claim, ClaimKind, ComponentName, Contents, Digest,
    EcosystemName, Envelope, EnvironmentName, Held, IndexMeta, Layout, Manifest, Name, NamedSigner,
    Not, Outcome, Output, Packed, Pinned, Projection, Put, Range, Rendered, Required,
    RequiredClaim, Requirement, Seam, Serving, SignerId, Source, Store, Version, Why,
};
use world::{
    build_of, build_ops, by_host, eco, output, published_before_notes, version_of, Watched, World,
    HOST, LANDING,
};

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
fn an_output_packed_as_notes_is_refused_before_the_build_is_asked() {
    let world =
        World::with_policy(&world::POLICY.replace("lib-docs = \"eco-b\"", "lib-docs = \"notes\""));
    let Err(Not::Refused(r)) = world.publish(&world.ask()) else {
        panic!("a policy file packing an output as notes was published")
    };
    assert_eq!(r.why, Why::Said);
    let said = r.said.unwrap();
    assert!(said.contains("component lib"), "{said}");
    assert!(said.contains("lib-docs"), "{said}");
    assert_eq!(world.fakes.build.calls(), []);
    no_pack(&world);
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
    assert_eq!(r.source, world::store());
    assert!(indexes(&world).is_empty());
    assert!(world.fakes.store.contents().names.is_empty());
}

/// A projection that packs as its fake does and names no metadata,
/// whatever it was given.
struct Metaless(FakeProjection);

impl Projection for Metaless {
    fn pack(&self, output: &Output, at: &BuildId, version: Option<&Version>) -> Answer<Packed> {
        let packed = self.0.pack(output, at, version)?;
        Ok(Packed {
            meta: None,
            ..packed
        })
    }
    fn render(&self, contents: Contents<'_>, at: &str) -> Answer<Rendered> {
        self.0.render(contents, at)
    }
    fn serve(&self, layout: &Layout) -> Answer<Serving> {
        self.0.serve(layout)
    }
    fn pinned(&self, checkout: &str) -> Answer<Vec<Pinned>> {
        self.0.pinned(checkout)
    }
    fn requires(&self, _: &Artifact) -> Answer<Vec<Requirement>> {
        Ok(Vec::new())
    }
    fn satisfying(&self, _: &Range, _: &[Version]) -> Answer<Vec<Version>> {
        Ok(Vec::new())
    }
}

#[test]
fn a_pack_given_a_version_that_names_no_metadata_is_refused_naming_the_output() {
    let mut world = World::new();
    let inner = world.fakes.projection(&eco("eco-a"));
    let history = world.history();
    let mut seams = world.fakes.seams();
    seams
        .projections
        .insert(eco("eco-a"), Box::new(Metaless(inner)));
    let answer = publish(&seams, &history, &Kept::at(&history), &world.ask_for("lib")).unwrap();
    let r = refused(answer.components.into_iter().next().unwrap().1);
    assert_eq!(r.source, Source::new(Seam::Projection, "eco-a"));
    assert_eq!(r.why, Why::Said);
    assert_eq!(
        r.said.as_deref(),
        Some("lib-pack was packed at 0.2.0 and the answer names no metadata")
    );
    assert!(
        !store_ops(&world).contains(&StoreOp::PutArtifact),
        "an artifact was put: {:?}",
        store_ops(&world)
    );
    nothing_offered(&world);
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
    assert_eq!(
        signed.len(),
        3,
        "two outputs' provenance and the notes' claim"
    );
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

/// What a store call put: the ecosystem of an artifact's manifest,
/// `index` for an index's, and nothing for any other call.
fn manifest_of(call: &tailrace_fakes::Call<StoreOp>) -> Option<String> {
    let manifest = call.given.get("manifest")?;
    match manifest.get("artifact") {
        Some(artifact) => artifact["meta"]["ecosystem"].as_str().map(str::to_string),
        None => Some("index".into()),
    }
}

#[test]
fn the_artifacts_and_their_attachments_come_before_the_index_and_the_name_after() {
    let world = World::new();
    world.publish_one(&world.ask_for("lib")).unwrap();
    let calls = world.fakes.store.calls();
    let order: Vec<(StoreOp, Option<String>)> = calls
        .iter()
        .filter(|c| c.op != StoreOp::Resolve)
        .map(|c| (c.op, manifest_of(c)))
        .collect();
    let put = |ecosystem: &str| (StoreOp::PutArtifact, Some(ecosystem.to_string()));
    assert_eq!(
        order,
        [
            (StoreOp::PutBytes, None),
            put("eco-a"),
            (StoreOp::PutBytes, None),
            put("eco-b"),
            (StoreOp::PutBytes, None),
            put("notes"),
            (StoreOp::Attach, None),
            (StoreOp::Attach, None),
            (StoreOp::Attach, None),
            put("index"),
            (StoreOp::Name, None),
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

    let notes = &value["notes"];
    let held = meta.artifacts().last().unwrap();
    assert_eq!(held.meta.ecosystem().as_str(), NOTES);
    assert_eq!(notes["digest"], held.digest.as_str());
    let Held::Manifest(Manifest::Artifact { layers, .. }) = &contents.held[&held.digest] else {
        panic!("no artifact under {}", held.digest)
    };
    assert_eq!(notes["bytes"], layers[0].as_str());
    let Held::Bytes(bytes) = &contents.held[&layers[0]] else {
        panic!("no bytes under {}", layers[0])
    };
    assert_eq!(notes["text"].as_str().unwrap().as_bytes(), bytes.as_slice());
    let claims = notes["claims"].as_array().unwrap();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0]["kind"], "provenance");
    assert_eq!(claims[0]["signer"], HOST);
    assert_eq!(claims[0]["on"], held.digest.as_str());
    let attached = &contents.attached[&held.digest];
    assert_eq!(claims[0]["envelope"], attached[0].digest().as_str());
}

#[test]
fn the_entry_and_the_answer_name_the_store_as_the_ask_names_it() {
    let world = World::new();
    let store = Source::new(Seam::Store, "layout /srv/elsewhere/layout");
    let ask = Ask {
        store: store.clone(),
        ..world.ask_for("lib")
    };
    let Ok(Published::Version {
        store: answered, ..
    }) = world.publish_one(&ask)
    else {
        panic!("lib was not published")
    };
    assert_eq!(answered, store);
    let entries = world
        .kept()
        .read(&tailrace_core::Filter::default())
        .unwrap();
    let kept: Vec<&Source> = entries.iter().map(|e| &e.store).collect();
    assert_eq!(kept, [&store]);
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

// Requirement: The index carries the publisher's layout policy

/// One claim of `kind` by `signer`, as a policy for layouts names it.
fn required(kind: ClaimKind, signer: &str) -> RequiredClaim {
    RequiredClaim {
        kind,
        check: None,
        signers: vec![NamedSigner {
            id: SignerId::new(signer),
            verified_by: format!("{signer}'s key"),
            kinds: vec![kind],
            checks: vec![],
        }],
    }
}

/// The layout-policy claims attached to the index `published` names.
fn layout_policies(world: &World, published: &Published) -> Vec<(Envelope, Claim)> {
    let (_, index) = version_of(published);
    world
        .fakes
        .store
        .claims_of(index)
        .unwrap()
        .into_iter()
        .filter_map(|e| {
            let claim = Claim::from_envelope(&e)?;
            (claim.kind == ClaimKind::LayoutPolicy).then_some((e, claim))
        })
        .collect()
}

/// `world`'s policy for layouts requiring `claims`: a layout line its
/// file writes, or the given its file answers where it writes none.
fn layout(world: &World, claims: Vec<RequiredClaim>, given: bool) {
    world.fakes.policy.require(
        Audience::Layout,
        Required::Layout {
            claims,
            given,
            signers: vec![],
        },
    );
}

#[test]
fn a_publish_under_a_layout_line_attaches_its_policy_to_the_index_before_the_name() {
    let world = World::new();
    let claims = vec![required(ClaimKind::Provenance, HOST)];
    layout(&world, claims.clone(), false);
    let published = world.publish_one(&world.ask_for("lib")).unwrap();
    let [(envelope, claim)] = <[_; 1]>::try_from(layout_policies(&world, &published)).unwrap();
    let (_, index) = version_of(&published);
    assert_eq!(claim.subject, *index);
    assert_eq!(claim.revision, world.head);
    assert_eq!(claim.build, None);
    assert_eq!(claim.body, layout_policy_body(&claims));
    assert_eq!(envelope.sole().unwrap().signer, SignerId::new(HOST));

    let asked: Vec<_> = world
        .fakes
        .policy
        .calls()
        .into_iter()
        .filter(|c| c.given["audience"] == "layout")
        .collect();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].given["revision"], world.head.as_str());

    let order: Vec<(StoreOp, bool)> = world
        .fakes
        .store
        .calls()
        .iter()
        .filter(|c| !matches!(c.op, StoreOp::Resolve | StoreOp::ClaimsOf))
        .map(|c| (c.op, c.given["manifest"].get("index").is_some()))
        .collect();
    assert_eq!(
        order[order.len() - 3..],
        [
            (StoreOp::PutArtifact, true),
            (StoreOp::Attach, false),
            (StoreOp::Name, false),
        ],
        "the layout policy was not attached between the index and the name"
    );
}

#[test]
fn a_publish_with_no_layout_line_attaches_nothing_of_the_given() {
    let world = World::new();
    layout(&world, vec![required(ClaimKind::Acceptance, "gate")], true);
    let published = world.publish_one(&world.ask_for("lib")).unwrap();
    assert_eq!(layout_policies(&world, &published), []);
    let (_, index) = version_of(&published);
    assert_eq!(world.fakes.store.claims_of(index).unwrap(), []);
    let signed: Vec<_> = world
        .fakes
        .signer
        .calls()
        .into_iter()
        .filter(|c| c.op == SignerOp::Sign && c.given["claim"]["kind"] == "layout_policy")
        .collect();
    assert_eq!(signed, [], "the given was signed");
}

#[test]
fn a_publish_with_no_policy_file_attaches_no_layout_policy() {
    let world = World::new();
    let published = world.publish_one(&world.ask_for("lib")).unwrap();
    assert_eq!(layout_policies(&world, &published), []);
    let (_, index) = version_of(&published);
    assert_eq!(world.fakes.store.claims_of(index).unwrap(), []);
}

#[test]
fn an_index_put_again_carrying_the_policy_has_nothing_more_signed() {
    let world = World::new();
    layout(&world, vec![required(ClaimKind::Provenance, HOST)], false);
    world.fakes.store.fault(
        StoreOp::Name,
        Fault::Unreadable("connection refused".into()),
        Times::Once,
    );
    let stopped = world.publish_one(&world.ask_for("lib"));
    assert!(matches!(stopped, Err(Not::Unreadable(_))), "{stopped:?}");
    let published = world.publish_one(&world.ask_for("lib")).unwrap();
    assert_eq!(layout_policies(&world, &published).len(), 1);
    let again = world.publish_one(&world.ask_for("lib")).unwrap();
    assert_eq!(again, published);
    assert_eq!(layout_policies(&world, &again).len(), 1);
    let signed: Vec<_> = world
        .fakes
        .signer
        .calls()
        .into_iter()
        .filter(|c| c.op == SignerOp::Sign && c.given["claim"]["kind"] == "layout_policy")
        .collect();
    assert_eq!(signed.len(), 1, "the layout policy was signed again");
}

#[test]
fn a_policy_for_layouts_that_cannot_be_read_stops_the_publish_before_the_name() {
    let world = World::new();
    world.fakes.policy.fault(
        PolicyOp::PolicyFor,
        Fault::Unreadable("the policy file does not parse".into()),
        Times::UntilHealed,
    );
    let answer = world.publish_one(&world.ask_for("lib"));
    assert!(
        matches!(&answer, Err(Not::Unreadable(u)) if u.said.as_deref() == Some("the policy file does not parse")),
        "{answer:?}"
    );
    assert!(world.fakes.store.resolve(&Name::new("lib-v0.2.0")).is_err());
}

// Requirement: The notes are an artifact of the version named in its index

/// The notes `world`'s first publish of `lib` renders under the
/// repository's own template: its one feature since `lib-v0.1.0`.
fn lib_notes(world: &World) -> String {
    format!(
        "## lib-v0.2.0\n\nSince lib-v0.1.0.\n\n### Features\n\n- lib grows ({}) by A. Uthor\n",
        &world.head.as_str()[..7]
    )
}

/// The artifact of ecosystem `notes` the index `published` names.
fn notes_artifact(world: &World, published: &Published) -> tailrace_seams::Artifact {
    let (_, index) = version_of(published);
    let Held::Manifest(Manifest::Index(meta)) = &world.fakes.store.contents().held[index] else {
        panic!("no index under {index}")
    };
    let notes: Vec<_> = meta
        .artifacts()
        .iter()
        .filter(|a| a.meta.ecosystem() == &eco(NOTES))
        .cloned()
        .collect();
    let [notes] = <[_; 1]>::try_from(notes).unwrap();
    notes
}

#[test]
fn the_index_names_the_two_outputs_and_the_notes_last() {
    let world = World::new();
    let published = world.publish_one(&world.ask_for("lib")).unwrap();
    let (_, index) = version_of(&published);
    let held = world.fakes.store.contents().held;
    let Held::Manifest(Manifest::Index(meta)) = &held[index] else {
        panic!("no index under {index}")
    };
    let ecosystems: Vec<&str> = meta
        .artifacts()
        .iter()
        .map(|a| a.meta.ecosystem().as_str())
        .collect();
    assert_eq!(ecosystems, ["eco-a", "eco-b", "notes"]);

    let notes = notes_artifact(&world, &published);
    assert_eq!(notes.meta.name(), &EcosystemName::new("lib"));
    assert_eq!(notes.meta.version(), &Version::new("0.2.0"));
    let Held::Manifest(Manifest::Artifact { layers, .. }) = &held[&notes.digest] else {
        panic!("no artifact under {}", notes.digest)
    };
    let text = lib_notes(&world);
    assert_eq!(layers, &[Digest::of(text.as_bytes())]);
    assert_eq!(held[&layers[0]], Held::Bytes(text.into_bytes()));
}

#[test]
fn a_name_held_by_another_revision_puts_nothing_of_the_notes() {
    let world = World::new();
    // The name is held at a revision of another branch, which the head
    // does not reach.
    world
        .repo
        .git(&["checkout", "-q", "-b", "side", world.start.as_str()]);
    let side = world.repo.touch("fix: elsewhere", &["crates/lib"]);
    world.repo.git(&["checkout", "-q", "main"]);
    world.repo.tag("lib-v0.2.0", &side);
    let r = refused(world.publish_one(&world.ask_for("lib")));
    assert!(matches!(r.why, Why::NameHeld { .. }), "{r:?}");
    assert_eq!(puts(&world), [], "the store was offered something");
    assert!(!world
        .fakes
        .store
        .calls()
        .iter()
        .any(|c| manifest_of(c).as_deref() == Some(NOTES)));
    nothing_offered(&world);
}

/// A store that puts as its fake does, and fails every index put.
struct NoIndex(FakeStore);

impl Store for NoIndex {
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        self.0.put_bytes(bytes)
    }
    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        if let Manifest::Index(_) = manifest {
            return Err(Not::unreadable(
                &Source::new(Seam::Store, "fake"),
                "the put of the index",
                Some("connection refused".into()),
            ));
        }
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
fn a_publish_stopped_after_the_notes_keeps_no_notes_as_a_version() {
    let world = World::new();
    let history = world.history();
    let mut seams = world.fakes.seams();
    seams.store = Box::new(NoIndex(world.fakes.store.clone()));
    let publish = publish(&seams, &history, &Kept::at(&history), &world.ask_for("lib")).unwrap();
    let Err(Not::Unreadable(u)) = &publish.components[0].1 else {
        panic!("{publish:?}")
    };
    assert_eq!(u.said.as_deref(), Some("connection refused"));
    let held = world.fakes.store.contents().held;
    assert!(held.values().any(|h| matches!(
        h,
        Held::Manifest(Manifest::Artifact { meta, .. }) if meta.ecosystem() == &eco(NOTES)
    )));
    assert!(indexes(&world).is_empty());
    assert!(matches!(
        world.fakes.store.resolve(&Name::new("lib-v0.2.0")),
        Err(Not::Absent(_))
    ));
}

// Requirement: The notes carry the publisher's own claim

#[test]
fn the_notes_carry_one_provenance_of_the_hosts_naming_every_commit_and_no_build() {
    let world = World::new();
    let published = world.publish_one(&world.ask_for("lib")).unwrap();
    let notes = notes_artifact(&world, &published);
    let envelopes = world.fakes.store.claims_of(&notes.digest).unwrap();
    let [envelope] = <[_; 1]>::try_from(envelopes).unwrap();
    assert_eq!(envelope.sole().unwrap().signer.as_str(), HOST);
    let claim = Claim::from_envelope(&envelope).unwrap();
    assert_eq!(claim.kind, ClaimKind::Provenance);
    assert_eq!(claim.subject, notes.digest);
    assert_eq!(claim.revision, world.head);
    assert_eq!(claim.build, None);
    assert_eq!(
        claim.body,
        serde_json::json!({
            "notes": {
                "component": "lib",
                "version": "0.2.0",
                "last": "lib-v0.1.0",
                "commits": [world.head.as_str()],
            }
        })
    );
    assert_eq!(Provenance::of(&claim), None);
}

#[test]
fn no_identity_at_hand_puts_no_index_naming_the_notes() {
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
    assert_eq!(a.source.seam, Seam::Signer);
    assert!(a.what.contains("nobody"), "{}", a.what);
    assert!(indexes(&world).is_empty());
    assert!(world.fakes.store.contents().attached.is_empty());
    assert!(world.fakes.store.contents().names.is_empty());
}

// Requirement: Nothing writes a version's notes after it is offered

#[test]
fn the_same_notes_again_put_and_attach_nothing() {
    let world = World::new();
    let first = world.publish_one(&world.ask_for("lib")).unwrap();
    let notes = notes_artifact(&world, &first);
    let store = world.fakes.store.contents();
    let calls_before = world.fakes.store.calls().len();

    let second = world.publish_one(&world.ask_for("lib")).unwrap();
    assert_eq!(version_of(&second), version_of(&first));
    let again = world.fakes.store.calls()[calls_before..].to_vec();
    assert!(
        again
            .iter()
            .all(|c| matches!(c.op, StoreOp::Resolve | StoreOp::Read | StoreOp::ClaimsOf)),
        "{again:?}"
    );
    assert_eq!(world.fakes.store.contents(), store);
    assert_eq!(notes_artifact(&world, &second), notes);
    let notes_of = |p: &Published| match p {
        Published::Version { notes, .. } => notes.clone().unwrap(),
        Published::Unchanged { .. } => panic!("unchanged: {p:?}"),
    };
    assert_eq!(notes_of(&second), notes_of(&first));
    assert_eq!(notes_of(&second).digest, notes.digest);
}

#[test]
fn notes_that_render_differently_are_refused_naming_both_digests() {
    let world = World::new();
    let first = world.publish_one(&world.ask_for("lib")).unwrap();
    let (_, index) = version_of(&first);
    let notes = notes_artifact(&world, &first);

    // The index's notes replaced by an artifact of other bytes, and the
    // name moved to the index naming it.
    let store = &world.fakes.store;
    let Held::Manifest(Manifest::Index(meta)) = store.read(index).unwrap() else {
        panic!("no index under {index}")
    };
    let other = store
        .put_bytes(b"## 0.2.0\n\nOther notes.\n")
        .unwrap()
        .digest;
    let replaced = store
        .put_artifact(&Manifest::Artifact {
            meta: notes.meta.clone(),
            layers: vec![other.clone()],
        })
        .unwrap()
        .digest;
    let artifacts = meta
        .artifacts()
        .iter()
        .map(|a| match a.digest == notes.digest {
            true => Artifact {
                digest: replaced.clone(),
                meta: a.meta.clone(),
            },
            false => a.clone(),
        })
        .collect();
    let elsewhere = IndexMeta::new(meta.component().clone(), meta.version().clone(), artifacts);
    let moved = store
        .put_artifact(&Manifest::Index(elsewhere.unwrap()))
        .unwrap()
        .digest;
    let name = Name::new("lib-v0.2.0");
    store.name(&name, &moved, Some(index)).unwrap();
    let calls_before = store.calls().len();

    let r = refused(world.publish_one(&world.ask_for("lib")));
    assert_eq!(
        r.why,
        Why::Differed {
            artifact: notes.meta.clone(),
            first: Some(other),
            now: Digest::of(lib_notes(&world).as_bytes()),
        }
    );
    let again = store.calls()[calls_before..].to_vec();
    assert!(
        again
            .iter()
            .all(|c| matches!(c.op, StoreOp::Resolve | StoreOp::Read)),
        "{again:?}"
    );
    assert_eq!(store.resolve(&name).unwrap(), moved);
}

#[test]
fn a_version_published_before_notes_is_published_again_with_none_added() {
    let world = World::new();
    let first = published_before_notes(&world);
    let ecosystems: Vec<String> = indexes(&world)
        .iter()
        .flat_map(|i| i.artifacts().iter().map(|a| a.meta.ecosystem().to_string()))
        .collect();
    assert_eq!(ecosystems, ["eco-a", "eco-b"]);
    let store = world.fakes.store.contents();
    let record = std::fs::read_dir(world.kept().dir().join("lib"))
        .unwrap()
        .map(|e| std::fs::read(e.unwrap().path()).unwrap())
        .collect::<Vec<_>>();
    let calls_before = world.fakes.store.calls().len();

    let second = world.publish_one(&world.ask_for("lib")).unwrap();
    assert_eq!(version_of(&second), version_of(&first));
    assert!(
        matches!(&second, Published::Version { notes: None, .. }),
        "{second:?}"
    );
    let json = serde_json::to_value(&second).unwrap();
    assert!(json.get("notes").is_none(), "{json}");
    let again = world.fakes.store.calls()[calls_before..].to_vec();
    assert!(
        again
            .iter()
            .all(|c| matches!(c.op, StoreOp::Resolve | StoreOp::Read | StoreOp::ClaimsOf)),
        "{again:?}"
    );
    assert_eq!(world.fakes.store.contents(), store);
    let now = std::fs::read_dir(world.kept().dir().join("lib"))
        .unwrap()
        .map(|e| std::fs::read(e.unwrap().path()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(now, record);
}
