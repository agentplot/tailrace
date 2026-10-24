# Design

## Context

This change builds on two before it. `the-projection-seam-carries-cache-entries`
has merged into the bolt (3b6d0f4): pack is `pack(output, at: &BuildId,
version: Option<&Version>)`, a cache entry is `CacheEntry { build,
digest }`, and `no_cache_entries(source)` is the absent answer of a
projection that packs none. `moons-cache-namespace` (1e0ca8f, not yet
merged) adds `crates/tailrace-core/src/cache.rs` with the cache name
`cache/<build identity>`, `cache::write(seams, output, build) ->
Answer<Written>` (its design D2), `cache::sync` and `History::base`;
the moon cache projection, whose pack reads
`<checkout>/.moon/cache/outputs/<hash>.tar.gz` and ignores the output it
is given (its D5); the bound projections taking the store and the
checkout, and `For::Sync` (its D6); and `tailrace sync` (its D7). Every
name of theirs used below is as their design has it; the code stage
starts once this worktree is rebased onto the bolt with both merged.

The host's record is `published.rs`: `Kept::at(history)` under
`<common dir>/tailrace/published/<component>/<tag>.json`, one `Entry {
component, version, tag, revision, branch, index, store }` per version,
written by publish after the version's name moves to its index, read by
status alone. `Kept::write` leaves an entry with the same content
untouched (`Written::Held`) and refuses one with other content
(`Why::KeptOther`). A publish names its index in the store by the tag's
name (`Name::new(&versioned.name)`), and the entry's revision is the
tag's.

What moon 2.6.1 and sluice do, read for this change:

- moon archives every task that passes, with declared outputs or none:
  a task with no outputs leaves `<hash>.tar.gz` holding its
  `stdout.log` and `stderr.log`, and a fresh clone given only that
  archive reports the task `cached` (run in a scratch workspace). moon
  in CI mode (`--ci`, as sluice runs `moon exec`) archives the same.
- sluice runs a candidate's checks in a place of its own and releases
  the place once the verdict is read (sluice 25). `[tests] toolchain` is
  a command sluice runs in the place before the run, outside the
  executor, and what it prints is sourced before every command (sluice
  `profiles/checks.rs`). This repository's devenv shell, sourced so,
  links each checkout's `.moon/cache/outputs` to
  `<common dir>/moon-outputs`, so the archives of a landing's checks
  outlive its place and every checkout on the host reads them.
- On the host that advances the branch, sluice sets the host's
  `refs/heads/<branch>` to each landing after the push (sluice C.6
  pushed refs), so the local branch's head there is the last landing.

## Goals / Non-Goals

**Goals:**

- One command, run on the landing host after a landing, makes every
  task the landing passed replayable on any host that shares the store.
- The record of published versions survives its deletion: a rebuild
  from the store and git gives it back byte for byte (S12).

**Non-Goals:**

- A record of pushes. 61 lists what tailrace keeps apart from the
  store, and nothing in it is a push; the landing's verdicts and the
  store's cache names answer what a push did.
- Building anything: the push packs what the landing's build left and
  never asks the build seam to ensure (53, 54); a task whose archive is
  gone builds again where it is next needed.
- Moving the record to another host, or merging two hosts' records.

## Decisions

### D1. The push writes each passing build identity through `cache::write`

`crates/tailrace-core/src/push.rs`:

```rust
pub struct PushAsk { pub branch: Branch, pub revision: Revision }

pub struct Push {
    pub branch: Branch,
    pub revision: Revision,
    /// Each build identity a passing verdict names, once, in the order
    /// the verdicts name them, with what its write answered.
    pub entries: Vec<(BuildId, Answer<cache::Written>)>,
}

pub fn push(seams: &Seams, ask: &PushAsk) -> Answer<Push>;
```

`Landed::check` first, so a revision not landed is refused `NotLanded`
before the store or a projection is asked (77). Then `verdicts_of`,
whose failure is the whole answer; each verdict whose outcome is a pass
gives its build identity once. Per build identity, `cache::write(seams,
&Output { name: OutputName::new(verdict.check.as_str()), files: vec![] },
&build)`: a landing's verdicts name checks, not the outputs the policy
file declares, so the output is named for the first passing verdict's
check and carries no files; the moon cache reads only the build
identity. Whatever `write` answers is that build identity's answer:
`Written` with `held_at: None` for a name this push made, `Some(d)` for
one held already (core.6), absent where the projection found no archive
(54), and any refusal or unreadable answer as given. One build
identity's answer never stops the next. A failed verdict's build
identity is not written, since moon archives no task that failed and an
entry carries no outcome.

