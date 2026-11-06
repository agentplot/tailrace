//! The clan push profile (C.12's Nix fleet push row), bound as `clan`:
//! one machine of a clan fleet converged on the closure its version
//! names, from the host that holds the machine's key, with clan's own
//! deploy (design D2 to D5).
//!
//! An environment names it in its `target` table, which names the
//! machine as the fleet's inventory names it and the component's output
//! whose closure file names the machine's closure, and nothing else
//! (45, C.10; design D2):
//!
//! ```toml
//! [environments.swancloud-srv.srv]
//! channel = "stable"
//! target.clan = { machine = "swancloud-srv", closure = "swancloud-srv" }
//! ```
//!
//! Apply runs these steps, and nothing runs on the machine before every
//! check before it holds (42, I3; design D3):
//!
//! 1. The verified version's index, read through the profile's own store
//!    by digest, and the revision and the publishing repository it names
//!    (rule core.60); the closure file of the table's output, read by
//!    digest through [`nix_closure`], naming exactly one root.
//! 2. A detached worktree of the checkout at that revision, under
//!    `<common>/timber/fleet/<commit>`, made once and reused.
//! 3. The machine's toplevel evaluated from the worktree's flake, under
//!    `nixosConfigurations` or else `darwinConfigurations`, which must be
//!    the root.
//! 4. The root realised in this host's Nix store from its substituters
//!    alone, `nix-store --realise <root> --option max-jobs 0`, so a path
//!    no substituter holds refuses the apply in Nix's words.
//! 5. One `clan machines update <machine> --flake <worktree> --build-host
//!    localhost --host-key-check strict`, which finds the root realised,
//!    copies it to the machine, uploads its vars and switches it.
//! 6. The marker written on the machine (see [`marker`]), then read back:
//!    the machine must run the root and its marker name the index.
//!
//! clan reaches the machine through the host's own ssh configuration and
//! keys, checking its host key against the host's known hosts and asking
//! no person; timber reads, writes and passes no key and keeps nothing of
//! the session (45, 64, rule core.21).
//!
//! current and report read what the machine runs and the environment's
//! marker in one `clan ssh`, through the checkout's flake, and answer the
//! marker's index where the machine runs the closure the marker names;
//! otherwise they answer unread, naming what runs and what the marker
//! says, and never guess (89, 91, 119; design D4). desired answers the
//! head the core hands it (88).
//!
//! The marker's path names the repository this checkout is of, as its
//! landing's remote names it; an index naming another publishing
//! repository is refused, since its marker would be one current never
//! reads.

mod fleet;
mod marker;
#[cfg(test)]
mod scripted;
mod table;
#[cfg(test)]
mod tests;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use timber_seams::{
    Answer, Digest, Environment, Head, IndexMeta, Not, RepositoryName, Revision, Running, Seam,
    Source, Store, Target, Verified, Why,
};

use super::super::nix_closure::{self, ClosureFile, StorePath};
use crate::profiles::run::{Ran, Tool, GIT_REPOSITORY_ENV};
use fleet::{Clan, Fleet};
use marker::{Marker, MarkerPath, Read};
use table::{Named, Table};

/// The profile name an environment's `target` table names this target by.
pub const PROFILE: &str = "clan";

/// The clan push target over the checkout at `checkout`, reading through
/// a store binding of its own, opened and never made.
pub struct ClanPush {
    store: Box<dyn Store>,
    checkout: PathBuf,
    repository: Answer<RepositoryName>,
    git: Tool,
    fleet: Box<dyn Fleet>,
    source: Source,
}

impl ClanPush {
    /// The target reading through `store`, its markers under
    /// `repository`, the checkout's, running git in the checkout at
    /// `checkout` and the fleet's `clan`, `nix` and `nix-store` as found
    /// on `PATH`.
    pub fn new(
        store: Box<dyn Store>,
        checkout: &Path,
        repository: Answer<RepositoryName>,
    ) -> ClanPush {
        ClanPush::with_fleet(
            store,
            checkout,
            repository,
            Box::new(Clan::new(&ClanPush::source())),
        )
    }

    fn with_fleet(
        store: Box<dyn Store>,
        checkout: &Path,
        repository: Answer<RepositoryName>,
        fleet: Box<dyn Fleet>,
    ) -> ClanPush {
        ClanPush {
            store,
            checkout: checkout.to_path_buf(),
            repository,
            git: Tool::new("git", ClanPush::source()),
            fleet,
            source: ClanPush::source(),
        }
    }

