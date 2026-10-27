//! Crates made in memory for the profile's tests (design D7).

use flate2::write::GzEncoder;
use flate2::Compression;

/// A crate of `name` at `version` whose normalized manifest is
/// `manifest`, beside a library source, as `cargo package` lays one out.
pub(super) fn crate_of(name: &str, version: &str, manifest: &str) -> Vec<u8> {
    let dir = format!("{name}-{version}");
    let mut archive = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
    for (file, text) in [
        ("Cargo.toml", manifest),
        ("src/lib.rs", "pub fn hello() {}\n"),
    ] {
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
