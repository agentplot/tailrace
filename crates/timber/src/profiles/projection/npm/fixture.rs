//! Tarballs a test makes, as `npm pack` lays them out, and their
//! artifacts put in a store, so the projection's tests need no npm.

use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::Value;
use timber_seams::{Artifact, ArtifactMeta, Ecosystem, EcosystemName, Manifest, Store, Version};

use super::ECOSYSTEM;

/// A gzipped tar holding `package/package.json` as `manifest` and a
/// `package/index.js`.
pub fn tarball(manifest: &Value) -> Vec<u8> {
    let mut archive = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
    for (path, bytes) in [
        (
            "package/package.json",
            serde_json::to_vec_pretty(manifest).unwrap(),
        ),
        ("package/index.js", b"module.exports = {};\n".to_vec()),
    ] {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(499_162_500);
        header.set_cksum();
        archive
            .append_data(&mut header, path, bytes.as_slice())
            .unwrap();
    }
    archive.into_inner().unwrap().finish().unwrap()
}

/// The artifact of `bytes` as `name` at `version`, its tarball and its
/// manifest put in `store`.
pub fn artifact_of(store: &dyn Store, name: &str, version: &str, bytes: &[u8]) -> Artifact {
    let layer = store.put_bytes(bytes).unwrap().digest;
    let meta = ArtifactMeta::new(
        Ecosystem::new(ECOSYSTEM),
        EcosystemName::new(name),
        Version::new(version),
    )
    .unwrap();
    let digest = store
        .put_artifact(&Manifest::Artifact {
            meta: meta.clone(),
            layers: vec![layer],
        })
        .unwrap()
        .digest;
    Artifact { digest, meta }
}

/// `manifest`, a package's at its `name` and `version`, packed and put
/// in `store`.
pub fn held(store: &dyn Store, manifest: &Value) -> Artifact {
    let name = manifest["name"].as_str().unwrap();
    let version = manifest["version"].as_str().unwrap();
    artifact_of(store, name, version, &tarball(manifest))
}
