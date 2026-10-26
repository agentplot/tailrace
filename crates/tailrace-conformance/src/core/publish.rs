//! The publish group: a landed revision offered as a version, S1 and S2,
//! and what publish refuses (2 to 20, 102, 110, 111).

use tailrace_core::{answer_json, publish, Provenance, Publish, Published};
use tailrace_fakes::{FakeStore, PROFILE};
use tailrace_seams::{
    Answer, Audience, BuildId, ChannelId, ChannelName, Claim, ClaimKind, Digest, Ecosystem,
    EcosystemName, Envelope, Held, Manifest, Name, Not, Outcome, Put, Required, Revision, Seam,
    Source, Store, Verdict, Version, Why,
};

use crate::world::{build_of, FakeWorld, Write, FILE, IMAGE, LANDING};
use crate::{ok, Scenario};

pub const GROUP: &str = "publish";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        Scenario {
            name: "a landed revision published",
            plays: Some("S1"),
            cites: &[6, 7, 11, 12, 13, 17, 19, 20, 22, 110],
            run: a_landed_revision_published,
        },
        Scenario {
            name: "the same revision again",
            plays: Some("S2"),
            cites: &[3, 5, 18],
            run: the_same_revision_again,
        },
        Scenario {
            name: "a revision not landed",
            plays: None,
            cites: &[14],
            run: a_revision_not_landed,
        },
        Scenario {
            name: "an output that did not pass",
            plays: None,
            cites: &[16, 111],
            run: an_output_that_did_not_pass,
        },
        Scenario {
            name: "a build no verdict names",
            plays: None,
            cites: &[15],
            run: a_build_no_verdict_names,
        },
        Scenario {
            name: "a publish that stops before its index",
            plays: None,
            cites: &[17],
            run: a_publish_that_stops_before_its_index,
        },
        Scenario {
            name: "a store that names bytes wrongly",
            plays: None,
            cites: &[2],
            run: a_store_that_names_bytes_wrongly,
        },
        Scenario {
            name: "a repeat that packs differently",
            plays: None,
            cites: &[18, 87],
            run: a_repeat_that_packs_differently,
        },
        Scenario {
            name: "channels declared or not",
            plays: None,
            cites: &[20, 102],
            run: channels_declared_or_not,
        },
        Scenario {
            name: "attestations beside their digest",
            plays: None,
            cites: &[8],
            run: attestations_beside_their_digest,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}

/// The world's two outputs, as the policy file declares them.
const OUTPUTS: [&str; 2] = ["site", "server"];

/// The version `head` takes: a feature since `v0.1.0`.
pub(crate) const VERSION: &str = "0.2.0";
pub(crate) const TAG: &str = "v0.2.0";

/// `revision` landed with a passing verdict of `test` naming each
/// output's build, and each output built and passing; the verdicts.
pub(crate) fn passing(world: &FakeWorld, revision: &Revision) -> Vec<Verdict> {
    let verdicts: Vec<Verdict> = OUTPUTS
        .iter()
        .map(|o| {
            world.verdict(
                revision,
                "test",
                &build_of(o, revision),
                Outcome::Pass,
                LANDING,
            )
        })
        .collect();
    world.land(revision, &verdicts);
    world.outputs(
        revision,
        &[("site", Outcome::Pass), ("server", Outcome::Pass)],
    );
    verdicts
}

/// The one component's answer of a publish that answered.
pub(crate) fn one(answer: Answer<Publish>) -> Answer<Published> {
    let publish = answer?;
    assert_eq!(
        publish.components.len(),
        1,
        "the publish answered {} components: {publish:?}",
        publish.components.len()
    );
    publish
        .components
        .into_iter()
        .next()
        .expect("one component")
        .1
}

/// The version a published answer names, and its index.
pub(crate) fn version_of(published: &Published) -> (&Version, &Digest) {
    match published {
        Published::Version { version, index, .. } => (version, index),
        Published::Unchanged { .. } => panic!("the component is unchanged: {published:?}"),
    }
}

fn refused(answer: Answer<Published>, asked: &str) -> tailrace_seams::Refusal {
    match answer {
        Err(Not::Refused(r)) => r,
        other => panic!("{asked} answered {other:?}, not refused"),
    }
}

