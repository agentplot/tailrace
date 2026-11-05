//! The projection for npm (C.11; design D6 to D9, D13, D14): a package
//! packed by npm's own packager, a lockfile's pins and a manifest's
//! family ranges read as npm spells them, what a held package requires of
//! the family, a registry read layout of tarballs and packuments, and
//! npm's registry protocol served from that layout (84 to 87, 115, 137,
//! 140). It names npm and nothing else, and reaches the store only
//! through the store seam it is handed (rules core.1, core.2).

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use timber_seams::{
    no_cache_entries, no_lockfile, Answer, Artifact, BuildId, Contents, EcosystemName, Layout,
    Miss, Not, Output, PackAt, Packed, Pin, Projection, Range, Rendered, Request, Requirement,
    Response, Seam, Source, Store, Version,
};

mod declared;
mod family;
#[cfg(test)]
mod fixture;
mod layout;
mod lockfile;
mod pack;
mod serve;
mod tarball;
#[cfg(test)]
mod tests;

/// The ecosystem this projection packs in, as the bindings bind it.
pub const ECOSYSTEM: &str = "npm";

/// The npm projection of one checkout: the checkout whose `.npmrc` names
/// the family, the store it reads tarballs through, and the host's
/// serving process's address, which a family scope's registry begins
/// with.
pub struct NpmProjection {
    root: PathBuf,
    store: Box<dyn Store>,
    origin: String,
    path: Option<OsString>,
}

impl NpmProjection {
    /// The projection of the checkout at `root`, reading tarballs through
    /// `store`, its family the scopes the checkout's `.npmrc` sends to
    /// `origin`, the host's serving process (design D13).
    pub fn new(
        root: impl Into<PathBuf>,
        store: Box<dyn Store>,
        origin: impl Into<String>,
    ) -> NpmProjection {
        NpmProjection {
            root: root.into(),
            store,
            origin: origin.into(),
            path: None,
        }
    }

    /// The projection running npm under `path` as its `PATH`, in place of
    /// the process's own.
    pub fn with_path(self, path: impl Into<OsString>) -> NpmProjection {
        NpmProjection {
            path: Some(path.into()),
            ..self
        }
    }

    /// The projection seam and this profile, as every answer names them.
    pub fn source() -> Source {
        Source::new(Seam::Projection, ECOSYSTEM)
    }
}

impl Projection for NpmProjection {
    /// 84, 87: `output` packed at the component's version by npm's own
    /// packager, its bytes the tarball npm wrote and its name the
    /// package's own; npm promises no same bytes across its versions, so
    /// it answers it cannot promise them (design D6). With no version,
    /// absent naming cache entries: npm packs none.
    fn pack(&self, output: &Output, _: &BuildId, version: Option<&PackAt<'_>>) -> Answer<Packed> {
        let Some(at) = version else {
            return Err(no_cache_entries(&NpmProjection::source()));
        };
        pack::pack(self.path.as_deref(), output, at.version)
    }

    /// 85, 47, 49: a registry read layout at `at` of exactly the
    /// artifacts given, each tarball read through the store and checked
    /// against its layer's digest, and one packument per name listing the
    /// versions given (design D8); absent naming cache entries for cache
    /// entries, which npm packs none of.
    fn render(&self, contents: Contents<'_>, at: &str) -> Answer<Rendered> {
        match contents {
            Contents::Artifacts(artifacts) => layout::render(self.store.as_ref(), artifacts, at),
            Contents::CacheEntries(_) => Err(no_cache_entries(&NpmProjection::source())),
        }
    }

    /// 86, 46, 115, 99: `request`, one of npm's own, answered from the
    /// layout in npm's registry protocol: a packument by its name, from
    /// the layout alone, its tarballs' addresses made absolute under the
    /// request's base, and a tarball by the path its packument names, one
    /// the layout lacks asked of the core once through `miss` (design
    /// D9). Every answer npm reads is a response, a refusal, an absence or
    /// an unreadable answer one whose `error` carries its words.
    fn serve(&self, layout: &Layout, request: &Request, miss: Miss<'_>) -> Answer<Response> {
        serve::serve(layout, request, miss)
    }

    /// 137: every package the checkout's `npm-shrinkwrap.json`, else
    /// its `package-lock.json`, pins, as it spells them, then each range
    /// its root `package.json` and its workspaces' declare for a family
    /// name, of a scope its `.npmrc` sends to the host's serving process,
    /// that the lockfile pins at no version, as the manifest spells it,
    /// each name and range once (design D7, D13); absent naming the
    /// lockfile where it holds no lockfile and no family range.
    fn pinned(&self, checkout: &str) -> Answer<Vec<Pin>> {
        let checkout = Path::new(checkout);
        let pinned = match lockfile::find(checkout) {
            Some(path) => Some(lockfile::pins(&path)?),
            None => None,
        };
        let scopes = family::scopes(checkout, &self.origin)?;
        let declared = declared::declared(checkout, &scopes)?;
        let held: BTreeSet<&EcosystemName> = pinned.iter().flatten().map(|p| &p.name).collect();
        let ranges: Vec<Requirement> = declared
            .into_iter()
            .filter(|r| !held.contains(&r.name))
            .collect();
        if pinned.is_none() && ranges.is_empty() {
            return Err(no_lockfile(&NpmProjection::source()));
        }
        let pins = pinned.into_iter().flatten().map(Pin::Version);
        Ok(pins.chain(ranges.into_iter().map(Pin::Range)).collect())
    }

    /// 140: each family package the `package.json` inside `held`'s
    /// tarball requires, with its range as written: the names of the
    /// scopes the bound checkout's `.npmrc` sends to the host's serving
    /// process, of its `dependencies`, `optionalDependencies` and the
    /// `peerDependencies` npm installs, a bundled name and a spec that is
    /// no range passed over (design D13, D14). The tarball is read
    /// through the store by the digest `held` names and checked against
    /// it; one the store does not hold, or with no `package.json`, is
    /// unreadable naming the artifact.
    fn requires(&self, held: &Artifact) -> Answer<Vec<Requirement>> {
        let scopes = family::scopes(&self.root, &self.origin)?;
        let (_, bytes) = tarball::read(self.store.as_ref(), held)?;
        let manifest = tarball::package_json(held, &bytes)?;
        let manifest: family::Manifest = serde_json::from_slice(&manifest).map_err(|e| {
            Not::unreadable(
                &NpmProjection::source(),
                format!(
                    "the package.json of {} {} at {}",
                    held.meta.name(),
                    held.meta.version(),
                    held.digest
                ),
                Some(e.to_string()),
            )
        })?;
        Ok(manifest.requires(&scopes))
    }

    /// 140, 23: those of `versions` within `range` by npm's own rules,
    /// newest first by npm's own order of versions, a prerelease within
    /// a range only where npm takes it there; a version or a range that
    /// does not parse is within nothing (design D14).
    fn satisfying(&self, range: &Range, versions: &[Version]) -> Answer<Vec<Version>> {
        let Ok(range) = nodejs_semver::Range::parse(range.as_str()) else {
            return Ok(Vec::new());
        };
        let mut within: Vec<(nodejs_semver::Version, &Version)> = versions
            .iter()
            .filter_map(|v| {
                nodejs_semver::Version::parse(v.as_str())
                    .ok()
                    .map(|p| (p, v))
            })
            .filter(|(parsed, _)| range.satisfies(parsed))
            .collect();
        within.sort_by(|(a, _), (b, _)| b.cmp(a));
        within.dedup_by(|(a, _), (b, _)| a == b);
        Ok(within.into_iter().map(|(_, v)| v.clone()).collect())
    }
}
