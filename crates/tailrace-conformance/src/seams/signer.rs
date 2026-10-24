//! The signer's group: 79 to 81, and 7 and 10 as a signer keeps them
//! (B.4).

use serde_json::json;
use tailrace_seams::{
    Claim, ClaimKind, Digest, Envelope, Made, NamedSigner, Not, Revision, Seam, Signer, SignerId,
    Why,
};

use crate::seams::{absent, unreadable};
use crate::{ok, Rig, Scenario};

pub const GROUP: &str = "signer";

/// A signer, made new for each scenario.
pub trait SignerRig: Rig {
    fn signer(&self) -> &dyn Signer;

    /// The identity at hand, of the party running tailrace.
    fn identity(&self) -> SignerId;

    /// `identity` as a policy names it, for every kind of claim.
    fn named(&self, identity: &SignerId) -> NamedSigner;

    /// A signer a policy names that is not the identity at hand.
    fn stranger(&self) -> NamedSigner;

    /// No identity at hand from now.
    fn no_identity(&self);

    /// `identity` expired from now.
    fn expire(&self, identity: &SignerId);

    /// The signer made unreachable, every operation answering unreadable,
    /// and the words it answers with.
    fn unreachable(&self) -> String;
}

pub fn scenarios() -> Vec<Scenario<dyn SignerRig>> {
    vec![
        Scenario {
            name: "signed and verified",
            plays: None,
            cites: &[7, 79, 80],
            run: signed_and_verified,
        },
        Scenario {
            name: "no identity at hand",
            plays: None,
            cites: &[79],
            run: no_identity_at_hand,
        },
        Scenario {
            name: "an envelope changed after signing",
            plays: None,
            cites: &[10, 80],
            run: an_envelope_changed_after_signing,
        },
        Scenario {
            name: "a signer the policy does not name",
            plays: None,
            cites: &[80],
            run: a_signer_the_policy_does_not_name,
        },
        Scenario {
            name: "an expired identity",
            plays: None,
            cites: &[81],
            run: an_expired_identity,
        },
        Scenario {
            name: "a signer that cannot be reached",
            plays: None,
            cites: &[58, 59],
            run: a_signer_that_cannot_be_reached,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}

/// A claim about one digest.
fn claim(says: &str) -> Claim {
    Claim {
        subject: Digest::of(b"an index"),
        kind: ClaimKind::Provenance,
        revision: Revision::new("4f2a9c1"),
        build: None,
        body: json!({ "says": says }),
    }
}

fn signed(rig: &dyn SignerRig) -> Envelope {
    ok(
        rig.signer().sign(&claim("published"), &rig.identity()),
        "sign in the identity at hand",
    )
}

fn no_named_signer(answer: tailrace_seams::Answer<Made>, checked: &[SignerId], asked: &str) {
    match answer {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::NoNamedSigner {
                checked: checked.to_vec()
            },
            "{asked} was refused for another reason"
        ),
        other => panic!("{asked} answered {other:?}, not that no named signer made it"),
    }
}

fn signed_and_verified(rig: &dyn SignerRig) {
    let identity = rig.identity();
    let envelope = signed(rig);
    assert_eq!(
        envelope.signature.signer, identity,
        "the envelope is signed by another"
    );
    let held = Claim::from_envelope(&envelope)
        .unwrap_or_else(|| panic!("the envelope holds no claim: {envelope:?}"));
    assert_eq!(
        held.subject,
        Digest::of(b"an index"),
        "the envelope is about another digest"
    );
    assert_eq!(held, claim("published"), "the envelope holds another claim");
    let made = ok(
        rig.signer().verify(&envelope, &[rig.named(&identity)]),
        "verify",
    );
    assert_eq!(
        made,
        Made {
            signer: identity,
            expired: false
        },
        "verify named another signer"
    );
}

fn no_identity_at_hand(rig: &dyn SignerRig) {
    let identity = rig.identity();
    rig.no_identity();
    absent(
        rig.signer().sign(&claim("published"), &identity),
        Seam::Signer,
        identity.as_str(),
        "sign with no identity at hand",
    );
}

fn an_envelope_changed_after_signing(rig: &dyn SignerRig) {
    let identity = rig.identity();
    let envelope = signed(rig);
    let changed = Envelope {
        payload: claim("published twice").payload(),
        ..envelope.clone()
    };
    let named = [rig.named(&identity)];
    no_named_signer(
        rig.signer().verify(&changed, &named),
        std::slice::from_ref(&identity),
        "verify of a changed envelope",
    );
    assert_eq!(
        ok(
            rig.signer().verify(&envelope, &named),
            "verify of the first"
        )
        .signer,
        identity,
        "the envelope as signed no longer verifies"
    );
}

fn a_signer_the_policy_does_not_name(rig: &dyn SignerRig) {
    let envelope = signed(rig);
    let stranger = rig.stranger();
    assert_ne!(
        stranger.id,
        rig.identity(),
        "the stranger is the identity at hand"
    );
    no_named_signer(
        rig.signer()
            .verify(&envelope, std::slice::from_ref(&stranger)),
        std::slice::from_ref(&stranger.id),
        "verify against a signer that did not sign",
    );
}

fn an_expired_identity(rig: &dyn SignerRig) {
    let identity = rig.identity();
    let envelope = signed(rig);
    rig.expire(&identity);
    let made = ok(
        rig.signer().verify(&envelope, &[rig.named(&identity)]),
        "verify after the identity expired",
    );
    assert_eq!(
        made,
        Made {
            signer: identity,
            expired: true
        },
        "an expired identity was not told as expired"
    );
}

fn a_signer_that_cannot_be_reached(rig: &dyn SignerRig) {
    let identity = rig.identity();
    let envelope = signed(rig);
    let words = rig.unreachable();
    let signer = rig.signer();
    unreadable(
        signer.verify(&envelope, &[rig.named(&identity)]),
        Seam::Signer,
        &words,
        "verify",
    );
    unreadable(
        signer.sign(&claim("published"), &identity),
        Seam::Signer,
        &words,
        "sign",
    );
}
