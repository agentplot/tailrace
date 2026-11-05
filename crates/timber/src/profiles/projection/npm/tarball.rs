//! An npm artifact's tarball, read through the store seam by the one
//! layer its manifest names and checked against that layer's digest, and
//! the `package.json` inside it, read with `flate2` and `tar` and never
//! written back (C.11, rule core.5).

use std::io::Read as _;

use flate2::read::GzDecoder;
use timber_seams::{Answer, Artifact, Digest, Held, Manifest, Not, Store, Why};

use super::NpmProjection;

/// Where a packed npm tarball holds its manifest.
const PACKAGE_JSON: &str = "package/package.json";

/// `artifact` unreadable, naming it, with `said`.
fn unreadable(artifact: &Artifact, said: String) -> Not {
    Not::unreadable(
        &NpmProjection::source(),
        format!(
            "the tarball of {} {} at {}",
            artifact.meta.name(),
            artifact.meta.version(),
            artifact.digest
        ),
        Some(said),
    )
}

/// The tarball `artifact` names, read through `store`: its layer's digest
/// and its bytes. A manifest or layer the store does not hold, or a
/// manifest naming other than one layer, is unreadable naming the
/// artifact (58); bytes whose digest is not the layer's are refused
/// naming both (rule core.5); a read the store cannot answer is its own
/// answer.
pub(super) fn read(store: &dyn Store, artifact: &Artifact) -> Answer<(Digest, Vec<u8>)> {
    let layers = match store.read(&artifact.digest) {
        Ok(Held::Manifest(Manifest::Artifact { layers, .. })) => layers,
        Ok(_) => {
            return Err(unreadable(
                artifact,
                format!("{} holds no artifact's manifest", artifact.digest),
            ))
        }
        Err(Not::Absent(_)) => {
            return Err(unreadable(
                artifact,
                format!("{} is not held", artifact.digest),
            ))
        }
        Err(not) => return Err(not),
    };
    let [layer] = layers.as_slice() else {
        return Err(unreadable(
            artifact,
            format!(
                "its manifest names {} layers, and an npm artifact one tarball",
                layers.len()
            ),
        ));
    };
    let bytes = match store.read(layer) {
        Ok(Held::Bytes(bytes)) => bytes,
        Ok(Held::Manifest(_)) => {
            return Err(unreadable(
                artifact,
                format!("{layer} holds a manifest and not a tarball"),
            ))
        }
        Err(Not::Absent(_)) => return Err(unreadable(artifact, format!("{layer} is not held"))),
        Err(not) => return Err(not),
    };
    let answered = Digest::of(&bytes);
    if &answered != layer {
        return Err(Not::refused(
            &NpmProjection::source(),
            Why::OtherBytes {
                expected: layer.clone(),
                answered,
            },
            None,
        ));
    }
    Ok((layer.clone(), bytes))
}

/// The bytes of `package/package.json` in the tarball `bytes` of
/// `artifact`: unreadable naming the artifact where it holds none or
/// cannot be read as a gzipped tar, with the reader's words.
pub(super) fn package_json(artifact: &Artifact, bytes: &[u8]) -> Answer<Vec<u8>> {
    let read = || -> std::io::Result<Option<Vec<u8>>> {
        let mut archive = tar::Archive::new(GzDecoder::new(bytes));
        for entry in archive.entries()? {
            let mut entry = entry?;
            if entry.path()?.to_str() == Some(PACKAGE_JSON) {
                let mut manifest = Vec::new();
                entry.read_to_end(&mut manifest)?;
                return Ok(Some(manifest));
            }
        }
        Ok(None)
    };
    match read() {
        Ok(Some(manifest)) => Ok(manifest),
        Ok(None) => Err(unreadable(artifact, format!("it holds no {PACKAGE_JSON}"))),
        Err(e) => Err(unreadable(artifact, e.to_string())),
    }
}