/// Nothing was offered, packed or claimed: no write of the store, no pack
/// and no tag of the version in either history.
fn nothing_offered(world: &FakeWorld, case: &str) {
    assert_eq!(
        world.writes(),
        [],
        "{case}: the store was offered something"
    );
    assert_eq!(world.packs(), [], "{case}: an output was packed");
    for repo in [world.repo(), world.shared()] {
        assert_eq!(
            repo.tag_at(TAG),
            None,
            "{case}: the version's name was claimed"
        );
    }
}

fn a_landed_revision_published(world: &FakeWorld) {
    let verdicts = passing(world, &world.head);
    let published = ok(one(world.publish(&world.ask(&world.head))), "the publish");
    let Published::Version {
        component,
        version,
        tag,
        revision,
        index,
        artifacts,
        ..
    } = &published
    else {
        panic!("the landed revision was not published: {published:?}")
    };
    assert_eq!(
        (component.as_str(), version.as_str(), tag.as_str()),
        ("repository", VERSION, TAG),
        "the version is not the one history derives"
    );
    assert_eq!(revision, &world.head);

    // 6, 110: the index names the version's artifacts, each at its one
    // version.
    let store = &world.fakes.store;
    let Ok(Held::Manifest(Manifest::Index(meta))) = store.read(index) else {
        panic!("{index} holds no index")
    };
    assert_eq!((meta.component(), meta.version()), (component, version));
    let named: Vec<&Digest> = meta.artifacts().iter().map(|a| &a.digest).collect();
    let answered: Vec<&Digest> = artifacts.iter().map(|a| &a.digest).collect();
    assert_eq!(named, answered, "the index names other artifacts");
    let mut ecosystems: Vec<&str> = meta
        .artifacts()
        .iter()
        .map(|a| {
            assert_eq!(a.meta.version(), version, "{a:?} carries another version");
            a.meta.ecosystem().as_str()
        })
        .collect();
    ecosystems.sort_unstable();
    assert_eq!(ecosystems, [FILE, IMAGE], "two packages, one per ecosystem");

    // 17: every artifact and its claims before the index, the index
    // before the name.
    let writes = world.writes();
    let indexes: Vec<usize> = (0..writes.len())
        .filter(|i| writes[*i].is_index())
        .collect();
    assert_eq!(
        indexes.len(),
        1,
        "the index was put {} times",
        indexes.len()
    );
    let at = indexes[0];
    assert_eq!(
        writes[..at]
            .iter()
            .filter(|w| matches!(w, Write::Attach { .. }))
            .count(),
        2,
        "the index was put before every artifact's claim: {writes:?}"
    );
    assert!(
        writes[..at]
            .iter()
            .all(|w| !matches!(w, Write::Name { .. })),
        "a name was moved before the index was put: {writes:?}"
    );
    assert_eq!(
        &writes[at + 1..],
        [Write::Name {
            name: Name::new(TAG),
            to: index.clone(),
            expected: None,
        }],
        "after the index, the version's name was not made from nothing at it, alone"
    );

    // 7, 11 to 13: one provenance per artifact, in the host's identity,
    // about that artifact, naming the revision and the build, the
    // landing's verdict carried as made.
    for artifact in artifacts {
        let held = ok(store.claims_of(&artifact.digest), "claims-of");
        assert_eq!(held.len(), 1, "{} carries {held:?}", artifact.digest);
        let envelope = &held[0];
        assert_eq!(
            envelope.sole().map(|s| &s.signer),
            Some(&world.host),
            "signed by another"
        );
        let claim = Claim::from_envelope(envelope).expect("a claim of tailrace's form");
        assert_eq!(claim.kind, ClaimKind::Provenance);
        assert_eq!(claim.subject, artifact.digest, "about another digest");
        assert_eq!(claim.revision, world.head, "about another revision");
        let build = build_of(artifact.output.as_str(), &world.head);
        assert_eq!(claim.build.as_ref(), Some(&build), "about another build");
        let provenance = Provenance::of(&claim).expect("a provenance");
        let made = verdicts
            .iter()
            .find(|v| v.build == build)
            .expect("a verdict names the build");
        assert_eq!(provenance.carried.len(), 1, "{provenance:?}");
        let carried = &provenance.carried[0];
        assert_eq!(
            (&carried.envelope.payload, &carried.envelope.signatures),
            (&made.envelope.payload, &made.envelope.signatures),
            "the landing's verdict was not carried byte for byte"
        );
        assert_eq!(carried.found.seam, Seam::Landing);
        assert_eq!(artifact.claims.len(), 1);
        assert_eq!(artifact.claims[0].on, artifact.digest);
        assert_eq!(artifact.claims[0].envelope, envelope.digest());
    }
    for call in world.fakes.signer.calls() {
        assert_eq!(
            call.given["claim"]["kind"], "provenance",
            "the host signed a claim not its own to make"
        );
    }

    // 22: the name claimed at the revision in the shared history, before
    // any pack.
    assert_eq!(world.shared().tag_at(TAG).as_ref(), Some(&world.head));
    let packs = world.packs();
    assert_eq!(packs.len(), 2);
    for packing in &packs {
        assert_eq!(&packing.version, version);
        assert!(
            packing.shared_tags.iter().any(|t| t == TAG),
            "{} was packed before the name was claimed: {packing:?}",
            packing.ecosystem
        );
    }

    // 19: the answer names every digest and where each claim is attached.
    let json = answer_json(&Ok::<_, Not>(published.clone()));
    let value = &json["value"];
    assert_eq!(value["index"], index.as_str());
    assert_eq!(value["version"], VERSION);
    for (i, artifact) in artifacts.iter().enumerate() {
        let answered = &value["artifacts"][i];
        assert_eq!(answered["digest"], artifact.digest.as_str());
        assert_eq!(answered["ecosystem"], artifact.ecosystem.as_str());
        assert_eq!(answered["name"], artifact.name.as_str());
        assert_eq!(answered["claims"][0]["on"], artifact.digest.as_str());
    }

    // 6: the host keeps the index and nothing it names.
    for (path, bytes, _) in world.record() {
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            text.contains(index.as_str()),
            "{} names no index",
            path.display()
        );
        for artifact in artifacts {
            assert!(
                !text.contains(artifact.digest.as_str()),
                "{} keeps an artifact apart from the index",
                path.display()
            );
        }
    }

    // 20: nothing else moved.
    assert_eq!(world.keeper_calls(), [], "the keeper was asked");
    assert_eq!(world.target_calls(), [], "a target was asked");
}

