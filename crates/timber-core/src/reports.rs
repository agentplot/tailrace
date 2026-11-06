//! The host's copy of each environment's reports, as status last fetched
//! them from the store (60, 61, 63, 98, S10, S12, rules core.20, core.27,
//! core.32; design D4).
//!
//! Status asks the store the claims of each index an environment's
//! channel's chain moved to, once per status whichever environments'
//! chains moved to it, and keeps that environment's report claims, each
//! with the index it is attached to and none yet counted, so a changed
//! policy counts the copy again with no fetch. The copy is written only
//! once every index's claims are read, beside its file and renamed into
//! place, and where the store does not answer the reports are answered
//! from it, saying when they were fetched and with the store's words.
//!
//! The copy is the host's, kept as the record of published versions is,
//! under the repository's common git directory: the reports are always
//! the store's, read through the store seam alone, so it names no profile
//! (rules core.1, core.2), and deleting it loses nothing, since the next
//! status fetches it again. It holds nothing the store does not, and no
//! credential (64).

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use timber_seams::{
    Answer, ChannelId, Claim, Digest, Envelope, EnvironmentName, Not, Read, Source, Store,
};

use crate::after::{index_absent, reports, ReportClaim, Reports};
use crate::{History, Reported};

/// The host's copy of every environment's reports, under the common git
/// directory of the repository, `timber/reports/` there, beside the
/// store ledger's `timber/ledger/` (61, 62).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportsCopy {
    dir: PathBuf,
}

/// One environment's reports as the copy holds them: the moment they
/// were fetched, in seconds since the Unix epoch, and each report claim
/// with the index it is attached to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Copied {
    at: u64,
    reports: Vec<Held>,
}

/// One report claim as the copy holds it: the index it is attached to
/// and its envelope (rule core.27).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Held {
    index: Digest,
    envelope: Envelope,
}

impl ReportsCopy {
    /// The copy of the repository `history` opens.
    pub fn at(history: &History) -> ReportsCopy {
        ReportsCopy {
            dir: history.common_dir().join("timber").join("reports"),
        }
    }

    /// The directory the copy is kept in.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Where the reports of `environment`, which follows `channel`, are
    /// kept: `<environment>.json` where the repository declares no
    /// component, and `<component>/<environment>.json` where it does
    /// (125).
    pub fn path(&self, environment: &EnvironmentName, channel: &ChannelId) -> PathBuf {
        let dir = match &channel.component {
            Some(component) => self.dir.join(component.as_str()),
            None => self.dir.clone(),
        };
        dir.join(format!("{environment}.json"))
    }

    /// 58: a file of the copy the system could not read or write,
    /// unreadable naming it with the system's words.
    fn cannot(path: &Path, said: impl ToString) -> Not {
        Not::unreadable(
            &Source::host(path),
            path.display().to_string(),
            Some(said.to_string()),
        )
    }

    /// 61: `reports` kept as fetched at `at`, written beside the file and
    /// renamed over it, so a reader never sees half a copy.
    fn keep(&self, channel: &ChannelId, reports: &Reports, at: u64) -> Answer<()> {
        let path = self.path(&reports.environment, channel);
        let dir = path.parent().expect("a copy's file is under its directory");
        fs::create_dir_all(dir).map_err(|e| Self::cannot(dir, e))?;
        let copied = Copied {
            at,
            reports: reports
                .attached
                .iter()
                .map(|r| Held {
                    index: r.index.clone(),
                    envelope: r.envelope.clone(),
                })
                .collect(),
        };
        let mut text = serde_json::to_string_pretty(&copied).expect("a copy is plain data");
        text.push('\n');
        let beside = dir.join(format!(
            ".{}.json.{}",
            reports.environment,
            std::process::id()
        ));
        fs::write(&beside, text).map_err(|e| Self::cannot(&beside, e))?;
        fs::rename(&beside, &path).map_err(|e| {
            let _ = fs::remove_file(&beside);
            Self::cannot(&path, e)
        })
    }

