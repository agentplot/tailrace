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
//! Apply moves no name and appends nothing (rules core.6, core.16). It
//! signs one claim, the report of what the host observed the target
//! answer once it converged, as the party running it, and writes nothing
//! to the store but that report beside the index it is about (11, 43,
//! 91, rules core.8, core.27; design D6): only once the target's report
//! names the index applied, and, where the environment ran the head
//! already, only where its newest report names another index or none, so
//! a report the store failed to hold is made again and one held is never
//! made twice. A chain the ledger answers as last read is not applied
//! from, since its head may be behind the one held and an apply from it
//! could take the environment back (60). What stops the target is the
//! answer, in its own words (59, rule core.19).

use serde::{Deserialize, Serialize, Serializer};
use timber_seams::{
    Answer, AskedOf, Audience, Branch, ChannelId, ChannelName, Claim, ClaimKind, Component,
    ComponentName, Digest, Environment, EnvironmentName, Held, Manifest, NamedSigner, Not, Read,
    Required, RequiredClaim, Revision, Running, Seam, Seams, SignerId, Source, Verified, Version,
    Why,
};

use crate::ledger::judge;
use crate::{
    answer_json, channel, chosen, newest, policy_at, Channel, Declared, History, Landed, BOUND,
    LOCAL,
};

/// What an apply is asked: the environment of that name in the one
/// component named, or the only one declared (123, 125), at the landed
/// head of the branch whose policy declares it (65, 136); the identity of
/// the party running it, which signs the report (11, 79); and the moment
/// of the act, in seconds since the Unix epoch, so the core reads no
/// clock.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ApplyAsk {
    pub branch: Branch,
    pub component: Option<ComponentName>,
    pub environment: EnvironmentName,
    pub signer: SignerId,
    pub at: u64,
}

/// 43, 57, 91: what an apply answers: the environment, its component,
/// the channel it follows, and the version and index its chain's head
/// names; whether the target was asked to apply it, which it is not where
/// the environment ran it already (89, 90); what `current` answered
/// before, and what the target's report says it `running` after, each its
/// digest or its own answer, an unread one with its reason (119); and the
/// `report` claim's digest, or the answer given in its place (design D6).
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
    pub running: Answer<Running>,
    #[serde(serialize_with = "one_answer")]
    pub report: Answer<Digest>,
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
/// only then the verified version handed to the target, its report read
/// after, and the report claim made of it (design D6). A refusal at any
/// step before the report is the answer, nothing after it is asked, and
/// the target is asked nothing before the chain's head is read; what
/// stops the report claim is its part of the answer, the convergence
/// answered as made (59).
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

    // 6: what the target reports the environment runs after (91), and
    // the report claim the host makes of it (11, 43; design D6).
    let running = target.report(&environment);
    let report = reported(
        seams,
        &Converged {
            history,
            at_head: &at_head,
            component,
            environment: &environment,
            profile: table.profile.as_str(),
            version: &version,
            index: &index,
            already,
        },
        &running,
        ask,
    );
    Ok(Applied {
        environment: environment.name.clone(),
        component: component.name.clone(),
        channel: environment.channel.name.clone(),
        version,
        index,
        applied: !already,
        current,
        running,
        report,
    })
}

/// What an apply knew once the target converged, which its report
/// claim is made of.
struct Converged<'a> {
    history: &'a History,
    at_head: &'a Landed,
    component: &'a Component,
    environment: &'a Environment,
    profile: &'a str,
    version: &'a Version,
    index: &'a Digest,
    already: bool,
}

/// 11, 43, 91, rules core.8, core.9, core.27: the report claim of an
/// apply, signed as `ask`'s identity by the host's signer and attached to
/// the index applied, and its envelope's digest (design D6). The target's
/// own answer in its place where its report is not one, and unreadable
/// naming both digests where it names another than the index applied,
/// nothing signed for either. Where the environment ran the head already,
/// the digest of its newest report where that names the head, nothing
/// signed (design D8), and its answer where it cannot be read. What
/// stops the signer or the store is the answer, in its own words (59).
fn reported(
    seams: &Seams,
    c: &Converged<'_>,
    running: &Answer<Running>,
    ask: &ApplyAsk,
) -> Answer<Digest> {
    let environment = &c.environment.name;
    let running = running.as_ref().map_err(Not::clone)?;
    if running.digest != *c.index {
        return Err(Not::unreadable(
            &Source::new(Seam::Target, c.profile),
            format!(
                "the report of {environment}, which names {} where {} was applied",
                running.digest, c.index
            ),
            None,
        ));
    }

    // Where nothing was applied: the newest report, among those that
    // count through the signers the policy lets report, answered where it
    // names the head already (design D8).
    if c.already {
        let required = RequiredClaim {
            kind: ClaimKind::Report,
            check: None,
            environment: Some(environment.clone()),
            signers: reporters(seams, c.at_head)?,
            asked_of: AskedOf::Every,
        };
        let channel = &c.environment.channel;
        match newest(seams, environment, &c.component.name, channel, &required) {
            Ok(held) if held.report.index == *c.index => return Ok(held.claim),
            Ok(_) | Err(Not::Absent(_)) => {}
            Err(not) => return Err(not),
        }
    }

    // The revision the index names, which the claim names (13, rules
    // core.9, core.60); for an index published before indexes named one,
    // the revision its version's tag names, as a promotion finds it.
    let revision = match seams.store.read(c.index)? {
        Held::Manifest(Manifest::Index(meta)) => match meta.revision() {
            Some(revision) => revision.clone(),
            None => tagged(c)?,
        },
        _ => {
            return Err(Not::unreadable(
                &Source::new(Seam::Store, BOUND),
                format!("{} as an index", c.index),
                Some(format!("{} holds no index", c.index)),
            ))
        }
    };
    let claim = Reported {
        environment: environment.clone(),
        component: c.component.name.clone(),
        channel: c.environment.channel.name.clone(),
        version: c.version.clone(),
        index: c.index.clone(),
        runs: running.digest.clone(),
        converged_at: running.converged_at.clone(),
        at: ask.at,
    }
    .claim(revision);
    let signer = seams
        .signers
        .first()
        .ok_or_else(|| Not::absent(&Source::new(Seam::Signer, BOUND), "signer bound"))?;
    let envelope = signer.sign(&claim, &ask.signer)?;
    seams.store.attach(c.index, &envelope)?;
    Ok(envelope.digest())
}

/// 13: the revision the tag of `c`'s version names, reached from the
/// branch's landed head; absent naming the tag where none is.
fn tagged(c: &Converged<'_>) -> Answer<Revision> {
    let tag = format!("{}{}", c.component.prefix, c.version);
    c.history
        .tags(c.at_head, &c.component.prefix)?
        .into_iter()
        .find(|t| t.name == tag)
        .map(|t| t.revision)
        .ok_or_else(|| {
            Not::absent(
                &Source::history(LOCAL),
                format!("tag {tag} reached from {}", c.at_head.revision()),
            )
        })
}

/// 116, C.10: the signers the policy at `landed` lets claim a report,
/// read from the signers its answer for a layout carries, each with what
/// it may claim, the one answer that lists every signer the policy names;
/// none where it declares no policy for a layout.
fn reporters(seams: &Seams, landed: &Landed) -> Answer<Vec<NamedSigner>> {
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
