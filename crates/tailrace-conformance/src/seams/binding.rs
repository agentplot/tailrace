//! The binding's group: 92, and 93 held of the suite itself (B.9).

use std::panic::{catch_unwind, AssertUnwindSafe};

use serde_json::json;
use tailrace_seams::{
    Claim, ClaimKind, Digest, Ecosystem, EnvironmentName, Output, OutputName, Revision, Seams,
    SignerId, Version,
};

use crate::seams::store::{self, StoreRig};
use crate::{ok, run, Rig, Scenario};

pub const GROUP: &str = "binding";

/// What binds the seams, made new for each scenario.
pub trait BindingRig: Rig {
    /// One store, build, landing, policy and keeper, a signer for each of
    /// `signers`, a projection for each of `ecosystems` and a target for
    /// each of `environments`, bound as one binding.
    fn bind(
        &self,
        ecosystems: &[Ecosystem],
        environments: &[EnvironmentName],
        signers: &[SignerId],
    ) -> Seams;

    /// A rig whose store holds other bytes under a digest, which breaks
    /// the store's contract.
    fn broken_store(&self) -> Box<dyn StoreRig>;
}

pub fn scenarios() -> Vec<Scenario<dyn BindingRig>> {
    vec![
        Scenario {
            name: "one of each, any number of projections and targets",
            plays: None,
            cites: &[92],
            run: one_of_each_any_number_of_projections_and_targets,
        },
        Scenario {
            name: "a broken profile fails the suite",
            plays: None,
            cites: &[93],
            run: a_broken_profile_fails_the_suite,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}

fn one_of_each_any_number_of_projections_and_targets(rig: &dyn BindingRig) {
    let ecosystems = [Ecosystem::new("file"), Ecosystem::new("image")];
    let environments = [
        EnvironmentName::new("staging"),
        EnvironmentName::new("production"),
    ];
    let signers = [SignerId::new("host"), SignerId::new("gate")];
    let seams = rig.bind(&ecosystems, &environments, &signers);

    let output = Output {
        name: OutputName::new("site"),
        files: vec!["dist/site".into()],
    };
    for ecosystem in &ecosystems {
        let projection = seams
            .projection(ecosystem)
            .unwrap_or_else(|| panic!("no projection is bound for {ecosystem}"));
        let packed = ok(
            projection.pack(&output, &Version::new("1.0.0")),
            &format!("the pack of the projection bound for {ecosystem}"),
        );
        assert_eq!(
            packed.meta.ecosystem(),
            ecosystem,
            "the projection bound for {ecosystem} packs in another"
        );
    }
    assert!(
        seams.projection(&Ecosystem::new("unbound")).is_none(),
        "a projection was found for an ecosystem none is bound for"
    );
    assert_eq!(seams.projections.len(), ecosystems.len());

    for environment in &environments {
        assert!(
            seams.target(environment).is_some(),
            "no target is bound for {environment}"
        );
    }
    assert!(
        seams.target(&EnvironmentName::new("unbound")).is_none(),
        "a target was found for an environment none is bound for"
    );
    assert_eq!(seams.targets.len(), environments.len());

    assert_eq!(
        seams.signers.len(),
        signers.len(),
        "not every signer is bound"
    );
    for (signer, identity) in seams.signers.iter().zip(&signers) {
        let claim = Claim {
            subject: Digest::of(b"an index"),
            kind: ClaimKind::Provenance,
            revision: Revision::new("4f2a9c1"),
            build: None,
            body: json!({}),
        };
        let envelope = ok(
            signer.sign(&claim, identity),
            &format!("the signer bound for {identity}"),
        );
        assert_eq!(&envelope.signature.signer, identity);
    }
}

fn a_broken_profile_fails_the_suite(rig: &dyn BindingRig) {
    let profile = rig.broken_store().profile();
    let report = run(store::GROUP, &store::scenarios(), &|| rig.broken_store());
    assert!(
        !report.passed(),
        "a store holding other bytes under a digest passed the store's group"
    );
    assert_eq!(report.profile, profile, "the failure is not the profile's");
    assert_eq!(report.group, store::GROUP);
    assert!(
        report
            .failures
            .iter()
            .any(|f| f.scenario == "bytes read back"),
        "reading other bytes back did not fail: {report}"
    );
    let text = report.to_string();
    assert!(
        text.starts_with(&format!("the store group over the profile `{profile}`")),
        "{text}"
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| report.assert())).is_err(),
        "a report with failures did not fail its run"
    );
}