The push reads no policy and no record, and writes nothing but what
`write` puts: bytes and a name per entry (53, 100). It takes no
`History`: the revision comes resolved from the command, and nothing
else of git is read.

Alternative: push the archives of every output the policy file
declares, by the build seam's `report`. Rejected: a landing's checks are
tasks the gate requires, most with no output the authors publish, and
"every task the landing ran" replays only if each of those is pushed;
`report` also reads the build's own checkout under
`<common>/tailrace/builds/`, where the landing's archives are not.

### D2. The push keeps nothing

No file of the host's record is written by a push, and no other file is
kept (61). A second push of a landing finds each name held and answers
it so, which is what a record of pushes would have told it. The proposal
says so, and the cache group's scenario holds it.

### D3. Rebuild: the record from the store and git

`crates/tailrace-core/src/record.rs`:

```rust
pub struct RebuildAsk { pub branch: Branch, pub revision: Revision, pub store: Source }

pub struct Rebuilt {
    pub record: String,
    /// Each entry the store and git give, in component order and by the
    /// version rules, with what writing it answered.
    pub entries: Vec<(Entry, Answer<Written>)>,
}

pub fn rebuild(seams: &Seams, history: &History, kept: &Kept, ask: &RebuildAsk) -> Answer<Rebuilt>;
```

In order, every read before any write:

1. `Landed::check(branch, revision)`, refused before anything else.
2. `Declared::read(history, &landed)` for the components.
3. Per component, `history.tags(&landed, &component.prefix)`: the
   record of claimed versions (22), local and shared.
4. Per tag, `seams.landing.landed(branch, &tag.revision)`; a tag the
   branch did not land is passed over, since a publish from that branch
   never made it.
5. `seams.store.resolve(&Name::new(&tag.name))`: absent passes the tag
   over (a claim whose publish stopped before its index); a digest is
   read, and must be `Held::Manifest(Manifest::Index(meta))` with
   `meta.component()` the component and `meta.version()` the tag's
   version, else unreadable naming the tag and the digest (core.4,
   core.19).
6. The entry: `Entry { component, version: Version::new(tag.version
   .to_string()), tag: tag.name, revision: tag.revision, branch:
   ask.branch, index, store: ask.store }`.

Any answer other than an absence at steps 1 to 5 is the whole answer,
and nothing is written. Then each entry is written with `Kept::write`,
whose answer is that entry's: `Wrote`, `Held` for the same content, or
refused `KeptOther` naming the file, which stays as it is. Nothing is
deleted. The store is asked only to resolve and read; the signer, the
build, the policy and every projection are asked nothing.

An entry rebuilt is the entry a publish wrote: the same fields, the same
serializer, so the same bytes. The store is named as the bindings name
it, as publish names it (61, 118).

What the record holds after a rebuild is a function of the store and
git alone, as 63 requires: every version the bound store holds an index
for under a tag the branch landed. On a host that is the store's only
publisher this is exactly what it kept; where several hosts publish to
one store, it is also what the others published, since the store keeps
no word of which host offered an index. That is named for the design
agent.

Alternative: rebuild from the provenance claims attached to each
artifact, which name the revision and the signer. Rejected: the tag
already names the revision, and reading claims to filter by signer would
make the record hold "this host's" versions, which the next rebuild
after another host's publish could not give back the same.

### D4. The commands

`commands/push.rs`: `PushArgs { revision: Option<String>, branch: String
= "main", json: bool }`. The revision is resolved with
`History::resolve`, or by default `History::head(&branch)`, the last
landing on the landing host; then `push(&binding.seams, &ask)`. Text:
one line per build identity, `<build> pushed, <digest>`, `<build> held
already at <digest>`, `<build> absent: <what>`, or `render::not`'s
lines. Exit: the whole answer's code when it is no answer; otherwise
`render::worst` over the entries, an absent entry counting as an answer
(54), as `tailrace sync` exits.

`commands/rebuild.rs`: `RebuildArgs { revision: Option<String>, branch:
String = "main", json: bool }`, the revision defaulted the same way;
`rebuild(&binding.seams, &history, &Kept::at(&history), &ask)` with
`store: binding.store`. Text: one line per entry, `<component>
<version> (<tag>) from <revision>: written | held`, or the refusal's
lines. Exit as push's, every entry's code counting.