    /// 60, S10: the reports of `environment`, which follows `channel`, as
    /// the copy holds them, read as last fetched at the copy's moment for
    /// `said`, the store's words; unreadable naming them with the store's
    /// answer, `failed`, where there is no copy, and naming the file where
    /// it cannot be read or holds anything but reports of the environment.
    fn kept(
        &self,
        environment: &EnvironmentName,
        channel: &ChannelId,
        failed: Not,
    ) -> Answer<Fetched> {
        let component = channel.component_name();
        let path = self.path(environment, channel);
        let text = match fs::read(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let what = format!("the reports of environment {environment} of {component}");
                return Err(match failed {
                    Not::Unreadable(u) => Not::unreadable(&u.source, what, u.said),
                    other => Not::unreadable(other.source(), what, Some(other.to_string())),
                });
            }
            Err(e) => return Err(Self::cannot(&path, e)),
        };
        let copied: Copied = serde_json::from_slice(&text).map_err(|e| Self::cannot(&path, e))?;
        let mut attached = Vec::new();
        for held in copied.reports {
            let report = Claim::from_envelope(&held.envelope)
                .as_ref()
                .and_then(Reported::of)
                .filter(|r| r.environment == *environment && r.component == component)
                .ok_or_else(|| {
                    Self::cannot(
                        &path,
                        format!(
                            "{} on {} is no report of environment {environment} of {component}",
                            held.envelope.digest(),
                            held.index
                        ),
                    )
                })?;
            attached.push(ReportClaim {
                index: held.index,
                envelope: held.envelope,
                report,
            });
        }
        let said = match failed {
            Not::Unreadable(u) => u
                .said
                .clone()
                .unwrap_or_else(|| Not::Unreadable(u).to_string()),
            other => other.to_string(),
        };
        Ok(Fetched {
            reports: Reports {
                environment: environment.clone(),
                component,
                attached,
            },
            read: Read::Kept {
                at: copied.at,
                said,
            },
        })
    }
}

/// An environment's reports as status read them, none yet counted, and
/// how: from the store, or from the copy as last fetched, saying when and
/// with the store's words (60).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Fetched {
    pub reports: Reports,
    pub read: Read,
}

/// One status's fetch of the reports: the store asked the claims of each
/// index once and its answer shared by every environment whose chain
/// moved to it, the copy each environment's reports are kept in, and the
/// moment of the ask, in seconds since the Unix epoch, which each copy
/// written names, so the core reads no clock (36).
pub struct Fetch<'a> {
    store: &'a dyn Store,
    copy: &'a ReportsCopy,
    at: u64,
    claims: BTreeMap<Digest, Answer<Vec<Envelope>>>,
}

impl<'a> Fetch<'a> {
    pub fn new(store: &'a dyn Store, copy: &'a ReportsCopy, at: u64) -> Fetch<'a> {
        Fetch {
            store,
            copy,
            at,
            claims: BTreeMap::new(),
        }
    }

    /// The claims of `index` as the store answered them, asked on the
    /// first ask of this fetch and the same answer on every ask after.
    fn claims_of(&mut self, index: &Digest) -> Answer<Vec<Envelope>> {
        self.claims
            .entry(index.clone())
            .or_insert_with(|| self.store.claims_of(index))
            .clone()
    }

    /// 60, 61, 98: the reports of `environment`, which follows `channel`,
    /// attached to each of `indexes`, the indexes its chain moved to in
    /// the order it first moved to each (design D4). Where the store
    /// answers every index's claims, they are kept in the copy and
    /// answered as read from the source; where it answers the first it
    /// does not answer unreadable, nothing is written and the reports are
    /// answered from the copy ([`ReportsCopy`]), saying when; where it
    /// answers it absent, unreadable naming the index, the copy left as it
    /// was (58). A copy that cannot be written is unreadable naming its
    /// file.
    pub fn reports(
        &mut self,
        environment: &EnvironmentName,
        channel: &ChannelId,
        indexes: &[Digest],
    ) -> Answer<Fetched> {
        for index in indexes {
            match self.claims_of(index) {
                Ok(_) => {}
                Err(Not::Unreadable(u)) => {
                    return self.copy.kept(environment, channel, Not::Unreadable(u))
                }
                Err(not) => return Err(index_absent(channel, index, not)),
            }
        }
        let read = reports(
            &mut |index| self.claims_of(index),
            indexes,
            environment,
            &channel.component_name(),
        )?;
        self.copy.keep(channel, &read, self.at)?;
        Ok(Fetched {
            reports: read,
            read: Read::Source,
        })
    }
}
