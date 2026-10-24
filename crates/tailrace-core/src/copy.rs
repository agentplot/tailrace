//! A version copied from one store to another through the store seam
//! alone (103, design D10): its name resolved in the source to its index,
//! then into the target every artifact's bytes, each artifact and every
//! attestation on it, the index last and every attestation on it, and
//! the version's name made from nothing (3, 6, 8, rules core.4, core.12,
//! core.27). Every digest the target answers is the source's, so a copy
//! changes no digest and loses no claim (1, 2). A channel's name is never
//! copied: a channel is the repository's, not a store's (103).

use serde::Serialize;
use tailrace_seams::{Answer, Digest, Held, Manifest, Name, Not, Source, Store, Why};

/// One end of a copy: a store, and the source the copy's own answers
/// name it by, as the host's bindings name it (61, 118).
#[derive(Clone, Copy)]
pub struct End<'a> {
    pub store: &'a dyn Store,
    pub source: &'a Source,
}

/// CopiedKind a copy answers: the version's name and its index, every digest
/// put into the target in the order put with whether the target held it
/// already, and whether it held the name at the index already, so a
/// second copy is seen to write nothing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Copied {
    pub name: Name,
    pub index: Digest,
    pub written: Vec<CopiedDigest>,
    pub name_held_already: bool,
}

/// One digest a copy put into the target.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CopiedDigest {
    pub what: CopiedKind,
    pub digest: Digest,
    pub held_already: bool,
}

/// CopiedKind a digest a copy put is.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CopiedKind {
    Bytes,
    Artifact,
    Index,
    /// An attestation, by its envelope's digest, on the digest named.
    Attestation {
        on: Digest,
    },
}

/// That the target named what it was handed by `answered`, the source's
/// `expected`; refused naming both otherwise (1, 2).
fn same(to: End, expected: &Digest, answered: Digest) -> Answer<Digest> {
    if &answered != expected {
        return Err(Not::refused(
            to.source,
            Why::OtherBytes {
                expected: expected.clone(),
                answered,
            },
            None,
        ));
    }
    Ok(answered)
}

/// 103: the version `name` names in `from` copied whole into `to`. A
/// digest `from` does not hold makes the copy absent naming it, and the
/// target then holds no index for the version; a name `to` holds at the
/// index already is answered held already, and at another digest is
/// refused naming it, after everything else is held, with nothing
/// retried (70, rule core.6).
pub fn copy(from: End, to: End, name: &Name) -> Answer<Copied> {
    let index = from.store.resolve(name)?;
    let not_a = |what: &str, digest: &Digest| {
        Not::unreadable(
            from.source,
            format!("{digest} as {what}"),
            Some(format!("{digest} holds no {what}")),
        )
    };
    let Held::Manifest(Manifest::Index(meta)) = from.store.read(&index)? else {
        return Err(not_a("an index", &index));
    };

    let mut written = Vec::new();
    for artifact in meta.artifacts() {
        let Held::Manifest(manifest @ Manifest::Artifact { .. }) =
            from.store.read(&artifact.digest)?
        else {
            return Err(not_a("an artifact", &artifact.digest));
        };
        let Manifest::Artifact { layers, .. } = &manifest else {
            unreachable!("matched as an artifact")
        };
        for layer in layers {
            let Held::Bytes(bytes) = from.store.read(layer)? else {
                return Err(not_a("bytes", layer));
            };
            same(from, layer, Digest::of(&bytes))?;
            let put = to.store.put_bytes(&bytes)?;
            written.push(CopiedDigest {
                what: CopiedKind::Bytes,
                digest: same(to, layer, put.digest)?,
                held_already: put.held_already,
            });
        }
        let put = to.store.put_artifact(&manifest)?;
        written.push(CopiedDigest {
            what: CopiedKind::Artifact,
            digest: same(to, &artifact.digest, put.digest)?,
            held_already: put.held_already,
        });
        attach(from, to, &artifact.digest, &mut written)?;
    }

    let put = to.store.put_artifact(&Manifest::Index(meta))?;
    written.push(CopiedDigest {
        what: CopiedKind::Index,
        digest: same(to, &index, put.digest)?,
        held_already: put.held_already,
    });
    attach(from, to, &index, &mut written)?;

    let name_held_already = match to.store.name(name, &index, None) {
        Ok(()) => false,
        Err(Not::Refused(r))
            if r.why
                == (Why::NameAt {
                    at: Some(index.clone()),
                }) =>
        {
            true
        }
        Err(not) => return Err(not),
    };
    Ok(Copied {
        name: name.clone(),
        index,
        written,
        name_held_already,
    })
}

