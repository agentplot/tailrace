//! The world the core's groups run over: every seam a fake, bound as one
//! binding, and a real git history in a temporary directory, the same on
//! every run (design D4). The core's groups take it as it is, not a rig,
//! since many of their scenarios script what no real profile can be told
//! to do.

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::SystemTime;

use serde_json::json;
use tailrace_core::{publish, verify, Ask, History, Kept, Publish, Verified, VerifyAsk};
use tailrace_fakes::{
    BuildOp, Call, FakeProjection, FakeSigner, Fakes, KeeperOp, StoreOp, TargetOp,
};
use tailrace_seams::{
    Answer, Audience, Branch, BuildId, ChannelId, ChannelName, Check, Claim, ClaimKind, Contents,
    Digest, Ecosystem, Envelope, Environment, EnvironmentName, Layout, Manifest, Name, Outcome,
    Output, OutputName, Packed, Projection, Rendered, Report, Revision, Seam, Seams, Serving,
    Signer, SignerId, Source, Verdict, Version,
};

use crate::fixture::{Fixture, Repo, Snapshot, MAIN};
use crate::Rig;

/// The identity of the party running tailrace, which signs (11, 79).
pub const HOST: &str = "host";

/// The identity the landing signs its verdicts as (78).
pub const LANDING: &str = "landing";

/// The two ecosystems the world binds a projection for.
pub const FILE: &str = "file";
pub const IMAGE: &str = "image";

/// The one environment the world binds a target for, and the channel it
/// follows.
pub const PRODUCTION: &str = "production";
pub const STABLE: &str = "stable";

/// One component, the repository, with two outputs: the site's files and
/// the server's image.
pub const POLICY: &str = r#"outputs = ["site", "server"]

[projections]
site = "file"
server = "image"
"#;

/// Two components, each with its own paths, tag prefix and one output.
pub const TWO_COMPONENTS: &str = r#"[[component]]
name = "lib"
outputs = ["library"]
paths = ["lib/**"]
prefix = "lib-v"
projections = { library = "file" }

[[component]]
name = "app"
outputs = ["server"]
paths = ["app/**"]
prefix = "app-v"
projections = { server = "image" }
"#;

/// A write the store was asked, in the order asked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Write {
    /// put-bytes, by the digest of the bytes given.
    Bytes(Digest),
    /// put-artifact, an artifact's manifest or an index's.
    Manifest(Manifest),
    /// attach, of `envelope` beside `subject`.
    Attach { subject: Digest, envelope: Envelope },
    /// name, moving `name` to `to` from `expected`.
    Name {
        name: Name,
        to: Digest,
        expected: Option<Digest>,
    },
}

impl Write {
    /// Whether this is the put of an index.
    pub fn is_index(&self) -> bool {
        matches!(self, Write::Manifest(Manifest::Index(_)))
    }
}

/// A pack the core asked of a projection, and the tags the shared history
/// held when it was asked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Packing {
    pub ecosystem: Ecosystem,
    pub version: Version,
    pub shared_tags: Vec<String>,
}

/// Every seam a fake, a shared history and a clone of it, the clone's
/// record, and the revisions the world was made at.
pub struct FakeWorld {
    pub fixture: Fixture,
    pub fakes: Fakes,
    pub main: Branch,
    pub host: SignerId,
    pub environment: Environment,
    /// The revision every component was first released at, tagged.
    pub start: Revision,
    /// One commit after `start`, a feature, neither landed nor built.
    pub head: Revision,
    packs: Rc<RefCell<Vec<Packing>>>,
}

impl Rig for FakeWorld {
    fn profile(&self) -> String {
        tailrace_fakes::PROFILE.into()
    }
}

impl Default for FakeWorld {
    fn default() -> Self {
        Self::new()
    }
}

