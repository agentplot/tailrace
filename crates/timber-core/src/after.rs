//! An environment's newest report, read across its channel's chain, and
//! each channel an `after` makes moved to what the environments it comes
//! after last reported (37, 41, 43, 83, 91, 97, 98, C.10, C.12, rules
//! core.7, core.34, core.42, core.50; record 0028's design D8, D9, and
//! the design D3 of the change that reads environments in status).
//!
//! Every version an environment ran came to it through its channel (40,
//! rule core.16), so the claims attached to the indexes the channel's
//! chain moved to hold every report of it. Reading them ([`reports`]) and
//! counting them ([`newest_of`]) are apart, so the gate and apply read
//! the chain and the store with no copy ([`newest`]) and status reads
//! them through the host's copy, each counting alike. A report counts
//! only through a signer the policy lets claim one ([`reporters`], rule
//! core.7), and the newest is the one whose apply came last by the
//! moment it names, never by the time the target says it converged,
//! which is the machine's clock. Reports are told newest first and none
//! older than the first that counts is told. Reading writes nothing.
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
    Answer, Audience, Branch, ChannelId, Claim, ClaimKind, ComponentName, DeclaredChannel,
    DeclaredEnvironment, Digest, Envelope, EnvironmentName, Moved, Moves, NamedSigner, Not,
    Required, Seam, Seams, SignerId, Source, Why,
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

/// 43, 91, rule core.7: an environment's newest report and the signer
/// the policy names that made it, which counted it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Counted {
    pub newest: Newest,
    pub signer: SignerId,
}

/// One report claim as read, not yet counted: the index it is attached
/// to, its envelope, and what it reports.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReportClaim {
    pub index: Digest,
    pub envelope: Envelope,
    pub report: Reported,
}

/// 43, 91: the report claims of `environment` of `component` attached to
/// the indexes its channel's chain moved to, in chain order, none yet
/// counted (design D3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Reports {
    pub environment: EnvironmentName,
    pub component: ComponentName,
    pub attached: Vec<ReportClaim>,
}

/// 40, 43, 91, rule core.50: the report claims of `environment` of
/// `component` attached to each of `indexes`, each index asked of
/// `claims_of` once, in the order given, which is the order the chain
/// first moved to it, and each index's claims in the order it answers
/// them; read by body and not yet counted (design D3). What `claims_of`
/// answers in place of an index's claims is the answer, the first in
/// that order.
pub fn reports(
    claims_of: &mut dyn FnMut(&Digest) -> Answer<Vec<Envelope>>,
    indexes: &[Digest],
    environment: &EnvironmentName,
    component: &ComponentName,
) -> Answer<Reports> {
    let mut asked: Vec<&Digest> = Vec::new();
    let mut attached = Vec::new();
    for index in indexes {
        if asked.contains(&index) {
            continue;
        }
        asked.push(index);
        for envelope in claims_of(index)? {
            let Some(report) = Claim::from_envelope(&envelope)
                .as_ref()
                .and_then(Reported::of)
                .filter(|r| r.environment == *environment && r.component == *component)
            else {
                continue;
            };
            attached.push(ReportClaim {
                index: index.clone(),
                envelope,
                report,
            });
        }
    }
    Ok(Reports {
        environment: environment.clone(),
        component: component.clone(),
        attached,
    })
}

/// 43, 91, 116, rules core.7, core.42: of `reports`, the newest that
/// counts through `reporters`: taken latest `at` first, the later in
/// chain order where two name one moment, each told until one counts,
/// and none older than it told (design D3). A report whose signer cannot
/// be told before one counts makes the answer unreadable naming it, with
/// what could not be read, since it may be the newest. Absent naming the
/// environment where none counts, saying so where `reporters` is empty,
/// the policy letting no signer claim a report (58).
pub fn newest_of(seams: &Seams, reporters: &[NamedSigner], reports: &Reports) -> Answer<Counted> {
    let mut newest_first: Vec<(usize, &ReportClaim)> =
        reports.attached.iter().enumerate().collect();
    newest_first.sort_by(|(i, a), (j, b)| (b.report.at, j).cmp(&(a.report.at, i)));
    for (_, attached) in newest_first {
        let told = tell(
            seams,
            reporters,
            &attached.index,
            &attached.envelope,
            Some(ClaimKind::Report),
            None,
            None,
        )
        .map_err(|not| untold(reports, attached, not))?;
        if told.counts {
            let signer = told.made_by.expect("a counted claim names who made it");
            return Ok(Counted {
                newest: Newest {
                    claim: told.envelope,
                    report: attached.report.clone(),
                },
                signer,
            });
        }
    }
    let (environment, component) = (&reports.environment, &reports.component);
    let what = format!("a counted report of environment {environment} of {component}");
    Err(Not::absent(
        &Source::new(Seam::Store, BOUND),
        if reporters.is_empty() {
            format!("{what}: the policy lets no signer claim a report")
        } else {
            what
        },
    ))
}