    /// The target seam and this profile, as every answer names them.
    pub fn source() -> Source {
        Source::new(Seam::Target, PROFILE)
    }

    /// Refused, by this target, for `said`.
    fn refused(&self, said: String) -> Not {
        Not::refused(&self.source, Why::Said, Some(said))
    }

    /// Where `environment`'s marker is on its machine, or why there is
    /// none: the checkout's repository unread, or a part of the path
    /// that is no part of one.
    fn marker(&self, environment: &Environment) -> Answer<(RepositoryName, MarkerPath)> {
        let repository = self.repository.clone()?;
        let path = MarkerPath::of(&repository, environment, &self.source)?;
        Ok((repository, path))
    }

    /// The marker `read` holds, where the machine runs the closure it
    /// names; otherwise unread naming what the machine runs and what the
    /// marker says, or that it holds none (89, 119).
    fn held(
        &self,
        environment: &Environment,
        table: &Table,
        path: &MarkerPath,
        read: Read,
    ) -> Answer<Marker> {
        let (machine, runs) = (&table.machine, &read.runs);
        let unread = |said: String| {
            Not::unreadable(
                &self.source,
                format!("what {} runs", Named(environment)),
                Some(said),
            )
        };
        let Some(bytes) = read.marker else {
            return Err(unread(format!(
                "{machine} runs {runs}, and holds no marker of timber's at {path}"
            )));
        };
        let marker: Marker = serde_json::from_slice(&bytes).map_err(|e| {
            Not::unreadable(
                &self.source,
                format!("the marker {path} on {machine}"),
                Some(format!("it is no marker timber wrote: {e}")),
            )
        })?;
        if &marker.closure != runs {
            return Err(unread(format!(
                "{machine} runs {runs}, and its marker at {path} names {}, so it was switched \
                 since timber applied it",
                marker.closure
            )));
        }
        Ok(marker)
    }

    /// What `environment` runs, read through the checkout's flake.
    fn running(&self, environment: &Environment) -> Answer<Marker> {
        let table = Table::read(environment, &self.source)?;
        let (_, path) = self.marker(environment)?;
        let read = self.fleet.read(&self.checkout, &table.machine, &path)?;
        self.held(environment, &table, &path, read)
    }

    /// The one root the closure file of the table's output names in
    /// `index`: absent where the index names no file of the output, and
    /// refused where its file names no root or several (design D3).
    fn root(&self, table: &Table, version: &Verified, index: &IndexMeta) -> Answer<StorePath> {
        let artifact = nix_closure::file_of(index, &table.closure).ok_or_else(|| {
            Not::absent(
                &self.source,
                format!(
                    "the file artifact of the output {} in the index {} of version {}",
                    table.closure,
                    version.index(),
                    version.version()
                ),
            )
        })?;
        let bytes = nix_closure::bytes(self.store.as_ref(), artifact, &self.source)?;
        let roots = nix_closure::roots(artifact, &bytes, &nix_closure::store_dir(), &self.source)?;
        match <[StorePath; 1]>::try_from(roots) {
            Ok([root]) => Ok(root),
            Err(roots) => Err(self.refused(format!(
                "{} names {} roots, {}, and a machine's closure is one",
                ClosureFile(artifact),
                roots.len(),
                roots
                    .iter()
                    .map(StorePath::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ))),
        }
    }

    /// git run in the checkout with `args`, answering when it exits with
    /// one of `allowed`.
    fn git<S: AsRef<OsStr>>(&self, args: &[S], allowed: &[i32]) -> Answer<Ran> {
        let mut all = vec![OsStr::new("-C"), self.checkout.as_os_str()];
        all.extend(args.iter().map(AsRef::as_ref));
        self.git.output(None, &all, GIT_REPOSITORY_ENV, allowed)
    }

    /// The checkout of `revision`: a detached worktree under the
    /// checkout's common git directory, `<common>/timber/fleet/<commit>`,
    /// made by `git worktree add --detach` the first time and reused
    /// after; absent where the checkout holds no such revision (design D3).
    fn worktree(&self, revision: &Revision) -> Answer<PathBuf> {
        let common = self.git(
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
            &[0],
        )?;
        let common = PathBuf::from(String::from_utf8_lossy(&common.stdout).trim_end_matches('\n'));
        let commit = self.git(
            &[
                "rev-parse",
                "--verify",
                "--quiet",
                &format!("{revision}^{{commit}}"),
            ],
            &[0, 1],
        )?;
        if commit.code != Some(0) {
            return Err(Not::absent(
                &self.source,
                format!(
                    "revision {revision} in the checkout at {}",
                    self.checkout.display()
                ),
            ));
        }
        let commit = String::from_utf8_lossy(&commit.stdout).trim().to_string();
        let at = common.join("timber").join("fleet").join(&commit);
        if !at.join(".git").is_file() {
            self.git(
                &[
                    OsStr::new("worktree"),
                    OsStr::new("add"),
                    OsStr::new("--quiet"),
                    OsStr::new("--detach"),
                    OsStr::new("--force"),
                    at.as_os_str(),
                    OsStr::new(&commit),
                ],
                &[0],
            )?;
        }
        Ok(at)
    }
}

/// The words `not` carries: its source's own where it gave any, else what
/// it names.
fn words(not: &Not) -> String {
    match not {
        Not::Unreadable(u) => u.said.clone().unwrap_or_else(|| u.what.clone()),
        Not::Refused(r) => r.said.clone().unwrap_or_else(|| format!("{:?}", r.why)),
        Not::Absent(a) => format!("{} is not there", a.what),
    }
}

impl Target for ClanPush {
    /// 88: the head the core handed, as a target that is pushed to
    /// answers it, once the table is read.
    fn desired(&self, environment: &Environment, head: Head<'_>) -> Answer<Digest> {
        Table::read(environment, &self.source)?;
        head()
    }

