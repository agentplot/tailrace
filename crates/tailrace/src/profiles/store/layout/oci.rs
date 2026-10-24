//! The OCI forms of what the layout holds, and the keys tailrace names
//! (design D4): an artifact as an image manifest whose config is its
//! metadata and whose layers are its bytes, an index as an image index
//! naming each artifact, and an attestation as the envelope's own JSON.
//! Each is written through one struct whose fields are in a fixed order
//! and whose annotations are sorted, so the same manifest writes the same
//! bytes, and so the same digest, on every host (1, 5).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tailrace_seams::{
    canonical, Answer, Artifact, ArtifactMeta, ComponentName, Digest, Ecosystem, EcosystemName,
    IndexMeta, Manifest, Not, Source, Version, Why,
};

pub(super) const IMAGE_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
pub(super) const IMAGE_INDEX: &str = "application/vnd.oci.image.index.v1+json";
/// An artifact's config: its metadata as canonical JSON.
pub(super) const META: &str = "application/vnd.agentplot.tailrace.meta.v1+json";
/// An artifact's layer, and a blob a name names that no manifest entry
/// does.
pub(super) const OCTETS: &str = "application/octet-stream";
/// An attestation: the envelope as canonical JSON.
pub(super) const ENVELOPE: &str = "application/vnd.agentplot.tailrace.envelope.v1+json";
/// An index's artifact type.
pub(super) const INDEX_TYPE: &str = "application/vnd.agentplot.tailrace.index";
/// An artifact's artifact type, before its ecosystem.
const ARTIFACT_TYPE: &str = "application/vnd.agentplot.tailrace.artifact.";

/// A name's entry in `index.json`.
pub(super) const REF_NAME: &str = "org.opencontainers.image.ref.name";
/// An attestation's entry in `index.json`: the digest it is about.
pub(super) const SUBJECT: &str = "io.github.agentplot.tailrace.subject";
const VERSION: &str = "org.opencontainers.image.version";
const COMPONENT: &str = "io.github.agentplot.tailrace.component";
const ECOSYSTEM: &str = "io.github.agentplot.tailrace.ecosystem";
const NAME: &str = "io.github.agentplot.tailrace.name";

/// An OCI descriptor: what a manifest, an index or `index.json` names a
/// blob by.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Descriptor {
    pub media_type: String,
    pub digest: Digest,
    pub size: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_type: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub annotations: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImageManifest {
    schema_version: u32,
    media_type: String,
    artifact_type: String,
    config: Descriptor,
    layers: Vec<Descriptor>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImageIndex {
    schema_version: u32,
    media_type: String,
    artifact_type: String,
    manifests: Vec<Descriptor>,
    annotations: BTreeMap<String, String>,
}

/// A manifest as the layout writes it: the media type and artifact type
/// its `index.json` entry names, its bytes, and for an artifact the bytes
/// of its config.
#[derive(Debug)]
pub(super) struct Written {
    pub media_type: &'static str,
    pub artifact_type: String,
    pub bytes: Vec<u8>,
    pub config: Option<Vec<u8>>,
}

/// What a manifest's bytes say, before an artifact's config is read.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Read {
    Artifact { config: Digest, layers: Vec<Digest> },
    Index(IndexMeta),
}

/// Whether `artifact_type` is one tailrace writes.
pub(super) fn is_tailraces(artifact_type: &str) -> bool {
    artifact_type == INDEX_TYPE || artifact_type.starts_with(ARTIFACT_TYPE)
}

/// The artifact type of an artifact of `ecosystem`, refused naming an
/// ecosystem holding a character outside RFC 6838's restricted name
/// characters, which no media type carries.
pub(super) fn artifact_type(ecosystem: &Ecosystem, source: &Source) -> Answer<String> {
    let restricted = |c: char| c.is_ascii_alphanumeric() || "!#$&-^_.+".contains(c);
    match ecosystem.as_str().chars().find(|c| !restricted(*c)) {
        Some(c) => Err(Not::refused(
            source,
            Why::Said,
            Some(format!(
                "the ecosystem `{ecosystem}` holds {c:?}, which a media type cannot carry"
            )),
        )),
        None => Ok(format!("{ARTIFACT_TYPE}{ecosystem}")),
    }
}

