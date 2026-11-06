//! The core: what timber does with its seams, Part A of the statement in
//! agentplot/blueprints `design/timber/requirements.md`. Code here names
//! no registry, task runner, merge queue, key service, package manager,
//! deploy target or forge (rule core.1); it reaches each through the one
//! trait of `timber_seams`, and the repository's history through git,
//! which section 9 fixes.
//!
//! | module | what it keeps | statement, rules |
//! |---|---|---|
//! | [`landed`] | a revision is read only once the landing seam says its branch landed it, and the policy for an audience only at such a revision | 14, 65, 77, 82, 83; core.11, core.22 |
//! | [`history`] | the files, commits and tags of a landed revision, from the local history and the shared one; a commit's message, and the commit as the rule's tool reads it, from the local history alone; a remote's address as written; a branch's head and the newest revision of it a checkout descends from, a revision resolved to its commit, the working tree's root, the common directory every worktree shares, and each worktree's own git directory | 21, 22, 50, 62, 65, 114, 123, 134; core.4 |
//! | [`declared`] | the components, the version rule and the projection each output is packed by, read from the policy file at the landed revision | 65, 123, C.10; core.22 |
//! | [`version`] | a version per component from the history alone, the commits it is derived from, the components a landing touched, and the claim of the version's name | 21 to 24, 124, 134; core.6, core.13, core.37 |
//! | [`notes`] | a version's notes, the commits it is derived from rendered by the rule's own tool under the template the version's revision holds, the tool's own default where it holds none, and the ecosystem reserved for them; the commits a tagged version's notes were rendered from, derived again at its landed revision | 18, 21, 63, 134, section 9 |
//! | [`provenance`] | the claim each artifact carries: what the host observed of its build, and the landing's verdicts carried as made | 11 to 13; core.8, core.9 |
//! | [`publish`](mod@publish) | a landed revision offered as a version per component: where the landing carries verdicts its outputs replayed from the cache entries the store holds and one it lacks refused, judged before the name is claimed, each artifact put and attested, the index last and carrying the layout line the publisher's file writes, a repeat ending where the first did, and the answer a program reads | 2, 5, 6, 8, 14 to 20, 52, 57, 74, 75, 78, 84, 111, 113, 124, 135; core.5, core.6, core.8 to core.13, core.27 |
//! | [`published`] | the host's record of the versions it published, a file per version a reader opens, naming the commits each version's notes were rendered from and never their text | 61, 62, 64, 134; core.20, core.21 |
//! | [`served`] | the one address the host's serving process answers at, the core's and moved by no setting, a repository's owner and name in its path from its landing's remote, and a served layout's address under them | 49, 114; core.18, core.36 |
//! | [`status`](mod@status) | every channel the policy declares at the branch's landed head, at its version or empty with the chain that put it there, read from the ledger alone, and the notes of every version the chain moved it to, rendered again from the repository's own history as a rebuild recovers them; beside which versions this host published, answered from the record alone, each with its notes rendered again from the repository's own history, and the environments, unreadable while no target is read; each part, each channel and each version's notes its own answer; and a repository's row of every channel's head and its environments' counts | 23, 31, 33, 36, 56 to 61, 63, 98, 101, 125, 131, 132, 134; core.19, core.31, core.32 |
//! | [`in_force`](mod@in_force) | the policy in force at the branch's landed head: what acceptance and a layout require, and every channel the policy's channels answer lists with how it moves and its policy, each part and each channel its own answer | 58, 60, 82, 131, C.10; core.31, core.32 |
//! | [`record`] | the host's record rebuilt from the store and git: an entry per tag the branch landed whose name the store resolves to its component's index at that version, its notes' commits derived again at the tag's revision, every read before any write, nothing rewritten or deleted | 22, 58 to 63, 134; core.4, core.19, core.20 |
//! | [`copy`](mod@copy) | a version copied from one store to another through the store seam, every attestation going with its digest, the index last and the name made from nothing, and no channel | 1 to 3, 6, 8, 103; core.4, core.6, core.12, core.27 |
//! | [`repair`] | a channel's name moved in the store to the version the ledger's head promotion is about, and a race lost on it reported and repaired from the ledger | 39, 101, 106, 109; core.6 |
//! | [`verify`](mod@verify) | which signer the policy names made each attestation on a version, and whether each counts, and the notes its index names | 3, 9, 65, 80, 81, 116, 117, 134; core.4, core.7 |
//! | [`cache`](mod@cache) | a cache entry under its build identity in the store, written by packing with no version and named from nothing, never an artifact; a checkout's cache filled from a landed revision's passing verdicts, an entry gone answered and refusing nothing | 52 to 54, 100, C.3; core.6, core.11, core.30 |
//! | [`push`](mod@push) | the step after a landing: the cache entry of each build identity a landed revision's passing verdicts name, written from what its build left on the host; and the push before a landing: the cache entry of each build identity a checkout's run left with a pass, asking no landing; each offering nothing and keeping nothing | 52 to 54, 61, 77, 78, 100, 135, 139, C.3; core.6 |
//! | [`render`](mod@render) | a consumer layout: each version a checkout's lockfile pins, found by its ecosystem, name and version through the version's index, a name the store does not hold absent, judged under the consumer's layout line, else its publisher's through a signer the host trusts, else the given, rendered at `.timber/layouts/<ecosystem>/` holding only what the policy admits, every read before any write, a layout that cannot be refreshed answered from the host's record of it under the checkout's own git directory; a checkout synced, its cache filled and then each layout its lockfiles pin | 4, 9, 47 to 51, 58, 61 to 63, 81, 99, 113, 116, 117, 118, 138, C.10; core.4, core.7, core.17 to core.20, core.22 |
//! | [`gate`](mod@gate) | an offered version judged under the policy for acceptance at the governed branch's landed head: accepted whole with one claim of the gate's own on its index, no byte changed, or refused whole naming the artifact and the claim found wanting; the gate never accepting what it offered but on the one host; and each channel of the component that the policy's channels answer says follows each publish promoted by the gate, where the channel's policy names it a promoter, the ledger appended before the name moves | 7, 9, 13, 25 to 30, 32, 33, 35, 39, 94 to 96, 105, 107, 108, 125, 131; core.4, core.6, core.7, core.14, core.15, core.34 |
//! | [`ledger`] | a channel the policy lists for its component moved by a promotion the host's signer makes, where the channel's policy names it a promoter, judged under the channel's policy and its binding, held by the ledger before the store is written and undone by a promotion back; the store rewritten from the ledger; and a channel read from the ledger alone | 9, 31 to 39, 58, 101, 105, 106, 109, 125, 131; core.4, core.6, core.7, core.15, core.16, core.34 |
//! | [`after`](mod@after) | an environment's newest report: the reports of it attached to each index its channel's chain moved to, read apart from counting them, and counted through the signers the policy lets claim a report, the one whose apply came last told first and none older than the first that counts; absent where none counts, saying so where the policy lets no signer report, and unreadable naming the chain, the index or the report that could not be read or told; and each channel an `after` makes that comes after an environment, answered on its own: held naming each environment's newest report where they name two indexes or one cannot be found, refused naming the host's signer and the promoters where the host is none of them, and otherwise moved to the one index by the move a promotion makes, as the gate's own act | 32, 33, 37, 40, 41, 43, 58, 83, 91, 95, 97, 98, 116, 129, C.10, C.12; core.7, core.34, core.42, core.50, core.52 |
//! | [`apply`](mod@apply) | one environment the policy declares at the branch's landed head converged on its channel's head, read from the ledger's chain: the target bound for its profile handed that head as desired, nothing applied where it runs it already, and the claims the channel's policy requires judged of the head's index before the target is handed the version, verified; its report after, and what it ran before, each its own answer; once the report names the index applied, a report claim the host signs attached beside it, made again where the environment's newest report names another index or none and never twice; no name moved and nothing else written | 11, 31, 40, 42, 43, 88 to 92, 119, 136; core.4, core.8, core.9, core.16, core.19, core.27, core.29, core.50 |
//!
//! Every read and the claim take a [`Landed`], which only the landing
//! seam's yes makes, and answer an `Answer`: absent, refused and
//! unreadable kept apart, with the source's own words (58, 59, rule
//! core.19). Deriving and claiming ask nothing of the build seam, so no
//! build learns its version (24). Publish never accepts, promotes, moves
//! a channel or reaches a target (20, rule core.10); status asks the
//! landing, the policy, the ledger and the history alone, the policy in
//! force the landing and the policy alone, and status, the policy in
//! force and verify write nothing (60, rule core.32); apply hands a
//! target a version only once its claims are judged, moves no name, and
//! writes only the report claim it signs beside the index applied (42,
//! 43, rules core.16, core.27, core.29). Every surface renders an
//! answer through [`answer_json`]'s one form (57, rule core.31).

