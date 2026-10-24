//! Every group of the suite over the fakes: one test per group, each run
//! with the runner and reported as any profile is (93; design D9).

use tailrace_conformance::core::{publish, status, verify, version};
use tailrace_conformance::fakes::{
    FakeBindingRig, FakeBuildRig, FakeKeeperRig, FakeLandingRig, FakePolicyRig, FakeProjectionRig,
    FakeSignerRig, FakeStoreRig, FakeTargetRig,
};
use tailrace_conformance::seams::store::{self, StoreRig, BYTES};
use tailrace_conformance::seams::{
    binding, build, keeper, landing, policy, projection, signer, target,
};
use tailrace_conformance::world::FakeWorld;
use tailrace_conformance::{run, Report, Scenario};
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
fn a_fake_landing_recording_no_verdicts_keeps_the_landings_contract() {
    let report = run(landing::GROUP, &landing::scenarios(), &|| {
        Box::new(FakeLandingRig::recording_none())
    });
    report.assert();
    let not: Vec<(&str, &[u16], &str)> = report
        .not_applicable
        .iter()
        .map(|n| (n.scenario, n.cites, n.why.as_str()))
        .collect();
    assert_eq!(
        not,
        [
            (
                "verdicts as the landing recorded them",
                [78].as_slice(),
                "the landing records no verdicts"
            ),
            (
                "two signers, each by its own name",
                [104].as_slice(),
                "the landing records no verdicts"
            )
        ],
        "{report}"
    );
    let text = report.to_string();
    for words in [
        "the landing group over the profile `fake`: 3 passed, 2 not applicable",
        "\n- `verdicts as the landing recorded them` (clause 78): not applicable: \
         the landing records no verdicts",
        "\n- `two signers, each by its own name` (clause 104): not applicable: \
         the landing records no verdicts",
    ] {
        assert!(text.contains(words), "{text}\ndoes not say: {words}");
    }
}

#[test]
fn the_fake_signer_keeps_the_signers_contract() {
    run(signer::GROUP, &signer::scenarios(), &|| {
        Box::new(FakeSignerRig::new())
    })
    .assert();
}

#[test]
fn the_fake_policy_keeps_the_policys_contract() {
    run(policy::GROUP, &policy::scenarios(), &|| {
        Box::new(FakePolicyRig::new())
    })
    .assert();
}

/// The scenarios a projection group's report names not applicable.
fn inapplicable(report: &Report) -> Vec<(&str, &str)> {
    report
        .not_applicable
        .iter()
        .map(|n| (n.scenario, n.why.as_str()))
        .collect()
}

#[test]
fn the_fake_projection_keeps_a_projections_contract() {
    let report = run(projection::GROUP, &projection::scenarios(), &|| {
        Box::new(FakeProjectionRig::new())
    });
    report.assert();
    assert_eq!(
        inapplicable(&report),
        [(
            "a projection that packs no cache entries",
            "the projection packs cache entries"
        )],
        "{report}"
    );
}

#[test]
fn a_fake_projection_packing_no_cache_entries_keeps_a_projections_contract() {
    let report = run(projection::GROUP, &projection::scenarios(), &|| {
        Box::new(FakeProjectionRig::packing_no_cache_entries())
    });
    report.assert();
    assert_eq!(
        inapplicable(&report),
        [
            (
                "a cache entry packed with no version",
                "the projection packs no cache entries"
            ),
            (
                "a layout of the build identities given",
                "the projection packs no cache entries"
            )
        ],
        "{report}"
    );
}

#[test]
fn the_fake_target_keeps_a_targets_contract() {
    run(target::GROUP, &target::scenarios(), &|| {
        Box::new(FakeTargetRig::new())
    })
    .assert();
}

#[test]
fn the_fake_keeper_keeps_the_keepers_contract() {
    run(keeper::GROUP, &keeper::scenarios(), &|| {
        Box::new(FakeKeeperRig::new())
    })
    .assert();
}

#[test]
fn the_fakes_bound_keep_the_bindings_contract() {
    run(binding::GROUP, &binding::scenarios(), &|| {
        Box::new(FakeBindingRig::new())
    })
    .assert();
}

#[test]
fn the_core_publishes_over_the_fakes() {
    run(publish::GROUP, &publish::scenarios(), &|| {
        Box::new(FakeWorld::new())
    })
    .assert();
}

#[test]
fn the_core_verifies_over_the_fakes() {
    run(verify::GROUP, &verify::scenarios(), &|| {
        Box::new(FakeWorld::new())
    })
    .assert();
}

#[test]
fn the_core_versions_over_the_fakes() {
    run(version::GROUP, &version::scenarios(), &|| {
        Box::new(FakeWorld::two_components())
    })
    .assert();
}

#[test]
fn the_core_answers_status_over_the_fakes() {
    run(status::GROUP, &status::scenarios(), &|| {
        Box::new(FakeWorld::new())
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
