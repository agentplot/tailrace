//! The store-naming group: what a publish names in the store is found by
//! a consumer by ecosystem, name and version alone, on the index the
//! publish answered, with the claims a layout reads beside it (138).

use serde_json::Value;
use timber_core::{PolicyOf, Published, Refresh};
use timber_seams::{AskedOf, Audience, ClaimKind, NamedSigner, Required, RequiredClaim};

use crate::core::publish::{one, passing, TAG, VERSION};
use crate::world::{FakeWorld, FILE};
use crate::{ok, Scenario};

pub const GROUP: &str = "naming";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
        Scenario {
            name: "a pin found through what a publish wrote",
            plays: None,
            cites: &[138],
            run: a_pin_found_through_what_a_publish_wrote,
        },
    ]
}

/// The host's provenance, which the publisher's policy for layouts asks
/// of every artifact.
fn provenance(world: &FakeWorld) -> RequiredClaim {
    RequiredClaim {
        kind: ClaimKind::Provenance,
        check: None,
        environment: None,
        signers: vec![NamedSigner {
            id: world.host.clone(),
            verified_by: "the host's key".into(),
            kinds: vec![ClaimKind::Provenance],
            checks: vec![],
        }],
        asked_of: AskedOf::Every,
    }
}

fn a_pin_found_through_what_a_publish_wrote(world: &FakeWorld) {
    passing(world, &world.head);
    world.fakes.policy.require_at(
        &world.head,
        Audience::Layout,
        Required::Layout {
            claims: vec![provenance(world)],
            given: false,
            signers: vec![],
        },
    );
    let published = ok(one(world.publish(&world.ask(&world.head))), "the publish");
    let Published::Version {
        index, artifacts, ..
    } = &published
    else {
        panic!("the head was not published: {published:?}")
    };
    let site = artifacts
        .iter()
        .find(|a| a.ecosystem.as_str() == FILE)
        .expect("the site's artifact");

    // The consumer, a later revision whose file writes no layout line,
    // pins the site by its name and version alone.
    let consumer = world.land_policy("outputs = [\"site\", \"server\"]\n");
    world.lockfile(FILE, &[(site.name.as_str(), VERSION)]);
    let trusted = [NamedSigner {
        id: world.host.clone(),
        verified_by: "the host's key".into(),
        kinds: vec![],
        checks: vec![],
    }];
    let synced = ok(
        world.sync_checkout(&consumer, 1, &trusted),
        "the consumer's sync",
    );
    let layout = match synced.layouts.as_slice() {
        [(ecosystem, Ok(Refresh::Refreshed(layout)))] if ecosystem.as_str() == FILE => layout,
        other => panic!("the sync answered the layouts {other:?}, not the site's rendered"),
    };
    let [pin] = layout.pins.as_slice() else {
        panic!("the layout holds {:?}, not the one pin", layout.pins)
    };
    let held = pin
        .answer
        .as_ref()
        .unwrap_or_else(|not| panic!("the pin answered {not:?}"));
    assert_eq!(
        (&held.index, &held.artifact),
        (index, &site.digest),
        "the pin was held from another index than the publish answered"
    );
    assert_eq!(
        pin.policy,
        Some(PolicyOf::Publisher {
            revision: world.head.clone()
        }),
        "the pin was not judged under the layout policy the publish put on its index"
    );

    // Nothing the consumer asked or was answered names the publisher's
    // component, its tag or a repository (49, rule core.18).
    let answered: Value = serde_json::to_value(&synced).expect("an answer a program reads");
    let text = answered.to_string();
    for named in [TAG, "\"repository\""] {
        assert!(
            !text.contains(named),
            "the consumer's answer names {named}: {text}"
        );
    }
}
