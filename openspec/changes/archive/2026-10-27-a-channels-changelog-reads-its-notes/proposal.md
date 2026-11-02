# A channel's changelog reads its notes

## Why

`tailrace status` and the host's page already show each channel's chain,
every promotion with who asked for it, but not what any of those
versions changed: the notes a version carries are shown only beside the
versions this host published. The statement makes a channel's changelog
its chain read with each promotion's notes and its asker, and has the
page show each version's notes where the version carries them (134,
132). Every version now carries notes, so a reader of a channel on any
machine can be shown what each move brought.

## What Changes

- **Each promotion of a chain shows its version's notes.** Status
  answers, beside each channel, the notes of every version its chain
  moved it to, rendered again from the repository's own history by the
  version rule's tool, the same bytes the version's `notes` artifact
  holds (134, 21). A version moved to twice, a move back, is rendered
  once and shown under both promotions.
- **The changelog reads on any machine, from git alone.** The notes are
  rendered from the version's tag in the history the landed head
  reaches, its revision landed, and the policy file at that revision, as
  a rebuild recovers them (63): status still reaches no store, signer or
  target and writes nothing (60, core.32). A host that never published
  the version answers the same notes as the one that did (36).
- **One version's notes that cannot be read leave the chain standing.**
  A version whose tag the local history does not hold, or whose commits
  it no longer holds, answers its notes unreadable naming what is
  missing; the channel's version, its chain and every other version's
  notes are shown as they are (58, core.19).
- **The command line prints the changelog.** `tailrace status` prints
  each promotion's line, with who asked, and the notes of the version it
  moved to indented beneath it; `--json` carries the notes beside the
  channel in the one answer form (56, 57).
- **The page shows the changelog beside the channel, and every
  version's notes.** The repository's page renders each promotion with
  its asker and its version's notes beneath it, and the notes of each
  version this host published beneath that version's row, which the
  status answer already carries; the page computes nothing (132,
  core.31). The conformance register's pending part of 132, "each
  version's notes", leaves it with the scenario that cites it.

Out of scope, and what each waits on:

- The notes of every version between what an environment runs and what
  it should (134, 98): what an environment runs is read from a target,
  and every target waits on phase 2 (the register's "phase 2 · the first
  target").
- The herdr pane showing the changelog (132): the pane is
  `the-herdr-plugin`'s, which renders the same status answer.
- An asker the owner's identity provider names, and a record that says
  it names a host and no person (129): the changelog shows the asker the
  promotion claim holds, a signer; the identity provider waits on phase
  2, as the register says.
- A conformance scenario citing 134: the suite's statement list stops
  at 132, and the scenario this change adds cites 132; adopting a later
  statement into the list is its own unit's.
- Telling whether a version's index names notes at all, on a host whose
  record does not hold the version: that is the store's to answer, which
  status never asks (60); see the design's risks.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `channel-status`: each channel's chain is shown with the notes of
  every version it moved the channel to, read from git on any machine,
  each version's notes its own answer.
- `serving-process`: a repository's page renders each promotion's notes
  beneath it and each published version's notes beneath its row.

## Impact

- `crates/tailrace-core`: `status.rs` answers each channel's notes;
  `notes.rs` gains the derivation of a tagged version's commits that
  `record.rs`'s rebuild uses today, now shared by both.
- `crates/tailrace`: the status command's text and the page render the
  notes; tests in `tests/cli.rs`, `tests/status_across_machines.rs` and
  `tests/serve.rs`.
- `crates/tailrace-conformance`: a surfaces scenario citing 132, and the
  register's pending part for 132's notes removed.
- `site/`: the surfaces guide says status and the page show each
  promotion's notes.
- No new crate, language, file type or dependency.

## Touches

```
crates/tailrace-core/src/status.rs
crates/tailrace-core/src/notes.rs
crates/tailrace-core/src/record.rs
crates/tailrace-core/src/lib.rs
crates/tailrace-core/tests/status.rs
crates/tailrace-core/tests/record.rs
crates/tailrace-core/tests/notes.rs
crates/tailrace-core/tests/world/**
crates/tailrace-core/tests/common/**
crates/tailrace/src/commands/status.rs
crates/tailrace/src/commands/serve/page.rs
crates/tailrace/tests/cli.rs
crates/tailrace/tests/serve.rs
crates/tailrace/tests/status_across_machines.rs
crates/tailrace-conformance/src/core/surfaces.rs
crates/tailrace-conformance/src/register.rs
crates/tailrace-conformance/src/world.rs
crates/tailrace/tests/conformance.rs
site/src/content/docs/guides/integrations/surfaces.mdx
```
