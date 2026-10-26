## Why

A consumer's lockfile pins a package by its name and version, and
nothing in the store answers that pair today: a publish names a version
only by its component's tag, `<prefix><version>`, which a consumer
cannot spell without knowing the publisher's components and prefixes.
This change makes every publish name each artifact by its ecosystem, its
name and its version, and lets the store list the versions a name holds,
so a consumer finds what its lockfile pins by those alone and reads the
version's claims from the index it lands on. On a registry it also keeps
three spaces a credential can be scoped to apart: each component's own
repository for its indexes and channels, a space for cache entries, so a
credential that writes cache entries and nothing else can push them, and
a space for the artifacts' names
(`agentplot/blueprints:design/tailrace/requirements.md` at 4c14235: 138,
with 3, 13, 49, 60, 109, 113, 125, 135, 137, B.6 and C.1).

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
  under `artifact/<ecosystem>/<name>@`, an empty list where it names
  none, and unreadable only where the read fails. The layout store and
  the fake answer from the names they hold (138). The listing is this
  change's alone; cargos-registry reads it and adds none of its own.
- **A registry keeps three spaces apart.** The registry profile holds
  three spaces of repositories under the one the binding declares, each
  name lowercased and escaped into a path component as the distribution
  protocol allows:
  - **A component's own repository.** A declared component's channels
    are tags of the repository named for the component (`lib` for the
    component `lib`), the channel's name the tag (`beta`), and each of
    its indexes, with every artifact and blob it names, is held there
    from the publish that puts it. A repository that declares no
    component has the declared repository as its component's own, and
    nothing changes for it (125). A promotion moves the channel's tag
    in the component's repository, reading it back there (109).
  - **Cache entries.** A cache entry, `cache/<build identity>`, moves
    out of the declared repository into `cache/<build>` under it, one
    repository per build profile (`cache/moon`), its build identity the
    tag; a push binds the store so its bytes go there too, so a
    credential that writes `cache/` alone pushes entries and can offer,
    accept or move nothing.
  - **Artifact names.** `artifact/<ecosystem>/<name>@<version>` is the
    tag `<version>` of the repository `artifact/<ecosystem>/<name>`, so
    a name's versions are that repository's tag list. The version's
    characters a tag refuses, `+` among them, are escaped one way and
    read back. The index and what it names are mounted into the
    repository before its tag is put.

  Bytes, artifacts, indexes and claims are put in the declared
  repository, where the seam reads every digest, and a version's tag
  stays a tag of it (C.1, 135). An ADR records this, in part in place of
  ADR 0014's one repository.
- **A component's name names one repository.** A policy file that
  declares a component named `artifact`, `cache` or `repository`, or
  holding a `/`, or two components whose names differ only by case, is
  refused when it is read, naming the components, so every declared
  component has a repository of its own apart from the other spaces and
  from the undeclared component's (125, C.1).
- **A missing repository is reported naming it.** A registry that makes
  repositories on push only by a creation template, ECR among them,
  refuses a put into a repository nobody made; the put is answered in
  the registry's words naming the repository, and the self-hosting guide
  says what the operator makes, or what template covers, the
  components' repositories, `cache/` and `artifact/`, before the first
  publish (C.1, 58).
- **No component's tag enters a reserved space.** A policy file whose
  component prefix, or whose rule's prefix, begins `artifact/`, `cache/`
  or `channel/` is refused when it is read, naming the component and the
  space (138).
- **The answer names each artifact's name.** A publish's answer gives
  each artifact the store name it was made under (19, 57).
- **The suite cites 138.** The store group proves the listing on every
  store profile, the publish group's S1 proves the names and the index's
  revision, and a store-naming group proves a pin found through what a
  publish wrote; 138 leaves the register, and the parts of 137 about a
  range and the closure are named in the register as cargos-registry's.

Out of scope, and what each waits on:

- The core's loop over a checkout's names (137): a manifest's range
  listed through this change's listing, each version judged under the
  layout's policy, the newest admitted held, and what every held version
  requires asked again until nothing held requires a name the layout
  lacks; with the range types and their order: cargos-registry, which
  lands after this change and builds on its listing
  (`crates/tailrace-core/src/render.rs`).
