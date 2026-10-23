//! What a build leaves and what an artifact says of its bytes: the
//! revision and branch, the component and its outputs, the build identity
//! and outcome, the version, the ecosystem, artifact and index metadata
//! (4, 6), the manifest the store is handed (67), what a digest holds (72)
//! and the names that point at digests (3).

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::Digest;

text! {
    /// One commit of a repository, by its git id.
    Revision
}

text! {
    /// A branch a revision lands on (77).
    Branch
}

text! {
    /// The name the authors give a component (123).
    ComponentName
}

/// The outputs the authors declare to take one version together, the
/// paths whose changes version it, and the prefix its tags carry (123).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    pub name: ComponentName,
    pub outputs: Vec<OutputName>,
    pub paths: Vec<String>,
    pub prefix: String,
}

text! {
    /// The name the authors declare an output by.
    OutputName
}

/// An output a build left: its name and where its files are (75).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Output {
    pub name: OutputName,
    pub files: Vec<String>,
}

text! {
    /// The name a build gives the inputs it ran over, by which two parties
    /// name the same build without seeing each other (13, 76).
    BuildId
}

/// How a build, or a check over it, ended (16, 75, 78).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Pass,
    Fail,
}

text! {
    /// The name a publish gives a component's artifacts, in the
    /// ecosystem's own version rules (21).
    Version
}

text! {
    /// An ecosystem, by the name a projection is bound under (92).
    Ecosystem
}

text! {
    /// An artifact's name in its ecosystem (4).
    EcosystemName
}

text! {
    /// A name the store moves to a digest: a version's, an ecosystem
    /// name's or a channel's, each a pointer to a digest and never the
    /// other way (3, 70).
    Name
}

text! {
    /// A channel, by name (31).
    ChannelName
}

/// 4: what an artifact's bytes are: the ecosystem whose form they take,
/// their name in it and their version. None of the three is ever empty.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawArtifactMeta")]
pub struct ArtifactMeta {
    ecosystem: Ecosystem,
    name: EcosystemName,
    version: Version,
}

#[derive(Deserialize)]
struct RawArtifactMeta {
    ecosystem: Ecosystem,
    name: EcosystemName,
    version: Version,
}

impl TryFrom<RawArtifactMeta> for ArtifactMeta {
    type Error = BadMeta;

    fn try_from(raw: RawArtifactMeta) -> Result<Self, BadMeta> {
        ArtifactMeta::new(raw.ecosystem, raw.name, raw.version)
    }
}

impl ArtifactMeta {
    /// Metadata that names all three, or rejected naming the first that
    /// is empty (4).
    pub fn new(
        ecosystem: Ecosystem,
        name: EcosystemName,
        version: Version,
    ) -> Result<Self, BadMeta> {
        if ecosystem.as_str().is_empty() {
            return Err(BadMeta::Empty(Part::Ecosystem));
        }
        if name.as_str().is_empty() {
            return Err(BadMeta::Empty(Part::Name));
        }
        if version.as_str().is_empty() {
            return Err(BadMeta::Empty(Part::Version));
        }
        Ok(ArtifactMeta {
            ecosystem,
            name,
            version,
        })
    }

    pub fn ecosystem(&self) -> &Ecosystem {
        &self.ecosystem
    }

    pub fn name(&self) -> &EcosystemName {
        &self.name
    }

    pub fn version(&self) -> &Version {
        &self.version
    }
}

/// An artifact: the digest the store names its bytes by, and what they
/// are.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    pub digest: Digest,
    pub meta: ArtifactMeta,
}

/// 6, 110: an index's metadata: one component, one version, and every
/// artifact of the component at that version by digest and metadata,
/// each carrying the index's version.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawIndexMeta")]
pub struct IndexMeta {
    component: ComponentName,
    version: Version,
    artifacts: Vec<Artifact>,
}

#[derive(Deserialize)]
struct RawIndexMeta {
    component: ComponentName,
    version: Version,
    artifacts: Vec<Artifact>,
}

