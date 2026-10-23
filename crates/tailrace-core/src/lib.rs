//! The core: what tailrace does with its seams, Part A of the statement in
//! agentplot/blueprints `design/tailrace/requirements.md`. Code here names
//! no registry, task runner, merge queue, key service, package manager,
//! deploy target or forge (rule core.1); it reaches each through the one
//! trait of `tailrace_seams`. Empty until bolt 1's units bring the policy,
//! the version rule and publish.

pub mod landed;

pub use landed::*;
