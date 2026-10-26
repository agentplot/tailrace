# Design

## Context

The ledger seam is built as a trait and a fake and nothing more.
`tailrace_seams::Keeper` (`keeper.rs`) has `append(channel, promotion,
predecessor) -> Answer<Digest>` and `chain(channel) ->
Answer<Vec<Envelope>>`; `Why::NotHead { head }` and `Why::Fork {
predecessor, claims }` are its refusals; `Envelope::digest()` is the
digest a promotion names its predecessor by. `FakeKeeper` keeps a chain
per channel and can be scripted into a fork. `tailrace_core::repair`
already moves a channel's name to the head's index and repairs a lost
race (`follow(store, keeper, channel, name, expected) ->
Answer<Followed>`), reading the head's subject from a claim of kind
`Promotion`. The conformance suite has the keeper's group (`seams/
keeper.rs`, a `KeeperRig`) and the core's ledger group (`core/
ledger.rs`, one scenario of 109), each with its marker. The host's
bindings file refuses `[ledger]` and binds `NoLedger`
(`bindings/mod.rs`). The policy file already reads `[channels.<name>]`
with `require` and `bound_to` into `Required::Channel` (ADR 0013), and a
signer's `kinds` may name `acceptance` and `promotion`. `verify.rs`
tells each attestation on an index by the signer that made it and
whether it counts, in a private `tell`.

What git does, run with git 2.54 for this change: a push of a commit to
`refs/tailrace/ledger/<channel>` that is not a fast-forward is rejected
`(non-fast-forward)` with exit 1, as for a branch, unless forced; a
fetch into a local ref of that name that is not a fast-forward is
rejected the same way and leaves the local ref; `git -c
core.logAllRefUpdates=always fetch` writes a reflog entry for a ref
outside `refs/heads`, which `git reflog show --date=unix` reads back
with its time; `git update-ref <ref> <new> <old>` refuses when the ref
is not at `<old>`.

The design rulings this change takes (`agentplot/blueprints` 673d334,
carried to this unit by the plan's proposal 84): the ledger's binding
is this unit's to add, in the `[ledger]` table ADR 0013 holds for it;
a git ref ledger is read from the refs as fetched, and as last fetched
where the remote does not answer, saying when (C.13); a store ledger is
read from the store, and from the chain the host last read where the
store does not answer, which 61 now lets the host keep (C.14).
`status-on-any-machine` comes after this unit and reads the promotion
body and the chain's read time this unit fixes; the policy seam's
`channels` (131) is `the-policy-seam-lists-the-channels`, a trait
change landed before the status unit.

## Goals / Non-Goals

**Goals:**

- A person promotes a version to a channel with one command, and
  undoes a wrong move by promoting back.
- Every promotion judged under the channel's policy and its binding,
  signed by the gate's signer, held by the ledger before anything is
  written to the store.
- The git ref and store ledgers, each passing the keeper's group.
- A chain read on another machine from the ledger alone, and answered
  as last read where its source does not answer.

**Non-Goals:**

- Acceptance, the acceptance claim's body, and `beta` following each
  publish: the gate's unit.
- Status showing channels: `status-on-any-machine`.
- The policy seam's list of channels with their components (131), and
  so a channel's one component (125):
  `the-policy-seam-lists-the-channels`.
- Environments, approvals as environments' channels, targets: phase 2.
- Several promotions in one act (128), pattern channels (127), the
  asker's identity from an identity provider (129).

## Decisions

### D1. The promotion body is a seam type, re-exported by the core

`keeper.rs` gains `Promotion`, the body of every promotion claim (33,
39, 105):

```rust
pub struct Promotion {
    pub channel: ChannelName,
    pub component: ComponentName,
    pub to: Moved,               // { version, index }
    pub from: Option<Moved>,     // None: the channel's first promotion
    pub predecessor: Option<Digest>, // the envelope digest it follows
    pub asked_by: SignerId,      // D7
    pub policy: Satisfied,       // { revision, claims: Vec<Digest> }
    pub at: u64,                 // seconds since the Unix epoch, UTC
}
```

with `Promotion::of(&Claim) -> Option<Promotion>` (none for any other
kind, or a body that does not read as one) and `Promotion::claim(self,
revision) -> Claim` (subject `to.index`, kind `Promotion`, the tag's
revision, no build: core.9 does not rule it out). It sits with the
seam because a ledger profile walks by it (C.14), and `Claim`'s own
doc leaves a kind's body to the unit that makes the kind;
`tailrace_core::ledger` re-exports it, which is where the status unit
reads it. The time is a count of seconds so that no date crate is
added; a surface renders it. Rejected: the body in the core, which the
store ledger profile could then not read without reaching into the
core (core.2).

### D2. The channel's store name is `channel/<channel>`

`keeper.rs` gains `channel_name(&ChannelName) -> Name`, `channel/`
followed by the channel, as `cache/` names a cache entry. It cannot be
a version's name, which is a component's prefix and a version, nor a
cache entry's. The core's promotion moves it, and the store ledger
walks from it. A channel belongs to one component (125), so one name
per channel is enough; the component is in every claim.

### D3. The chain's answer says how it was read

`Keeper::chain` answers `Chain { claims: Vec<Envelope>, read: Read }`,
`Read::Source` or `Read::Kept { at: u64, said: String }`: the claims as
last read, when, and the source's words of why it did not answer (60,
61, C.13, C.14). `append` never answers from a kept copy. This changes
the trait in the unit that owns its seam, as the ruling asks; the
users are `repair::follow`, which refuses a kept read unreadable with
its words (a name moved from a stale head could move it backwards),
the fake, the conformance groups and `bindings`' unbound ledger. The
fake keeper keeps the chain it last answered per channel and answers
it once made unreachable (`FakeKeeper::unreachable(words)`), and the
keeper's group adds "a chain read as last read" below its marker.

### D4. The promotion, in `tailrace-core/src/ledger.rs`

`promote(seams, history, ask) -> Answer<Promoted>`, `ask` being
`PromoteAsk { branch, component: Option<_>, to: To, from:
Option<Version>, signer: SignerId, at: u64 }` with `To::Version(v)`
or `To::Back`. In order, every step's refusal the answer and
nothing after it run:

1. The branch's head, landed (`Landed::check`), the policy file there
   (`Declared::read`) and the component, as verify chooses it.
2. The channel's policy at that head (`policy_at`, Audience::Channel),
   absent where the repository declares none for it (82).
3. The channel's chain through the ledger; a kept read is unreadable
   with its words; a fork is the ledger's refusal (105). The head is
   the last claim, read with `Promotion::of`; a head that is not one is
   unreadable naming it.
4. The version: for `Back`, the head's `from`, absent naming it when
   the head has none or the chain is empty; else the version given. Its
   tag (`<prefix><version>`) resolved in the store to its index (core.4)
   and the tag's revision from the history, landed.
5. `--from`: given and not the head's version, refused
   `Why::ChannelAt { channel, at: Option<Moved> }` (35, S7).
6. Where the head is already at the index, `Promoted::There` after D6,
   and nothing is appended.
7. Bound (`required.bound_to()`): the bound channel's chain, kept reads
   allowed since a chain only grows; the index must be the subject of
   one of its promotions, else refused `Why::NeverHeld { channel,
   bound_to }` (37, 83).
8. Judged (D5).
9. The claim: `Promotion { from: the head's to, predecessor: the
   head's envelope digest, .. }` signed through the signer seam as
   publish signs.
10. `ledger.append(channel, envelope, predecessor)`: a refusal is the
    answer, never retried (core.6, core.34).
11. D6, its answer beside the promotion: `Promoted::Moved { channel,
    to, from, claim, predecessor, policy, store: Answer<Mirrored> }`.

`channel(seams.keeper, channel) -> Answer<Channel>` is the read of D8.

### D5. What a promotion is judged by

`verify.rs`'s `tell` becomes `pub(crate)` and is used unchanged.
`judge(seams, required, index) -> Answer<Vec<Digest>>` tells each
attestation on the index under the signers the channel's policy names,
merged as verify merges them, then requires, in order: an acceptance
claim that counts (31), whether or not the channel's `require` lists
one, then each `RequiredClaim` of the policy, its check where it is a
verdict. The first with no counting attestation is refused
`Why::Wanting { kind, check, signers }`, naming the signers that may
make it (58). It answers the digests of the envelopes that satisfied
each. The host's signer must be among the merged signers with kind
`promotion`, else refused `Wanting { kind: Promotion, .. }` before
anything is signed (9, 32). Only the index's claims are judged: a claim
about a version is a claim about its index (7). An acceptance claim
therefore counts only where the channel's policy names the gate,
`[signers.gate] kinds = ["acceptance", "promotion"]` and `[channels.
stable] require = [{ kind = "acceptance", signers = ["gate"] }]`, which
is how the policy file's example already reads.

Rejected: requiring acceptance only where the channel's policy lists
it, which lets a channel name a version no gate accepted (31).

### D6. The store rewritten from the ledger

`mirror(store, ledger, channel) -> Answer<Mirrored>` reads the chain
(a kept read writes nothing), and for each claim whose envelope digest
is not among `claims_of(subject)`, attaches it (39, 106); then
`follow`s the channel's name from where it resolves to the head's
index (109). It answers the claims attached and the `Followed`. It runs
after every append and when the channel is already there, so a store
that missed a write is caught up by the next promotion. Rejected: a
command of its own, which no clause or ruling asks for yet.

### D7. Who asked

33 asks the ledger to record who asked, and 129 asks for an identity
the owner's identity provider issued, which phase 1 binds none of. The
promotion records `asked_by` as the host's own signer, the gate that
signs it: on the solo host 108 admits, the person at the command line
asks as the host. The gap goes to the design agent; when 129 is built,
`asked_by` becomes the verified identity and the field's type changes
with it.

### D8. Reading a channel from the ledger alone

`channel(ledger, channel) -> Answer<Channel>`: `Channel::Empty`, or
`Channel::At { head: Promotion, chain: Vec<(Digest, Promotion)>, read:
Read }`, every claim read with `Promotion::of`, a claim not of that
form unreadable naming its digest. It asks only the ledger (60, 101,
core.32). The proof across machines is a test of two clones of one
bare remote, D9: the first promotes, the second, bound to an empty
layout, reads the same head and chain, and its store is asked nothing.

### D9. The git ref ledger (`profiles/ledger/git_ref/`)

`GitRef::new(repository, remote: Option<String>)`, git run through
`profiles::run::Tool` as the plain branch runs it.

- A commit per claim: a tree of one blob, `promotion.json`, the
  envelope's canonical JSON; one parent, the commit of the claim
  before; author and committer `tailrace <tailrace@localhost>` at the
  promotion's time where the claim reads as one; the message
  `promotion <envelope digest>`. Made with `hash-object -w`, `mktree`
  and `commit-tree`, so no index or working tree is touched.
- The read, with a remote: `git -c core.logAllRefUpdates=always fetch
  --no-write-fetch-head <remote> +<ref>:refs/tailrace/fetched/<channel>`
  into a scratch ref, then: the local ref absent or an ancestor of the
  fetched commit, the local ref fast-forwarded to it (`update-ref` with
  its old value, logged); the fetched an ancestor of the local, the
  remote lost claims, unreadable naming them (122); neither, a fork,
  refused `Why::Fork` naming the claim at the merge base, or none, and
  the first claim of each side after it (105), the local ref kept. A
  remote ref that is absent while the local holds one is the same
  loss. A fetch that fails for any other reason answers the local ref
  as kept, `at` from its newest reflog entry and git's words; no local
  ref, unreadable with git's words. Without a remote, the local ref is
  read as it is, `Read::Source`.
- The walk: `rev-list --reverse --parents <ref>`; a commit with two
  parents, or whose tree holds no `promotion.json` that parses as an
  envelope, unreadable naming the commit.
- Append: the read, then the given predecessor against the head's
  envelope digest, `NotHead { head }` where they differ; the commit on
  the head's; with a remote, `push <remote> <commit>:<ref>` (never
  `+`), the local ref moved to it on success; a refused push read again
  and answered `NotHead` naming the remote's head where it moved, else
  refused `Why::Said` with git's words (59). Without a remote,
  `update-ref <ref> <commit> <head commit or zero>`, its refusal
  `NotHead` after a read.

