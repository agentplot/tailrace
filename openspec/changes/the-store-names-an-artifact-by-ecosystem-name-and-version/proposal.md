## Why

A consumer's lockfile pins a package by its name and version, and
nothing in the store answers that pair today: a publish names a version
only by its component's tag, `<prefix><version>`, which a consumer
cannot spell without knowing the publisher's components and prefixes.
This change makes every publish also name each artifact by its
ecosystem, its name and its version, so a consumer finds what its
lockfile pins by those three alone and reads the version's claims from
the index it lands on (`agentplot/blueprints:design/tailrace/requirements.md`
138 at e267a30, with 3, 13, 49, 60, 109, 113, 137, B.6 and C.1).

## What Changes

- **Each artifact gets a name of its own in the store.** For every
  artifact of the version, a publish makes the name
  `artifact/<ecosystem>/<name>@<version>` at the digest of the version's
  index, beside the component's tag for the same index. The name is made
  from nothing; it is never moved once made. It is built from the
  artifact's metadata alone, so it names no host, no owner's store, no
  component, no prefix and no repository (138, 3, 49).
- **The names are read first and made before the version's tag.** A
  publish reads every artifact's name before it moves any of them. One
  already at this index is left as it is; one at another index refuses
  the publish, naming the name, the index it is at and this index, and
  the version's tag is not made. Only once every artifact's name stands
  is the version's tag made, so a version the store names by its tag is
  always findable by name and version (138, 70, 109, core.6).
- **The layout policy is in place before any name is made.** The
  publisher's layout-policy claim, which render-under-a-policy attaches
  to the index, is attached before the first artifact's name is made, so
  a consumer that resolves a name finds the claims it needs on the index
  (113, 8).
- **A second publish fills in what is missing.** A publish of a revision
  whose version is already named makes any artifact name the store lacks
  at that index, and writes nothing else. A version published before this
  change gets its names the next time it is published.
- **The index names where it came from.** A new index names the revision
  it was published from and the publishing repository by the owner and
  name in its landed branch's remote's address (13, 114). A repository
  whose remote gives no owner and name has its index name none. An index
  written before this change still reads back, naming neither, and keeps
  its digest.
- **The registry spells the name as it spells a channel's.** The
  registry profile escapes `artifact/<ecosystem>/<name>@<version>` into
  a tag by the same rule it uses for every name, a channel's among them,
  in the one repository the binding declares (C.1). The layout store
  keeps it as written.
- **No component's tag enters a reserved space.** A policy file whose
  component prefix, or whose rule's prefix, begins `artifact/`, `cache/`
  or `channel/` is refused when it is read, naming the component and the
  space (138).
- **The answer names each artifact's name.** A publish's answer gives
  each artifact the store name it was made under (19, 57).

Out of scope, and what each waits on:

- Finding a pin through these names, judging it under a layout's policy,
  and the publisher's layout-policy claim itself (137, 113, 99):
  render-under-a-policy, which this change builds on. Its code must be
  on this bolt before this change's code starts.
- Listing the versions the store holds of one name, so a range with no
  pin resolves (138 as it stands on `main` after `d91401e`): the
  cargos-registry unit adds the store's listing.
- A copy to another store making these names there (103, store-copy):
  not in this unit's intent; named for the planner.
- A conformance scenario citing 138 or 137: the suite's statement list
  stops at `agentplot/blueprints 5b00e15`; the commit adopting the
  statement past it cites them. Until then the scenarios here cite 3,
  13, 49 and 70.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `publishing`: a publish names each artifact by ecosystem, name and
  version before the version's tag; a second publish fills in missing
  names; the answer names them; a publish moves no other name.
- `seam-contracts`: index metadata names the revision it was published
  from and the publishing repository where it has one.
- `component-versions`: a component's prefix may not enter `artifact/`,
  `cache/` or `channel/`.
- `layout-store`: an index's revision and repository are held in its
  bytes and read back, and an index without them keeps its digest.
- `registry-store`: the artifact name is escaped into a tag as every
  name is.
- `conformance-suite`: S1 over the fakes holds the artifact names and
  the index's revision and repository.

## Impact

- `crates/tailrace-seams`: index metadata gains the revision and the
  repository; the repository's owner and name type moves here from
  `tailrace-core`'s `served.rs`, which re-exports it.
- `crates/tailrace-core`: publish makes the artifact names and reads the
  repository from the landed branch's remote; the policy file reader
  refuses a reserved prefix.
- `crates/tailrace/src/profiles/store`: the OCI encoding writes and reads
  the index's revision and repository annotations; tests for the
  registry's spelling of the name.
- The fake store, the conformance suite's S1 and publish scenarios, and
  every test that counts the names a publish moves.
- The site's stores page says how a consumer finds an artifact by name.
- No new dependency.
- Depends on bolt tailrace-the-gate-and-the-channels being on this bolt
  (it holds the repository's owner and name type and the read of a
  remote's address), as npm-rendered-and-served does.
- Rules kept: core.1 and core.2 (the core spells the name; only the
  registry profile turns it into a tag), core.4 (a name is resolved
  before it is acted on, and the answer names the digest), core.5, core.6
  (a name made only from nothing, a refused make reported and never
  retried), core.9 (the index names its revision), core.12 (the index is
  still offered last; names are not offers), core.18 (the name names no
  host or owner's store), core.33 (the S1 scenario cites what it
  exercises).

## Touches

```
crates/tailrace-seams/src/artifact.rs
crates/tailrace-seams/src/answer.rs
crates/tailrace-seams/src/lib.rs
crates/tailrace-core/src/publish.rs
crates/tailrace-core/src/declared.rs
crates/tailrace-core/src/served.rs
crates/tailrace-core/src/history.rs
crates/tailrace-core/src/copy.rs
crates/tailrace-core/src/render.rs
crates/tailrace-core/src/lib.rs
crates/tailrace-core/tests/**
crates/tailrace-fakes/src/store.rs
crates/tailrace-conformance/src/**
crates/tailrace/src/commands/publish.rs
crates/tailrace/src/profiles/store/oci.rs
crates/tailrace/src/profiles/store/layout/**
crates/tailrace/src/profiles/store/registry/**
crates/tailrace/tests/**
site/src/content/docs/guides/integrations/stores.mdx
```
