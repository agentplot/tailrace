//! The one conformance suite that holds every profile to Parts A and B of
//! the statement (93): a profile that fails it is not one. Every scenario
//! cites the clauses it exercises (rule core.33). It runs over the fakes
//! here and over each real profile in that profile's own tests.

pub mod scenario;

pub use scenario::*;