    /// 89: the marker's index, where the machine runs the closure the
    /// marker names.
    fn current(&self, environment: &Environment) -> Answer<Digest> {
        self.running(environment).map(|marker| marker.index)
    }

    /// 90: the version's one root, checked against the revision's flake
    /// and realised from substituters alone, switched to with one update
    /// of the machine, then marked and read back (design D3).
    fn apply(&self, environment: &Environment, version: &Verified) -> Answer<()> {
        let table = Table::read(environment, &self.source)?;
        let (repository, path) = self.marker(environment)?;
        let index = nix_closure::index(self.store.as_ref(), version, &self.source)?;
        if let Some(named) = index.repository().filter(|named| **named != repository) {
            return Err(self.refused(format!(
                "the index {} of version {} names the publishing repository {named}, and this \
                 checkout is of {repository}, under which the marker of {} is kept",
                version.index(),
                version.version(),
                Named(environment)
            )));
        }
        let root = self.root(&table, version, &index)?;
        let revision = index.revision().ok_or_else(|| {
            self.refused(format!(
                "the index {} of version {} names no revision, so no flake can be evaluated \
                 where it was published",
                version.index(),
                version.version()
            ))
        })?;
        let worktree = self.worktree(revision)?;
        let machine = &table.machine;
        let evaluated = self.fleet.eval_toplevel(&worktree, machine)?;
        if evaluated != root.as_str() {
            return Err(self.refused(format!(
                "the flake at revision {revision} evaluates the toplevel of {machine} to \
                 {evaluated}, and version {} names {root}",
                version.version()
            )));
        }
        self.fleet.realise(&root)?;
        self.fleet.update(&worktree, machine)?;
        let head = Marker::head(version.index(), version.version(), root.as_str());
        self.fleet
            .mark(&worktree, machine, &path, &head)
            .map_err(|not| {
                Not::unreadable(
                    &self.source,
                    format!("the marker {path} on {machine}"),
                    Some(format!(
                        "the switch to {root} was made, and the marker could not be written: {}",
                        words(&not)
                    )),
                )
            })?;
        let read = self.fleet.read(&worktree, machine, &path)?;
        let marker = self.held(environment, &table, &path, read)?;
        if marker.closure != root.as_str() || &marker.index != version.index() {
            return Err(Not::unreadable(
                &self.source,
                format!("what {} runs", Named(environment)),
                Some(format!(
                    "after the switch to {root} at {}, {machine} runs {} and its marker names {}",
                    version.index(),
                    marker.closure,
                    marker.index
                )),
            ));
        }
        Ok(())
    }

    /// 91: the marker's index and the time it was written, where the
    /// machine runs the closure the marker names.
    fn report(&self, environment: &Environment) -> Answer<Running> {
        self.running(environment).map(|marker| Running {
            digest: marker.index,
            converged_at: marker.at,
        })
    }
}
