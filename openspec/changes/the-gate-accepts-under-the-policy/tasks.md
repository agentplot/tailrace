# Tasks

Every cargo and moon command runs inside `devenv shell --`. A task's
verification is the moon task named, which the gate runs under
`check-fast`: `tailrace-core:test`, `tailrace-conformance:test`,
`tailrace-conformance:coverage` and `site:build`.

## 1. The gate's judgment

- [x] 1.1 Make `verify::tell` and its `Told` reachable from the crate (`pub(crate)`), changing nothing of what verify answers (design D3); verify with `tailrace-core:test`, every existing verify test passing unchanged
- [x] 1.2 Add `crates/tailrace-core/src/gate.rs` with `AcceptAsk`, `ToldBy`, `Judged`, `Moved` and the `why` of a refusal, all `Serialize`, exported from the core, and the module's row in the core's module table in `lib.rs` naming its clauses and rules (design D1); verify with a test that each answer renders through `answer_json` and names the index, the revision and the revision whose policy was read
- [x] 1.3 Judge in the order of design D2, steps 1 to 5: the branch's head landed, the components at the head, the tag resolved to the index before anything of the version is read, the tag's revision landed, the policy for acceptance at the head; verify with tests that a name the store does not hold is absent naming the tag, an unlanded head or version revision is refused naming the branch and revision with nothing written, and a policy landed after the publish is the one read (107)
- [x] 1.4 Count each artifact against each required claim in order, with the joins of design D3: the revision named, a verdict carried in the provenance for the check and build, a claim on the artifact or the index, an expired maker not counting, the first artifact and first required claim wanting named with `Missing` or `Expired`; verify with tests for every required claim found, an unnamed signer, a signer beyond its reach, another revision's provenance, an expired signer, and the second of two artifacts wanting refusing the version whole with nothing attached (9, 13, 28, 58, 81)
- [x] 1.5 Count a rebuild only from a signer other than the provenance's maker and only with `matched: true`; verify with tests for a rebuild by the offering host and one that did not match, each refused naming the required rebuild with `Unmatched` (30)
- [x] 1.6 Answer unreadable, with the store's own words, where the index is not an index or a digest's claims cannot be read, never a refusal; verify with a test over a store scripted to fail `claims-of` (58, 59)

## 2. Acceptance

- [x] 2.1 Sign the acceptance claim of design D4 as `ask.gate` through the bound signers before any write, absent naming the identity where none holds it, then attach it to the index alone; verify with tests that an accepted version's index holds one more claim signed as the gate, every artifact's claims and every digest are unchanged and no put was asked, and that a host with no gate key answers absent having written nothing (7, 26, 79, 95)
- [x] 2.2 Refuse `OfferedByGate` where the counted provenance's maker is the gate's identity, unless the policy's signer lists both provenance and acceptance, and write `offered.by` and `one_host` into the claim; verify with tests for a build host running the gate refused, the solo laptop's claim saying one host, and a separated gate's claim naming the offerer (27, 96, 108)
- [ ] 2.3 Answer an index already holding the gate's acceptance as accepted with that claim's digest and `already`, attaching nothing, whatever the head's policy now requires; verify with a test that judges again after a stricter policy lands (94)

## 3. The channels that follow each publish

- [x] 3.1 Read `[channels.<name>]` tables with `on_each_publish = true` and their `component` from the policy file at the head through `History::blob`, unreadable naming the table for `bound_to`, a missing component among several or an undeclared one (design D6); verify with tests over a one-component and a two-component history, where a publish of one component follows only its own channels (37, 124, 125)
- [ ] 3.2 For each following channel, only when told by a publish, judge the index under the channel's own policy at the head, answering `Held` with the claim wanting where it refuses; verify with tests that an asked judgment moves nothing and that a channel requiring a review the version lacks stays where it was while the version is accepted (95)
- [ ] 3.3 Promote as design D7 steps 2 to 6: the chain's head read, `There` where it is already about the index, the promotion signed as the gate naming its predecessor, appended, attached to the index, then the name moved through `repair::follow` from where the old head left it; a refused append or a fork answered with nothing after it done or retried; verify with tests that the keeper's append comes before the store's name move, that a chain moved by another append between read and append is refused naming the head with the name unmoved, and that a fork is reported (33, 35, 39, 105, core.6, core.34)
- [ ] 3.4 Show `on_each_publish`, `component` and the one-host signer (`kinds = ["provenance", "acceptance"]`) in the policy section of `site/src/content/docs/guides/self-hosting.mdx`, and say there that the gate moves a following channel and a publish never does; verify with `site:build`

## 4. The suite

- [ ] 4.1 Add the gate group's scenarios below its marker in `crates/tailrace-conformance/src/core/gate.rs`, with any helper the world needs added to `world.rs` without changing one above (design D8): S3 accepted with the acceptance attached, the bytes the ones offered and `beta` moved with the ledger holding the move first; S5 a verdict from an unnamed signer refused with nothing written and the version readable as offered; S9 a thief's acceptance not counted and `beta` unmoved, a build host running the gate refused, a host with no gate key absent, and a publish alone moving nothing; and one scenario each for 26 to 30, 94 to 96, 107 and 108; verify with `tailrace-conformance:test`
- [ ] 4.2 Take the gate's block, 25 to 30, 94 to 96, 107 and 108, out of `register.rs`, drop its now-unused unit constant, set the gate's row in the suite's table in `lib.rs` to the `FakeWorld`, and leave only `layouts` in the empty-group test; verify with `tailrace-conformance:coverage` reporting none of those clauses pending and no clause both cited and registered, and `tailrace-conformance:test`
- [ ] 4.3 Run the gate's fast layer, `moon run ':#check-fast'`, and verify every task passes, format and lint among them
