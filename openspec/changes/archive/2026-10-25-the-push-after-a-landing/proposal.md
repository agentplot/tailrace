# Proposal

## Why

Bolt 3 is usable when a task that passed on one host replays from the
store on another. The unit before this one, `moons-cache-namespace`,
writes a cache entry and fills a checkout's cache from the landing's
verdicts with `tailrace sync`, but nothing writes the entries a landing
leaves: the archives moon wrote on the host that ran the landing's
checks stay on that host. This unit adds the step after a landing that
pushes them, run on the landing host as the next step after sluice's
push or by hand (C.3), so a landing on one host followed by `tailrace
sync` on another replays every task the landing ran. It also makes what
tailrace keeps rebuildable: everything kept but the record of claimed
versions can be deleted and rebuilt from the store and git to the same
state (63, S12).

## What Changes

- **One command pushes a landing's archives.** `tailrace push [revision]
  [--branch main] [--json]`, on a host whose bindings file names a store
  and a landing, asks the landing whether the branch landed the
  revision, by default the local branch's head, which on the landing
  host is sluice's last landing. It reads the landing's verdicts and, for
  each build identity a passing verdict names, writes its cache entry
  through `moons-cache-namespace`'s core function, which packs the
  archive moon left for it and makes the entry's name in the store (52,
  77, 78, C.3). A revision the branch has not landed is refused before
  anything else is read.
- **Each build identity has its own answer.** An entry written, one the
  store held already (by an earlier push or another host), one whose
  archive is not on this host, which is absent and refuses nothing (54),
  or the refusal or unreadable answer its write gave. A second push of
  the same landing writes nothing.
- **The push offers nothing and keeps nothing.** It puts no manifest,
  attaches no claim, signs nothing and moves no name but each entry's
  own (53, 100). It adds nothing to what the host keeps: which entries a
  landing's archives became is what the landing's verdicts and the store
  answer, and tailrace keeps nothing the store or git holds already (61).
- **Whoever drives the landing runs the push.** sluice runs nothing after
  a landing and the core starts nothing (ADR 0005): the push is a
  command a driver runs on the landing host once sluice reports the
  landing, or a person runs by hand. Neither this unit nor sluice adds a
  hook.
- **The host's record is rebuilt from the store and git.** `tailrace
  rebuild [revision] [--branch main] [--json]` rewrites the host's record
  of published versions from what the store and git hold: for each
  component the policy file declares at a landed revision, each tag
  carrying its prefix that names a revision the branch landed, and whose
  name the store resolves to that component's index at that version,
  one entry, the same file a publish writes (61 to 63). A version whose
  tag was claimed but whose index the store never named gets no entry,
  as a publish that stopped before its index keeps none. A rebuild
  writes only the record, never the store, and never deletes or rewrites
  an entry it finds.
- **The record's conformance group** gains S12 and its scenarios over
  the fakes, citing 63, which leaves the register; the cache's group
  gains the push's scenarios.
- **The site** says how the push runs after a landing and that `tailrace
  rebuild` rebuilds the record, and the command line's row names both.

Out of scope, and what each waits on:

- Proving the push against a registry, and so the replay from ECR the
  bolt's goal names: the push needs nothing of its own there. A
  registry names a cache entry through a manifest of tailrace's
  cache-entry type, whose one layer is the entry's bytes, and a tag at
  that manifest resolves to the layer's digest, so `cache/<build
  identity>` is at the bytes' digest on a registry as on a layout (C.1,
  ADR 0014). `the-registry-store` builds that manifest and proves it
  against zot and ECR; this unit proves the push over the fakes and the
  OCI layout.
- Wiring a driver to run the push after every landing in a given
  repository (a watcher of sluice's log, a forge's push event, a plan's
  step): the host operator's, outside tailrace (ADR 0005).
- Where moon's archives are on the landing host: the push reads moon's
  outputs directory in the checkout it runs in, which holds the
  landing's archives where the host's checkouts share it, as this
  repository's devenv shell links it (C.3; `moons-cache-namespace`'s
  out-of-scope). Sharing it is not this unit's.
- The record of rendered layouts (61): bolt 5's unit `render under a
  policy`, which keeps it and makes it rebuildable.
- Status across machines, and status reading the store: bolt 4's
  `status across machines`; status here stays a read of the record
  alone (60).

## Capabilities

### New Capabilities

- `cache-push`: the step after a landing that writes the cache entries
  of every build identity the landing's verdicts passed, from the
  archives the landing's build left on the host, offering nothing and
  keeping nothing.

### Modified Capabilities

- `host-record`: the record is rebuilt from the store and git to the
  same state.
- `command-line`: `tailrace push` and `tailrace rebuild` are commands of
  the one core.
- `host-bindings`: push, like publish, makes the layout the file names;
  rebuild, like verify, never does.
- `conformance-suite`: S12 and the push over the fakes.

## Impact

- `crates/tailrace-core/src/push.rs` (new), `record.rs` (new) and
  `lib.rs`'s module table.
- `crates/tailrace/src/commands/push.rs` and `rebuild.rs` (new),
  `commands/mod.rs`, `lib.rs`, `main.rs`, and `bindings/mod.rs` with its
  tests, for `For::Push` and `For::Rebuild`.
- `crates/tailrace-conformance/src/core/record.rs` and `core/cache.rs`,
  below their markers; `register.rs`; `lib.rs`'s list of groups that
  hold no scenario.
- `crates/tailrace/tests/push_after_landing.rs` (new, tools layer), the
  scratch gate's toolchain, and `crates/tailrace/moon.yml`'s `tools`
  task.
- Four site pages.
- No crate, dependency, language or file type is added.

## Touches

```
crates/tailrace-core/src/push.rs
crates/tailrace-core/src/record.rs
crates/tailrace-core/src/lib.rs
crates/tailrace-core/tests/push.rs
crates/tailrace-core/tests/record.rs
crates/tailrace/src/commands/push.rs
crates/tailrace/src/commands/rebuild.rs
crates/tailrace/src/commands/mod.rs
crates/tailrace/src/lib.rs
crates/tailrace/src/main.rs
crates/tailrace/src/bindings/mod.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/tests/cli.rs
crates/tailrace/tests/push_after_landing.rs
crates/tailrace/tests/scratch/**
crates/tailrace/moon.yml
crates/tailrace-conformance/src/core/record.rs
crates/tailrace-conformance/src/core/cache.rs
crates/tailrace-conformance/src/register.rs
crates/tailrace-conformance/src/lib.rs
crates/tailrace-conformance/src/world.rs
crates/tailrace-conformance/tests/fakes.rs
crates/tailrace-fakes/src/lib.rs
crates/tailrace-fakes/src/projection.rs
site/src/content/docs/guides/integrations/builds.mdx
site/src/content/docs/guides/integrations/surfaces.mdx
site/src/content/docs/guides/self-hosting.mdx
site/src/content/docs/how-it-works.mdx
```
