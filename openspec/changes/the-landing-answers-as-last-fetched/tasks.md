# Tasks

Every command runs inside `devenv shell -- <command>`.

## 1. The seam says how the landing read its answer

- [x] 1.1 Move `Read` from `crates/tailrace-seams/src/keeper.rs` to `answer.rs`, its form and JSON unchanged and its doc saying how any answer was read (design D1); the keeper's tests pass unchanged: `moon run seams:test`
- [x] 1.2 Move the UTC rendering `render::utc` holds into `tailrace-seams` beside `Read`, `crates/tailrace/src/render.rs` calling it, with a test in the seams crate of an epoch time and a leap day rendered (design D5): `moon run seams:test cli:test`
- [x] 1.3 In `landing.rs`, add `IsLanded { landed, read }` and `Verdicts { verdicts, read }`, change `landed` and `verdicts_of` to answer them, and write the contract's docs from the landing's contract as the seam-contracts delta words it; round-trip tests for both, with `Read::Source` and `Read::Kept`: `moon run seams:test`
- [x] 1.4 Add `Why::NotLandedAsFetched { branch, revision, fetched }`, its text naming the time in UTC and saying to ask again once the landing's remote answers (design D5); a test that it and `NotLanded` read back from JSON as made, told apart, the remote's words kept verbatim in `said`: `moon run seams:test`
- [x] 1.5 Bring `Nowhere` in `lib.rs` and `bind.rs` to the new signatures: `moon run seams:test`

## 2. The fake landing answers as last fetched

- [x] 2.1 Change `FakeLanding` to the new answers, every answer while reachable `Read::Source`, and add `unreachable(words)`: the whole state copied at each answer with its time, every question answered from the copy once unreachable, unreadable with the words before any answer (design D7); tests for the three scenarios of the seam-fakes delta, scripted faults still answered first: `moon run fakes:test`
- [x] 2.2 Bring `crates/tailrace-fakes/src/lib.rs`'s tests to the new answers: `moon run fakes:test`

## 3. The core: a landed revision says how it was read

- [x] 3.1 In `crates/tailrace-core/src/landed.rs`, give `Landed` its `read` and `read()`, and make `Landed::check` answer a yes from either read, `NotLanded` for a no from the landing itself and `NotLandedAsFetched` with the remote's words for a no from the copy (design D5); unit tests for all three: `moon run core:test`
- [x] 3.2 In `publish.rs`, take `.verdicts` from verdicts-of's answer; tests in `crates/tailrace-core/tests/publish.rs` for the publishing delta's two new scenarios, no seam but the landing asked on the refusal and the published answer equal to a world's whose landing answers from itself: `moon run core:test`
- [x] 3.3 In `status.rs`, give `Channels` its `landing: Read` from the landed head (design D6); tests in `crates/tailrace-core/tests/status.rs` for the channel-status delta's scenarios over the fakes, the channels shown and the read carried, and the head refused from a copy as the channels' part's refusal with the record's part answering: `moon run core:test`
- [x] 3.4 Bring every other test of `crates/tailrace-core/tests/` that calls the landing or matches `NotLanded` to the new answers: `moon run core:test`

## 4. The plain branch reads its copy where the remote does not answer

- [x] 4.1 In `crates/tailrace/src/profiles/landing/branch/mod.rs`, after each fetch of a branch remake its copy `refs/tailrace/landing/heads/<branch>` so its reflog's one entry is the fetch's time, and read that time as the git ref ledger reads its own (design D2, D3): a test that a second read moves the copy's time and writes no remote-tracking ref or `FETCH_HEAD`: `moon run cli:test`
- [x] 4.2 Where the remote's listing or fetch answers unreadable, answer landed from the copy of the branch and verdicts-of from the repository with the newest copy's time, each `Read::Kept` with git's words, and the failure itself where no copy answers; with no remote bound answer `Read::Source` (design D4); tests in `crates/tailrace/tests/landing_branch.rs` for the branch-landing delta's scenarios, a remote never fetched among them: `moon run cli:test`
- [x] 4.3 Give the branch rig `reads_a_remote`, and run the landing group a second time over a branch bound to a scratch remote, the rig pushing what it lands and cut off by removing the remote, each run's not-applicable scenarios as the branch-landing delta names them (design D7): `moon run cli:test`

