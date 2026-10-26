## Why

A consumer's lockfile pins a package by its name and version, and
nothing in the store answers that pair today: a publish names a version
only by its component's tag, `<prefix><version>`, which a consumer
cannot spell without knowing the publisher's components and prefixes.
And a package a manifest declares but no lockfile pins yet has no pin to
look up at all. This change makes every publish name each artifact by
its ecosystem, its name and its version, lets the store list the
versions a name holds, and lets sync resolve a manifest's range by that
listing under the layout's policy, so a consumer finds what it depends
on by those alone and reads the version's claims from the index it lands
on (`agentplot/blueprints:design/tailrace/requirements.md` at fbecdbf:
138, with 3, 13, 49, 60, 109, 113, 137, B.1, B.6 and C.1).

## What Changes

- **Each artifact gets a name of its own in the store.** For every
  artifact of the version, a publish makes the name
  `artifact/<ecosystem>/<name>@<version>` at the digest of the version's
  index, beside the component's tag for the same index. The name is made
  from nothing and never moved once made. It is built from the
  artifact's metadata alone, so it names no host, no owner's store, no
  component, no prefix and no repository (138, 3, 49).
- **The names are read first and made before the version's tag.** A
  publish reads every artifact's name before it makes any. One already
  at this index is left as it is; one at another index refuses the
  publish, naming the name, the index it is at and this index, and the
  version's tag is not made. Only once every artifact's name stands is
  the version's tag made, so a version the store names by its tag is
  always findable by name and version (138, 70, 109, core.6).
- **The publisher's layout policy is on the index before any name.** The
  layout-policy claim a publish already attaches to the index, the
  claims a layout must require as the repository declared them at that
  revision, is attached before the first artifact's name is made, on a
  repeat publish too where the index lacks it, so a consumer that
  resolves a name finds the claims it needs on the index (113, 8).
- **A second publish fills in what is missing.** A publish of a revision
  whose version is already named makes any artifact name the store lacks
  at that index, and writes nothing else. A version published before
  this change gets its names the next time it is published.
- **The index names where it came from.** A new index names the revision
  it was published from and the publishing repository by the owner and
  name in its landed branch's remote's address (13, 138). A repository
  whose remote gives no owner and name has its index name none. An index
  written before this change still reads back, naming neither, and keeps
  its digest.
- **The store lists the versions a name holds.** The store seam gains a
  listing: given an ecosystem and a name, the versions the store names
  under `artifact/<ecosystem>/<name>@`, or absent where it names none.
  The layout store and the fake answer from the names they hold (138).
- **A registry gives each artifact name a repository of its own.** The
  registry profile holds `artifact/<ecosystem>/<name>@<version>` as the
  tag `<version>` of the repository `artifact/<ecosystem>/<name>` under
  the one the binding declares, lowercased and escaped as the
  distribution protocol allows, so a name's versions are that
  repository's tag list and no tag carries a package name. The version's
  characters a tag refuses, `+` among them, are escaped one way and read
  back. The index and what it names are mounted into that repository
  from the declared one before the tag is put; everything else stays in
  the declared repository (C.1). An ADR records this beside ADR 0014.
- **A manifest's range resolves to the newest version the policy
  admits.** A projection's pinned may answer, for a name the checkout's
  manifest declares and no lockfile pins yet, the manifest's range. Sync
  lists that name's versions, judges each under the layout's policy, and
  has the projection's render hold the newest admitted version within
  the range, so a first install resolves and the client writes the pin.
  Every listed version the policy refuses is answered naming its claim
  (137, 138, 99, 23).
- **No component's tag enters a reserved space.** A policy file whose
  component prefix, or whose rule's prefix, begins `artifact/`, `cache/`
  or `channel/` is refused when it is read, naming the component and the
  space (138).
- **The answer names each artifact's name.** A publish's answer gives
  each artifact the store name it was made under (19, 57).
- **The suite cites 138, and 137's range.** The store group proves the
  listing on every store profile, the publish group's S1 proves the
  names and the index's revision, and the layouts group proves a pin
  and a range resolved through what a publish wrote; 138, and the part
  of 137 about a range, leave the register.

Out of scope, and what each waits on:

- A copy to another store carrying these names (103): later work the
  planner places; the clause's scenario comes with the unit that does it.
- Which versions fall within a range, and their order, for npm and Cargo
  (137, 23): npm-rendered-and-served and cargos-registry answer pinned's
  range and render the newest within it for their ecosystems; this
  change gives the seam and the core's half, and the fake projection.
