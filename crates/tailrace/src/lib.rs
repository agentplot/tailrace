//! The `tailrace` binary: the command line, tailrace's first surface.
//! `tailrace publish`, `tailrace verify`, `tailrace status`, `tailrace
//! sync`, `tailrace push`, `tailrace rebuild`, `tailrace promote` and
//! `tailrace policy` each turn their arguments into one question to the one core and render its
//! answer, as text for a person or, with `--json`, as the answer itself
//! for a program; none reads a seam, the record or the history itself,
//! or computes a version, a digest, a signer or a channel of its own (56,
//! 57, rule core.31). Absent, refused and unreadable are three answers
//! with three exit codes, and a refusal prints the source's own words
//! beside tailrace's (58, 59, rule core.19).
//!
//! `publish`, `verify`, `sync`, `push`, `rebuild` and `promote` are bound
//! by the host's bindings file (118) and the repository's policy file
//! ([`bindings`]): on a host with no bindings file they answer absent
//! naming it, and a broken one unreadable, each in its own form and with
//! its own exit. `status` is bound to the landing, the policy and the
//! ledger alone, and on a host with no bindings file as 118 binds such a
//! host; a binding it cannot make is its channels' answer, and the
//! host's record answers beside it (60). `policy` is bound as status is,
//! with no store and no signer, and a binding it cannot make is its
//! answer. `tailrace serve` is the host's one serving process, tailrace's
//! second surface: it binds nothing itself, and answers each request from
//! the one core over the binding of the checkout that named the
//! repository (57, 114, 132).

use std::io::Write;
use std::path::Path;

use clap::{Parser, Subcommand};
use tailrace_seams::{Answer, Not, Seam, Seams, SignerId, Source};

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
    /// Where each channel is and the promotions that put it there, from
    /// the ledger, beside the versions this host published, from its own
    /// record.
    Status(commands::status::StatusArgs),
    /// Render the cache entries a landed revision's passing verdicts name
    /// into this checkout, where its tools read them.
    Sync(commands::sync::SyncArgs),
    /// Write the cache entries of a landing's passing builds to the store,
    /// from what its build left on this host, after the landing.
    Push(commands::push::PushArgs),
    /// Rebuild this host's record of the versions it published from the
    /// store and git.
    Rebuild(commands::rebuild::RebuildArgs),
    /// Move a channel to a version of its component, judged under the
    /// channel's policy and held by the ledger before the store follows,
    /// or back to the version it moved from.
    Promote(commands::promote::PromoteArgs),
    /// The policy in force at the branch's landed head: what acceptance
    /// and a layout require, and every channel, the givens among them,
    /// with how each moves, its claims and its promoters.
    Policy(commands::policy::PolicyArgs),
    /// The host's one serving process: every repository a checkout named,
    /// each as a page and as JSON, at the core's address, until idle.
    Serve(commands::serve::ServeArgs),
}

/// What a host binds (92): the seams; the store as the host's bindings
/// name it, which the host's record keeps of each version (61, 118);
/// the identity of the party running tailrace, which signs what it
/// observed (11, 79), `None` on a host that binds no key and only
/// verifies; and what tells the host's gate of a version a publish
/// offered, which only publish reads (130).
pub struct Binding {
    pub seams: Seams,
    pub store: Source,
    pub signer: Option<SignerId>,
    pub gate: bindings::Teller,
}

/// `cli` asked in the repository at `at`, over the binding the host's
/// file made or why it made none, its answer written to `out`; answers
/// the exit code.
pub fn run(cli: &Cli, binding: &Answer<Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    match &cli.command {
        Command::Publish(args) => commands::publish::run(args, binding, at, out),
        Command::Verify(args) => commands::verify::run(args, binding, at, out),
        Command::Status(args) => commands::status::run(args, binding, at, out),
        Command::Sync(args) => commands::sync::run(args, binding, at, out),
        Command::Push(args) => commands::push::run(args, binding, at, out),
        Command::Rebuild(args) => commands::rebuild::run(args, binding, at, out),
        Command::Promote(args) => commands::promote::run(args, binding, at, out),
        Command::Policy(args) => commands::policy::run(args, binding, at, out),
        Command::Serve(args) => commands::serve::run(args, out),
    }
}

/// 79: what publish and promote answer where the host binds no signer,
/// before anything is read: each signs what it makes, and no identity is
/// at hand.
pub(crate) fn unsigned() -> Not {
    Not::absent(
        &Source::new(Seam::Host, "bindings"),
        "binding of the signer, [signing] in the bindings file",
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
