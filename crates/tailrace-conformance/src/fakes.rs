//! The rigs over the fakes, one per seam's group: each a [`Fakes`] with
//! the fake's own scripting behind the rig's setup methods (design D3).
//! Each answers as the profile `fake`, as the fakes' sources name it, and
//! makes its source unreachable by faulting every operation of the seam
//! unreadable, with [`WORDS`], until healed.

use std::cell::Cell;

use serde_json::json;
use tailrace_fakes::{
    BuildOp, FakeKeeper, FakePolicy, FakeProjection, FakeSigner, FakeTarget, Fakes, Fault,
    KeeperOp, LandingOp, PolicyOp, ProjectionOp, SignerOp, StoreOp, Times, PROFILE,
};
use tailrace_seams::{
    Artifact, ArtifactMeta, Audience, Branch, Build, BuildId, ChannelName, Claim, ClaimKind,
    Digest, Ecosystem, EcosystemName, Envelope, Environment, EnvironmentName, Keeper, Landing,
    NamedSigner, Outcome, Output, OutputName, Pinned, Policy, Projection, Range, Report, Required,
    Requirement, Revision, Seams, Signature, Signer, SignerId, Store, Target, Verdict, Verified,
    Version, IN_TOTO,
};

use crate::seams::binding::BindingRig;
use crate::seams::build::{BuildRig, Made};
use crate::seams::keeper::KeeperRig;
use crate::seams::landing::{LandingRig, Recorded};
use crate::seams::policy::PolicyRig;
use crate::seams::projection::ProjectionRig;
use crate::seams::signer::SignerRig;
use crate::seams::store::{StoreRig, BYTES};
use crate::seams::target::TargetRig;
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

/// The fake landing, landing on `main`, recording each verdict as a
/// landing does: it makes the signed claim, and the suite hands it only
/// what it records (design D3a).
pub struct FakeLandingRig {
    pub fakes: Fakes,
    records: bool,
    revisions: Cell<u32>,
}

impl FakeLandingRig {
    /// The fake landing, recording verdicts.
    pub fn new() -> Self {
        FakeLandingRig {
            fakes: Fakes::new(),
            records: true,
            revisions: Cell::new(0),
        }
    }

