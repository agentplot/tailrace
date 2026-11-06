//! The fake policy (B.5): what each audience requires, for every revision
//! or for one, which channels move on each publish, which come after
//! environments, the givens it holds off, and the environments each
//! revision declares, as a test scripts it. A channel carries its component (125), and the fake gives and
//! holds off no channel a test did not script: the channels a profile
//! gives with nothing declared are that profile's (C.10).

use std::collections::BTreeMap;

use serde_json::json;
use timber_seams::{
    Answer, AskedOf, Audience, ChannelId, ClaimKind, ComponentName, DeclaredChannel,
    DeclaredEnvironment, EnvironmentName, HeldChannel, Moves, NamedSigner, Not, Policy,
    PolicyChannels, Required, RequiredClaim, Revision, Seam,
};

use crate::Fake;

/// The fake policy's operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyOp {
    PolicyFor,
    Channels,
    Environments,
}

/// What each audience requires, at every revision and at one, the
/// channels that move on each publish, at every revision and at one, the
/// environments each bound channel comes after and the signers that may
/// report them, the givens held off, each with how it would move and its
/// line, and the environments each revision declares, by component and
/// then name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PolicyState {
    every: BTreeMap<Audience, Required>,
    at: BTreeMap<(Revision, Audience), Required>,
    follows: BTreeMap<ChannelId, bool>,
    follows_at: BTreeMap<(Revision, ChannelId), bool>,
    after: BTreeMap<ChannelId, Vec<EnvironmentName>>,
    reporters: Vec<NamedSigner>,
    holds: BTreeMap<ChannelId, (Moves, String)>,
    environments: BTreeMap<Revision, Declared>,
}

/// The environments one revision declares, by component and then name.
type Declared = BTreeMap<(Option<ComponentName>, EnvironmentName), DeclaredEnvironment>;

/// A policy in memory.
pub type FakePolicy = Fake<PolicyState, PolicyOp>;

impl Default for FakePolicy {
    fn default() -> Self {
        Self::new()
    }
}

fn of(audience: &Audience) -> String {
    match audience {
        Audience::Acceptance => "a policy for acceptance".into(),
        Audience::Channel(channel) => format!("a policy for channel {channel}"),
        Audience::Layout => "a policy for a layout".into(),
    }
}

impl FakePolicy {
    pub fn new() -> Self {
        Fake::with(Seam::Policy, PolicyState::default())
    }

    /// What `audience` requires at every revision.
    pub fn require(&self, audience: Audience, required: Required) {
        self.state().every.insert(audience, required);
    }

    /// What `audience` requires at `revision`, over what it requires at
    /// every revision (107).
    pub fn require_at(&self, revision: &Revision, audience: Audience, required: Required) {
        self.state()
            .at
            .insert((revision.clone(), audience), required);
    }

    /// 95: `channel`, where a policy is held for it, moves on each
    /// accepted publish at every revision, taking prereleases too where
    /// `prereleases` (C.10).
    pub fn follows(&self, channel: &ChannelId, prereleases: bool) {
        self.state().follows.insert(channel.clone(), prereleases);
    }

    /// 95: `channel` moves on each accepted publish at `revision`, taking
    /// prereleases too where `prereleases` (C.10).
    pub fn follows_at(&self, revision: &Revision, channel: &ChannelId, prereleases: bool) {
        self.state()
            .follows_at
            .insert((revision.clone(), channel.clone()), prereleases);
    }

