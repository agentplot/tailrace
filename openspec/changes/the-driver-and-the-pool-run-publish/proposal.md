# Proposal

## Why

After a landing, a version should be published with no hook in sluice
and with nothing built again. Today `tailrace publish` asks the build to
make every output on the host it runs on, so a publish on a fresh GitHub
runner would build the outputs itself, which no verdict of the landing
vouches for. And a repository whose checks run on sluice's Actions pool
has no host that keeps what its checks built: each job's runner is gone
when the job ends, so nothing is left for `tailrace push` to send after
the landing. The statement settles both (135, 139; rulings of
`madswan-design`, agentplot/blueprints 8e649e5; C.1 at 2b8fc53): a
pool's check job pushes what its own run left before the landing is
decided, and a publish takes the outputs from the store's cache entries
that the landing's verdicts name, refusing when any is missing.

## What Changes

- **The build answers what a checkout's run left.** The build seam gains
  a third operation, `left`: given a checkout, the build identities whose
  outputs the checkout's own build state holds, each with its tasks, its
  outcome and where its files are, read from the build's own records and
  nothing of tailrace's (139). Through moon, that is moon's run report
  and the archives in its outputs directory (C.3). The fakes and the
  build's conformance group follow.
- **A check job pushes before the landing.** `tailrace push --left`
  writes the cache entry of each passing build identity the checkout it
  runs in left, asking no landing and no verdict, so a pool's run job
  pushes what its run made before the landing is decided (135, 139). Like
  the push after a landing it offers, signs and keeps nothing, and each
  build identity has its own answer. It binds with a `[store]` and needs
  no `[landing]` or `[signing]`.
- **Publish takes the outputs from the store.** Where the landing carries
  verdicts, a publish first fills the checkout's build cache from the
  store with the cache entries the passing verdicts name, so the build
  replays them rather than building, and refuses the version whole,
  naming the output and its build identity, when an output's entry is not
  in the store; nothing is claimed, packed or offered then (15, 16, 58,
  135). A landing with no verdicts (a plain branch) builds and offers
  what it left, as before. **BREAKING** for a host that publishes a
  sluice landing without pushing it first: it now runs `tailrace push`
  before `tailrace publish`.
- **The build's checkout of a revision shares the outputs directory** of
  the checkout tailrace runs in, as every checkout on a host shares it
  (C.3), so what the publish fills is what the build replays.
- **The self-hosting guide says which of two ways applies.** A
  repository that lands from the operator's own machines: whatever landed
  it, crew or a waiting `wt submit`, runs `tailrace push` and then
  `tailrace publish` once it learns of the landing, and the repository
  carries nothing for it (118, C.16). A repository whose checks run on
  sluice's Actions pool: the repository adds one step of its own to the
  pool's run job, after the queue's run step, that runs `tailrace push
  --left` with a credential that writes cache entries and nothing else in
  the store, and carries a workflow of its own that runs `tailrace
  publish` on each push to its landed branch. The guide shows both
  workflows to copy, and the credential's scope as an ECR policy on the
  cache entries' repositories alone, `cache/*`, which the operator makes
  before the first push (135, 96, C.1, core.14). There is no setup
  command: the guide shows the workflow and the repository copies it.

Out of scope, and what each waits on:

- Cache entries in repositories of their own, which the cache-only
  credential's scope rests on: the store-naming unit, which puts them
  under `cache/<build>` (`cache/moon`), apart from every component's
  repository (C.1). Until it lands the registry profile keeps every name
  in the one repository its binding names, so no key can be scoped to
  the cache alone; the guide shows the scope as C.1 has it and says so.
- Room for the repository's own steps in sluice's run job, and where that
  job leaves the checkout it ran the checks in: sluice's Actions pool
  (`actions-pool`, in flight in agentplot/sluice), which design is asking
  to keep that room when the template is regenerated. sluice's template
  names no tailrace.
- crew and worktrunk running push and publish after a landing they
  waited on: theirs, outside this repository (ADR 0005). This change says
  in the guide what they run.
- Signing a forge job's publish with the job's own identity, keyless
  (C.9) or a key in a key service (C.7): those signer profiles are later
  units. The guide's publish job signs with a host key held as a secret of
  a GitHub environment the job alone uses (C.8).
- Telling the gate after the publish, and acceptance: bolt 4's gate.
- The conformance suite's statement list adopting clauses 126 to 139:
  the commit that adopts the statement past agentplot/blueprints
  5b00e15. Until then no scenario can cite 135 or 139, so this change's
  scenarios cite 15, 16, 52 to 54, 58 and 74 to 76.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `seam-contracts`: the build's contract gains `left`.
- `seam-fakes`: the fake build answers what a checkout left, as a test
  scripts it.
- `moon-build`: `left` from moon's run report and outputs directory; the
  revision's checkout shares the outputs directory.
- `cache-push`: the push of what a checkout's run left, before a landing.
- `publishing`: where the landing carries verdicts, the outputs come from
  the store's cache entries, and a missing one refuses the version.
- `command-line`: `tailrace push --left`; publish renders into the
  checkout it runs in.
- `host-bindings`: `push --left` binds with no `[landing]`.
- `conformance-suite`: the build group holds `left`; the publish and push
  groups hold the store's entries and the push before a landing.

## Impact

- `crates/tailrace-seams/src/build.rs` (the trait and `Left`) and
  `answer.rs` (one refusal, an output whose entry the store does not
  hold).
- `crates/tailrace-core/src/publish.rs`, `push.rs`, `cache.rs`, `lib.rs`.
- `crates/tailrace/src/profiles/build/moon/` (left, the shared outputs
  directory), `commands/push.rs`, `commands/publish.rs`,
  `bindings/mod.rs` with its tests, `lib.rs`.
- `crates/tailrace-fakes/src/build.rs`; the conformance crate's build,
  publish and cache groups, its world, and the core and tools-layer tests
  that publish a landing with verdicts, which now push first.
- The site: the self-hosting guide, the builds and surfaces pages, and
  how it works.
- No crate, dependency, language or file type is added.

## Touches

```
crates/tailrace-seams/src/build.rs
crates/tailrace-seams/src/answer.rs
crates/tailrace-seams/src/lib.rs
crates/tailrace-core/src/publish.rs
crates/tailrace-core/src/push.rs
crates/tailrace-core/src/cache.rs
crates/tailrace-core/src/lib.rs
crates/tailrace-core/tests/**
crates/tailrace-fakes/src/build.rs
crates/tailrace-fakes/src/lib.rs
crates/tailrace-conformance/src/**
crates/tailrace-conformance/tests/**
crates/tailrace/src/profiles/build/**
crates/tailrace/src/commands/push.rs
crates/tailrace/src/commands/publish.rs
crates/tailrace/src/bindings/mod.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/src/lib.rs
crates/tailrace/src/main.rs
crates/tailrace/src/render.rs
crates/tailrace/tests/**
crates/tailrace/moon.yml
site/src/content/docs/guides/self-hosting.mdx
site/src/content/docs/guides/integrations/builds.mdx
site/src/content/docs/guides/integrations/surfaces.mdx
site/src/content/docs/how-it-works.mdx
```
