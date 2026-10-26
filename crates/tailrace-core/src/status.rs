//! `tailrace status`: where each channel the policy declares is and the
//! chain of promotions that put it there, read from the ledger on any
//! machine, beside which versions of which components this host
//! published, from which revisions, at which index, answered from the
//! host's record alone (31, 36, 56 to 61, 101, 125, 131; design D1, D2).
//!
//! Status asks the landing, the policy and the ledger alone, and the
//! host's record: it reaches no store, signer, target or projection, and
//! it writes nothing (60, rule core.32). Every reader of one ledger is
//! answered one version per channel as of when it asked, and a chain the
//! ledger answers as last read says when (36, 61). Each part, and each
//! channel, carries its own answer, so absent, refused and unreadable are
//! kept apart at every depth (58, 59), and every surface renders the one
//! answer made here (57, rule core.31).
//!
//! Beside the channels and the record, the environments are a part of
//! their own, answered unreadable while no target is read, never empty or
//! a count of none, and asking no target (58, 60, 98). One repository's
//! row of every channel's head and its environments' counts is made from
//! its status answer alone, so the host's serving process shows every
//! repository it knows together from the one core (132; design D14).

use serde::{Deserialize, Serialize, Serializer};
use tailrace_seams::{
    Answer, Branch, ChannelName, ComponentName, DeclaredChannel, Digest, Not, Read, Revision, Seam,
    Seams, Source, Version,
};

use crate::{
    answer_json, channel, channels_at, Channel, Entry, Filter, History, Kept, Landed,
    RepositoryName,
};

/// What status is asked: every version this host published, or those of
/// one component, of one revision, or both.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusAsk {
    pub component: Option<ComponentName>,
    pub revision: Option<Revision>,
}

/// 61: per component, the versions this host published, ordered by the
/// version rules (23), and the record they were read from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    pub record: String,
    pub components: Vec<Versions>,
}

/// One component's versions this host published.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Versions {
    pub component: ComponentName,
    pub versions: Vec<Entry>,
}

/// 60: the versions `ask` names, read from `kept` and nothing else;
/// absent when the record holds none, unreadable naming an entry that
/// cannot be read (58).
pub fn status(kept: &Kept, ask: &StatusAsk) -> Answer<Status> {
    let entries = kept.read(&Filter {
        component: ask.component.clone(),
        revision: ask.revision.clone(),
    })?;
    let mut components: Vec<Versions> = Vec::new();
    for entry in entries {
        match components.last_mut() {
            Some(last) if last.component == entry.component => last.versions.push(entry),
            _ => components.push(Versions {
                component: entry.component.clone(),
                versions: vec![entry],
            }),
        }
    }
    Ok(Status {
        record: kept.dir().display().to_string(),
        components,
    })
}

/// What the whole of status is asked: the branch whose landed head holds
/// the policy, one component for both parts or every one, one revision
/// for the record's part or every one, and the asker's moment, in
/// seconds since the Unix epoch, so the core reads no clock (36).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OverviewAsk {
    pub branch: Branch,
    pub component: Option<ComponentName>,
    pub revision: Option<Revision>,
    pub at: u64,
}

/// 36, 57: the whole of status, as of `asked_at`: the channels, read
/// from the ledger, and what this host published, from its record, each
/// part its own answer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Overview {
    pub branch: Branch,
    pub asked_at: u64,
    #[serde(serialize_with = "one_answer")]
    pub channels: Answer<Channels>,
    #[serde(serialize_with = "one_answer")]
    pub environments: Answer<EnvironmentCounts>,
    #[serde(serialize_with = "one_answer")]
    pub published: Answer<Status>,
}

/// 131: the channels the policy declares at `revision`, the branch's
/// landed head, in the policy seam's order, and how the landing read the
/// head as landed: from itself, or from the host's copy as last fetched
/// (77; design D6).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Channels {
    pub revision: Revision,
    pub landing: Read,
    pub channels: Vec<ChannelStatus>,
}

/// 31, 101: one channel as the policy declares it, and where its chain
/// in the ledger puts it, its own answer (58).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ChannelStatus {
    #[serde(flatten)]
    pub declared: DeclaredChannel,
    #[serde(serialize_with = "one_answer")]
    pub at: Answer<Channel>,
}

impl ChannelStatus {
    pub fn name(&self) -> &ChannelName {
        &self.declared.channel.name
    }
}

/// 57: an inner answer in the one form every surface renders.
fn one_answer<T: Serialize, S: Serializer>(
    answer: &Answer<T>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    answer_json(answer).serialize(serializer)
}

fn record_ask(ask: &OverviewAsk) -> StatusAsk {
    StatusAsk {
        component: ask.component.clone(),
        revision: ask.revision.clone(),
    }
}

