//! The fake store (B.1): what it holds by digest, the attestations beside
//! each, and its names, in memory.

use std::collections::BTreeMap;

use serde_json::json;
use tailrace_seams::{
    canonical, Answer, Digest, Envelope, Held, Manifest, Name, Not, Put, Seam, Store, Why,
};

use crate::Fake;

/// The fake store's operations, as its record and its faults name them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreOp {
    PutBytes,
    PutArtifact,
    Attach,
    ClaimsOf,
    Name,
    Resolve,
    Read,
}

/// Everything the fake store holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StoreState {
    /// Bytes and manifests, by the digest they are held under.
    pub held: BTreeMap<Digest, Held>,
    /// The attestations attached beside each digest, in the order
    /// attached.
    pub attached: BTreeMap<Digest, Vec<Envelope>>,
    pub names: BTreeMap<Name, Digest>,
}

impl StoreState {
    fn hold(&mut self, digest: Digest, what: Held) -> Put {
        let held_already = self.held.contains_key(&digest);
        self.held.entry(digest.clone()).or_insert(what);
        Put {
            digest,
            held_already,
        }
    }
}

/// A store in memory. A manifest's digest is the digest of its canonical
/// JSON (design D4).
pub type FakeStore = Fake<StoreState, StoreOp>;

impl Default for FakeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeStore {
    pub fn new() -> Self {
        Fake::with(Seam::Store, StoreState::default())
    }

    /// 2: `bytes`, not its own, held under `digest`, so read answers them
    /// and the core can refuse naming both.
    pub fn corrupt(&self, digest: &Digest, bytes: &[u8]) {
        self.state()
            .held
            .insert(digest.clone(), Held::Bytes(bytes.to_vec()));
    }

    /// 54: `name` removed and what it named left held, as a store's own
    /// retention removes a cache entry's name; no operation of the store
    /// seam removes anything.
    pub fn forget(&self, name: &Name) {
        self.state().names.remove(name);
    }

    /// Everything the store holds now.
    pub fn contents(&self) -> StoreState {
        self.state().clone()
    }
}