- What a held version's artifact requires of the family (140, B.6):
  the-projection-answers-what-an-artifact-requires.
- A copy to another store carrying these names (103): later work the
  planner places.
- The templates a fleet makes for ECR in its own configuration
  (swancloud's at adoption): told to that fleet separately.
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
  the store's contract gains the listing of a name's versions; an
  artifact's store name is spelled once.
- `component-versions`: a component's prefix may not enter `artifact/`,
  `cache/` or `channel/`, and a component's name names one repository.
- `layout-store`: an index's revision and repository are held in its
  bytes; a name's versions are listed from the index's names.
- `registry-store`: a declared component's channels are tags of its own
  repository, which holds its indexes; an artifact's name is a tag of a
  repository of its own, and its versions are that repository's tags; a
  cache entry is a tag of its build profile's cache repository; a
  missing repository is refused naming it.
- `conformance-suite`: the store group lists a name's versions; S1 holds
  the artifact names and the index's revision; a store-naming group over
  the fakes finds a pin through what a publish wrote; 138 leaves the
  register.

## Impact

- `crates/tailrace-seams`: index metadata gains the revision and the
  repository; a channel's store name is read back to its channel;
  `RepositoryName` moves here from `tailrace-core`'s `served.rs`, which
  re-exports it; the artifact space's spelling and
  its reading back; the store seam's listing.
- `crates/tailrace-core`: publish makes the artifact names and reads the
  repository from the landed branch's remote; the policy file reader
  refuses a reserved prefix and a component's name that names no
  repository of its own; `render.rs` re-exports the moved spelling.
- `crates/tailrace/src/profiles/store`: the OCI encoding writes and reads
  the index's revision and repository; the layout store lists; the
  registry maps a declared component's channel, an artifact name and a
  cache entry to their repositories, mounts what each needs, and lists
  an artifact name's tags.
- `crates/tailrace/src/bindings` and `profiles/build`: the store binding
  is given the build profile's name and, for a push, writes bytes into
  the cache space.
- The fake store, the conformance suite's store, publish and cache
  groups and register, and every test that counts the names a publish
  moves.
- A new ADR beside ADRs 0014 and 0015; the site's stores page and its
  self-hosting guide.
- No new dependency.
- Depends on bolt tailrace-the-gate-and-the-channels being on this bolt
  (`RepositoryName`, `History::remote_url`, the register's parts), and on
  the fix to render-under-a-policy that adopts the statement through 138
  and registers 138 as this unit's.
- Overlaps cargos-registry at the store seam: it reads this change's
  listing, and this change lands first.
- Rules kept: core.1 and core.2 (the core spells the name; only the
  registry profile turns it into a repository and tag, and the bindings
  name no tool), core.4 (every name resolved before it is acted on),
  core.5 (a mount copies bytes under their own digest), core.6 (a name
  made only from nothing, a refused make reported and never retried),
  core.9 (the index names its revision), core.12 (the index is still
  offered last; names are not offers), core.14 (a push's credential
  writes cache entries alone), core.18 (the name names no host or
  owner's store), core.19 (a listing that fails is unreadable, never an
  empty list), core.30 (a cache entry stays apart from every artifact),
  core.33 (the scenarios cite what they exercise).

## Touches

```
crates/tailrace-seams/src/artifact.rs
crates/tailrace-seams/src/answer.rs
crates/tailrace-seams/src/store.rs
crates/tailrace-seams/src/keeper.rs
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
crates/tailrace-fakes/src/lib.rs
crates/tailrace-conformance/src/**
crates/tailrace-conformance/tests/**
crates/tailrace/src/bindings/*
crates/tailrace/src/commands/publish.rs
crates/tailrace/src/profiles/build/mod.rs
crates/tailrace/src/profiles/store/mod.rs
crates/tailrace/src/profiles/store/oci.rs
crates/tailrace/src/profiles/store/layout/**
crates/tailrace/src/profiles/store/registry/**
crates/tailrace/tests/**
docs/adr/*
site/src/content/docs/guides/integrations/stores.mdx
site/src/content/docs/guides/self-hosting.mdx
```
