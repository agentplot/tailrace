//! The eight seams of tailrace, Part B of the statement in
//! agentplot/blueprints `design/tailrace/requirements.md`: the store (B.1),
//! the build (B.2), the landing (B.3), the signer (B.4), the policy (B.5),
//! a projection (B.6), a target (B.7) and the keeper (B.8). Each is one
//! trait here, which a profile implements and nothing of the core (rule
//! core.2). Nothing is bound yet: bolt 1's units add each trait with the
//! profile that first implements it.

pub mod digest;

pub use digest::*;