    /// 41, C.10: `channel` comes after `environments` at every revision,
    /// in the order named, as an environment's `after` makes it: listed
    /// bound to the channel `required` binds it to and coming after them,
    /// and its policy `required` with, after its own claims, a report of
    /// each environment by `reporters`, in the order named (83). A
    /// binding is answered as scripted, never checked, and a channel
    /// `required` binds to none is listed as its policy says, coming
    /// after none. As the policy file's profile lets claim a report the
    /// signers it lists with `report` among their kinds, and names them
    /// in each report an `after` requires (116), the policy for a layout
    /// lists `reporters` with what each may claim, beside the signers a
    /// test scripted it.
    pub fn after(
        &self,
        channel: &ChannelId,
        environments: &[EnvironmentName],
        reporters: &[NamedSigner],
        mut required: Required,
    ) {
        if let Required::Channel { claims, .. } = &mut required {
            claims.extend(environments.iter().map(|environment| RequiredClaim {
                kind: ClaimKind::Report,
                check: None,
                environment: Some(environment.clone()),
                signers: reporters.to_vec(),
                asked_of: AskedOf::Every,
            }));
        }
        let mut state = self.state();
        state.after.insert(channel.clone(), environments.to_vec());
        for reporter in reporters {
            if !state.reporters.iter().any(|r| r.id == reporter.id) {
                state.reporters.push(reporter.clone());
            }
        }
        state
            .every
            .insert(Audience::Channel(channel.clone()), required);
    }

    /// C.10: `channel` held off at every revision, moving as `moves` once
    /// `switch`, the line that switches it on, is written; carried in the
    /// channels answer apart from the channels in force.
    pub fn holds(&self, channel: &ChannelId, moves: Moves, switch: impl Into<String>) {
        self.state()
            .holds
            .insert(channel.clone(), (moves, switch.into()));
    }

    /// 136: the policy at `revision` declares `environment`, in the
    /// component its channel carries (125), over one of that component
    /// and name set before.
    pub fn declares_at(&self, revision: &Revision, environment: DeclaredEnvironment) {
        let key = (
            environment.channel.component.clone(),
            environment.name.clone(),
        );
        self.state()
            .environments
            .entry(revision.clone())
            .or_default()
            .insert(key, environment);
    }
}

/// The policy for a layout `scripted`, or one requiring nothing where
/// none is, listing each of `reporters` it does not list already.
fn with_reporters(scripted: Option<Required>, reporters: &[NamedSigner]) -> Required {
    let mut required = scripted.unwrap_or(Required::Layout {
        claims: vec![],
        given: false,
        signers: vec![],
    });
    if let Required::Layout { signers, .. } = &mut required {
        for reporter in reporters {
            if !signers.iter().any(|s| s.id == reporter.id) {
                signers.push(reporter.clone());
            }
        }
    }
    required
}

impl Policy for FakePolicy {
    fn policy_for(&self, revision: &Revision, audience: &Audience) -> Answer<Required> {
        let given = json!({ "revision": revision, "audience": audience });
        self.ask(PolicyOp::PolicyFor, given, of(audience), |s, source| {
            let scripted =
                s.at.get(&(revision.clone(), audience.clone()))
                    .or_else(|| s.every.get(audience))
                    .cloned();
            match (audience, scripted) {
                (Audience::Layout, scripted) if !s.reporters.is_empty() => {
                    Ok(with_reporters(scripted, &s.reporters))
                }
                (_, scripted) => scripted.ok_or_else(|| Not::absent(source, of(audience))),
            }
        })
    }

