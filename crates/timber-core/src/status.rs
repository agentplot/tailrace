//! `timber status`: where each channel the policy declares is and the
//! chain of promotions that put it there, read from the ledger on any
//! machine, with the notes of every version the chain moved it to, its
//! changelog, rendered again from the repository's own history as a
//! rebuild recovers them; beside which versions of which components this
//! host published, from which revisions, at which index, answered from
//! the host's record alone, each with its notes rendered again from the
//! repository's own history (31, 33, 36, 56 to 61, 63, 101, 125, 131,
//! 132, 134; design D1, D2, D7).
//!
//! Status asks the landing, the policy and the ledger, the host's
//! record, and the history its notes are rendered from; it reaches the
//! store only to fetch a chain a store ledger holds, or the reports
//! attached to the indexes an environment's chain moved to, into the
//! host's copy, and it writes nothing but that copy (60, 61, rule
//! core.32). It asks no target and no projection, and tells each report
//! through the signer profiles bound, which status binds to verify
//! reaching no service (60, C.7). Every reader of one ledger, one store
//! and one history is answered one version per channel, and the same
//! notes for each version its chain moved it to, as of when it asked, and
//! a chain or reports read as last fetched say when (36, 61). Each part,
//! each channel, each environment and each version's notes carries its
//! own answer, so absent, refused and unreadable are kept apart at every
//! depth (58, 59), and every surface renders the one answer made here
//! (57, rule core.31).
//!
//! Beside the channels and the record, the environments are a part of
//! their own: every environment the policy declares at the landed head,
//! with what it should run, its channel's head as the channels part reads
//! it, what it runs, its newest counted report, its state and what its
//! channel moves on, beside one that is behind the notes of every version
//! between what it runs and what it should, rendered once per ask with
//! the channels' changelogs, and four counts beside them (98, 132, 134,
//! 136; design D2, D6 to D9). The landed head, the policy and each chain are read
//! once, and both parts are made from them, so they never disagree on a
//! head (rule core.50). One repository's row of every channel's head and
//! its environments' counts is made from its status answer alone, so the
//! host's serving process shows every repository it knows together from
//! the one core (132).

use std::cmp::Ordering;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize, Serializer};
use serde_json::json;
use timber_seams::{
    Answer, Branch, ChannelId, ChannelName, Component, ComponentName, DeclaredChannel,
    DeclaredEnvironment, Digest, EnvironmentName, Landing, Moves, NamedSigner, Not, Promotion,
    Read, Revision, Seam, Seams, SignerId, Source, Version,
};