/// `manifest` in its OCI form, each digest it names described by the size
/// `size` answers: refused naming an ecosystem no media type carries, and
/// absent naming the first digest `size` does not hold (design D4).
pub(super) fn write(
    manifest: &Manifest,
    source: &Source,
    size: &dyn Fn(&Digest) -> Answer<u64>,
) -> Answer<Written> {
    match manifest {
        Manifest::Artifact { meta, layers } => {
            let artifact_type = artifact_type(meta.ecosystem(), source)?;
            let layers = layers
                .iter()
                .map(|digest| {
                    Ok(Descriptor {
                        media_type: OCTETS.into(),
                        digest: digest.clone(),
                        size: size(digest)?,
                        artifact_type: None,
                        annotations: BTreeMap::new(),
                    })
                })
                .collect::<Answer<Vec<_>>>()?;
            let config = canonical(meta);
            let bytes = canonical(&ImageManifest {
                schema_version: 2,
                media_type: IMAGE_MANIFEST.into(),
                artifact_type: artifact_type.clone(),
                config: Descriptor {
                    media_type: META.into(),
                    digest: Digest::of(&config),
                    size: config.len() as u64,
                    artifact_type: None,
                    annotations: BTreeMap::new(),
                },
                layers,
            });
            Ok(Written {
                media_type: IMAGE_MANIFEST,
                artifact_type,
                bytes,
                config: Some(config),
            })
        }
        Manifest::Index(index) => {
            let types = index
                .artifacts()
                .iter()
                .map(|a| artifact_type(a.meta.ecosystem(), source))
                .collect::<Answer<Vec<_>>>()?;
            let manifests = index
                .artifacts()
                .iter()
                .zip(types)
                .map(|(a, artifact_type)| {
                    Ok(Descriptor {
                        media_type: IMAGE_MANIFEST.into(),
                        digest: a.digest.clone(),
                        size: size(&a.digest)?,
                        artifact_type: Some(artifact_type),
                        annotations: BTreeMap::from([
                            (ECOSYSTEM.into(), a.meta.ecosystem().to_string()),
                            (NAME.into(), a.meta.name().to_string()),
                            (VERSION.into(), a.meta.version().to_string()),
                        ]),
                    })
                })
                .collect::<Answer<Vec<_>>>()?;
            let bytes = canonical(&ImageIndex {
                schema_version: 2,
                media_type: IMAGE_INDEX.into(),
                artifact_type: INDEX_TYPE.into(),
                manifests,
                annotations: BTreeMap::from([
                    (COMPONENT.into(), index.component().to_string()),
                    (VERSION.into(), index.version().to_string()),
                ]),
            });
            Ok(Written {
                media_type: IMAGE_INDEX,
                artifact_type: INDEX_TYPE.into(),
                bytes,
                config: None,
            })
        }
    }
}

