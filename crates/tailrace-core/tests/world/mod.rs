//! A repository with a shared history, two components and the fakes
//! bound, for the tests of publish, status and verify (design D12).

#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;

use serde_json::json;
use tailrace_core::{publish, Ask, Entry, History, Kept, Publish, Published, NOTES, POLICY_FILE};
use tailrace_fakes::{FakeBuild, FakeProjection, FakeSigner, FakeStore, Fakes};
use tailrace_seams::{
    Answer, Artifact, Branch, BuildId, Check, Claim, ClaimKind, ComponentName, Contents, Digest,
    Ecosystem, Envelope, Held, IndexMeta, Layout, Manifest, Name, Outcome, Output, OutputName,
    Packed, Pinned, Projection, Put, Range, Rendered, Report, Requirement, Revision, Seam, Serving,
    SignerId, Source, Store, Verdict, Version,
};

use crate::common::{Fixture, Repo, CLIFF_TOML};

/// Two components: `lib`, two outputs packed in two ecosystems, and
/// `web`, one.
pub const POLICY: &str = r#"
[[component]]
name = "lib"
outputs = ["lib-pack", "lib-docs"]
paths = ["crates/lib/**"]
prefix = "lib-v"
projections = { lib-pack = "eco-a", lib-docs = "eco-b" }

[[component]]
name = "web"
outputs = ["site"]
paths = ["site/**"]
prefix = "web-v"
projections = { site = "eco-a" }
"#;

pub const HOST: &str = "host";
pub const LANDING: &str = "landing";

/// The store as the host's bindings name it, which the record keeps of
/// each version (61, 118).
pub fn store() -> Source {
    Source::new(Seam::Store, "layout /srv/tailrace/layout")
}

pub struct World {
    pub fixture: Fixture,
    pub shared: Repo,
    pub repo: Repo,
    pub fakes: Fakes,
    pub main: Branch,
    /// The revision both components were first tagged at.
    pub start: Revision,
    /// The revision a publish is asked of: one commit after `start`,
    /// touching `lib` alone.
    pub head: Revision,
}

pub fn eco(name: &str) -> Ecosystem {
    Ecosystem::new(name)
}

pub fn output(name: &str) -> OutputName {
    OutputName::new(name)
}

/// The build identity the fake build reports `output` at `revision`
/// from.
pub fn build_of(output: &str, revision: &Revision) -> BuildId {
    BuildId::new(format!("{output}@{}", &revision.as_str()[..7]))
}

impl World {
    /// The repository, its shared history and the fakes: the policy and
    /// the notes' template, `lib` and `web` tagged at `start`, one feature on `lib` since, landed on `main`,
    /// every output built and passing, the host's identity at hand, and
    /// both ecosystems bound.
    pub fn new() -> World {
        World::with_policy(POLICY)
    }

    pub fn with_policy(policy: &str) -> World {
        let fixture = Fixture::new();
        let shared = fixture.bare("shared.git");
        let repo = fixture.clone_of(&shared, "repo");
        repo.commit(
            "chore: policy",
            &[(POLICY_FILE, policy), ("cliff.toml", CLIFF_TOML)],
        );
        let start = repo.touch("feat: start", &["crates/lib", "site"]);
        repo.tag("lib-v0.1.0", &start);
        repo.tag("web-v0.1.0", &start);
        let head = repo.touch("feat: lib grows", &["crates/lib"]);
        repo.git(&["push", "-q", "origin", "main", "lib-v0.1.0", "web-v0.1.0"]);

        let mut fakes = Fakes::new();
        let main = Branch::new("main");
        fakes.landing.land(&main, &head);
        fakes.signer.hold(&SignerId::new(HOST));
        fakes.projection(&eco("eco-a"));
        fakes.projection(&eco("eco-b"));
        let world = World {
            fixture,
            shared,
            repo,
            fakes,
            main,
            start,
            head,
        };
        for o in ["lib-pack", "lib-docs", "site"] {
            world.made(o, Outcome::Pass);
        }
        world
    }

    /// `output` built at the head from its own build identity, ending in
    /// `outcome`.
    pub fn made(&self, output: &str, outcome: Outcome) {
        self.fakes.build.made(
            &self.head,
            &OutputName::new(output),
            Report {
                build: build_of(output, &self.head),
                outcome,
                files: vec![format!("dist/{output}.tar")],
            },
        );
    }

    pub fn ask(&self) -> Ask {
        Ask {
            branch: self.main.clone(),
            revision: self.head.clone(),
            component: None,
            signer: SignerId::new(HOST),
            store: store(),
        }
    }

    pub fn ask_for(&self, component: &str) -> Ask {
        Ask {
            component: Some(ComponentName::new(component)),
            ..self.ask()
        }
    }

    pub fn history(&self) -> History {
        self.repo.history()
    }

    pub fn kept(&self) -> Kept {
        Kept::at(&self.history())
    }

    pub fn publish(&self, ask: &Ask) -> Answer<Publish> {
        let history = self.history();
        publish(&self.fakes.seams(), &history, &Kept::at(&history), ask)
    }

    /// The one component's answer of a publish that answered.
    pub fn publish_one(&self, ask: &Ask) -> Answer<Published> {
        let publish = self.publish(ask).expect("the publish answered");
        assert_eq!(publish.components.len(), 1, "{publish:?}");
        publish.components.into_iter().next().unwrap().1
    }

