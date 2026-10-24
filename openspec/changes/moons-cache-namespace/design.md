# Design

## Context

This change builds on `the-projection-seam-carries-cache-entries`
(773ddfa), which comes before it and is not yet merged: pack is
`pack(output, at: &BuildId, version: Option<&Version>)`, a cache entry is
`CacheEntry { build, digest }`, render is `render(Contents, at)` answering
`Holds::Versions` or `Holds::BuildIds`, `no_cache_entries(source)` is the
one absent answer of a projection that packs none, and the fake
projection packs cache entries by default (its design D1 to D7). Its
open question, the order a render's build identities come in, is this
unit's: the moon cache answers them in the order given.

The store seam (B.1) has no operation for a cache entry and needs none:
put-bytes holds bytes by digest, name makes a name at a digest from
nothing and refuses naming where it is otherwise, resolve and read find
them again (66, 70 to 72). The layout store names any digest, bytes or
manifest. The registry store, in code beside this unit, refuses a name
whose digest holds bytes, since no tag names a blob (its design's
operation table); that is named in the proposal and below.

What moon 2.6.1, the version `.prototools` pins, does, read from runs in
scratch workspaces for this change:

- A task that passes with declared outputs leaves one archive,
  `.moon/cache/outputs/<task hash>.tar.gz`, holding the outputs and the
  task's `stdout.log` and `stderr.log` under `.moon/cache/states/`, each
  entry with the time and owner of the run.
- A fresh clone of the same revision, with that one archive copied into
  its own `.moon/cache/outputs/` and nothing else of the first
  checkout's cache, reports the task `cached` in `runReport.json` and
  restores the first run's outputs; the task hash does not depend on
  the checkout's path. A clone without it runs the task.
- The archive may be a symbolic link to a file elsewhere; moon reads
  through it and replays.
- moon keeps `.moon/cache/` per checkout. Its workspace setting
  `unstable_sharedWorktreeCache` (since 2.3.0) shares only the blobs and
  manifests of its experimental digest-directory outputs cache
  (`MOON_EXPERIMENT_CAS_OUTPUTS_CACHE`), not the archives. This
  repository's devenv shell shares the archives by linking each
  worktree's `.moon/cache/outputs` into the git common directory.

The moon build profile (`profiles/build/moon/`) runs a revision's tasks
in its own checkout under `<common>/tailrace/builds/<commit>`, so the
archives of a publish's build are in that checkout's outputs directory.

## Goals / Non-Goals

**Goals:**

- A cache entry written by one host is found by its build identity on
  any host that shares the store, and rendered where moon reads it.
- Nothing about writing or reading an entry offers, accepts, names a
  channel or index, or reads a policy (53, core.30).
- An entry that is gone is an answer, never a refusal (54).

**Non-Goals:**

- The command that writes a landing's entries after the landing, and
  the record of it (`the-push-after-a-landing`).
- Linking an archive to a blob the store holds on disk: the projection
  reaches the store only through the seam (core.2), so it writes the
  bytes read; a link is a store profile's to offer.
- Making the checkouts of a host share one outputs directory.
- Removing an entry: no store operation removes anything; an entry goes
  when the store's own retention or its operator removes its name.

## Decisions

### D1. An entry is the name `cache/<build identity>` at the bytes' digest

`cache.rs` names an entry `Name::new(format!("cache/{build}"))`, made
with `name(&cache_name, &digest, None)` after `put_bytes`. The name is
the build identity's alone, as 52 names an entry by its digest and build
identity and nothing else; no ecosystem is in it. A version's name is a
component's prefix and a version, which holds a dot, and a moon task
hash holds none, so no version's name is a cache name. A channel's name
is bolt 4's to choose; `cache.rs`'s module doc says that no other name
begins `cache/`.

Alternatives: a name per projection, `cache/<ecosystem>/<build>`, which
lets two projections pack one build identity apart but names an entry by
more than 52 does; an attestation from the writing host naming the build
identity, which needs the subject found first and so a name anyway; a
manifest for the entry, which needs a `Manifest` variant of the seams
and so a seam unit of its own.

### D2. write packs through the first projection that packs cache entries

```rust
pub struct Written {
    pub entry: CacheEntry,
    pub projection: Ecosystem,
    /// None when this write made the name; the digest the name was at
    /// when it was held already, the same or another host's bytes.
    pub held_at: Option<Digest>,
    pub bytes_held_already: bool,
}

pub fn write(seams: &Seams, output: &Output, build: &BuildId) -> Answer<Written>;
```

