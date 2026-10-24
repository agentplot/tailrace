//! In-memory fakes of the eight seams, one per trait of `tailrace_seams`,
//! each implementing that trait and no other (rule core.2), so the core
//! and the conformance suite run with no registry, build, landing, key or
//! target on the host (93, rule core.33).
//!
//! | seam | fake | holds |
//! |---|---|---|
//! | the store | [`FakeStore`] | bytes and manifests by digest, attestations, names |
//! | the build | [`FakeBuild`] | what each revision left of each output |
//! | the landing | [`FakeLanding`] | revisions landed per branch, verdicts per revision |
//! | the signer | [`FakeSigner`] | the identity at hand, the identities expired |
//! | the policy | [`FakePolicy`] | what each audience requires, per revision |
//! | a projection | [`FakeProjection`] | its ecosystem, the name each output takes, whether it packs cache entries, the build identities it holds no archive for |
//! | a target | [`FakeTarget`] | per environment, its channel's head and what it runs |
//! | the keeper | [`FakeKeeper`] | per channel, a chain of promotion claims |
//!
//! Each fake is a [`Fake`] over its state and the one [`Script`] every
//! fake shares: every clone is a view of the same state, so a test keeps
//! one handle and binds another; a test sets what the fake holds with its
//! scripting methods, makes any operation refused or unreadable with
//! [`Fake::fault`], and reads every operation asked, in order, with
//! [`Fake::calls`]. [`Fakes`] keeps a handle on each and binds clones of
//! them as one [`Seams`] (92).
//!
//! The fakes are no profile of Part C: they bind no host, and count for
//! no seam under rule core.3 (design D10).

use std::collections::BTreeMap;

use tailrace_seams::{Ecosystem, EnvironmentName, Seams};

pub mod build;
pub mod keeper;
pub mod landing;
pub mod policy;
pub mod projection;
pub mod script;
pub mod signer;
pub mod store;
pub mod target;

pub use build::*;
pub use keeper::*;
pub use landing::*;
pub use policy::*;
pub use projection::*;
pub use script::*;
pub use signer::*;
pub use store::*;
pub use target::*;

/// A fake of every seam with a handle kept on each, ready to bind (92).
pub struct Fakes {
    pub store: FakeStore,
    pub build: FakeBuild,
    pub landing: FakeLanding,
    pub signer: FakeSigner,
    pub policy: FakePolicy,
    pub keeper: FakeKeeper,
    pub projections: BTreeMap<Ecosystem, FakeProjection>,
    pub targets: BTreeMap<EnvironmentName, FakeTarget>,
}

impl Default for Fakes {
    fn default() -> Self {
        Self::new()
    }
}

impl Fakes {
    /// One fake of each seam holding nothing, with no projection and no
    /// target bound.
    pub fn new() -> Self {
        Fakes {
            store: FakeStore::new(),
            build: FakeBuild::new(),
            landing: FakeLanding::new(),
            signer: FakeSigner::new(),
            policy: FakePolicy::new(),
            keeper: FakeKeeper::new(),
            projections: BTreeMap::new(),
            targets: BTreeMap::new(),
        }
    }

    /// The projection bound for `ecosystem`, bound now if none is.
    pub fn projection(&mut self, ecosystem: &Ecosystem) -> FakeProjection {
        self.projections
            .entry(ecosystem.clone())
            .or_insert_with(|| FakeProjection::new(ecosystem.clone()))
            .clone()
    }

    /// The target bound for `environment`, bound now if none is.
    pub fn target(&mut self, environment: &EnvironmentName) -> FakeTarget {
        self.targets.entry(environment.clone()).or_default().clone()
    }