/// The history of [`FakeWorld::new`], made with git once per process
/// and copied for each world after.
fn one_component() -> (Fixture, Revision, Revision) {
    let fixture = Fixture::new();
    let repo = &fixture.repo;
    repo.policy_file(POLICY);
    let start = repo.commit(
        &[
            ("site/index.html", "<p>the first site</p>"),
            ("server/main.txt", "the first server"),
        ],
        "feat: the first site and server",
    );
    repo.tag("v0.1.0", &start);
    let head = repo.commit(
        &[("server/main.txt", "the server answers")],
        "feat: the server answers",
    );
    repo.push();
    (fixture, start, head)
}

/// The history of [`FakeWorld::two_components`], made with git once per
/// process and copied for each world after.
fn two_components() -> (Fixture, Revision, Revision) {
    let fixture = Fixture::new();
    let repo = &fixture.repo;
    repo.policy_file(TWO_COMPONENTS);
    let start = repo.commit(
        &[
            ("lib/lib.txt", "the first lib"),
            ("app/app.txt", "the first app"),
        ],
        "feat: the first lib and app",
    );
    repo.tag("lib-v0.1.0", &start);
    repo.tag("app-v0.1.0", &start);
    let head = repo.commit(&[("lib/lib.txt", "the lib grows")], "feat: the lib grows");
    repo.push();
    (fixture, start, head)
}

/// The build identity the world's build reports `output` at `revision`
/// from.
pub fn build_of(output: &str, revision: &Revision) -> BuildId {
    BuildId::new(format!("{output}@{}", &revision.as_str()[..12]))
}

fn changes<T>(calls: Vec<Call<T>>) -> Vec<T> {
    calls.into_iter().map(|c| c.op).collect()
}

impl FakeWorld {
    /// The repository declaring no component, so one component, the
    /// repository, with outputs `site` and `server`: released as `v0.1.0`
    /// at `start`, one feature since at `head`.
    pub fn new() -> FakeWorld {
        static MADE: OnceLock<(Snapshot, Revision, Revision)> = OnceLock::new();
        let (snapshot, start, head) = MADE.get_or_init(|| {
            let (fixture, start, head) = one_component();
            (fixture.snapshot(), start, head)
        });
        FakeWorld::over(Fixture::from(snapshot), start.clone(), head.clone())
    }

    /// The repository declaring two components, `lib` over `lib/**` with
    /// prefix `lib-v` and `app` over `app/**` with prefix `app-v`, each
    /// with one output and its projection, released at `start`; `head`
    /// touches `lib` alone.
    pub fn two_components() -> FakeWorld {
        static MADE: OnceLock<(Snapshot, Revision, Revision)> = OnceLock::new();
        let (snapshot, start, head) = MADE.get_or_init(|| {
            let (fixture, start, head) = two_components();
            (fixture.snapshot(), start, head)
        });
        FakeWorld::over(Fixture::from(snapshot), start.clone(), head.clone())
    }

