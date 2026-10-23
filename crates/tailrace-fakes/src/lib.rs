//! In-memory profiles of every seam, one per trait of `tailrace_seams`,
//! scriptable per test so the core is tested with no registry, build,
//! landing, signer or target on the host. The conformance suite runs over
//! these and over every real profile alike (93, rule core.33).

pub mod build;
pub mod keeper;
pub mod landing;
pub mod policy;
pub mod projection;
pub mod script;
pub mod signer;
pub mod store;
pub mod target;

pub use build::*;
pub use keeper::*;
pub use landing::*;
pub use policy::*;
pub use projection::*;
pub use script::*;
pub use signer::*;
pub use store::*;
pub use target::*;
