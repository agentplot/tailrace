//! An environment's newest report, read across its channel's chain, and
//! each channel an `after` makes moved to what the environments it comes
//! after last reported (37, 41, 43, 83, 91, 97, C.10, C.12, rules
//! core.7, core.34, core.50; design D8, D9).
//!
//! Every version an environment ran came to it through its channel (40,
//! rule core.16), so the claims attached to the indexes the channel's
//! chain moved to hold every report of it. A report counts only through
//! a signer the requirement names that may claim one (rule core.7), and
//! the newest is the one whose apply came last by the moment it names,
//! never by the time the target says it converged, which is the
//! machine's clock (design D8). Reading writes nothing.
//!
//! A channel an `after` makes moves only where every environment it
//! comes after last reported one index, and then by the one move a
//! promotion makes, as the gate's own act: its policy judged, the host's
//! signer one of its promoters, the ledger appended before the store is
//! written (33, 95, 105, 129). Which channels come after an environment
//! is learned through the policy seam alone, its channels answer (131).
//! Nothing here starts on its own: what calls an apply and then this is
//! the host's binding of what tells its gate (130, rule core.52).

use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use timber_seams::{
    Answer, AskedOf, Audience, Branch, ChannelId, Claim, ClaimKind, ComponentName, DeclaredChannel,
    DeclaredEnvironment, Digest, Envelope, EnvironmentName, Moved, Moves, Not, Required,
    RequiredClaim, Seam, Seams, SignerId, Source, Why,
};

use crate::ledger::{held, moved, promotion_of, Asked, Move};
use crate::{
    answer_json, channels_at, chosen, policy_at, tell, Declared, History, Landed, Promoted,
    Reported, BOUND,
};

/// 43, 91: an environment's newest report: the digest of the claim's
/// envelope and what it reports.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Newest {
    pub claim: Digest,
    pub report: Reported,
}

/// 40, 43, 91, rule core.50: the newest report of `environment` of
/// `component`, which follows `channel`: the claims attached to each
/// index `channel`'s chain moved to, once each in the order the chain
/// first moved to it, the reports of that environment and component
/// among them that count through the signers `required` names (rule
/// core.7), and of those the one with the latest `at`, the later in that
/// order where two name one moment. Absent naming the environment where
/// none counts; unreadable naming the chain where the ledger answers it
/// as last read or cannot read it (60), and naming the index whose claims
/// the store could not read (58).
pub fn newest(
    seams: &Seams,
    environment: &EnvironmentName,
    component: &ComponentName,
    channel: &ChannelId,
    required: &RequiredClaim,
) -> Answer<Newest> {
    // 1: the chain of the channel the environment follows (rule
    // core.50), and each index it moved to, once.
    let mut indexes: Vec<Digest> = Vec::new();
    for envelope in held(seams.keeper.as_ref(), channel)? {
        let index = promotion_of(channel, &envelope)?.to.index;
        if !indexes.contains(&index) {
            indexes.push(index);
        }
    }

    // 2, 3: the reports of the environment and component attached to
    // each, kept where they count through the signers the requirement
    // names (rule core.7).
    let mut counted: Vec<Newest> = Vec::new();
    for index in &indexes {
        for envelope in claims_of(seams, channel, index)? {
            let Some(report) = Claim::from_envelope(&envelope)
                .as_ref()
                .and_then(Reported::of)
                .filter(|r| r.environment == *environment && r.component == *component)
            else {
                continue;
            };
            let told = tell(
                seams,
                &required.signers,
                index,
                &envelope,
                Some(ClaimKind::Report),
                None,
                None,
            )?;
            if told.counts {
                counted.push(Newest {
                    claim: told.envelope,
                    report,
                });
            }
        }
    }

    // 4: the one whose apply came last.
    counted
        .into_iter()
        .max_by_key(|n| n.report.at)
        .ok_or_else(|| {
            Not::absent(
                &Source::new(Seam::Store, BOUND),
                format!("a counted report of environment {environment} of {component}"),
            )
        })
}

/// The claims attached to `index`, which `channel`'s chain moved to: an
/// index the store does not hold is unreadable naming it, since the chain
/// says a version was there and its reports cannot be read (58).
fn claims_of(seams: &Seams, channel: &ChannelId, index: &Digest) -> Answer<Vec<Envelope>> {
    seams.store.claims_of(index).map_err(|not| match not {
        Not::Absent(a) => Not::unreadable(
            &a.source,
            format!(
                "the claims of {index}, which the chain of {channel} moved to and the store \
                 holds none of"
            ),
            None,
        ),
        other => other,
    })
}

