# Proposal

## Why

Bolt 3 is usable when a task that passed on one host replays from the
store on another, and a fresh checkout's first build of a landed
revision runs nothing the landing already ran. The projection seam can
now carry a cache entry (the unit `the-projection-seam-carries-cache-entries`,
84, 85, 87 as amended), but nothing writes one, reads one, or puts one
where moon looks: tailrace has no cache entries, no moon cache
projection, and no `tailrace sync`. This unit builds the cache
namespace: a cache entry in the store under its build identity, never an
artifact (52 to 54, 100, core.30), and moon's cache projection that packs
a task's output archive by its task hash and renders moon's outputs
directory from the entries the landing's verdicts name (C.3, C.11), so
S14 plays.

## What Changes

- **A cache entry is held in the store under its build identity.** The
  core's cache module writes an entry by packing an output at its build
  identity with no version through a projection that packs cache
  entries, putting the bytes, and making the name `cache/<build
  identity>` at their digest from nothing. It puts no manifest, attaches
  no claim and moves no other name, so writing an entry offers nothing
  (52, 53, 100). An entry already held for that build identity, at the
  same bytes or at other bytes another host packed, is answered as held,
  naming its digest, and nothing is retried (core.6).
- **The same bytes are one store object.** An entry whose bytes are
  already held, as an artifact's layer or as another entry, is put once
  and answered held already; the artifact is untouched (100).
- **Sync fills a checkout's cache from the landing's verdicts.** Given
  a landed revision on a branch, by default the newest one the checkout
  descends from, sync reads the build identity of every verdict that
  passed, resolves each one's entry in the store, and renders the
  entries it finds through every bound projection that packs cache
  entries, into the checkout. An entry that is gone is named in the
  answer as absent and refuses nothing: the build that needs it builds
  again (54, core.30). A revision the branch has not landed is refused
  before anything is read (14, rule core.11).
- **No channel, index or policy names a cache entry.** Sync reads no
  policy and writes nothing to the store, and nothing a publish, verify
  or status reads names a cache name (53, core.30).
- **moon's cache projection** (`moon-cache`, C.11): its pack reads the
  archive moon left for a task hash in the checkout's moon outputs
  directory, `.moon/cache/outputs/<hash>.tar.gz`, and answers its bytes
  with no metadata; it says it cannot promise the same bytes again,
  since moon's archives carry each file's time and owner (87). Its
  render writes each entry's bytes, read through the store seam and
  checked against the entry's digest, to `<hash>.tar.gz` in the
  checkout's moon outputs directory, following a link there, and
  answers the task hashes the directory holds. A task hash that is not
  one is refused naming it, so no verdict can write outside the
  directory. It packs no artifact: a pack given a version answers
  absent (53).
- **`tailrace sync`** on the command line: `tailrace sync [revision]
  [--branch main] [--json]`, bound by the host's bindings file and
  writing nothing to the store, answering each build identity as
  rendered or gone, or with the refusal its render gave.
- **The cache's conformance group** gains its scenarios over the fakes,
  S14 among them, citing 52, 53, 54 and 100, and those four clauses
  leave the register.
- **The site** says moon's cache is built, and how `tailrace sync`
  fills it.

Out of scope, and what each waits on:

- The step after a landing that pushes that landing's archives, its
  command, and what tailrace keeps of it (61 to 63): the unit
  `the-push-after-a-landing`, which comes after this one and writes
  entries through this unit's core function.
- Cache entries in an OCI registry: the registry store's name refuses a
  digest that holds bytes rather than a manifest, so on a registry a
  cache entry cannot be named until that is settled (named for the
  design agent). The entries here are proven over the fakes and the OCI
  layout store.
- Sharing moon's outputs directory between the checkouts of a host:
  moon 2.6.1 keeps it per checkout; where a host links each checkout's
  directory to one (as this repository's devenv shell does), one sync
  fills them all. Making that link is not this unit's.
- moon's experimental digest-directory outputs cache (C.3, "where moon
  keeps outputs as a digest directory instead"): moon 2.6.1 keeps it
  behind an experiment flag; a later unit.
- A verifier's rebuild reading no entry it did not write (55): phase 2.
- Consumer layouts of artifacts under a policy, and the general
  `tailrace sync` of bolt 5: the unit `render under a policy`.

## Capabilities

### New Capabilities

- `cache-entries`: a cache entry written to the store under its build
  identity and read back by any host, never an artifact, removable at
  any time, and a checkout's cache filled from the landing's verdicts.
- `moon-cache-projection`: the projection that packs a moon task's
  output archive by its task hash and renders moon's outputs directory
  from cache entries.

### Modified Capabilities

- `command-line`: `tailrace sync` is a command of the one core.
- `conformance-suite`: the cache's group holds the core to 52 to 54 and
  100 over the fakes, S14 among its scenarios.

## Impact

- `crates/tailrace-core/src/cache.rs` (new), `lib.rs`'s module table,
  and `history.rs` for the newest landed revision a checkout descends
  from.
- `crates/tailrace/src/profiles/projection/moon_cache/` (new), and
  `profiles/projection/mod.rs`, whose bound projections take the store
  and the checkout.
- `crates/tailrace/src/bindings/mod.rs`, which binds `[store]` once
  more for the projections, and its tests.
- `crates/tailrace/src/commands/sync.rs` (new), `commands/mod.rs`,
  `lib.rs` and `main.rs`.
- `crates/tailrace-conformance/src/core/cache.rs`, `register.rs`, and
  `lib.rs`'s list of the groups that hold no scenario yet;
  `crates/tailrace-fakes/src/store.rs` gains forgetting a name.
- `crates/tailrace/tests/moon_cache.rs` (new, tools layer) and
  `crates/tailrace/moon.yml`'s `tools` task, which runs it;
  `crates/tailrace/tests/moon_cache_projection.rs` (new, check-fast),
  the projection group over the moon cache.
- `site/src/content/docs/guides/integrations/projections.mdx` and
  `site/src/content/docs/guides/integrations/builds.mdx`.
- No crate, dependency, language or file type is added.

## Touches

```
crates/tailrace-core/src/cache.rs
crates/tailrace-core/src/lib.rs
crates/tailrace-core/src/history.rs
crates/tailrace-core/tests/cache.rs
crates/tailrace/src/profiles/projection/moon_cache/**
crates/tailrace/src/profiles/projection/mod.rs
crates/tailrace/src/bindings/mod.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/src/commands/sync.rs
crates/tailrace/src/commands/mod.rs
crates/tailrace/src/lib.rs
crates/tailrace/src/main.rs
crates/tailrace/tests/moon_cache.rs
crates/tailrace/tests/moon_cache_projection.rs
crates/tailrace/tests/cli.rs
crates/tailrace/moon.yml
crates/tailrace-conformance/src/core/cache.rs
crates/tailrace-conformance/src/register.rs
crates/tailrace-conformance/src/lib.rs
crates/tailrace-conformance/tests/fakes.rs
crates/tailrace-fakes/src/store.rs
site/src/content/docs/guides/integrations/projections.mdx
site/src/content/docs/guides/integrations/builds.mdx
```