    fn over(fixture: Fixture, start: Revision, head: Revision) -> FakeWorld {
        let mut fakes = Fakes::new();
        let host = SignerId::new(HOST);
        fakes.signer.hold(&host);
        fakes.projection(&Ecosystem::new(FILE));
        fakes.projection(&Ecosystem::new(IMAGE));
        let environment = Environment {
            name: EnvironmentName::new(PRODUCTION),
            channel: ChannelId::new(None, ChannelName::new(STABLE)),
            waits: false,
        };
        fakes.target(&environment.name);
        FakeWorld {
            fixture,
            fakes,
            main: Branch::new(MAIN),
            host,
            environment,
            start,
            head,
            packs: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// The clone the host publishes from.
    pub fn repo(&self) -> &Repo {
        &self.fixture.repo
    }

    /// The history every publisher shares.
    pub fn shared(&self) -> &Repo {
        &self.fixture.shared
    }

    /// `revision` landed on `main`, the landing recording `verdicts` for
    /// it.
    pub fn land(&self, revision: &Revision, verdicts: &[Verdict]) {
        self.fakes.landing.land(&self.main, revision);
        for verdict in verdicts {
            self.fakes.landing.verdict(revision, verdict.clone());
        }
    }

    /// Each of `outputs` left at `revision` by its own build, ending in
    /// its outcome.
    pub fn outputs(&self, revision: &Revision, outputs: &[(&str, Outcome)]) {
        for (output, outcome) in outputs {
            self.fakes.build.made(
                revision,
                &OutputName::new(*output),
                Report {
                    build: build_of(output, revision),
                    outcome: *outcome,
                    files: vec![format!("dist/{output}")],
                },
            );
        }
    }

    /// A verdict of `check` on `build` at `revision`, signed by `signer`
    /// as another party's signer signs it, not yet recorded.
    pub fn verdict(
        &self,
        revision: &Revision,
        check: &str,
        build: &BuildId,
        outcome: Outcome,
        signer: &str,
    ) -> Verdict {
        let id = SignerId::new(signer);
        let theirs = FakeSigner::new();
        theirs.hold(&id);
        let claim = Claim {
            subject: Digest::of(build.as_str().as_bytes()),
            kind: ClaimKind::Verdict,
            revision: revision.clone(),
            build: Some(build.clone()),
            body: json!({ "check": check, "outcome": outcome, "their own": [0, 255] }),
        };
        let envelope = theirs
            .sign(&claim, &id)
            .expect("their signer holds their identity");
        Verdict {
            check: Check::new(check),
            build: build.clone(),
            outcome,
            signer: id,
            envelope,
            found: format!("the fake landing's record of {check} on {build}"),
        }
    }

    /// What publish is asked for `revision` on `main`, every component the
    /// landing touched, signed as the host and offered to the world's
    /// store.
    pub fn ask(&self, revision: &Revision) -> Ask {
        Ask {
            branch: self.main.clone(),
            revision: revision.clone(),
            component: None,
            signer: self.host.clone(),
            store: self.store(),
        }
    }

    /// The store as a host's bindings would name it, which the record
    /// keeps of each version (61, 118): the fake store, by its profile.
    pub fn store(&self) -> Source {
        Source::new(Seam::Store, self.profile())
    }

    /// What verify is asked of `version` on `main` under `audience`.
    pub fn verify_ask(&self, version: &str, audience: Audience) -> VerifyAsk {
        VerifyAsk {
            branch: self.main.clone(),
            component: None,
            version: Version::new(version),
            audience,
        }
    }

    /// The fakes as the one binding, each projection watched so the world
    /// sees what the shared history held at each pack.
    pub fn seams(&self) -> Seams {
        let mut seams = self.fakes.seams();
        for (ecosystem, projection) in &self.fakes.projections {
            seams.projections.insert(
                ecosystem.clone(),
                Box::new(Watched {
                    inner: projection.clone(),
                    shared: self.shared().clone(),
                    packs: self.packs.clone(),
                }),
            );
        }
        seams
    }

    pub fn history(&self) -> History {
        History::open(&self.repo().path).expect("the clone is a repository")
    }

    /// The host's record of what it published from the clone.
    pub fn kept(&self) -> Kept {
        Kept::at(&self.history())
    }

    pub fn publish(&self, ask: &Ask) -> Answer<Publish> {
        let history = self.history();
        publish(&self.seams(), &history, &Kept::at(&history), ask)
    }

    pub fn verify(&self, ask: &VerifyAsk) -> Answer<Verified> {
        verify(&self.seams(), &self.history(), ask)
    }

    /// Every write the store was asked, in order: puts, attaches and name
    /// moves.
    pub fn writes(&self) -> Vec<Write> {
        self.fakes
            .store
            .calls()
            .into_iter()
            .filter_map(|call| {
                let given = call.given;
                let read = |key: &str| given[key].clone();
                Some(match call.op {
                    StoreOp::PutBytes => {
                        let bytes: Vec<u8> = serde_json::from_value(read("bytes")).ok()?;
                        Write::Bytes(Digest::of(&bytes))
                    }
                    StoreOp::PutArtifact => {
                        Write::Manifest(serde_json::from_value(read("manifest")).ok()?)
                    }
                    StoreOp::Attach => Write::Attach {
                        subject: serde_json::from_value(read("subject")).ok()?,
                        envelope: serde_json::from_value(read("attestation")).ok()?,
                    },
                    StoreOp::Name => Write::Name {
                        name: serde_json::from_value(read("name")).ok()?,
                        to: serde_json::from_value(read("to")).ok()?,
                        expected: serde_json::from_value(read("expected")).ok()?,
                    },
                    StoreOp::ClaimsOf | StoreOp::Resolve | StoreOp::Read => return None,
                })
            })
            .collect()
    }

    /// Every operation asked of the store, in order.
    pub fn store_calls(&self) -> Vec<StoreOp> {
        changes(self.fakes.store.calls())
    }

    pub fn keeper_calls(&self) -> Vec<KeeperOp> {
        changes(self.fakes.keeper.calls())
    }

    /// Every operation asked of every target bound.
    pub fn target_calls(&self) -> Vec<TargetOp> {
        self.fakes
            .targets
            .values()
            .flat_map(|t| changes(t.calls()))
            .collect()
    }

    pub fn build_calls(&self) -> Vec<Call<BuildOp>> {
        self.fakes.build.calls()
    }

    /// Every pack asked, in order, with what the shared history held then.
    pub fn packs(&self) -> Vec<Packing> {
        self.packs.borrow().clone()
    }

    /// The host's record deleted, its directory and every file in it,
    /// as a person who deletes what tailrace keeps does (63).
    pub fn delete_record(&self) {
        let dir = self.kept().dir().to_path_buf();
        fs::remove_dir_all(&dir)
            .unwrap_or_else(|e| panic!("the record at {} was not deleted: {e}", dir.display()));
    }

    /// Every file of the host's record, by path, with its bytes and when
    /// it was last written; none when the record holds nothing.
    pub fn record(&self) -> Vec<(PathBuf, Vec<u8>, SystemTime)> {
        fn walk(dir: &Path, found: &mut Vec<(PathBuf, Vec<u8>, SystemTime)>) {
            let Ok(entries) = fs::read_dir(dir) else {
                return;
            };
            for entry in entries {
                let path = entry.expect("the record is readable").path();
                if path.is_dir() {
                    walk(&path, found);
                } else {
                    let modified = fs::metadata(&path)
                        .and_then(|m| m.modified())
                        .expect("the record is readable");
                    let bytes = fs::read(&path).expect("the record is readable");
                    found.push((path, bytes, modified));
                }
            }
        }
        let mut found = Vec::new();
        walk(self.kept().dir(), &mut found);
        found.sort();
        found
    }

    /// How many operations every fake has been asked, all told.
    pub fn asked(&self) -> usize {
        let fakes = &self.fakes;
        fakes.store.calls().len()
            + fakes.build.calls().len()
            + fakes.landing.calls().len()
            + fakes.signer.calls().len()
            + fakes.policy.calls().len()
            + fakes.keeper.calls().len()
            + fakes
                .projections
                .values()
                .map(|p| p.calls().len())
                .sum::<usize>()
            + fakes
                .targets
                .values()
                .map(|t| t.calls().len())
                .sum::<usize>()
    }
}

/// A projection that hands every operation to its fake and, at each
/// pack of an artifact, keeps the tags the shared history holds.
struct Watched {
    inner: FakeProjection,
    shared: Repo,
    packs: Rc<RefCell<Vec<Packing>>>,
}

impl Projection for Watched {
    fn pack(&self, output: &Output, at: &BuildId, version: Option<&Version>) -> Answer<Packed> {
        let packed = self.inner.pack(output, at, version);
        if let (
            Ok(Packed {
                meta: Some(meta), ..
            }),
            Some(version),
        ) = (&packed, version)
        {
            self.packs.borrow_mut().push(Packing {
                ecosystem: meta.ecosystem().clone(),
                version: version.clone(),
                shared_tags: self.shared.tags(),
            });
        }
        packed
    }

    fn render(&self, contents: Contents<'_>, at: &str) -> Answer<Rendered> {
        self.inner.render(contents, at)
    }

    fn serve(&self, layout: &Layout) -> Answer<Serving> {
        self.inner.serve(layout)
    }
}

/// The identity the gate signs as, held by a signer of its own beside
/// the host's (95).
pub const GATE: &str = "gate";

/// The moment the world's gate judges at, in seconds since the Unix
/// epoch.
pub const WHEN: u64 = 1_767_225_600;

impl FakeWorld {
    /// The policy file committed with `text` at the clone, pushed, and
    /// landed on `main` with no verdicts; its revision.
    pub fn land_policy(&self, text: &str) -> Revision {
        let revision = self.repo().policy_file(text);
        self.repo().push();
        self.land(&revision, &[]);
        revision
    }

    /// What the gate is asked of `version` on `main`, told of it by
    /// `told`, signing as `gate`.
    pub fn gate_ask(
        &self,
        version: &str,
        told: tailrace_core::ToldBy,
        gate: &str,
    ) -> tailrace_core::AcceptAsk {
        tailrace_core::AcceptAsk {
            branch: self.main.clone(),
            component: None,
            version: Version::new(version),
            told,
            gate: SignerId::new(gate),
            when: WHEN,
        }
    }

    /// The fakes as the one binding, a signer of its own holding each of
    /// `holding` bound after the host's.
    pub fn seams_holding(&self, holding: &[&str]) -> Seams {
        let mut seams = self.seams();
        for id in holding {
            let signer = FakeSigner::new();
            signer.hold(&SignerId::new(*id));
            seams.signers.push(Box::new(signer));
        }
        seams
    }

    /// The gate's judgment of `ask`, each of `holding` at hand beside the
    /// host's identity.
    pub fn judge(
        &self,
        ask: &tailrace_core::AcceptAsk,
        holding: &[&str],
    ) -> Answer<tailrace_core::Judged> {
        tailrace_core::accept(&self.seams_holding(holding), &self.history(), ask)
    }
}

/// The second signer a channel's policy may require a rebuild by, beside
/// the gate (30).
pub const REBUILDER: &str = "rebuild";

impl FakeWorld {
    /// A claim of `kind` about `subject` for `revision`, its body `body`,
    /// signed as `signer` by a signer of its own and attached to the
    /// subject; its envelope's digest.
    pub fn attach_as(
        &self,
        signer: &str,
        subject: &Digest,
        kind: ClaimKind,
        revision: &Revision,
        body: serde_json::Value,
    ) -> Digest {
        let id = SignerId::new(signer);
        let theirs = FakeSigner::new();
        theirs.hold(&id);
        let claim = Claim {
            subject: subject.clone(),
            kind,
            revision: revision.clone(),
            build: None,
            body,
        };
        let envelope = theirs
            .sign(&claim, &id)
            .expect("their signer holds their identity");
        tailrace_seams::Store::attach(&self.fakes.store, subject, &envelope)
            .expect("the subject is held");
        envelope.digest()
    }

    /// The fake policy for `channel`, of the one component: the gate's
    /// acceptance required and, where `rebuild`, a rebuild by
    /// [`REBUILDER`], the gate named for acceptance and promotion and the
    /// rebuild signer for rebuilds (82, 108); bound to `bound_to` where it
    /// names a channel (83); and the gate its one promoter (32).
    pub fn channel_policy(&self, channel: &str, rebuild: bool, bound_to: Option<&str>) {
        let named = |id: &str, kinds: &[ClaimKind]| tailrace_seams::NamedSigner {
            id: SignerId::new(id),
            verified_by: format!("{id}'s key"),
            kinds: kinds.to_vec(),
            checks: vec![],
        };
        let gate = named(GATE, &[ClaimKind::Acceptance, ClaimKind::Promotion]);
        let mut claims = vec![tailrace_seams::RequiredClaim {
            kind: ClaimKind::Acceptance,
            check: None,
            signers: vec![gate.clone()],
        }];
        if rebuild {
            claims.push(tailrace_seams::RequiredClaim {
                kind: ClaimKind::Rebuild,
                check: None,
                signers: vec![named(REBUILDER, &[ClaimKind::Rebuild])],
            });
        }
        self.fakes.policy.require(
            Audience::Channel(ChannelId::new(None, ChannelName::new(channel))),
            tailrace_seams::Required::Channel {
                claims,
                bound_to: bound_to.map(ChannelName::new),
                promoted_by: vec![gate],
            },
        );
    }

    /// What a promotion of `channel` to `to` on `main` is asked, the gate
    /// asking and signing at [`WHEN`].
    pub fn promote_ask(&self, channel: &str, to: tailrace_core::To) -> tailrace_core::PromoteAsk {
        tailrace_core::PromoteAsk {
            branch: self.main.clone(),
            component: None,
            channel: ChannelName::new(channel),
            to,
            from: None,
            signer: SignerId::new(GATE),
            at: WHEN,
        }
    }

    /// The fakes as the one binding with a signer holding the gate's
    /// identity bound first, as on the one host that is the gate (108).
    pub fn seams_as_gate(&self) -> Seams {
        let mut seams = self.seams();
        let gate = FakeSigner::new();
        gate.hold(&SignerId::new(GATE));
        seams.signers.insert(0, Box::new(gate));
        seams
    }

    /// The promotion `ask` names, over [`FakeWorld::seams_as_gate`].
    pub fn promote(&self, ask: &tailrace_core::PromoteAsk) -> Answer<tailrace_core::Promoted> {
        tailrace_core::promote(&self.seams_as_gate(), &self.history(), ask)
    }
}

/// The moment the status group's first promotion was made at, in
/// seconds since the Unix epoch; each after it a second later.
pub const PROMOTED: u64 = 1_767_312_000;

impl FakeWorld {
    /// 131: each of `channels` declared through the fake policy in the
    /// world's one component, `repository`, its policy requiring nothing,
    /// bound to the channel beside it where one is named (37, 125).
    pub fn declare_channels(&self, channels: &[(&str, Option<&str>)]) {
        for (channel, bound_to) in channels {
            self.fakes.policy.require(
                Audience::Channel(ChannelId::new(None, ChannelName::new(*channel))),
                tailrace_seams::Required::Channel {
                    claims: vec![],
                    bound_to: bound_to.map(ChannelName::new),
                    promoted_by: vec![],
                },
            );
        }
    }

    /// 33, 105: a promotion of `channel` to `version`, a version the
    /// fake store names `v<version>`, appended to the fake keeper after
    /// the chain's head, from the version the head moved to, asked by the
    /// gate and sealed by a fake signer holding its identity; its
    /// claim's digest. Each is made a second after the one before.
    pub fn promote_fake(&self, channel: &str, version: &str) -> Digest {
        let channel = ChannelId::new(None, ChannelName::new(channel));
        let index =
            tailrace_seams::Store::resolve(&self.fakes.store, &Name::new(format!("v{version}")))
                .expect("the version is named in the store");
        let read = tailrace_core::channel(&self.fakes.keeper, &channel).expect("the chain is read");
        let (from, predecessor, made) = match read {
            tailrace_core::Channel::Empty => (None, None, 0),
            tailrace_core::Channel::At { head, chain, .. } => (
                Some(head.to),
                chain.last().map(|(d, _)| d.clone()),
                chain.len() as u64,
            ),
        };
        let promotion = tailrace_seams::Promotion {
            channel: channel.name.clone(),
            component: channel.component_name(),
            to: tailrace_seams::Moved {
                version: Version::new(version),
                index,
            },
            from,
            predecessor: predecessor.clone(),
            asked_by: SignerId::new(GATE),
            policy: tailrace_seams::Satisfied {
                revision: self.head.clone(),
                claims: vec![],
            },
            at: PROMOTED + made,
        };
        let gate = FakeSigner::new();
        gate.hold(&SignerId::new(GATE));
        let envelope = gate
            .sign(&promotion.claim(self.head.clone()), &SignerId::new(GATE))
            .expect("the gate's signer holds its identity");
        tailrace_seams::Keeper::append(
            &self.fakes.keeper,
            &channel,
            &envelope,
            predecessor.as_ref(),
        )
        .expect("the keeper holds the promotion")
    }
}

#[cfg(test)]
mod tests {
    use tailrace_core::Landed;
    use tailrace_seams::Store;

    use super::*;

    #[test]
    fn two_worlds_built_alike_have_equal_revisions() {
        let ((_a, a_start, a_head), (_b, b_start, b_head)) = (one_component(), one_component());
        assert_eq!((&a_start, &a_head), (&b_start, &b_head));
        assert_ne!(a_start, a_head);
        let ((_c, c_start, c_head), (_d, d_start, d_head)) = (two_components(), two_components());
        assert_eq!((&c_start, &c_head), (&d_start, &d_head));
    }

    #[test]
    fn a_world_copied_holds_the_history_it_was_copied_from() {
        let world = FakeWorld::new();
        let repo = world.repo();
        assert_eq!(repo.head(), world.head);
        assert_eq!(repo.tags(), ["v0.1.0"]);
        assert_eq!(world.shared().tag_at("v0.1.0"), Some(world.start.clone()));
        let remote = repo.git(&["config", "remote.origin.url"]);
        assert_eq!(
            Path::new(&remote).canonicalize().unwrap(),
            world.shared().path.canonicalize().unwrap()
        );
        let next = repo.commit(&[("site/index.html", "<p>again</p>")], "fix: again");
        repo.push();
        assert_eq!(world.shared().git(&["rev-parse", MAIN]), next.as_str());
        assert_ne!(
            FakeWorld::new().repo().path,
            repo.path,
            "two worlds share a directory"
        );
    }

    #[test]
    fn a_tag_made_in_one_clone_is_read_by_the_other_through_the_shared_history() {
        let world = FakeWorld::new();
        let second = world.fixture.clone_again();
        world.repo().tag("v9.0.0", &world.head);
        world.repo().push();
        assert_eq!(second.tag_at("v9.0.0"), None, "the second clone fetched it");

        world.land(&world.head, &[]);
        let landed = Landed::check(&world.fakes.landing, &world.main, &world.head).unwrap();
        let history = History::open(&second.path).unwrap();
        let tags = history.tags(&landed, "v").unwrap();
        let names: Vec<&str> = tags.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, ["v0.1.0", "v9.0.0"]);
        assert_eq!(tags[1].revision, world.head);
    }

    #[test]
    fn writes_are_listed_in_the_order_they_were_asked() {
        let world = FakeWorld::new();
        let store = &world.fakes.store;
        let digest = store.put_bytes(b"the bytes").unwrap().digest;
        let envelope = Envelope::signed(
            tailrace_seams::IN_TOTO,
            b"{}".to_vec(),
            tailrace_seams::Signature {
                signer: world.host.clone(),
                keyid: String::new(),
                sig: vec![1],
            },
        );
        store.attach(&digest, &envelope).unwrap();
        store.resolve(&Name::new("v1")).unwrap_err();
        store.name(&Name::new("v1"), &digest, None).unwrap();
        assert_eq!(
            world.writes(),
            [
                Write::Bytes(digest.clone()),
                Write::Attach {
                    subject: digest.clone(),
                    envelope,
                },
                Write::Name {
                    name: Name::new("v1"),
                    to: digest,
                    expected: None,
                },
            ]
        );
        assert_eq!(
            world.store_calls(),
            [
                StoreOp::PutBytes,
                StoreOp::Attach,
                StoreOp::Resolve,
                StoreOp::Name
            ]
        );
        assert_eq!(world.profile(), "fake");
        assert_eq!(
            Source::new(Seam::Store, world.profile()),
            Source::new(Seam::Store, tailrace_fakes::PROFILE)
        );
    }
}
