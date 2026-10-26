Every command runs inside `devenv shell -- <command>`. Each group's
tasks are verified by the moon tasks named, which `check-fast` runs.

## 1. The seam: pinned and the layout-policy claim

- [x] 1.1 Add `Pinned { name: EcosystemName, version: Version }` and `Projection::pinned(&self, checkout: &str) -> Answer<Vec<Pinned>>` to `crates/tailrace-seams/src/projection.rs`, exported from `lib.rs`, with a round-trip test (design D1); verify with `moon run seams:test`
- [ ] 1.2 Add `ClaimKind::LayoutPolicy`, named `layout_policy`, to `claim.rs` and its `ALL`, with a test that a layout-policy claim about an index reads back from its envelope with the revision, no build identity and its body (design D8); verify with `moon run seams:test`
- [ ] 1.3 Add `Why::Wanting { version, index, wanting: RequiredClaim, checked: Vec<SignerId> }` to `answer.rs`, rendered in words that name the version, the claim's kind and check and the signers checked, with a round-trip test (design D4); verify with `moon run seams:test`
- [ ] 1.4 Give the fake projection pins per checkout set by the test, absent naming `lockfile` for any other, and a render of artifacts that records the place and the artifacts given and answers `Holds::Versions` (seam-fakes spec); verify with `moon run fakes:test`
- [ ] 1.5 Answer `pinned` absent naming `lockfile` in the file, image and moon cache profiles, each with a unit test (file-, image- and moon-cache-projection specs); verify with `moon run cli:test`

## 2. Judging and rendering one layout

- [ ] 2.1 Make `tell` in `crates/tailrace-core/src/verify.rs` `pub(crate)`, changing nothing else, and verify `moon run core:test` passes unchanged
- [ ] 2.2 Create `crates/tailrace-core/src/render.rs` with `ARTIFACT` and `artifact_name`, and a pin's resolution through the index to the one artifact of its ecosystem and name, absent where the name is not held and unreadable naming name and digest otherwise (design D2); add it to `lib.rs` and its module table; verify with tests in `crates/tailrace-core/tests/render.rs` under `moon run core:test`
- [ ] 2.3 Read the policy for a layout: the consumer's at the landed revision once, else the one layout-policy claim on each version's index, refused naming the publisher's layout policy where there is none and unreadable naming both where two differ (design D3); verify with tests for each case in `tests/render.rs`
- [ ] 2.4 Judge a version: each required claim in the policy's order satisfied by an attestation on the index or the artifact, or a verdict carried in its provenance, told by a named signer allowed its kind and check, the first unsatisfied refused with `Why::Wanting`, an expired signer counting and reported (design D4); verify with tests in `tests/render.rs` for satisfied, unnamed signer, wrong kind, expired, and the first of two wanting named
- [ ] 2.5 Write `render::render`: every read before any write, then the projection's render given exactly the satisfied artifacts at `.tailrace/layouts/<ecosystem>/`, `.tailrace/.gitignore` holding `*` written when `.tailrace/` is made, answering `Refreshed`; and the `Kept` answer from the record when the policy or any pin's read is unreadable or refused by its source, or the render fails (design D5, D6); verify with tests in `tests/render.rs` for a refused version leaving the layout, a third-party pin absent, and the store unreachable with and without a record

## 3. The record and the sync

- [ ] 3.1 Add `History::git_dir()`, the worktree's own git directory, with a test in `tests/history.rs` that two worktrees answer two directories and one common directory; verify with `moon run core:test`
- [ ] 3.2 Write `Layouts`, the record at `<git dir>/tailrace/layouts/<ecosystem>.json`, written beside and renamed, read back for `Kept`, unreadable naming the file with the parser's words (design D7); verify with tests in `tests/render.rs` that the file reads as JSON naming every held and refused pin, and that a deleted record is written again the same but for `rendered_at`
- [ ] 3.3 Write `render::layouts` and `render::sync_checkout`: the cache's `sync` unchanged, then every bound projection's pins at the working tree rendered per ecosystem, a projection answering absent passed over, answering `{ branch, revision, entries, layouts }` in the one JSON form (design D5); verify with tests in `tests/render.rs` that the policy is not asked when nothing is pinned and that no name the layouts resolve begins `cache/`
- [ ] 3.4 Point `tailrace sync` at `sync_checkout`, passing the clock's seconds and the record at the checkout's git directory, rendering text per layout and pin with a count of absent pins, and the exit as design D9; update the command's doc comment; verify with tests in `crates/tailrace/tests/` for `sync --json` over the fakes with one pin held, one refused and one absent (exit 1, JSON equal to the core's answer) and with the store's resolve unreadable as `connection refused` (exit 3, the words printed), under `moon run cli:test`
- [ ] 3.5 Add a section to `site/src/content/docs/guides/integrations/projections.mdx` saying what `tailrace sync` now does for layouts: the lockfile's versions only, the consumer's `[layout]` policy or the publisher's, refusals named with their claim, `.tailrace/layouts/<ecosystem>/` as the place a client's configuration names, and what a layout that cannot be refreshed answers; verify with `moon run site:build`

## 4. The publisher's layout policy at publish

- [ ] 4.1 In `crates/tailrace-core/src/publish.rs`, after the index is put and before the version's name moves, sign and attach the layout-policy claim where the policy declares one at the landed revision, attaching nothing where it declares none or the index already carries one of the same body (design D8); update the module table's row; verify with tests in `crates/tailrace-core/tests/publish.rs` for a policy declared, none declared, a repeat publish, and the attach coming before the name's move in the fake store's calls, under `moon run core:test`

## 5. The conformance suite

- [ ] 5.1 Add to the projection group, below its marker, "pinned as the lockfile spells it" and "no lockfile, absent", citing 50 and 58, with the rig's lockfile methods implemented by the fakes' rig, the file, image and moon cache rigs, and the broken and lagging rigs in `crates/tailrace-conformance/tests/` (conformance-suite spec); verify with `moon run conformance:test` and `moon run cli:test`
- [ ] 5.2 Add the layouts group's scenarios below its marker: S4, a pin the store does not name, the consumer's own policy at its landed revision, the publisher's where it declares none, no policy anywhere, a version refused naming its claim, a version refused after it was held, a layout that cannot be refreshed, and the record deleted and rebuilt, citing 47, 50, 51, 61, 62, 63, 99 and 113 as the spec gives them; the fake world gains pins and the `artifact/` names a test writes; verify with `moon run conformance:test`
- [ ] 5.3 Remove 47, 51, 99 and 113 from `register.rs` in the same commit as 5.2, and verify the coverage check prints none of them as pending with `moon run conformance:test conformance:coverage`
