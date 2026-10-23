//! The eight seams of tailrace, Part B of the statement in
//! agentplot/blueprints `design/tailrace/requirements.md`: the store (B.1),
//! the build (B.2), the landing (B.3), the signer (B.4), the policy (B.5),
//! a projection (B.6), a target (B.7) and the keeper (B.8). Each is one
//! trait here, which a profile implements and nothing of the core (rule
//! core.2). Nothing is bound yet: bolt 1's units add each trait with the
//! profile that first implements it.

/// A name the statement gives a thing, held as its text.
macro_rules! text {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(
            Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(text: impl Into<String>) -> Self {
                $name(text.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

pub mod artifact;
pub mod digest;

pub use artifact::*;
pub use digest::*;
