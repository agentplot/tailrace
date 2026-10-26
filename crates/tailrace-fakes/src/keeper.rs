//! The fake keeper (B.8): per channel and its component (125), a chain
//! of promotion claims, each
//! held with the predecessor it named, and the chain as it last answered
//! it, which it answers as last read once made unreachable (C.13, C.14).

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;
use tailrace_seams::{Answer, Chain, ChannelId, Digest, Envelope, Keeper, Not, Read, Seam, Why};

use crate::Fake;

/// The fake keeper's operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeeperOp {
    Append,
    Chain,
}

/// A promotion claim held, and the predecessor it named.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Link {
    promotion: Envelope,
    predecessor: Option<Digest>,
}

/// Each channel's chain, in the order held; per channel, the claims it
/// last answered and when, in seconds since the Unix epoch; and the words
/// it answers with once unreachable.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeeperState {
    chains: BTreeMap<ChannelId, Vec<Link>>,
    answered: BTreeMap<ChannelId, (Vec<Envelope>, u64)>,
    unreachable: Option<String>,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// A keeper in memory.
pub type FakeKeeper = Fake<KeeperState, KeeperOp>;

impl Default for FakeKeeper {
    fn default() -> Self {
        Self::new()
    }
}

/// 105: the claims of a chain naming one predecessor, where two do.
fn fork(chain: &[Link]) -> Option<Why> {
    chain.iter().enumerate().find_map(|(i, link)| {
        let claims: Vec<Digest> = chain[i..]
            .iter()
            .filter(|l| l.predecessor == link.predecessor)
            .map(|l| l.promotion.digest())
            .collect();
        (claims.len() > 1).then(|| Why::Fork {
            predecessor: link.predecessor.clone(),
            claims,
        })
    })
}

impl FakeKeeper {
    pub fn new() -> Self {
        Fake::with(Seam::Keeper, KeeperState::default())
    }

    /// 105: `a` and `b` both held in `channel`'s chain naming its head as
    /// their predecessor, a fork that chain then refuses.
    pub fn fork(&self, channel: &ChannelId, a: &Envelope, b: &Envelope) {
        let mut state = self.state();
        let chain = state.chains.entry(channel.clone()).or_default();
        let predecessor = chain.last().map(|l| l.promotion.digest());
        for promotion in [a, b] {
            chain.push(Link {
                promotion: promotion.clone(),
                predecessor: predecessor.clone(),
            });
        }
    }

    /// The source holding the ledger no longer answers, for `words`, its
    /// own: a chain answered before is answered as then, [`Read::Kept`]
    /// with the time of that answer and the words; one never answered,
    /// and every append, unreadable with them (60, 61).
    pub fn unreachable(&self, words: impl Into<String>) {
        self.state().unreachable = Some(words.into());
    }
}

impl Keeper for FakeKeeper {
    fn append(
        &self,
        channel: &ChannelId,
        promotion: &Envelope,
        predecessor: Option<&Digest>,
    ) -> Answer<Digest> {
        let given =
            json!({ "channel": channel, "promotion": promotion, "predecessor": predecessor });
        self.ask(
            KeeperOp::Append,
            given,
            format!("the chain of {channel}"),
            |s, source| {
                if let Some(words) = &s.unreachable {
                    let what = format!("the chain of {channel}");
                    return Err(Not::unreadable(source, what, Some(words.clone())));
                }
                let chain = s.chains.entry(channel.clone()).or_default();
                if let Some(why) = fork(chain) {
                    return Err(Not::refused(source, why, None));
                }
                let head = chain.last().map(|l| l.promotion.digest());
                if head.as_ref() != predecessor {
                    return Err(Not::refused(source, Why::NotHead { head }, None));
                }
                chain.push(Link {
                    promotion: promotion.clone(),
                    predecessor: predecessor.cloned(),
                });
                Ok(promotion.digest())
            },
        )
    }

