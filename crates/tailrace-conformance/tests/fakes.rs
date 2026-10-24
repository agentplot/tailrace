//! Every group of the suite over the fakes: one test per group, each run
//! with the runner and reported as any profile is (93; design D9).

use tailrace_conformance::fakes::{FakeBuildRig, FakeLandingRig, FakeSignerRig, FakeStoreRig};
use tailrace_conformance::seams::store::{self, StoreRig, BYTES};
use tailrace_conformance::seams::{build, landing, signer};
use tailrace_conformance::{run, Scenario};
use tailrace_seams::{Digest, Not};

#[test]
fn the_fake_store_keeps_the_stores_contract() {
    run(store::GROUP, &store::scenarios(), &|| {
        Box::new(FakeStoreRig::new())
    })
    .assert();
}

#[test]
fn the_fake_build_keeps_the_builds_contract() {
    run(build::GROUP, &build::scenarios(), &|| {
        Box::new(FakeBuildRig::new())
    })
    .assert();
}

#[test]
fn the_fake_landing_keeps_the_landings_contract() {
    run(landing::GROUP, &landing::scenarios(), &|| {
        Box::new(FakeLandingRig::new())
    })
    .assert();
}

#[test]
fn the_fake_signer_keeps_the_signers_contract() {
    run(signer::GROUP, &signer::scenarios(), &|| {
        Box::new(FakeSignerRig::new())
    })
    .assert();
}

// Requirement: A seam's group runs over any profile of that seam

fn puts(rig: &dyn StoreRig) {
    rig.store().put_bytes(BYTES).unwrap();
}

fn reads(rig: &dyn StoreRig) {
    let answer = rig.store().read(&Digest::of(BYTES));
    assert!(matches!(answer, Err(Not::Absent(_))), "{answer:?}");
}

#[test]
fn one_scenarios_store_is_not_the_nexts() {
    let scenarios: Vec<Scenario<dyn StoreRig>> = vec![
        Scenario {
            name: "puts",
            plays: None,
            cites: &[66],
            run: puts,
        },
        Scenario {
            name: "reads what the first put",
            plays: None,
            cites: &[72],
            run: reads,
        },
    ];
    run(store::GROUP, &scenarios, &|| Box::new(FakeStoreRig::new())).assert();
}
