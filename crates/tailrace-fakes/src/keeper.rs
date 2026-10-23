//! The fake keeper (B.8): per channel, a chain of promotion claims, each
//! held with the predecessor it named.

use std::collections::BTreeMap;

use serde_json::json;
use tailrace_seams::{Answer, ChannelName, Digest, Envelope, Keeper, Not, Seam, Why};

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

/// Each channel's chain, in the order held.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeeperState {
    chains: BTreeMap<ChannelName, Vec<Link>>,
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
    pub fn fork(&self, channel: &ChannelName, a: &Envelope, b: &Envelope) {
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
}

impl Keeper for FakeKeeper {
    fn append(
        &self,
        channel: &ChannelName,
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

    fn chain(&self, channel: &ChannelName) -> Answer<Vec<Envelope>> {
        let given = json!({ "channel": channel });
        self.ask(
            KeeperOp::Chain,
            given,
            format!("the chain of {channel}"),
            |s, source| {
                let chain = match s.chains.get(channel) {
                    Some(chain) if !chain.is_empty() => chain,
                    _ => return Err(Not::absent(source, format!("chain of {channel}"))),
                };
                if let Some(why) = fork(chain) {
                    return Err(Not::refused(source, why, None));
                }
                Ok(chain.iter().map(|l| l.promotion.clone()).collect())
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use tailrace_seams::{Signature, SignerId, Source, IN_TOTO};

    use super::*;
    use crate::{Fault, Times};

    fn promotion(to: &str) -> Envelope {
        Envelope {
            payload_type: IN_TOTO.into(),
            payload: to.as_bytes().to_vec(),
            signature: Signature {
                signer: SignerId::new("gate"),
                sig: vec![5],
            },
        }
    }

    fn source() -> Source {
        Source::new(Seam::Keeper, "fake")
    }

    #[test]
    fn a_chain_is_held_in_order_from_the_first() {
        let keeper = FakeKeeper::new();
        let stable = ChannelName::new("stable");
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
        assert_eq!(keeper.chain(&stable), Ok(vec![one, two]));
    }

    #[test]
    fn a_stale_predecessor_is_refused_naming_the_head() {
        let keeper = FakeKeeper::new();
        let stable = ChannelName::new("stable");
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
            keeper.chain(&stable),
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
            keeper.append(&ChannelName::new("beta"), &b, Some(&root.digest())),
            Err(Not::refused(&source(), Why::NotHead { head: None }, None))
        );
    }

    #[test]
    fn a_fork_is_refused_naming_both() {
        let keeper = FakeKeeper::new();
        let stable = ChannelName::new("stable");
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
            forked.map(|_: Vec<Envelope>| unreachable!())
        );
    }

    #[test]
    fn every_operation_can_be_faulted_and_is_recorded() {
        let keeper = FakeKeeper::new();
        let stable = ChannelName::new("stable");
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
}
