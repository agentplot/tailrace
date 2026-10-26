//! `tailrace status`: which versions of which components this host
//! published, from which revisions, at which index, answered from the
//! host's record alone, each with its notes rendered again from the
//! repository's own history (132, 134). It is handed no binding, so it
//! reaches no store, signer or target, and it writes nothing (60, 61,
//! rule core.32; design D7, D9).

use serde::{Deserialize, Serialize, Serializer};
use serde_json::json;
use tailrace_seams::{Answer, ComponentName, Revision};

use crate::{answer_json, notes, Entry, Filter, History, Kept, Noted};

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