impl TryFrom<RawIndexMeta> for IndexMeta {
    type Error = BadMeta;

    fn try_from(raw: RawIndexMeta) -> Result<Self, BadMeta> {
        IndexMeta::new(raw.component, raw.version, raw.artifacts)
    }
}

impl IndexMeta {
    /// An index naming `artifacts`, or rejected naming an empty component
    /// or version, or the first artifact of another version (110).
    pub fn new(
        component: ComponentName,
        version: Version,
        artifacts: Vec<Artifact>,
    ) -> Result<Self, BadMeta> {
        if component.as_str().is_empty() {
            return Err(BadMeta::Empty(Part::Component));
        }
        if version.as_str().is_empty() {
            return Err(BadMeta::Empty(Part::Version));
        }
        if let Some(other) = artifacts.iter().find(|a| a.meta.version != version) {
            return Err(BadMeta::OtherVersion {
                artifact: other.digest.clone(),
                version: other.meta.version.clone(),
                index: version,
            });
        }
        Ok(IndexMeta {
            component,
            version,
            artifacts,
        })
    }

    pub fn component(&self) -> &ComponentName {
        &self.component
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn artifacts(&self) -> &[Artifact] {
        &self.artifacts
    }
}

/// What put-artifact is handed (67): an artifact's metadata and the
/// digests of its bytes, or an index, an artifact whose bytes are the
/// artifacts it names (6). The store chooses the digest it names either
/// by (design D4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Manifest {
    Artifact {
        meta: ArtifactMeta,
        layers: Vec<Digest>,
    },
    Index(IndexMeta),
}

/// What a digest holds, as read answers it (72): the bytes put-bytes
/// held, or the manifest put-artifact held, so an index is read back to
/// name its artifacts (design D4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Held {
    Bytes(Vec<u8>),
    Manifest(Manifest),
}

/// Metadata that cannot be made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BadMeta {
    /// 4, 6: a part the metadata must name is empty.
    Empty(Part),
    /// 110: an artifact the index names carries a version not the
    /// index's.
    OtherVersion {
        artifact: Digest,
        version: Version,
        index: Version,
    },
}

/// A part metadata names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Ecosystem,
    Name,
    Version,
    Component,
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Part::Ecosystem => "ecosystem",
            Part::Name => "name",
            Part::Version => "version",
            Part::Component => "component",
        })
    }
}

impl fmt::Display for BadMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BadMeta::Empty(part) => write!(f, "the metadata names no {part}"),
            BadMeta::OtherVersion {
                artifact,
                version,
                index,
            } => write!(
                f,
                "the index is of version {index} and names {artifact}, of version {version}"
            ),
        }
    }
}

