//! What this host keeps of the versions it published: one file per
//! version under the repository's common git directory, which every
//! worktree of the repository on the host shares and no push carries
//! (61, 62, rule core.20; design D9). It holds nothing the store or git
//! holds besides, and no credential (64, rule core.21).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tailrace_seams::{Answer, Branch, ComponentName, Digest, Not, Revision, Source, Version, Why};

use crate::History;

/// 61: the record of one version this host published: the component,
/// the version and its tag, the revision and branch it was published
/// from, the index's digest and the store it was offered to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub component: ComponentName,
    pub version: Version,
    pub tag: String,
    pub revision: Revision,
    pub branch: Branch,
    pub index: Digest,
    pub store: Source,
}

/// Which entries a read keeps: those of one component, of one revision,
/// or both; every entry where neither is given.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Filter {
    pub component: Option<ComponentName>,
    pub revision: Option<Revision>,
}

impl Filter {
    fn keeps(&self, entry: &Entry) -> bool {
        self.component
            .as_ref()
            .is_none_or(|c| *c == entry.component)
            && self.revision.as_ref().is_none_or(|r| *r == entry.revision)
    }
}

/// What a write did: wrote the entry, or found it held already and left
/// it untouched.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Written {
    Wrote,
    Held,
}

/// The host's record of the versions it published from one repository.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kept {
    dir: PathBuf,
}

impl Kept {
    /// The record of the repository `history` opens, under its common git
    /// directory, `tailrace/published/` there (62).
    pub fn at(history: &History) -> Kept {
        Kept {
            dir: history.common_dir().join("tailrace").join("published"),
        }
    }

    /// The directory the record is kept in.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn source(&self) -> Source {
        Source::host(&self.dir)
    }

    fn path_of(&self, entry: &Entry) -> PathBuf {
        self.dir
            .join(entry.component.as_str())
            .join(format!("{}.json", entry.tag))
    }

    /// `entry` kept as a file of its own, written beside it first and
    /// renamed, so a reader never sees half an entry. An entry held with
    /// the same content is left untouched; one held with other content is
    /// refused [`Why::KeptOther`] and left as it is.
    pub fn write(&self, entry: &Entry) -> Answer<Written> {
        let path = self.path_of(entry);
        match fs::read(&path) {
            Ok(held) => {
                return match serde_json::from_slice::<Entry>(&held) {
                    Ok(held) if held == *entry => Ok(Written::Held),
                    Ok(_) => Err(Not::refused(
                        &self.source(),
                        Why::KeptOther {
                            path: path.display().to_string(),
                        },
                        None,
                    )),
                    Err(e) => Err(Not::unreadable(
                        &self.source(),
                        path.display().to_string(),
                        Some(e.to_string()),
                    )),
                };
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(self.cannot(&path, e)),
        }
        let dir = path.parent().expect("an entry's path is under the record");
        fs::create_dir_all(dir).map_err(|e| self.cannot(dir, e))?;
        let mut text = serde_json::to_string_pretty(entry).expect("an entry is plain data");
        text.push('\n');
        let beside = dir.join(format!(".{}.json.{}", entry.tag, std::process::id()));
        fs::write(&beside, text).map_err(|e| self.cannot(&beside, e))?;
        fs::rename(&beside, &path).map_err(|e| {
            let _ = fs::remove_file(&beside);
            self.cannot(&path, e)
        })?;
        Ok(Written::Wrote)
    }

    fn cannot(&self, path: &Path, e: io::Error) -> Not {
        Not::unreadable(
            &self.source(),
            path.display().to_string(),
            Some(e.to_string()),
        )
    }

    /// Every entry `filter` keeps, ordered by component and then by the
    /// version rules (23). Absent, naming the record's directory, when it
    /// holds none; unreadable, naming the file with the parser's words,
    /// when an entry cannot be read (58, 59).
    pub fn read(&self, filter: &Filter) -> Answer<Vec<Entry>> {
        let mut entries = Vec::new();
        let components = match fs::read_dir(&self.dir) {
            Ok(components) => components,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(self.nothing(filter)),
            Err(e) => return Err(self.cannot(&self.dir, e)),
        };
        for component in components {
            let component = component.map_err(|e| self.cannot(&self.dir, e))?.path();
            if !component.is_dir() {
                continue;
            }
            let files = fs::read_dir(&component).map_err(|e| self.cannot(&component, e))?;
            for file in files {
                let path = file.map_err(|e| self.cannot(&component, e))?.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.starts_with('.') || !name.ends_with(".json") {
                    continue;
                }
                let text = fs::read_to_string(&path).map_err(|e| self.cannot(&path, e))?;
                let entry: Entry = serde_json::from_str(&text).map_err(|e| {
                    Not::unreadable(
                        &self.source(),
                        path.display().to_string(),
                        Some(e.to_string()),
                    )
                })?;
                if filter.keeps(&entry) {
                    entries.push(entry);
                }
            }
        }
        if entries.is_empty() {
            return Err(self.nothing(filter));
        }
        entries.sort_by(|a, b| {
            a.component
                .cmp(&b.component)
                .then_with(|| by_version(&a.version, &b.version))
        });
        Ok(entries)
    }

    fn nothing(&self, filter: &Filter) -> Not {
        let mut what = String::from("version it published");
        if let Some(component) = &filter.component {
            what.push_str(&format!(" of {component}"));
        }
        if let Some(revision) = &filter.revision {
            what.push_str(&format!(" at {revision}"));
        }
        Not::absent(&self.source(), what)
    }
}

/// 23: semantic version precedence, a version the rules do not read
/// after every one they do, by its text.
fn by_version(a: &Version, b: &Version) -> std::cmp::Ordering {
    match (
        semver::Version::parse(a.as_str()),
        semver::Version::parse(b.as_str()),
    ) {
        (Ok(a), Ok(b)) => a.cmp_precedence(&b).then_with(|| a.cmp(&b)),
        (Ok(_), Err(_)) => std::cmp::Ordering::Less,
        (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
        (Err(_), Err(_)) => a.cmp(b),
    }
}
