//! The cache: a build's cache entry held in the store under its build
//! identity, the lowest level of trust, so any host whose build identity
//! matches reads what another host built (52; design D1, D2). An entry is
//! bytes and the name `cache/<build identity>` at their digest, made from
//! nothing; no other name begins `cache/`. Writing one puts no manifest,
//! attaches no claim and moves no other name, so it offers nothing, and
//! bytes an artifact already holds are the one store object they were
//! (53, 100). No channel, index or policy names a cache entry (rule
//! core.30).

use serde::Serialize;
use tailrace_seams::{
    no_cache_entries, Answer, BuildId, CacheEntry, Digest, Ecosystem, Name, Not, Output, Refusal,
    Seam, Seams, Source, Why,
};

use crate::BOUND;

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
