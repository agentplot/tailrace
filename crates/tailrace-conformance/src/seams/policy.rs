//! The policy's group: 82, 83 and 131, and 116 and 125 as a policy keeps
//! them (B.5).

use tailrace_seams::{
    AskedOf, Audience, ChannelId, ChannelName, Check, ClaimKind, ComponentName, DeclaredChannel,
    HeldChannel, Moves, NamedSigner, Policy, Required, RequiredClaim, Revision, Seam, SignerId,
};

use crate::seams::{absent, unreadable};
use crate::{ok, Rig, Scenario};

pub const GROUP: &str = "policy";

/// A policy, made new for each scenario.
pub trait PolicyRig: Rig {
    fn policy(&self) -> &dyn Policy;

    /// A landed revision at which the policy declares what each audience
    /// of `policies` requires, and nothing for any other audience but a
    /// channel one of them is bound to, which a profile that binds a
    /// channel only to one it declares (37) declares requiring no claim.
    fn at(&self, policies: &[(Audience, Required)]) -> Revision;

    /// A landed revision at which the repository declares `components`
    /// (none: the one component of its own) and each of `channels`, in the
    /// component it names, moving as it says and requiring no claim.
    fn declaring(&self, components: &[ComponentName], channels: &[DeclaredChannel]) -> Revision;

    /// 131: the channels the profile lists for a repository declaring
    /// `components` with no channel declared, which it lists beside the
    /// declared like any of them; none where it gives none.
    fn given(&self, components: &[ComponentName]) -> Vec<DeclaredChannel>;

    /// C.10: the givens the profile holds off for a repository declaring
    /// `components` with no channel declared, each with how it would move
    /// and the line that switches it on, in the order the channels answer
    /// carries them; none where it holds none.
    fn held(&self, components: &[ComponentName]) -> Vec<HeldChannel>;

    /// The policy made unreachable, every operation answering unreadable,
    /// and the words it answers with.
    fn unreachable(&self) -> String;
}

