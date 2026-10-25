//! A channel's name moved in the store to the version the ledger's head
//! promotion is about, and a race lost on the name reported and repaired
//! from the ledger, never retried blind (39, 101, 106, 109, rule core.6;
//! design D9). The chain is read through the ledger seam and nothing is
//! written to it; the store name a channel takes is the caller's.

use serde::Serialize;
use tailrace_seams::{
    Answer, ChannelName, Claim, ClaimKind, Digest, Keeper, Name, Not, Seam, Source, Store, Why,
};

use crate::BOUND;

/// What following the ledger's head answers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "followed", rename_all = "snake_case")]
pub enum Followed {
    /// The name moved from where the caller expected it to `to`, the
    /// digest the head promotion is about.
    Moved { to: Digest },
    /// The name was at `at`, the digest the head promotion is about,
    /// already, and nothing was written.
    There { at: Digest },
    /// 109: another writer had moved the name to `found`, or away from
    /// every digest; the name was moved once from there to `to`, the
    /// digest the head promotion is about.
    LostRace { found: Option<Digest>, to: Digest },
}

/// The digest the head of `channel`'s chain is a promotion about (39):
/// absent for an empty chain and refused naming a fork, as the keeper
/// answers them (105, 121); unreadable naming the head when it holds no
/// promotion claim of tailrace's form.
fn head(keeper: &dyn Keeper, channel: &ChannelName) -> Answer<Digest> {
    let chain = keeper.chain(channel)?.claims;
    let Some(head) = chain.last() else {
        return Err(Not::absent(
            &Source::new(Seam::Keeper, BOUND),
            format!("chain of {channel}"),
        ));
    };
    match Claim::from_envelope(head) {
        Some(claim) if claim.kind == ClaimKind::Promotion => Ok(claim.subject),
        _ => Err(Not::unreadable(
            &Source::new(Seam::Keeper, BOUND),
            format!("the head of the chain of {channel}, {}", head.digest()),
            Some(format!(
                "{} holds no promotion claim of tailrace's form",
                head.digest()
            )),
        )),
    }
}

