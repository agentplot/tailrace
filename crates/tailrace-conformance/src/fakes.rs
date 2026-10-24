//! The rigs over the fakes, one per seam's group: each a [`Fakes`] with
//! the fake's own scripting behind the rig's setup methods (design D3).
//! Each answers as the profile `fake`, as the fakes' sources name it, and
//! makes its source unreachable by faulting every operation of the seam
//! unreadable, with [`WORDS`], until healed.

use std::cell::Cell;

use tailrace_fakes::{BuildOp, Fakes, Fault, LandingOp, SignerOp, StoreOp, Times, PROFILE};
use tailrace_seams::{
    Branch, Build, BuildId, ClaimKind, Landing, NamedSigner, Outcome, OutputName, Report, Revision,
    Signer, SignerId, Store, Verdict,
};

use crate::seams::build::{BuildRig, Made};
use crate::seams::landing::LandingRig;
use crate::seams::signer::SignerRig;
use crate::seams::store::StoreRig;
use crate::Rig;

/// What a fake made unreachable answers, as its source's own words.
pub const WORDS: &str = "connection refused";

/// The branch the fake landing lands on.
pub const MAIN: &str = "main";

/// Faults every operation of `ops` unreadable with [`WORDS`] until
/// healed.
fn unreachable<Op: Copy + PartialEq>(fault: impl Fn(Op, Fault, Times), ops: &[Op]) -> String {
    for op in ops {
        fault(*op, Fault::Unreadable(WORDS.into()), Times::UntilHealed);
    }
    WORDS.into()
}

/// A new revision id for a rig, the `n`th it hands out.
fn revision(n: &Cell<u32>) -> Revision {
    n.set(n.get() + 1);
    Revision::new(format!("{:040x}", n.get()))
}

/// The fake store.
pub struct FakeStoreRig {
    pub fakes: Fakes,
}

impl FakeStoreRig {
    pub fn new() -> Self {
        FakeStoreRig {
            fakes: Fakes::new(),
        }
    }
}

impl Default for FakeStoreRig {
    fn default() -> Self {
        Self::new()
    }
}

impl Rig for FakeStoreRig {
    fn profile(&self) -> String {
        PROFILE.into()
    }
}

impl StoreRig for FakeStoreRig {
    fn store(&self) -> &dyn Store {
        &self.fakes.store
    }

    fn unreachable(&self) -> String {
        use StoreOp::*;
        let store = &self.fakes.store;
        unreachable(
            |op, f, t| store.fault(op, f, t),
            &[PutBytes, PutArtifact, Attach, ClaimsOf, Name, Resolve, Read],
        )
    }
}

/// The fake build, which a second host asks through another handle on
/// the same state.
pub struct FakeBuildRig {
    pub fakes: Fakes,
    second: tailrace_fakes::FakeBuild,
    revisions: Cell<u32>,
}

impl FakeBuildRig {
    pub fn new() -> Self {
        let fakes = Fakes::new();
        let second = fakes.build.clone();
        FakeBuildRig {
            fakes,
            second,
            revisions: Cell::new(0),
        }
    }
}

impl Default for FakeBuildRig {
    fn default() -> Self {
        Self::new()
    }
}

impl Rig for FakeBuildRig {
    fn profile(&self) -> String {
        PROFILE.into()
    }
}

impl BuildRig for FakeBuildRig {
    fn build(&self) -> &dyn Build {
        &self.fakes.build
    }

    fn second_host(&self) -> &dyn Build {
        &self.second
    }

    fn revision(&self, outputs: &[(OutputName, Made)]) -> Revision {
        let at = revision(&self.revisions);
        for (output, made) in outputs {
            let outcome = match made {
                Made::Pass => Outcome::Pass,
                Made::Fail => Outcome::Fail,
                Made::Unmade => continue,
            };
            self.fakes.build.made(
                &at,
                output,
                Report {
                    build: BuildId::new(format!("{output}@{}", &at.as_str()[32..])),
                    outcome,
                    files: vec![format!("dist/{output}")],
                },
            );
        }
        at
    }

