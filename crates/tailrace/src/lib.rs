//! The `tailrace` binary: every surface asks this one core (56, rule
//! core.31). The command line is the first surface; nothing of the design
//! is built yet beyond what the binary says of itself.

/// The revision this binary was built from. A binary carries the revision
/// and the build that made it, never a version (24, C.11): the release's
/// version is its tag's, and `tailrace status` says which version a
/// revision became.
pub const REVISION: &str = env!("TAILRACE_REVISION");

/// The build that made these bytes: the compiler and the target.
pub const BUILD: &str = env!("TAILRACE_BUILD");

/// What `tailrace --version` prints after the name.
pub const VERSION_LINE: &str = concat!(
    "revision ",
    env!("TAILRACE_REVISION"),
    ", built by ",
    env!("TAILRACE_BUILD")
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_binary_names_a_revision_and_no_version() {
        assert!(!REVISION.is_empty());
        assert!(VERSION_LINE.starts_with("revision "));
        assert!(!VERSION_LINE.contains("0.0.0"));
    }
}
