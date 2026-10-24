//! The projection seam's profiles (B.6), each by the ecosystem it packs
//! in, the binding of every one built, and what they share: an output's
//! one file, read with the operating system's words. The policy file's
//! `[projections]` picks among them for an output; the moon cache packs
//! no artifact, so a publish naming it for one is answered absent (53).

pub mod file;
pub mod image;
pub mod moon_cache;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use tailrace_seams::{
    Answer, ArtifactMeta, Ecosystem, EcosystemName, Not, Output, Projection, Source, Store,
    Version, Why,
};

use file::FileProjection;
use image::ImageProjection;
use moon_cache::MoonCache;

/// Every projection built, by the ecosystem it packs in (92, C.10,
/// C.11): the moon cache of the checkout at `checkout`, reading cache
/// entries through `store` (design D6).
pub fn bound(store: Box<dyn Store>, checkout: &Path) -> BTreeMap<Ecosystem, Box<dyn Projection>> {
    BTreeMap::from([
        (
            Ecosystem::new(file::ECOSYSTEM),
            Box::new(FileProjection) as Box<dyn Projection>,
        ),
        (Ecosystem::new(image::ECOSYSTEM), Box::new(ImageProjection)),
        (
            Ecosystem::new(moon_cache::ECOSYSTEM),
            Box::new(MoonCache::new(checkout, store)),
        ),
    ])
}

/// The one file `output` left; refused naming the output and every file
/// it left where it left none or several, since `projection` packs one.
fn one_file<'a>(output: &'a Output, source: &Source, projection: &str) -> Answer<&'a str> {
    match output.files.as_slice() {
        [one] => Ok(one),
        files => {
            let left = match files {
                [] => "no file".to_string(),
                several => format!("{} files, {}", several.len(), several.join(", ")),
            };
            Err(Not::refused(
                source,
                Why::Said,
                Some(format!(
                    "the output {} left {left}, and the {projection} projection packs one",
                    output.name
                )),
            ))
        }
    }
}

/// The bytes at `path`, or unreadable naming it with the operating
/// system's words (58, 59).
fn read(path: &str, source: &Source) -> Answer<Vec<u8>> {
    fs::read(path).map_err(|e| Not::unreadable(source, path, Some(e.to_string())))
}

/// The metadata of `output` packed in `ecosystem` at `version`: the
/// output's name, which the authors declare, and not its file's, which
/// the build chose (4).
fn meta(
    ecosystem: &str,
    output: &Output,
    version: &Version,
    source: &Source,
) -> Answer<ArtifactMeta> {
    ArtifactMeta::new(
        Ecosystem::new(ecosystem),
        EcosystemName::new(output.name.as_str()),
        version.clone(),
    )
    .map_err(|bad| Not::refused(source, Why::Said, Some(bad.to_string())))
}

/// What a projection that renders nothing yet answers for `operation`.
fn not_built(operation: &str, projection: &str, source: &Source) -> Not {
    Not::absent(
        source,
        format!("{operation} of the {projection} projection"),
    )
}