    fn chain(&self, channel: &ChannelId) -> Answer<Chain> {
        let given = json!({ "channel": channel });
        self.ask(
            KeeperOp::Chain,
            given,
            format!("the chain of {channel}"),
            |s, source| {
                if let Some(words) = &s.unreachable {
                    return match s.answered.get(channel) {
                        Some((claims, at)) => Ok(Chain {
                            claims: claims.clone(),
                            read: Read::Kept {
                                at: *at,
                                said: words.clone(),
                            },
                        }),
                        None => Err(Not::unreadable(
                            source,
                            format!("the chain of {channel}"),
                            Some(words.clone()),
                        )),
                    };
                }
                let chain = match s.chains.get(channel) {
                    Some(chain) if !chain.is_empty() => chain,
                    _ => return Err(Not::absent(source, format!("chain of {channel}"))),
                };
                if let Some(why) = fork(chain) {
                    return Err(Not::refused(source, why, None));
                }
                let claims: Vec<Envelope> = chain.iter().map(|l| l.promotion.clone()).collect();
                s.answered.insert(channel.clone(), (claims.clone(), now()));
                Ok(Chain {
                    claims,
                    read: Read::Source,
                })
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use tailrace_seams::{ChannelName, ComponentName, Signature, SignerId, Source, IN_TOTO};

    use super::*;
    use crate::{Fault, Times};

    fn promotion(to: &str) -> Envelope {
        Envelope::signed(
            IN_TOTO,
            to.as_bytes().to_vec(),
            Signature {
                signer: SignerId::new("gate"),
                keyid: String::new(),
                sig: vec![5],
            },
        )
    }

    fn source() -> Source {
        Source::new(Seam::Keeper, "fake")
    }

    /// A channel of `component`, or of none where it names none.
    fn id(component: Option<&str>, name: &str) -> ChannelId {
        ChannelId::new(component.map(ComponentName::new), ChannelName::new(name))
    }

    #[test]
    fn a_chain_is_held_in_order_from_the_first() {
        let keeper = FakeKeeper::new();
        let stable = id(None, "stable");
        assert_eq!(
            keeper.chain(&stable),
            Err(Not::absent(&source(), "chain of stable"))
        );
        let (one, two) = (promotion("1.0.0"), promotion("1.1.0"));
        assert_eq!(keeper.append(&stable, &one, None), Ok(one.digest()));
        assert_eq!(
            keeper.append(&stable, &two, Some(&one.digest())),
            Ok(two.digest())
        );
        assert_eq!(keeper.chain(&stable).map(|c| c.claims), Ok(vec![one, two]));
    }

    #[test]
    fn a_stale_predecessor_is_refused_naming_the_head() {
        let keeper = FakeKeeper::new();
        let stable = id(None, "stable");
        let (root, a, b) = (promotion("1.0.0"), promotion("1.1.0"), promotion("1.2.0"));
        keeper.append(&stable, &root, None).unwrap();
        assert_eq!(
            keeper.append(&stable, &a, Some(&root.digest())),
            Ok(a.digest())
        );
        assert_eq!(
            keeper.append(&stable, &b, Some(&root.digest())),
            Err(Not::refused(
                &source(),
                Why::NotHead {
                    head: Some(a.digest())
                },
                None
            ))
        );
        assert_eq!(
            keeper.chain(&stable).map(|c| c.claims),
            Ok(vec![root.clone(), a.clone()]),
            "the chain is unchanged"
        );
        assert_eq!(
            keeper.append(&stable, &b, None),
            Err(Not::refused(
                &source(),
                Why::NotHead {
                    head: Some(a.digest())
                },
                None
            ))
        );
        assert_eq!(
            keeper.append(&id(None, "beta"), &b, Some(&root.digest())),
            Err(Not::refused(&source(), Why::NotHead { head: None }, None))
        );
    }

    #[test]
    fn a_fork_is_refused_naming_both() {
        let keeper = FakeKeeper::new();
        let stable = id(None, "stable");
        let (root, a, b) = (promotion("1.0.0"), promotion("1.1.0"), promotion("1.2.0"));
        keeper.append(&stable, &root, None).unwrap();
        keeper.fork(&stable, &a, &b);
        let forked = Err(Not::refused(
            &source(),
            Why::Fork {
                predecessor: Some(root.digest()),
                claims: vec![a.digest(), b.digest()],
            },
            None,
        ));
        assert_eq!(keeper.chain(&stable), forked);
        assert_eq!(
            keeper.append(&stable, &promotion("1.3.0"), Some(&b.digest())),
            forked.map(|_: Chain| unreachable!())
        );
    }

    #[test]
    fn every_operation_can_be_faulted_and_is_recorded() {
        let keeper = FakeKeeper::new();
        let stable = id(None, "stable");
        keeper.fault(
            KeeperOp::Append,
            Fault::Refused("ref locked".into()),
            Times::Once,
        );
        keeper.fault(
            KeeperOp::Chain,
            Fault::Unreadable("ref unreadable".into()),
            Times::Once,
        );
        assert!(matches!(
            keeper.append(&stable, &promotion("1.0.0"), None),
            Err(Not::Refused(r)) if r.said.as_deref() == Some("ref locked")
        ));
        assert!(matches!(keeper.chain(&stable), Err(Not::Unreadable(_))));
        assert_eq!(
            keeper.chain(&stable),
            Err(Not::absent(&source(), "chain of stable"))
        );
        assert_eq!(
            keeper.calls().iter().map(|c| c.op).collect::<Vec<_>>(),
            [KeeperOp::Append, KeeperOp::Chain, KeeperOp::Chain]
        );
    }

    #[test]
    fn unreachable_after_a_read_answers_the_chain_as_last_read() {
        let keeper = FakeKeeper::new();
        let (stable, beta) = (id(None, "stable"), id(None, "beta"));
        let (one, two) = (promotion("1.0.0"), promotion("1.1.0"));
        keeper.append(&stable, &one, None).unwrap();
        keeper.append(&beta, &two, None).unwrap();
        let before = now();
        let read = keeper.chain(&stable).unwrap();
        let after = now();
        assert_eq!(
            read,
            Chain {
                claims: vec![one.clone()],
                read: Read::Source
            }
        );

        keeper.unreachable("ref unreadable");
        let Ok(Chain {
            claims,
            read: Read::Kept { at, said },
        }) = keeper.chain(&stable)
        else {
            panic!("an unreachable keeper answered no kept chain")
        };
        assert_eq!(claims, std::slice::from_ref(&one));
        assert!(
            (before..=after).contains(&at),
            "{at} is not the first answer's time"
        );
        assert_eq!(said, "ref unreadable");

        let unreadable =
            |what: &str| Not::unreadable(&source(), what, Some("ref unreadable".into()));
        assert_eq!(keeper.chain(&beta), Err(unreadable("the chain of beta")));
        assert_eq!(
            keeper.append(&stable, &two, Some(&one.digest())),
            Err(unreadable("the chain of stable"))
        );
        assert_eq!(
            keeper.calls().iter().map(|c| c.op).collect::<Vec<_>>(),
            [
                KeeperOp::Append,
                KeeperOp::Append,
                KeeperOp::Chain,
                KeeperOp::Chain,
                KeeperOp::Chain,
                KeeperOp::Append
            ]
        );
    }

    #[test]
    fn two_components_channels_of_one_name_are_two_chains() {
        let keeper = FakeKeeper::new();
        let libs = id(Some("lib"), "beta");
        let one = promotion("lib 1.0.0");
        assert_eq!(keeper.append(&libs, &one, None), Ok(one.digest()));
        assert_eq!(keeper.chain(&libs).map(|c| c.claims), Ok(vec![one]));
        assert_eq!(
            keeper.chain(&id(Some("web"), "beta")),
            Err(Not::absent(&source(), "chain of web/beta"))
        );
        assert_eq!(
            keeper.chain(&id(None, "beta")),
            Err(Not::absent(&source(), "chain of beta"))
        );
    }
}