fn the_same_revision_again(world: &FakeWorld) {
    passing(world, &world.head);
    let ask = world.ask(&world.head);
    let first = ok(world.publish(&ask), "the first publish");
    let held = world.fakes.store.contents();
    let writes = world.writes().len();
    let asked = world.fakes.store.calls().len();
    let tags = (world.repo().tags(), world.shared().tags());
    let record = world.record();

    let second = ok(world.publish(&ask), "the second publish");
    assert_eq!(second, first, "the second publish answered otherwise");
    assert_eq!(
        &world.writes()[writes..],
        [],
        "the second publish wrote to the store"
    );
    assert_eq!(world.fakes.store.contents(), held, "the store changed");
    assert_eq!(
        (world.repo().tags(), world.shared().tags()),
        tags,
        "a tag moved"
    );
    assert_eq!(world.record(), record, "the host's record changed");

    // 3: the name resolved first, and every read of the digest it named
    // or of the digests that index names.
    let (_, index) = version_of(first.components[0].1.as_ref().expect("published"));
    let calls = &world.fakes.store.calls()[asked..];
    assert_eq!(calls[0].op, tailrace_fakes::StoreOp::Resolve, "{calls:?}");
    assert_eq!(calls[0].given["name"], TAG);
    let Ok(Held::Manifest(Manifest::Index(meta))) = world.fakes.store.read(index) else {
        panic!("{index} holds no index")
    };
    let mut known: Vec<&str> = meta.artifacts().iter().map(|a| a.digest.as_str()).collect();
    known.push(index.as_str());
    for call in calls
        .iter()
        .filter(|c| c.op == tailrace_fakes::StoreOp::Read)
    {
        let read = call.given["digest"].as_str().expect("a digest");
        assert!(
            known.contains(&read),
            "the repeat read {read}, which the name does not resolve to"
        );
    }
}

