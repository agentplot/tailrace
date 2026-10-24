//! The cache: a build's cache entry held in the store under its build
//! identity, the lowest level of trust, so any host whose build identity
//! matches reads what another host built (52; design D1, D2). An entry is
//! bytes and the name `cache/<build identity>` at their digest, made from
//! nothing; no other name begins `cache/`. Writing one puts no manifest,
//! attaches no claim and moves no other name, so it offers nothing, and
//! bytes an artifact already holds are the one store object they were
//! (53, 100). Sync fills a checkout's cache from what a landed
//! revision's passing verdicts name, reading no policy and writing
//! nothing to the store, and an entry that is gone is an answer, never a
//! refusal (54; design D3). No channel, index or policy names a cache
//! entry (rule core.30).

use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use tailrace_seams::{
    no_cache_entries, Answer, Branch, BuildId, CacheEntry, Contents, Digest, Ecosystem, Name, Not,
    Outcome, Output, Refusal, Revision, Seam, Seams, Source, Why,
};

use crate::{answer_json, Landed, BOUND};

/// What every cache name begins with, and no other name does.
pub const CACHE: &str = "cache/";

/// 52: the store's name for the cache entry of `build`.
pub fn cache_name(build: &BuildId) -> Name {
    Name::new(format!("{CACHE}{build}"))
}

/// What a write answers: the entry packed, the projection that packed
/// it, and whether the store held it already.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Written {
    pub entry: CacheEntry,
    pub projection: Ecosystem,
    /// None when this write made the name; the digest the name was at
    /// when it was held already, the same or another host's bytes.
    pub held_at: Option<Digest>,
    pub bytes_held_already: bool,
}

/// Whether `answer` is a projection saying it packs no cache entries,
/// which a write or a sync passes over (84, 85).
pub(crate) fn packs_none<T>(answer: &Answer<T>) -> bool {
    matches!(answer, Err(not) if *not == no_cache_entries(not.source()))
}

/// 52: the cache entry of `output` at `build`, packed with no version by
/// the first bound projection, in ecosystem order, that packs cache
/// entries; absent naming cache entries when none does, and nothing is
/// written. The bytes are put, then the name made from nothing; a name
/// held already, at these bytes or another host's, is answered held at
/// the digest it names, unmoved and with nothing retried (rule core.6).
/// A pack that answers metadata is refused before anything is put (53).
pub fn write(seams: &Seams, output: &Output, build: &BuildId) -> Answer<Written> {
    let mut found = None;
    for (ecosystem, projection) in &seams.projections {
        let answer = projection.pack(output, build, None);
        if !packs_none(&answer) {
            found = Some((ecosystem, answer?));
            break;
        }
    }
    let Some((ecosystem, packed)) = found else {
        return Err(no_cache_entries(&Source::new(Seam::Projection, BOUND)));
    };
    if packed.meta.is_some() {
        return Err(Not::refused(
            &Source::new(Seam::Projection, ecosystem.as_str()),
            Why::Said,
            Some(format!(
                "{} was packed at {build} with no version and the answer names metadata",
                output.name
            )),
        ));
    }

    let digest = Digest::of(&packed.bytes);
    let put = seams.store.put_bytes(&packed.bytes)?;
    if put.digest != digest {
        return Err(Not::refused(
            &Source::new(Seam::Store, BOUND),
            Why::OtherBytes {
                expected: digest,
                answered: put.digest,
            },
            None,
        ));
    }
    let held_at = match seams.store.name(&cache_name(build), &digest, None) {
        Ok(()) => None,
        Err(Not::Refused(Refusal {
            why: Why::NameAt { at: Some(at) },
            ..
        })) => Some(at),
        Err(not) => return Err(not),
    };
    Ok(Written {
        entry: CacheEntry {
            build: build.clone(),
            digest,
        },
        projection: ecosystem.clone(),
        held_at,
        bytes_held_already: put.held_already,
    })
}

/// What sync is asked: the revision whose verdicts name the entries, the
/// branch that landed it, and the checkout rendered into.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SyncAsk {
    pub branch: Branch,
    pub revision: Revision,
    pub at: String,
}

/// 57: what sync answers, per build identity a passing verdict names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Synced {
    pub branch: Branch,
    pub revision: Revision,
    /// Each build identity a passing verdict names, once, in the order
    /// the verdicts name them, with the answer for it: the digest and
    /// the projections that rendered it, or why not.
    pub entries: Vec<(BuildId, Answer<Filled>)>,
}

/// An entry rendered: its digest, and the projections that rendered it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Filled {
    pub digest: Digest,
    pub rendered: Vec<Ecosystem>,
}

impl Serialize for Synced {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let entries: Vec<Value> = self
            .entries
            .iter()
            .map(|(build, answer)| {
                let mut value = answer_json(answer);
                value["build"] = json!(build);
                value
            })
            .collect();
        json!({
            "branch": self.branch,
            "revision": self.revision,
            "entries": entries,
        })
        .serialize(serializer)
    }
}

/// 52, C.3: the cache entries `ask.revision`'s passing verdicts name,
/// each rendered by itself at `ask.at` through every bound projection
/// that packs cache entries. The landing is asked first, and a revision
/// its branch has not landed is refused before the store is asked (14,
/// rule core.11). A failed verdict's build identity is not asked for. An
/// entry whose name the store does not hold is answered by what resolve
/// answered, and every other entry is rendered all the same (54). Sync
/// asks the store only to resolve, and never the policy (53).
pub fn sync(seams: &Seams, ask: &SyncAsk) -> Answer<Synced> {
    let landed = Landed::check(seams.landing.as_ref(), &ask.branch, &ask.revision)?;
    let mut builds: Vec<BuildId> = Vec::new();
    for verdict in seams.landing.verdicts_of(landed.revision())? {
        if verdict.outcome == Outcome::Pass && !builds.contains(&verdict.build) {
            builds.push(verdict.build);
        }
    }
    let entries = builds
        .into_iter()
        .map(|build| {
            let answer = seams.store.resolve(&cache_name(&build)).and_then(|digest| {
                fill(
                    seams,
                    CacheEntry {
                        build: build.clone(),
                        digest,
                    },
                    &ask.at,
                )
            });
            (build, answer)
        })
        .collect();
    Ok(Synced {
        branch: ask.branch.clone(),
        revision: ask.revision.clone(),
        entries,
    })
}

/// `entry` rendered alone through every bound projection, passing over
/// each that packs no cache entries, so one bad entry refuses only
/// itself: the projections that rendered it, or the first other answer a
/// render gave; absent naming cache entries when none rendered it.
fn fill(seams: &Seams, entry: CacheEntry, at: &str) -> Answer<Filled> {
    let mut rendered = Vec::new();
    let mut first = None;
    for (ecosystem, projection) in &seams.projections {
        let answer = projection.render(Contents::CacheEntries(std::slice::from_ref(&entry)), at);
        if packs_none(&answer) {
            continue;
        }
        match answer {
            Ok(_) => rendered.push(ecosystem.clone()),
            Err(not) => {
                first.get_or_insert(not);
            }
        }
    }
    if let Some(not) = first {
        return Err(not);
    }
    if rendered.is_empty() {
        return Err(no_cache_entries(&Source::new(Seam::Projection, BOUND)));
    }
    Ok(Filled {
        digest: entry.digest,
        rendered,
    })
}
