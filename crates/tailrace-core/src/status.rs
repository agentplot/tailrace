//! `tailrace status`: where each channel the policy declares is and the
//! chain of promotions that put it there, read from the ledger on any
//! machine, with the notes of every version the chain moved it to, its
//! changelog, rendered again from the repository's own history as a
//! rebuild recovers them; beside which versions of which components this
//! host published, from which revisions, at which index, answered from
//! the host's record alone, each with its notes rendered again from the
//! repository's own history (31, 33, 36, 56 to 61, 63, 101, 125, 131,
//! 132, 134; design D1, D2, D7).
//!
//! Status asks the landing, the policy and the ledger alone, the host's
//! record, and the history its notes are rendered from: it reaches no
//! store, signer, target or projection, and it writes nothing (60, rule
//! core.32; design D9). Every reader of one ledger and one history is
//! answered one version per channel, and the same notes for each version
//! its chain moved it to, as of when it asked, and a chain the ledger
//! answers as last read says when (36, 61). Each part, each channel and
//! each version's notes carries its own answer, so absent, refused and
//! unreadable are kept apart at every depth (58, 59), and every surface
//! renders the one answer made here (57, rule core.31).
//!
//! Beside the channels and the record, the environments are a part of
//! their own, answered unreadable while no target is read, never empty or
//! a count of none, and asking no target (58, 60, 98). One repository's
//! row of every channel's head and its environments' counts is made from
//! its status answer alone, so the host's serving process shows every
//! repository it knows together from the one core (132; design D14).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize, Serializer};
use serde_json::json;
use tailrace_seams::{
    Answer, Branch, ChannelName, Component, ComponentName, DeclaredChannel, Digest, Landing, Not,
    Read, Revision, Seam, Seams, Source, Version,
};

use crate::{
    answer_json, channel, channels_at, notes, Channel, Declared, Entry, Filter, History, Kept,
    Landed, Noted, RepositoryName, Tag, LOCAL, POLICY_FILE,
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Status {
    pub record: String,
    pub components: Vec<Versions>,
}

/// One component's versions this host published.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Versions {
    pub component: ComponentName,
    pub versions: Vec<Shown>,
}

/// 134: a version as status shows it: its entry, and, where the entry
/// names the commits its notes were rendered from, those notes rendered
/// again, or why they cannot be (design D7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shown {
    pub entry: Entry,
    pub notes: Option<Answer<String>>,
}

/// The entry's fields, with `notes` the notes in the one answer form (57)
/// where the entry names any; the commits they were rendered from are
/// the record's file's to tell.
impl Serialize for Shown {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut value = json!(self.entry);
        if let Some(notes) = &self.notes {
            value["notes"] = answer_json(notes);
        }
        value.serialize(serializer)
    }
}

/// 60: the versions `ask` names, read from `kept` and nothing else, each
/// with its notes rendered from `history`'s commits; absent when the
/// record holds none, unreadable naming an entry that cannot be read
/// (58). One version's notes that cannot be read leave every version
/// answered (rule core.19).
pub fn status(history: &History, kept: &Kept, ask: &StatusAsk) -> Answer<Status> {
    let entries = kept.read(&Filter {
        component: ask.component.clone(),
        revision: ask.revision.clone(),
    })?;
    let mut components: Vec<Versions> = Vec::new();
    for entry in entries {
        let shown = Shown {
            notes: entry
                .notes
                .as_ref()
                .map(|noted| notes_of(history, &entry, noted)),
            entry,
        };
        match components.last_mut() {
            Some(last) if last.component == shown.entry.component => last.versions.push(shown),
            _ => components.push(Versions {
                component: shown.entry.component.clone(),
                versions: vec![shown],
            }),
        }
    }
    Ok(Status {
        record: kept.dir().display().to_string(),
        components,
    })
}