fn a_revision_not_landed(world: &FakeWorld) {
    world.outputs(
        &world.head,
        &[("site", Outcome::Pass), ("server", Outcome::Pass)],
    );
    match world.publish(&world.ask(&world.head)) {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::NotLanded {
                branch: world.main.clone(),
                revision: world.head.clone()
            },
            "the refusal names another branch or revision"
        ),
        other => panic!("a revision not landed answered {other:?}"),
    }
    let landing: Vec<_> = world
        .fakes
        .landing
        .calls()
        .into_iter()
        .map(|c| c.op)
        .collect();
    assert_eq!(landing, [tailrace_fakes::LandingOp::Landed]);
    let fakes = &world.fakes;
    assert_eq!(
        world.asked(),
        landing.len(),
        "a seam but the landing was asked: store {:?}, build {:?}, signer {:?}, policy {:?}",
        fakes.store.calls(),
        fakes.build.calls(),
        fakes.signer.calls(),
        fakes.policy.calls()
    );
    nothing_offered(world, "a revision not landed");
}

fn an_output_that_did_not_pass(world: &FakeWorld) {
    let server = || tailrace_seams::OutputName::new("server");
    let built = |w: &FakeWorld| build_of("server", &w.head);

    // The server never built.
    let verdicts: Vec<Verdict> = OUTPUTS
        .iter()
        .map(|o| {
            world.verdict(
                &world.head,
                "test",
                &build_of(o, &world.head),
                Outcome::Pass,
                LANDING,
            )
        })
        .collect();
    world.land(&world.head, &verdicts);
    world.outputs(&world.head, &[("site", Outcome::Pass)]);
    let r = refused(
        one(world.publish(&world.ask(&world.head))),
        "an output unmade",
    );
    assert_eq!(
        r.why,
        Why::Unmade {
            outputs: vec![server()]
        }
    );
    nothing_offered(world, "an output unmade");

    // The server's build failed, though a passing verdict names it.
    let failed = FakeWorld::new();
    let verdicts: Vec<Verdict> = OUTPUTS
        .iter()
        .map(|o| {
            failed.verdict(
                &failed.head,
                "test",
                &build_of(o, &failed.head),
                Outcome::Pass,
                LANDING,
            )
        })
        .collect();
    failed.land(&failed.head, &verdicts);
    failed.outputs(
        &failed.head,
        &[("site", Outcome::Pass), ("server", Outcome::Fail)],
    );
    let r = refused(
        one(failed.publish(&failed.ask(&failed.head))),
        "a build that failed",
    );
    assert_eq!(
        r.why,
        Why::NotPassed {
            output: server(),
            build: built(&failed),
            outcome: Outcome::Fail
        }
    );
    assert_eq!(r.source.seam, Seam::Build);
    nothing_offered(&failed, "a build that failed");

    // Only a failing verdict names the server's build, which passed.
    let judged = FakeWorld::new();
    let verdicts = [
        judged.verdict(
            &judged.head,
            "test",
            &build_of("site", &judged.head),
            Outcome::Pass,
            LANDING,
        ),
        judged.verdict(
            &judged.head,
            "test",
            &built(&judged),
            Outcome::Fail,
            LANDING,
        ),
    ];
    judged.land(&judged.head, &verdicts);
    judged.outputs(
        &judged.head,
        &[("site", Outcome::Pass), ("server", Outcome::Pass)],
    );
    let r = refused(
        one(judged.publish(&judged.ask(&judged.head))),
        "a failing verdict",
    );
    assert_eq!(
        r.why,
        Why::NotPassed {
            output: server(),
            build: built(&judged),
            outcome: Outcome::Fail
        }
    );
    assert_eq!(r.source.seam, Seam::Landing);
    nothing_offered(&judged, "a failing verdict");
}

