//! The `tailrace` binary: the command line, tailrace's first surface.
//! `tailrace publish`, `tailrace verify` and `tailrace status` each turn
//! their arguments into one question to the one core and render its
//! answer, as text for a person or, with `--json`, as the answer itself
//! for a program; none reads a seam, the record or the history itself,
//! or computes a version, a digest or a signer of its own (56, 57, rule
//! core.31). Absent, refused and unreadable are three answers with three
//! exit codes, and a refusal prints the source's own words beside
//! tailrace's (58, 59, rule core.19).
//!
//! Until the host binds its seams (118), the installed binary answers
//! `publish` and `verify` absent, naming the seams no binding holds;
//! `status` needs no seam and reads the host's record (60).

use std::io::Write;
use std::path::Path;

use clap::{Parser, Subcommand};
use tailrace_seams::{Not, Seam, Seams, SignerId, Source};

pub mod bindings;
pub mod commands;
pub mod profiles;
pub mod render;

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

/// Build once, sign everything, promote with proof, in your own account.
#[derive(Debug, Parser)]
#[command(name = "tailrace", version = VERSION_LINE, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Offer a landed revision's artifacts as a version of each component
    /// it publishes, each with its provenance, the index last.
    Publish(commands::publish::PublishArgs),
    /// Tell which signer the policy names made each attestation on a
    /// version, and whether each counts.
    Verify(commands::verify::VerifyArgs),
    /// The versions this host published, from its own record.
    Status(commands::status::StatusArgs),
}

/// What a host binds (92): the seams, and the identity of the party
/// running tailrace, which signs what it observed (11, 79).
pub struct Binding {
    pub seams: Seams,
    pub signer: SignerId,
}

/// `cli` asked in the repository at `at`, over `binding` where the host
/// binds one, its answer written to `out`; answers the exit code.
pub fn run(cli: &Cli, binding: Option<&Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    match &cli.command {
        Command::Publish(args) => commands::publish::run(args, binding, at, out),
        Command::Verify(args) => commands::verify::run(args, binding, at, out),
        Command::Status(args) => commands::status::run(args, at, out),
    }
}

/// 118: what a command that needs the seams answers on a host that binds
/// none.
pub(crate) fn unbound() -> Not {
    Not::absent(
        &Source::new(Seam::Host, "unbound"),
        "binding of the store, the build, the landing, the signer, the policy, \
         the projections, the targets or the keeper",
    )
}

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