use crate::{
    answer_json, channel, channels_at, newest_of, notes, reporters, Channel, Declared, Entry,
    Fetch, Filter, History, Kept, Landed, Noted, ReportsCopy, RepositoryName, Tag, BOUND, LOCAL,
    POLICY_FILE,
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
/// from the ledger, the environments, read from their channels' chains
/// and their reports, and what this host published, from its record,
/// each part its own answer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Overview {
    pub branch: Branch,
    pub asked_at: u64,
    #[serde(serialize_with = "one_answer")]
    pub channels: Answer<Channels>,
    #[serde(serialize_with = "one_answer")]
    pub environments: Answer<Environments>,
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

/// What one ask reads once, which both the channels and the environments
/// are made from (design D2): the branch's head, landed, and the channels
/// the policy lists at it of the component asked, each with its chain as
/// the ledger answers it.
struct Heads {
    head: Revision,
    landed: Landed,
    listed: Vec<(DeclaredChannel, Answer<Channel>)>,
}

impl Heads {
    /// The chain of `channel` as this ask read it, `None` where the
    /// policy lists no such channel.
    fn of(&self, channel: &ChannelId) -> Option<&(DeclaredChannel, Answer<Channel>)> {
        self.listed.iter().find(|(d, _)| d.channel == *channel)
    }
}

/// 31, 101, 125, 131: the branch's head, landed, and every channel the
/// policy at it declares, of `ask.component` where it names one, each with
/// its chain as the ledger answers it, read once. The head not landed is
/// refused before the policy is read (65, 77), and refused as not landed
/// as last fetched where the landing read that from its copy; a head
/// landed as last fetched is answered as one landed on the remote, saying
/// so. A channel's absent, refused or unreadable chain is that channel's
/// answer, never the whole one's (58, 105).
fn heads(seams: &Seams, history: &History, ask: &OverviewAsk) -> Answer<Heads> {
    let head = history.head(&ask.branch)?;
    let landed = Landed::check(seams.landing.as_ref(), &ask.branch, &head)?;
    let listed = channels_at(seams.policy.as_ref(), &landed)?
        .into_iter()
        .filter(|c| {
            ask.component
                .as_ref()
                .is_none_or(|named| c.channel.component_name() == *named)
        })
        .map(|declared| {
            let at = channel(seams.keeper.as_ref(), &declared.channel);
            (declared, at)
        })
        .collect();
    Ok(Heads {
        head,
        landed,
        listed,
    })
}

/// 31, 101, 134: every channel `heads` read, each with its chain and the
/// notes of every version the chain moved it to, oldest first and each
/// version once; a version's notes that cannot be rendered are that
/// version's answer, never the channel's (rule core.19; design D3).
fn channels_part(heads: &Heads, changelog: &mut Changelog<'_>) -> Channels {
    Channels {
        revision: heads.head.clone(),
        landing: heads.landed.read().clone(),
        channels: heads
            .listed
            .iter()
            .map(|(declared, at)| {
                let notes = match at {
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
                    declared: declared.clone(),
                    at: at.clone(),
                    notes,
                }
            })
            .collect(),
    }
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

impl<'a> Changelog<'a> {
    /// What one ask renders its notes with, nothing read yet but the
    /// components the policy file at the landed `head` declares.
    fn new(landing: &'a dyn Landing, history: &'a History, head: &'a Landed) -> Changelog<'a> {
        Changelog {
            landing,
            history,
            head,
            components: Declared::read(history, head).map(|d| d.components),
            tags: BTreeMap::new(),
            rendered: BTreeMap::new(),
        }
    }

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
        let rendered = self
            .declared(component, &what)
            .and_then(|declared| {
                let tags = self.tags_of(&declared);
                tagged(
                    self.landing,
                    self.history,
                    self.head,
                    &tags,
                    &declared,
                    version,
                )
            })
            .map_err(|not| never_absent(&what, not));
        self.rendered.insert(key, rendered.clone());
        rendered
    }

    /// `component` as the policy file at the landed head declares it;
    /// unreadable naming `what` where it declares none of that name, and
    /// the file's own answer where it cannot be read.
    fn declared(&self, component: &ComponentName, what: &str) -> Answer<Component> {
        match &self.components {
            Err(not) => Err(not.clone()),
            Ok(components) => components
                .iter()
                .find(|c| c.name == *component)
                .cloned()
                .ok_or_else(|| {
                    Not::unreadable(
                        &Source::history(LOCAL),
                        what,
                        Some(format!(
                            "{POLICY_FILE} at {} declares no component {component}",
                            self.head.revision()
                        )),
                    )
                }),
        }
    }

    /// The tags of `component` the landed head reaches, read on the
    /// first ask and the same answer on every ask after.
    fn tags_of(&mut self, component: &Component) -> Answer<Vec<Tag>> {
        self.tags
            .entry(component.name.clone())
            .or_insert_with(|| self.history.tags(self.head, &component.prefix))
            .clone()
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
/// and the environments over `seams`, made from one read of the landed
/// head, the policy and each chain, the environments' reports fetched
/// into the host's copy under `history`'s common directory ([`ReportsCopy`]),
/// and what this host published from `kept`. Where the landed head or the
/// policy's channels cannot be read, both parts are that answer.
pub fn overview(seams: &Seams, history: &History, kept: &Kept, ask: &OverviewAsk) -> Overview {
    let copy = ReportsCopy::at(history);
    let (channels, environments) = match heads(seams, history, ask) {
        Err(not) => (Err(not.clone()), Err(not)),
        Ok(heads) => {
            let mut changelog = Changelog::new(seams.landing.as_ref(), history, &heads.landed);
            let channels = channels_part(&heads, &mut changelog);
            let environments = environments(seams, ask, &heads, &copy, &mut changelog);
            (Ok(channels), environments)
        }
    };
    Overview {
        branch: ask.branch.clone(),
        asked_at: ask.at,
        channels,
        environments,
        published: status(history, kept, &record_ask(ask)),
    }
}

/// The whole of status where no binding could be made, for `not`: the
/// channels' and the environments' parts that answer, and the record's
/// part answered as ever.
pub fn overview_unbound(not: Not, history: &History, kept: &Kept, ask: &OverviewAsk) -> Overview {
    Overview {
        branch: ask.branch.clone(),
        asked_at: ask.at,
        channels: Err(not.clone()),
        environments: Err(not),
        published: status(history, kept, &record_ask(ask)),
    }
}

/// 98, 132: of a repository's environments, how many run what they
/// should, how many are behind, how many wait on a person and how many
/// cannot be read, each environment in exactly one.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct EnvironmentCounts {
    pub converged: usize,
    pub behind: usize,
    pub waiting: usize,
    pub unreadable: usize,
}

/// 98, 132, 136: every environment the policy declares at the landed
/// head, of the component asked, in the policy seam's order, with the four
/// counts of them flattened beside the list, so the part and a
/// repository's row read the same keys (57; design D9).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Environments {
    #[serde(flatten)]
    pub counts: EnvironmentCounts,
    pub environments: Vec<EnvironmentStatus>,
}

/// 41, 97, 98, 132, 134: one environment as status shows it: its name,
/// its component, the channel it follows, whether it waits and on whom,
/// what its channel moves on, what it should run, what it runs, its state
/// and, where it is behind, the versions between, each its own answer
/// (design D9).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EnvironmentStatus {
    pub environment: EnvironmentName,
    pub component: ComponentName,
    pub channel: ChannelName,
    pub waits: bool,
    pub approvers: Vec<SignerId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moves_on: Option<MovesOn>,
    #[serde(serialize_with = "one_answer")]
    pub should: Answer<HeadAt>,
    #[serde(serialize_with = "one_answer")]
    pub runs: Answer<Runs>,
    #[serde(serialize_with = "one_answer")]
    pub state: Answer<State>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "some_answer"
    )]
    pub between: Option<Answer<Between>>,
}