fn a_build_no_verdict_names(world: &FakeWorld) {
    let elsewhere = BuildId::new("elsewhere");
    let site = build_of("site", &world.head);
    world.land(
        &world.head,
        &[
            world.verdict(&world.head, "test", &site, Outcome::Pass, LANDING),
            world.verdict(&world.head, "test", &elsewhere, Outcome::Pass, LANDING),
        ],
    );
    world.outputs(
        &world.head,
        &[("site", Outcome::Pass), ("server", Outcome::Pass)],
    );
    let r = refused(
        one(world.publish(&world.ask(&world.head))),
        "a build no verdict names",
    );
    assert_eq!(
        r.why,
        Why::Unnamed {
            output: tailrace_seams::OutputName::new("server"),
            build: build_of("server", &world.head),
            carried: vec![site, elsewhere],
        }
    );
    nothing_offered(world, "a build no verdict names");
}

/// A store over the fake that refuses every index it is given.
struct RefusingIndex(FakeStore);

/// A store over the fake that names every bytes it is given by another
/// digest.
struct Misnaming(FakeStore);

/// What `Misnaming` names every bytes by.
const NOT_THESE: &[u8] = b"not these bytes";

/// The store's operations handed to the fake, but for the puts each
/// wrapper answers its own way.
macro_rules! delegate {
    ($store:ident) => {
        impl Store for $store {
            fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
                self.put_bytes_as(bytes)
            }
            fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
                self.put_artifact_as(manifest)
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
    };
}

delegate!(RefusingIndex);
delegate!(Misnaming);

impl RefusingIndex {
    fn put_bytes_as(&self, bytes: &[u8]) -> Answer<Put> {
        self.0.put_bytes(bytes)
    }

    fn put_artifact_as(&self, manifest: &Manifest) -> Answer<Put> {
        match manifest {
            Manifest::Index(_) => Err(Not::refused(
                &Source::new(Seam::Store, PROFILE),
                Why::Said,
                Some("the index is refused".into()),
            )),
            Manifest::Artifact { .. } => self.0.put_artifact(manifest),
        }
    }
}

impl Misnaming {
    fn put_bytes_as(&self, bytes: &[u8]) -> Answer<Put> {
        let put = self.0.put_bytes(bytes)?;
        Ok(Put {
            digest: Digest::of(NOT_THESE),
            ..put
        })
    }

    fn put_artifact_as(&self, manifest: &Manifest) -> Answer<Put> {
        self.0.put_artifact(manifest)
    }
}

/// `world`'s publish of its head over its seams with `store` bound in the
/// fake store's place.
fn over(world: &FakeWorld, store: Box<dyn Store>) -> Answer<Published> {
    let mut seams = world.seams();
    seams.store = store;
    let history = world.history();
    let kept = world.kept();
    one(publish(&seams, &history, &kept, &world.ask(&world.head)))
}

fn a_publish_that_stops_before_its_index(world: &FakeWorld) {
    passing(world, &world.head);
    let answer = over(world, Box::new(RefusingIndex(world.fakes.store.clone())));
    let r = refused(answer, "a publish whose index is refused");
    assert_eq!(r.said.as_deref(), Some("the index is refused"));
    let held = world.fakes.store.contents().held;
    assert!(
        held.values()
            .any(|h| matches!(h, Held::Manifest(Manifest::Artifact { .. }))),
        "no artifact was put before the index"
    );
    assert!(
        !held
            .values()
            .any(|h| matches!(h, Held::Manifest(Manifest::Index(_)))),
        "an index is held"
    );
    match world.fakes.store.resolve(&Name::new(TAG)) {
        Err(Not::Absent(_)) => {}
        other => panic!("the version's name names {other:?}"),
    }
    assert_eq!(world.record(), [], "the host recorded a version");
}

fn a_store_that_names_bytes_wrongly(world: &FakeWorld) {
    passing(world, &world.head);
    let answer = over(world, Box::new(Misnaming(world.fakes.store.clone())));
    let r = refused(answer, "a publish to a store that names bytes wrongly");
    let Some(Write::Bytes(expected)) = world.writes().into_iter().next() else {
        panic!("no bytes were put")
    };
    assert_eq!(
        r.why,
        Why::OtherBytes {
            expected,
            answered: Digest::of(NOT_THESE)
        },
        "the refusal names other digests"
    );
    assert!(
        !world.writes().iter().any(Write::is_index),
        "an index was put"
    );
    assert!(
        world.fakes.store.contents().names.is_empty(),
        "a name moved"
    );
}

