//! `tailrace rebuild`: the host's record of the versions it published,
//! rebuilt from what the store and git hold, so everything kept but the
//! record of claimed versions can be deleted and given back to the same
//! state (61 to 63, rule core.20; design D3). The tags carrying each
//! component's prefix are the record of claimed versions (22); a tag
//! whose revision the branch landed and whose name the store resolves to
//! that component's index at that version is one entry, the same file a
//! publish of it writes. A tag whose name the store does not hold gets no
//! entry, as a publish that stopped before its index keeps none.
//!
//! Every read comes before any write: the landing first, then the policy
//! file, the tags and the store, and an answer that cannot be read, a
//! name at anything but its index among them, is the whole answer, with
//! the source's own words and nothing written (58, 59, rules core.4,
//! core.19). The store is asked only to resolve and read; nothing is
//! signed, and nothing but the record is written. An entry the record
//! holds already is left as it is, held or refused, and nothing is
//! deleted (63).
//!
//! What a rebuilt record holds is a function of the store and git alone:
//! every version the store holds an index for under a tag the branch
//! landed. On a host that is the store's only publisher that is what it
//! kept; where several hosts publish to one store, it holds what the
//! others published too, since the store keeps no word of which host
//! offered an index (60).

use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use tailrace_seams::{
    Answer, Branch, ComponentName, Digest, Held, Manifest, Name, Not, Revision, Seams, Source,
    Version,
};

use crate::{answer_json, Declared, Entry, History, Kept, Landed, Tag, Written};

/// What a rebuild is asked: the landed revision whose policy file
/// declares the components, the branch that landed it, and the store as
/// the host's bindings name it, which each entry keeps (61, 118).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RebuildAsk {
    pub branch: Branch,
    pub revision: Revision,
    pub store: Source,
}

/// 57: what a rebuild answers: the record it wrote, and each entry the
/// store and git give with what writing it answered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rebuilt {
    pub record: String,
    /// Each entry the store and git give, in the order the policy file
    /// declares the components and by the version rules, with what
    /// writing it answered.
    pub entries: Vec<(Entry, Answer<Written>)>,
}

impl Serialize for Rebuilt {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let entries: Vec<Value> = self
            .entries
            .iter()
            .map(|(entry, answer)| {
                let mut value = answer_json(answer);
                value["entry"] = json!(entry);
                value
            })
            .collect();
        json!({ "record": self.record, "entries": entries }).serialize(serializer)
    }
}

/// 63: the record `kept` rebuilt from the store and the history at
/// `ask.revision`. The landing is asked first, and a revision its branch
/// has not landed is refused before anything else is read (77). A tag
/// whose revision the branch did not land, or whose name the store does
/// not hold, or whose index it does not hold, is passed over; every
/// other answer that is not an entry is the whole answer, and nothing is
/// written.
pub fn rebuild(seams: &Seams, history: &History, kept: &Kept, ask: &RebuildAsk) -> Answer<Rebuilt> {
    let landed = Landed::check(seams.landing.as_ref(), &ask.branch, &ask.revision)?;
    let declared = Declared::read(history, &landed)?;
    let mut entries = Vec::new();
    for component in &declared.components {
        let mut tags = history.tags(&landed, &component.prefix)?;
        tags.sort_by(|a, b| {
            a.version
                .cmp_precedence(&b.version)
                .then_with(|| a.version.cmp(&b.version))
        });
        for tag in tags {
            match seams.landing.landed(&ask.branch, &tag.revision) {
                Ok(true) => {}
                Ok(false) | Err(Not::Absent(_)) => continue,
                Err(not) => return Err(not),
            }
            let index = match seams.store.resolve(&Name::new(&tag.name)) {
                Ok(index) => index,
                Err(Not::Absent(_)) => continue,
                Err(not) => return Err(not),
            };
            let held = match seams.store.read(&index) {
                Ok(held) => held,
                Err(Not::Absent(_)) => continue,
                Err(not) => return Err(not),
            };
            let version = Version::new(tag.version.to_string());
            indexed(ask, &component.name, &version, &tag, &index, held)?;
            entries.push(Entry {
                component: component.name.clone(),
                version,
                tag: tag.name,
                revision: tag.revision,
                branch: ask.branch.clone(),
                index,
                store: ask.store.clone(),
            });
        }
    }
    let entries = entries
        .into_iter()
        .map(|entry| {
            let written = kept.write(&entry);
            (entry, written)
        })
        .collect();
    Ok(Rebuilt {
        record: kept.dir().display().to_string(),
        entries,
    })
}

/// Whether `held`, what the digest `tag`'s name resolves to holds, is
/// the index of `component` at `version`; unreadable naming the tag and
/// the digest when it is anything else (rule core.4).
fn indexed(
    ask: &RebuildAsk,
    component: &ComponentName,
    version: &Version,
    tag: &Tag,
    index: &Digest,
    held: Held,
) -> Answer<()> {
    let what = format!(
        "the index of {component} {version} that {} names, {index}",
        tag.name
    );
    let holds = match held {
        Held::Manifest(Manifest::Index(meta))
            if meta.component() == component && meta.version() == version =>
        {
            return Ok(())
        }
        Held::Manifest(Manifest::Index(meta)) => {
            format!("the index of {} {}", meta.component(), meta.version())
        }
        Held::Manifest(Manifest::Artifact { .. }) => "an artifact's manifest".to_string(),
        Held::Bytes(_) => "bytes".to_string(),
    };
    Err(Not::unreadable(
        &ask.store,
        what,
        Some(format!("{index} holds {holds}")),
    ))
}
