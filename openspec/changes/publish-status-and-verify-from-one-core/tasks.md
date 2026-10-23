# Tasks

Every cargo and moon command runs as `devenv shell -- <command>`. This
change builds on `the-seams-and-their-fakes` and
`a-landed-revision-takes-its-version` as they land: read the seams'
`answer.rs`, `artifact.rs`, `claim.rs` and `bind.rs`, the fakes, and the
core's `landed.rs`, `history.rs`, `declared.rs` and `version.rs` with
their test fixture before starting, and take the names that landed where
this design's differ. It adds no crate, language or file type, so no
project or formatter entry is added. Nothing under
`crates/tailrace-core/src/` names a word core.1 lists, in code or in
comments.

## 1. The answers this change gives

- [x] 1.1 In `crates/tailrace-seams/src/answer.rs` add `Source::host(<path>)` and the refusals `Why::NotPassed { output, build, outcome }`, `Why::Unnamed { output, build, carried }`, `Why::NoProjection { component, output }`, `Why::OtherBytes { expected, answered }`, `Why::Differed { artifact, first, now }`, `Why::KeptOther { path }` and `Why::WhichComponent { declared }` (design D2 to D4, D6, D9, D10); in `claim.rs` add `ClaimKind::Provenance` if the seams do not hold it. Verify with unit tests in `answer.rs` that each round-trips through JSON with its source and the source's words byte for byte.

## 2. What publish reads before it offers

- [x] 2.1 In `crates/tailrace-core/src/declared.rs` read `projections` per component and at top level, and give `Component::projection_of` (design D3). Verify with tests in `crates/tailrace-core/tests/declared.rs`: a component's outputs read with their ecosystems; the one component's projections read from the top level; a projection naming an undeclared output unreadable naming it; the earlier unit's tests still pass.
- [x] 2.2 In `crates/tailrace-core/src/history.rs` add `head(&Branch)`, the commit the local branch names, `resolve(&str)`, a revision text to its commit id with git's words when it names none, and `common_dir()` (design D9 to D11). Verify with tests in `crates/tailrace-core/tests/history.rs`: a short id and `HEAD` resolve to the full id; a name that is no commit is absent with git's words; two worktrees of one repository answer one common directory.

## 3. Publish

- [x] 3.1 Write `crates/tailrace-core/src/provenance.rs`: the provenance claim's body with its carried verdicts and where each was found (design D7). Verify with unit tests: a claim names the subject, revision and build identity; a carried envelope reads back byte for byte; a claim with no verdicts carries an empty list.
- [x] 3.2 Write `crates/tailrace-core/src/publish.rs`: `Ask`, `Publish`, `Published`, `PublishedArtifact`, `Attached` and `publish` in the order of design D1, judging as D2, bytes checked as D4, the store name as D5, the repeat as D6, and `answer_json` (D8). Verify with tests in `crates/tailrace-core/tests/publish.rs`, one per scenario of spec publishing, over `Fakes`, the history fixture and a bare shared repository: not landed and unreadable landing (no other fake called); two components one touched, and one untouched answered unchanged; no projection and an unbound projection; one ensure for every output, and an output unmade; a failed build, a failing verdict alone, no verdicts (S13), and a build no verdict names refused naming both; no tag after a refusal and the tag before the first pack; two artifacts put and a store that names bytes wrongly; provenance attached and no identity at hand; verdicts carried byte for byte; nothing attested elsewhere; the order of puts and a stopped publish; the same revision again with nothing changed in the store, history or record (S2), and a projection packing differently refused naming the artifact with the name kept; the answer naming every digest (S1); and no keeper or target call.

## 4. The host's record and status

