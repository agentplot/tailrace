//! Consumer layouts: each version a checkout's lockfile pins, found in
//! the store by its ecosystem, name and version alone, judged under the
//! layout's policy, and rendered into the checkout when its claims
//! satisfy it (47 to 51, 113, 138; design D2). A pin is resolved
//! through the name `artifact/<ecosystem>/<name>@<version>` to the
//! version's index, and the artifact of that ecosystem and name found in
//! the index by its metadata; the digests resolved are the ones acted on
//! (rule core.4). A name the store does not hold is absent, as most of a
//! lockfile's pins are, and never a refusal (51, 58).

use tailrace_seams::{
    Answer, Artifact, Digest, Ecosystem, EcosystemName, Held, Manifest, Name, Not, Pinned, Seam,
    Source, Store, Version,
};

use crate::BOUND;

/// What every name of a version's index by ecosystem, name and version
/// begins with (138).
pub const ARTIFACT: &str = "artifact/";

/// 138: the store's name for the index of `version` of `name` in
/// `ecosystem`.
pub fn artifact_name(ecosystem: &Ecosystem, name: &EcosystemName, version: &Version) -> Name {
    Name::new(format!("{ARTIFACT}{ecosystem}/{name}@{version}"))
}

/// A pin's own answer inside the answer of the reads it took. A read its
/// source answered unreadable or refused is the outer answer, and leaves
/// the layout as it was (51; design D6). Absent, refused under the
/// policy, and unreadable for what the core found in what the store held
/// are the pin's own, inner answer, and every other pin is judged and
/// rendered all the same.
pub type Pinning<T> = Answer<Answer<T>>;

/// 138, 4: a pin found: the index its name resolved to, and the one
/// artifact of the pin's ecosystem, name and version the index names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Located {
    pub index: Digest,
    pub artifact: Artifact,
}

/// 138, 4: `pin` of `ecosystem` resolved to its index and the artifact in
/// it (rule core.4). A name the store does not hold is absent naming the
/// name (51); a name at a digest holding no index, or at an index naming
/// no one artifact of that ecosystem, name and version, is unreadable
/// naming the name and the digest. A resolve or read its source answers
/// unreadable or refused is the outer answer.
pub fn locate(store: &dyn Store, ecosystem: &Ecosystem, pin: &Pinned) -> Pinning<Located> {
    let name = artifact_name(ecosystem, &pin.name, &pin.version);
    let index = match store.resolve(&name) {
        Ok(index) => index,
        Err(Not::Absent(a)) => return Ok(Err(Not::absent(&a.source, format!("name {name}")))),
        Err(not) => return Err(not),
    };
    let not_found = |said: String| {
        Not::unreadable(
            &Source::new(Seam::Store, BOUND),
            format!("{name} at {index}"),
            Some(said),
        )
    };
    let meta = match store.read(&index) {
        Ok(Held::Manifest(Manifest::Index(meta))) => meta,
        Ok(_) => return Ok(Err(not_found(format!("{index} holds no index")))),
        Err(Not::Absent(_)) => return Ok(Err(not_found(format!("{index} is not held")))),
        Err(not) => return Err(not),
    };
    let named: Vec<&Artifact> = meta
        .artifacts()
        .iter()
        .filter(|a| {
            a.meta.ecosystem() == ecosystem
                && a.meta.name() == &pin.name
                && a.meta.version() == &pin.version
        })
        .collect();
    match named.as_slice() {
        [artifact] => Ok(Ok(Located {
            index,
            artifact: (*artifact).clone(),
        })),
        none_or_several => Ok(Err(not_found(format!(
            "{index} names {} artifacts of {ecosystem} {} {}, and a pin takes one",
            none_or_several.len(),
            pin.name,
            pin.version
        )))),
    }
}
