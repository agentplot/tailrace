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

use serde::{Deserialize, Serialize, Serializer};
use tailrace_seams::{
    Answer, Branch, ChannelName, ComponentName, DeclaredChannel, Not, Revision, Seams,
};

use crate::{answer_json, channel, channels_at, Channel, Entry, Filter, History, Kept, Landed};

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
    pub published: Answer<Status>,
}

/// 131: the channels the policy declares at `revision`, the branch's
/// landed head, in the policy seam's order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Channels {
    pub revision: Revision,
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
        &self.declared.name
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
/// the policy is read (65, 77). A channel's absent, refused or unreadable
/// chain is that channel's answer, never the whole one's (58, 105).
pub fn channels(seams: &Seams, history: &History, ask: &OverviewAsk) -> Answer<Channels> {
    let head = history.head(&ask.branch)?;
    let landed = Landed::check(seams.landing.as_ref(), &ask.branch, &head)?;
    let declared = channels_at(seams.policy.as_ref(), &landed)?;
    Ok(Channels {
        revision: head,
        channels: declared
            .into_iter()
            .filter(|c| {
                ask.component
                    .as_ref()
                    .is_none_or(|named| c.component == *named)
            })
            .map(|declared| ChannelStatus {
                at: channel(seams.keeper.as_ref(), &declared.name),
                declared,
            })
            .collect(),
    })
}

/// 36, 60: the whole of status, each part its own answer: the channels
/// over `seams`, and what this host published from `kept`.
pub fn overview(seams: &Seams, history: &History, kept: &Kept, ask: &OverviewAsk) -> Overview {
    Overview {
        branch: ask.branch.clone(),
        asked_at: ask.at,
        channels: channels(seams, history, ask),
        published: status(kept, &record_ask(ask)),
    }
}

/// The whole of status where no binding could be made, for `not`: the
/// channels' part that answer, and the record's part answered as ever.
pub fn overview_unbound(not: Not, kept: &Kept, ask: &OverviewAsk) -> Overview {
    Overview {
        branch: ask.branch.clone(),
        asked_at: ask.at,
        channels: Err(not),
        published: status(kept, &record_ask(ask)),
    }
}