/// Rule core.42: what a signer answered in place of telling `attached`,
/// unreadable naming the report, with the signer's source and words.
fn untold(reports: &Reports, attached: &ReportClaim, not: Not) -> Not {
    let what = format!(
        "the report {} of environment {} of {} on {}",
        attached.envelope.digest(),
        reports.environment,
        reports.component,
        attached.index
    );
    match not {
        Not::Unreadable(u) => Not::unreadable(&u.source, format!("{what}: {}", u.what), u.said),
        other => Not::unreadable(other.source(), what, Some(other.to_string())),
    }
}

/// 116, C.10: the signers the policy at `landed` lets claim a report,
/// read from the signers its answer for a layout carries, each with what
/// it may claim, the one answer that lists every signer the policy names;
/// none where it declares no policy for a layout. Apply, the gate's move
/// on reports and status count an environment's reports through them
/// alike (record 0028; design D3).
pub fn reporters(seams: &Seams, landed: &Landed) -> Answer<Vec<NamedSigner>> {
    let signers = match policy_at(seams.policy.as_ref(), landed, &Audience::Layout) {
        Ok(Required::Layout { signers, .. }) => signers,
        Ok(_) | Err(Not::Absent(_)) => Vec::new(),
        Err(not) => return Err(not),
    };
    Ok(signers
        .into_iter()
        .filter(|s| s.kinds.contains(&ClaimKind::Report))
        .collect())
}

/// 40, 43, 91, rules core.45, core.50: the gate's and apply's newest
/// report of `environment` of `component`, which follows `channel`: the
/// chain as the ledger holds it, unreadable naming it where the ledger
/// answers it as last read or cannot read it (60), the claims of each
/// index it moved to as the store answers them, with no copy, unreadable
/// naming the index whose claims the store could not read or holds none
/// of (58), and of the reports among them the newest that counts through
/// `reporters` ([`newest_of`]).
pub fn newest(
    seams: &Seams,
    environment: &EnvironmentName,
    component: &ComponentName,
    channel: &ChannelId,
    reporters: &[NamedSigner],
) -> Answer<Newest> {
    let mut indexes: Vec<Digest> = Vec::new();
    for envelope in held(seams.keeper.as_ref(), channel)? {
        indexes.push(promotion_of(channel, &envelope)?.to.index);
    }
    let reports = reports(
        &mut |index| claims_of(seams, channel, index),
        &indexes,
        environment,
        component,
    )?;
    newest_of(seams, reporters, &reports).map(|counted| counted.newest)
}

/// The claims attached to `index`, which `channel`'s chain moved to: an
/// index the store does not hold is unreadable naming it, since the chain
/// says a version was there and its reports cannot be read (58).
pub(crate) fn claims_of(
    seams: &Seams,
    channel: &ChannelId,
    index: &Digest,
) -> Answer<Vec<Envelope>> {
    seams
        .store
        .claims_of(index)
        .map_err(|not| index_absent(channel, index, not))
}

/// An index whose claims the store answered absent, unreadable naming
/// it; any other answer as it is.
pub(crate) fn index_absent(channel: &ChannelId, index: &Digest, not: Not) -> Not {
    match not {
        Not::Absent(a) => Not::unreadable(
            &a.source,
            format!(
                "the claims of {index}, which the chain of {channel} moved to and the store \
                 holds none of"
            ),
            None,
        ),
        other => other,
    }
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
    // channel the chain its reports are read across (40), and the signers
    // the policy lets claim a report, which they count through (116); what
    // stops reading either is each report's answer.
    let environments = seams.policy.environments(at_head.revision());
    let reporters = reporters(seams, &at_head);

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
                    reporters: &reporters,
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
    reporters: &'a Answer<Vec<NamedSigner>>,
}

/// One channel an `after` makes, coming after `after`, followed: its
/// policy at the head (82, 107), the host's signer one of its promoters
/// (32, 108), the newest report of each environment it comes after,
/// counted through the signers the policy lets claim a report, which its
/// policy's report of that environment names (83, 116, rule core.7), and
/// the move to the one index they name.
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
            let newest = reported(seams, f, environment);
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
/// counted through the signers the policy lets claim a report (116);
/// absent naming the environment where the policy declares none of that
/// name in the component.
fn reported(seams: &Seams, f: &Following<'_>, environment: &EnvironmentName) -> Answer<Newest> {
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
    let reporters = f.reporters.as_ref().map_err(Not::clone)?;
    newest(seams, environment, component, &declared.channel, reporters)
}