pub fn scenarios() -> Vec<Scenario<dyn PolicyRig>> {
    vec![
        Scenario {
            name: "the claims an audience requires",
            plays: None,
            cites: &[82, 116],
            run: the_claims_an_audience_requires,
        },
        Scenario {
            name: "acceptance names no channel",
            plays: None,
            cites: &[83],
            run: acceptance_names_no_channel,
        },
        Scenario {
            name: "no policy for an audience",
            plays: None,
            cites: &[82],
            run: no_policy_for_an_audience,
        },
        Scenario {
            name: "a policy that cannot be reached",
            plays: None,
            cites: &[58, 59],
            run: a_policy_that_cannot_be_reached,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
        Scenario {
            name: "the channels a policy declares",
            plays: None,
            cites: &[131, 125, 37, 83],
            run: the_channels_a_policy_declares,
        },
        Scenario {
            name: "each way a channel moves",
            plays: None,
            cites: &[131, 37, 83, 95],
            run: each_way_a_channel_moves,
        },
        Scenario {
            name: "one component has every channel",
            plays: None,
            cites: &[131, 125, 123],
            run: one_component_has_every_channel,
        },
        Scenario {
            name: "a policy declaring no channel",
            plays: None,
            cites: &[131],
            run: a_policy_declaring_no_channel,
        },
        Scenario {
            name: "the channels of a policy that cannot be reached",
            plays: None,
            cites: &[131, 58, 59],
            run: the_channels_of_a_policy_that_cannot_be_reached,
        },
        Scenario {
            name: "the givens held off",
            plays: None,
            cites: &[131],
            run: the_givens_held_off,
        },
    ]
}

/// `id` named for `kinds`, and for a verdict the `checks`, by the names
/// the landing gives them (116).
fn named(id: &str, kinds: &[ClaimKind], checks: &[&str]) -> NamedSigner {
    NamedSigner {
        id: SignerId::new(id),
        verified_by: format!("{id}'s key"),
        kinds: kinds.to_vec(),
        checks: checks.iter().map(|c| Check::new(*c)).collect(),
    }
}

/// A provenance from the host of every artifact, and a verdict of `test`
/// from the landing of built artifacts alone (134).
fn claims() -> Vec<RequiredClaim> {
    vec![
        RequiredClaim {
            kind: ClaimKind::Provenance,
            check: None,
            signers: vec![named("host", &[ClaimKind::Provenance], &[])],
            asked_of: AskedOf::Every,
        },
        RequiredClaim {
            kind: ClaimKind::Verdict,
            check: Some(Check::new("test")),
            signers: vec![named("landing", &[ClaimKind::Verdict], &["test"])],
            asked_of: AskedOf::Built,
        },
    ]
}

fn the_claims_an_audience_requires(rig: &dyn PolicyRig) {
    let required = Required::Acceptance { claims: claims() };
    let at = rig.at(&[(Audience::Acceptance, required.clone())]);
    let read = ok(
        rig.policy().policy_for(&at, &Audience::Acceptance),
        "policy-for acceptance",
    );
    assert_eq!(
        read, required,
        "the policy answered other claims, signers or reaches than it declares"
    );
    let verdict = &read.claims()[1];
    assert_eq!(
        (
            verdict.signers[0].kinds.as_slice(),
            verdict.signers[0].checks.as_slice()
        ),
        (
            [ClaimKind::Verdict].as_slice(),
            [Check::new("test")].as_slice()
        ),
        "the verdict's signer is bound to other kinds or checks"
    );
}

fn acceptance_names_no_channel(rig: &dyn PolicyRig) {
    let (next, edge) = (
        ChannelId::new(None, ChannelName::new("next")),
        ChannelName::new("edge"),
    );
    let at = rig.at(&[
        (
            Audience::Acceptance,
            Required::Acceptance { claims: claims() },
        ),
        (
            Audience::Channel(next.clone()),
            Required::Channel {
                claims: claims(),
                bound_to: Some(edge.clone()),
                promoted_by: vec![],
                approvers: vec![],
            },
        ),
    ]);
    let policy = rig.policy();
    let acceptance = ok(
        policy.policy_for(&at, &Audience::Acceptance),
        "policy-for acceptance",
    );
    assert!(
        matches!(acceptance, Required::Acceptance { .. }),
        "the policy for acceptance answered {acceptance:?}"
    );
    assert_eq!(acceptance.bound_to(), None, "acceptance names a channel");
    let bound = ok(
        policy.policy_for(&at, &Audience::Channel(next)),
        "policy-for a bound channel",
    );
    assert_eq!(
        bound.bound_to(),
        Some(&edge),
        "the bound channel's policy answered {bound:?}"
    );
}

fn no_policy_for_an_audience(rig: &dyn PolicyRig) {
    let at = rig.at(&[(
        Audience::Acceptance,
        Required::Acceptance { claims: claims() },
    )]);
    // The policy's absence names the audience alone; the revision beside
    // it is verify's to name (82). A channel, since a layout with no
    // policy written has the given (C.10).
    absent(
        rig.policy().policy_for(
            &at,
            &Audience::Channel(ChannelId::new(None, ChannelName::new("nightly"))),
        ),
        Seam::Policy,
        "nightly",
        "policy-for an audience with no policy",
    );
}

fn a_policy_that_cannot_be_reached(rig: &dyn PolicyRig) {
    let at = rig.at(&[(
        Audience::Acceptance,
        Required::Acceptance { claims: claims() },
    )]);
    let words = rig.unreachable();
    unreadable(
        rig.policy().policy_for(&at, &Audience::Acceptance),
        Seam::Policy,
        &words,
        "policy-for",
    );
}

/// `name` of `component`, or of none, moving as `moves` says.
fn declared(name: &str, component: Option<&str>, moves: Moves) -> DeclaredChannel {
    DeclaredChannel {
        channel: ChannelId::new(component.map(ComponentName::new), ChannelName::new(name)),
        moves,
    }
}

fn bound(to: &str) -> Moves {
    Moves::Bound {
        to: ChannelName::new(to),
    }
}

/// `declared` and what the rig's profile gives `components`, once each
/// by name and then component, as the channels answer them (131).
fn with_given(
    rig: &dyn PolicyRig,
    components: &[ComponentName],
    declared: &[DeclaredChannel],
) -> Vec<DeclaredChannel> {
    let mut all = rig.given(components);
    all.extend(declared.iter().cloned());
    all.sort_by(|a, b| a.channel.cmp(&b.channel));
    all
}

fn the_channels_a_policy_declares(rig: &dyn PolicyRig) {
    let components = [ComponentName::new("lib"), ComponentName::new("web")];
    let channels = [
        declared("edge", Some("lib"), Moves::Promotion),
        declared(
            "nightly",
            Some("web"),
            Moves::EachPublish { prereleases: false },
        ),
        declared("next", Some("lib"), bound("edge")),
    ];
    // Declared in another order than the answer's, which is by name.
    let at = rig.declaring(
        &components,
        &[
            channels[2].clone(),
            channels[0].clone(),
            channels[1].clone(),
        ],
    );
    let listed = ok(rig.policy().channels(&at), "channels").in_force;
    assert_eq!(
        listed,
        with_given(rig, &components, &channels),
        "the policy answered other channels, components or ways they move than it declares and gives, or not by name and then component"
    );
    let next = ok(
        rig.policy()
            .policy_for(&at, &Audience::Channel(channels[2].channel.clone())),
        "policy-for a bound channel",
    );
    assert_eq!(
        next.bound_to(),
        Some(&ChannelName::new("edge")),
        "policy-for and channels disagree on the channel next is bound to"
    );
}

fn each_way_a_channel_moves(rig: &dyn PolicyRig) {
    let channels = [
        declared("edge", None, Moves::EachPublish { prereleases: false }),
        declared("nightly", None, Moves::EachPublish { prereleases: true }),
        declared("rc", None, Moves::Promotion),
        declared("next", None, bound("edge")),
    ];
    let at = rig.declaring(&[], &channels);
    let listed = ok(rig.policy().channels(&at), "channels").in_force;
    assert_eq!(
        listed,
        with_given(rig, &[], &channels),
        "the policy answered other ways a channel moves than it declares"
    );
    let moves = |name: &str| {
        listed
            .iter()
            .find(|c| c.channel.name.as_str() == name)
            .map(|c| c.moves.clone())
    };
    assert_eq!(
        (moves("nightly"), moves("edge")),
        (
            Some(Moves::EachPublish { prereleases: true }),
            Some(Moves::EachPublish { prereleases: false })
        ),
        "a following channel's prereleases are not answered as declared"
    );
    for channel in &channels {
        let policy = ok(
            rig.policy()
                .policy_for(&at, &Audience::Channel(channel.channel.clone())),
            "policy-for a declared channel",
        );
        assert_eq!(
            policy.bound_to().cloned(),
            channel.bound_to().map(|c| c.name),
            "policy-for and channels disagree on what {} is bound to",
            channel.channel
        );
    }
}

fn one_component_has_every_channel(rig: &dyn PolicyRig) {
    for components in [vec![], vec![ComponentName::new("lib")]] {
        let component = components.first().map(ComponentName::as_str);
        let channels = [
            declared("edge", component, Moves::Promotion),
            declared("next", component, bound("edge")),
        ];
        let at = rig.declaring(&components, &channels);
        let listed = ok(rig.policy().channels(&at), "channels").in_force;
        assert_eq!(
            listed,
            with_given(rig, &components, &channels),
            "a repository declaring {components:?} has a channel outside its one component"
        );
        assert!(
            listed
                .iter()
                .all(|c| c.channel.component.as_ref().map(ComponentName::as_str) == component),
            "a channel listed is of another component than {component:?}: {listed:?}"
        );
    }
}

fn a_policy_declaring_no_channel(rig: &dyn PolicyRig) {
    let at = rig.at(&[(
        Audience::Acceptance,
        Required::Acceptance { claims: claims() },
    )]);
    assert_eq!(
        ok(rig.policy().channels(&at), "channels").in_force,
        rig.given(&[]),
        "a policy declaring no channel answered other channels than its profile gives"
    );
}

fn the_channels_of_a_policy_that_cannot_be_reached(rig: &dyn PolicyRig) {
    let at = rig.at(&[(
        Audience::Acceptance,
        Required::Acceptance { claims: claims() },
    )]);
    let words = rig.unreachable();
    unreadable(rig.policy().channels(&at), Seam::Policy, &words, "channels");
}

/// 131, as C.10 has a profile hold givens off: the channels answer
/// carries, apart from the channels in force, what the profile holds off,
/// each with how it would move and the line that switches it on, and no
/// channel is both.
fn the_givens_held_off(rig: &dyn PolicyRig) {
    let components = [ComponentName::new("lib"), ComponentName::new("web")];
    let at = rig.declaring(
        &components,
        &[declared("edge", Some("lib"), Moves::Promotion)],
    );
    let answer = ok(rig.policy().channels(&at), "channels");
    assert_eq!(
        answer.held,
        rig.held(&components),
        "the policy held off other givens, ways they would move or lines than its profile holds off"
    );
    for held in &answer.held {
        assert!(
            !held.switch.is_empty(),
            "{} is held off with no line that switches it on",
            held.channel.channel
        );
        assert!(
            !answer
                .in_force
                .iter()
                .any(|c| c.channel == held.channel.channel),
            "{} is both held off and in force",
            held.channel.channel
        );
    }
}
