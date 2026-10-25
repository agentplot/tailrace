# Tasks

Every cargo and moon command runs inside `devenv shell --`. A task's
verification is the moon task named, which the gate runs under
`check-fast`: `seams:test`, `core:test`, `fakes:test`, `cli:test`,
`conformance:test`, `conformance:coverage` and `site:build`. The
workspace builds only once every `Policy` implementor answers
`channels`, so groups 1 and 2 land in one commit.

## 1. The seam

- [x] 1.1 Add `DeclaredChannel` and `Policy::channels` to `crates/tailrace-seams/src/policy.rs` as design D1 gives them, with doc comments citing 131, 125, 37 and 83, and the policy's row of the seams' table in `lib.rs` naming 131; add a round-trip of a bound and an unbound `DeclaredChannel` to the module's tests and a test that an unbound one writes `bound_to` as null (spec `seam-contracts`, a declared channel names its component); verify with `seams:test`
- [x] 1.2 Add `REPOSITORY_COMPONENT` to `crates/tailrace-seams/src/artifact.rs` and make `crates/tailrace-core/src/declared.rs`'s `REPOSITORY` its re-export (design D2); verify with `core:test`, every `declared` test passing unchanged
- [x] 1.3 Give the seams' `Nowhere` a `channels` answering as its `policy_for` does, and add `channels` to the operations its test asks of the policy; verify with `seams:test` (spec `seam-contracts`, each seam's operations)
- [x] 1.4 Add `channels_at` to `crates/tailrace-core/src/landed.rs` beside `policy_at` (design D7), and `channels` delegating to the inner fake on both `Watched` policies (`crates/tailrace-core/tests/verify.rs`, `crates/tailrace-conformance/src/core/verify.rs`); verify with a test in `core:test` that `channels_at` asks the policy at the landed revision

## 2. The fake and the file

- [x] 2.1 Add `PolicyOp::Channels`, `belongs`, `belongs_at` and `channels` to `crates/tailrace-fakes/src/policy.rs` as design D4 says; verify with tests in `fakes:test` for the spec's scripted channels: a channel at one revision in `lib` and at another in `repository`, a bound channel's binding from its policy, the revision's policy over every revision's, no channel answering an empty list, and `Channels` faulted unreadable and recorded (spec `seam-fakes`)
- [x] 2.2 In `crates/tailrace/src/profiles/policy/file/mod.rs`, read `[[component]]` names and a channel's `component`, and resolve a channel as design D3 steps 1 to 3, shared by `channels` and by `policy_for` of a channel; update the module's doc to name `component` among the keys it reads and the TOML example to show one; verify with `cli:test`
- [x] 2.3 Add to `crates/tailrace/src/profiles/policy/file/tests.rs` one test for each scenario of spec `policy-file`'s three added requirements: one component and no `[[component]]`, the one declared component, no component among several (the channels and the policy for the channel both unreadable, naming the table and both components), an undeclared component, a binding across components and to an undeclared channel, two components' channels in name order, no channel and no file answering empty, and a checkout's channel not read; verify with `cli:test`

## 3. The suite

- [ ] 3.1 Add `declaring` to `PolicyRig` in `crates/tailrace-conformance/src/seams/policy.rs`, implement it on `FakePolicyRig` (`src/fakes.rs`, which also faults `Channels` in `unreachable`) and on `PolicyFileRig` (`crates/tailrace/tests/policy_file.rs`) as design D5 says; verify with `conformance:test` and `cli:test`, every scenario above the marker passing unchanged
- [ ] 3.2 Below the policy group's marker, add the four scenarios of design D5 with the citations its table gives, changing no scenario above the marker (spec `conformance-suite`, the policy's contract); verify with `conformance:test` over the fakes and `cli:test` over the policy file, each group's report naming no scenario not applicable
- [ ] 3.3 Bring `crates/tailrace-conformance/src/statement.rs` to agentplot/blueprints 673d334 as design D6 says: `READ_AT`, the six clauses where the statement places them, B.8 named the ledger, 124 once, and its test asserting 1 to 131 once each in 20 sections (spec `clause-coverage`); verify with `conformance:test`
- [ ] 3.4 In `crates/tailrace-conformance/src/register.rs`, remove 125 and add 126 to 130 with the units and reasons of design D6's table, a `PHASE_2` constant naming phase 2; verify with `conformance:coverage` passing, naming 126 to 130 pending, 125 and 131 cited, and no clause both cited and registered

## 4. The guide and the gate

- [ ] 4.1 In `site/src/content/docs/guides/self-hosting.mdx`, say beside the policy file's example that a channel belongs to one component, named by `component` and left out where one is declared, that a channel is bound only within its component, and that a file breaking either is unreadable naming the table (37, 125, 131); verify with `site:build`
- [ ] 4.2 Run the fast layer, `moon run ':#check-fast'`, and verify every task passes, format and lint among them