impl std::error::Error for BadMeta {}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::round_trip;

    fn meta(version: &str) -> ArtifactMeta {
        ArtifactMeta::new(
            Ecosystem::new("eco"),
            EcosystemName::new("pkg"),
            Version::new(version),
        )
        .unwrap()
    }

    fn artifact(bytes: &[u8], version: &str) -> Artifact {
        Artifact {
            digest: Digest::of(bytes),
            meta: meta(version),
        }
    }

    #[test]
    fn metadata_with_an_empty_part_is_rejected_naming_it() {
        let made = |e: &str, n: &str, v: &str| {
            ArtifactMeta::new(Ecosystem::new(e), EcosystemName::new(n), Version::new(v))
        };
        assert_eq!(
            made("", "pkg", "1.0.0"),
            Err(BadMeta::Empty(Part::Ecosystem))
        );
        assert_eq!(made("eco", "", "1.0.0"), Err(BadMeta::Empty(Part::Name)));
        assert_eq!(made("eco", "pkg", ""), Err(BadMeta::Empty(Part::Version)));
        assert_eq!(
            made("eco", "pkg", "").unwrap_err().to_string(),
            "the metadata names no version"
        );
        let m = made("eco", "pkg", "1.0.0").unwrap();
        assert_eq!(
            (
                m.ecosystem().as_str(),
                m.name().as_str(),
                m.version().as_str()
            ),
            ("eco", "pkg", "1.0.0")
        );
    }

    #[test]
    fn metadata_read_with_an_empty_part_is_rejected_naming_it() {
        let err = serde_json::from_str::<ArtifactMeta>(
            r#"{"ecosystem":"eco","name":"","version":"1.0.0"}"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("names no name"), "{err}");
    }

    #[test]
    fn an_index_names_each_artifact_at_its_version() {
        let a = artifact(b"a", "1.0.0");
        let b = artifact(b"b", "1.0.0");
        let index = IndexMeta::new(
            ComponentName::new("web"),
            Version::new("1.0.0"),
            vec![a.clone(), b.clone()],
        )
        .unwrap();
        assert_eq!(index.component().as_str(), "web");
        assert_eq!(index.version().as_str(), "1.0.0");
        assert_eq!(index.artifacts(), &[a, b]);
    }

    #[test]
    fn an_index_rejects_an_artifact_of_another_version() {
        let a = artifact(b"a", "1.0.0");
        let b = artifact(b"b", "0.9.0");
        let err = IndexMeta::new(
            ComponentName::new("web"),
            Version::new("1.0.0"),
            vec![a, b.clone()],
        )
        .unwrap_err();
        assert_eq!(
            err,
            BadMeta::OtherVersion {
                artifact: b.digest.clone(),
                version: Version::new("0.9.0"),
                index: Version::new("1.0.0"),
            }
        );
        assert!(err.to_string().contains(b.digest.as_str()));
    }

    #[test]
    fn an_index_rejects_an_empty_component_or_version() {
        assert_eq!(
            IndexMeta::new(ComponentName::new(""), Version::new("1.0.0"), vec![]),
            Err(BadMeta::Empty(Part::Component))
        );
        assert_eq!(
            IndexMeta::new(ComponentName::new("web"), Version::new(""), vec![]),
            Err(BadMeta::Empty(Part::Version))
        );
    }

    #[test]
    fn an_index_read_with_an_artifact_of_another_version_is_rejected() {
        let a = artifact(b"a", "0.9.0");
        let json = serde_json::json!({"component": "web", "version": "1.0.0", "artifacts": [a]});
        let err = serde_json::from_value::<IndexMeta>(json).unwrap_err();
        assert!(err.to_string().contains("of version 0.9.0"), "{err}");
    }

    #[test]
    fn every_type_round_trips_through_json() {
        round_trip(Revision::new("4f2a9c1"));
        round_trip(Branch::new("main"));
        round_trip(Component {
            name: ComponentName::new("web"),
            outputs: vec![OutputName::new("site"), OutputName::new("cli")],
            paths: vec!["site/".into(), "crates/".into()],
            prefix: "web-v".into(),
        });
        round_trip(Output {
            name: OutputName::new("site"),
            files: vec!["dist/index.html".into()],
        });
        round_trip(BuildId::new("b7e1"));
        round_trip(Outcome::Pass);
        round_trip(Outcome::Fail);
        round_trip(meta("1.0.0"));
        round_trip(artifact(b"a", "1.0.0"));
        let index = IndexMeta::new(
            ComponentName::new("web"),
            Version::new("1.0.0"),
            vec![artifact(b"a", "1.0.0")],
        )
        .unwrap();
        round_trip(index.clone());
        round_trip(Manifest::Artifact {
            meta: meta("1.0.0"),
            layers: vec![Digest::of(b"layer")],
        });
        round_trip(Manifest::Index(index.clone()));
        round_trip(Held::Bytes(vec![0, 159, 146, 150, 255]));
        round_trip(Held::Manifest(Manifest::Index(index)));
        round_trip(Name::new("web-v1.0.0"));
        round_trip(ChannelName::new("beta"));
    }
}