The gate's own identity pushes because the push uses the host's git
credentials, reached at the moment of use (64, core.21).

### D10. The store ledger (`profiles/ledger/store/`)

`InStore::new(store: Box<dyn Store>, kept: PathBuf)`, the store a
binding of `[store]` that never makes it, `kept` the repository's
common git directory joined with `tailrace/ledger`.

- The walk: resolve `channel_name(channel)`; absent, absent. The
  promotion claims of the channel attached to that index are the
  candidates. Each is walked back: its `from.index`, the claim there
  whose envelope digest is its `predecessor`, until a claim with no
  predecessor. A candidate whose walk holds every other candidate is
  the head; two candidates neither of which holds the other, or two
  claims met on one index naming one predecessor, `Fork`. A
  predecessor not found on its index, unreadable naming both digests.
- Kept: each chain read from the store is written, with the time, to
  `<kept>/<channel>.json` beside the record of published versions (61,
  62), a write to a temporary file renamed over the last, as
  `published.rs` writes. Where the store is unreadable, the kept chain
  is answered `Read::Kept` with the store's words; nothing kept,
  unreadable.
- Append: the walk; the given predecessor against the head's digest
  and against the claim's own `predecessor`, `NotHead` where either
  differs; `attach(to.index, claim)`; `name(channel_name, to.index,
  Some(head's index))`, its `NameAt` refusal answered `NotHead` naming
  the head walked again. The claim attached before the refused move
  stays, and a later walk that meets it reports the fork it is.