    /// 131: a channel for each channel audience held at `revision` or at
    /// every revision, the revision's policy over every revision's, by
    /// name and then component, each in the component its identity
    /// carries (125): bound where its policy names a binding, coming after
    /// the environments the test scripted it after, moving on each publish
    /// where the test scripted it so, and by a promotion alone otherwise,
    /// taking prereleases as scripted. A binding is answered as scripted,
    /// never checked. Apart from them, each given
    /// scripted held off, by name and then component, with how it would
    /// move and its line; an empty list where none is (C.10).
    fn channels(&self, revision: &Revision) -> Answer<PolicyChannels> {
        let given = json!({ "revision": revision });
        self.ask(PolicyOp::Channels, given, "the channels", |s, _| {
            let mut scripted: BTreeMap<&ChannelId, &Required> = BTreeMap::new();
            for (audience, required) in &s.every {
                if let Audience::Channel(channel) = audience {
                    scripted.insert(channel, required);
                }
            }
            for ((at, audience), required) in &s.at {
                if let (true, Audience::Channel(channel)) = (at == revision, audience) {
                    scripted.insert(channel, required);
                }
            }
            let in_force = scripted
                .into_iter()
                .map(|(channel, required)| {
                    let follows = s
                        .follows_at
                        .get(&(revision.clone(), channel.clone()))
                        .or_else(|| s.follows.get(channel));
                    let moves = match (required.bound_to(), follows) {
                        (Some(to), _) => Moves::Bound {
                            to: to.clone(),
                            after: s.after.get(channel).cloned().unwrap_or_default(),
                        },
                        (None, Some(prereleases)) => Moves::EachPublish {
                            prereleases: *prereleases,
                        },
                        (None, None) => Moves::Promotion,
                    };
                    DeclaredChannel {
                        channel: channel.clone(),
                        moves,
                    }
                })
                .collect();
            let held = s
                .holds
                .iter()
                .map(|(channel, (moves, switch))| HeldChannel {
                    channel: DeclaredChannel {
                        channel: channel.clone(),
                        moves: moves.clone(),
                    },
                    switch: switch.clone(),
                })
                .collect();
            Ok(PolicyChannels { in_force, held })
        })
    }

    /// 136: the environments the test declared at `revision`, by
    /// component and then name, as set; an empty list at a revision given
    /// none.
    fn environments(&self, revision: &Revision) -> Answer<Vec<DeclaredEnvironment>> {
        let given = json!({ "revision": revision });
        self.ask(PolicyOp::Environments, given, "the environments", |s, _| {
            Ok(s.environments
                .get(revision)
                .map(|declared| declared.values().cloned().collect())
                .unwrap_or_default())
        })
    }
}

#[cfg(test)]
mod tests {
    use timber_seams::{ChannelName, Check, SignerId, Source, TargetProfile, TargetTable};

    use super::*;
    use crate::{Fault, Times};

    fn claim(check: &str) -> RequiredClaim {
        RequiredClaim {
            kind: ClaimKind::Verdict,
            check: Some(Check::new(check)),
            environment: None,
            signers: vec![],
            asked_of: AskedOf::Every,
        }
    }

    /// A channel of `component`, or of none where it names none.
    fn id(component: Option<&str>, name: &str) -> ChannelId {
        ChannelId::new(component.map(ComponentName::new), ChannelName::new(name))
    }

    #[test]
    fn acceptance_names_no_channel() {
        let policy = FakePolicy::new();
        policy.require(
            Audience::Acceptance,
            Required::Acceptance {
                claims: vec![claim("test")],
            },
        );
        let required = policy
            .policy_for(&Revision::new("4f2a9c1"), &Audience::Acceptance)
            .unwrap();
        assert_eq!(required.bound_to(), None);
        assert_eq!(required.claims(), &[claim("test")]);
    }

    #[test]
    fn a_bound_channel_names_its_channel() {
        let policy = FakePolicy::new();
        let stable = Audience::Channel(id(None, "stable"));
        policy.require(stable.clone(), channel(Some("beta")));
        let required = policy
            .policy_for(&Revision::new("4f2a9c1"), &stable)
            .unwrap();
        assert_eq!(required.bound_to(), Some(&ChannelName::new("beta")));
    }

    #[test]
    fn a_policy_at_one_revision_is_read_at_that_revision() {
        let policy = FakePolicy::new();
        let (old, new) = (Revision::new("old"), Revision::new("new"));
        policy.require(
            Audience::Layout,
            Required::Layout {
                claims: vec![],
                given: false,
                signers: vec![],
            },
        );
        policy.require_at(
            &new,
            Audience::Layout,
            Required::Layout {
                claims: vec![claim("test")],
                given: false,
                signers: vec![],
            },
        );
        assert_eq!(
            policy.policy_for(&old, &Audience::Layout).unwrap().claims(),
            &[]
        );
        assert_eq!(
            policy.policy_for(&new, &Audience::Layout).unwrap().claims(),
            &[claim("test")]
        );
    }