    /// Clones of every fake as the one binding the core is handed: what
    /// the core asks through it, the handles kept here see.
    pub fn seams(&self) -> Seams {
        Seams {
            store: Box::new(self.store.clone()),
            build: Box::new(self.build.clone()),
            landing: Box::new(self.landing.clone()),
            policy: Box::new(self.policy.clone()),
            keeper: Box::new(self.keeper.clone()),
            signers: vec![Box::new(self.signer.clone())],
            projections: self
                .projections
                .iter()
                .map(|(e, p)| (e.clone(), Box::new(p.clone()) as _))
                .collect(),
            targets: self
                .targets
                .iter()
                .map(|(n, t)| (n.clone(), Box::new(t.clone()) as _))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use tailrace_seams::{
        Artifact, Audience, Branch, BuildId, ChannelName, Claim, ClaimKind, Contents, Environment,
        Layout, Manifest, Name, NamedSigner, Output, OutputName, Required, Revision, SignerId,
        Verified, Version,
    };

    use super::*;

    #[test]
    fn the_core_bound_to_fakes_alone_reaches_each_through_the_binding() {
        let mut fakes = Fakes::new();
        let (eco_a, eco_b) = (Ecosystem::new("eco-a"), Ecosystem::new("eco-b"));
        let production = Environment {
            name: EnvironmentName::new("production"),
            channel: ChannelName::new("stable"),
            waits: false,
        };
        let projection_a = fakes.projection(&eco_a);
        let projection_b = fakes.projection(&eco_b);
        let target = fakes.target(&production.name);

        let rev = Revision::new("4f2a9c1");
        let gate = SignerId::new("gate");
        fakes.landing.land(&Branch::new("main"), &rev);
        fakes.signer.hold(&gate);
        fakes.policy.require(
            Audience::Acceptance,
            Required::Acceptance { claims: vec![] },
        );

        let seams = fakes.seams();
        assert!(seams.projection(&Ecosystem::new("eco-c")).is_none());
        assert!(seams.target(&EnvironmentName::new("staging")).is_none());
        assert_eq!(seams.signers.len(), 1);

        let digest = seams.store.put_bytes(b"site").unwrap().digest;
        let packed = seams
            .projection(&eco_a)
            .expect("eco-a is bound")
            .pack(
                &Output {
                    name: OutputName::new("site"),
                    files: vec![],
                },
                &BuildId::new("b7e1"),
                Some(&Version::new("1.0.0")),
            )
            .unwrap();
        let meta = packed.meta.expect("a versioned pack names metadata");
        assert_eq!(meta.ecosystem(), &eco_a);
        let artifact = Artifact {
            digest: digest.clone(),
            meta: meta.clone(),
        };
        let index = seams
            .store
            .put_artifact(&Manifest::Artifact {
                meta,
                layers: vec![digest.clone()],
            })
            .unwrap()
            .digest;
        let claim = Claim {
            subject: index.clone(),
            kind: ClaimKind::Acceptance,
            revision: rev.clone(),
            build: None,
            body: Value::Null,
        };
        let signer = &seams.signers[0];
        let envelope = signer.sign(&claim, &gate).unwrap();
        let named = NamedSigner {
            id: gate.clone(),
            verified_by: "gate's key".into(),
            kinds: vec![ClaimKind::Acceptance],
            checks: vec![],
        };
        assert_eq!(signer.verify(&envelope, &[named]).unwrap().signer, gate);
        seams.store.attach(&index, &envelope).unwrap();
        assert_eq!(
            seams.store.claims_of(&index).unwrap(),
            vec![envelope.clone()]
        );
        let name = Name::new("site-v1.0.0");
        seams.store.name(&name, &index, None).unwrap();
        assert_eq!(seams.store.resolve(&name), Ok(index.clone()));
        assert!(seams.store.read(&digest).is_ok());

        assert!(seams
            .build
            .ensure(&rev, &[OutputName::new("site")])
            .is_err());
        assert!(seams.build.report(&rev, &OutputName::new("site")).is_err());
        assert_eq!(seams.landing.landed(&Branch::new("main"), &rev), Ok(true));
        assert_eq!(seams.landing.verdicts_of(&rev), Ok(vec![]));
        assert!(seams.policy.policy_for(&rev, &Audience::Acceptance).is_ok());

        let projection = seams.projection(&eco_b).expect("eco-b is bound");
        assert!(projection
            .pack(
                &Output {
                    name: OutputName::new("cli"),
                    files: vec![],
                },
                &BuildId::new("b7e1"),
                Some(&Version::new("1.0.0")),
            )
            .is_ok());
        let rendered = projection
            .render(Contents::Artifacts(&[artifact]), "layouts/eco-b")
            .unwrap();
        assert_eq!(
            projection.serve(&rendered.layout).unwrap().address,
            "fake://eco-b/layouts/eco-b"
        );
        let projection = seams.projection(&eco_a).unwrap();
        assert!(projection
            .render(Contents::Artifacts(&[]), "layouts/eco-a")
            .is_ok());
        assert!(projection
            .serve(&Layout {
                at: "layouts/eco-a".into()
            })
            .is_ok());

        let bound = seams.target(&production.name).expect("production is bound");
        assert!(bound.desired(&production).is_err());
        assert!(bound.current(&production).is_err());
        bound
            .apply(
                &production,
                &Verified::new(index.clone(), Version::new("1.0.0")),
            )
            .unwrap();
        assert_eq!(bound.report(&production).unwrap().digest, index);

        let stable = ChannelName::new("stable");
        assert_eq!(
            seams.keeper.append(&stable, &envelope, None),
            Ok(envelope.digest())
        );
        assert_eq!(seams.keeper.chain(&stable), Ok(vec![envelope]));

        let ops = |calls: Vec<Call<StoreOp>>| calls.into_iter().map(|c| c.op).collect::<Vec<_>>();
        assert_eq!(
            ops(fakes.store.calls()),
            [
                StoreOp::PutBytes,
                StoreOp::PutArtifact,
                StoreOp::Attach,
                StoreOp::ClaimsOf,
                StoreOp::Name,
                StoreOp::Resolve,
                StoreOp::Read
            ]
        );
        let ops = |calls: Vec<Call<BuildOp>>| calls.into_iter().map(|c| c.op).collect::<Vec<_>>();
        assert_eq!(ops(fakes.build.calls()), [BuildOp::Ensure, BuildOp::Report]);
        let ops = |calls: Vec<Call<LandingOp>>| calls.into_iter().map(|c| c.op).collect::<Vec<_>>();
        assert_eq!(
            ops(fakes.landing.calls()),
            [LandingOp::Landed, LandingOp::VerdictsOf]
        );
        let ops = |calls: Vec<Call<SignerOp>>| calls.into_iter().map(|c| c.op).collect::<Vec<_>>();
        assert_eq!(
            ops(fakes.signer.calls()),
            [SignerOp::Sign, SignerOp::Verify]
        );
        let ops = |calls: Vec<Call<PolicyOp>>| calls.into_iter().map(|c| c.op).collect::<Vec<_>>();
        assert_eq!(ops(fakes.policy.calls()), [PolicyOp::PolicyFor]);
        let ops =
            |calls: Vec<Call<ProjectionOp>>| calls.into_iter().map(|c| c.op).collect::<Vec<_>>();
        assert_eq!(
            ops(projection_a.calls()),
            [
                ProjectionOp::Pack,
                ProjectionOp::Render,
                ProjectionOp::Serve
            ]
        );
        assert_eq!(
            ops(projection_b.calls()),
            [
                ProjectionOp::Pack,
                ProjectionOp::Render,
                ProjectionOp::Serve
            ]
        );
        let ops = |calls: Vec<Call<TargetOp>>| calls.into_iter().map(|c| c.op).collect::<Vec<_>>();
        assert_eq!(
            ops(target.calls()),
            [
                TargetOp::Desired,
                TargetOp::Current,
                TargetOp::Apply,
                TargetOp::Report
            ]
        );
        let ops = |calls: Vec<Call<KeeperOp>>| calls.into_iter().map(|c| c.op).collect::<Vec<_>>();
        assert_eq!(
            ops(fakes.keeper.calls()),
            [KeeperOp::Append, KeeperOp::Chain]
        );
        assert!(fakes.store.contents().held.contains_key(&index));
        assert_eq!(fakes.store.contents().names.get(&name), Some(&index));
    }

    #[test]
    fn a_projection_or_target_bound_twice_is_the_one_bound() {
        let mut fakes = Fakes::new();
        let eco = Ecosystem::new("eco");
        let first = fakes.projection(&eco);
        fakes.projection(&eco).unsteady();
        assert_eq!(fakes.projections.len(), 1);
        assert!(
            !tailrace_seams::Projection::pack(
                &first,
                &Output {
                    name: OutputName::new("site"),
                    files: vec![],
                },
                &BuildId::new("b7e1"),
                Some(&Version::new("1.0.0")),
            )
            .unwrap()
            .same_again
        );
        let production = EnvironmentName::new("production");
        fakes.target(&production);
        fakes.target(&production);
        assert_eq!(fakes.targets.len(), 1);
    }
}
