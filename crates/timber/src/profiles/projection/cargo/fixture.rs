//! Crates made in memory for the profile's tests (design D7).

use flate2::write::GzEncoder;
use flate2::Compression;

/// A crate of `name` at `version` whose normalized manifest is
/// `manifest`, beside a library source, as `cargo package` lays one out.
pub(super) fn crate_of(name: &str, version: &str, manifest: &str) -> Vec<u8> {
    packed_crate(name, version, manifest, None)
}

/// [`crate_of`], packed in a git checkout from the package's directory
/// `path_in_vcs` where one is given, as `.cargo_vcs_info.json` names it.
pub(super) fn packed_crate(
    name: &str,
    version: &str,
    manifest: &str,
    path_in_vcs: Option<&str>,
) -> Vec<u8> {
    let dir = format!("{name}-{version}");
    let vcs = path_in_vcs.map(|path| {
        format!(
            "{{\n  \"git\": {{\n    \"sha1\": \"b7e1\"\n  }},\n  \"path_in_vcs\": \"{path}\"\n}}"
        )
    });
    let mut files = vec![
        ("Cargo.toml", manifest),
        ("src/lib.rs", "pub fn hello() {}\n"),
    ];
    if let Some(vcs) = &vcs {
        files.push((".cargo_vcs_info.json", vcs));
    }
    let mut archive = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
    for (file, text) in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(text.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(1_153_704_088);
        header.set_cksum();
        archive
            .append_data(&mut header, format!("{dir}/{file}"), text.as_bytes())
            .unwrap();
    }
    archive.into_inner().unwrap().finish().unwrap()
}