/// 37, 41, 97, 132: what an environment's channel moves on, where it
/// does not move on its own (design D7).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "by", rename_all = "snake_case")]
pub enum MovesOn {
    /// It waits: its channel moves to what `bound_to` is at only when one
    /// of its approvers asks (41, 97); `at` is that channel's head as
    /// this status read it.
    Asking {
        bound_to: ChannelName,
        #[serde(serialize_with = "one_answer")]
        at: Answer<HeadAt>,
    },
    /// It comes after `after`: its channel moves on their reports (37,
    /// C.10).
    Reports { after: Vec<EnvironmentName> },
}

/// 43, 91, 98: what an environment runs, its newest counted report: the
/// version and index it reports, the time the target says it converged,
/// the moment of the apply that reported it, the report claim's digest,
/// the signer that made it, and how its reports were read, from the store
/// or from the host's copy as last fetched (60; design D3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Runs {
    pub version: Version,
    pub index: Digest,
    pub converged_at: String,
    pub at: u64,
    pub claim: Digest,
    pub signer: SignerId,
    pub read: Read,
}

/// 98: an environment's state where what it should run and what it runs
/// are answered: running what it should, behind its channel's head, or
/// waiting on a person (design D6). One that cannot be read is the
/// unreadable answer in its place, carrying the reason (58).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Converged,
    Behind,
    Waiting,
}

/// 23, 134: the versions between what an environment that is behind
/// runs and what it should, `ahead` where what it should run is higher by
/// semantic precedence and `back` where it is lower: the component's
/// releases the landed head reaches above the lower and at or below the
/// higher, with the higher where it is a prerelease, ascending, each with
/// its notes (design D8). One behind on the same version at another index
/// is `ahead` with none.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Between {
    pub direction: Direction,
    pub versions: Vec<VersionBetween>,
}

