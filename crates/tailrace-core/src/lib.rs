//! The core: what tailrace does with its seams, Part A of the statement in
//! agentplot/blueprints `design/tailrace/requirements.md`. Code here names
//! no registry, task runner, merge queue, key service, package manager,
//! deploy target or forge (rule core.1); it reaches each through the one
//! trait of `tailrace_seams`, and the repository's history through git,
//! which section 9 fixes.
//!
//! | module | what it keeps | statement, rules |
//! |---|---|---|
//! | [`landed`] | a revision is read only once the landing seam says its branch landed it, and the policy for an audience only at such a revision | 14, 65, 77, 82, 83; core.11, core.22 |
//! | [`history`] | the files, commits and tags of a landed revision, from the local history and the shared one | 21, 22, 65, 123 |
//! | [`declared`] | the components and the version rule, read from the policy file at the landed revision | 65, 123, C.10; core.22 |
//! | [`version`] | a version per component from the history alone, the components a landing touched, and the claim of the version's name | 21 to 24, 124; core.6, core.13, core.37 |
//!
//! Every read and the claim take a [`Landed`], which only the landing
//! seam's yes makes, and answer an `Answer`: absent, refused and
//! unreadable kept apart, with the source's own words (58, 59, rule
//! core.19). Deriving and claiming ask nothing of the build seam, so no
//! build learns its version (24).

pub mod declared;
pub mod history;
pub mod landed;
pub mod provenance;
pub mod publish;
pub mod published;
pub mod status;
pub mod verify;
pub mod version;

pub use declared::*;
pub use history::*;
pub use landed::*;
pub use provenance::*;
pub use publish::*;
pub use published::*;
pub use status::*;
pub use verify::*;
pub use version::*;