It asks each bound projection in ecosystem order to pack `output` at
`build` with no version, and takes the first answer that is not
`no_cache_entries` of that projection's source; with none, it answers
absent naming cache entries and writes nothing. An answer carrying
metadata is refused naming the projection, before anything is put (53).
It puts the bytes, then makes the name from nothing. A `NameAt { at:
Some(d) }` refusal is answered `held_at: Some(d)`, the name unmoved and
nothing retried (core.6); a `NameAt { at: None }` cannot come from a
make and is passed through. It attaches nothing: the entry's only trust
is the writing host's credential to the store (52). The push after a
landing calls it per output it pushes; this unit's tests and scenarios
call it directly.

### D3. sync renders each entry by itself

```rust
pub struct SyncAsk { pub branch: Branch, pub revision: Revision, pub at: String }

pub struct Synced {
    pub branch: Branch,
    pub revision: Revision,
    /// Each build identity a passing verdict names, once, in the order
    /// the verdicts name them, with the answer for it: the digest and
    /// the projections that rendered it, or why not.
    pub entries: Vec<(BuildId, Answer<Filled>)>,
}

pub struct Filled { pub digest: Digest, pub rendered: Vec<Ecosystem> }

pub fn sync(seams: &Seams, ask: &SyncAsk) -> Answer<Synced>;
```

`Landed::check` first, so a revision not landed is refused before the
store is asked (14, core.11). Then `verdicts_of`, keeping each verdict
whose outcome is a pass and its build identity once; a failed verdict's
build identity is not asked for, since moon archives no task that
failed. Per build identity, `resolve` the cache name: absent is that
entry's answer, absent naming the build identity's entry, and so is any
other answer resolve gives. Per entry resolved, `render` it alone,
`Contents::CacheEntries(&[entry])`, at `ask.at`, through every bound
projection, passing over each answering `no_cache_entries`; the entry's
answer is `Filled` with the projections that rendered it, or the first
refusal or unreadable answer a render gave. An entry that no projection
renders is absent naming cache entries. Rendering each entry alone means
one bad entry refuses only itself. Sync never asks the policy, the build,
the signers, the ledger or a target, and asks the store only to resolve;
the projection reads what it renders (D5).

Verdicts are read as the landing recorded them and are not verified:
they only choose which entries to fetch, and an entry carries no trust
beyond the writing host's (52). The moon cache's own checks (D5) keep a
verdict from placing a file outside moon's directory.

### D4. The newest landed revision the checkout descends from

`History::base(&self, branch: &Branch) -> Answer<Revision>`: the merge
base of the checkout's `HEAD` and the local branch, by git2's
`merge_base`; absent naming the branch when the repository holds no such
branch or the two share no history. The command defaults its revision to
it. A landing that leaves a merge commit on the branch (as sluice's do)
makes that commit the base a checkout branched from, and its verdicts
are the landing's. One revision is synced per asking: an older landed
revision's entries are reached by naming it.

### D5. The moon cache projection

`profiles/projection/moon_cache/mod.rs`, ecosystem `moon-cache`:

```rust
pub struct MoonCache { root: PathBuf, store: Box<dyn Store> }
impl MoonCache { pub fn new(root: impl Into<PathBuf>, store: Box<dyn Store>) -> MoonCache; }
```

`root` is the checkout it is bound in; the outputs directory of a
checkout is `<checkout>/.moon/cache/outputs`.

- **pack(output, build, None)**: the build identity checked as a task
  hash (non-empty, `[0-9a-f]` only) or refused `Why::Said` naming it;
  the bytes of `<root>/.moon/cache/outputs/<build>.tar.gz`; absent
  naming that path when it is not there, unreadable with the operating
  system's words otherwise; `meta: None`, `same_again: false`, since a
  rebuild of one task hash writes each file's time and owner again (87,
  C.11). `output` is not read: the archive holds the task's outputs as
  moon declared them.
- **pack(_, _, Some(version))** and **render(Contents::Artifacts(_), _)**:
  absent naming artifacts (`Not::absent(&source, "artifacts")`), reading
  nothing (53).
- **render(Contents::CacheEntries(entries), at)**: `at` is the checkout
  rendered into, absolute, or relative to `root`. Every build identity is
  checked first, and one that is no task hash refuses the render naming
  it before anything is read or written. The outputs directory is made
  with `create_dir_all`, which follows a link there. Per entry: an
  archive already there is left as it is; else `store.read(&digest)`,
  `Held::Bytes` whose `Digest::of` is the entry's digest, else refused
  `Why::OtherBytes`, and `Held::Manifest` refused `Why::Said` saying the
  entry names a manifest; written to a temporary file in the directory
  and renamed onto `<build>.tar.gz`, so moon never reads half an archive.
  The answer is `Rendered { layout: Layout { at: <the directory> },
  holds: Holds::BuildIds(<the given, in order>) }`. Any answer of the
  store is passed through as the render's.
- **serve**: absent, since moon reads its outputs directory before any
  remote (C.11).

