# Proposal

## Why

Bolt 4 is usable when a family repository's `stable` moves when a person
promotes, the chain is in `refs/tailrace/ledger/<channel>` and in the
store, and any machine reads where each channel is and why. Today the
ledger seam exists with only its fake: the host's bindings file refuses
a `[ledger]` table, nothing signs a promotion, and no command moves a
channel. This unit builds the ledger's two profiles and the promotion
itself, so a person promotes a version with one command, undoes a wrong
move by promoting back, and a second machine reads the move from the
ledger alone (B.8, C.13, C.14, 34 to 37, 39, 101, 105, 106, 120 to 122;
ADR 0009; `agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`,
Bolt 4).

## What Changes

- **One command promotes.** `tailrace promote <version> --to <channel>`
  moves a channel to a version of its component. The gate's path judges
  it first: the version's index must carry an acceptance claim that
  counts under the channel's policy (31), then every claim the channel's
  policy requires, in order, and the first found wanting is named (58);
  the host's signer must be one the channel's policy lets claim a
  promotion (9, 32). Then it signs the promotion claim, appends it to
  the channel's chain, and only after the ledger holds it attaches the
  claim to the version's index and moves the channel's name in the
  store (33, 39).
- **A promotion names where it starts.** Each promotion claim names the
  version the channel moves from and the digest of the promotion it
  follows (105). `--from <version>` that is not where the channel is now
  is refused naming the version it is at (35, S7). An append the ledger
  refuses because another promotion got there first is the answer,
  never retried (core.6, core.34). A fork, two claims naming one
  predecessor, is reported naming both, and nothing is written.
- **A wrong move is undone by a promotion back.** `tailrace promote
  --back --to <channel>` promotes the version the head moved from; it
  is a promotion like any other, judged and recorded, and the version
  left stays accepted and readable by digest (34, 38, S8).
- **A bound channel takes only what its bound channel held.** Where the
  channel's policy binds it to another, the version must be the subject
  of a promotion in the bound channel's chain, now or before, so a
  rollback to a version `beta` has since left is allowed (37, 83).
- **The store is rewritten from the ledger.** After every promotion the
  chain is walked and each claim missing from its index is attached and
  the channel's name moved to the head's version, so a store that
  disagrees with the ledger is brought back to it (101, 106, 109).
- **The git ref ledger (C.13).** One ref per channel,
  `refs/tailrace/ledger/<channel>`, each commit holding one promotion
  claim and parented on the one before; append is a push that only
  fast-forwards, and a refused push is a predecessor that is not the
  head. A fetch whose remote no longer extends what this host last read
  is a fork, reported naming both claims.
- **The store ledger (C.14).** The chain read from the promotion claims
  attached to the versions they move to, walked by predecessor from the
  channel's name in the store, for a store the operator trusts to
  retain them.
- **A chain read says when it was read.** Where the git ref's remote or
  the store does not answer, the chain is answered as this host last
  fetched or last read it, saying when and carrying the source's words
  (60, 61, C.13, C.14, as ruled by `agentplot/blueprints` 673d334). The
  ledger seam's `chain` answers that time beside the claims. Nothing
  moves a channel or a name from a chain read that way.
- **The host binds the ledger.** The bindings file gains `[ledger]`,
  naming `git_ref` with an optional remote or `store`; a host whose file
  names no `[ledger]` answers absent naming it.
- **The promotion's body is fixed here.** The claim names the channel,
  the component, the version and index it moves to and from, its
  predecessor, who asked, the policy revision and the claims that
  satisfied it, and when (33). It is defined beside the ledger seam,
  since the store ledger walks by it, and `status-on-any-machine` reads
  it.
- **Conformance.** The core's ledger group gains S6 as far as the ledger
  goes, S7, S8, a bound channel, a promotion found wanting, a fork, the
  store rewritten from the ledger, and a chain read on another machine
  from the ledger alone; the keeper's group gains a chain read as last
  read. Both ledger profiles run the keeper's group. 32, 33, 37, 38, 39,
  101 and 106 leave the register.
- **ADR 0016** records the ref's commit form, the channel's name in the
  store, `[ledger]`'s keys, and what promote judges and records; the
  ledgers, self-hosting and surfaces pages say what is built.

Out of scope, and what each waits on:

- The gate's acceptance, and `beta` following each publish as the gate's
  act: bolt 4's `the-gate-accepts-under-the-policy`. This unit reads an
  acceptance claim as any attached claim; it writes none. The gate's
  move of `beta` should go through this unit's promotion function.
- Keeping the offering and accepting credentials apart, and S9: the
  gate's unit. Promote signs with the host's `[signing]` signer, which a
  solo host's policy names as the gate (108).