/// 134: `entry`'s notes, the commits `noted` names read from the local
/// history and rendered as a publish renders them, under the template its
/// revision holds (134; design D7); unreadable naming a commit the history
/// no longer holds (58).
fn notes_of(history: &History, entry: &Entry, noted: &Noted) -> Answer<String> {
    let rendered = notes::render(
        history,
        &entry.component,
        &entry.version,
        &entry.tag,
        &entry.revision,
        noted,
    )?;
    Ok(rendered.text)
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

/// 31, 101, 134: one channel as the policy declares it, where its chain
/// in the ledger puts it, its own answer (58), and its changelog: the
/// notes of every version its chain moved it to, by version and index,
/// oldest first as the chain is held, none where the channel is empty or
/// its chain is not answered (33; design D2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ChannelStatus {
    #[serde(flatten)]
    pub declared: DeclaredChannel,
    #[serde(serialize_with = "one_answer")]
    pub at: Answer<Channel>,
    pub notes: Vec<Logged>,
}

/// 134: the notes of one version a channel's chain moved it to, each
/// promotion to it shown with them: rendered again from the history, or
/// why they cannot be, never absent, since the version's index carries
/// them (58; design D2, D3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Logged {
    pub version: Version,
    pub index: Digest,
    #[serde(serialize_with = "one_answer")]
    pub notes: Answer<String>,
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

/// 31, 101, 125, 131, 134: every channel the policy at the branch's
/// landed head declares, of `ask.component` where it names one, each with
/// its chain as the ledger answers it and the notes of every version the
/// chain moved it to. The head not landed is refused before the policy
/// is read (65, 77), and refused as not landed as last fetched where the
/// landing read that from its copy; a head landed as last fetched is
/// answered as one landed on the remote, saying so. A channel's absent,
/// refused or unreadable chain is that channel's answer, never the whole
/// one's (58, 105), and a version's notes that cannot be rendered are
/// that version's answer, never the channel's (rule core.19; design D3).
pub fn channels(seams: &Seams, history: &History, ask: &OverviewAsk) -> Answer<Channels> {
    let head = history.head(&ask.branch)?;
    let landed = Landed::check(seams.landing.as_ref(), &ask.branch, &head)?;
    let declared = channels_at(seams.policy.as_ref(), &landed)?;
    let mut changelog = Changelog {
        landing: seams.landing.as_ref(),
        history,
        head: &landed,
        components: Declared::read(history, &landed).map(|d| d.components),
        tags: BTreeMap::new(),
        rendered: BTreeMap::new(),
    };
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
            .map(|declared| {
                let at = channel(seams.keeper.as_ref(), &declared.channel);
                let notes = match &at {
                    Ok(Channel::At { chain, .. }) => {
                        let component = declared.channel.component_name();
                        let mut logged: Vec<Logged> = Vec::new();
                        for (_, promotion) in chain {
                            let to = &promotion.to;
                            if logged
                                .iter()
                                .any(|l| l.version == to.version && l.index == to.index)
                            {
                                continue;
                            }
                            logged.push(Logged {
                                version: to.version.clone(),
                                index: to.index.clone(),
                                notes: changelog.notes(&component, &to.version),
                            });
                        }
                        logged
                    }
                    Ok(Channel::Empty { .. }) | Err(_) => Vec::new(),
                };
                ChannelStatus {
                    declared,
                    at,
                    notes,
                }
            })
            .collect(),
    })
}

/// 134: what one ask of the channels reads toward their notes: the
/// components the policy file at the landed head declares, read once,
/// each component's tags that head reaches, read once, and each
/// component's version's notes, rendered once and shared by every
/// channel that moved to it (design D1, D2).
struct Changelog<'a> {
    landing: &'a dyn Landing,
    history: &'a History,
    head: &'a Landed,
    components: Answer<Vec<Component>>,
    tags: BTreeMap<ComponentName, Answer<Vec<Tag>>>,
    rendered: BTreeMap<(ComponentName, Version), Answer<String>>,
}

