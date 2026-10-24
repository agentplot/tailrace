# Tasks

Every cargo and moon command runs as `devenv shell -- <command>`. Code
starts once `the-projection-seam-carries-cache-entries` has merged into
the bolt and this worktree is rebased onto it: every task here codes
against its widened `pack`, `render`, `CacheEntry`, `Contents`, `Holds`
and `no_cache_entries`. Read `crates/tailrace-seams/src/projection.rs`
and `store.rs`, `crates/tailrace-core/src/landed.rs`, `history.rs` and
`publish.rs`, `crates/tailrace-fakes/src/store.rs` and `projection.rs`,
`crates/tailrace/src/profiles/projection/`, `bindings/mod.rs`,
`commands/verify.rs`, and `crates/tailrace/tests/scratch/mod.rs` before
starting. The change adds no crate, language, file type or dependency,
so no moon project or formatter entry; its tests run in the existing
`core`, `fakes`, `conformance` and `cli` projects' `test` tasks and in
`cli:tools`.

## 1. The core's cache

- [x] 1.1 Add `crates/tailrace-core/src/cache.rs` with the cache name, `write` and `Written` as design D1 and D2 have them, its module doc citing 52, 53, 100 and core.30 and saying no other name begins `cache/`, and add it to `lib.rs`'s module table and exports. In `crates/tailrace-core/tests/cache.rs`, over the fakes: a write puts the bytes and makes `cache/<build>` from nothing, with no put-artifact, attach or other name asked; a second write answers `held_at` the same digest and the store holds the bytes once; a write whose name is already at other bytes answers `held_at` that digest and the name is unmoved; with every projection scripted to pack none the answer is absent naming cache entries and nothing is written; a projection that answers metadata to a pack with no version is refused naming it before any put (spec `cache-entries`, written, held already, one store object). Verify `devenv shell -- moon run core:test` passes.
- [x] 1.2 Add `History::base` as design D4 has it, with tests in `crates/tailrace-core/tests/history.rs`'s style in `tests/cache.rs`: a checkout with commits of its own on a landed revision answers that revision; a branch the repository lacks is absent naming it (spec `cache-entries`, the default revision). Verify `devenv shell -- moon run core:test` passes.
- [x] 1.3 Add `sync`, `SyncAsk`, `Synced` and `Filled` to `cache.rs` as design D3 has them, every type serialized for `--json` (57). In `tests/cache.rs`: a revision not landed is refused `NotLanded` and the store is asked nothing; a failed verdict's build identity is never resolved; two passing verdicts naming one build identity resolve it once; an entry the store has no name for is absent naming its build identity and the others render; each entry is rendered alone, through every projection that packs cache entries, one scripted to pack none passed over; a render's refusal is that entry's answer alone; the policy fake is never asked and the store only resolves (spec `cache-entries`, sync, a gone entry, never an artifact). Verify `devenv shell -- moon run core:test` passes.

## 2. The cache over the fakes

- [x] 2.1 Add `FakeStore::forget(&Name)` in `crates/tailrace-fakes/src/store.rs`, removing the name and leaving the bytes, with a test in its module that a forgotten name resolves absent and its digest still reads. Verify `devenv shell -- moon run fakes:test` passes.
- [ ] 2.2 Add the six scenarios of design D8 below the marker in `crates/tailrace-conformance/src/core/cache.rs`, citing as its table says, "a removed entry builds again" playing S14; remove 52, 53, 54 and 100 from `register.rs`; in `src/lib.rs` take `cache` out of the groups that hold no scenario and give the cache's row of the module table the `FakeWorld`; run the group over the fakes in `tests/fakes.rs` as the core's other groups are (spec `conformance-suite`, the cache over the fakes). Verify `devenv shell -- moon run conformance:test conformance:coverage` passes, and the coverage output lists 55 as pending and none of 52, 53, 54 or 100.

## 3. moon's cache projection

- [ ] 3.1 Add `crates/tailrace/src/profiles/projection/moon_cache/mod.rs` as design D5 has it, its module doc naming C.11, 53, 84, 85, 87 and core.2, with unit tests over a temporary checkout and an OCI layout store: an archive packed is its bytes unchanged with no metadata and `same_again` false; a hash with no archive is absent naming the path; a pack at a version and a render of artifacts are absent naming artifacts and read nothing; two entries render as archives with the entries' bytes and the answer holds both build identities in the order given; an outputs directory that is a link to another directory is written through and left a link; an archive already there is not written over; bytes the store answers under another digest are refused `OtherBytes` and no archive is written; a build identity `../escape` refuses the render and nothing is written anywhere under the checkout; an outputs directory that is a file answers pack unreadable with the operating system's words (spec `moon-cache-projection`). Verify `devenv shell -- moon run cli:test` passes.
- [ ] 3.2 Make `profiles::projection::bound` take the store and the checkout and bind the moon cache under `moon-cache`, and have `bindings::bind` pass a second binding of `[store]`, opened and never made, and the repository, as design D6 has it, adding `For::Sync`; update `bindings/tests.rs` for the bound projections and add one that a publish's first binding makes the layout and the moon cache's store reads it. Verify `devenv shell -- moon run cli:test` passes.
- [ ] 3.3 Add `crates/tailrace/tests/moon_cache_projection.rs` running the projection group's two scenarios of a projection that packs cache entries over the moon cache, with the rig design D9 describes and the module doc stating the departure from core.33 (spec `moon-cache-projection`, the projection's contract). Verify `devenv shell -- moon run cli:test` passes with both scenarios reported passed.
- [ ] 3.4 Add `crates/tailrace/tests/moon_cache.rs` behind the `tools` feature, as design D9 has it, and add `--test moon_cache` to `cli:tools` in `crates/tailrace/moon.yml` (spec `moon-cache-projection`, moon replays a rendered entry; spec `cache-entries`, a task that passed on one host replays on another). Verify `devenv shell -- moon run cli:tools` passes, the clone's run report naming the task `cached` in both the plain and the linked directory.
- [ ] 3.5 Verify `grep -rniwE 'oci|ecr|ghcr|oras|cosign|moon|sluice|kms|npm|maven|cargo|flux|gh' crates/tailrace-core/src/cache.rs crates/tailrace-conformance/src/core/cache.rs` finds nothing (core.1).

## 4. The command and the site

- [ ] 4.1 Add `crates/tailrace/src/commands/sync.rs` as design D7 has it, register it in `commands/mod.rs`, `lib.rs` (the `Command` enum, `run`, and the module doc's list of commands) and `main.rs` (`For::Sync`). In `crates/tailrace/tests/cli.rs`: `tailrace sync --json` over the fakes for a landed revision with one of two entries gone prints one rendered and one absent and exits 0, and its JSON is the core's answer value for value; the built binary on a host with no bindings file answers absent naming the file, exits 4 and writes nothing (spec `command-line`). Verify `devenv shell -- moon run cli:test` passes.
- [ ] 4.2 In `site/src/content/docs/guides/integrations/projections.mdx`, mark the moon cache row built and say what pack and render do, that sync renders the entries of the newest landed revision the checkout descends from, and that one directory is shared only where the checkouts link it; in `builds.mdx`'s moon row, say a task's archive is a cache entry under its task hash that `tailrace sync` renders. Verify `devenv shell -- moon run ':#check-fast'` passes: format, lint, toolchain, ADR numbers, the suite's coverage, every crate's tests and the site's build.