- `tailrace status` showing channels and their chains on any machine,
  and the policy seam's list of a repository's channels with their
  components (131): bolt 4's `status-on-any-machine`, which comes after
  this unit and reads the promotion body and the chain's read time.
- A channel belonging to one component (125): with 131 the channels'
  components come from the policy, which the status unit adds; this
  unit names the component in every promotion and leaves 125 in the
  register.
- An environment converging on a channel's move, and an approval as an
  environment's own channel (41, 97, the environment half of S6): the
  first target, phase 2.
- The asker named by an identity the owner's identity provider issued
  (129): no identity provider is bound in phase 1; the promotion records
  the host that asked.
- Several promotions in one act (128) and channels by branch pattern
  (127): no unit of phase 1 carves them.
- The store ledger against a registry's retention: proven over the OCI
  layout; ECR's retention rules are the operator's to trust (C.14).

## Capabilities

### New Capabilities

- `promotion`: a channel moved to an accepted version by a promotion the
  gate signs, judged under the channel's policy and its binding,
  recorded in the ledger before the store, undone by a promotion back,
  the store rewritten from the ledger, and a channel read from the
  ledger alone.
- `git-ref-ledger`: the ledger seam held as one git ref per channel,
  appended by a fast-forward push, read from the refs as last fetched
  where the remote does not answer.
- `store-ledger`: the ledger seam held as promotion claims attached in
  the store, walked by predecessor from the channel's name, read from
  the chain the host last read where the store does not answer.

### Modified Capabilities

- `seam-contracts`: the keeper's chain answers when it was read, the
  promotion claim's body, and the refusals a promotion gives.
- `seam-fakes`: the fake keeper answers a chain as last read once it
  cannot be reached.
- `conformance-suite`: the keeper's group holds a chain read as last
  read; the ledger's group holds promotion, S6 to S8 and bound channels.
- `host-bindings`: the file binds the ledger.
- `command-line`: `tailrace promote` is a command of the one core.
- `name-repair`: a chain read as last read moves no name.

## Impact

- `crates/tailrace-seams/src/keeper.rs` (the chain's read time, the
  promotion body, the channel's store name), `answer.rs` (three
  refusals), `lib.rs`'s test of every seam.
- `crates/tailrace-fakes/src/keeper.rs`.
- `crates/tailrace-core/src/ledger.rs` (new), `repair.rs`, `verify.rs`
  (its telling of one attestation shared), `lib.rs`.
- `crates/tailrace/src/profiles/ledger/` (new: `mod.rs`, `git_ref/`,
  `store/`), `profiles/mod.rs`, `bindings/` and its tests,
  `commands/promote.rs` (new), `commands/mod.rs`, `lib.rs`, `main.rs`.
- `crates/tailrace-conformance/src/seams/keeper.rs` and
  `core/ledger.rs` below their markers, `register.rs`, `world.rs`,
  `tests/fakes.rs`.
- `crates/tailrace/tests/ledger_git_ref.rs`, `ledger_store.rs` and
  `promote.rs` (new), all in check-fast, needing git alone.
- ADR 0016 and five site pages.
- No crate, dependency, language or file type is added.

## Touches

```
crates/tailrace-seams/src/keeper.rs
crates/tailrace-seams/src/answer.rs
crates/tailrace-seams/src/lib.rs
crates/tailrace-fakes/src/keeper.rs
crates/tailrace-fakes/src/lib.rs
crates/tailrace-core/src/ledger.rs
crates/tailrace-core/src/repair.rs
crates/tailrace-core/src/verify.rs
crates/tailrace-core/src/lib.rs
crates/tailrace-core/tests/ledger.rs
crates/tailrace/src/profiles/mod.rs
crates/tailrace/src/profiles/ledger/**
crates/tailrace/src/bindings/mod.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/src/commands/promote.rs
crates/tailrace/src/commands/mod.rs
crates/tailrace/src/lib.rs
crates/tailrace/src/main.rs
crates/tailrace/tests/ledger_git_ref.rs
crates/tailrace/tests/ledger_store.rs
crates/tailrace/tests/promote.rs
crates/tailrace/tests/cli.rs
crates/tailrace-conformance/src/seams/keeper.rs
crates/tailrace-conformance/src/core/ledger.rs
crates/tailrace-conformance/src/register.rs
crates/tailrace-conformance/src/world.rs
crates/tailrace-conformance/tests/fakes.rs
docs/adr/0016-*.md
site/src/content/docs/guides/integrations/ledgers.mdx
site/src/content/docs/guides/integrations/surfaces.mdx
site/src/content/docs/guides/self-hosting.mdx
site/src/content/docs/how-it-works.mdx
site/src/content/docs/roadmap.mdx
```