- Pack given the component and its declared outputs (84): cargos-registry.
- Serving a layout, and a sparse index leaving a refused version out of
  its listing (99, 114, 115): npm-rendered-and-served and
  cargos-registry.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `publishing`: a publish names each artifact by ecosystem, name and
  version after the layout policy and before the version's tag; a second
  publish fills in missing names and a missing layout policy; the answer
  names them; a publish moves no other name.
- `seam-contracts`: index metadata names its revision and repository;
  the store's contract gains the listing of a name's versions; a
  projection's pinned may answer a range and its render holds the newest
  version within it; an artifact's store name is spelled once.
- `component-versions`: a component's prefix may not enter `artifact/`,
  `cache/` or `channel/`.
- `layout-store`: an index's revision and repository are held in its
  bytes; a name's versions are listed from the index's names.
- `registry-store`: an artifact's name is a tag of a repository of its
  own, and its versions are that repository's tags.
- `consumer-layouts`: a range with no pin resolves to the newest version
  the policy admits within it.
- `seam-fakes`: the fake projection answers ranges and holds the newest
  version within each.
- `conformance-suite`: the store group lists a name's versions; S1 holds
  the artifact names and the index's revision; a store-naming group over
  the fakes resolves a pin and a range through what a publish wrote; 138
  and 137's range part leave the register.

## Impact

- `crates/tailrace-seams`: index metadata gains the revision and the
  repository; `RepositoryName` moves here from `tailrace-core`'s
  `served.rs`, which re-exports it; the artifact space's spelling and
  its reading back; the store seam's listing; the projection seam's
  range (`Pin`, `Ranged`, `Range`, `Contents::Ranged`).
- `crates/tailrace-core`: publish makes the artifact names and reads the
  repository from the landed branch's remote; render resolves ranges;
  the policy file reader refuses a reserved prefix.
- `crates/tailrace/src/profiles/store`: the OCI encoding writes and reads
  the index's revision and repository; the layout store lists; the
  registry maps an artifact name to its repository and tag, mounts the
  index into it, and lists its tags.
- Every store and projection implementation: the fakes, the layout and
  registry stores, the file, image and moon cache projections, and the
  conformance rigs.
- The fake store and projection, the conformance suite's store,
  publish and layouts groups and register, and every test that counts
  the names a publish moves.
- A new ADR beside ADR 0014; the site's stores and projections pages.
- No new dependency.
- Depends on bolt tailrace-the-gate-and-the-channels being on this bolt
  (`RepositoryName`, `History::remote_url`, the register's parts), and on
  the fix to render-under-a-policy that adopts the statement through 138
  and registers 138 and 137's range as this unit's.
- Overlaps cargos-registry and npm-rendered-and-served at the projection
  seam: each implements pinned and render, which this change reshapes.
- Rules kept: core.1 and core.2 (the core spells the name; only the
  registry profile turns it into a repository and tag; the projection
  never reads the policy), core.4 (every name resolved before it is
  acted on, every listed version resolved to its index), core.5 (a
  mount copies bytes under their own digest), core.6 (a name made only
  from nothing, a refused make reported and never retried), core.9 (the
  index names its revision), core.12 (the index is still offered last;
  names are not offers), core.17 (every listed version judged before it
  can be held), core.18 (the name names no host or owner's store),
  core.19 (a name with no versions is absent, a listing that fails
  unreadable), core.33 (the scenarios cite what they exercise).

## Touches

```
crates/tailrace-seams/src/artifact.rs
crates/tailrace-seams/src/answer.rs
crates/tailrace-seams/src/store.rs
crates/tailrace-seams/src/projection.rs
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
crates/tailrace-fakes/src/projection.rs
crates/tailrace-fakes/src/lib.rs
crates/tailrace-conformance/src/**
crates/tailrace-conformance/tests/**
crates/tailrace/src/commands/publish.rs
crates/tailrace/src/commands/sync.rs
crates/tailrace/src/profiles/store/oci.rs
crates/tailrace/src/profiles/store/layout/**
crates/tailrace/src/profiles/store/registry/**
crates/tailrace/src/profiles/projection/file/mod.rs
crates/tailrace/src/profiles/projection/image/mod.rs
crates/tailrace/src/profiles/projection/moon_cache/mod.rs
crates/tailrace/src/profiles/projection/moon_cache/tests.rs
crates/tailrace/tests/**
docs/adr/*
site/src/content/docs/guides/integrations/stores.mdx
site/src/content/docs/guides/integrations/projections.mdx
```