pub mod after;
pub mod apply;
pub mod cache;
pub mod copy;
pub mod declared;
pub mod gate;
pub mod history;
pub mod in_force;
pub mod landed;
pub mod ledger;
pub mod notes;
pub mod provenance;
pub mod publish;
pub mod published;
pub mod push;
pub mod record;
pub mod render;
pub mod repair;
pub mod served;
pub mod status;
pub mod verify;
pub mod version;

pub use after::*;
pub use apply::*;
pub use cache::*;
pub use copy::*;
pub use declared::*;
pub use gate::*;
pub use history::*;
pub use in_force::*;
pub use landed::*;
pub use ledger::*;
pub use provenance::*;
pub use publish::*;
pub use published::*;
pub use push::*;
pub use record::*;
pub use render::*;
pub use repair::*;
pub use served::*;
pub use status::*;
pub use verify::*;
pub use version::*;

// The record and the cache each answer a write with a `Written`; the
// root names the record's, and the cache's is `cache::Written`. The
// record of published versions and a layout left as it was are each a
// `Kept`; the root names the record's, and the layout's is
// `render::Kept`.
pub use published::{Kept, Written};

// A layout and a promotion each judge a version under a policy; the root
// names the layout's, and a promotion's is `ledger::judge`.
pub use render::judge;

// A channel's name in the store follows its chain, and a channel an
// `after` makes follows its environments' reports; the root names the
// store's, and the gate's move on reports is `after::follow`.
pub use repair::follow;

// The root's `render` is a consumer layout's; a version's notes are
// rendered by `notes::render`.
pub use notes::NOTES;
