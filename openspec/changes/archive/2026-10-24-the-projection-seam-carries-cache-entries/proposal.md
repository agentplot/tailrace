# Proposal

## Why

Bolt 3 makes a task that passed on one host replay from the registry on
another: moon's cache namespace packs a task's output archive by its
task hash and renders moon's outputs directory from the archives the
landing's verdicts name (C.3, C.11). Today the projection seam cannot
say that. Its pack takes an output and a version and answers artifact
metadata, and its render takes artifacts and answers versions, so a
cache entry, which has a build identity and no version and is never an
artifact (52, 53), has no way through it. The statement's projection
contract was amended so a cache entry is one case of pack and render,
not a pair of twin operations (84, 85, 87, as amended for cache
entries). The phase's rule is that a trait never changes inside a
profile unit (phase-1-bolts.md, How the units stay apart), so the seam
changes here, in a unit of its own, before moon's cache namespace and
the push after a landing use it.

## What Changes

- **pack is given an output at a build identity, and the version only
  where the bytes are an artifact's.** Given a version, pack answers the
  ecosystem's own bytes and the metadata that makes them an artifact, as
  today (84). Given no version, it packs a cache entry: the bytes carry
  no metadata that makes an offer, and the answer carries no artifact
  metadata at all (100). **BREAKING** for every implementer of the
  projection interface: its pack takes the build identity and an
  optional version, and its answer's metadata is present only for an
  artifact.
- **A cache entry is a type of the seams.** It is named by its build
  identity and its digest, and by nothing else: no ecosystem, no name,
  no version, so nothing that takes an artifact can be handed one
  (52, 53, core.30).
- **render is given artifacts, or cache entries by build identity.** It
  answers the consumer layout and which versions it holds, or which
  build identities it holds (85). **BREAKING** for every implementer and
  caller of render.
- **The same build identity packs to the same bytes**, or the projection
  says it cannot promise so, as the same outputs and version already do
  (87).
- **A projection that packs no cache entries answers absent for one.**
  The seams give every projection one shared answer for it, absent
  naming cache entries, never refused or unreadable (58). The file and
  image projections give it for a pack with no version and a render of
  cache entries; their artifact packing is unchanged.
- **The fake projection scripts both cases.** By default it packs and
  renders cache entries; a test can make it a projection that packs
  none, which answers absent for one.
- **The conformance suite holds every projection to both cases.** The
  projection group gains three scenarios below its marker: a cache entry
  packed with no version, a layout of the build identities given, and a
  projection that packs no cache entries. Each projection's rig says
  which case it is; the scenarios of the other case are not applicable
  to it. The fakes run the group twice, once for each case; the file and
  image projections run the scenario of a projection that packs none.
  The group's scenarios above the marker change only in how they call
  the widened operations, never in what they hold.
- **publish passes each output's build identity and its version to
  pack**, and refuses a pack given a version that answers no artifact
  metadata, naming the output, before anything is offered (84, 15).
- **The site's projections page** says pack takes the build identity
  and, for an artifact, the version, and that a cache entry packs with
  none.

Out of scope, and what each waits on:

- Writing, reading or removing cache entries in the store, the core's
  cache module, and the scenarios that cite 52 to 54 and 100: the unit
  `moons-cache-namespace`, which comes after this one. Those clauses
  stay in the conformance register under bolt 3's cache.
- moon's cache projection itself, packing a task's output archive by
  task hash and rendering moon's outputs directory (C.11): the same
  unit.
- Pushing a landing's archives as cache entries (C.3): the unit
  `the-push-after-a-landing`, which packs them through this seam.
- A file or image projection that renders anything: bolt 5's render
  under a policy.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `seam-contracts`: a cache entry is one of the types that cross a
  seam, and a projection's contract takes the build identity, an
  optional version and cache entries, answering absent where it packs
  none.
- `seam-fakes`: the fake projection packs and renders cache entries, or
  is scripted to pack none.
- `conformance-suite`: the projection group holds a projection to the
  cache-entry case of 84, 85 and 87, and to absent where it packs none.

## Impact

- `crates/tailrace-seams/src/projection.rs`: the trait, `CacheEntry`,
  what render is given and what it holds, and the shared absent answer;
  `crates/tailrace-seams/src/lib.rs`'s test implementation of every
  seam.
- `crates/tailrace-fakes/src/projection.rs` and the fakes' own test in
  `crates/tailrace-fakes/src/lib.rs`.
- `crates/tailrace-conformance/src/seams/projection.rs` (the calls above
  the marker, three scenarios below it), the fakes' rig in
  `src/fakes.rs`, the binding group's pack call in
  `src/seams/binding.rs`, the watched projection in `src/world.rs`, and
  `tests/fakes.rs`.
- `crates/tailrace-core/src/publish.rs`, the watched projection in
  `crates/tailrace-core/tests/world/mod.rs`, and one test in
  `crates/tailrace-core/tests/publish.rs`.
- `crates/tailrace/src/profiles/projection/file/mod.rs`,
  `image/mod.rs`, and `crates/tailrace/tests/projections.rs`.
- `site/src/content/docs/guides/integrations/projections.mdx`.
- No crate, dependency, language or file type is added; the register is
  not changed.

## Touches

```
crates/tailrace-seams/src/projection.rs
crates/tailrace-seams/src/lib.rs
crates/tailrace-fakes/src/projection.rs
crates/tailrace-fakes/src/lib.rs
crates/tailrace-conformance/src/seams/projection.rs
crates/tailrace-conformance/src/seams/binding.rs
crates/tailrace-conformance/src/fakes.rs
crates/tailrace-conformance/src/world.rs
crates/tailrace-conformance/tests/fakes.rs
crates/tailrace-core/src/publish.rs
crates/tailrace-core/tests/world/mod.rs
crates/tailrace-core/tests/publish.rs
crates/tailrace/src/profiles/projection/file/mod.rs
crates/tailrace/src/profiles/projection/image/mod.rs
crates/tailrace/tests/projections.rs
site/src/content/docs/guides/integrations/projections.mdx
```
