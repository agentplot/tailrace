//! A small OCI image archive, as `docker save` writes one since Docker
//! Engine 25, made from fixed bytes so it is the same archive on every
//! host and the tree holds no binary file (design D5): a POSIX ustar
//! archive whose members are each written as one 512-byte header (the
//! name, mode `0000644` or `0000755`, uid and gid `0`, the size in
//! octal, mtime `0`, the checksum, the type `0` for a file or `5` for a
//! directory, magic `ustar` and version `00`) followed by the bytes in
//! 512-byte blocks, ended by two zero blocks. It holds `blobs/sha256/`
//! with a layer (an empty tar), the image's config and its manifest,
//! then `index.json` naming the manifest, docker's own `manifest.json`,
//! and `oci-layout`.
//!
//! The image projection's unit tests read this file by its path, and the
//! command line's tests as `fixtures::image`.

#![allow(dead_code)]

use tailrace_seams::Digest;

const BLOCK: usize = 512;

/// One member of an archive: its name, and its bytes, or `None` for a
/// directory.
pub type Member<'a> = (&'a str, Option<&'a [u8]>);

/// The ustar archive of `members`, in order.
pub fn tar(members: &[Member]) -> Vec<u8> {
    let mut out = Vec::new();
    for (name, bytes) in members {
        let mut header = [0u8; BLOCK];
        assert!(name.len() <= 100, "{name} is longer than a ustar name");
        header[..name.len()].copy_from_slice(name.as_bytes());
        let (mode, kind, size) = match bytes {
            Some(bytes) => (b"0000644\0", b'0', bytes.len()),
            None => (b"0000755\0", b'5', 0),
        };
        header[100..108].copy_from_slice(mode);
        header[108..116].copy_from_slice(b"0000000\0");
        header[116..124].copy_from_slice(b"0000000\0");
        header[124..136].copy_from_slice(format!("{size:011o}\0").as_bytes());
        header[136..148].copy_from_slice(b"00000000000\0");
        header[148..156].copy_from_slice(b"        ");
        header[156] = kind;
        header[257..263].copy_from_slice(b"ustar\0");
        header[263..265].copy_from_slice(b"00");
        let sum: u32 = header.iter().map(|b| u32::from(*b)).sum();
        header[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
        out.extend_from_slice(&header);
        if let Some(bytes) = bytes {
            out.extend_from_slice(bytes);
            out.resize(out.len().div_ceil(BLOCK) * BLOCK, 0);
        }
    }
    out.resize(out.len() + 2 * BLOCK, 0);
    out
}

fn sha256(bytes: &[u8]) -> String {
    Digest::of(bytes).hex().to_string()
}

/// The image archive.
pub fn archive() -> Vec<u8> {
    let layer = tar(&[]);
    let config = br#"{"architecture":"amd64","os":"linux","rootfs":{"type":"layers","diff_ids":["sha256:LAYER"]},"config":{}}"#
        .to_vec();
    let config = String::from_utf8(config)
        .unwrap()
        .replace("LAYER", &sha256(&layer))
        .into_bytes();
    let manifest = format!(
        r#"{{"schemaVersion":2,"mediaType":"application/vnd.oci.image.manifest.v1+json","config":{{"mediaType":"application/vnd.oci.image.config.v1+json","digest":"sha256:{}","size":{}}},"layers":[{{"mediaType":"application/vnd.oci.image.layer.v1.tar","digest":"sha256:{}","size":{}}}]}}"#,
        sha256(&config),
        config.len(),
        sha256(&layer),
        layer.len()
    )
    .into_bytes();
    let index = format!(
        r#"{{"schemaVersion":2,"mediaType":"application/vnd.oci.image.index.v1+json","manifests":[{{"mediaType":"application/vnd.oci.image.manifest.v1+json","digest":"sha256:{}","size":{},"annotations":{{"io.containerd.image.name":"example.org/api:latest","org.opencontainers.image.ref.name":"latest"}}}}]}}"#,
        sha256(&manifest),
        manifest.len()
    )
    .into_bytes();
    let docker = format!(
        r#"[{{"Config":"blobs/sha256/{}","RepoTags":["example.org/api:latest"],"Layers":["blobs/sha256/{}"]}}]"#,
        sha256(&config),
        sha256(&layer)
    )
    .into_bytes();
    let blob = |bytes: &[u8]| format!("blobs/sha256/{}", sha256(bytes));
    let (layer_at, config_at, manifest_at) = (blob(&layer), blob(&config), blob(&manifest));
    tar(&[
        ("blobs/", None),
        ("blobs/sha256/", None),
        (&layer_at, Some(&layer)),
        (&config_at, Some(&config)),
        (&manifest_at, Some(&manifest)),
        ("index.json", Some(&index)),
        ("manifest.json", Some(&docker)),
        ("oci-layout", Some(br#"{"imageLayoutVersion":"1.0.0"}"#)),
    ])
}

/// A legacy docker archive: `manifest.json` and no OCI image layout.
pub fn legacy() -> Vec<u8> {
    tar(&[(
        "manifest.json",
        Some(br#"[{"Config":"config.json","RepoTags":[],"Layers":[]}]"#),
    )])
}

/// `bytes` compressed with gzip, in one stored deflate block per 65535
/// bytes, as `gzip -1` of incompressible bytes would hold them.
pub fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut out = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 3];
    let chunks: Vec<&[u8]> = bytes.chunks(0xffff).collect();
    for (i, chunk) in chunks.iter().enumerate() {
        out.push(u8::from(i + 1 == chunks.len()));
        let len = chunk.len() as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(chunk);
    }
    out.extend_from_slice(&crc32(bytes).to_le_bytes());
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out
}

/// The CRC-32 gzip ends with, bit by bit.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}
