//! The core: what tailrace does with its seams, Part A of the statement in
//! agentplot/blueprints `design/tailrace/requirements.md`. Code here names
//! no registry, task runner, merge queue, key service, package manager,
//! deploy target or forge (rule core.1); it reaches each through the one
//! trait of `tailrace_seams`, and the repository's history through git,
//! which section 9 fixes.
//!
//! | module | what it keeps | statement, rules |
//! |---|---|---|
//! | [`landed`] | a revision is read only once the landing seam says its branch landed it, and the policy for an audience only at such a revision | 14, 65, 77, 82, 83; core.11, core.22 |
//! | [`history`] | the files, commits and tags of a landed revision, from the local history and the shared one; a commit's message from the local history alone; a branch's head and the newest revision of it a checkout descends from, a revision resolved to its commit, the working tree's root, the common directory every worktree shares, and each worktree's own git directory | 21, 22, 50, 62, 65, 123, 134; core.4 |
//! | [`declared`] | the components, the version rule and the projection each output is packed by, read from the policy file at the landed revision | 65, 123, C.10; core.22 |
//! | [`version`] | a version per component from the history alone, the commits it is derived from, the components a landing touched, and the claim of the version's name | 21 to 24, 124, 134; core.6, core.13, core.37 |
//! | [`notes`] | a version's notes, the commits it is derived from rendered by the rule's own tool in one fixed form, and the ecosystem reserved for them | 21, 134, section 9 |
//! | [`provenance`] | the claim each artifact carries: what the host observed of its build, and the landing's verdicts carried as made | 11 to 13; core.8, core.9 |
//! | [`publish`](mod@publish) | a landed revision offered as a version per component: judged before the name is claimed, each artifact put and attested, the index last and carrying the layout line the publisher's file writes, a repeat ending where the first did, and the answer a program reads | 2, 5, 6, 8, 14 to 20, 57, 74, 75, 78, 84, 111, 113, 124; core.5, core.6, core.8 to core.13, core.27 |
//! | [`published`] | the host's record of the versions it published, a file per version a reader opens, naming the commits each version's notes were rendered from and never their text | 61, 62, 64, 134; core.20, core.21 |
//! | [`status`](mod@status) | which versions this host published, answered from the record alone, each with its notes rendered again from the repository's own history | 23, 58, 60, 61, 132, 134; core.19, core.32 |
//! | [`record`] | the host's record rebuilt from the store and git: an entry per tag the branch landed whose name the store resolves to its component's index at that version, its notes' commits derived again at the tag's revision, every read before any write, nothing rewritten or deleted | 22, 58 to 63, 134; core.4, core.19, core.20 |
//! | [`copy`](mod@copy) | a version copied from one store to another through the store seam, every attestation going with its digest, the index last and the name made from nothing, and no channel | 1 to 3, 6, 8, 103; core.4, core.6, core.12, core.27 |
//! | [`repair`] | a channel's name moved in the store to the version the ledger's head promotion is about, and a race lost on it reported and repaired from the ledger | 39, 101, 106, 109; core.6 |
//! | [`verify`](mod@verify) | which signer the policy names made each attestation on a version, and whether each counts, and the notes its index names | 3, 9, 65, 80, 81, 116, 117, 134; core.4, core.7 |
//! | [`cache`](mod@cache) | a cache entry under its build identity in the store, written by packing with no version and named from nothing, never an artifact; a checkout's cache filled from a landed revision's passing verdicts, an entry gone answered and refusing nothing | 52 to 54, 100, C.3; core.6, core.11, core.30 |
//! | [`push`](mod@push) | the step after a landing: the cache entry of each build identity a landed revision's passing verdicts name, written from what its build left on the host, offering nothing and keeping nothing | 52 to 54, 61, 77, 78, 100, C.3; core.6 |
//! | [`render`](mod@render) | a consumer layout: each version a checkout's lockfile pins, found by its ecosystem, name and version through the version's index, a name the store does not hold absent, judged under the consumer's layout line, else its publisher's through a signer the host trusts, else the given, rendered at `.tailrace/layouts/<ecosystem>/` holding only what the policy admits, every read before any write, a layout that cannot be refreshed answered from the host's record of it under the checkout's own git directory; a checkout synced, its cache filled and then each layout its lockfiles pin | 4, 9, 47 to 51, 58, 61 to 63, 81, 99, 113, 116, 117, 118, 138, C.10; core.4, core.7, core.17 to core.20, core.22 |
//!
//! Every read and the claim take a [`Landed`], which only the landing
//! seam's yes makes, and answer an `Answer`: absent, refused and
//! unreadable kept apart, with the source's own words (58, 59, rule
//! core.19). Deriving and claiming ask nothing of the build seam, so no
//! build learns its version (24). Publish never accepts, promotes, moves
//! a channel or reaches a target (20, rule core.10); status is handed no
//! binding, and status and verify write nothing (60, rule core.32). Every
//! surface renders an answer through [`answer_json`]'s one form (57,
//! rule core.31).

pub mod cache;
pub mod copy;
pub mod declared;
pub mod history;
pub mod landed;
pub mod notes;
pub mod provenance;
pub mod publish;
pub mod published;
pub mod push;
pub mod record;
pub mod render;
pub mod repair;
pub mod status;
pub mod verify;
pub mod version;

pub use cache::*;
pub use copy::*;
pub use declared::*;
pub use history::*;
pub use landed::*;
pub use provenance::*;
pub use publish::*;
pub use published::*;
pub use push::*;
pub use record::*;
pub use render::*;
pub use repair::*;
pub use status::*;
pub use verify::*;
pub use version::*;

// The record and the cache each answer a write with a `Written`; the
// root names the record's, and the cache's is `cache::Written`. The
// record of published versions and a layout left as it was are each a
// `Kept`; the root names the record's, and the layout's is
// `render::Kept`.
pub use published::{Kept, Written};

// The root's `render` is a consumer layout's; a version's notes are
// rendered by `notes::render`.
pub use notes::NOTES;
