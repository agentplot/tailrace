# Tasks

Every cargo and moon command runs as `devenv shell -- <command>`. This
change builds on `the-seams-and-their-fakes`,
`a-landed-revision-takes-its-version` and
`publish-status-and-verify-from-one-core` as they stand. Before
starting, read the seams (`answer.rs`, `artifact.rs`, `claim.rs`,
`bind.rs` and the eight trait modules), the fakes and their scripting
(`script.rs` and each fake's own methods), the core's `landed.rs`,
`history.rs`, `declared.rs`, `version.rs`, `publish.rs`, `verify.rs`,
`status.rs` and `published.rs`, the core's test world and fixture
(`crates/tailrace-core/tests/world/mod.rs`, `tests/common/mod.rs`), and
the binary's `src/lib.rs` and `tests/cli.rs`. Where a name differs from
the design's, take the name as it stands. This change adds a binary to an existing crate and no
crate, language or file type. Its one new check is a moon task of the
conformance project (core.25). No trait, fake or core function is
changed. A scenario that cannot be written against what landed is a
`crew needs`. Nothing under `crates/tailrace-conformance/src/` names a
word core.1 lists, in code, strings or comments.

## 1. The harness

- [x] 1.1 In `crates/tailrace-conformance/Cargo.toml` take `tempfile` from the workspace as a dependency (the git fixture is in `src/`), and in `moon.yml` add the `coverage` task of design D8 with the `test` task's inputs. Verify `cargo check -p tailrace-conformance --all-targets` succeeds and `Cargo.lock` gains only the conformance crate's `tempfile` edge.
- [x] 1.2 Write `src/scenario.rs`: `Scenario<R>`, `Rig`, `run` with a fresh rig per scenario under `catch_unwind`, and `Report` with `assert()` (design D1, D3). Verify with unit tests: a group of three scenarios, two of which panic, reports both failures with their names, what they play, their clauses and the panic messages, and runs the third; a group that passes reports nothing and `assert()` returns (spec conformance-suite: A failing profile is reported the same way; Every scenario is run).
- [x] 1.3 Write `src/lib.rs`'s doc comment (what the suite is, its groups, how a later unit adds below a marker, how a profile runs its seam's group) and `citations()` over every group's list (design D1, D2). Verify with a unit test that every group of design D2 appears in `citations()` or is listed with no scenario, and that every scenario has a name and at least one clause (spec: Every scenario cites the clauses it exercises; Scenarios sit in groups that later units only add to).
- [x] 1.4 In `src/scenario.rs` add `not_applicable(why) -> !`, unwinding with a `NotApplicable` payload through `resume_unwind`, and the report's `not_applicable` list and its text (design D3). Verify with a unit test: a group of three where one scenario calls `not_applicable` and two pass passes, `assert()` returns, and the report's text says `2 passed, 1 not applicable` and names that scenario, its clauses and the reason; a scenario that panics after the rig fails to stage what it does is still a failure (spec conformance-suite: A scenario that does not apply is named as not applicable).

## 2. The seams' groups over the fakes

- [x] 2.1 Write `src/seams/store.rs`, `build.rs` and `signer.rs`, each with its rig trait (design D3), its scenarios with the citations of design D5, and its marker as the last line of its list. Write their fake rigs in `src/fakes.rs`. Write `tests/fakes.rs` with one `#[test]` per group, running the group over its fake rig with `run(..).assert()`. Verify `cargo test -p tailrace-conformance --test fakes` passes every scenario of spec conformance-suite under: The store's contract; The build's contract; The signer's contract; A source that cannot be reached is unreadable on every seam, for those seams.
- [ ] 2.2 Write `src/seams/policy.rs`, `projection.rs`, `target.rs` and `keeper.rs` the same way, with their fake rigs and tests. The target rig's `verified` makes the seams' `Verified` with `Verified::new`, as the core does after its verify (design D5). The policy group's "no policy for an audience" asserts an absence from the policy naming the layout audience, and never a refusal; it asserts nothing of the revision, which is verify's to name (design D5). Verify `cargo test -p tailrace-conformance --test fakes` passes every scenario of spec conformance-suite under: The policy's contract; A projection's contract; A target's contract; The keeper's contract.
- [x] 2.3 Write `src/seams/binding.rs` with its two scenarios, and `tests/broken.rs` with a wrapper over the fake store that always answers "not held already" and a fake store scripted to hold other bytes under a digest (design D9). Verify with `cargo test -p tailrace-conformance --test broken`: each broken store fails the store group, and the report's text names the profile given for the test, the store, the failing scenarios, their clauses (1, 5 and 66 for the bytes put twice) and the store's answer, in the same form as any profile's report. Verify the binding group passes in `tests/fakes.rs` (spec: The binding; A failing profile is reported the same way, a fake included).
- [ ] 2.4 Rewrite `src/seams/landing.rs` as design D3a says: `Recorded`, `LandingRig` with `records_verdicts()` and `landed(&[Recorded])`, the scenario of verdicts as the landing recorded them calling `not_applicable` where the rig records none and otherwise asserting the four as recorded, each once in any order, and each envelope of the in-toto payload type with a payload and its signature naming the verdict's signer, a scenario of a revision with no verdict citing 78, and a landing that cannot be reached staging no verdict; the marker stays the last line. Move the envelope's making into `FakeLandingRig` in `src/fakes.rs`, and add `FakeLandingRig::recording_none()`. In `tests/fakes.rs` run the group over both fake rigs. Verify `cargo test -p tailrace-conformance --test fakes` passes every scenario of spec conformance-suite under: The landing's contract; A landing that cannot be reached; and that the run over `recording_none()` passes with its report naming verdicts as the landing recorded them as not applicable. Verify `grep -n 'Envelope\|Signature' crates/tailrace-conformance/src/seams/landing.rs` finds no envelope made there. Commit this group on its own, since `moon-and-the-landings` builds its rigs on that commit.

