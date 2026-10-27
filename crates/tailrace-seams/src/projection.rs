//! B.6 A projection: an artifact, or a cache entry, in one ecosystem's
//! own form, as that ecosystem's clients read it, one request of those
//! clients answered from a served layout, the versions a checkout's
//! lockfile pins in it (84 to 87, 115, 137), and what a held version's
//! manifest requires and which versions satisfy a range, by the
//! ecosystem's own rules (140).

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

/// 86: one request of the ecosystem's own client of a served layout: its
/// method; its path below the layout's prefix, the query dropped; and
/// `base`, the absolute address of that prefix as the client reached it,
/// with no trailing `/`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub base: String,
}

/// 86, 99: the answer to a request in the client's own protocol: its
/// status, the type of its body, and the body's bytes, written in its
/// machine-readable form as base64.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    pub content_type: String,
    #[serde(with = "base64_body")]
    pub body: Vec<u8>,
}

mod base64_body {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine as _;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(body: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&STANDARD.encode(body))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let text = String::deserialize(deserializer)?;
        STANDARD.decode(text).map_err(serde::de::Error::custom)
    }
}

/// 115: the core asked for one version a served layout lacks, by its name
/// in the ecosystem and its version: `Ok(())` where the layout now holds
/// it, and otherwise absent, refused naming the claim found wanting, or
/// unreadable, with the source's own words (58, 99).
pub type Miss<'a> = &'a dyn Fn(&Pinned) -> Answer<()>;

/// 86: what a projection whose ecosystem's clients read files answers
/// every request with: absent, never refused or unreadable (58).
pub fn no_serving(source: &Source) -> Not {
    Not::absent(source, "serving")
}

/// 137: a version a checkout's lockfile pins, by its name in the
/// ecosystem and its version, each as the lockfile spells it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Pinned {
    pub name: EcosystemName,
    pub version: Version,
}

text! {
    /// 140: a range of versions, as the manifest that requires it spells
    /// it.
    Range
}

/// 140: a name of the ecosystem a held version's manifest requires, and
/// the range it requires of it, each as the manifest spells it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Requirement {
    pub name: EcosystemName,
    pub range: Range,
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

    /// 86, 115: `request`, one request of the ecosystem's own client of
    /// `layout`, answered in the client's own protocol from the layout
    /// alone. A version the layout lacks is asked of the core once through
    /// `miss`, which judges it and renders it into the layout, and is then
    /// answered from the layout; a refusal, an absence or an unreadable
    /// answer of the core is answered in a response the client reads,
    /// carrying the source's own words (99, 59). Every answer the client
    /// reads is a [`Response`]; a projection whose ecosystem's clients read
    /// files answers [`no_serving`] for every request and asks the core
    /// nothing. It reaches no store, policy or claim: the judgment is the
    /// core's (rule core.2).
    fn serve(&self, layout: &Layout, request: &Request, miss: Miss<'_>) -> Answer<Response>;

    /// 137: the versions the lockfile of the ecosystem in the checkout at
    /// `checkout` pins, as the lockfile spells them; [`no_lockfile`] where
    /// the checkout holds none. A name the checkout's manifest declares
    /// and its lockfile does not pin is no pin, so its layout holds none
    /// of it: a manifest's range, resolved by the store's listing of a
    /// name's versions, waits on the unit that lists them. It reads the
    /// checkout alone: the store, the policy and a version's claims are
    /// the core's (rule core.2).
    fn pinned(&self, checkout: &str) -> Answer<Vec<Pinned>>;

    /// 140: every name the manifest of `held`, an artifact a layout
    /// holds, requires that the layout serves, with the range it requires
    /// of it, as the manifest spells them, in the order it gives them. A
    /// name the layout serves is one the consuming checkout's own
    /// configuration of the ecosystem routes to the served address, as
    /// the ecosystem spells that routing; every other name is the public
    /// registry's and no concern of the layout's (section 8). A
    /// projection with a manifest reads
    /// the artifact's bytes through the store seam it was bound with,
    /// never from a layout (48, rule core.2). An ecosystem with no
    /// manifest answers none, an empty list, reading nothing, and never
    /// absent.
    fn requires(&self, held: &Artifact) -> Answer<Vec<Requirement>>;

    /// 140: those of `versions` that satisfy `range` by the ecosystem's
    /// own rules, newest first by the ecosystem's own ordering, never by
    /// spelling or by the order given (23), so the core learns both
    /// which satisfy and which is newest from this one answer. An
    /// ecosystem with no manifest answers none, an empty list, and never
    /// absent.
    fn satisfying(&self, range: &Range, versions: &[Version]) -> Answer<Vec<Version>>;
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
    }

    #[test]
    fn a_request_and_a_response_round_trip_bytes_and_all() {
        round_trip(Request {
            method: "GET".into(),
            path: "@scope/pkg/-/pkg-1.0.0.tgz".into(),
            base: "http://127.0.0.1:7420/o/r/eco".into(),
        });
        let response = Response {
            status: 403,
            content_type: "application/octet-stream".into(),
            body: vec![0, 1, 127, 128, 255, b'"'],
        };
        round_trip(response.clone());
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["body"], "AAF/gP8i", "{json}");
        round_trip(Response {
            body: vec![],
            ..response
        });
    }

    #[test]
    fn a_projection_that_serves_none_answers_absent_naming_serving() {
        let source = Source::new(Seam::Projection, "file");
        match no_serving(&source) {
            Not::Absent(a) => {
                assert_eq!(a.source, source);
                assert_eq!(a.what, "serving");
            }
            other => panic!("answered {other:?}"),
        }
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
    fn a_requirement_round_trips_as_the_manifest_spells_it() {
        let requirement = Requirement {
            name: EcosystemName::new("@scope/pkg"),
            range: Range::new(">=1.2.0-rc.1 <2"),
        };
        round_trip(requirement.clone());
        let json = serde_json::to_value(&requirement).unwrap();
        assert_eq!(json["name"], "@scope/pkg", "{json}");
        assert_eq!(json["range"], ">=1.2.0-rc.1 <2", "{json}");
        round_trip(vec![requirement.clone(), requirement]);
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