/// 109: `name`, the store name `channel` takes, moved from `expected` to
/// the digest the head of the channel's chain is about. A refusal naming
/// that digest is the name already there; one naming any other is a lost
/// race, answered with where the name was found after one move from
/// there to the head's digest, whose own refusal is the answer. The
/// caller's move is never retried, and nothing is written to the ledger.
pub fn follow(
    store: &dyn Store,
    keeper: &dyn Keeper,
    channel: &ChannelName,
    name: &Name,
    expected: Option<&Digest>,
) -> Answer<Followed> {
    let to = head(keeper, channel)?;
    let found = match store.name(name, &to, expected) {
        Ok(()) => return Ok(Followed::Moved { to }),
        Err(Not::Refused(r)) => match &r.why {
            Why::NameAt { at: Some(at) } if *at == to => return Ok(Followed::There { at: to }),
            Why::NameAt { at } => at.clone(),
            _ => return Err(Not::Refused(r)),
        },
        Err(not) => return Err(not),
    };
    store.name(name, &to, found.as_ref())?;
    Ok(Followed::LostRace { found, to })
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use serde_json::json;
    use tailrace_fakes::{FakeKeeper, FakeStore, StoreOp};
    use tailrace_seams::{Envelope, Held, Manifest, Put, Revision, Signature, SignerId, IN_TOTO};

    use super::*;

    fn stable() -> ChannelName {
        ChannelName::new("stable")
    }

    fn channel_name() -> Name {
        Name::new("stable")
    }

    fn signed(subject: &Digest, kind: ClaimKind) -> Envelope {
        let claim = Claim {
            subject: subject.clone(),
            kind,
            revision: Revision::new("4f2a9c1"),
            build: None,
            body: json!({ "channel": "stable" }),
        };
        Envelope::signed(
            IN_TOTO,
            claim.payload(),
            Signature {
                signer: SignerId::new("gate"),
                keyid: String::new(),
                sig: vec![9],
            },
        )
    }

    /// A keeper whose chain of `stable` promotes `first`, then `head`.
    fn keeper(first: &Digest, head: &Digest) -> FakeKeeper {
        let keeper = FakeKeeper::new();
        let one = signed(first, ClaimKind::Promotion);
        keeper.append(&stable(), &one, None).unwrap();
        keeper
            .append(
                &stable(),
                &signed(head, ClaimKind::Promotion),
                Some(&one.digest()),
            )
            .unwrap();
        keeper
    }

    fn names(store: &FakeStore) -> usize {
        store
            .calls()
            .iter()
            .filter(|c| c.op == StoreOp::Name)
            .count()
    }

    fn digests() -> (Digest, Digest, Digest) {
        (
            Digest::of(b"the version before"),
            Digest::of(b"the head's version"),
            Digest::of(b"another writer's version"),
        )
    }

    #[test]
    fn the_name_moves_to_the_heads_digest() {
        let (before, head, _) = digests();
        let (store, keeper) = (FakeStore::new(), keeper(&before, &head));
        store.name(&channel_name(), &before, None).unwrap();
        assert_eq!(
            follow(&store, &keeper, &stable(), &channel_name(), Some(&before)),
            Ok(Followed::Moved { to: head.clone() })
        );
        assert_eq!(store.resolve(&channel_name()), Ok(head));
        assert_eq!(names(&store), 2);
    }

    #[test]
    fn an_empty_chain_is_absent_and_a_fork_refused_and_neither_writes_the_store() {
        let (before, head, other) = digests();
        let store = FakeStore::new();
        let empty = FakeKeeper::new();
        assert!(matches!(
            follow(&store, &empty, &stable(), &channel_name(), None),
            Err(Not::Absent(a)) if a.what == "chain of stable"
        ));
        let forked = FakeKeeper::new();
        let root = signed(&before, ClaimKind::Promotion);
        forked.append(&stable(), &root, None).unwrap();
        let (a, b) = (
            signed(&head, ClaimKind::Promotion),
            signed(&other, ClaimKind::Promotion),
        );
        forked.fork(&stable(), &a, &b);
        match follow(&store, &forked, &stable(), &channel_name(), Some(&before)) {
            Err(Not::Refused(r)) => assert_eq!(
                r.why,
                Why::Fork {
                    predecessor: Some(root.digest()),
                    claims: vec![a.digest(), b.digest()],
                }
            ),
            other => panic!("a forked chain answered {other:?}"),
        }
        assert_eq!(store.calls().len(), 0, "the store was asked something");
    }

    #[test]
    fn a_head_that_is_no_promotion_is_unreadable_naming_it() {
        let (_, head, _) = digests();
        let (store, keeper) = (FakeStore::new(), FakeKeeper::new());
        let acceptance = signed(&head, ClaimKind::Acceptance);
        keeper.append(&stable(), &acceptance, None).unwrap();
        match follow(&store, &keeper, &stable(), &channel_name(), None) {
            Err(Not::Unreadable(u)) => {
                assert!(u.what.contains(acceptance.digest().as_str()), "{u:?}")
            }
            other => panic!("answered {other:?}"),
        }
        assert_eq!(names(&store), 0);
    }

    #[test]
    fn a_name_already_at_the_heads_digest_is_answered_there() {
        let (before, head, _) = digests();
        let (store, keeper) = (FakeStore::new(), keeper(&before, &head));
        store.name(&channel_name(), &head, None).unwrap();
        assert_eq!(
            follow(&store, &keeper, &stable(), &channel_name(), Some(&before)),
            Ok(Followed::There { at: head.clone() })
        );
        assert_eq!(names(&store), 2, "more than the caller's move was asked");
        assert_eq!(store.resolve(&channel_name()), Ok(head));
    }

    #[test]
    fn another_writers_move_is_a_lost_race_repaired_with_two_moves_in_all() {
        let (before, head, other) = digests();
        let (store, keeper) = (FakeStore::new(), keeper(&before, &head));
        store.name(&channel_name(), &other, None).unwrap();
        let asked = names(&store);
        assert_eq!(
            follow(&store, &keeper, &stable(), &channel_name(), Some(&before)),
            Ok(Followed::LostRace {
                found: Some(other),
                to: head.clone()
            })
        );
        assert_eq!(names(&store) - asked, 2);
        assert_eq!(store.resolve(&channel_name()), Ok(head));
        assert_eq!(
            keeper.chain(&stable()).unwrap().claims.len(),
            2,
            "the chain changed"
        );
    }

    /// A store a second writer moves the name in once, to `then`, right
    /// after the first move it is asked.
    struct Racing {
        inner: FakeStore,
        then: Digest,
        asked: Cell<usize>,
    }

    impl Store for Racing {
        fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
            self.inner.put_bytes(bytes)
        }
        fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
            self.inner.put_artifact(manifest)
        }
        fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
            self.inner.attach(subject, attestation)
        }
        fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>> {
            self.inner.claims_of(digest)
        }
        fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
            self.asked.set(self.asked.get() + 1);
            let answer = self.inner.name(name, to, expected);
            if self.asked.get() == 1 {
                let at = self.inner.resolve(name).ok();
                self.inner.name(name, &self.then, at.as_ref()).unwrap();
            }
            answer
        }
        fn resolve(&self, name: &Name) -> Answer<Digest> {
            self.inner.resolve(name)
        }
        fn read(&self, digest: &Digest) -> Answer<Held> {
            self.inner.read(digest)
        }
    }

    #[test]
    fn a_repair_refused_in_turn_is_the_answer_and_nothing_is_retried() {
        let (before, head, other) = digests();
        let again = Digest::of(b"a third writer's version");
        let store = Racing {
            inner: FakeStore::new(),
            then: again.clone(),
            asked: Cell::new(0),
        };
        store.inner.name(&channel_name(), &other, None).unwrap();
        let keeper = keeper(&before, &head);
        match follow(&store, &keeper, &stable(), &channel_name(), Some(&before)) {
            Err(Not::Refused(r)) => assert_eq!(
                r.why,
                Why::NameAt {
                    at: Some(again.clone())
                }
            ),
            other => panic!("a repair refused answered {other:?}"),
        }
        assert_eq!(store.asked.get(), 2, "a move was retried");
        assert_eq!(store.resolve(&channel_name()), Ok(again));
    }
}