impl Store for FakeStore {
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        let digest = Digest::of(bytes);
        self.ask(
            StoreOp::PutBytes,
            json!({ "bytes": bytes }),
            format!("the put of {digest}"),
            |s, _| Ok(s.hold(digest, Held::Bytes(bytes.to_vec()))),
        )
    }

    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        let digest = Digest::of(&canonical(manifest));
        self.ask(
            StoreOp::PutArtifact,
            json!({ "manifest": manifest }),
            format!("the put of {digest}"),
            |s, _| Ok(s.hold(digest, Held::Manifest(manifest.clone()))),
        )
    }

    fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
        let given = json!({ "subject": subject, "attestation": attestation });
        self.ask(
            StoreOp::Attach,
            given,
            format!("the claims of {subject}"),
            |s, source| {
                if !s.held.contains_key(subject) {
                    return Err(Not::absent(source, subject.to_string()));
                }
                let attached = s.attached.entry(subject.clone()).or_default();
                let held_already = attached.contains(attestation);
                if !held_already {
                    attached.push(attestation.clone());
                }
                Ok(Put {
                    digest: attestation.digest(),
                    held_already,
                })
            },
        )
    }

    fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>> {
        self.ask(
            StoreOp::ClaimsOf,
            json!({ "digest": digest }),
            format!("the claims of {digest}"),
            |s, source| {
                if !s.held.contains_key(digest) {
                    return Err(Not::absent(source, digest.to_string()));
                }
                Ok(s.attached.get(digest).cloned().unwrap_or_default())
            },
        )
    }

    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
        let given = json!({ "name": name, "to": to, "expected": expected });
        self.ask(
            StoreOp::Name,
            given,
            format!("the name {name}"),
            |s, source| {
                let at = s.names.get(name).cloned();
                if at.as_ref() != expected {
                    return Err(Not::refused(source, Why::NameAt { at }, None));
                }
                s.names.insert(name.clone(), to.clone());
                Ok(())
            },
        )
    }

    fn resolve(&self, name: &Name) -> Answer<Digest> {
        self.ask(
            StoreOp::Resolve,
            json!({ "name": name }),
            format!("the name {name}"),
            |s, source| {
                s.names
                    .get(name)
                    .cloned()
                    .ok_or_else(|| Not::absent(source, format!("name {name}")))
            },
        )
    }

    fn read(&self, digest: &Digest) -> Answer<Held> {
        self.ask(
            StoreOp::Read,
            json!({ "digest": digest }),
            digest.to_string(),
            |s, source| {
                s.held
                    .get(digest)
                    .cloned()
                    .ok_or_else(|| Not::absent(source, digest.to_string()))
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use tailrace_seams::{
        Artifact, ArtifactMeta, ComponentName, Ecosystem, EcosystemName, IndexMeta, Signature,
        SignerId, Source, Version, IN_TOTO,
    };

    use super::*;
    use crate::{Fault, Times};

    fn artifact(name: &str) -> Manifest {
        Manifest::Artifact {
            meta: ArtifactMeta::new(
                Ecosystem::new("eco"),
                EcosystemName::new(name),
                Version::new("1.0.0"),
            )
            .unwrap(),
            layers: vec![Digest::of(name.as_bytes())],
        }
    }

    fn envelope(payload: &[u8]) -> Envelope {
        Envelope::signed(
            IN_TOTO,
            payload.to_vec(),
            Signature {
                signer: SignerId::new("host"),
                keyid: String::new(),
                sig: vec![1],
            },
        )
    }

    fn source() -> Source {
        Source::new(Seam::Store, "fake")
    }

    #[test]
    fn the_same_bytes_put_twice_are_held_once() {
        let store = FakeStore::new();
        let first = store.put_bytes(b"site").unwrap();
        let after_first = store.contents();
        let second = store.put_bytes(b"site").unwrap();
        assert_eq!(first.digest, Digest::of(b"site"));
        assert!(!first.held_already);
        assert_eq!(second.digest, first.digest);
        assert!(second.held_already);
        assert_eq!(store.contents(), after_first);
        assert_eq!(store.contents().held.len(), 1);
    }

    #[test]
    fn nothing_is_written_the_second_time() {
        let store = FakeStore::new();
        store.put_bytes(b"site").unwrap();
        store.put_artifact(&artifact("site")).unwrap();
        let after_first = store.contents();
        assert!(store.put_bytes(b"site").unwrap().held_already);
        assert!(store.put_artifact(&artifact("site")).unwrap().held_already);
        assert_eq!(store.contents(), after_first);
        let ops: Vec<StoreOp> = store.calls().iter().map(|c| c.op).collect();
        assert_eq!(
            ops,
            [
                StoreOp::PutBytes,
                StoreOp::PutArtifact,
                StoreOp::PutBytes,
                StoreOp::PutArtifact
            ]
        );
    }

    #[test]
    fn the_order_of_puts_is_recorded() {
        let store = FakeStore::new();
        let a = store.put_artifact(&artifact("a")).unwrap();
        let b = store.put_artifact(&artifact("b")).unwrap();
        let meta = |m: Manifest| match m {
            Manifest::Artifact { meta, .. } => meta,
            Manifest::Index(_) => unreachable!(),
        };
        let index = IndexMeta::new(
            ComponentName::new("web"),
            Version::new("1.0.0"),
            vec![
                Artifact {
                    digest: a.digest,
                    meta: meta(artifact("a")),
                },
                Artifact {
                    digest: b.digest,
                    meta: meta(artifact("b")),
                },
            ],
        )
        .unwrap();
        store.put_artifact(&Manifest::Index(index)).unwrap();
        let given: Vec<String> = store
            .calls()
            .iter()
            .map(|c| match &c.given["manifest"] {
                m if m.get("index").is_some() => "index".to_string(),
                m => m["artifact"]["meta"]["name"].as_str().unwrap().to_string(),
            })
            .collect();
        assert_eq!(given, ["a", "b", "index"]);
    }

    #[test]
    fn an_artifact_reads_back_as_its_manifest() {
        let store = FakeStore::new();
        let put = store.put_artifact(&artifact("site")).unwrap();
        assert_eq!(put.digest, Digest::of(&canonical(&artifact("site"))));
        assert_eq!(
            store.read(&put.digest),
            Ok(Held::Manifest(artifact("site")))
        );
        assert_eq!(
            store.read(&Digest::of(b"site")),
            Err(Not::absent(&source(), Digest::of(b"site").to_string()))
        );
    }

    #[test]
    fn a_digest_not_held_is_absent_never_refused_or_unreadable() {
        let store = FakeStore::new();
        let d = Digest::of(b"nowhere");
        assert_eq!(store.read(&d), Err(Not::absent(&source(), d.to_string())));
    }

    #[test]
    fn a_name_moves_only_from_where_it_is_expected() {
        let store = FakeStore::new();
        let name = Name::new("web-v1.0.0");
        let (a, b, c) = (Digest::of(b"a"), Digest::of(b"b"), Digest::of(b"c"));
        assert_eq!(
            store.resolve(&name),
            Err(Not::absent(&source(), "name web-v1.0.0"))
        );
        assert_eq!(
            store.name(&name, &a, None),
            Ok(()),
            "a name made from nothing"
        );
        assert_eq!(store.resolve(&name), Ok(a.clone()));
        assert_eq!(
            store.name(&name, &b, Some(&c)),
            Err(Not::refused(
                &source(),
                Why::NameAt {
                    at: Some(a.clone())
                },
                None
            ))
        );
        assert_eq!(
            store.name(&name, &b, None),
            Err(Not::refused(
                &source(),
                Why::NameAt {
                    at: Some(a.clone())
                },
                None
            ))
        );
        assert_eq!(store.resolve(&name), Ok(a.clone()), "the name stays");
        assert_eq!(
            store.name(&Name::new("other"), &b, Some(&a)),
            Err(Not::refused(&source(), Why::NameAt { at: None }, None))
        );
        assert_eq!(store.name(&name, &b, Some(&a)), Ok(()));
        assert_eq!(store.resolve(&name), Ok(b));
    }

    #[test]
    fn attestations_are_held_beside_their_digest() {
        let store = FakeStore::new();
        let d = store.put_bytes(b"site").unwrap().digest;
        let (e1, e2) = (envelope(b"one"), envelope(b"two"));
        assert_eq!(store.claims_of(&d), Ok(vec![]));
        assert_eq!(
            store.attach(&d, &e1),
            Ok(Put {
                digest: e1.digest(),
                held_already: false
            })
        );
        store.attach(&d, &e2).unwrap();
        assert!(store.attach(&d, &e1).unwrap().held_already);
        assert_eq!(store.claims_of(&d), Ok(vec![e1.clone(), e2]));

        let elsewhere = Digest::of(b"not held");
        assert_eq!(
            store.claims_of(&elsewhere),
            Err(Not::absent(&source(), elsewhere.to_string()))
        );
        assert_eq!(
            store.attach(&elsewhere, &e1),
            Err(Not::absent(&source(), elsewhere.to_string()))
        );
        assert!(!store.contents().attached.contains_key(&elsewhere));
    }

    #[test]
    fn a_store_that_cannot_be_reached_answers_unreadable_never_empty() {
        let store = FakeStore::new();
        let d = store.put_bytes(b"site").unwrap().digest;
        store.fault(
            StoreOp::ClaimsOf,
            Fault::Unreadable("connection refused".into()),
            Times::UntilHealed,
        );
        let answer = store.claims_of(&d);
        assert_eq!(
            answer,
            Err(Not::unreadable(
                &source(),
                format!("the claims of {d}"),
                Some("connection refused".into())
            ))
        );
        let Err(Not::Unreadable(u)) = answer else {
            unreachable!()
        };
        assert_eq!(
            u.said.as_deref().map(str::as_bytes),
            Some("connection refused".as_bytes())
        );
        store.heal(StoreOp::ClaimsOf);
        assert_eq!(store.claims_of(&d), Ok(vec![]));
    }

    #[test]
    fn a_put_refused_once_holds_nothing_and_the_next_holds_the_bytes() {
        let store = FakeStore::new();
        store.fault(
            StoreOp::PutBytes,
            Fault::Refused("quota exceeded".into()),
            Times::Once,
        );
        assert_eq!(
            store.put_bytes(b"site"),
            Err(Not::refused(
                &source(),
                Why::Said,
                Some("quota exceeded".into())
            ))
        );
        assert!(store.contents().held.is_empty());
        assert!(!store.put_bytes(b"site").unwrap().held_already);
        assert_eq!(store.contents().held.len(), 1);
    }

    #[test]
    fn other_bytes_under_a_digest_are_read_back() {
        let store = FakeStore::new();
        let d = store.put_bytes(b"site").unwrap().digest;
        store.corrupt(&d, b"other");
        assert_eq!(store.read(&d), Ok(Held::Bytes(b"other".to_vec())));
    }

    #[test]
    fn a_forgotten_name_resolves_absent_and_its_digest_still_reads() {
        let store = FakeStore::new();
        let name = Name::new("cache/b7e1");
        let d = store.put_bytes(b"archive").unwrap().digest;
        store.name(&name, &d, None).unwrap();
        store.forget(&name);
        assert_eq!(
            store.resolve(&name),
            Err(Not::absent(&source(), "name cache/b7e1"))
        );
        assert_eq!(store.read(&d), Ok(Held::Bytes(b"archive".to_vec())));
        assert_eq!(
            store.name(&name, &d, None),
            Ok(()),
            "made again from nothing"
        );
    }

    #[test]
    fn state_is_seen_through_a_copy() {
        let kept = FakeStore::new();
        let bound: Box<dyn Store> = Box::new(kept.clone());
        let d = bound.put_bytes(b"site").unwrap().digest;
        assert!(kept.contents().held.contains_key(&d));
        assert_eq!(kept.calls().len(), 1);
    }

    #[test]
    fn every_operation_can_be_faulted_and_is_recorded() {
        let store = FakeStore::new();
        let d = Digest::of(b"x");
        let name = Name::new("n");
        let ops = [
            StoreOp::PutBytes,
            StoreOp::PutArtifact,
            StoreOp::Attach,
            StoreOp::ClaimsOf,
            StoreOp::Name,
            StoreOp::Resolve,
            StoreOp::Read,
        ];
        for op in ops {
            store.fault(op, Fault::Refused("no".into()), Times::Once);
        }
        let answers = [
            store.put_bytes(b"x").map(drop),
            store.put_artifact(&artifact("x")).map(drop),
            store.attach(&d, &envelope(b"x")).map(drop),
            store.claims_of(&d).map(drop),
            store.name(&name, &d, None).map(drop),
            store.resolve(&name).map(drop),
            store.read(&d).map(drop),
        ];
        for answer in answers {
            assert_eq!(
                answer,
                Err(Not::refused(&source(), Why::Said, Some("no".into())))
            );
        }
        assert_eq!(store.calls().iter().map(|c| c.op).collect::<Vec<_>>(), ops);
        assert_eq!(store.contents(), StoreState::default());
    }
}
