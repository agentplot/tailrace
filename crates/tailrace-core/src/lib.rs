//! The core: what tailrace does with its seams, Part A of the statement in
//! agentplot/blueprints `design/tailrace/requirements.md`. Code here names
//! no registry, task runner, merge queue, key service, package manager,
//! deploy target or forge (rule core.1); it reaches each through the one
//! trait of `tailrace_seams`. Empty until bolt 1's units bring the policy,
//! the version rule and publish.

pub mod declared;
pub mod history;
pub mod landed;
pub mod provenance;
pub mod publish;
pub mod published;
pub mod status;
pub mod version;

pub use declared::*;
pub use history::*;
pub use landed::*;
pub use provenance::*;
pub use publish::*;
pub use published::*;
pub use status::*;
pub use version::*;