/// What `bytes`, written as `media_type`, say: an artifact's config and
/// layers, or an index rebuilt from its descriptors' annotations without
/// opening an artifact; the parser's or tailrace's words when they hold
/// no manifest of this form.
pub(super) fn read(media_type: &str, bytes: &[u8]) -> Result<Read, String> {
    match media_type {
        IMAGE_MANIFEST => {
            let manifest: ImageManifest =
                serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
            if manifest.config.media_type != META {
                return Err(format!(
                    "its config is of media type {}, not {META}",
                    manifest.config.media_type
                ));
            }
            Ok(Read::Artifact {
                config: manifest.config.digest,
                layers: manifest.layers.into_iter().map(|l| l.digest).collect(),
            })
        }
        IMAGE_INDEX => {
            let index: ImageIndex = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
            let artifacts = index
                .manifests
                .into_iter()
                .map(|d| {
                    let meta = ArtifactMeta::new(
                        Ecosystem::new(annotation(&d.annotations, ECOSYSTEM)?),
                        EcosystemName::new(annotation(&d.annotations, NAME)?),
                        Version::new(annotation(&d.annotations, VERSION)?),
                    )
                    .map_err(|e| format!("the descriptor of {}: {e}", d.digest))?;
                    Ok(Artifact {
                        digest: d.digest,
                        meta,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            IndexMeta::new(
                ComponentName::new(annotation(&index.annotations, COMPONENT)?),
                Version::new(annotation(&index.annotations, VERSION)?),
                artifacts,
            )
            .map(Read::Index)
            .map_err(|e| e.to_string())
        }
        other => Err(format!("{other} is no manifest's media type")),
    }
}

/// An artifact's metadata, from its config's bytes.
pub(super) fn meta(config: &[u8]) -> Result<ArtifactMeta, String> {
    serde_json::from_slice(config).map_err(|e| e.to_string())
}

fn annotation(annotations: &BTreeMap<String, String>, key: &str) -> Result<String, String> {
    annotations
        .get(key)
        .cloned()
        .ok_or_else(|| format!("it names no annotation {key}"))
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};
    use tailrace_seams::{Seam, Source};

    use super::*;

    fn source() -> Source {
        Source::new(Seam::Store, "OCI layout /tmp/layout")
    }

    fn meta(ecosystem: &str, name: &str) -> ArtifactMeta {
        ArtifactMeta::new(
            Ecosystem::new(ecosystem),
            EcosystemName::new(name),
            Version::new("1.3.0"),
        )
        .unwrap()
    }

    /// The size of a blob as a layout would answer it, here its digest's
    /// first byte.
    fn size(digest: &Digest) -> Answer<u64> {
        Ok(u64::from_str_radix(&digest.hex()[..2], 16).unwrap())
    }

    /// `manifest` written and read back as the layout reads it.
    fn round_trip(manifest: &Manifest) -> (Written, Manifest) {
        let written = write(manifest, &source(), &size).unwrap();
        let back = match read(written.media_type, &written.bytes).unwrap() {
            Read::Index(index) => Manifest::Index(index),
            Read::Artifact { config, layers } => {
                let bytes = written.config.clone().unwrap();
                assert_eq!(config, Digest::of(&bytes));
                Manifest::Artifact {
                    meta: super::meta(&bytes).unwrap(),
                    layers,
                }
            }
        };
        (written, back)
    }

    fn artifact() -> Manifest {
        Manifest::Artifact {
            meta: meta("npm", "@acme/site"),
            layers: vec![Digest::of(b"one"), Digest::of(b"two")],
        }
    }

    fn index() -> Manifest {
        Manifest::Index(
            IndexMeta::new(
                ComponentName::new("web"),
                Version::new("1.3.0"),
                vec![
                    Artifact {
                        digest: Digest::of(b"site"),
                        meta: meta("npm", "@acme/site"),
                    },
                    Artifact {
                        digest: Digest::of(b"image"),
                        meta: meta("oci", "acme/site"),
                    },
                ],
            )
            .unwrap(),
        )
    }

    #[test]
    fn an_artifact_round_trips_to_the_manifest_it_came_from() {
        let (written, back) = round_trip(&artifact());
        assert_eq!(back, artifact());
        assert_eq!(written.media_type, IMAGE_MANIFEST);
        assert_eq!(
            written.artifact_type,
            "application/vnd.agentplot.tailrace.artifact.npm"
        );
        let json: Value = serde_json::from_slice(&written.bytes).unwrap();
        assert_eq!(json["schemaVersion"], 2);
        assert_eq!(json["mediaType"], IMAGE_MANIFEST);
        assert_eq!(json["artifactType"], written.artifact_type);
        assert_eq!(json["config"]["mediaType"], META);
        let config = written.config.unwrap();
        assert_eq!(json["config"]["size"], config.len());
        assert_eq!(
            json["layers"][1],
            json!({
                "mediaType": OCTETS,
                "digest": Digest::of(b"two"),
                "size": size(&Digest::of(b"two")).unwrap(),
            })
        );
    }

    #[test]
    fn an_index_round_trips_to_the_manifest_it_came_from() {
        let (written, back) = round_trip(&index());
        assert_eq!(back, index());
        assert_eq!(written.media_type, IMAGE_INDEX);
        assert_eq!(written.artifact_type, INDEX_TYPE);
        assert_eq!(written.config, None);
        let json: Value = serde_json::from_slice(&written.bytes).unwrap();
        assert_eq!(json["annotations"][COMPONENT], "web");
        assert_eq!(json["annotations"][VERSION], "1.3.0");
        assert_eq!(json["manifests"][1]["mediaType"], IMAGE_MANIFEST);
        assert_eq!(
            json["manifests"][1]["artifactType"],
            "application/vnd.agentplot.tailrace.artifact.oci"
        );
        assert_eq!(json["manifests"][1]["annotations"][NAME], "acme/site");
    }

    #[test]
    fn the_same_manifest_writes_the_same_bytes() {
        for manifest in [artifact(), index()] {
            let first = write(&manifest, &source(), &size).unwrap();
            let second = write(&manifest, &source(), &size).unwrap();
            assert_eq!(first.bytes, second.bytes);
            assert_eq!(first.config, second.config);
        }
    }

    #[test]
    fn an_ecosystem_no_media_type_carries_is_refused_naming_it() {
        let manifest = Manifest::Artifact {
            meta: meta("my eco", "site"),
            layers: vec![],
        };
        match write(&manifest, &source(), &size) {
            Err(Not::Refused(r)) => {
                assert_eq!(r.why, Why::Said);
                assert!(r.said.unwrap().contains("`my eco`"));
            }
            other => panic!("a space in an ecosystem answered {other:?}"),
        }
    }

    #[test]
    fn a_digest_not_held_is_absent_naming_it() {
        let missing = Digest::of(b"two");
        let size = |d: &Digest| {
            if d == &missing {
                Err(Not::absent(&source(), d.to_string()))
            } else {
                Ok(1)
            }
        };
        assert_eq!(
            write(&artifact(), &source(), &size).unwrap_err(),
            Not::absent(&source(), missing.to_string())
        );
    }

    #[test]
    fn a_manifest_of_another_form_is_read_as_none_in_words() {
        let words = read(IMAGE_MANIFEST, b"{}").unwrap_err();
        assert!(words.contains("missing field"), "{words}");
        let words = read(
            "application/vnd.docker.distribution.manifest.v2+json",
            b"{}",
        )
        .unwrap_err();
        assert!(words.contains("no manifest's media type"), "{words}");
    }
}