    #[test]
    fn no_policy_declared_is_absent() {
        let policy = FakePolicy::new();
        assert_eq!(
            policy.policy_for(&Revision::new("4f2a9c1"), &Audience::Acceptance),
            Err(Not::absent(
                &Source::new(Seam::Policy, "fake"),
                "a policy for acceptance"
            ))
        );
    }

    #[test]
    fn its_operation_can_be_faulted_and_is_recorded() {
        let policy = FakePolicy::new();
        policy.fault(
            PolicyOp::PolicyFor,
            Fault::Unreadable("no such revision".into()),
            Times::Once,
        );
        assert!(matches!(
            policy.policy_for(&Revision::new("4f2a9c1"), &Audience::Acceptance),
            Err(Not::Unreadable(u)) if u.said.as_deref() == Some("no such revision")
        ));
        assert_eq!(policy.calls().len(), 1);
    }

    fn channel(bound_to: Option<&str>) -> Required {
        Required::Channel {
            claims: vec![],
            bound_to: bound_to.map(ChannelName::new),
            promoted_by: vec![],
            approvers: vec![],
        }
    }

    fn declared(channel: ChannelId, moves: Moves) -> DeclaredChannel {
        DeclaredChannel { channel, moves }
    }

    /// The channels in force `policy` answers at `at`.
    fn listed(policy: &FakePolicy, at: &Revision) -> Answer<Vec<DeclaredChannel>> {
        policy.channels(at).map(|channels| channels.in_force)
    }

    /// Moving on each publish with releases alone.
    const RELEASES: Moves = Moves::EachPublish { prereleases: false };

    fn bound(to: &str) -> Moves {
        Moves::Bound {
            to: ChannelName::new(to),
            after: vec![],
        }
    }

