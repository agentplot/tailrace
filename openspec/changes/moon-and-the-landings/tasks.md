# Tasks

Every cargo and moon command runs as `devenv shell -- <command>`. This
change builds on `the-conformance-suite-over-the-fakes` as its code
stage lands it, with the landing rig that stages `Recorded` verdicts
(its design D3a): before group 7, read
`crates/tailrace-conformance/src/seams/landing.rs`, `seams/build.rs`,
`src/scenario.rs`, `src/fakes.rs` and `src/register.rs` as they stand,
and take the names that landed where this design's differ. Before group
4, read sluice v0.37.0's `crates/sluice/src/profiles/remote_refs.rs`,
`local_files.rs`, `signing/dsse.rs`, `signing/statement.rs`,
`signing/ssh.rs`, `sluice-core/src/records.rs` and `text.rs`, and its
`crates/sluice/tests/peers.rs` (design Context). The profiles name no
word core.1 lists outside `crates/tailrace/src/profiles/` and the
tests.

## 1. The envelope and the verdict

- [x] 1.1 In `crates/tailrace-seams/src/claim.rs`, make `Envelope` hold `signatures: Vec<Signature>` and `Signature` a `keyid`, with `Envelope::signed` and `Envelope::sole`, and make `Claim::from_envelope` answer `None` unless the envelope holds exactly one signature (design D1); say so in `signer.rs`'s doc of `verify`. Verify with unit tests in `claim.rs`: an envelope of two signatures with key ids round-trips through JSON in order; a key id left empty is not written; a claim of tailrace's form with two signatures, or none, reads back as no claim; the existing tests pass with `Envelope::signed`.
- [x] 1.2 In `crates/tailrace-seams/src/landing.rs`, add `found: String` to `Verdict`; in `crates/tailrace-core/src/provenance.rs`, add `at: String` to `Found`, filled from the verdict's `found` by `Carried::from_landing` (design D1). Verify with a unit test in `provenance.rs`: a carried verdict's `found.at` is the verdict's `found`, and its envelope with two signatures reads back byte for byte with both key ids.
- [x] 1.3 Make every other `Envelope`, `Signature` and `Verdict` literal follow: `crates/tailrace-fakes/src/` (the signer's `sign` through `Envelope::signed`, its `verify` naming a named signer one of whose signatures verifies; the landing, the keeper, the store), `crates/tailrace-core/src/publish.rs` and `tests/`, and `crates/tailrace-conformance/src/` and `tests/`. Verify `devenv shell -- moon run ':#check-fast'` passes, every scenario and test unchanged in what it asserts.

## 2. The profiles' module path and the one way they run a tool

- [x] 2.1 Add `pub mod build;`, `pub mod landing;` and `mod run;` to `crates/tailrace/src/profiles/mod.rs` (beside the layout store's `pub mod store;`, making the file and `pub mod profiles;` in `crates/tailrace/src/lib.rs` if that unit has not landed), and write `profiles/build/mod.rs` and `profiles/landing/mod.rs` (design D2). Verify `devenv shell -- cargo build -p tailrace` passes.
- [x] 2.2 Write `profiles/run.rs`: `Tool` and its `output` answering unreadable with the spawn's or the command's own words verbatim (design D2). Verify with unit tests: a program that does not exist answers unreadable carrying `No such file or directory (os error 2)` as the operating system words it; `git -C <a missing directory> rev-parse --git-common-dir` answers unreadable carrying git's stderr trimmed and unchanged; an exit the caller allows answers its output.

## 3. moon

- [x] 3.1 Write `profiles/build/moon/mod.rs` with the reading of `[build.outputs.<name>] task` from the policy file at the revision (design D3), and `profiles/build/moon/checkout.rs` with the revision's worktree under the common directory (design D4). Verify with unit tests over a scratch git repository: the task named at a revision, not the one an uncommitted edit names; an output with no table names no task; the checkout holds the revision's tree and is reused for it; a checkout's own uncommitted file is not in it.
- [x] 3.2 Write `profiles/build/moon/report.rs`: reading moon's run report (actions, `hash-generation`, status) and a task's declared outputs from `moon task <target> --json` (design D4). Verify with unit tests over run reports captured from moon 2.6.1 (passed, failed, cached, an action that is not `RunTask`), and the paths an output glob matches.
- [x] 3.3 Implement `Build` for `Moon`: `ensure` and `report` (design D4). Verify in `crates/tailrace/tests/build_moon.rs` (task 7.1).

## 4. sluice

- [x] 4.1 Write `profiles/landing/sluice/dsse.rs`: an envelope read from sluice's DSSE JSON into the seams' `Envelope` and written back (design D7); add `base64` to `[workspace.dependencies]` in `Cargo.toml` and to `crates/tailrace/Cargo.toml`. Verify with unit tests over an envelope captured from sluice v0.37.0's own tests' shape: payload, type, key ids and signatures read and written back byte for byte; text that is not base64 or not a DSSE envelope is an error naming why.
- [x] 4.2 Write `profiles/landing/sluice/log.rs`: the branch's log read from the remote's state commit or the host's files, and the landings it holds done (design D5). Verify with unit tests over log lines: a landing done, one begun and not done, two landings, lines of other events and fields ignored.
- [x] 4.3 Write `profiles/landing/sluice/records.rs`: the run statement, the records under a combination, and the replay search (design D6). Verify with unit tests over statements: results as verdicts with check, key, outcome and signer; a statement naming another combination left out; a replay carried as the ran passes for its key and system, not one on another system; a malformed record unreadable naming its ref and path.
- [x] 4.4 Write `profiles/landing/sluice/mod.rs`: `Sluice`, its opening listing, and `Landing` for it (design D2, D5, D6). Verify in `crates/tailrace/tests/landing_sluice.rs` (task 7.2).

## 5. A plain branch

- [ ] 5.1 Write `profiles/landing/branch/mod.rs`: `PlainBranch` and `Landing` for it (design D8). Verify in `crates/tailrace/tests/landing_branch.rs` (task 7.3).

## 6. The tools layer

- [ ] 6.1 In `devenv.nix`, add sluice v0.37.0's release binary fetched as proto's is, each asset's sha256 got with `nix-prefetch-url`, and `pkgs.openssh`; add `--all-features` to the commit hook's clippy (design D9, D10). Verify `devenv shell -- sluice --version` prints `sluice 0.37.0` and `devenv shell -- which ssh-keygen` names a path in the Nix store.
- [ ] 6.2 Add the `tools` task tagged `check-tools` to `crates/tailrace/moon.yml`, `--all-features` to the root `moon.yml`'s `lint`, `#check-tools` to `required` on `bolt/*` and `main` in `.config/sluice.toml`, and the layer to `.moon/workspace.yml`'s comment and `CLAUDE.md`'s layers table (design D9, D10). Verify, once group 7 is written, that `devenv shell -- moon run ':#check-tools'` runs the three tools test files and passes, and that `devenv shell -- moon run ':#check-fast'` runs none of them.

## 7. The rigs, the suite and the end-to-end tests

- [ ] 7.1 Write `crates/tailrace/tests/scratch/mod.rs`'s repository and moon workspace (design D9), and `tests/build_moon.rs` under `#![cfg(feature = "tools")]` with `[features] tools = []` in `crates/tailrace/Cargo.toml`: `MoonRig`, the suite's build group over it, and one test per moon-build spec scenario the group does not play (an output and its task, an output with no task, a checkout's edit, three outputs in one asking, the same revision again, a failed task's report, nothing ensured, two checkouts, moon not there). Verify `devenv shell -- cargo test -p tailrace --features tools --test build_moon` passes, the group's report naming `moon` with every scenario passed.
- [ ] 7.2 Add the recorder to `tests/scratch/` and write `tests/landing_sluice.rs` under the `tools` feature: `SluiceRig`, the suite's landing group over it, one test per sluice-landing spec scenario, and the real landing by `sluice submit main` (design D9). Verify `devenv shell -- cargo test -p tailrace --features tools --test landing_sluice` passes, the group's report naming `sluice` with every scenario passed.
- [ ] 7.3 Write `tests/landing_branch.rs`, in the fast layer: `BranchRig`, the suite's landing group over it, and the branch-landing spec's scenarios but S13. Verify `devenv shell -- cargo test -p tailrace --test landing_branch` passes, the report naming `branch`, verdicts as the landing recorded them not applicable with its reason, and every other scenario passed.
- [ ] 7.4 Below the landing group's marker in `crates/tailrace-conformance/src/seams/landing.rs`, add *two signers, each by its own name* citing 104, and take 104's entry out of `src/register.rs` (design D11). Verify `devenv shell -- cargo test -p tailrace-conformance` passes with the fake rig passing it and the recording-none rig reporting it not applicable, and `devenv shell -- moon run conformance:coverage` passes with 104 no longer printed pending.
- [ ] 7.5 Write `tests/landings_end_to_end.rs` under the `tools` feature: the join, the refusal of an output no verdict names, and S13 over the plain branch and moon (design D9). Verify `devenv shell -- cargo test -p tailrace --features tools --test landings_end_to_end` passes.
- [ ] 7.6 In `site/src/content/docs/guides/integrations/builds.mdx`, mark moon `built` and say in its row that the repository names each output's task in its policy file and that ensure runs in a checkout of the revision; in `landings.mdx`, mark sluice and the plain branch `built`, and say in sluice's row that a replayed result is carried as the signed pass that ran and every envelope is carried whole. Verify the site's moon build task passes in the gate (8.1).

## 8. The gate

- [ ] 8.1 Verify `devenv shell -- moon run ':#check-fast'` and `devenv shell -- moon run ':#check-tools'` pass: format, lint with every feature, toolchain, ADR numbers, the suite's coverage, every crate's tests, the site's build, and the tools layer.
- [ ] 8.2 Verify `grep -rn 'tailrace_core' crates/tailrace/src/profiles` finds nothing (core.2), and `grep -rniwE 'oci|ecr|ghcr|oras|cosign|moon|sluice|kms|npm|maven|cargo|flux|gh' crates/tailrace/src crates/tailrace-seams crates/tailrace-core/src crates/tailrace-fakes crates/tailrace-conformance/src --exclude-dir=profiles` finds nothing new (core.1).