    /// The same fake, saying it records no verdicts, as a plain branch
    /// records none (C.6).
    pub fn recording_none() -> Self {
        FakeLandingRig {
            records: false,
            ..Self::new()
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

/// `recorded` as the fake landing records it for `revision`: a verdict
/// claim about the build, its payload, and a signature by the verdict's
/// signer over the payload's digest, which tailrace only carries (12,
/// 78).
fn signed(revision: &Revision, recorded: &Recorded) -> Verdict {
    let claim = Claim {
        subject: Digest::of(recorded.build.as_str().as_bytes()),
        kind: ClaimKind::Verdict,
        revision: revision.clone(),
        build: Some(recorded.build.clone()),
        body: json!({ "check": recorded.check, "outcome": recorded.outcome }),
    };
    let payload = claim.payload();
    let sig = Digest::of(&payload).as_str().as_bytes().to_vec();
    Verdict {
        check: recorded.check.clone(),
        build: recorded.build.clone(),
        outcome: recorded.outcome,
        signer: recorded.signer.clone(),
        envelope: Envelope::signed(
            IN_TOTO,
            payload,
            Signature {
                signer: recorded.signer.clone(),
                keyid: String::new(),
                sig,
            },
        ),
        found: format!(
            "the fake landing's record of {} by {}",
            recorded.check, recorded.signer
        ),
    }
}

impl LandingRig for FakeLandingRig {
    fn landing(&self) -> &dyn Landing {
        &self.fakes.landing
    }

    fn branch(&self) -> Branch {
        Branch::new(MAIN)
    }

    fn records_verdicts(&self) -> bool {
        self.records
    }

    fn landed(&self, verdicts: &[Recorded]) -> Revision {
        assert!(
            self.records || verdicts.is_empty(),
            "a landing that records no verdicts was handed {} to record",
            verdicts.len()
        );
        let at = revision(&self.revisions);
        self.fakes.landing.land(&self.branch(), &at);
        for recorded in verdicts {
            self.fakes.landing.verdict(&at, signed(&at, recorded));
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

/// The fake policy.
pub struct FakePolicyRig {
    pub policy: FakePolicy,
    revisions: Cell<u32>,
}

impl FakePolicyRig {
    pub fn new() -> Self {
        FakePolicyRig {
            policy: Fakes::new().policy,
            revisions: Cell::new(0),
        }
    }
}

impl Default for FakePolicyRig {
    fn default() -> Self {
        Self::new()
    }
}

impl Rig for FakePolicyRig {
    fn profile(&self) -> String {
        PROFILE.into()
    }
}

impl PolicyRig for FakePolicyRig {
    fn policy(&self) -> &dyn Policy {
        &self.policy
    }

    fn at(&self, policies: &[(Audience, Required)]) -> Revision {
        let at = revision(&self.revisions);
        for (audience, required) in policies {
            self.policy
                .require_at(&at, audience.clone(), required.clone());
        }
        at
    }

    fn unreachable(&self) -> String {
        let policy = &self.policy;
        unreachable(|op, f, t| policy.fault(op, f, t), &[PolicyOp::PolicyFor])
    }
}

/// The fake projection of the ecosystem `file`, whose clients the group
/// treats as reading no files, so it serves; it packs cache entries
/// unless made with [`FakeProjectionRig::packing_no_cache_entries`].
pub struct FakeProjectionRig {
    pub projection: FakeProjection,
}

impl FakeProjectionRig {
    /// The ecosystem the rig's projection packs in.
    pub const ECOSYSTEM: &'static str = "file";

    pub fn new() -> Self {
        FakeProjectionRig {
            projection: FakeProjection::new(Ecosystem::new(Self::ECOSYSTEM)),
        }
    }

    /// The fake projection scripted to pack no cache entries (84, 85).
    pub fn packing_no_cache_entries() -> Self {
        let rig = Self::new();
        rig.projection.packs_no_cache_entries();
        rig
    }
}

impl Default for FakeProjectionRig {
    fn default() -> Self {
        Self::new()
    }
}

impl Rig for FakeProjectionRig {
    fn profile(&self) -> String {
        PROFILE.into()
    }
}

impl ProjectionRig for FakeProjectionRig {
    fn projection(&self) -> &dyn Projection {
        &self.projection
    }

    fn output(&self) -> Output {
        Output {
            name: OutputName::new("site"),
            files: vec!["dist/site/index.html".into()],
        }
    }

    fn artifacts(&self, n: usize) -> Vec<Artifact> {
        (0..n)
            .map(|i| {
                let meta = ArtifactMeta::new(
                    Ecosystem::new(Self::ECOSYSTEM),
                    EcosystemName::new("site"),
                    Version::new(format!("1.{i}.0")),
                )
                .expect("the metadata names all three");
                Artifact {
                    digest: Digest::of(format!("site 1.{i}.0").as_bytes()),
                    meta,
                }
            })
            .collect()
    }

    fn serves(&self) -> bool {
        true
    }

    fn caches(&self) -> bool {
        self.projection.caches()
    }

    fn has_lockfile(&self) -> bool {
        true
    }

    /// The checkout `checkouts/with-lockfile`, its lockfile set to pin a
    /// scoped name and a prerelease as a lockfile spells them, or
    /// `checkouts/without`, which holds none.
    fn checkout(&self, lockfile: bool) -> (String, Vec<Pinned>) {
        if !lockfile {
            return ("checkouts/without".into(), Vec::new());
        }
        let at = "checkouts/with-lockfile";
        let pins = vec![
            Pinned {
                name: EcosystemName::new("@scope/site"),
                version: Version::new("1.2.0-rc.1"),
            },
            Pinned {
                name: EcosystemName::new("left-pad"),
                version: Version::new("1.3.0"),
            },
        ];
        self.projection.lockfile(at, &pins);
        (at.into(), pins)
    }

    fn has_manifest(&self) -> bool {
        true
    }

    /// The second of [`ProjectionRig::artifacts`], its manifest set to
    /// require a scoped name with a caret range and a plain name with a
    /// range carrying a prerelease, as a manifest spells them.
    fn held(&self) -> (Artifact, Vec<Requirement>) {
        let held = self.artifacts(2).remove(1);
        let requirements = vec![
            Requirement {
                name: EcosystemName::new("@scope/lib"),
                range: Range::new("^1.2.0"),
            },
            Requirement {
                name: EcosystemName::new("left-pad"),
                range: Range::new(">=1.2.0-rc.1 <2"),
            },
        ];
        self.projection.manifest(&held.digest, &requirements);
        (held, requirements)
    }

    /// `^1.2` over versions listed out of order, which Semantic
    /// Versioning orders `1.10.0` above `1.9.0` and spelling below it.
    fn ranged(&self) -> (Range, Vec<Version>, Vec<Version>) {
        let versions = |spelled: &[&str]| spelled.iter().map(|v| Version::new(*v)).collect();
        (
            Range::new("^1.2"),
            versions(&["1.9.0", "2.0.0", "1.10.0", "1.2.0", "1.1.9"]),
            versions(&["1.10.0", "1.9.0", "1.2.0"]),
        )
    }

    fn unreachable(&self) -> String {
        let projection = &self.projection;
        unreachable(
            |op, f, t| projection.fault(op, f, t),
            &[
                ProjectionOp::Pack,
                ProjectionOp::Render,
                ProjectionOp::Serve,
                ProjectionOp::Pinned,
                ProjectionOp::Requires,
                ProjectionOp::Satisfying,
            ],
        )
    }
}

/// The fake target.
pub struct FakeTargetRig {
    pub target: FakeTarget,
    environments: Cell<u32>,
}

impl FakeTargetRig {
    /// What the fake target says of an environment whose state it cannot
    /// read.
    pub const REASON: &'static str = "the marker of what was applied is gone";

    pub fn new() -> Self {
        FakeTargetRig {
            target: FakeTarget::new(),
            environments: Cell::new(0),
        }
    }

    fn next(&self) -> Environment {
        self.environments.set(self.environments.get() + 1);
        let n = self.environments.get();
        Environment {
            name: EnvironmentName::new(format!("environment-{n}")),
            channel: ChannelName::new(format!("channel-{n}")),
            waits: false,
        }
    }
}

impl Default for FakeTargetRig {
    fn default() -> Self {
        Self::new()
    }
}

impl Rig for FakeTargetRig {
    fn profile(&self) -> String {
        PROFILE.into()
    }
}

impl TargetRig for FakeTargetRig {
    fn target(&self) -> &dyn Target {
        &self.target
    }

    fn verified(&self, digest: &Digest) -> Verified {
        // The suite stands in the core's place here, which makes a
        // `Verified` after its verify (42); no profile makes one.
        Verified::new(digest.clone(), Version::new("1.0.0"))
    }

    fn environment(&self, head: Option<&Digest>) -> Environment {
        let environment = self.next();
        if let Some(head) = head {
            self.target.head(&environment.name, head);
        }
        environment
    }

    fn unreadable_environment(&self) -> (Environment, String) {
        let environment = self.next();
        self.target.unreadable(&environment.name, Self::REASON);
        (environment, Self::REASON.into())
    }
}

/// The fake keeper, its promotions signed by a fake signer as the gate.
pub struct FakeKeeperRig {
    pub keeper: FakeKeeper,
    gate: FakeSigner,
}

impl FakeKeeperRig {
    /// The identity the rig's promotions are signed in.
    pub const GATE: &'static str = "gate";

    pub fn new() -> Self {
        let gate = FakeSigner::new();
        gate.hold(&SignerId::new(Self::GATE));
        FakeKeeperRig {
            keeper: FakeKeeper::new(),
            gate,
        }
    }
}

impl Default for FakeKeeperRig {
    fn default() -> Self {
        Self::new()
    }
}

impl Rig for FakeKeeperRig {
    fn profile(&self) -> String {
        PROFILE.into()
    }
}

impl KeeperRig for FakeKeeperRig {
    fn keeper(&self) -> &dyn Keeper {
        &self.keeper
    }

    fn promotion(&self, channel: &ChannelName, n: u32) -> Envelope {
        let claim = Claim {
            subject: Digest::of(format!("the index {channel} moves to, {n}").as_bytes()),
            kind: ClaimKind::Promotion,
            revision: Revision::new(format!("{n:040x}")),
            build: None,
            body: json!({ "channel": channel, "n": n }),
        };
        self.gate
            .sign(&claim, &SignerId::new(Self::GATE))
            .expect("the gate's identity is at hand")
    }

    fn fork(&self, channel: &ChannelName) -> [Envelope; 2] {
        let (a, b) = (self.promotion(channel, 1), self.promotion(channel, 2));
        self.keeper.fork(channel, &a, &b);
        [a, b]
    }

    fn unreachable(&self) -> String {
        let keeper = &self.keeper;
        unreachable(
            |op, f, t| keeper.fault(op, f, t),
            &[KeeperOp::Append, KeeperOp::Chain],
        )
    }
}

/// The fake store, scripted to hold other bytes under the digest of the
/// bytes the store's group puts, so it breaks the store's contract (2,
/// 73); it answers as the profile it is named.
pub struct OtherBytesRig {
    pub fakes: Fakes,
    name: String,
}

impl OtherBytesRig {
    /// The bytes it holds under the digest of [`BYTES`].
    pub const OTHER: &'static [u8] = b"other bytes under that digest";

    pub fn named(name: impl Into<String>) -> Self {
        let fakes = Fakes::new();
        fakes.store.corrupt(&Digest::of(BYTES), Self::OTHER);
        OtherBytesRig {
            fakes,
            name: name.into(),
        }
    }
}

impl Rig for OtherBytesRig {
    fn profile(&self) -> String {
        self.name.clone()
    }
}

impl StoreRig for OtherBytesRig {
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

/// The fakes bound as one binding, a fake signer holding each identity
/// given.
pub struct FakeBindingRig;

impl FakeBindingRig {
    /// The profile its broken store answers as.
    pub const BROKEN: &'static str = "fake, holding other bytes under a digest";

    pub fn new() -> Self {
        FakeBindingRig
    }
}

impl Default for FakeBindingRig {
    fn default() -> Self {
        Self::new()
    }
}

impl Rig for FakeBindingRig {
    fn profile(&self) -> String {
        PROFILE.into()
    }
}

impl BindingRig for FakeBindingRig {
    fn bind(
        &self,
        ecosystems: &[Ecosystem],
        environments: &[EnvironmentName],
        signers: &[SignerId],
    ) -> Seams {
        let mut fakes = Fakes::new();
        for ecosystem in ecosystems {
            fakes.projection(ecosystem);
        }
        for environment in environments {
            fakes.target(environment);
        }
        let mut seams = fakes.seams();
        seams.signers = signers
            .iter()
            .map(|id| {
                let signer = FakeSigner::new();
                signer.hold(id);
                Box::new(signer) as Box<dyn Signer>
            })
            .collect();
        seams
    }

    fn broken_store(&self) -> Box<dyn StoreRig> {
        Box::new(OtherBytesRig::named(Self::BROKEN))
    }
}
