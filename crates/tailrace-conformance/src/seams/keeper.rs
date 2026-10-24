//! The keeper's group: 120 to 122, and 34, 35 and 105 as a keeper keeps
//! them (B.8).

use tailrace_seams::{ChannelName, Envelope, Keeper, Not, Seam, Why};

use crate::seams::{absent, ok, unreadable};
use crate::{Rig, Scenario};

pub const GROUP: &str = "keeper";

/// A keeper, made new for each scenario.
pub trait KeeperRig: Rig {
    fn keeper(&self) -> &dyn Keeper;

    /// The `n`th promotion claim of `channel`, signed, each `n` another
    /// claim.
    fn promotion(&self, channel: &ChannelName, n: u32) -> Envelope;

    /// Two promotions held in `channel`'s chain naming one predecessor, a
    /// fork, as the keeper came to hold them.
    fn fork(&self, channel: &ChannelName) -> [Envelope; 2];

    /// The keeper made unreachable, every operation answering unreadable,
    /// and the words it answers with.
    fn unreachable(&self) -> String;
}

pub fn scenarios() -> Vec<Scenario<dyn KeeperRig>> {
    vec![
        Scenario {
            name: "a chain appended in order",
            plays: None,
            cites: &[34, 120, 121, 122],
            run: a_chain_appended_in_order,
        },
        Scenario {
            name: "a stale predecessor",
            plays: None,
            cites: &[35, 105, 120],
            run: a_stale_predecessor,
        },
        Scenario {
            name: "a fork",
            plays: None,
            cites: &[105, 122],
            run: a_fork,
        },
        Scenario {
            name: "a channel with no promotion",
            plays: None,
            cites: &[31, 121],
            run: a_channel_with_no_promotion,
        },
        Scenario {
            name: "a keeper that cannot be reached",
            plays: None,
            cites: &[58, 59],
            run: a_keeper_that_cannot_be_reached,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}

fn stable() -> ChannelName {
    ChannelName::new("stable")
}

/// Two promotions of `channel`, the second following the first, appended.
fn two(rig: &dyn KeeperRig, channel: &ChannelName) -> [Envelope; 2] {
    let keeper = rig.keeper();
    let (first, second) = (rig.promotion(channel, 1), rig.promotion(channel, 2));
    let held = ok(keeper.append(channel, &first, None), "the first append");
    assert_eq!(
        held,
        first.digest(),
        "the first append answered another digest"
    );
    let held = ok(
        keeper.append(channel, &second, Some(&first.digest())),
        "the second append",
    );
    assert_eq!(
        held,
        second.digest(),
        "the second append answered another digest"
    );
    [first, second]
}

fn a_chain_appended_in_order(rig: &dyn KeeperRig) {
    let channel = stable();
    let appended = two(rig, &channel);
    let keeper = rig.keeper();
    let first = ok(keeper.chain(&channel), "the first reader's chain");
    let second = ok(keeper.chain(&channel), "the second reader's chain");
    assert_eq!(first, appended, "the chain is not the promotions in order");
    assert_eq!(first, second, "two readers were answered two chains");
}

fn not_head(answer: tailrace_seams::Answer<tailrace_seams::Digest>, head: &Envelope, asked: &str) {
    match answer {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::NotHead {
                head: Some(head.digest())
            },
            "{asked} was refused naming another head"
        ),
        other => panic!("{asked} answered {other:?}, not refused"),
    }
}

fn a_stale_predecessor(rig: &dyn KeeperRig) {
    let channel = stable();
    let [first, second] = two(rig, &channel);
    let keeper = rig.keeper();
    let third = rig.promotion(&channel, 3);
    not_head(
        keeper.append(&channel, &third, Some(&first.digest())),
        &second,
        "an append naming a claim that is not the head",
    );
    not_head(
        keeper.append(&channel, &third, None),
        &second,
        "an append naming no predecessor on a chain that has one",
    );
    assert_eq!(
        ok(keeper.chain(&channel), "chain"),
        [first, second],
        "a refused append changed the chain"
    );
}

fn a_fork(rig: &dyn KeeperRig) {
    let channel = stable();
    let [a, b] = rig.fork(&channel);
    for reader in ["the first reader", "the second reader"] {
        match rig.keeper().chain(&channel) {
            Err(Not::Refused(r)) => match r.why {
                Why::Fork { claims, .. } => {
                    for claim in [&a, &b] {
                        assert!(
                            claims.contains(&claim.digest()),
                            "the fork told {reader} names {claims:?}, without {}",
                            claim.digest()
                        );
                    }
                }
                why => panic!("the chain of a fork was refused to {reader} for {why:?}"),
            },
            other => panic!("the chain of a fork answered {reader} {other:?}"),
        }
    }
}

fn a_channel_with_no_promotion(rig: &dyn KeeperRig) {
    absent(
        rig.keeper().chain(&ChannelName::new("never-promoted")),
        Seam::Keeper,
        "never-promoted",
        "the chain of a channel never appended to",
    );
}

fn a_keeper_that_cannot_be_reached(rig: &dyn KeeperRig) {
    let channel = stable();
    let promotion = rig.promotion(&channel, 1);
    let words = rig.unreachable();
    let keeper = rig.keeper();
    unreadable(keeper.chain(&channel), Seam::Keeper, &words, "chain");
    unreadable(
        keeper.append(&channel, &promotion, None),
        Seam::Keeper,
        &words,
        "append",
    );
}
