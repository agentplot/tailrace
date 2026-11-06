//! `timber apply`: one environment converged on the version its
//! channel's chain names, the claims the channel's policy requires of it
//! judged before the target is handed anything (40, 42, 88 to 91, 136,
//! rules core.4, core.16, core.29, core.50; design D5).
//!
//! The environment, its channel and its target are learned through the
//! policy seam alone, at the governed branch's landed head (136, rule
//! core.22); the channel's head through the ledger seam alone, read from
//! its chain and never from the channel's name in the store (rule
//! core.50); the target through the one bound for the profile the
//! environment's `target` table names (92). The target is handed the
//! environment with its own settings as the policy answered them, and
//! reads them itself (45, rule core.43).
//!
//! Apply moves no name, appends nothing and signs nothing: it reads the
//! store for the head's claims and writes nothing there (rules core.6,
//! core.16). A chain the ledger answers as last read is not applied
//! from, since its head may be behind the one held and an apply from it
//! could take the environment back (60). What stops the target is the
//! answer, in its own words (59, rule core.19).

use serde::{Deserialize, Serialize, Serializer};
use timber_seams::{
    Answer, Audience, Branch, ChannelId, ChannelName, Claim, ClaimKind, ComponentName, Digest,
    Environment, EnvironmentName, Not, Read, Revision, Running, Seam, Seams, Source, Verified,
    Version, Why,
};

use crate::ledger::judge;
use crate::{answer_json, channel, chosen, policy_at, Channel, Declared, History, Landed, BOUND};

/// What an apply is asked: the environment of that name in the one
/// component named, or the only one declared (123, 125), at the landed
/// head of the branch whose policy declares it (65, 136).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ApplyAsk {
    pub branch: Branch,
    pub component: Option<ComponentName>,
    pub environment: EnvironmentName,
}

/// 43, 57, 91: what an apply answers: the environment, its component,
/// the channel it follows, and the version and index its chain's head
/// names; whether the target was asked to apply it, which it is not where
/// the environment ran it already (89, 90); what `current` answered
/// before, and the target's `report` after, each its digest or its own
/// answer, an unread one with its reason (119).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Applied {
    pub environment: EnvironmentName,
    pub component: ComponentName,
    pub channel: ChannelName,
    pub version: Version,
    pub index: Digest,
    pub applied: bool,
    #[serde(serialize_with = "one_answer")]
    pub current: Answer<Digest>,
    #[serde(serialize_with = "one_answer")]
    pub report: Answer<Running>,
}

/// 43, 91, C.12: the body of a report claim, what the host that ran an
/// apply observed the target answer once it converged: the environment
/// and its component, the channel it follows, the version and index
/// applied, the digest the target reported it runs and the time the
/// target says it converged, and `at`, the moment of the apply that made
/// it, in seconds since the Unix epoch, by which an environment's newest
/// report is ordered (design D6, D8).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reported {
    pub environment: EnvironmentName,
    pub component: ComponentName,
    pub channel: ChannelName,
    pub version: Version,
    pub index: Digest,
    pub runs: Digest,
    pub converged_at: String,
    pub at: u64,
}

impl Reported {
    /// The report `claim` holds, or `None` for a claim of another kind,
    /// one whose body is not a report's, or one about another digest than
    /// the index it reports.
    pub fn of(claim: &Claim) -> Option<Reported> {
        if claim.kind != ClaimKind::Report {
            return None;
        }
        let reported: Reported = serde_json::from_value(claim.body.clone()).ok()?;
        (reported.index == claim.subject).then_some(reported)
    }

    /// 13, rule core.9: the claim about the index applied, made for
    /// `revision`, the revision the index names, with no build identity,
    /// which a report has none of.
    pub fn claim(&self, revision: Revision) -> Claim {
        Claim {
            subject: self.index.clone(),
            kind: ClaimKind::Report,
            revision,
            build: None,
            body: serde_json::to_value(self).expect("a report is plain data"),
        }
    }
}

/// 57: an answer in the one form every surface renders.
fn one_answer<T: Serialize, S: Serializer>(
    answer: &Answer<T>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    answer_json(answer).serialize(serializer)
}

fn policy() -> Source {
    Source::new(Seam::Policy, BOUND)
}

fn keeper() -> Source {
    Source::new(Seam::Keeper, BOUND)
}