fn a_repeat_that_packs_differently(world: &FakeWorld) {
    passing(world, &world.head);
    let ask = world.ask(&world.head);
    let first = ok(one(world.publish(&ask)), "the first publish");
    let (_, index) = version_of(&first);
    let Published::Version { artifacts, .. } = &first else {
        unreachable!()
    };
    let site = artifacts
        .iter()
        .find(|a| a.ecosystem.as_str() == FILE)
        .expect("the site was published");
    world.fakes.projections[&Ecosystem::new(FILE)].unsteady();
    let writes = world.writes().len();

    let r = refused(one(world.publish(&ask)), "a repeat that packs differently");
    let Why::Differed {
        artifact,
        first: held,
        now,
    } = &r.why
    else {
        panic!("the repeat was refused for {:?}", r.why)
    };
    assert_eq!(
        (artifact.ecosystem(), artifact.name()),
        (&Ecosystem::new(FILE), &EcosystemName::new("site")),
        "the refusal names another artifact"
    );
    assert_eq!(
        held.as_ref(),
        Some(&site.bytes),
        "the first digest is not named"
    );
    assert_ne!(now, &site.bytes, "the digest packed now is the first");
    assert_eq!(
        &world.writes()[writes..],
        [],
        "the repeat wrote to the store"
    );
    assert_eq!(
        world.fakes.store.resolve(&Name::new(TAG)).as_ref(),
        Ok(index),
        "the version's name left the first index"
    );
}

fn channels_declared_or_not(world: &FakeWorld) {
    let declared = FakeWorld::new();
    for w in [world, &declared] {
        passing(w, &w.head);
    }
    declared.fakes.policy.require(
        Audience::Channel(ChannelId::new(None, ChannelName::new(crate::world::STABLE))),
        Required::Channel {
            claims: vec![],
            bound_to: None,
            promoted_by: vec![],
        },
    );
    let plain = ok(world.publish(&world.ask(&world.head)), "the publish");
    let with = ok(
        declared.publish(&declared.ask(&declared.head)),
        "the publish where a channel is declared",
    );
    assert_eq!(
        with, plain,
        "declaring a channel changed what a publish answers"
    );
    for (w, case) in [(world, "no channel"), (&declared, "a channel declared")] {
        assert_eq!(w.keeper_calls(), [], "{case}: the keeper was asked");
        assert_eq!(w.target_calls(), [], "{case}: a target was asked");
    }
}

fn attestations_beside_their_digest(world: &FakeWorld) {
    passing(world, &world.head);
    ok(one(world.publish(&world.ask(&world.head))), "the publish");
    let store = &world.fakes.store;
    let signed: Vec<Envelope> = world
        .fakes
        .signer
        .calls()
        .into_iter()
        .filter_map(|c| c.answer.ok())
        .map(|e| serde_json::from_value(e).expect("an envelope"))
        .collect();
    assert_eq!(signed.len(), 2, "one claim per artifact");
    for envelope in &signed {
        let subject = Claim::from_envelope(envelope)
            .expect("a claim of tailrace's form")
            .subject;
        assert!(
            ok(store.claims_of(&subject), "claims-of").contains(envelope),
            "a claim is not held beside {subject}"
        );
    }
    for write in world.writes() {
        if let Write::Attach { subject, envelope } = write {
            assert_eq!(
                Claim::from_envelope(&envelope).map(|c| c.subject),
                Some(subject.clone()),
                "a claim was attached beside another digest"
            );
        }
    }
    for (path, bytes, _) in world.record() {
        let text = String::from_utf8_lossy(&bytes);
        for envelope in &signed {
            assert!(
                !text.contains(envelope.digest().as_str()) && !text.contains("signature"),
                "{} keeps a claim apart from its digest",
                path.display()
            );
        }
    }
}
