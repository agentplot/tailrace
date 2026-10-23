//! The eight seams of tailrace, Part B of the statement in
//! agentplot/blueprints `design/tailrace/requirements.md`: the store (B.1),
//! the build (B.2), the landing (B.3), the signer (B.4), the policy (B.5),
//! a projection (B.6), a target (B.7) and the keeper (B.8). Each is one
//! trait here, which a profile implements and nothing of the core (rule
//! core.2). Nothing is bound yet: bolt 1's units add each trait with the
//! profile that first implements it.

/// A name the statement gives a thing, held as its text.
macro_rules! text {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(
            Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(text: impl Into<String>) -> Self {
                $name(text.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

pub mod answer;
pub mod artifact;
pub mod build;
pub mod claim;
pub mod digest;
pub mod keeper;
pub mod landing;
pub mod policy;
pub mod projection;
pub mod signer;
pub mod store;
pub mod target;

pub use answer::*;
pub use artifact::*;
pub use build::*;
pub use claim::*;
pub use digest::*;
pub use keeper::*;
pub use landing::*;
pub use policy::*;
pub use projection::*;
pub use signer::*;
pub use store::*;
pub use target::*;

/// A value written in its machine-readable form and read back is the same
/// value (57).
#[cfg(test)]
pub(crate) fn round_trip<T>(value: T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(serde_json::from_str::<T>(&json).unwrap(), value, "{json}");
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    /// A profile of every seam that holds nothing, so each trait is seen
    /// boxed as the binding holds it.
    struct Nowhere;

    fn nothing<T>(seam: Seam, what: &str) -> Answer<T> {
        Err(Not::absent(&Source::new(seam, "nowhere"), what))
    }

    impl Store for Nowhere {
        fn put_bytes(&self, _: &[u8]) -> Answer<Put> {
            nothing(Seam::Store, "put-bytes")
        }
        fn put_artifact(&self, _: &Manifest) -> Answer<Put> {
            nothing(Seam::Store, "put-artifact")
        }
        fn attach(&self, _: &Digest, _: &Envelope) -> Answer<Put> {
            nothing(Seam::Store, "attach")
        }
        fn claims_of(&self, _: &Digest) -> Answer<Vec<Envelope>> {
            nothing(Seam::Store, "claims-of")
        }
        fn name(&self, _: &Name, _: &Digest, _: Option<&Digest>) -> Answer<()> {
            nothing(Seam::Store, "name")
        }
        fn resolve(&self, _: &Name) -> Answer<Digest> {
            nothing(Seam::Store, "resolve")
        }
        fn read(&self, _: &Digest) -> Answer<Held> {
            nothing(Seam::Store, "read")
        }
    }

    impl Build for Nowhere {
        fn ensure(&self, _: &Revision, _: &[OutputName]) -> Answer<()> {
            nothing(Seam::Build, "ensure")
        }
        fn report(&self, _: &Revision, _: &OutputName) -> Answer<Report> {
            nothing(Seam::Build, "report")
        }
    }

    impl Landing for Nowhere {
        fn landed(&self, _: &Branch, _: &Revision) -> Answer<bool> {
            nothing(Seam::Landing, "landed")
        }
        fn verdicts_of(&self, _: &Revision) -> Answer<Vec<Verdict>> {
            nothing(Seam::Landing, "verdicts-of")
        }
    }

    impl Signer for Nowhere {
        fn sign(&self, _: &Claim, _: &SignerId) -> Answer<Envelope> {
            nothing(Seam::Signer, "sign")
        }
        fn verify(&self, _: &Envelope, _: &[NamedSigner]) -> Answer<Made> {
            nothing(Seam::Signer, "verify")
        }
    }

    impl Policy for Nowhere {
        fn policy_for(&self, _: &Revision, _: &Audience) -> Answer<Required> {
            nothing(Seam::Policy, "policy-for")
        }
    }

    impl Projection for Nowhere {
        fn pack(&self, _: &Output, _: &Version) -> Answer<Packed> {
            nothing(Seam::Projection, "pack")
        }
        fn render(&self, _: &[Artifact], _: &str) -> Answer<Rendered> {
            nothing(Seam::Projection, "render")
        }
        fn serve(&self, _: &Layout) -> Answer<Serving> {
            nothing(Seam::Projection, "serve")
        }
    }

    impl Target for Nowhere {
        fn desired(&self, _: &Environment) -> Answer<Digest> {
            nothing(Seam::Target, "desired")
        }
        fn current(&self, _: &Environment) -> Answer<Digest> {
            nothing(Seam::Target, "current")
        }
        fn apply(&self, _: &Environment, _: &Verified) -> Answer<()> {
            nothing(Seam::Target, "apply")
        }
        fn report(&self, _: &Environment) -> Answer<Running> {
            nothing(Seam::Target, "report")
        }
    }

    impl Keeper for Nowhere {
        fn append(&self, _: &ChannelName, _: &Envelope, _: Option<&Digest>) -> Answer<Digest> {
            nothing(Seam::Keeper, "append")
        }
        fn chain(&self, _: &ChannelName) -> Answer<Vec<Envelope>> {
            nothing(Seam::Keeper, "chain")
        }
    }

    fn asked<T>(answer: Answer<T>) -> (Seam, String) {
        match answer {
            Err(Not::Absent(a)) => (a.source.seam, a.what),
            _ => panic!("nowhere answered something"),
        }
    }

    #[test]
    fn each_seam_is_one_trait_boxed_as_the_binding_holds_it() {
        let d = Digest::of(b"x");
        let rev = Revision::new("4f2a9c1");
        let envelope = Envelope {
            payload_type: IN_TOTO.into(),
            payload: vec![],
            signature: Signature {
                signer: SignerId::new("host"),
                sig: vec![],
            },
        };
        let meta = ArtifactMeta::new(
            Ecosystem::new("eco"),
            EcosystemName::new("pkg"),
            Version::new("1.0.0"),
        )
        .unwrap();
        let environment = Environment {
            name: EnvironmentName::new("production"),
            channel: ChannelName::new("stable"),
            waits: false,
        };
        let claim = Claim {
            subject: d.clone(),
            kind: ClaimKind::Provenance,
            revision: rev.clone(),
            build: None,
            body: Value::Null,
        };

        let store: Box<dyn Store> = Box::new(Nowhere);
        let build: Box<dyn Build> = Box::new(Nowhere);
        let landing: Box<dyn Landing> = Box::new(Nowhere);
        let signer: Box<dyn Signer> = Box::new(Nowhere);
        let policy: Box<dyn Policy> = Box::new(Nowhere);
        let projection: Box<dyn Projection> = Box::new(Nowhere);
        let target: Box<dyn Target> = Box::new(Nowhere);
        let keeper: Box<dyn Keeper> = Box::new(Nowhere);

        let operations = [
            asked(store.put_bytes(b"x")),
            asked(store.put_artifact(&Manifest::Artifact {
                meta: meta.clone(),
                layers: vec![d.clone()],
            })),
            asked(store.attach(&d, &envelope)),
            asked(store.claims_of(&d)),
            asked(store.name(&Name::new("n"), &d, None)),
            asked(store.resolve(&Name::new("n"))),
            asked(store.read(&d)),
            asked(build.ensure(&rev, &[OutputName::new("site")])),
            asked(build.report(&rev, &OutputName::new("site"))),
            asked(landing.landed(&Branch::new("main"), &rev)),
            asked(landing.verdicts_of(&rev)),
            asked(signer.sign(&claim, &SignerId::new("host"))),
            asked(signer.verify(&envelope, &[])),
            asked(policy.policy_for(&rev, &Audience::Acceptance)),
            asked(projection.pack(
                &Output {
                    name: OutputName::new("site"),
                    files: vec![],
                },
                &Version::new("1.0.0"),
            )),
            asked(projection.render(
                &[Artifact {
                    digest: d.clone(),
                    meta,
                }],
                "at",
            )),
            asked(projection.serve(&Layout { at: "at".into() })),
            asked(target.desired(&environment)),
            asked(target.current(&environment)),
            asked(target.apply(
                &environment,
                &Verified::new(d.clone(), Version::new("1.0.0")),
            )),
            asked(target.report(&environment)),
            asked(keeper.append(&ChannelName::new("stable"), &envelope, None)),
            asked(keeper.chain(&ChannelName::new("stable"))),
        ];
        let named = |seam| {
            operations
                .iter()
                .filter(|(s, _)| *s == seam)
                .map(|(_, op)| op.as_str())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            named(Seam::Store),
            [
                "put-bytes",
                "put-artifact",
                "attach",
                "claims-of",
                "name",
                "resolve",
                "read"
            ]
        );
        assert_eq!(named(Seam::Build), ["ensure", "report"]);
        assert_eq!(named(Seam::Landing), ["landed", "verdicts-of"]);
        assert_eq!(named(Seam::Signer), ["sign", "verify"]);
        assert_eq!(named(Seam::Policy), ["policy-for"]);
        assert_eq!(named(Seam::Projection), ["pack", "render", "serve"]);
        assert_eq!(
            named(Seam::Target),
            ["desired", "current", "apply", "report"]
        );
        assert_eq!(named(Seam::Keeper), ["append", "chain"]);
    }
}
