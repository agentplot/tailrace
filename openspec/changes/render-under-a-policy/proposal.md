## Why

A family repository that depends on another family repository's
packages cannot yet get them from anything tailrace holds: `tailrace
sync` fills moon's cache and nothing else, no projection says what a
checkout's lockfile pins, and nothing judges a version under a policy
before a consumer's client sees it. This unit makes `tailrace sync` fill
each ecosystem's local layout in the checkout with exactly the versions
the checkout's lockfiles pin, and only those whose claims satisfy the
policy, so a developer's install and an agent's build read only what the
policy admits (`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`,
Bolt 5, "render under a policy"; ADR 0006).

## What Changes

- **A projection says what a checkout's lockfile pins.** The projection
  seam gains its fourth operation, pinned: given a checkout, the names
  and versions its lockfile pins as the lockfile spells them, or absent
  where it holds no lockfile of that ecosystem (137). The file, image and
  moon cache projections answer absent; the fake projection answers what
  the test sets. The npm and Cargo units implement it for their
  ecosystems.
- **Each pin is found in the store by name and version alone.** The core
  resolves `artifact/<ecosystem>/<name>@<version>` to the version's
  index and finds the artifact of that ecosystem and name in it by its
  metadata (138, 4). A name the store does not hold is answered absent:
  most of a lockfile's pins are third-party packages tailrace never
  published, and none of them is a refusal (51, 58).
- **Each version is judged under the layout's policy.** The policy is the
  consuming repository's own policy for layouts at the landed revision
  sync runs for (113, 65); where the repository declares none, it is the
  publisher's policy for layouts, read from the claim the publish
  attached to the version's index (113). A version is held only when,
  for every claim the policy requires in its order, a claim attached to
  the index or to the artifact counts through a signer the policy names
  (47, 9, core.7). A version refused is answered to whoever ran sync,
  naming the version and the first claim found wanting (99). A version
  for which no policy for layouts can be found anywhere is refused,
  naming the publisher's layout policy as the claim found wanting, so no
  layout is ever rendered with no policy (core.17).
- **A layout lives in its checkout and holds only what it was given.**
  Each ecosystem's layout is rendered at `.tailrace/layouts/<ecosystem>/`
  in the checkout's working tree, a path the consumer's own configuration
  names once and that is the same on every host (49, S4); a version the
  policy no longer admits leaves the layout on the next sync (47, 48).
- **A layout that cannot be refreshed says what it holds.** When the
  store or the policy cannot be read for a layout, the layout is left as
  it was and sync answers what it holds, when it was last rendered and
  under which policy, beside what could not be read; it never answers an
  absent version as refused or a refused one as absent (51, S10).
- **What tailrace keeps of rendered layouts is the host's.** One file per
  layout, under the checkout's own git directory, names the place, when
  it was rendered, the policy it was rendered under, and each version
  held and refused, readable with a file reader and rebuilt by the next
  sync (61 to 63).
- **A publish attaches the publisher's layout policy to the index.** Where
  the publishing repository declares a policy for layouts at the landed
  revision, the publish signs it as a claim of the publisher's about the
  index and attaches it before the version's name is moved, so a
  consumer with no policy of its own reads it from the store (113, 8,
  core.8, core.9).
- **`tailrace sync` answers the cache and the layouts in one answer.** It
  fills moon's cache as it does now and then each layout, naming every
  pin as held, refused with its claim, or absent, and each layout as
  refreshed or as what it still holds; `--json` prints the one answer
  (56, 57).

Out of scope, and what each waits on:

- Writing and moving the `artifact/<ecosystem>/<name>@<version>` pointer
  at publish, the store profiles carrying it, and keeping component tags
  out of `artifact/` (109, 138): the unit
  the-store-names-an-artifact-by-ecosystem-name-and-version. Until it
  lands, sync finds a pin only where a test or a store already holds the
  name.
- npm's and Cargo's pinned, render and serve, and the one serving process
  per host (114, 115, C.11): the units npm-rendered-and-served and
  cargos-registry. A served layout answering a refusal in its response to
  the client (99) and the render on a miss (50, 115) call this unit's
  render.
- The conformance suite's statement list adopting clauses 126 to 138: the
  commit that adopts the statement past `agentplot/blueprints 5b00e15`.
  Until then no scenario can cite 137 or 138, so the pinned scenario
  cites 50 and 58.
- A layout policy requiring a signature still valid when the layout
  renders (117): the policy file's form names no such requirement yet.
- Showing layouts in `tailrace status` and on the status page (60, 132):
  status across machines and the herdr plugin.

## Capabilities

### New Capabilities

- `consumer-layouts`: a checkout's layouts rendered from the store under
  the consuming repository's policy or the publisher's, holding only
  satisfied versions, refusals answered with their claim, a layout that
  cannot be refreshed answering what it holds, and the host's record of
  each layout.

### Modified Capabilities

- `seam-contracts`: the projection seam gains pinned.
- `seam-fakes`: the fake projection answers pinned from what the test
  sets.
- `cache-entries`: filling the cache still reads no policy, now that sync
  also renders layouts that do.
- `command-line`: `tailrace sync` answers the layouts beside the cache
  entries.
- `publishing`: a publish attaches the publisher's layout policy to the
  index.
- `conformance-suite`: the projection group holds pinned, and the
  layouts group holds the core to 47, 50, 51, 61 to 63, 99 and 113,
  playing S4.
- `file-projection`, `image-projection`, `moon-cache-projection`: each
  answers pinned absent.

## Impact

- New core module `crates/tailrace-core/src/render.rs`; the projection
  trait and claim kinds in `tailrace-seams`; publish, verify's telling of
  a claim, and the history's per-checkout git directory in
  `tailrace-core`; `tailrace sync` in `crates/tailrace`.
- Every `Projection` implementation gains pinned: the fake, the file,
  image and moon cache profiles, and the conformance rigs.
- The conformance register loses 47, 51, 99 and 113.
- The site's projections page says how a consumer reads under a policy.
- No new dependency.
- Rules kept: core.1 and core.2 (the core names no ecosystem; pins read
  by the projection, resolved by the core through the store seam),
  core.4 (every pin resolved to a digest and the digest recorded),
  core.7 (a claim counts only through a signer the policy names), core.8
  and core.9 (the publisher's claim signed as the publishing host, naming
  the revision), core.17, core.18 (a layout path names no host), core.19
  and core.20 (three answers kept apart; the record readable with no
  process and rebuilt), core.22 (the consumer's policy read at a landed
  revision), core.31 (one core answer rendered by the command), core.33.

## Touches

```
crates/tailrace-seams/src/projection.rs
crates/tailrace-seams/src/claim.rs
crates/tailrace-seams/src/answer.rs
crates/tailrace-seams/src/lib.rs
crates/tailrace-core/src/render.rs
crates/tailrace-core/src/lib.rs
crates/tailrace-core/src/publish.rs
crates/tailrace-core/src/verify.rs
crates/tailrace-core/src/history.rs
crates/tailrace-core/tests/**
crates/tailrace-fakes/src/projection.rs
crates/tailrace-conformance/src/**
crates/tailrace-conformance/tests/**
crates/tailrace/src/commands/sync.rs
crates/tailrace/src/render.rs
crates/tailrace/src/profiles/projection/file/mod.rs
crates/tailrace/src/profiles/projection/image/mod.rs
crates/tailrace/src/profiles/projection/moon_cache/mod.rs
crates/tailrace/src/profiles/projection/moon_cache/tests.rs
crates/tailrace/tests/**
site/src/content/docs/guides/integrations/projections.mdx
```