/// Which way an environment that is behind moves to what it should run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Ahead,
    Back,
}

/// 134: one version between, by its version and its tag's revision,
/// since status resolves no name in the store (60), with its notes
/// rendered from the landed history as the channels' changelog renders
/// them, or why they cannot be (rule core.19).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct VersionBetween {
    pub version: Version,
    pub revision: Revision,
    #[serde(serialize_with = "one_answer")]
    pub notes: Answer<String>,
}

/// 57: an answer given only where there is one, in the one form.
fn some_answer<T: Serialize, S: Serializer>(
    answer: &Option<Answer<T>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match answer {
        Some(answer) => answer_json(answer).serialize(serializer),
        None => serializer.serialize_none(),
    }
}

/// 31: where `channel`'s head puts it.
fn head_at(channel: &Channel) -> HeadAt {
    match channel {
        Channel::Empty { .. } => HeadAt::Empty,
        Channel::At { head, .. } => HeadAt::At {
            version: head.to.version.clone(),
            index: head.to.index.clone(),
        },
    }
}

fn store() -> Source {
    Source::new(Seam::Store, BOUND)
}

/// 98, 136: the environments' part: every environment the policy declares
/// at the landed head `heads` read, of `ask.component` where it names one,
/// each answered from the chains `heads` holds and its reports, fetched
/// once per index into `copy` ([`Fetch`]) and counted through the signers
/// the policy lets claim a report (116); the policy's own answer where it
/// cannot list its environments, and an empty list with four counts of
/// none where it declares none (58).
fn environments(
    seams: &Seams,
    ask: &OverviewAsk,
    heads: &Heads,
    copy: &ReportsCopy,
    changelog: &mut Changelog<'_>,
) -> Answer<Environments> {
    let declared: Vec<DeclaredEnvironment> = seams
        .policy
        .environments(heads.landed.revision())?
        .into_iter()
        .filter(|e| {
            ask.component
                .as_ref()
                .is_none_or(|named| e.channel.component_name() == *named)
        })
        .collect();
    let reporters = if declared.is_empty() {
        Ok(Vec::new())
    } else {
        reporters(seams, &heads.landed)
    };
    let mut fetch = Fetch::new(seams.store.as_ref(), copy, ask.at);
    let environments: Vec<EnvironmentStatus> = declared
        .iter()
        .map(|declared| environment(seams, heads, &reporters, &mut fetch, changelog, declared))
        .collect();
    let mut counts = EnvironmentCounts::default();
    for environment in &environments {
        match environment.state {
            Ok(State::Converged) => counts.converged += 1,
            Ok(State::Behind) => counts.behind += 1,
            Ok(State::Waiting) => counts.waiting += 1,
            Err(_) => counts.unreadable += 1,
        }
    }
    Ok(Environments {
        counts,
        environments,
    })
}

/// 40, 41, 88, 97, 98, 134: one environment `declared`: what it should
/// run, its channel's head as `heads` read it (rule core.50); what it
/// runs, its newest counted report across that chain's indexes, as last
/// read included; what its channel moves on; its state from them; and,
/// where it is behind, the versions between with their notes from
/// `changelog`.
fn environment(
    seams: &Seams,
    heads: &Heads,
    reporters: &Answer<Vec<NamedSigner>>,
    fetch: &mut Fetch<'_>,
    changelog: &mut Changelog<'_>,
    declared: &DeclaredEnvironment,
) -> EnvironmentStatus {
    let channel = &declared.channel;
    let listed = heads.of(channel);
    let should = match listed {
        None => Err(unlisted(heads, channel)),
        Some((_, at)) => at.as_ref().map(head_at).map_err(Not::clone),
    };
    let runs = match listed {
        None => Err(unlisted(heads, channel)),
        Some((_, Err(not))) => Err(not.clone()),
        Some((_, Ok(Channel::Empty { .. }))) => Err(empty(declared)),
        Some((_, Ok(Channel::At { chain, .. }))) => runs(seams, reporters, fetch, declared, chain),
    };
    let moves_on = moves_on(heads, declared, listed.map(|(d, _)| d));
    let state = state(declared, &should, &runs, moves_on.as_ref());
    let between = match (&state, &should, &runs) {
        (Ok(State::Behind), Ok(HeadAt::At { version, .. }), Ok(runs)) => Some(between(
            changelog,
            &channel.component_name(),
            version,
            &runs.version,
        )),
        _ => None,
    };
    EnvironmentStatus {
        environment: declared.name.clone(),
        component: channel.component_name(),
        channel: channel.name.clone(),
        waits: declared.waits,
        approvers: declared.approvers.clone(),
        moves_on,
        should,
        runs,
        state,
        between,
    }
}