### D11. `[ledger]` and the binding

`profiles/ledger/mod.rs` has `bind(table, repository, store_table)`:
`git_ref` with `deny_unknown_fields` options `{ remote: Option<String>
}`, `store` with none, any other key `table.unknown(&["git_ref",
"store"])`. `bindings::read` takes `"ledger"` as it takes `"store"` and
`"landing"`, `Bindings` gains `ledger: Option<Table>`, and `bind` binds
it where the file names it and an unbound ledger answering absent
naming `[ledger]` in the file where it does not. `For::Promote` needs
`[signing]` as publish does (absent naming it before anything is
read), and `[ledger]` (absent naming it), and never makes the store.
`NoLedger` keeps its shape for the other commands and answers absent
naming `[ledger]` in the file.

### D12. `tailrace promote`

`commands/promote.rs`, clap's `Args`: `version: Option<String>`, `--to
<channel>` (required), `--back` (conflicts with `version`; one of the
two is required), `--from <version>`, `--component`, `--branch`
(`main`), `--json`. The asker's time is the process's clock, passed in
the ask. Text: `stable moved from 0.1.0 to 0.2.0: claim <digest>
after <digest>, the store followed` or with the store's answer beside;
`stable is at 0.2.0 already`. JSON is `render::json` of the answer.
Exit codes as every command's.

### D13. The conformance groups

Below the ledger group's marker, over the `FakeWorld` with two
published versions, the gate's acceptance attached by a test signer
`gate` the policy names with `acceptance` and `promotion`, and a
`rebuild` signer: "S6 a version promoted to stable" (33, 39, 101;
plays S6), "S7 a promotion from a version the channel has left" (35,
105; plays S7), "S8 stable moved back" (34, 38; plays S8), "a bound
channel takes what its bound channel held" (37, 83), "a promotion found
wanting" (9, 31, 32, 58), "a forked chain refuses a promotion" (105),
"the store rewritten from the ledger" (106, 109), "a channel read from
the ledger alone" (60, 101). Below the keeper group's marker: "a chain
read as last read" (60, 61, 121). The register loses 32, 33, 37, 38,
39, 101 and 106. 41 and 97 move from this unit's block to the first
target's, with S6's environment half. 125 moves from this unit's block
to one for `the-policy-seam-lists-the-channels`, since 131 gives a
channel its component; where that unit has merged into the bolt first
with a block of its own, 125 joins it.

### D14. ADR 0016

"The ledger's git ref holds one claim per commit, and a channel is
named `channel/<channel>` in the store": D2, D3, D5, D7, D9 to D11,
with ADR 0009 and 0013 as its context and the rulings of 673d334. It
fills the `[ledger]` table 0013 holds for the ledger and lifts 0013's
refusal of it, saying so; 0013 itself is not edited. Proposed, for the
user to accept with the change.

## Risks / Trade-offs

- **The gate's unit builds beside this one.** Both add to `Why` and
  both judge claims against a policy: this unit's `Wanting` and
  `judge` are what the gate's acceptance should use, and the gate's
  move of `beta` should call `promote`. Whichever merges second
  rebases onto the other's; the conductor knows.
- **A store ledger's refused move leaves a claim attached.** The claim
  is held beside a chain it did not join, and a walk that meets it
  reports a fork. That is 105's answer, and C.14 is for an operator
  who trusts one gate to write.
- **The reflog's time is the last write of the local ref**, a fetch or
  an append, which is when the host last read the chain from the
  remote or wrote it there.
- **A chain read on a host writes its kept copy** (store ledger) or its
  local ref (git ref). The ruling counts a fetch into the repository's
  own copy as a read; the store ledger's kept chain is the same act,
  as 61 now allows.
