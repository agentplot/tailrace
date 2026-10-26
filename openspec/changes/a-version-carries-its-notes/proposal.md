## Why

A version today says what bytes it holds and who vouched for them, but
not what changed in it. A reader has to go back to git and guess which
commits a version took. The design makes a version carry its notes:
the run that derives the version renders them from the same commits,
with the version rule's own tool, and they travel in the store as an
artifact of the version, named in its index and claimed like the rest
(`agentplot/blueprints:design/tailrace/requirements.md` 134 at 80ab275,
unchanged since ab0dfea, with 6, 17, 18, 21 and 132).

## What Changes

- **Every new version gets notes.** When publish derives a component's
  version, it keeps the commits that version was derived from (those
  since the component's last release tag that touch its paths) and
  renders them with git-cliff, the rule's tool, into Markdown: one
  heading for the version, then features, fixes and the rest, one line
  per commit with its subject and short revision. Nothing in the notes
  depends on the clock or the host, so the same revision always renders
  the same bytes (21, 18).
- **The notes are an artifact of the version.** After the version's
  name is claimed, the notes are put in the store as an artifact of the
  reserved ecosystem `notes`, named by the component, at the version;
  they get one provenance claim signed as the publisher, naming the
  revision and the commits they were rendered from, and no build; and
  the index names them beside the outputs' artifacts. All of it is held
  before the index is put and the version's name moves (6, 17, 134).
- **Notes are never written again.** A second publish of a revision
  renders the notes again and compares them with what the index holds:
  the same, nothing is written; different, the publish is refused
  naming the notes and both digests, as a projection that packs
  differently is (18). A version published before this change has no
  notes, and a later publish of it adds none (134: nothing writes a
  version's notes after it is offered).
- **No output may take the notes' ecosystem.** A policy file that packs
  an output with the ecosystem `notes` is refused when it is read,
  naming the component and the output.
- **Publish, status and verify show the notes beside each version.**
  Publish's answer carries the notes' digest, their claim and their
  text. Verify reads the notes the index names from the store and shows
  them after the claims. Status, which reaches no store, renders them
  from git: the host's record now keeps, beside each version, which
  commits its notes were rendered from, and status renders those
  commits again with the same tool. A version with no notes shows none,
  and a commit git no longer holds makes that version's notes
  unreadable, naming the commit (58, 60, 61, 132).
- **The record is still rebuilt from the store and git.** A rebuild
  recovers each version's commits by deriving the version again at its
  tag's revision, so a deleted record comes back byte for byte (63).

Out of scope, and what each waits on:

- The notes of every version between what an environment runs and what
  it should, and a channel's changelog read from its chain with each
  promotion's notes and asker (134, 98): `a-channels-changelog-reads-its-notes`,
  after this unit, once bolt tailrace-the-gate-and-the-channels is taken
  into this bolt.
- The web page and the herdr pane showing notes (132): the page's
  serving process is on the gate-and-channels bolt; once it is taken in,
  the page renders the status answer, which carries the notes from this
  change.
- The gate's judgment of the notes artifact (28): the gate is on the
  gate-and-channels bolt and judges every artifact of an index against
  every required claim, so a policy requiring a verdict would find the
  notes, which have no build, wanting. What acceptance requires of a
  version's notes is a question for the design agent, answered when that
  bolt is taken in.
- A template the repository's authors choose for their notes: the
  design names none, so the notes take one fixed form here.
- A conformance scenario citing 134: the suite's statement list stops at
  `agentplot/blueprints 5b00e15`, whose clauses end at 125; the commit adopting the
  statement past it cites 134. Until then the publish group's checks of
  the notes cite 6, 17, 18 and 21.

## Capabilities

### New Capabilities

- `version-notes`: a version's notes rendered from the commits it was
  derived from by the rule's tool, carried as an artifact of the
  reserved `notes` ecosystem named in the index, claimed as the
  publisher, never written again, and no output packed under that
  ecosystem.

### Modified Capabilities

- `publishing`: the provenance claim is per output's artifact, the
  notes carrying their own; a second publish compares the notes too; the
  answer names the notes.
- `host-record`: an entry keeps the commits its version's notes were
  rendered from; status shows each version's notes rendered from git;
  a rebuild recovers the commits by deriving again.
- `attestation-verification`: verify shows the notes the index names.
- `conformance-suite`: S1 holds the notes in the index before it, S2
  holds them unchanged, and the status group holds the notes beside the
  version.

## Impact

- `crates/tailrace-core`: `version.rs` keeps the commits a version is
  derived from and renders its notes; `publish.rs` puts, attests, names
  and compares them; `published.rs` keeps the commits in an entry;
  `status.rs`, `record.rs` and `verify.rs` show and rebuild them;
  `declared.rs` refuses the reserved ecosystem; `history.rs` reads a commit's message.
- `crates/tailrace/src/commands`: publish, status and verify print the
  notes beside the version; status opens the history it already opens.
- The conformance suite's publish and status groups, the core's tests,
  and the site's how-it-works page.
- No new dependency: git-cliff-core's templates are already built in.
- Overlaps the in-flight unit
  `the-store-names-an-artifact-by-ecosystem-name-and-version`, which
  edits `publish.rs` too and would name the notes
  `artifact/notes/<component>@<version>` like any artifact.
- Rules kept: core.4 (status and rebuild act on revisions and digests
  they resolved), core.5 (the notes are put under their own digest and
  never rewritten), core.8 and core.9 (the notes' claim is the
  publisher's own, naming the revision and no build), core.12 (the
  notes are put and claimed before the index), core.13 and core.37 (the
  notes are rendered from the component's commits after its name is
  claimed, and no build sees them), core.19 (no notes and unreadable
  notes are two answers), core.20 (the record is rebuilt from the store
  and git), core.27, core.31 and core.32 (status reaches no store and
  writes nothing).

## Touches

```
crates/tailrace-core/src/version.rs
crates/tailrace-core/src/publish.rs
crates/tailrace-core/src/published.rs
crates/tailrace-core/src/status.rs
crates/tailrace-core/src/record.rs
crates/tailrace-core/src/verify.rs
crates/tailrace-core/src/declared.rs
crates/tailrace-core/src/history.rs
crates/tailrace-core/src/lib.rs
crates/tailrace-core/src/notes.rs
crates/tailrace-core/tests/**
crates/tailrace-conformance/src/core/publish.rs
crates/tailrace-conformance/src/core/status.rs
crates/tailrace-conformance/src/core/verify.rs
crates/tailrace/src/commands/publish.rs
crates/tailrace/src/commands/status.rs
crates/tailrace/src/commands/verify.rs
crates/tailrace/tests/**
site/src/content/docs/how-it-works.mdx
```
