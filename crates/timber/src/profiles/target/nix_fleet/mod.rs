//! The Nix fleet's target profiles (C.12's Nix fleet rows): a fleet's
//! machines, each an environment, converged on the one closure a version
//! names. [`push`] converges one machine from the host that holds its
//! key, with the fleet's own deploy tool; a pull profile, each machine
//! reading its own channel, takes its place beside it (44, 126). Each
//! reads the version's closure through the family's
//! [`nix_closure`](super::nix_closure), so none calls another (rule
//! core.2; design D2).

pub mod push;
