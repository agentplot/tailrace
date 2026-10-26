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
//! | [`history`] | the files, commits and tags of a landed revision, from the local history and the shared one; a remote's address as written; a branch's head and the newest revision of it a checkout descends from, a revision resolved to its commit, the working tree's root, and the common directory every worktree shares | 21, 22, 62, 65, 114, 123; core.4 |
//! | [`declared`] | the components, the version rule and the projection each output is packed by, read from the policy file at the landed revision | 65, 123, C.10; core.22 |
//! | [`version`] | a version per component from the history alone, the components a landing touched, and the claim of the version's name | 21 to 24, 124; core.6, core.13, core.37 |
//! | [`provenance`] | the claim each artifact carries: what the host observed of its build, and the landing's verdicts carried as made | 11 to 13; core.8, core.9 |
//! | [`publish`](mod@publish) | a landed revision offered as a version per component: judged before the name is claimed, each artifact put and attested, the index last, a repeat ending where the first did, and the answer a program reads | 2, 5, 6, 8, 14 to 20, 57, 74, 75, 78, 84, 111, 124; core.5, core.6, core.10 to core.13, core.27 |
//! | [`published`] | the host's record of the versions it published, a file per version a reader opens | 61, 62, 64; core.20, core.21 |
//! | [`served`] | the one address the host's serving process answers at, the core's and moved by no setting, and a repository's owner and name in its path from its landing's remote | 49, 114; core.36 |
//! | [`status`](mod@status) | every channel the policy declares at the branch's landed head, at its version or empty with the chain that put it there, read from the ledger alone, beside which versions this host published, answered from the record alone, and the environments, unreadable while no target is read; each part and each channel its own answer; and a repository's row of every channel's head and its environments' counts | 23, 31, 36, 56 to 61, 98, 101, 125, 131, 132; core.19, core.31, core.32 |
//! | [`record`] | the host's record rebuilt from the store and git: an entry per tag the branch landed whose name the store resolves to its component's index at that version, every read before any write, nothing rewritten or deleted | 22, 58 to 63; core.4, core.19, core.20 |
//! | [`copy`](mod@copy) | a version copied from one store to another through the store seam, every attestation going with its digest, the index last and the name made from nothing, and no channel | 1 to 3, 6, 8, 103; core.4, core.6, core.12, core.27 |
//! | [`repair`] | a channel's name moved in the store to the version the ledger's head promotion is about, and a race lost on it reported and repaired from the ledger | 39, 101, 106, 109; core.6 |
//! | [`verify`](mod@verify) | which signer the policy names made each attestation on a version, and whether each counts | 3, 9, 65, 80, 81, 116, 117; core.4, core.7 |
//! | [`cache`](mod@cache) | a cache entry under its build identity in the store, written by packing with no version and named from nothing, never an artifact; a checkout's cache filled from a landed revision's passing verdicts, an entry gone answered and refusing nothing | 52 to 54, 100, C.3; core.6, core.11, core.30 |
//! | [`push`](mod@push) | the step after a landing: the cache entry of each build identity a landed revision's passing verdicts name, written from what its build left on the host, offering nothing and keeping nothing | 52 to 54, 61, 77, 78, 100, C.3; core.6 |
//! | [`gate`](mod@gate) | an offered version judged under the policy for acceptance at the governed branch's landed head: accepted whole with one claim of the gate's own on its index, no byte changed, or refused whole naming the artifact and the claim found wanting; the gate never accepting what it offered but on the one host; and each channel that follows each publish promoted by the gate, the ledger appended before the name moves | 7, 9, 13, 25 to 30, 33, 35, 39, 94 to 96, 105, 107, 108; core.4, core.6, core.7, core.14, core.15, core.34 |
//! | [`ledger`] | a channel moved by a promotion the host's signer makes, judged under the channel's policy and its binding, held by the ledger before the store is written and undone by a promotion back; the store rewritten from the ledger; and a channel read from the ledger alone | 9, 31 to 39, 58, 101, 105, 106, 109; core.4, core.6, core.7, core.15, core.16, core.34 |
//!
//! Every read and the claim take a [`Landed`], which only the landing
//! seam's yes makes, and answer an `Answer`: absent, refused and
//! unreadable kept apart, with the source's own words (58, 59, rule
//! core.19). Deriving and claiming ask nothing of the build seam, so no
//! build learns its version (24). Publish never accepts, promotes, moves
//! a channel or reaches a target (20, rule core.10); status asks the
//! landing, the policy and the ledger alone, and status and verify write
//! nothing (60, rule core.32). Every
//! surface renders an answer through [`answer_json`]'s one form (57,
//! rule core.31).

pub mod cache;
pub mod copy;
pub mod declared;
pub mod gate;
pub mod history;
pub mod landed;
pub mod ledger;
pub mod provenance;
pub mod publish;
pub mod published;
pub mod push;
pub mod record;
pub mod repair;
pub mod served;
pub mod status;
pub mod verify;
pub mod version;

pub use cache::*;
pub use copy::*;
pub use declared::*;
pub use gate::*;
pub use history::*;
pub use landed::*;
pub use ledger::*;
pub use provenance::*;
pub use publish::*;
pub use published::*;
pub use push::*;
pub use record::*;
pub use repair::*;
pub use served::*;
pub use status::*;
pub use verify::*;
pub use version::*;

// The record and the cache each answer a write with a `Written`; the
// root names the record's, and the cache's is `cache::Written`.
pub use published::Written;
