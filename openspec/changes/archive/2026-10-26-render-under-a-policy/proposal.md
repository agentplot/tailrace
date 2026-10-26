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
- **A layout's policy is stacked: the consumer's line, the publisher's,
  the given.** The policy is the consuming repository's own `[layout]`
  where its file at the landed revision sync runs for writes one (113,
  65). Where it writes none, with no policy file or a file with no
  `[layout]` alike, it is the publisher's policy for layouts, read from
  the layout-policy claim the publish attached to the version's index,
  and that claim and every claim it requires count only through the
  signers the host's bindings trust (C.10, 118, 9). Where the index
  carries no such claim a trusted signer made, it is the given policy
  for a layout: a version the gate accepted and no other (C.10).
- **The given names no signer; the consumer's file does.** The policy
  file answers a file with no `[layout]` with the given, marked as
  given so the core can stack it beneath the publisher's: acceptance by
  the signers the file lets claim acceptance, or, where it names none,
  by the one signer it names for provenance, the host as the one-host
  gate (116, 108, C.10). A consumer with no policy file names no signer
  at all, and its given counts acceptance through the signers its host
  trusts (C.10, 118). A repository whose revision holds no policy file
  has no policy for layouts of its own.
- **Each version is judged under its policy.** A version is held only
  when, for every claim the policy requires in its order, a claim
  attached to the index or to the artifact counts through a signer the
  policy names (47, 9, core.7). A version refused is answered to
  whoever ran sync, naming the version and the first claim found
  wanting, and sync prints it, so a client's own "no matching version"
  is never the only word (99). Every version has a policy, the given at
  the least, so no layout is ever rendered with none (core.17).
- **The host names the signers it trusts.** The host's bindings file
  gains `[trust.<signer>]`, each naming one signer and the key it is
  verified by, in the policy file's `[signers]` shape; a host whose file
  names none trusts no signer (section 2: the operator decides which
  signers a host trusts; 118).
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
  the publishing repository's file writes a `[layout]` at the landed
  revision, the publish signs it as a claim of the publisher's about the
  index and attaches it before the version's name is moved, so a
  consumer that writes none reads it from the store (113, 8, core.8,
  core.9). A publisher that writes none attaches nothing: the given is
  laid beneath at the consumer, never carried on the index.
- **`tailrace sync` answers the cache and the layouts in one answer.** It
  fills moon's cache as it does now and then each layout, naming every
  pin as held, refused with its claim, or absent, and each layout as
  refreshed or as what it still holds; `--json` prints the one answer
  (56, 57).

Out of scope, and what each waits on:

- A name the checkout's manifest declares and no lockfile pins yet,
  answered by pinned as the manifest's range and rendered as the newest
  version the policy admits within it (137, 138): the store's listing of
  the versions a name holds is the store-naming unit's, beside the
  pointers it writes, and which listed versions fall within a range is
  the ecosystem's, which the npm and Cargo units answer. This unit's
  pinned answers the lockfile's pins alone, as S4 asks.
- Writing and moving the `artifact/<ecosystem>/<name>@<version>` pointer
  at publish, the store profiles carrying it, and keeping component tags
  out of `artifact/` (109, 138): the unit
  the-store-names-an-artifact-by-ecosystem-name-and-version. Until it
  lands, sync finds a pin only where a test or a store already holds the
  name.
- npm's and Cargo's pinned, render and serve, and the one serving process
  per host (114, 115, C.11): the units npm-rendered-and-served and
  cargos-registry. A served layout answering a refusal in its response to
  the client, and a sparse index leaving a refused version out of its
  listing and answering a request for its bytes with the refusal (99),
  and the render on a miss (50, 115) call this unit's render.
- The givens as a fragment the policy command shows, and the given
  acceptance and channels (C.10, 108): the gate and the channels. This
  unit gives the policy for a layout alone, in the policy file profile,
  resolving its gate from `[signers]` as the gate's unit resolves the
  given gate for acceptance.
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
  the consuming repository's own layout line, else, through the host's
  trusted signers, the publisher's, else the given, holding only
  satisfied versions, refusals
  answered with their claim, a layout that cannot be refreshed answering
  what it holds, and the host's record of each layout.