/// 31, 60, rule core.50: the version and index the head promotion of
/// `channel` moved it to, as the ledger holds the chain now: absent
/// naming the channel where it holds none, and unreadable naming the
/// chain, with the ledger's words, where the ledger answered it as last
/// read.
fn head_of(seams: &Seams, channel_id: &ChannelId) -> Answer<(Version, Digest)> {
    let (read, head) = match channel(seams.keeper.as_ref(), channel_id)? {
        Channel::Empty { read } => (read, None),
        Channel::At { head, read, .. } => (read, Some(head)),
    };
    if let Read::Kept { said, .. } = read {
        return Err(Not::unreadable(
            &keeper(),
            format!("the chain of {channel_id}"),
            Some(said),
        ));
    }
    head.map(|promotion| (promotion.to.version, promotion.to.index))
        .ok_or_else(|| Not::absent(&keeper(), format!("head of the chain of {channel_id}")))
}

/// 40, 42, 88 to 91: the environment `ask` names converged on its
/// channel's head, in the order of design D5: the environment found
/// among those the policy declares at the branch's landed head, with its
/// target table and the target bound for its profile (92, 136); the
/// chain's head (31, rule core.50); the target's desired, which must be
/// that head (40, rule core.16), and its current, where nothing is
/// applied if it answers the head already; the claims the channel's
/// policy requires judged of the head's index (42, rule core.29); and
/// only then the verified version handed to the target, and its report
/// read after. A refusal at any step is the answer, nothing after it is
/// asked, and the target is asked nothing before the chain's head is
/// read.
pub fn apply(seams: &Seams, history: &History, ask: &ApplyAsk) -> Answer<Applied> {
    // 1: the branch's head, landed (65, 77), and the component it
    // declares (123).
    let head = history.head(&ask.branch)?;
    let at_head = Landed::check(seams.landing.as_ref(), &ask.branch, &head)?;
    let declared = Declared::read(history, &at_head)?;
    let component = chosen(&declared, ask.component.as_ref(), &head)?;

    // 2: the environment of that name in that component, its target
    // table, and the target bound for the table's profile (92, 125,
    // 136).
    let declared_environment = seams
        .policy
        .environments(at_head.revision())?
        .into_iter()
        .find(|e| e.name == ask.environment && e.channel.component_name() == component.name)
        .ok_or_else(|| {
            Not::absent(
                &policy(),
                format!(
                    "environment {} of {} at {head}",
                    ask.environment, component.name
                ),
            )
        })?;
    let table = declared_environment.target.ok_or_else(|| {
        Not::absent(
            &policy(),
            format!(
                "target of environment {} of {} at {head}",
                ask.environment, component.name
            ),
        )
    })?;
    let target = seams.target(&table.profile).ok_or_else(|| {
        Not::absent(
            &Source::new(Seam::Target, BOUND),
            format!("target profile {}", table.profile),
        )
    })?;
    let environment = Environment {
        name: declared_environment.name,
        channel: declared_environment.channel,
        waits: declared_environment.waits,
        settings: table.settings,
    };

    // 3: the chain's head, the target asked nothing where there is none
    // (31).
    let (version, index) = head_of(seams, &environment.channel)?;

    // 4: what the target says the environment should run, handed the
    // head, which it must answer (40, 88); and what it runs, where
    // nothing is applied if it runs the head already (89, 90).
    let desired = target.desired(&environment, &|| Ok(index.clone()))?;
    if desired != index {
        return Err(Not::refused(
            &Source::new(Seam::Target, table.profile.as_str()),
            Why::Desired {
                channel: environment.channel.clone(),
                head: index,
                desired,
            },
            None,
        ));
    }
    let current = target.current(&environment);
    let already = current.as_ref() == Ok(&index);

    // 5: the claims the channel's policy requires at the landed head,
    // judged of the head's index before the target is handed anything
    // (42, 58, rule core.29), absent where no policy is declared for the
    // channel (82); then the verified version handed to the target.
    if !already {
        let audience = Audience::Channel(environment.channel.clone());
        let required =
            policy_at(seams.policy.as_ref(), &at_head, &audience).map_err(|not| match not {
                Not::Absent(a) => Not::absent(&a.source, format!("{} at {head}", a.what)),
                other => other,
            })?;
        judge(seams, &required, &index)?;
        target.apply(&environment, &Verified::new(index.clone(), version.clone()))?;
    }

    // 6: what the target reports the environment runs after (91).
    Ok(Applied {
        environment: environment.name.clone(),
        component: component.name.clone(),
        channel: environment.channel.name.clone(),
        version,
        index,
        applied: !already,
        current,
        report: target.report(&environment),
    })
}
