//! B.6 A projection: an artifact in one ecosystem's own form, as that
//! ecosystem's clients read it (84 to 87).

use serde::{Deserialize, Serialize};

use crate::{Answer, Artifact, ArtifactMeta, Output, Version};

/// 84, 87: an output packed: the ecosystem's own bytes, the metadata the
/// ecosystem gives them, and whether the same output and version pack to
/// the same bytes again.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Packed {
    pub bytes: Vec<u8>,
    pub meta: ArtifactMeta,
    pub same_again: bool,
}

/// A consumer layout, by where it was rendered.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub at: String,
}

/// 85: a layout rendered, and the artifacts, by metadata, it holds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rendered {
    pub layout: Layout,
    pub holds: Vec<ArtifactMeta>,
}

/// 86: where the ecosystem's own clients read a served layout.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Serving {
    pub address: String,
}

pub trait Projection {
    /// 84: the ecosystem's own bytes for `output` at `version`, and their
    /// metadata (4); `same_again` false where the projection cannot
    /// promise the same output and version pack to the same bytes (87).
    fn pack(&self, output: &Output, version: &Version) -> Answer<Packed>;

    /// 85: a consumer layout of `artifacts` in the ecosystem's own form,
    /// rendered at `at`, and the versions it holds.
    fn render(&self, artifacts: &[Artifact], at: &str) -> Answer<Rendered>;

    /// 86: `layout` served over the protocol the ecosystem's clients
    /// read, for an ecosystem whose clients read no files.
    fn serve(&self, layout: &Layout) -> Answer<Serving>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{round_trip, Ecosystem, EcosystemName};

    #[test]
    fn every_type_round_trips_through_json() {
        let meta = ArtifactMeta::new(
            Ecosystem::new("eco"),
            EcosystemName::new("pkg"),
            Version::new("1.0.0"),
        )
        .unwrap();
        round_trip(Packed {
            bytes: vec![0, 1, 255],
            meta: meta.clone(),
            same_again: false,
        });
        let layout = Layout {
            at: "layouts/eco".into(),
        };
        round_trip(layout.clone());
        round_trip(Rendered {
            layout,
            holds: vec![meta],
        });
        round_trip(Serving {
            address: "serve://eco/repo".into(),
        });
    }
}