## 3. The core's groups over the fakes

- [x] 3.1 Write `src/fixture.rs` (the git fixture under a fixed author, committer and date: a bare shared repository, a clone, `commit`, `tag`, `clone_again`, `policy_file`) and `src/world.rs` (`FakeWorld` and `FakeWorld::two_components()`, with its observation methods) as design D4 says. Verify with unit tests: two worlds built alike have equal revisions; a tag made in one clone is read by the other through the shared history; `writes()` lists a put, an attach and a name move in the order they were asked.
- [x] 3.2 Write `src/core/publish.rs` with S1, S2 and the refusals, citing as design D5, and its test in `tests/fakes.rs`. Verify every scenario of spec conformance-suite under S1 over the fakes, S2 over the fakes, and Publish's refusals over the fakes passes, and that S1 asserts the order of puts (every artifact and its claims before the index, the index before the name), the provenance's subject, revision and build identity, the carried verdict byte for byte, and no keeper or target call.
- [x] 3.3 Write `src/core/verify.rs` with S5 and S13 to verify's answer and the other verify scenarios, citing as design D5. Verify every scenario of spec conformance-suite under S5 over the fakes, S13 over the fakes, and Verify's answers over the fakes passes, and that S5 asserts no name move but the version's own, no keeper append, and the index read back by digest equal to what was offered.
- [x] 3.4 Write `src/core/version.rs` with S15 over `FakeWorld::two_components()` and the other version scenarios, citing as design D5. Verify every scenario of spec conformance-suite under S15 over a two-component fixture passes, and that S15 asserts the untouched component has no new tag in either history, no index, no name in the store, and that the fake build recorded no version.
- [x] 3.5 Write `src/core/status.rs` with its three scenarios. Write `gate.rs`, `ledger.rs`, `cache.rs`, `record.rs` and `layouts.rs`, each holding an empty list with its marker and a doc comment naming the unit that will fill it (design D2). Verify every scenario of spec conformance-suite under Status over the fakes passes, and the unit test of 1.3 lists the five empty groups (spec: The gate's group waits for its unit).

## 4. The surfaces group over the command line

- [x] 4.1 Write `src/core/surfaces.rs` with the `Surface` trait, `Question`, and the one scenario (design D6). Write `crates/tailrace/tests/conformance.rs`, implementing `Surface` with `tailrace::run` over the world's seams and repository with `--json`, and running the group. Verify `cargo test -p tailrace --test conformance` passes, and that changing one field of the core's answer in a local trial makes it fail naming the question (spec: The surfaces answer from the one core).

## 5. Clause coverage

- [x] 5.1 Write `src/statement.rs` with `READ_AT` and `PARTS`, each section of Parts A and B with its clause numbers as `agentplot/blueprints` 5b00e15 orders them (design D7). Verify with a unit test that `PARTS` holds 1 to 125 exactly once each, in 20 sections (spec clause-coverage: The suite knows the clauses of Parts A and B).
- [x] 5.2 Write `src/register.rs` with `Pending` and the 47 entries of design D7, one per line, grouped by unit in the plan's order, each `why` one sentence (design D7). Verify `grep -niwE 'oci|ecr|ghcr|oras|cosign|moon|sluice|kms|npm|maven|cargo|flux|gh' crates/tailrace-conformance/src` finds nothing (core.1).
- [x] 5.3 Write `src/coverage.rs` (`check`, `Coverage`, its findings) and `src/bin/coverage.rs` (design D8). Verify with unit tests in `coverage.rs`, one per scenario of spec clause-coverage: a clause left out is named; an entry with no reason is named; a clause cited and registered is named with its scenario; a citation of 126 and a scenario with no clause are named; a clause registered twice is named; the pending entries and counts are returned on a pass.
- [x] 5.4 Verify `cargo run -q -p tailrace-conformance --bin coverage` exits 0 on this tree, prints the 47 pending clauses of design D7 with their units, and reports 78 clauses cited (spec: What the register holds in bolt 1; Pending on a pass). Then, in a local trial left uncommitted, remove the citation of 14 from its scenario and verify the binary exits 1 naming 14. Add 25 to a scenario's citations and verify it exits 1 naming 25 and the scenario.

## 6. The gate

- [ ] 6.1 Verify `devenv shell -- moon run ':#check-fast'` passes: format, lint, toolchain, ADR numbers, every crate's tests, the conformance project's `coverage` task, and the site's build. Verify the gate's output shows the coverage task's pending lines on this passing run (spec: Coverage is a check of the gate; A registered clause is reported as pending on every run).