    /// A verdict the landing recorded for the head, of `check` on `build`,
    /// signed by `signer` as the fake signer signs.
    pub fn verdict(&self, check: &str, build: &BuildId, outcome: Outcome, signer: &str) -> Verdict {
        let id = SignerId::new(signer);
        let theirs = FakeSigner::new();
        theirs.hold(&id);
        let claim = Claim {
            subject: Digest::of(build.as_str().as_bytes()),
            kind: ClaimKind::Verdict,
            revision: self.head.clone(),
            build: Some(build.clone()),
            body: json!({ "check": check, "outcome": outcome, "their own": [0, 255] }),
        };
        let envelope = tailrace_seams::Signer::sign(&theirs, &claim, &id).unwrap();
        let verdict = Verdict {
            check: Check::new(check),
            build: build.clone(),
            outcome,
            signer: id,
            envelope,
            found: format!("the fake landing's record of {check} on {build}"),
        };
        self.fakes.landing.verdict(&self.head, verdict.clone());
        verdict
    }

    /// A passing verdict of `check` naming each output's build.
    pub fn verdicts_for_every_output(&self, check: &str) {
        for o in ["lib-pack", "lib-docs", "site"] {
            self.verdict(check, &build_of(o, &self.head), Outcome::Pass, LANDING);
        }
    }
}

/// The version a published answer names, and its index.
pub fn version_of(published: &Published) -> (&Version, &Digest) {
    match published {
        Published::Version { version, index, .. } => (version, index),
        Published::Unchanged { .. } => panic!("unchanged: {published:?}"),
    }
}

/// What a projection was asked to pack, and what `watch` saw then.
pub type Seen<T> = Rc<RefCell<Vec<T>>>;

/// A projection that delegates to a fake and, at each pack, records what
/// `watch` sees, so a test reads what held when packing began.
pub struct Watched<T, F: Fn() -> T> {
    pub inner: FakeProjection,
    pub watch: F,
    pub seen: Seen<T>,
}

impl<T, F: Fn() -> T> Projection for Watched<T, F> {
    fn pack(&self, output: &Output, at: &BuildId, version: Option<&Version>) -> Answer<Packed> {
        self.seen.borrow_mut().push((self.watch)());
        self.inner.pack(output, at, version)
    }

    fn render(&self, contents: Contents<'_>, at: &str) -> Answer<Rendered> {
        self.inner.render(contents, at)
    }

    fn serve(&self, layout: &Layout) -> Answer<Serving> {
        self.inner.serve(layout)
    }

    fn pinned(&self, checkout: &str) -> Answer<Vec<Pinned>> {
        self.inner.pinned(checkout)
    }

    fn requires(&self, held: &Artifact) -> Answer<Vec<Requirement>> {
        self.inner.requires(held)
    }

    fn satisfying(&self, range: &Range, versions: &[Version]) -> Answer<Vec<Version>> {
        self.inner.satisfying(range, versions)
    }
}

/// The calls a fake build recorded, by operation.
pub fn build_ops(build: &FakeBuild) -> Vec<tailrace_fakes::BuildOp> {
    build.calls().into_iter().map(|c| c.op).collect()
}

/// Every envelope `envelopes` holds that the host signed.
pub fn by_host(envelopes: &[Envelope]) -> Vec<&Envelope> {
    envelopes
        .iter()
        .filter(|e| e.signatures.iter().any(|s| s.signer.as_str() == HOST))
        .collect()
}

/// A store that puts as its fake does, but an index with no artifact of
/// the notes' ecosystem, as a publisher before notes put it.
pub struct BeforeNotes(pub FakeStore);

impl Store for BeforeNotes {
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        self.0.put_bytes(bytes)
    }
    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        let Manifest::Index(meta) = manifest else {
            return self.0.put_artifact(manifest);
        };
        let outputs = meta
            .artifacts()
            .iter()
            .filter(|a| a.meta.ecosystem().as_str() != NOTES)
            .cloned()
            .collect();
        let index = IndexMeta::new(meta.component().clone(), meta.version().clone(), outputs);
        self.0.put_artifact(&Manifest::Index(index.unwrap()))
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

/// `world`'s `lib` published by a publisher before notes: its index names
/// the two outputs' artifacts and no notes, and its entry in the host's
/// record names no notes' commits, as that publisher wrote it.
pub fn published_before_notes(world: &World) -> Published {
    let history = world.history();
    let mut seams = world.fakes.seams();
    seams.store = Box::new(BeforeNotes(world.fakes.store.clone()));
    let kept = Kept::at(&history);
    let publish = publish(&seams, &history, &kept, &world.ask_for("lib")).unwrap();
    let first = publish.components.into_iter().next().unwrap().1.unwrap();
    let Published::Version { tag, .. } = &first else {
        panic!("{first:?}")
    };
    let path = kept.dir().join("lib").join(format!("{tag}.json"));
    let mut entry: Entry = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    entry.notes = None;
    let mut text = serde_json::to_string_pretty(&entry).unwrap();
    text.push('\n');
    std::fs::write(&path, text).unwrap();
    first
}