/// 21, 23, 134: the versions between `runs` and `should` of `component`
/// (design D8): which way it moves, and the releases of the component the
/// landed head reaches above the lower of the two and at or below the
/// higher by semantic precedence, with the higher where it is a
/// prerelease, ascending, so their notes cover every commit between and
/// none twice; each with its notes from `changelog`, rendered once per
/// status and shared with the channels' changelogs, a version's notes
/// that cannot be rendered its own answer (rule core.19). Unreadable,
/// never empty, naming what stopped it, where the component's tags
/// cannot be read or either end is no semantic version (58).
fn between(
    changelog: &mut Changelog<'_>,
    component: &ComponentName,
    should: &Version,
    runs: &Version,
) -> Answer<Between> {
    let what = format!("the versions of {component} between {runs} and {should}");
    let parse = |version: &Version| {
        semver::Version::parse(version.as_str()).map_err(|e| {
            Not::unreadable(
                &Source::history(LOCAL),
                &what,
                Some(format!("{version} is no semantic version: {e}")),
            )
        })
    };
    let (higher, lower, direction) = match (parse(should)?, parse(runs)?) {
        (should, runs) if should.cmp_precedence(&runs) == Ordering::Less => {
            (runs, should, Direction::Back)
        }
        (should, runs) => (should, runs, Direction::Ahead),
    };
    let declared = changelog
        .declared(component, &what)
        .map_err(|not| never_absent(&what, not))?;
    let tags = changelog
        .tags_of(&declared)
        .map_err(|not| never_absent(&what, not))?;
    let mut tags: Vec<&Tag> = tags
        .iter()
        .filter(|t| {
            t.version.cmp_precedence(&lower) == Ordering::Greater
                && t.version.cmp_precedence(&higher) != Ordering::Greater
                && (t.version.pre.is_empty()
                    || t.version.cmp_precedence(&higher) == Ordering::Equal)
        })
        .collect();
    tags.sort_by(|a, b| a.version.cmp_precedence(&b.version));
    tags.dedup_by(|a, b| a.version.cmp_precedence(&b.version) == Ordering::Equal);
    let versions = tags
        .into_iter()
        .map(|tag| {
            let version = Version::new(&tag.name[declared.prefix.len()..]);
            VersionBetween {
                notes: changelog.notes(component, &version),
                version,
                revision: tag.revision.clone(),
            }
        })
        .collect();
    Ok(Between {
        direction,
        versions,
    })
}

/// 131: a channel an environment follows that the policy does not list,
/// absent naming it at the landed head.
fn unlisted(heads: &Heads, channel: &ChannelId) -> Not {
    Not::absent(
        &Source::new(Seam::Policy, BOUND),
        format!(
            "channel {} of {} at {}",
            channel.name,
            channel.component_name(),
            heads.landed.revision()
        ),
    )
}

/// 98: what an environment whose channel is empty runs: no counted report
/// names it, and its channel is empty, so nothing ever came to it.
fn empty(declared: &DeclaredEnvironment) -> Not {
    Not::absent(
        &store(),
        format!(
            "a counted report of environment {} of {}: its channel {} is empty",
            declared.name,
            declared.channel.component_name(),
            declared.channel
        ),
    )
}