### Modified Capabilities

- `seam-contracts`: the projection seam gains pinned; the publisher's
  layout policy is a kind of claim; the policy for a layout says whether
  it is written or given.
- `seam-fakes`: the fake projection answers pinned from what the test
  sets.
- `policy-file`: a file with no `[layout]` answers the given policy for a
  layout, marked as given.
- `host-bindings`: the file names the signers the host trusts.
- `cache-entries`: filling the cache still reads no policy, now that sync
  also renders layouts that do.
- `command-line`: `tailrace sync` answers the layouts beside the cache
  entries.
- `publishing`: a publish attaches the publisher's written layout
  policy to the index.
- `conformance-suite`: the policy group's absence is asked of a channel
  no policy declares; the projection group holds pinned; the layouts
  group holds the core to 47, 50, 51, 61 to 63, 99 and 113, playing S4.
- `file-projection`, `image-projection`, `moon-cache-projection`: each
  answers pinned absent.

## Impact

- New core module `crates/tailrace-core/src/render.rs`; the projection
  trait and claim kinds in `tailrace-seams`; publish, verify's telling of
  a claim, and the history's per-checkout git directory in
  `tailrace-core`; `tailrace sync` in `crates/tailrace`.
- Every `Projection` implementation gains pinned: the fake, the file,
  image and moon cache profiles, and the conformance rigs.
- The policy seam's answer for a layout gains whether it is given, and
  every place that builds one: the fake policy, the policy file
  profile, the core's tests and the command's.
- The policy file profile's answer for a layout, and the bindings file's
  `[trust]` tables, carried on the binding to sync.
- The conformance register loses 47, 51, 99 and 113.
- The site's projections page says how a consumer reads under a policy.
- No new dependency.
- Rules kept: core.1 and core.2 (the core names no ecosystem; pins read
  by the projection, resolved by the core through the store seam),
  core.4 (every pin resolved to a digest and the digest recorded),
  core.7 (a claim counts only through a signer the policy names, and for
  the publisher's policy only through one the host trusts too), core.8
  and core.9 (the publisher's claim signed as the publishing host,
  naming the revision), core.17, core.18 (a layout path names no host),
  core.19 and core.20 (three answers kept apart; the record readable
  with no process and rebuilt), core.21 (a trusted signer is named by
  its public key, no credential), core.22 (the consumer's policy read
  at a landed revision), core.31 (one core answer rendered by the
  command), core.33.

## Touches

```
crates/tailrace-seams/src/projection.rs
crates/tailrace-seams/src/policy.rs
crates/tailrace-seams/src/claim.rs
crates/tailrace-seams/src/answer.rs
crates/tailrace-seams/src/lib.rs
crates/tailrace-core/src/render.rs
crates/tailrace-core/src/lib.rs
crates/tailrace-core/src/publish.rs
crates/tailrace-core/src/verify.rs
crates/tailrace-core/src/history.rs
crates/tailrace-core/tests/**
crates/tailrace-fakes/src/lib.rs
crates/tailrace-fakes/src/projection.rs
crates/tailrace-fakes/src/policy.rs
crates/tailrace-conformance/src/**
crates/tailrace-conformance/tests/**
crates/tailrace/src/lib.rs
crates/tailrace/src/bindings/**
crates/tailrace/src/commands/sync.rs
crates/tailrace/src/render.rs
crates/tailrace/src/profiles/policy/file/**
crates/tailrace/src/profiles/projection/file/mod.rs
crates/tailrace/src/profiles/projection/image/mod.rs
crates/tailrace/src/profiles/projection/moon_cache/mod.rs
crates/tailrace/src/profiles/projection/moon_cache/tests.rs
crates/tailrace/tests/**
site/src/content/docs/guides/integrations/projections.mdx
```
