//! B.6 A projection: an artifact, or a cache entry, in one ecosystem's
//! own form, as that ecosystem's clients read it, and the versions a
//! checkout's lockfile pins in it (84 to 87, 137).

use serde::{Deserialize, Serialize};

use crate::{
    Answer, Artifact, ArtifactMeta, BuildId, Digest, EcosystemName, Not, Output, Source, Version,
};

/// 84, 87, 100: an output packed: the ecosystem's own bytes, the
/// metadata that makes them an artifact, present exactly where a version
/// was given, and whether the same output and version, or the same build
/// identity, pack to the same bytes again.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Packed {
    pub bytes: Vec<u8>,
    pub meta: Option<ArtifactMeta>,
    pub same_again: bool,
}

/// 52, 53: bytes a build left, named by their build identity and their
/// digest, which is never an artifact: it has no ecosystem, name or
/// version, and no conversion makes one an [`Artifact`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheEntry {
    pub build: BuildId,
    pub digest: Digest,
}

/// 85: what a layout is rendered from: artifacts and their metadata, or
/// cache entries by build identity.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Contents<'a> {
    Artifacts(&'a [Artifact]),
    CacheEntries(&'a [CacheEntry]),
}

/// A consumer layout, by where it was rendered.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub at: String,
}

/// 85: what a layout holds: the artifacts, by metadata, it was rendered
/// from, or the build identities of the cache entries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Holds {
    Versions(Vec<ArtifactMeta>),
    BuildIds(Vec<BuildId>),
}

/// 85: a layout rendered, and what it holds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rendered {
    pub layout: Layout,
    pub holds: Holds,
}

/// 86: where the ecosystem's own clients read a served layout.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Serving {
    pub address: String,
}

/// 137: a version a checkout's lockfile pins, by its name in the
/// ecosystem and its version, each as the lockfile spells it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Pinned {
    pub name: EcosystemName,
    pub version: Version,
}

/// 84, 85: what a projection that packs no cache entries answers for
/// one: absent, never refused or unreadable (58).
pub fn no_cache_entries(source: &Source) -> Not {
    Not::absent(source, "cache entries")
}

/// 137: what a projection answers for the pins of a checkout holding no
/// lockfile of its ecosystem: absent, never refused, unreadable or an
/// empty list of pins (58).
pub fn no_lockfile(source: &Source) -> Not {
    Not::absent(source, "lockfile")
}

pub trait Projection {
    /// 84: the ecosystem's own bytes for `output` at the build identity
    /// `at`. Given a `version`, their metadata (4); given none, a cache
    /// entry, whose bytes carry no metadata that makes an offer and whose
    /// answer carries none (100), or [`no_cache_entries`] from a
    /// projection that packs none. `same_again` false where the projection
    /// cannot promise the same output and version, or the same build
    /// identity, pack to the same bytes (87).
    fn pack(&self, output: &Output, at: &BuildId, version: Option<&Version>) -> Answer<Packed>;

    /// 85: a consumer layout of `contents` in the ecosystem's own form,
    /// rendered at `at`, and the versions or build identities it holds;
    /// [`no_cache_entries`] for cache entries from a projection that packs
    /// none.
    fn render(&self, contents: Contents<'_>, at: &str) -> Answer<Rendered>;

    /// 86: `layout` served over the protocol the ecosystem's clients
    /// read, for an ecosystem whose clients read no files.
    fn serve(&self, layout: &Layout) -> Answer<Serving>;

    /// 137: the versions the lockfile of the ecosystem in the checkout at
    /// `checkout` pins, as the lockfile spells them; [`no_lockfile`] where
    /// the checkout holds none. It reads the checkout alone: the store,
    /// the policy and a version's claims are the core's (rule core.2).
    fn pinned(&self, checkout: &str) -> Answer<Vec<Pinned>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{round_trip, Ecosystem, EcosystemName, Seam};

    fn meta() -> ArtifactMeta {
        ArtifactMeta::new(
            Ecosystem::new("eco"),
            EcosystemName::new("pkg"),
            Version::new("1.0.0"),
        )
        .unwrap()
    }

    fn entry() -> CacheEntry {
        CacheEntry {
            build: BuildId::new("b7e1"),
            digest: Digest::of(b"archive"),
        }
    }

    #[test]
    fn every_type_round_trips_through_json() {
        round_trip(Packed {
            bytes: vec![0, 1, 255],
            meta: Some(meta()),
            same_again: false,
        });
        round_trip(Packed {
            bytes: vec![0, 1, 255],
            meta: None,
            same_again: true,
        });
        round_trip(entry());
        let layout = Layout {
            at: "layouts/eco".into(),
        };
        round_trip(layout.clone());
        round_trip(Rendered {
            layout: layout.clone(),
            holds: Holds::Versions(vec![meta()]),
        });
        round_trip(Rendered {
            layout,
            holds: Holds::BuildIds(vec![BuildId::new("b7e1"), BuildId::new("c3d4")]),
        });
        round_trip(Serving {
            address: "serve://eco/repo".into(),
        });
    }

    #[test]
    fn a_pin_round_trips_as_the_lockfile_spells_it() {
        let pin = Pinned {
            name: EcosystemName::new("@scope/pkg"),
            version: Version::new("1.0.0-rc.1+build.7"),
        };
        round_trip(pin.clone());
        let json = serde_json::to_value(&pin).unwrap();
        assert_eq!(json["name"], "@scope/pkg", "{json}");
        assert_eq!(json["version"], "1.0.0-rc.1+build.7", "{json}");
        round_trip(vec![pin.clone(), pin]);
    }

    #[test]
    fn a_checkout_with_no_lockfile_is_absent_naming_the_lockfile() {
        let source = Source::new(Seam::Projection, "file");
        match no_lockfile(&source) {
            Not::Absent(a) => {
                assert_eq!(a.source, source);
                assert_eq!(a.what, "lockfile");
            }
            other => panic!("answered {other:?}"),
        }
    }

    #[test]
    fn a_cache_entry_names_a_build_identity_and_a_digest_and_is_no_artifact() {
        let json = serde_json::to_value(entry()).unwrap();
        let mut keys: Vec<&str> = json
            .as_object()
            .expect("a cache entry is an object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort();
        assert_eq!(keys, ["build", "digest"], "{json}");
        assert!(
            serde_json::from_value::<ArtifactMeta>(json.clone()).is_err(),
            "a cache entry was read as artifact metadata: {json}"
        );
    }

    #[test]
    fn a_projection_that_packs_none_answers_absent_naming_cache_entries() {
        let source = Source::new(Seam::Projection, "file");
        match no_cache_entries(&source) {
            Not::Absent(a) => {
                assert_eq!(a.source, source);
                assert_eq!(a.what, "cache entries");
            }
            other => panic!("answered {other:?}"),
        }
    }
}
