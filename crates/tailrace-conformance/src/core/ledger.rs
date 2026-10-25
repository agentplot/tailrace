//! The ledger's group: a race lost on a channel's name repaired from the
//! ledger (109; design D11), and below the marker bolt 4's unit of the
//! keeper and promotion adds promotion, the ledger and bound channels.

use serde_json::json;
use tailrace_core::{follow, Followed, Published};
use tailrace_seams::{
    ChannelName, Claim, ClaimKind, ComponentName, Digest, IndexMeta, Keeper, Manifest, Name,
    Signer, Store, Version,
};

use crate::core::publish::{one, passing};
use crate::world::{FakeWorld, Write, STABLE};
use crate::{ok, Scenario};

pub const GROUP: &str = "ledger";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
        Scenario {
            name: "a race lost on a channel's name",
            plays: None,
            cites: &[109],
            run: a_race_lost_on_a_channels_name,
        },
    ]
}

/// The head published in the world, its index; and an index of another
/// version, put beside it.
fn two_indexes(world: &FakeWorld) -> (Digest, Digest) {
    passing(world, &world.head);
    let published = ok(one(world.publish(&world.ask(&world.head))), "the publish");
    let Published::Version { index, .. } = published else {
        panic!("the head was not published: {published:?}")
    };
    let other = IndexMeta::new(
        ComponentName::new("repository"),
        Version::new("0.3.0"),
        vec![],
    )
    .expect("an index of no artifact");
    let other = ok(
        world.fakes.store.put_artifact(&Manifest::Index(other)),
        "the other index's put",
    )
    .digest;
    (index, other)
}

fn a_race_lost_on_a_channels_name(world: &FakeWorld) {
    let (index, other) = two_indexes(world);
    let channel = ChannelName::new(STABLE);
    let promotion = Claim {
        subject: index.clone(),
        kind: ClaimKind::Promotion,
        revision: world.head.clone(),
        build: None,
        body: json!({ "channel": STABLE }),
    };
    let promotion = ok(world.fakes.signer.sign(&promotion, &world.host), "a sign");
    ok(
        world.fakes.keeper.append(&channel, &promotion, None),
        "the promotion's append",
    );

    let name = Name::new(STABLE);
    let store: &dyn Store = &world.fakes.store;
    ok(store.name(&name, &other, None), "another writer's move");
    let written = world.writes().len();
    let appended = world.fakes.keeper.calls().len();

    let answer = ok(
        follow(store, &world.fakes.keeper, &channel, &name, None),
        "the move from the ledger",
    );
    assert_eq!(
        answer,
        Followed::LostRace {
            found: Some(other.clone()),
            to: index.clone()
        },
        "the lost race was not reported naming the other writer's index"
    );
    assert_eq!(
        ok(store.resolve(&name), "resolve"),
        index,
        "the name is not where the ledger's head says"
    );
    let moves: Vec<Write> = world.writes().split_off(written);
    assert_eq!(
        moves,
        [
            Write::Name {
                name: name.clone(),
                to: index.clone(),
                expected: None,
            },
            Write::Name {
                name,
                to: index,
                expected: Some(other),
            },
        ],
        "the store was not asked the caller's move and one repair"
    );
    assert_eq!(
        world.fakes.keeper.calls().len() - appended,
        1,
        "the ledger was asked more than its chain"
    );
}