    #[test]
    fn the_channels_move_as_scripted_each_in_its_own_component() {
        let policy = FakePolicy::new();
        let (beta, stable, edge) = (
            id(Some("lib"), "beta"),
            id(Some("lib"), "stable"),
            id(None, "edge"),
        );
        let (at, other) = (Revision::new("4f2a9c1"), Revision::new("77b0e3d"));
        policy.require(Audience::Channel(beta.clone()), channel(None));
        policy.require(Audience::Channel(stable.clone()), channel(Some("beta")));
        policy.require_at(&at, Audience::Channel(edge.clone()), channel(None));
        policy.require(
            Audience::Acceptance,
            Required::Acceptance { claims: vec![] },
        );
        policy.follows(&beta, false);

        assert_eq!(
            listed(&policy, &at),
            Ok(vec![
                declared(beta.clone(), RELEASES),
                declared(edge, Moves::Promotion),
                declared(stable.clone(), bound("beta")),
            ])
        );
        assert_eq!(
            listed(&policy, &other),
            Ok(vec![
                declared(beta, RELEASES),
                declared(stable, bound("beta")),
            ])
        );
        let calls = policy.calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].op, PolicyOp::Channels);
        assert_eq!(calls[0].given["revision"], "4f2a9c1");
    }

    #[test]
    fn a_channel_follows_each_publish_at_one_revision_or_at_every_one() {
        let policy = FakePolicy::new();
        let (beta, edge) = (id(None, "beta"), id(None, "edge"));
        let (at, other) = (Revision::new("4f2a9c1"), Revision::new("77b0e3d"));
        policy.require(Audience::Channel(beta.clone()), channel(None));
        policy.require(Audience::Channel(edge.clone()), channel(None));
        policy.follows(&beta, false);
        policy.follows_at(&at, &edge, false);
        assert_eq!(
            listed(&policy, &at),
            Ok(vec![
                declared(beta.clone(), RELEASES),
                declared(edge.clone(), RELEASES),
            ])
        );
        assert_eq!(
            listed(&policy, &other),
            Ok(vec![
                declared(beta, RELEASES),
                declared(edge, Moves::Promotion),
            ])
        );
    }

    #[test]
    fn a_channel_scripted_after_an_environment_requires_a_report_of_it() {
        let policy = FakePolicy::new();
        let (first, second) = (id(Some("web"), "first"), id(Some("web"), "second"));
        let deployer = NamedSigner {
            id: SignerId::new("deployer"),
            verified_by: "a public key".into(),
            kinds: vec![ClaimKind::Provenance, ClaimKind::Report],
            checks: vec![],
        };
        let at = Revision::new("4f2a9c1");
        policy.require(Audience::Channel(first.clone()), channel(Some("stable")));
        policy.after(
            &second,
            &[EnvironmentName::new("first")],
            std::slice::from_ref(&deployer),
            Required::Channel {
                claims: vec![claim("test")],
                bound_to: Some(ChannelName::new("first")),
                promoted_by: vec![deployer.clone()],
                approvers: vec![],
            },
        );

        assert_eq!(
            listed(&policy, &at),
            Ok(vec![
                declared(first, bound("stable")),
                declared(
                    second.clone(),
                    Moves::Bound {
                        to: ChannelName::new("first"),
                        after: vec![EnvironmentName::new("first")],
                    }
                ),
            ])
        );
        let report = RequiredClaim {
            kind: ClaimKind::Report,
            check: None,
            environment: Some(EnvironmentName::new("first")),
            signers: vec![deployer.clone()],
            asked_of: AskedOf::Every,
        };
        assert_eq!(
            policy.policy_for(&at, &Audience::Channel(second.clone())),
            Ok(Required::Channel {
                claims: vec![claim("test"), report],
                bound_to: Some(ChannelName::new("first")),
                promoted_by: vec![deployer.clone()],
                approvers: vec![],
            })
        );
        let calls = policy.calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].op, PolicyOp::Channels);
        assert_eq!(
            calls[0].answer.as_ref().unwrap()["in_force"][1]["moves"]["after"],
            json!(["first"])
        );
        assert_eq!(calls[1].op, PolicyOp::PolicyFor);
        assert_eq!(
            calls[1].answer.as_ref().unwrap()["channel"]["claims"][1]["environment"],
            "first"
        );

        for op in [PolicyOp::Channels, PolicyOp::PolicyFor] {
            policy.fault(
                op,
                Fault::Unreadable("no such revision".into()),
                Times::Once,
            );
        }
        assert!(matches!(
            listed(&policy, &at),
            Err(Not::Unreadable(u)) if u.said.as_deref() == Some("no such revision")
        ));
        assert!(matches!(
            policy.policy_for(&at, &Audience::Channel(second)),
            Err(Not::Unreadable(u)) if u.said.as_deref() == Some("no such revision")
        ));
        assert_eq!(policy.calls().len(), 4);

        // The policy for a layout lists the deployer, as the file's does
        // every signer that may report, and keeps a scripted signer.
        assert_eq!(
            policy.policy_for(&at, &Audience::Layout),
            Ok(Required::Layout {
                claims: vec![],
                given: false,
                signers: vec![deployer.clone()],
            })
        );
        let rebuild = NamedSigner {
            id: SignerId::new("rebuild"),
            kinds: vec![ClaimKind::Rebuild],
            ..deployer.clone()
        };
        policy.require(
            Audience::Layout,
            Required::Layout {
                claims: vec![claim("test")],
                given: false,
                signers: vec![rebuild.clone()],
            },
        );
        assert_eq!(
            policy.policy_for(&at, &Audience::Layout),
            Ok(Required::Layout {
                claims: vec![claim("test")],
                given: false,
                signers: vec![rebuild, deployer],
            })
        );
    }

    #[test]
    fn a_following_channel_scripted_to_take_prereleases_is_listed_so() {
        let policy = FakePolicy::new();
        let (beta, nightly) = (id(None, "beta"), id(None, "nightly"));
        policy.require(Audience::Channel(beta.clone()), channel(None));
        policy.require(Audience::Channel(nightly.clone()), channel(None));
        policy.follows(&beta, false);
        policy.follows(&nightly, true);
        assert_eq!(
            listed(&policy, &Revision::new("4f2a9c1")),
            Ok(vec![
                declared(beta, RELEASES),
                declared(nightly, Moves::EachPublish { prereleases: true }),
            ])
        );
    }

    #[test]
    fn two_components_channels_of_one_name_are_two() {
        let policy = FakePolicy::new();
        let (libs, webs) = (id(Some("lib"), "beta"), id(Some("web"), "beta"));
        policy.require(Audience::Channel(webs.clone()), channel(None));
        policy.require(Audience::Channel(libs.clone()), channel(None));
        policy.follows(&webs, false);
        assert_eq!(
            listed(&policy, &Revision::new("4f2a9c1")),
            Ok(vec![
                declared(libs, Moves::Promotion),
                declared(webs, RELEASES),
            ])
        );
    }

    #[test]
    fn a_channels_policy_at_a_revision_is_read_over_every_revisions() {
        let policy = FakePolicy::new();
        let stable = Audience::Channel(id(None, "stable"));
        let (old, new) = (Revision::new("old"), Revision::new("new"));
        policy.require(stable.clone(), channel(None));
        policy.require_at(&new, stable, channel(Some("beta")));
        policy.require_at(&new, Audience::Channel(id(None, "edge")), channel(None));
        assert_eq!(
            listed(&policy, &old),
            Ok(vec![declared(id(None, "stable"), Moves::Promotion)])
        );
        assert_eq!(
            listed(&policy, &new),
            Ok(vec![
                declared(id(None, "edge"), Moves::Promotion),
                declared(id(None, "stable"), bound("beta")),
            ])
        );
    }

    #[test]
    fn no_channel_scripted_answers_an_empty_list() {
        let policy = FakePolicy::new();
        policy.require(
            Audience::Acceptance,
            Required::Acceptance { claims: vec![] },
        );
        policy.follows(&id(None, "beta"), false);
        assert_eq!(listed(&policy, &Revision::new("4f2a9c1")), Ok(vec![]));
    }

    #[test]
    fn a_given_held_off_is_carried_apart_from_the_channels_in_force() {
        let policy = FakePolicy::new();
        let rev = Revision::new("4f2a9c1");
        assert_eq!(
            policy.channels(&rev),
            Ok(PolicyChannels {
                in_force: vec![],
                held: vec![],
            })
        );
        let canary = id(None, "canary");
        policy.holds(&canary, RELEASES, "[channels.canary]");
        let answer = policy.channels(&rev).unwrap();
        assert_eq!(
            answer.held,
            [HeldChannel {
                channel: declared(canary.clone(), RELEASES),
                switch: "[channels.canary]".into(),
            }]
        );
        assert!(
            !answer.in_force.iter().any(|c| c.channel == canary),
            "{answer:?}"
        );
        policy.fault(
            PolicyOp::Channels,
            Fault::Unreadable("no such revision".into()),
            Times::Once,
        );
        assert!(matches!(
            policy.channels(&rev),
            Err(Not::Unreadable(u)) if u.said.as_deref() == Some("no such revision")
        ));
        let calls = policy.calls();
        assert_eq!(calls.len(), 3);
        assert!(calls.iter().all(|c| c.op == PolicyOp::Channels));
    }

    #[test]
    fn the_channels_can_be_faulted_and_are_recorded() {
        let policy = FakePolicy::new();
        let beta = Audience::Channel(id(None, "beta"));
        policy.require(beta.clone(), channel(None));
        policy.fault(
            PolicyOp::Channels,
            Fault::Unreadable("no such revision".into()),
            Times::Once,
        );
        let rev = Revision::new("4f2a9c1");
        assert!(matches!(
            listed(&policy, &rev),
            Err(Not::Unreadable(u)) if u.said.as_deref() == Some("no such revision")
        ));
        assert_eq!(listed(&policy, &rev).map(|c| c.len()), Ok(1));
        let calls = policy.calls();
        assert_eq!(calls.len(), 2);
        assert!(calls.iter().all(|c| c.op == PolicyOp::Channels));
        assert!(policy.policy_for(&rev, &beta).is_ok());
    }

    /// `name` of `component`, following `channel` of it, with a target of
    /// `profile` where it names one.
    fn environment(
        component: &str,
        name: &str,
        channel: &str,
        target: Option<&str>,
    ) -> DeclaredEnvironment {
        DeclaredEnvironment {
            name: EnvironmentName::new(name),
            channel: id(Some(component), channel),
            waits: false,
            approvers: vec![],
            target: target.map(|profile| TargetTable {
                profile: TargetProfile::new(profile),
                settings: json!({ "at": "a place of its own", "outputs": [name] }),
            }),
        }
    }

    #[test]
    fn the_environments_are_answered_as_set_at_their_revision_alone() {
        let policy = FakePolicy::new();
        let (at, other) = (Revision::new("4f2a9c1"), Revision::new("77b0e3d"));
        let cache = environment("web", "cache", "beta", Some("pushed"));
        policy.declares_at(&at, cache.clone());
        assert_eq!(policy.environments(&at), Ok(vec![cache]));
        assert_eq!(policy.environments(&other), Ok(vec![]));
        let calls = policy.calls();
        assert_eq!(calls.len(), 2);
        assert!(calls.iter().all(|c| c.op == PolicyOp::Environments));
        assert_eq!(calls[0].given["revision"], "4f2a9c1");
        assert_eq!(
            calls[0].answer.as_ref().unwrap()[0]["target"]["profile"],
            "pushed"
        );
    }

    #[test]
    fn the_environments_are_ordered_by_component_and_then_name() {
        let policy = FakePolicy::new();
        let at = Revision::new("4f2a9c1");
        let prod = DeclaredEnvironment {
            waits: true,
            approvers: vec![SignerId::new("box")],
            ..environment("web", "prod", "prod", Some("pulled"))
        };
        let cache = environment("web", "cache", "beta", Some("pushed"));
        let staging = environment("lib", "staging", "beta", None);
        policy.declares_at(&at, prod.clone());
        policy.declares_at(&at, cache.clone());
        policy.declares_at(&at, staging.clone());
        assert_eq!(policy.environments(&at), Ok(vec![staging, cache, prod]));

        let moved = environment("web", "cache", "stable", None);
        policy.declares_at(&at, moved.clone());
        assert_eq!(
            policy.environments(&at).map(|e| e[1].clone()),
            Ok(moved),
            "a second environment of one component and name is the one set"
        );
    }

    #[test]
    fn the_environments_can_be_faulted_and_are_recorded() {
        let policy = FakePolicy::new();
        let rev = Revision::new("4f2a9c1");
        policy.declares_at(&rev, environment("web", "cache", "beta", Some("pushed")));
        policy.fault(
            PolicyOp::Environments,
            Fault::Unreadable("no such revision".into()),
            Times::Once,
        );
        assert!(matches!(
            policy.environments(&rev),
            Err(Not::Unreadable(u)) if u.said.as_deref() == Some("no such revision")
        ));
        assert_eq!(policy.environments(&rev).map(|e| e.len()), Ok(1));
        let calls = policy.calls();
        assert_eq!(calls.len(), 2);
        assert!(calls.iter().all(|c| c.op == PolicyOp::Environments));
        assert!(matches!(&calls[0].answer, Err(Not::Unreadable(_))));
    }
}
