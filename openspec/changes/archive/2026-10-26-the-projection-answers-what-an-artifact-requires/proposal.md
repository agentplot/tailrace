## Why

A layout can hold only the versions a lockfile pins, so a first install
of a family package fails as soon as that package needs another family
package nobody has pinned yet. The statement now closes a layout in the
core, level by level, by asking the projection what each held version
requires of the family and whether a version satisfies a range (137,
140). This unit gives the projection seam that question, so the units
that resolve ranges have something to ask; the core never reads a
manifest itself (rule core.1).

## What Changes

- **A projection says what a held version requires.** The projection
  seam gains **requires**: given an artifact the layout holds, the names
  of the projection's own ecosystem and the ranges its manifest requires
  of them, each spelled as the manifest spells it (140). Which of those
  names the family publishes is for the store's listing to say (138),
  not the projection.
- **A projection says which versions satisfy a range, newest first.**
  The seam gains **satisfying**: given a range and some versions, the
  ones that satisfy the range by the ecosystem's own rules, newest
  first by the ecosystem's own ordering. The core never orders versions
  itself (23), so this one answer gives it both "does this version
  satisfy" and "which is newest" (137: "the ranges and their order the
  projection's").
- **An ecosystem with no manifest answers none.** The file, image and
  moon cache projections answer an empty list to both questions, and
  read nothing. "None" is not absent: nothing went unread, and an empty
  list is what ends the core's loop (140, 58).
- **The fake answers what a test sets.** The fake projection answers
  requires with the requirements a test sets for an artifact. For any
  other artifact it answers absent, naming the manifest it lacks. It
  answers satisfying by Semantic Versioning's ordering and range rules,
  so a test can list versions out of order and get the newest that fits.
  Both operations can be faulted and are recorded.
- **The conformance suite holds every projection to it.** The
  projection group gains three scenarios below its marker:
  - requires answered as the manifest spells it;
  - satisfying answered by the ecosystem's own ordering, not by spelling
    or by the order the versions were listed;
  - none from an ecosystem with no manifest.

  Each projection's rig says whether its ecosystem has a manifest.
- **Every other implementor answers too.** The unbound projection
  answers nothing. The test projections in the core's and the suite's
  tests either pass through to the fake or answer none.
- **The site says the seam has six operations.**

Out of scope, and what each waits on:

- The core's closure loop: asking requires for each held version,
  listing the name's versions in the store, judging them and holding
  the newest admitted, through every level (137). It waits on the store
  listing a name's versions (138), and both are the units that resolve
  ranges, which build on this one.
- requires and satisfying for npm and Cargo, which belong to those
  projections' own units.
- Adding clauses 126 to 140 to the conformance suite's statement list,
  which is read at an older revision of the statement. Until that
  happens, these scenarios cite 23, 50 and 58, as the pinned scenarios
  do.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `seam-contracts`: a projection has six operations; requires and
  satisfying join the projection's contract, and requirement and range
  join the types that cross a seam.
- `seam-fakes`: the fake projection answers the requirements a test sets
  and orders versions by Semantic Versioning.
- `conformance-suite`: the projection group holds requires, satisfying
  and an ecosystem with no manifest.
- `file-projection`: a file has no manifest, so both questions are
  answered none.
- `image-projection`: an image has no manifest, so both questions are
  answered none.
- `moon-cache-projection`: a cache has no manifest, so both questions
  are answered none.

## Impact

- The `Projection` trait, in `crates/tailrace-seams`, gains two required
  methods. Every implementor in the tree gains them in the same change:
  the three profiles, the fake, the unbound projection, and the test
  projections in `tailrace-core`'s and `tailrace-conformance`'s tests.
- `tailrace-fakes` gains the workspace's `semver` dependency, which
  changes `Cargo.lock`.
- No change to the core's code, a command's output or any record.

## Touches

```
crates/tailrace-seams/src/projection.rs
crates/tailrace-seams/src/lib.rs
crates/tailrace-fakes/Cargo.toml
crates/tailrace-fakes/src/projection.rs
Cargo.lock
crates/tailrace-conformance/src/**
crates/tailrace-conformance/tests/**
crates/tailrace-core/tests/**
crates/tailrace/src/profiles/projection/**
crates/tailrace/tests/projections.rs
crates/tailrace/tests/moon_cache_projection.rs
site/src/content/docs/guides/integrations/projections.mdx
```
