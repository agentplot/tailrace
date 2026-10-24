# Tasks

Every cargo and moon command runs as `devenv shell -- <command>`. Read
`crates/tailrace-seams/src/projection.rs`, `answer.rs` and `artifact.rs`,
`crates/tailrace-fakes/src/projection.rs` and `script.rs`,
`crates/tailrace-conformance/src/seams/projection.rs`, `src/scenario.rs`
(`not_applicable`) and `src/fakes.rs`'s projection rig, and
`crates/tailrace-core/src/publish.rs` steps 9 to 11 before starting.
The change adds no crate, language, file type or dependency, so no moon
project or formatter entry; its tests run in the existing `seams`,
`fakes`, `conformance`, `core` and `cli` projects' test tasks. The
crates do not compile between groups 1 and 4; run each group's tests
once the workspace builds again, at the latest in 5.1.

## 1. The seam

- [x] 1.1 In `crates/tailrace-seams/src/projection.rs`, widen the trait and its types as design D1 to D4 have it: `pack(output, at: &BuildId, version: Option<&Version>)`, `Packed.meta: Option<ArtifactMeta>`, `CacheEntry { build, digest }`, `Contents<'a>`, `render(contents, at)`, `Holds`, `Rendered.holds: Holds`, and `no_cache_entries(source)`, each with a doc comment citing its clause in the module's style. Extend its `every_type_round_trips_through_json` test with a `Packed` with `meta: None`, a `CacheEntry`, and a `Rendered` holding each kind; add a test that a `CacheEntry`'s JSON names exactly `build` and `digest` and is rejected when read as `ArtifactMeta` (spec `seam-contracts`, a cache entry is no artifact), and one that `no_cache_entries` is `Not::Absent` naming `cache entries`. Verify `devenv shell -- moon run seams:test` passes once 1.2 is done.
- [x] 1.2 Update `Nowhere` and the operations list in `crates/tailrace-seams/src/lib.rs`'s test to the widened signatures, pack asked once with `Some` version and render with `Contents::Artifacts`; the seam still names pack, render and serve and no other. Verify `devenv shell -- moon run seams:test` passes.

## 2. The fake projection

- [x] 2.1 Implement design D6 in `crates/tailrace-fakes/src/projection.rs`: `caches` in its state, `packs_no_cache_entries()`, pack and render of both cases, and the recorded `given`. Update its tests to the widened calls and add: one output packed twice at `b7e1` with no version gives equal bytes and `meta: None`, at `c3d4` other bytes, and none of the three byte strings contains the ecosystem's name or a version; two cache entries render as `Holds::BuildIds` of both; scripted to pack none, pack with no version and render of cache entries are `Not::Absent` and a pack with a version still answers metadata; a versioned pack's bytes are the same at two build identities (spec `seam-fakes`). Update the calls in `crates/tailrace-fakes/src/lib.rs`'s test. Verify `devenv shell -- moon run fakes:test` passes.

## 3. The suite

- [x] 3.1 In `crates/tailrace-conformance/src/seams/projection.rs`, add `fn caches(&self) -> bool` to `ProjectionRig`, change the scenarios above the marker only in their calls as design D7 says, and add the three scenarios of D7 below the marker with their functions after the module's last function. Change `src/seams/binding.rs`'s pack call the same way, reading the ecosystem from the metadata with an `expect` naming the projection. Update `Watched` in `src/world.rs` as design D8 says. Verify `devenv shell -- cargo build -p tailrace-conformance` passes.
- [x] 3.2 Give `FakeProjectionRig` in `src/fakes.rs` `caches()` and `packing_no_cache_entries()` (design D7), and run the projection group over both rigs in `tests/fakes.rs`, each report asserted and its `not_applicable` checked: the default rig's names only the scenario of a projection that packs none, the scripted rig's only the two cache-entry scenarios (spec `conformance-suite`, the fakes in both cases). Verify `devenv shell -- moon run conformance:test conformance:coverage` passes and the coverage output lists the same pending clauses as before the change.

## 4. The core and the profiles

- [x] 4.1 In `crates/tailrace-core/src/publish.rs`, pack each output with its report's build identity and `Some` version and refuse a pack answering no metadata, as design D5 has it, keeping steps 10 and 11 reading the metadata as before. Update `Watched` in `crates/tailrace-core/tests/world/mod.rs` to the widened signatures. Add a test in `crates/tailrace-core/tests/publish.rs` with a projection that answers `meta: None` to a versioned pack: publish is refused naming the output, and the fake store records no put-artifact. Verify `devenv shell -- moon run core:test` passes, every publish scenario among it.
- [x] 4.2 In `crates/tailrace/src/profiles/projection/file/mod.rs` and `image/mod.rs`, take the widened signatures (design D4): a pack with no version and a render of cache entries answer `no_cache_entries`, read nothing and write nothing; a versioned pack is unchanged. Update their tests' calls and add one per profile that a pack with no version is `Not::Absent` naming `cache entries` with the output's file removed (so nothing was read), and that render of a cache entry is the same absent. Update the modules' doc comments to say they pack no cache entries. Verify `devenv shell -- moon run cli:test` passes.
- [x] 4.3 In `crates/tailrace/tests/projections.rs`, give `FileRig` `caches()` answering false and add `"a projection that packs no cache entries"` to the scenarios run over the file and image projections, the report's `ran` count following. Verify `devenv shell -- moon run cli:test` passes with both reports naming that scenario as passed.
- [x] 4.4 Verify `grep -rn 'impl Projection for\|\.pack(\|\.render(' crates` finds no call or implementer left on the old signatures (design, risks), and `grep -rniwE 'oci|ecr|ghcr|oras|cosign|moon|sluice|kms|npm|maven|cargo|flux|gh' crates/tailrace-seams/src crates/tailrace-fakes/src crates/tailrace-conformance/src crates/tailrace-core/src` finds nothing this change added (core.1).

## 5. The site and the gate

- [ ] 5.1 In `site/src/content/docs/guides/integrations/projections.mdx`, change the contract's paragraph to say pack takes an output at a build identity and, where the bytes are an artifact's, the version; that a cache entry packs with no version and its bytes carry no metadata that makes an offer (100); that render takes artifacts, or cache entries by build identity, and answers which versions or build identities it holds; and that a projection that packs no cache entries answers absent for one, as the file and image projections do. Verify `devenv shell -- moon run ':#check-fast'` passes: format, lint, toolchain, ADR numbers, the suite's coverage, every crate's tests and the site's build.
