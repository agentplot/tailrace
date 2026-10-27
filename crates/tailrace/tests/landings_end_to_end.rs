//! The core's publish and verify over moon and a real landing, the store,
//! the signer, the policy and the projection the fakes (design D9): the
//! build identity moon reports is the key sluice's verdict carries (13);
//! an output whose task the gate never checked is refused naming it and
//! every identity the verdicts carry, and nothing is offered (15); and S13
//! over a plain branch and moon, as far as verify's answer. Needs moon
//! and sluice, so it is in the tools layer.

#![cfg(feature = "tools")]

mod scratch;

use scratch::gate::{Gate, SIGNER};
use scratch::recorder::Recorder;
use scratch::{moon_workspace, policy_file, Scratch, Task, MAIN};
use tailrace::profiles::build::moon::Moon;
use tailrace::profiles::landing::branch::PlainBranch;
use tailrace::profiles::landing::sluice::{as_written, Sluice};
use tailrace_core::{
    publish, verify, Ask, History, Kept, Provenance, Published, Verified, VerifyAsk,
};
use tailrace_fakes::{Fakes, StoreOp};
use tailrace_seams::{
    Audience, Branch, Build, Check, Claim, ClaimKind, Ecosystem, Landing, NamedSigner, Not,
    OutputName, Read, Required, RequiredClaim, Revision, Seam, Seams, SignerId, Source, Store,
    Version, Why,
};

/// The identity of the party running tailrace, which signs (11, 79).
const HOST: &str = "host";

/// The projection every output here is packed by.
const FILE: &str = "file";

/// A scratch repository and the fakes bound beside moon and a landing.
struct World {
    scratch: Scratch,
    fakes: Fakes,
}

impl World {
    fn new() -> World {
        let mut fakes = Fakes::new();
        fakes.signer.hold(&SignerId::new(HOST));
        fakes.projection(&Ecosystem::new(FILE));
        World {
            scratch: Scratch::new(),
            fakes,
        }
    }

    /// `tasks` and the policy file declaring `outputs`, each with its task
    /// and packed as files, committed and pushed.
    fn declare(&self, tasks: &[Task], outputs: &[(&str, &str)]) {
        let repo = &self.scratch.repo;
        moon_workspace(repo, tasks);
        let projections: String = outputs
            .iter()
            .map(|(output, _)| format!("{output} = \"{FILE}\"\n"))
            .collect();
        policy_file(repo, outputs, &format!("\n[projections]\n{projections}"));
        repo.commit("feat: the outputs and their tasks");
        repo.push();
    }

    fn moon(&self) -> Moon {
        Moon::new(&self.scratch.repo.path)
    }

    /// The fakes as the one binding, moon its build and `landing` its
    /// landing.
    fn seams(&self, landing: Box<dyn Landing>) -> Seams {
        let mut seams = self.fakes.seams();
        seams.build = Box::new(self.moon());
        seams.landing = landing;
        seams
    }

    fn history(&self) -> History {
        History::open(&self.scratch.repo.path).expect("the repository opens")
    }

    /// The one component published from `revision` on `main`.
    fn publish(&self, landing: Box<dyn Landing>, revision: &Revision) -> Result<Published, Not> {
        let history = self.history();
        let ask = Ask {
            branch: Branch::new(MAIN),
            revision: revision.clone(),
            component: None,
            signer: SignerId::new(HOST),
            store: Source::new(Seam::Store, tailrace_fakes::PROFILE),
        };
        let mut published = publish(&self.seams(landing), &history, &Kept::at(&history), &ask)
            .expect("the publish answered")
            .components;
        assert_eq!(published.len(), 1, "{published:?}");
        published.remove(0).1
    }

    /// `version` verified on `main` under the acceptance policy the
    /// fake policy holds.
    fn verify(&self, landing: Box<dyn Landing>, version: &Version) -> Verified {
        let ask = VerifyAsk {
            branch: Branch::new(MAIN),
            component: None,
            version: version.clone(),
            audience: Audience::Acceptance,
        };
        verify(&self.seams(landing), &self.history(), &ask).expect("verify answered")
    }

    fn sluice(&self) -> Box<dyn Landing> {
        Box::new(Sluice::new(&self.scratch.repo.path, Some("origin".into())))
    }

    fn branch(&self) -> Box<dyn Landing> {
        Box::new(PlainBranch::new(
            &self.scratch.repo.path,
            Some("origin".into()),
        ))
    }

    /// sluice gating `main`, requiring `required`, lands a submission
    /// writing `file`; answers the landing commit.
    fn landed_by_sluice(&self, required: &[&str], file: &str) -> Revision {
        let recorder = Recorder::new(&self.scratch.repo, true, &self.scratch.path().join("keys"));
        let gate = Gate::new(&self.scratch, &recorder, required);
        Revision::new(gate.land("x", &[(file, "x")]))
    }

    /// Every write the store was asked.
    fn puts(&self) -> Vec<StoreOp> {
        self.fakes
            .store
            .calls()
            .into_iter()
            .map(|c| c.op)
            .filter(|op| {
                matches!(
                    op,
                    StoreOp::PutBytes | StoreOp::PutArtifact | StoreOp::Attach | StoreOp::Name
                )
            })
            .collect()
    }
}

fn named(id: &str, kinds: &[ClaimKind], checks: &[&str]) -> NamedSigner {
    NamedSigner {
        id: SignerId::new(id),
        verified_by: format!("{id}'s key"),
        kinds: kinds.to_vec(),
        checks: checks.iter().map(|c| Check::new(*c)).collect(),
    }
}