- [ ] 4.1 Write `crates/tailrace-core/src/published.rs`: `Kept::at`, `Entry`, `Kept::write` by temporary file and rename, an equal entry left untouched, and `Kept::read` with absent and unreadable kept apart (design D9); call `Kept::write` as publish's last write (D1 step 12). Verify with tests in `crates/tailrace-core/tests/status.rs`: an entry after a publish names what the spec says and holds no envelope or artifact digest; nothing kept after a refused or stopped publish; an entry parsed as plain JSON by `serde_json::from_str` on the file's text; a second publish leaves the file's bytes and modification time unchanged.
- [ ] 4.2 Write `crates/tailrace-core/src/status.rs`: `StatusAsk`, `Status` and `status(&Kept, &StatusAsk)`, which takes no binding (design D9). Verify with tests in `crates/tailrace-core/tests/status.rs`, one per scenario of spec host-record: status after a publish, with the fakes recording no call after the publish's last; every file of the record unchanged and none added; one revision's versions alone; two worktrees sharing one record; nothing kept answered absent naming the directory; a malformed entry unreadable with serde's words verbatim; versions ordered 1.9.0 before 1.10.0.

## 5. Verify

- [ ] 5.1 Write `crates/tailrace-core/src/verify.rs`: `VerifyAsk`, `Verified`, `Told` and `verify` as design D10 orders it. Verify with tests in `crates/tailrace-core/tests/verify.rs`, one per scenario of spec attestation-verification, each over a version published by `publish` in the test: the index digest and every artifact named; a version never published absent; the policy asked once for acceptance at the tag's revision after the landing; no policy for the audience absent; provenance by the host's signer and carried verdicts by the landing's; an expired signer marked expired and still counting where in reach; a verdict from a signer the policy does not name kept, not counted, with the signers checked, and still attached (S5); a signer beyond its reach not counting; a changed attestation from no named signer; claims-of unreadable carrying `connection refused` verbatim; and no put, attach, name or sign recorded and the record unchanged.
- [ ] 5.2 Rewrite `crates/tailrace-core/src/lib.rs`'s doc comment to list the new modules and the clauses each keeps, and re-export them. Verify `grep -rniwE 'oci|ecr|ghcr|oras|cosign|moon|sluice|kms|npm|maven|cargo|flux|gh' crates/tailrace-core/src` finds nothing (core.1).

## 6. The command line

- [ ] 6.1 In `crates/tailrace/` add `serde_json` from the workspace and `tempfile` as a dev-dependency; write `Cli`, `run` and `render.rs` in `src/lib.rs`, and `src/commands/publish.rs`, `verify.rs` and `status.rs` (design D11); rewrite `src/main.rs` to call `run` with no binding and keep `--version`'s line; rewrite `lib.rs`'s doc comment to say what the binary now does. Verify with tests in `crates/tailrace/tests/cli.rs`, one per scenario of spec command-line: `run --json` over `Fakes::seams()` printing the core's answer value for value; a refusal as one JSON document; exits 1, 4 and 3 with `connection refused` printed verbatim; `--version` given to publish exiting 2 with no fake called; the built binary's `status --json` over a record written in the test exiting 0; the built binary's `publish HEAD` exiting 4 naming the unbound seams with no tag, file or record written.
- [ ] 6.2 Verify `grep -rnE '\.(put_bytes|put_artifact|attach|claims_of|ensure|landed|verdicts_of|sign|policy_for|pack|render|append|chain)\(' crates/tailrace/src` finds nothing: the commands call the core's functions and no seam's (core.31).

## 7. The site

- [x] 7.1 Rewrite the paragraph of `site/src/content/docs/how-it-works.mdx` that begins "`tailrace publish` is the gate's move" to say that publish offers a landed revision's artifacts, each with its provenance and the landing's verdicts carried, the index last, and moves nothing; and that `tailrace verify` says which signer the policy names made each claim, and counts none another made (20, 25, 9). Verify the site's moon build task passes in the gate (8.1) and `grep -n "gate's move" site/src/content/docs/how-it-works.mdx` finds nothing.

## 8. The gate

- [ ] 8.1 Verify `devenv shell -- moon run ':#check-fast'` passes: format, lint, toolchain, ADR numbers, every crate's tests and the site's build.