/// Every attestation `from` holds on `digest` attached beside it in `to`,
/// as made (8, rule core.27).
fn attach(from: End, to: End, digest: &Digest, written: &mut Vec<CopiedDigest>) -> Answer<()> {
    for envelope in from.store.claims_of(digest)? {
        let put = to.store.attach(digest, &envelope)?;
        written.push(CopiedDigest {
            what: CopiedKind::Attestation { on: digest.clone() },
            digest: same(to, &envelope.digest(), put.digest)?,
            held_already: put.held_already,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tailrace_fakes::{FakeStore, StoreOp};
    use tailrace_seams::{
        Artifact, ArtifactMeta, Claim, ClaimKind, ComponentName, Ecosystem, EcosystemName,
        Envelope, IndexMeta, Put, Revision, Seam, Signature, SignerId, Version, IN_TOTO,
    };

    use super::*;

    fn source(of: &str) -> Source {
        Source::new(Seam::Store, of)
    }

    fn meta(name: &str) -> ArtifactMeta {
        ArtifactMeta::new(
            Ecosystem::new("file"),
            EcosystemName::new(name),
            Version::new("1.3.0"),
        )
        .unwrap()
    }

    fn claim(subject: &Digest, kind: ClaimKind) -> Envelope {
        let claim = Claim {
            subject: subject.clone(),
            kind,
            revision: Revision::new("4f2a9c1"),
            build: None,
            body: json!({ "theirs": [0, 255] }),
        };
        Envelope::signed(
            IN_TOTO,
            claim.payload(),
            Signature {
                signer: SignerId::new("host"),
                keyid: String::new(),
                sig: vec![1, 2, 3],
            },
        )
    }

    /// A version of two artifacts, a provenance and a verdict on each and
    /// an acceptance on the index, published into `store` and named
    /// `web-v1.3.0`; its index.
    fn published(store: &FakeStore) -> Digest {
        let mut artifacts = Vec::new();
        for name in ["site", "server"] {
            let bytes = store.put_bytes(name.as_bytes()).unwrap().digest;
            let digest = store
                .put_artifact(&Manifest::Artifact {
                    meta: meta(name),
                    layers: vec![bytes],
                })
                .unwrap()
                .digest;
            for kind in [ClaimKind::Provenance, ClaimKind::Verdict] {
                store.attach(&digest, &claim(&digest, kind)).unwrap();
            }
            artifacts.push(Artifact {
                digest,
                meta: meta(name),
            });
        }
        let index = store
            .put_artifact(&Manifest::Index(
                IndexMeta::new(ComponentName::new("web"), Version::new("1.3.0"), artifacts)
                    .unwrap(),
            ))
            .unwrap()
            .digest;
        store
            .attach(&index, &claim(&index, ClaimKind::Acceptance))
            .unwrap();
        store.name(&version(), &index, None).unwrap();
        index
    }

    fn version() -> Name {
        Name::new("web-v1.3.0")
    }

    fn copied(from: &dyn Store, to: &dyn Store) -> Answer<Copied> {
        let (a, b) = (source("the layout"), source("the registry"));
        copy(
            End {
                store: from,
                source: &a,
            },
            End {
                store: to,
                source: &b,
            },
            &version(),
        )
    }

    fn indexes(store: &FakeStore) -> usize {
        store
            .contents()
            .held
            .values()
            .filter(|h| matches!(h, Held::Manifest(Manifest::Index(_))))
            .count()
    }

    #[test]
    fn a_version_is_copied_whole_with_every_claim_under_the_same_digests() {
        let (from, to) = (FakeStore::new(), FakeStore::new());
        let index = published(&from);
        let answer = copied(&from, &to).unwrap();
        assert_eq!(answer.index, index);
        assert!(!answer.name_held_already);
        assert!(answer.written.iter().all(|w| !w.held_already));
        let (source, target) = (from.contents(), to.contents());
        assert_eq!(target.held, source.held);
        assert_eq!(target.attached, source.attached);
        assert_eq!(target.names.get(&version()), Some(&index));
        assert_eq!(target.attached.values().flatten().count(), 5);

        let ops: Vec<StoreOp> = to.calls().iter().map(|c| c.op).collect();
        let index_put = to
            .calls()
            .iter()
            .position(|c| c.given["manifest"].get("index").is_some())
            .unwrap();
        let last_artifact = to
            .calls()
            .iter()
            .rposition(|c| c.given["manifest"].get("artifact").is_some())
            .unwrap();
        assert!(last_artifact < index_put, "{ops:?}");
        assert_eq!(ops.last(), Some(&StoreOp::Name), "{ops:?}");
        let kinds: Vec<&CopiedKind> = answer.written.iter().map(|w| &w.what).collect();
        assert_eq!(
            kinds.iter().filter(|w| ***w == CopiedKind::Index).count(),
            1
        );
        assert_eq!(
            answer.written.last().unwrap().what,
            CopiedKind::Attestation { on: index }
        );
    }

    #[test]
    fn a_second_copy_writes_nothing_and_says_so() {
        let (from, to) = (FakeStore::new(), FakeStore::new());
        published(&from);
        let first = copied(&from, &to).unwrap();
        let held = to.contents();
        let second = copied(&from, &to).unwrap();
        assert_eq!(to.contents(), held);
        assert!(second.name_held_already);
        assert_eq!(second.written.len(), first.written.len());
        for (a, b) in first.written.iter().zip(&second.written) {
            assert_eq!((&a.what, &a.digest), (&b.what, &b.digest));
            assert!(b.held_already, "{b:?} was written again");
        }
    }

    /// A store that names every artifact it is handed by another digest.
    struct Skewed(FakeStore);

    impl Store for Skewed {
        fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
            self.0.put_bytes(bytes)
        }
        fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
            let put = self.0.put_artifact(manifest)?;
            Ok(Put {
                digest: Digest::of(b"skewed"),
                ..put
            })
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

    #[test]
    fn a_target_naming_another_digest_is_refused_naming_both_and_takes_no_index() {
        let from = FakeStore::new();
        published(&from);
        let first = match from.read(&from.resolve(&version()).unwrap()).unwrap() {
            Held::Manifest(Manifest::Index(meta)) => meta.artifacts()[0].digest.clone(),
            held => panic!("{held:?}"),
        };
        let to = Skewed(FakeStore::new());
        assert_eq!(
            copied(&from, &to),
            Err(Not::refused(
                &source("the registry"),
                Why::OtherBytes {
                    expected: first,
                    answered: Digest::of(b"skewed"),
                },
                None
            ))
        );
        assert_eq!(indexes(&to.0), 0);
        assert!(to.0.contents().names.is_empty());
    }

    #[test]
    fn a_source_missing_an_artifact_is_absent_naming_it_and_takes_no_index() {
        let from = FakeStore::new();
        let missing = Digest::of(b"an artifact never put");
        let index = from
            .put_artifact(&Manifest::Index(
                IndexMeta::new(
                    ComponentName::new("web"),
                    Version::new("1.3.0"),
                    vec![Artifact {
                        digest: missing.clone(),
                        meta: meta("site"),
                    }],
                )
                .unwrap(),
            ))
            .unwrap()
            .digest;
        from.name(&version(), &index, None).unwrap();
        let to = FakeStore::new();
        assert_eq!(
            copied(&from, &to),
            Err(Not::absent(&source("fake"), missing.to_string()))
        );
        assert_eq!(indexes(&to), 0);
        assert!(to.contents().names.is_empty());
    }

    #[test]
    fn a_name_held_elsewhere_in_the_target_is_refused_naming_it_once_all_else_is_held() {
        let (from, to) = (FakeStore::new(), FakeStore::new());
        let index = published(&from);
        let elsewhere = to.put_bytes(b"elsewhere").unwrap().digest;
        to.name(&version(), &elsewhere, None).unwrap();
        assert_eq!(
            copied(&from, &to),
            Err(Not::refused(
                &source("fake"),
                Why::NameAt {
                    at: Some(elsewhere.clone())
                },
                None
            ))
        );
        let target = to.contents();
        assert!(target.held.contains_key(&index));
        assert_eq!(target.attached.get(&index).map(Vec::len), Some(1));
        assert_eq!(target.names.get(&version()), Some(&elsewhere));
        let names = to.calls().iter().filter(|c| c.op == StoreOp::Name).count();
        assert_eq!(names, 2, "the name was moved more than once");
    }
}
