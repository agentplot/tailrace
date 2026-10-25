//! Spec acceptance: the gate's judgment of a version published in the
//! test, under the policy for acceptance at the branch's landed head, and
//! the channels that follow each publish.

mod common;
mod world;

use serde_json::json;
use tailrace_core::{answer_json, Followed, Judged, Moved, WhyWanting};
use tailrace_seams::{
    Answer, ChannelName, ClaimKind, ComponentName, Digest, Not, RequiredClaim, Revision, Seam,
    SignerId, Source, Version, Why,
};

// Requirement: The gate judges a version by its index's digest

#[test]
fn each_answer_renders_in_the_one_form_naming_the_index_and_both_revisions() {
    let (index, revision, head) = (
        Digest::of(b"index"),
        Revision::new("4f2a9c1"),
        Revision::new("7b3e0d2"),
    );
    let wanting = RequiredClaim {
        kind: ClaimKind::Verdict,
        check: Some(tailrace_seams::Check::new("test")),
        signers: vec![],
    };
    let promotion = Digest::of(b"promotion");
    let accepted: Answer<Judged> = Ok(Judged::Accepted {
        component: ComponentName::new("lib"),
        version: Version::new("0.2.0"),
        index: index.clone(),
        revision: revision.clone(),
        policy_at: head.clone(),
        acceptance: Digest::of(b"acceptance"),
        already: false,
        one_host: false,
        channels: vec![
            (
                ChannelName::new("beta"),
                Ok(Moved::Promoted {
                    from: None,
                    to: index.clone(),
                    promotion: promotion.clone(),
                    name: Followed::Moved { to: index.clone() },
                }),
            ),
            (
                ChannelName::new("edge"),
                Ok(Moved::Held {
                    artifact: Digest::of(b"artifact"),
                    wanting: wanting.clone(),
                    why: WhyWanting::Missing,
                }),
            ),
            (
                ChannelName::new("next"),
                Err(Not::refused(
                    &Source::new(Seam::Keeper, "bound"),
                    Why::NotHead {
                        head: Some(promotion.clone()),
                    },
                    None,
                )),
            ),
        ],
    });
    let json = answer_json(&accepted);
    assert_eq!(json["answer"], "ok");
    let value = &json["value"];
    assert_eq!(value["judged"], "accepted");
    assert_eq!(value["index"], index.as_str());
    assert_eq!(value["revision"], revision.as_str());
    assert_eq!(value["policy_at"], head.as_str());
    let channels = value["channels"].as_array().unwrap();
    assert_eq!(channels[0]["channel"], "beta");
    assert_eq!(channels[0]["answer"], "ok");
    assert_eq!(channels[0]["value"]["moved"], "promoted");
    assert_eq!(channels[0]["value"]["promotion"], promotion.as_str());
    assert_eq!(channels[1]["value"]["moved"], "held");
    assert_eq!(channels[1]["value"]["why"], "missing");
    assert_eq!(channels[2]["answer"], "refused");
    assert_eq!(
        channels[2]["refused"]["why"],
        json!({ "not_head": { "head": promotion.as_str() } })
    );

    let refused: Answer<Judged> = Ok(Judged::Refused {
        component: ComponentName::new("lib"),
        version: Version::new("0.2.0"),
        index: index.clone(),
        revision: revision.clone(),
        policy_at: head.clone(),
        artifact: Digest::of(b"artifact"),
        wanting,
        why: WhyWanting::Expired {
            signer: SignerId::new("landing"),
        },
    });
    let value = &answer_json(&refused)["value"];
    assert_eq!(value["judged"], "refused");
    assert_eq!(value["index"], index.as_str());
    assert_eq!(value["revision"], revision.as_str());
    assert_eq!(value["policy_at"], head.as_str());
    assert_eq!(value["wanting"]["kind"], "verdict");
    assert_eq!(value["why"], json!({ "expired": { "signer": "landing" } }));
}