/// 43, 60, 91, 98: what `declared` runs, its newest report across the
/// indexes `chain` moved to, fetched through `fetch` and counted through
/// `reporters` (design D3, D4).
fn runs(
    seams: &Seams,
    reporters: &Answer<Vec<NamedSigner>>,
    fetch: &mut Fetch<'_>,
    declared: &DeclaredEnvironment,
    chain: &[(Digest, Promotion)],
) -> Answer<Runs> {
    let reporters = reporters.as_ref().map_err(Not::clone)?;
    let indexes: Vec<Digest> = chain.iter().map(|(_, p)| p.to.index.clone()).collect();
    let fetched = fetch.reports(&declared.name, &declared.channel, &indexes)?;
    let counted = newest_of(seams, reporters, &fetched.reports)?;
    let report = counted.newest.report;
    Ok(Runs {
        version: report.version,
        index: report.index,
        converged_at: report.converged_at,
        at: report.at,
        claim: counted.newest.claim,
        signer: counted.signer,
        read: fetched.read,
    })
}

/// 37, 41, 97: what the channel `listed` of `declared` moves on: the
/// environments it comes after, where an `after` made it; for one that
/// waits, the channel its own is bound to with that channel's head as
/// `heads` read it; nothing otherwise (design D7).
fn moves_on(
    heads: &Heads,
    declared: &DeclaredEnvironment,
    listed: Option<&DeclaredChannel>,
) -> Option<MovesOn> {
    match listed.map(|d| &d.moves) {
        Some(Moves::Bound { after, .. }) if !after.is_empty() => Some(MovesOn::Reports {
            after: after.clone(),
        }),
        Some(Moves::Bound { to, .. }) if declared.waits => {
            let bound = ChannelId::new(declared.channel.component.clone(), to.clone());
            let at = match heads.of(&bound) {
                None => Err(unlisted(heads, &bound)),
                Some((_, at)) => at.as_ref().map(head_at).map_err(Not::clone),
            };
            Some(MovesOn::Asking {
                bound_to: to.clone(),
                at,
            })
        }
        _ => None,
    }
}

/// 98, rule core.4: an environment's state, in the order of design D6:
/// unread where what it should run or what it runs is not answered,
/// naming the answer in its place; behind where its newest counted report
/// names another index than its channel's head; waiting where it waits
/// and the channel its own is bound to is at another index than its own,
/// unread naming that channel's answer where its head is not answered;
/// and running what it should otherwise. One that comes after others
/// moves on reports and never waits on a person.
fn state(
    declared: &DeclaredEnvironment,
    should: &Answer<HeadAt>,
    runs: &Answer<Runs>,
    moves_on: Option<&MovesOn>,
) -> Answer<State> {
    let should = should.as_ref().map_err(Not::clone)?;
    let runs = runs.as_ref().map_err(Not::clone)?;
    let HeadAt::At { index, .. } = should else {
        return Err(empty(declared));
    };
    if runs.index != *index {
        return Ok(State::Behind);
    }
    match moves_on {
        Some(MovesOn::Asking { at, .. }) => match at {
            Ok(HeadAt::At { index: bound, .. }) if bound != index => Ok(State::Waiting),
            Ok(_) => Ok(State::Converged),
            Err(not) => Err(not.clone()),
        },
        Some(MovesOn::Reports { .. }) | None => Ok(State::Converged),
    }
}

/// 132: one repository's row: every channel's head, or the channels'
/// part's own answer where it gives none, and its environments' four
/// counts, or the environments' part's own answer in their place.
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
/// environments' part's counts, or its answer where it gives none (58).
pub fn row(repository: &RepositoryName, overview: &Overview) -> Row {
    Row {
        repository: repository.clone(),
        heads: overview.channels.as_ref().map_err(Not::clone).map(|c| {
            c.channels
                .iter()
                .map(|channel| Head {
                    component: channel.declared.channel.component_name(),
                    channel: channel.declared.channel.name.clone(),
                    at: channel.at.as_ref().map_err(Not::clone).map(head_at),
                })
                .collect()
        }),
        environments: overview
            .environments
            .as_ref()
            .map(|e| e.counts.clone())
            .map_err(Not::clone),
    }
}