/// What the gate's move on reports is asked (design D9): the environment
/// applied, of the one component named or the only one declared (123,
/// 125), the branch whose landed head holds the policy, the identity of
/// the host running it, which signs each move and is recorded as who
/// asked (95, 129), and the moment of the act, in seconds since the Unix
/// epoch, so the core reads no clock.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FollowAsk {
    pub branch: Branch,
    pub component: Option<ComponentName>,
    pub environment: EnvironmentName,
    pub signer: SignerId,
    pub at: u64,
}

/// 43, 57, 58: what a channel an `after` makes came to on the newest
/// reports of the environments it comes after.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "after", rename_all = "snake_case")]
pub enum After {
    /// Every environment it comes after last reported one index, and the
    /// channel moved to it or was there already, as a promotion answers
    /// (37, 91).
    Promoted(Promoted),
    /// Not every environment it comes after last reported one index, or
    /// one's newest report could not be found: each named, in `after`'s
    /// order, with its newest report or the answer given in its place;
    /// nothing was signed or appended (58).
    Held {
        #[serde(serialize_with = "each_report")]
        reports: Vec<(EnvironmentName, Answer<Newest>)>,
    },
}

/// 35, 57: the channels an `after` makes that come after the environment
/// applied, in the order the channels answer lists them, each answered
/// on its own: a refusal of one leaves the moves of the others standing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Follows {
    #[serde(serialize_with = "each_channel")]
    pub channels: Vec<(ChannelId, Answer<After>)>,
}

/// 57: each environment's newest report in the one form, naming it.
fn each_report<S: Serializer>(
    reports: &[(EnvironmentName, Answer<Newest>)],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let answers: Vec<Value> = reports
        .iter()
        .map(|(environment, answer)| {
            let mut value = answer_json(answer);
            value["environment"] = json!(environment);
            value
        })
        .collect();
    answers.serialize(serializer)
}

/// 57: each channel's answer in the one form, naming its channel with
/// its component (125), as the gate names each channel it moves.
fn each_channel<S: Serializer>(
    channels: &[(ChannelId, Answer<After>)],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let answers: Vec<Value> = channels
        .iter()
        .map(|(channel, answer)| {
            let mut value = answer_json(answer);
            value["channel"] = json!(channel);
            value
        })
        .collect();
    answers.serialize(serializer)
}

/// 37, 41, 43, 83, 91, 97, C.10: each channel in force at the branch's
/// landed head that an `after` makes and that comes after the
/// environment `ask` names, moved to what the environments it comes
/// after last reported, in the order of design D9: the newest report of
/// each (D8), the channel held where any is absent or unreadable or they
/// name two indexes, and otherwise moved to the one index by the one
/// move a promotion makes ([`moved`]), the host's signer recorded as who
/// asked, whatever approvers the channel names (95, 129). A host whose
/// signer is none of a channel's promoters is refused for that channel
/// naming both, and reads none of its reports (32, 108). An empty list
/// where no channel comes after the environment.
pub fn follow(seams: &Seams, history: &History, ask: &FollowAsk) -> Answer<Follows> {
    // 1: the branch's head, landed (65, 77), the component it declares
    // (123), and the channels in force of it that come after the
    // environment (131).
    let head = history.head(&ask.branch)?;
    let at_head = Landed::check(seams.landing.as_ref(), &ask.branch, &head)?;
    let declared = Declared::read(history, &at_head)?;
    let component = chosen(&declared, ask.component.as_ref(), &head)?;
    let after: Vec<(DeclaredChannel, Vec<EnvironmentName>)> =
        channels_at(seams.policy.as_ref(), &at_head)?
            .into_iter()
            .filter(|c| c.channel.component_name() == component.name)
            .filter_map(|c| match &c.moves {
                Moves::Bound { after, .. } if after.contains(&ask.environment) => {
                    let after = after.clone();
                    Some((c, after))
                }
                _ => None,
            })
            .collect();
    if after.is_empty() {
        return Ok(Follows { channels: vec![] });
    }

    // The environments the policy declares at the head, each one's
    // channel the chain its reports are read across (40); what stops
    // reading them is each report's answer.
    let environments = seams.policy.environments(at_head.revision());

    // 2 to 4: each channel on its own (35).
    let channels = after
        .iter()
        .map(|(listed, after)| {
            let moved = one(
                seams,
                history,
                &Following {
                    ask,
                    at_head: &at_head,
                    component,
                    environments: &environments,
                },
                listed,
                after,
            );
            (listed.channel.clone(), moved)
        })
        .collect();
    Ok(Follows { channels })
}

