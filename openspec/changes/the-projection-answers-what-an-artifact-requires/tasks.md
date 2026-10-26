Run every command inside `devenv shell -- <command>`. Adding two methods
to the `Projection` trait breaks every implementor at once, so group 1
leaves the whole workspace compiling. Each group is verified by the moon
tasks it names, all of which `check-fast` runs.

## 1. The seam and every implementation

- [x] 1.1 In `crates/tailrace-seams/src/projection.rs`, make these changes and export the new items from `lib.rs`:
  - add the `Range` text type and `Requirement { name: EcosystemName, range: Range }`, deriving what `Pinned` derives;
  - add `Projection::requires(&self, held: &Artifact) -> Answer<Vec<Requirement>>` and `Projection::satisfying(&self, range: &Range, versions: &[Version]) -> Answer<Vec<Version>>`, each with a doc comment citing 140, saying an ecosystem with no manifest answers none (an empty list, never absent), and saying satisfying answers newest first by the ecosystem's own ordering (23; design D1 to D4);
  - change the module doc and the seam table in `lib.rs` to cite 140.

  Test: a requirement naming `@scope/pkg` with `>=1.2.0-rc.1 <2` round-trips through JSON as spelled. Verify with `moon run seams:test`.
- [x] 1.2 In `Nowhere` in `crates/tailrace-seams/src/lib.rs`, answer both operations with `nothing(Seam::Projection, "requires")` and `"satisfying"` (design D7). Verify with `moon run seams:test`.
- [x] 1.3 In the file, image and moon cache projections under `crates/tailrace/src/profiles/projection/`, answer both operations with `Ok(Vec::new())`, reading nothing. Each gets a doc comment saying its ecosystem has no manifest (140), each module's top doc comment gets a sentence saying so, and each module gets a unit test that both answers are empty (the file-, image- and moon-cache-projection specs; design D3). Verify with `moon run cli:test`.
- [x] 1.4 In `crates/tailrace-fakes/src/projection.rs` (seam-fakes spec; design D5), first add `semver.workspace = true` to `crates/tailrace-fakes/Cargo.toml`. Then:
  - add `manifests` to the state, with `FakeProjection::manifest(&Digest, &[Requirement])`;
  - add `ProjectionOp::Requires` and `ProjectionOp::Satisfying`;
  - requires answers what was set for the digest, else absent naming `the manifest of <name>@<version>`;
  - satisfying answers by `semver`: an unreadable range is refused with `Why::Said` carrying the parser's words, unreadable versions are dropped, and the rest are kept where the range matches and sorted newest first;
  - update the module's doc comment.

  Tests:
  - requirements set and an unset artifact answered absent;
  - `^1.2` over `1.9.0, 2.0.0, 1.10.0, 1.2.0, 1.1.9` answers `1.10.0, 1.9.0, 1.2.0`;
  - an unreadable range refused with the parser's words;
  - an unreadable version left out;
  - both ops added to `every_operation_can_be_faulted_and_is_recorded`, with their `given` asserted.

  Verify with `moon run fakes:test`.
- [x] 1.5 Give the test projections both operations (design D7):
  - pass through to the inner fake in `Watched` in `crates/tailrace-core/tests/world/mod.rs` and in `crates/tailrace-conformance/src/world.rs`;
  - answer none in `Offering` (`crates/tailrace-core/tests/cache.rs`), `Metaless` (`crates/tailrace-core/tests/publish.rs`) and `Carrying` (`crates/tailrace-conformance/src/core/cache.rs`).

  Verify with `moon run core:test conformance:test`.

## 2. The conformance suite

- [x] 2.1 In `crates/tailrace-conformance/src/seams/projection.rs`, add `has_manifest`, `held` and `ranged` to `ProjectionRig`. Below the marker, add these scenarios (conformance-suite spec; design D6):
  - "required as the manifest spells it", citing 50 and 58;
  - "satisfied by the ecosystem's own ordering", citing 23;
  - "no manifest, none", citing 58.

  The two manifest scenarios are not applicable with the reason "the ecosystem has no manifest", and the none scenario with the reason "the ecosystem has a manifest". The module doc gets a sentence saying they cite in place of 140 until the statement list holds it. Verify with `moon run conformance:test`.
- [x] 2.2 Implement the rig methods in `FakeProjectionRig` in `crates/tailrace-conformance/src/fakes.rs`:
  - the held artifact's manifest is set to a scoped name with a caret range and a plain name with a prerelease range;
  - `ranged` is `^1.2` over `1.9.0, 2.0.0, 1.10.0, 1.2.0, 1.1.9`, expecting `1.10.0, 1.9.0, 1.2.0`;
  - `unreachable` adds both new ops to the faults it sets.

  In `crates/tailrace-conformance/tests/fakes.rs`, both fake runs expect "no manifest, none" as not applicable. Verify with `moon run conformance:test`.
- [x] 2.3 Implement the rig methods in the file and image rigs (`crates/tailrace/tests/projections.rs`) and the moon cache rig (`crates/tailrace/tests/moon_cache_projection.rs`): no manifest, an artifact of the rig's ecosystem as `held` with no requirements, and an empty `ranged`. Add the three new scenario names to `PACK` and `CACHE`. Each test expects the two manifest scenarios as not applicable beside the lockfile one, and change each module's doc comment to say so. Verify with `moon run cli:test`.
- [x] 2.4 Confirm the register is unchanged and the coverage check passes, with no new pending clause and no citation of a clause outside the list. Verify with `moon run conformance:coverage`.

## 3. The site

- [x] 3.1 In `site/src/content/docs/guides/integrations/projections.mdx`, make the seam's paragraph say six operations (84 to 86, 137, 140), adding:
  - **requires**: given a held version's artifact, the names of the ecosystem and the ranges its manifest requires, as spelled;
  - **satisfying**: given a range and versions, those that satisfy it, newest first by the ecosystem's own ordering;
  - that an ecosystem with no manifest, a file, an image or a cache, answers none.

  Verify with `moon run site:build`.

## 4. The whole gate

- [ ] 4.1 Run the check-fast layer, which covers format, lint, every touched crate's tests, the coverage check and the site's build. Verify with `moon run ':#check-fast'`, and get a passing `sluice exec main` once committed.