/// 31, 101, 125, 131: every channel the policy at the branch's landed
/// head declares, of `ask.component` where it names one, each with its
/// chain as the ledger answers it. The head not landed is refused before
/// the policy is read (65, 77), and refused as not landed as last fetched
/// where the landing read that from its copy; a head landed as last
/// fetched is answered as one landed on the remote, saying so. A
/// channel's absent, refused or unreadable chain is that channel's
/// answer, never the whole one's (58, 105).
pub fn channels(seams: &Seams, history: &History, ask: &OverviewAsk) -> Answer<Channels> {
    let head = history.head(&ask.branch)?;
    let landed = Landed::check(seams.landing.as_ref(), &ask.branch, &head)?;
    let declared = channels_at(seams.policy.as_ref(), &landed)?;
    Ok(Channels {
        revision: head,
        landing: landed.read().clone(),
        channels: declared
            .into_iter()
            .filter(|c| {
                ask.component
                    .as_ref()
                    .is_none_or(|named| c.channel.component_name() == *named)
            })
            .map(|declared| ChannelStatus {
                at: channel(seams.keeper.as_ref(), &declared.channel),
                declared,
            })
            .collect(),
    })
}

/// 36, 60: the whole of status, each part its own answer: the channels
/// over `seams`, the environments, and what this host published from
/// `kept`.
pub fn overview(seams: &Seams, history: &History, kept: &Kept, ask: &OverviewAsk) -> Overview {
    Overview {
        branch: ask.branch.clone(),
        asked_at: ask.at,
        channels: channels(seams, history, ask),
        environments: environments(seams, ask),
        published: status(kept, &record_ask(ask)),
    }
}

/// The whole of status where no binding could be made, for `not`: the
/// channels' part that answer, the environments' as ever unread, and the
/// record's part answered as ever.
pub fn overview_unbound(not: Not, kept: &Kept, ask: &OverviewAsk) -> Overview {
    Overview {
        branch: ask.branch.clone(),
        asked_at: ask.at,
        channels: Err(not),
        environments: Err(no_target()),
        published: status(kept, &record_ask(ask)),
    }
}

/// 98, 132: of a repository's environments, how many run what they
/// should, how many are behind, how many wait on a person and how many
/// cannot be read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EnvironmentCounts {
    pub converged: usize,
    pub behind: usize,
    pub waiting: usize,
    pub unreadable: usize,
}

/// What the environments' part answers while no target is read.
fn no_target() -> Not {
    Not::unreadable(
        &Source::new(Seam::Target, "unread"),
        "the environments: what each environment should run and runs is read from a target, and no target is read",
        None,
    )
}

/// 58, 60, 98: the environments' part of status, unreadable naming the
/// environments, since what each should run and runs is read from a
/// target and no target is read. It asks no target of `seams`, however
/// many are bound, and the policy lists no environment, so nothing is
/// counted: a count of none would read as none behind.
pub fn environments(_seams: &Seams, _ask: &OverviewAsk) -> Answer<EnvironmentCounts> {
    Err(no_target())
}

/// 132: one repository's row: every channel's head, or the channels'
/// part's own answer where it gives none, and its environments' counts,
/// or the answer given in their place.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Row {
    pub repository: RepositoryName,
    #[serde(serialize_with = "one_answer")]
    pub heads: Answer<Vec<Head>>,
    #[serde(serialize_with = "one_answer")]
    pub environments: Answer<EnvironmentCounts>,
}

/// One channel's head in a row: its component, its name, and the version
/// it is at, empty, or its chain's own answer (58).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Head {
    pub component: ComponentName,
    pub channel: ChannelName,
    #[serde(serialize_with = "one_answer")]
    pub at: Answer<HeadAt>,
}

/// 31: where a channel's head puts it: empty, or at a version and its
/// index.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum HeadAt {
    Empty,
    At { version: Version, index: Digest },
}

/// 132: every repository the host knows, as of `asked_at`, each row its
/// own answer, ordered by owner and name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Repositories {
    pub asked_at: u64,
    #[serde(serialize_with = "every_answer")]
    pub repositories: Vec<Answer<Row>>,
}

/// 57: a list of answers, each in the one form every surface renders.
fn every_answer<S: Serializer>(answers: &[Answer<Row>], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(answers.iter().map(answer_json))
}

/// 132: `repository`'s row, made from `overview` alone: each channel's
/// head, its absent, refused or unreadable chain that head's own answer,
/// or the channels' part's answer in place of the heads; and the
/// environments as the overview answers them.
pub fn row(repository: &RepositoryName, overview: &Overview) -> Row {
    Row {
        repository: repository.clone(),
        heads: overview.channels.as_ref().map_err(Not::clone).map(|c| {
            c.channels
                .iter()
                .map(|channel| Head {
                    component: channel.declared.channel.component_name(),
                    channel: channel.declared.channel.name.clone(),
                    at: channel.at.as_ref().map_err(Not::clone).map(|at| match at {
                        Channel::Empty { .. } => HeadAt::Empty,
                        Channel::At { head, .. } => HeadAt::At {
                            version: head.to.version.clone(),
                            index: head.to.index.clone(),
                        },
                    }),
                })
                .collect()
        }),
        environments: overview.environments.clone(),
    }
}