impl Changelog<'_> {
    /// The notes of `component` at `version`, rendered on the first ask
    /// and the same answer on every ask after; unreadable naming them
    /// where the policy file at the landed head declares no such
    /// component, one renamed since, and the file's own answer where it
    /// cannot be read (design D1).
    fn notes(&mut self, component: &ComponentName, version: &Version) -> Answer<String> {
        let key = (component.clone(), version.clone());
        if let Some(rendered) = self.rendered.get(&key) {
            return rendered.clone();
        }
        let what = format!("the notes of {component} {version}");
        let rendered = match &self.components {
            Err(not) => Err(not.clone()),
            Ok(components) => match components.iter().find(|c| c.name == *component) {
                None => Err(Not::unreadable(
                    &Source::history(LOCAL),
                    &what,
                    Some(format!(
                        "{POLICY_FILE} at {} declares no component {component}",
                        self.head.revision()
                    )),
                )),
                Some(declared) => {
                    let tags = self
                        .tags
                        .entry(component.clone())
                        .or_insert_with(|| self.history.tags(self.head, &declared.prefix));
                    tagged(
                        self.landing,
                        self.history,
                        self.head,
                        tags,
                        declared,
                        version,
                    )
                }
            },
        }
        .map_err(|not| never_absent(&what, not));
        self.rendered.insert(key, rendered.clone());
        rendered
    }
}

/// 134: the notes of `component` at `version`, rendered again from git
/// as a rebuild recovers the commits they were rendered from: its tag
/// among `tags`, those the landed `head` reaches, its revision landed on
/// `head`'s branch, the commits the version rule derives there under the
/// policy file there, rendered by the rule's tool under the template
/// there (21, 63; design D1). A tag the history does not hold is
/// unreadable naming it; a revision not landed, a commit the history no
/// longer holds and the tool's words on the template are as the landing,
/// [`notes::noted`] and [`notes::render`] answer them (58; design D3).
fn tagged(
    landing: &dyn Landing,
    history: &History,
    head: &Landed,
    tags: &Answer<Vec<Tag>>,
    component: &Component,
    version: &Version,
) -> Answer<String> {
    let name = format!("{}{version}", component.prefix);
    let tag = tags
        .as_ref()
        .map_err(Not::clone)?
        .iter()
        .find(|t| t.name == name)
        .ok_or_else(|| {
            Not::unreadable(
                &Source::history(LOCAL),
                format!("the notes of {} {version}", component.name),
                Some(format!(
                    "the history holds no tag {name} reached from {}",
                    head.revision()
                )),
            )
        })?;
    let at = Landed::check(landing, head.branch(), &tag.revision)?;
    let noted = notes::noted(history, &at, &component.name, tag)?;
    let rendered = notes::render(
        history,
        &component.name,
        version,
        &tag.name,
        &tag.revision,
        &noted,
    )?;
    Ok(rendered.text)
}

/// A version's notes never answered absent: the version is in a chain,
/// so its index carries them, and only this host's history cannot render
/// them; an absent answer on the way is unreadable naming the notes,
/// with what was absent (58; design D3).
fn never_absent(what: &str, not: Not) -> Not {
    match not {
        Not::Absent(a) => {
            let source = a.source.clone();
            Not::unreadable(&source, what, Some(Not::Absent(a).to_string()))
        }
        not => not,
    }
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
        published: status(history, kept, &record_ask(ask)),
    }
}

/// The whole of status where no binding could be made, for `not`: the
/// channels' part that answer, the environments' as ever unread, and the
/// record's part answered as ever.
pub fn overview_unbound(not: Not, history: &History, kept: &Kept, ask: &OverviewAsk) -> Overview {
    Overview {
        branch: ask.branch.clone(),
        asked_at: ask.at,
        channels: Err(not),
        environments: Err(no_target()),
        published: status(history, kept, &record_ask(ask)),
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