/// The provenance each artifact of `published` carries.
fn provenances(world: &World, published: &Published) -> Vec<Provenance> {
    let Published::Version { artifacts, .. } = published else {
        panic!("nothing was offered: {published:?}")
    };
    artifacts
        .iter()
        .map(|a| {
            let held = world.fakes.store.claims_of(&a.digest).unwrap();
            Provenance::of(&Claim::from_envelope(&held[0]).expect("a claim of tailrace's form"))
                .expect("a provenance")
        })
        .collect()
}

/// sluice lands a revision whose check is the task that makes an output;
/// the build identity moon reports for that output at the landed
/// revision is the key sluice's verdict carries, and the publish carries
/// that verdict, whole, in the output's provenance (13, 15, C.3, C.5).
#[test]
fn the_landings_key_is_the_builds_identity() {
    let world = World::new();
    world.declare(
        &[Task::new("app", "test", "app.txt", true, 0)],
        &[("app", "app:test")],
    );
    let landed = world.landed_by_sluice(&["app:test"], "app/x.txt");

    let read = Sluice::new(&world.scratch.repo.path, Some("origin".into()))
        .verdicts_of(&landed)
        .unwrap();
    assert_eq!(read.read, Read::Source, "sluice's remote answered");
    let [verdict] = <[_; 1]>::try_from(read.verdicts).unwrap();
    assert_eq!(verdict.check, Check::new("app:test"));
    let moon = world.moon();
    let app = OutputName::new("app");
    assert_eq!(moon.ensure(&landed, std::slice::from_ref(&app)), Ok(()));
    let report = moon.report(&landed, &app).unwrap();
    assert_eq!(
        report.build, verdict.build,
        "moon's hash is not sluice's key"
    );

    let published = world
        .publish(world.sluice(), &landed)
        .expect("the landed revision was offered");
    let [provenance] = <[_; 1]>::try_from(provenances(&world, &published)).unwrap();
    let [carried] = <[_; 1]>::try_from(provenance.carried).unwrap();
    assert_eq!(carried.build, report.build);
    assert_eq!(carried.signer, SignerId::new(SIGNER));
    assert_eq!(carried.found.at, verdict.found);
    assert_eq!(
        as_written(&carried.envelope),
        as_written(&verdict.envelope),
        "the verdict was not carried as sluice wrote it"
    );
}

/// An output whose task the gate never checked came from a build no
/// verdict names: the publish is refused naming it and every identity
/// the verdicts carry, and nothing is put (15, 111).
#[test]
fn an_output_no_verdict_names_is_refused() {
    let world = World::new();
    world.declare(
        &[
            Task::new("app", "test", "app.txt", true, 0),
            Task::new("web", "build", "site.txt", true, 0),
        ],
        &[("app", "app:test"), ("site", "web:build")],
    );
    let landed = world.landed_by_sluice(&["app:test"], "app/x.txt");
    let moon = world.moon();
    assert_eq!(
        moon.ensure(&landed, &[OutputName::new("app"), OutputName::new("site")]),
        Ok(())
    );
    let app = moon.report(&landed, &OutputName::new("app")).unwrap().build;
    let site = moon
        .report(&landed, &OutputName::new("site"))
        .unwrap()
        .build;

    match world.publish(world.sluice(), &landed) {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::Unnamed {
                output: OutputName::new("site"),
                build: site,
                carried: vec![app],
            }
        ),
        other => panic!("an output no verdict names answered {other:?}"),
    }
    assert_eq!(world.puts(), [], "something was offered");
}

/// S13: a revision on a branch no landing gate runs, built by moon,
/// published with no verdict carried; under a policy requiring a verdict
/// none counts, and under one requiring provenance alone the provenance
/// counts (9, 15, 111).
#[test]
fn s13_over_a_plain_branch_and_moon() {
    let world = World::new();
    world.declare(
        &[Task::new("web", "build", "site.txt", true, 0)],
        &[("site", "web:build")],
    );
    let head = Revision::new(world.scratch.repo.head());

    let published = world
        .publish(world.branch(), &head)
        .expect("a revision with no verdicts was offered");
    let Published::Version {
        version, artifacts, ..
    } = &published
    else {
        unreachable!()
    };
    let built = world
        .moon()
        .report(&head, &OutputName::new("site"))
        .unwrap()
        .build;
    assert_eq!(artifacts[0].build, built, "the site was not moon's build");
    let [provenance] = <[_; 1]>::try_from(provenances(&world, &published)).unwrap();
    assert_eq!(provenance.carried, [], "a verdict was carried");

    let policy = &world.fakes.policy;
    policy.require(
        Audience::Acceptance,
        Required::Acceptance {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Verdict,
                check: Some(Check::new("web:build")),
                signers: vec![named(SIGNER, &[ClaimKind::Verdict], &["web:build"])],
            }],
        },
    );
    let requiring_a_verdict = world.verify(world.branch(), version);
    assert!(
        !requiring_a_verdict
            .claims
            .iter()
            .any(|t| t.kind == Some(ClaimKind::Verdict) || t.counts),
        "a verdict counted where none was carried: {:?}",
        requiring_a_verdict.claims
    );

    policy.require(
        Audience::Acceptance,
        Required::Acceptance {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Provenance,
                check: None,
                signers: vec![named(HOST, &[ClaimKind::Provenance], &[])],
            }],
        },
    );
    let requiring_provenance = world.verify(world.branch(), version);
    let counted: Vec<_> = requiring_provenance
        .claims
        .iter()
        .filter(|t| t.kind == Some(ClaimKind::Provenance))
        .collect();
    // The site's provenance and the notes'.
    assert_eq!(counted.len(), 2, "{counted:?}");
    for told in counted {
        assert!(told.counts, "the host's provenance did not count");
        assert_eq!(told.made_by, Some(SignerId::new(HOST)), "{told:?}");
    }
}