`lib.rs` gains `Command::Push` and `Command::Rebuild` and the module
doc's list of commands; `main.rs` loads `For::Push` and `For::Rebuild`.
In `bindings::bind`, `For::Push` makes the layout as `For::Publish` does
(`profiles::store::bind(store, matches!(command, For::Publish |
For::Push))`) and needs no `[signing]`; `For::Rebuild` is bound as
`For::Verify` is.

Rebuild is a command of its own because status must reach no store and
write nothing (60, core.32), and a person who deleted the record needs
one act to have it back. Alternative: the push rebuilding the record
after every landing. Rejected: it would make every push read the whole
history and the store, and give the landing host a record of versions
it may never have published.

### D5. The conformance scenarios

The cache group, after `moons-cache-namespace`'s scenarios, over
`FakeWorld` and its second world sharing the store (its D8):

| name | cites |
|---|---|
| a landing's archives pushed and synced on another host | 52 |
| a push of a revision not landed | 77 |
| a push repeated writes nothing | 52 |
| an archive not on the pushing host | 54 |
| a push offers and keeps nothing | 53 |

"An archive not on the pushing host" scripts the fake projection's pack
absent for one build identity. "A push offers and keeps nothing" checks
`world.writes()` holds one `Bytes` and one `Name` per build identity and
no `Manifest` or `Attach`, the signer and policy fakes record no call,
and `world.record()` is unchanged.

The record group, below its marker:

| name | plays | cites |
|---|---|---|
| everything kept deleted and rebuilt | S12 | 63 |
| a claimed version with no index | | 63 |
| a rebuild writes only the record | | 63 |
| a rebuild that cannot read the store | | 58, 63 |

S12 runs over `FakeWorld::two_components()`: publish both components'
versions, take `world.record()` and `status`, remove the record's
directory, rebuild, and compare the record's paths and bytes and the
status answer. "A claimed version with no index" forgets the version's
name in the fake store (`FakeStore::forget`, `moons-cache-namespace`'s
task 2.1). 63 leaves the register; the record group leaves the list of
groups holding no scenario, and its row of the module table names the
`FakeWorld`.

### D6. The tools layer: a landing on one host, a sync on another

`crates/tailrace/tests/push_after_landing.rs`, behind `tools`, added to
`cli:tools`'s `--test` list. Over the scratch repository with a bare
remote (`scratch::Scratch`):

1. The moon workspace declares two tasks the branch requires, one with
   a declared output and one with none.
2. The scratch gate's `[tests] toolchain` is a command that links the
   place's `.moon/cache/outputs` to `<common dir>/moon-outputs`, as this
   repository's devenv shell does; `Gate::new` gains a toolchain
   argument, or a second constructor, and existing callers keep theirs.
   The first checkout links its own the same way.
3. sluice lands a submission (`Gate::land`).
4. Host A, the first checkout, runs the built binary `tailrace push
   --json` under a bindings file naming a layout, sluice with remote
   `origin`, and no `[signing]`: both build identities written.
5. Host B, a second clone of the remote with its own common directory
   and a bindings file of its own naming the same layout and sluice,
   runs `tailrace sync <revision> --json`: both rendered.
6. moon in host B runs both tasks; its run report names both `cached`,
   and the output holds host A's bytes.

The hosts are two homes and two configuration directories in the
scratch directory, as `laptop.rs` keeps its host.

## Risks / Trade-offs

- [The landing host keeps archives per checkout and sluice has released
  the place] → The push answers each build identity absent naming where
  it looked, refusing nothing; the text says so, and the site says the
  host's checkouts must share moon's outputs directory, as this
  repository's devenv shell makes them.
- [A replayed verdict's archive is on another host] → It is answered
  absent here, or held already where that host pushed it; a sync then
  finds it or builds again (54).
- [Several hosts publish to one store] → A rebuilt record holds their
  versions too (D3), named for the design agent; status still reaches no
  store.
- [The policy file at the asked revision no longer declares a component
  it once did] → That component's versions are not rebuilt; asking a
  rebuild at an older landed revision reaches them.
- [Cache entries on a registry] → Unchanged from `moons-cache-namespace`:
  the registry store cannot name bytes yet, so the push is proven on the
  layout; on a registry, every write answers that refusal per build
  identity.

## Open Questions

None that change the specs or tasks.
