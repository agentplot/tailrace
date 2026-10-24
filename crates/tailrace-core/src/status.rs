//! `tailrace status`: which versions of which components this host
//! published, from which revisions, at which index, answered from the
//! host's record alone. It is handed no binding, so it reaches no store,
//! signer or target, and it writes nothing (60, 61, rule core.32; design
//! D9).

use serde::{Deserialize, Serialize};
use tailrace_seams::{Answer, ComponentName, Revision};

use crate::{Entry, Filter, Kept};

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
