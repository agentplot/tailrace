//! The profiles this binary binds a seam to (Part C). A profile
//! implements one seam's interface of `tailrace_seams` and nothing of the
//! core, and reaches no other profile (rule core.2); everything that names
//! a registry, a key service or a package manager lives here and in no
//! module outside (rule core.1).

pub mod signer;
pub mod store;