It reaches the store only through the store seam (core.2: what it rules
out is a projection reaching the store any other way): the bindings
hand it a store bound from the host's `[store]` table, opened and never
made, a second binding of the table the seams' store is bound from.
Alternative: a forwarding wrapper sharing one store between the seams
and the projection. Rejected for one more type that buys nothing a
second open does not; `Seams.store` stays the one store a publish
writes (92).

### D6. The bound projections take the store and the checkout

`profiles::projection::bound(store: Box<dyn Store>, checkout: &Path)`
binds the file, image and moon cache projections; `bindings::bind`
passes `profiles::store::bind(store_table, false)?` and the repository
path, after the seams' own store is bound, so a publish has made the
layout first. The policy file's `[projections]` never names
`moon-cache` for an output, and a publish that tried would be answered
absent naming artifacts. `For` gains `Sync`, bound as `Verify` is.

### D7. `tailrace sync`

`commands/sync.rs`: `SyncArgs { revision: Option<String>, branch: String
= "main", json: bool }`. It opens the history at the working directory,
resolves the revision or takes `History::base`, and asks `cache::sync`
with `at` the history's working directory, absolute. Text: one line per
build identity, `<build> rendered by <projection>, <digest>`, `<build>
absent: <what>`, or the answer's rendering by `render::not`; with
`--json` the answer. Exit: the whole answer's code when it is no answer;
otherwise the worst of the entries' refused or unreadable answers, an
absent entry counting as an answer (54). `main.rs` loads the bindings
`For::Sync`.

### D8. The cache group over the fakes

Below the group's marker in `crates/tailrace-conformance/src/core/cache.rs`,
each over `FakeWorld`, with helpers in the module:

| name | plays | cites |
|---|---|---|
| written on one host, read on another | | 52 |
| an entry held already | | 52 |
| one store object for an artifact and an entry | | 100 |
| an entry is never an artifact | | 53 |
| an entry removed | | 54 |
| a removed entry builds again | S14 | 54 |

The second host is a second `FakeWorld::new()` whose `fakes.store` is a
clone of the first's (fakes share state when cloned), landed and
verdicted the same; the fakes' `FakeStore::forget(&Name)` removes a name
and leaves the bytes, as a store's retention removing a cache name
would. "An entry is never an artifact" checks the world's writes hold
one `Bytes` and one `Name` per entry and no `Manifest` or `Attach`, and
the policy fake was not asked. "A removed entry builds again" lands
`start` with passing verdicts on both outputs, writes both entries,
forgets the site's, lands `head` with verdicts naming `head`'s builds,
syncs `start` (site absent, server rendered, no refusal) and publishes
`head`, then reads every name the store held before the sync and finds
each at the digest it named. 52, 53, 54 and 100 leave the register; the
conformance crate's test listing the groups that hold no scenario
loses `cache`, and its module table names the `FakeWorld`.

### D9. The tools layer runs moon

`crates/tailrace/tests/moon_cache.rs`, behind the `tools` feature and
added to `cli:tools`'s `--test` list, on the scratch module's moon
workspace: a task with one output run by moon in a first checkout; the
moon cache bound there packs its archive; `cache::write` puts it in an
OCI layout store; a fresh clone renders it through `cache::sync` over a
fake landing whose verdict names the task hash, with the clone's outputs
directory first a plain directory and then a link to a directory
outside it; moon run in the clone reports the task `cached` and its
output holds the first checkout's bytes. The projection group's two
scenarios of a projection that packs cache entries run over the moon
cache in `tests/moon_cache_projection.rs`, which needs no moon and so is
in `cli:test`, check-fast, with a rig whose checkout holds an archive at
each of the hashes the group packs at (`b7e1`, `c3d4`) and whose layout
store holds their bytes, `caches()` answering true. The group's other
scenarios each pack at a version, which the moon cache answers absent
naming artifacts, so they do not run over it: a departure from core.33,
stated in the module doc as `tests/projections.rs` states its own, and
the unreadable answers of pack and render are its unit tests'.

## Risks / Trade-offs

- [A cache entry cannot be written to the registry store, whose name
  refuses a digest holding bytes] → The entries here are proven on the
  fakes and the layout store; the bolt's goal of a replay from ECR
  waits on the registry profile naming bytes (for example, under a
  manifest of its own that carries no artifact metadata) or a seam
  change, named for the design agent.
- [Two hosts write one build identity's entry with other bytes, moon's
  archives being unsteady] → The first name stands and the second is
  answered held at the first's digest; either archive replays the same
  outputs.
- [A sync in one checkout fills only that checkout where checkouts do
  not link their outputs directory] → The answer names the directory
  written; sharing it is the host's (proposal, out of scope).
- [The archive is written as a copy of the store's bytes, not a link]
  → One copy per entry per host; linking is a later store profile's.
- [The seam change is not merged when code starts] → The worktree is
  rebased onto the bolt once it merges (the place's note); the code
  stage starts after it.

## Open Questions

None that change the specs or tasks.