    fn unreachable(&self) -> String {
        let build = &self.fakes.build;
        unreachable(
            |op, f, t| build.fault(op, f, t),
            &[BuildOp::Ensure, BuildOp::Report],
        )
    }
}

/// The fake landing, landing on `main`.
pub struct FakeLandingRig {
    pub fakes: Fakes,
    revisions: Cell<u32>,
}

impl FakeLandingRig {
    pub fn new() -> Self {
        FakeLandingRig {
            fakes: Fakes::new(),
            revisions: Cell::new(0),
        }
    }
}

impl Default for FakeLandingRig {
    fn default() -> Self {
        Self::new()
    }
}

impl Rig for FakeLandingRig {
    fn profile(&self) -> String {
        PROFILE.into()
    }
}

impl LandingRig for FakeLandingRig {
    fn landing(&self) -> &dyn Landing {
        &self.fakes.landing
    }

    fn branch(&self) -> Branch {
        Branch::new(MAIN)
    }

    fn landed(&self, verdicts: &[Verdict]) -> Revision {
        let at = revision(&self.revisions);
        self.fakes.landing.land(&self.branch(), &at);
        for verdict in verdicts {
            self.fakes.landing.verdict(&at, verdict.clone());
        }
        at
    }

    fn not_landed(&self) -> Revision {
        revision(&self.revisions)
    }

    fn unreachable(&self) -> String {
        let landing = &self.fakes.landing;
        unreachable(
            |op, f, t| landing.fault(op, f, t),
            &[LandingOp::Landed, LandingOp::VerdictsOf],
        )
    }
}

/// The fake signer, holding the host's identity.
pub struct FakeSignerRig {
    pub fakes: Fakes,
}

impl FakeSignerRig {
    /// The identity the rig's signer holds.
    pub const HOST: &'static str = "host";

    pub fn new() -> Self {
        let fakes = Fakes::new();
        fakes.signer.hold(&SignerId::new(Self::HOST));
        FakeSignerRig { fakes }
    }
}

impl Default for FakeSignerRig {
    fn default() -> Self {
        Self::new()
    }
}

impl Rig for FakeSignerRig {
    fn profile(&self) -> String {
        PROFILE.into()
    }
}

/// `id` as a policy names it, for every kind of claim.
pub fn named(id: &SignerId) -> NamedSigner {
    NamedSigner {
        id: id.clone(),
        verified_by: format!("{id}'s key"),
        kinds: vec![
            ClaimKind::Provenance,
            ClaimKind::Verdict,
            ClaimKind::Test,
            ClaimKind::Review,
            ClaimKind::Rebuild,
            ClaimKind::Acceptance,
            ClaimKind::Promotion,
            ClaimKind::Carried,
        ],
        checks: vec![],
    }
}

impl SignerRig for FakeSignerRig {
    fn signer(&self) -> &dyn Signer {
        &self.fakes.signer
    }

    fn identity(&self) -> SignerId {
        SignerId::new(Self::HOST)
    }

    fn named(&self, identity: &SignerId) -> NamedSigner {
        named(identity)
    }

    fn stranger(&self) -> NamedSigner {
        named(&SignerId::new("a stranger"))
    }

    fn no_identity(&self) {
        // A signer holding another identity has none of the host's at
        // hand.
        self.fakes.signer.hold(&SignerId::new("nobody here"));
    }

    fn expire(&self, identity: &SignerId) {
        self.fakes.signer.expire(identity);
    }

    fn unreachable(&self) -> String {
        let signer = &self.fakes.signer;
        unreachable(
            |op, f, t| signer.fault(op, f, t),
            &[SignerOp::Sign, SignerOp::Verify],
        )
    }
}
