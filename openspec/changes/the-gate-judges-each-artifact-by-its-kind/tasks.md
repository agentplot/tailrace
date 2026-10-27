# Tasks

The unit's branch already holds main's gate, taken into this bolt by
da0abc6: `crates/tailrace-core/src/gate.rs`, its tests and the acceptance
spec are on the tree. Until task 3 is done, `core:test`, `conformance:test`
and `cli:test` fail as sluice #62 did, refusing each version for its
notes' missing verdict. Every cargo and moon command runs inside
`devenv shell`.

## 1. The seam names the artifacts a claim is asked of

- [x] 1.1 In `crates/tailrace-seams/src/policy.rs`, add `AskedOf { Every, Built }` with `is_every` and `asks(made)`, and add `asked_of: AskedOf` to `RequiredClaim` with `#[serde(default, skip_serializing_if = "AskedOf::is_every")]` (seam-contracts spec; design D1). Re-export it from the seams' root. Add tests for both seam-contracts scenarios: the two claims written and read back, and a body with no `asked_of` read as `Every`. Verify with `moon run seams:test`.
- [x] 1.2 Name `asked_of` in every place that makes a `RequiredClaim`: the core (`ledger.rs`, `render.rs`), the fakes, the conformance suite, the policy file profile and every crate's tests. Use `AskedOf::Every` wherever the claim is not one the change asks of built artifacts alone, so no behaviour changes in this task. Verify with `moon run seams:test fakes:test core:test conformance:test cli:test`.

## 2. The policy file reads it from the kind

- [x] 2.1 In `crates/tailrace/src/profiles/policy/file/mod.rs`, answer each `[acceptance]` entry, each channel's `require` entry and the given channels' gate acceptance with `asked_of` from the kind: `Every` for provenance, acceptance and promotion, `Built` for verdict, test, review, rebuild and carried. Answer every `[layout]` entry `Every`. The file gains no key and `Entry` keeps `deny_unknown_fields` (policy-file spec; design D2). Say so in the module's doc comment. Verify with `moon run cli:test`.
- [x] 2.2 In `crates/tailrace/src/profiles/policy/file/tests.rs`, add a test for each policy-file scenario:
  - acceptance's two claims of two kinds;
  - a given channel's line;
  - the one host's own given acceptance;
  - a layout's claims;
  - a key naming the artifacts.

  Change the expected answers of the existing tests where a verdict, a review or a rebuild is now `Built`. Verify with `moon run cli:test`, and that the policy group still passes on the file profile.

## 3. The gate judges each artifact by its making

- [x] 3.1 In `crates/tailrace-core/src/gate.rs`, add `Made { Built, Rendered }`. Tell each artifact's making once in `accept`'s step 5, under the acceptance policy at the head, from the artifact's own provenance claims that count through the policy's provenance signers and name the version's revision (acceptance spec, the gate tells a rendered artifact; design D3). Keep it beside the artifact in `Judging.artifacts`. Do it before the already-accepted check, so that path has it too. Verify with `moon run core:test`.
- [x] 3.2 Make `Judging::count` take, for each artifact, only the required claims whose `asked_of` asks its making, in the policy's order. `promote` then judges each following channel by the same making (design D4). Leave `Judged`, `Moved` and the acceptance body's shape as they are. Update the module's doc comment to cite 134. Verify with `moon run core:test`.
- [x] 3.3 In `crates/tailrace-core/tests/gate.rs`, add tests for every new and modified acceptance scenario:
  - a claim asked of built artifacts alone;
  - the notes judged as rendered;
  - a build-less provenance from a signer the policy does not name;
  - an output's provenance names its build;
  - a policy that asks no provenance;
  - the outputs' claims count and the notes' provenance does not;
  - the outputs' verdict wanting;
  - beta moves with notes that carry no rebuild;
  - beta held for an output's rebuild.

  Make the existing tests' policies ask their verdicts, reviews and rebuilds of built artifacts alone where the publish now offers notes, which clears the 24 refusals sluice #62 showed. Verify with `moon run core:test`.
- [x] 3.4 In `crates/tailrace-conformance/src/core/gate.rs`, below the marker, add these scenarios to the gate group:
  - "a version whose notes carry no build verdict";
  - "a rendered artifact wanting the publisher's provenance", where the store hides the notes' claims, as `LedgerWatched` wraps the store;
  - "the outputs' claims wanting beside the notes";
  - "a following channel and the notes".

  Each cites 25 and 28, and the channel's 95 too. `statement.rs` lists no 134 (it reads the statement at 157c828), so none cites it: the module's doc says they cite 25 and 28 in place of 134 until the list adopts it, and the list is left alone. Move the group's acceptance and channel policies to ask verdicts, reviews and rebuilds of built artifacts alone, which clears the group's refusals in sluice #62. Verify with `moon run conformance:test conformance:coverage`.

## 4. The policy command shows it

- [x] 4.1 In `crates/tailrace/src/commands/policy.rs`, render a `Built` required claim with `, of built artifacts alone` after its signers, and nothing more for an `Every` claim (command-line spec; design D5). In `crates/tailrace/tests/cli.rs`, add a test for the command-line scenario, and update the expected text of the existing policy tests where a verdict or a rebuild now shows the mark. Verify with `moon run cli:test`.

## 5. A publish that tells the gate, end to end

- [x] 5.1 In `crates/tailrace/tests/publish_tells_the_gate.rs`, make `test_from_the_landing` ask its verdict of built artifacts alone, so the six tests sluice #62 failed accept the version. In `a_file_with_no_gate_table_tells_the_gate_as_publish_does` or a new test beside it, assert that the version is accepted, that its acceptance names the notes' provenance and no verdict of the notes, and that `beta` moved. Do the same for any other CLI test where a publish of a version with notes tells the gate (`landings_end_to_end.rs`, `cli.rs`). Verify with `moon run cli:test`.

## 6. The record and the guide

- [x] 6.1 Add ADR 0022 under `docs/adr/`, the next number on the tree, recording design D1 to D3 and the publisher-key risk. Verify with `moon run root:adrs root:format`.
- [x] 6.2 In `site/src/content/docs/guides/self-hosting.mdx`:
  - in the section on the gate, say that a version's notes are asked only the publisher's provenance and the claims about the version, and that a verdict, a review or a rebuild is asked of built artifacts alone (134);
  - show the mark in the `tailrace policy` example.

  Verify with `moon run site:build`.

## 7. The whole gate

- [x] 7.1 Run the check-fast tasks of every touched project: `moon run seams:test fakes:test core:test conformance:test conformance:coverage cli:test site:build root:format root:lint root:adrs`, plus `cli:tools` for the tools layer. The whole layers run in sluice at merge. Once committed, get a passing `sluice exec main`.