/// What every channel that comes after the environment is followed with.
struct Following<'a> {
    ask: &'a FollowAsk,
    at_head: &'a Landed,
    component: &'a timber_seams::Component,
    environments: &'a Answer<Vec<DeclaredEnvironment>>,
}

/// One channel an `after` makes, coming after `after`, followed: its
/// policy at the head (82, 107), the host's signer one of its promoters
/// (32, 108), the newest report of each environment it comes after,
/// counted through the signers its policy's report of that environment
/// names (83, rule core.7), and the move to the one index they name.
fn one(
    seams: &Seams,
    history: &History,
    f: &Following<'_>,
    listed: &DeclaredChannel,
    after: &[EnvironmentName],
) -> Answer<After> {
    let channel = &listed.channel;
    let head = f.at_head.revision();
    let required = policy_at(
        seams.policy.as_ref(),
        f.at_head,
        &Audience::Channel(channel.clone()),
    )
    .map_err(|not| match not {
        Not::Absent(a) => Not::absent(&a.source, format!("{} at {head}", a.what)),
        other => other,
    })?;

    // The host's signer one of the channel's promoters, or nothing of its
    // reports is read (32, 108).
    let promoters = required.promoted_by();
    if !promoters.iter().any(|s| s.id == f.ask.signer) {
        return Err(Not::refused(
            &Source::new(Seam::Policy, BOUND),
            Why::NotPromoter {
                channel: channel.clone(),
                signer: f.ask.signer.clone(),
                promoters: promoters.iter().map(|s| s.id.clone()).collect(),
            },
            None,
        ));
    }

    // 2: the newest report of each environment it comes after (D8).
    let reports: Vec<(EnvironmentName, Answer<Newest>)> = after
        .iter()
        .map(|environment| {
            let newest = reported(seams, f, &required, environment);
            (environment.clone(), newest)
        })
        .collect();
    let one = match reports.first() {
        Some((_, Ok(first)))
            if reports.iter().all(|(_, r)| {
                r.as_ref()
                    .is_ok_and(|n| n.report.index == first.report.index)
            }) =>
        {
            Moved {
                version: first.report.version.clone(),
                index: first.report.index.clone(),
            }
        }
        _ => return Ok(After::Held { reports }),
    };

    // 3, 4: the one move a promotion makes, the gate's own act (95, 129).
    moved(
        seams,
        history,
        &Move {
            branch: &f.ask.branch,
            at_head: f.at_head,
            component: f.component,
            listed,
            required: &required,
            asked: Asked::Gate {
                to: &one,
                gate: &f.ask.signer,
            },
            at: f.ask.at,
        },
    )
    .map(After::Promoted)
}

/// 40, 43, 83: the newest report of `environment`, read across the chain
/// of the channel it follows as the policy at the head declares it, and
/// counted through the signers `required`, the channel's policy, names
/// for a report of it, none where it names no such report; absent naming
/// the environment where the policy declares none of that name in the
/// component.
fn reported(
    seams: &Seams,
    f: &Following<'_>,
    required: &Required,
    environment: &EnvironmentName,
) -> Answer<Newest> {
    let component = &f.component.name;
    let declared = f
        .environments
        .as_ref()
        .map_err(Not::clone)?
        .iter()
        .find(|e| e.name == *environment && e.channel.component_name() == *component)
        .ok_or_else(|| {
            Not::absent(
                &Source::new(Seam::Policy, BOUND),
                format!(
                    "environment {environment} of {component} at {}",
                    f.at_head.revision()
                ),
            )
        })?;
    let wanted = required
        .claims()
        .iter()
        .find(|c| c.kind == ClaimKind::Report && c.environment.as_ref() == Some(environment))
        .cloned()
        .unwrap_or_else(|| RequiredClaim {
            kind: ClaimKind::Report,
            check: None,
            environment: Some(environment.clone()),
            signers: vec![],
            asked_of: AskedOf::Every,
        });
    newest(seams, environment, component, &declared.channel, &wanted)
}