## 5. sluice reads its copy where the remote does not answer

- [x] 5.1 In `crates/tailrace/src/profiles/landing/sluice/`, at every opening read the remote answers, fetch every ref its listing names under `refs/sluice/state/` and `refs/sluice/runs/` in one fetch, and remake the copy whole under `refs/tailrace/landing/sluice/` in two `update-ref --stdin` transactions, old copies deleted then new made (design D2, D3); a test that a ref the remote no longer lists leaves the copy, and that the module's docs no longer say no ref is written: `moon run cli:tools`
- [x] 5.2 Where the remote's listing or a fetch answers unreadable, read the listing from the copy, make no fetch, and answer landed and verdicts-of over it as from the remote, `Read::Kept` with the copy's time and git's words; the failure itself where the copy holds nothing the operation needs, a branch's log among it; with no remote bound, `Read::Source` (design D4); tests in `crates/tailrace/tests/landing_sluice.rs` for the sluice-landing delta's scenarios: `moon run cli:tools`
- [x] 5.3 Give the sluice rig `reads_a_remote`, true, and bring `landings_end_to_end.rs` to the new answers; the landing group passes over sluice with nothing not applicable: `moon run cli:tools`

## 6. The suite holds every landing to it

- [x] 6.1 Add `reads_a_remote` to `LandingRig` and `a landing read as last fetched` to the landing group below its marker, citing 59, 61, 77 and 78, and update `unreachable`'s doc (design D7); the fake rig in `crates/tailrace-conformance/src/fakes.rs` reads a remote and cuts itself off with `FakeLanding::unreachable`, and the group passes over it: `moon run conformance:test`
- [x] 6.2 Add the publish group's refusal from a copy and publish from a copy, citing 14, 59 and 77, and the status group's landing read as last fetched, citing 36, 60 and 77, each below its group's marker, over the fake world; bring `core/cache.rs`, `core/gate.rs`, `seams/host.rs` and `crates/tailrace-conformance/tests/` to the new answers; the coverage check passes with the register and the statement's list unchanged: `moon run conformance:test`

## 7. Status shows how the landing was read

- [x] 7.1 In `crates/tailrace/src/commands/status.rs`, print the landing's read under the branch and revision, `landing: as last fetched <UTC>` with the words indented beneath, nothing for a landing read from itself, and carry it in `--json` (design D6); a test in `crates/tailrace/tests/cli.rs` over the fakes for the channel-status delta's first scenario, status exiting 0: `moon run cli:test`
- [ ] 7.2 In `crates/tailrace/src/commands/serve/page.rs`, render the same on the repository's page; a test in `crates/tailrace/tests/serve.rs` that the page and its JSON carry the read as the command line does: `moon run cli:test`
- [ ] 7.3 A test in `crates/tailrace/tests/status_across_machines.rs` of a machine cut off from the one remote holding its branch and its ledger, the landing over a plain branch bound to it: every channel shown from the ledger's kept chain, the landing and each chain saying when, `--json` and the page the same answer: `moon run cli:test`
- [ ] 7.4 Bring `crates/tailrace/src/bindings/tests.rs` and every other test of `crates/tailrace/tests/` that calls the landing to the new answers: `moon run cli:test`

## 8. The site and the gate

- [ ] 8.1 Write on `site/src/content/docs/guides/integrations/landings.mdx` that with a remote bound the landing is read on the remote and, where it does not answer, from the host's copy as last fetched, saying when; that a no from the copy is refused saying to ask again; and where the copy lives (design D8): `moon run site:build`
- [ ] 8.2 Run the gate's layers: `moon run ':#check-fast'` and `moon run ':#check-tools'`
