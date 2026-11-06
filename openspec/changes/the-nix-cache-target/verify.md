# Verify: the-nix-cache-target

The work on `change/the-nix-cache-target` at `d7c336e`, cut from
`bolt/tailrace-the-nix-fleet-pushed` at `86456fe`, checked against this
change's proposal, design, eight delta specs and tasks. It holds: every
task is ticked done, every requirement of the deltas is met in the code,
each scenario has a test that exercises it, and the checks the tasks
name pass.

## The checks

| check | answer |
|---|---|
| `openspec validate the-nix-cache-target --strict` | valid |
| `moon run ':#check-fast'` | exit 0, 20 tasks: format, lint, toolchain pins, the records' numbers, every crate's tests, the conformance suite over the fakes and the site's build |
| `moon run ':#check-tools'` | exit 0; moon replayed `cli:tools` from its cache at these inputs, every test binary passing, `nix_cache` among them |
| `cargo test -p timber --features tools --test nix_cache`, run again uncached | 1 passed: the closure pushed through `NixCache` to the test's atticd and realised into a fresh store from the cache alone, then the target group over the profile's rig with "converged and reported" alone not applicable |
| the coverage check, `cargo run -p timber-conformance --bin coverage` | 140 clauses, 127 cited, 13 pending; 42, 45, 119 and 136 cited and in no entry; 41 and 97 pending with `bolt 6 · the clan push profile`; 44 and 126 with `phase 2`; 98 and the parts of 132 and 134 with `phase 2 · environments in status`; no entry says phase 1 leaves every target to phase 2 |

## The tasks

All of 1.1 to 11.3 are ticked, and each one's Verify holds: the seams'
round trips and the two-target binding (1.1, 1.2); the fakes' unit tests
for environments, the head handed, `holds_no_marker` and `reads` (2.1,
2.2); the policy file's `target` and environments tests, and the policy
group over the file profile's rig (3.1 to 3.3); `bindings/tests.rs`
binding `nix_cache` for apply through a store never made, and refusing
`[targets]` and `[environments]` as the repository's (4.1); the core's
`tests/apply.rs` over the fakes (5.1, 5.2); `commands/apply.rs`'s tests
(6.1); `nix_closure.rs`'s tests for one root, two, an empty file, an
empty line, a repeated line, `result`, a path outside the store
directory, a hash of 31 characters, characters outside Nix's base32 and
an output the index lacks (7.1); the profile's unit tests with stand-in
programs (8.1 to 8.4); the suite's scenarios, each clause leaving the
register in the commit that first cites it, 136 in `bdff606`, 45 and 119
in `ed71dcb`, 42 in `0e0c803` (9.1 to 9.4, rule core.33); attic pinned in
`devenv.nix` and the proof in the `tools` task (10.1 to 10.3); the site's
targets page and index, record 0026, proposed, and rule core.70 (11.1 to
11.3).

## The specs

- **environment-apply.** `timber-core/src/apply.rs` follows design D5's
  order: the landed head and component, the environment among those the
  policy answers there, its target table and the bound profile, the
  chain's head through the ledger (rule core.50), `desired` handed it and
  checked against it, `current`, the judgment of the head's index under
  the channel's policy, and only then `Verified` handed to `apply`, then
  `report`. Eleven of its twelve scenarios have a test in
  `timber-core/tests/apply.rs`, the fake target's recorded calls showing
  no apply before the claims are read; "No store made" is
  `commands/apply.rs`'s `apply_makes_no_store` and the bindings' test of
  apply. A chain the ledger answers as last read is unreadable and the
  target asked nothing (60).
- **nix-cache-target.** The profile under
  `profiles/target/nix_cache/`, bound as `nix_cache`; its table reads
  `attic` as `<server>:<cache>` and `closures`, any other key unreadable
  naming the environment and the key at every operation; the closure's
  form read in `nix_closure.rs` alone; `apply` reads the index and each
  closure file by digest, checks every root with `nix-store
  --check-validity --print-invalid`, refusing one this host lacks naming
  it and the host before attic runs, and runs `attic push
  <server>:<cache> <roots…>` with no token and none of attic's variables;
  `current` and `report` unread naming the cache. The tools-layer proof
  asserts attic's command line, its environment as timber set it, and
  every file timber wrote hold no token. Attic, Nix and store paths are
  named in no source of the core, the seams, the fakes or the suite.
- **seam-contracts.** The policy trait answers `environments`; `desired`
  takes `Head`; `Environment` carries `settings`; `Seams.targets` is keyed
  by `TargetProfile`.
- **policy-file.** `target` read as one key whose value is a table,
  unreadable naming the table and `target` otherwise, in the pass that
  reads the channels; environments answered from the table in force,
  ordered by component and then name, a waiting or `after` environment
  following its made channel.
- **host-bindings.** An apply binds every target profile built, each
  through a store binding of its own opened and never made; `[targets]`
  and `[environments]` in the host's file are unreadable for every
  command, naming the table as the repository's to bind.
- **command-line.** `timber apply <environment>` with `--branch`,
  `--component` and `--json` renders the core's answer; absent, refused
  and unreadable exit apart; the given layout is not made.
- **seam-fakes.** The fake policy's environments set, ordered, recorded
  and faultable; the fake target answers the head handed, holds no marker
  after `holds_no_marker`, and after `reads` answers unreadable naming
  the first other key, applying nothing.
- **conformance-suite.** The target group's rig declares
  `holds_marker()`, "converged and reported" and "applied where no marker
  is held" each not applicable to the other, and "a table with a key the
  target does not read" added; the policy group's "environments declared"
  and "none declared"; the core group apply with its five scenarios
  citing 31, 40, 42, 88, 90 and 136; the binding group's two targets by
  profile; the register as above.

## Observations, none blocking

- seam-contracts, "Two environments of one profile": the seams' test
  finds one target per profile for environments of two components, and
  the core's tests show an environment handed its own table's settings;
  no one test applies two environments of one profile with different
  settings. The core reads each environment's settings from its own
  `TargetTable` (`apply.rs`, step 2), so the scenario holds by
  construction.
- The target group's "a table with a key the target does not read"
  checks that nothing was applied only where the rig holds a marker,
  since a target that holds none cannot answer it through the seam. For
  the Nix cache target the profile's own test,
  `a_token_in_the_table_is_unreadable_at_every_operation_and_attic_is_not_run`,
  shows attic never runs.
- The target group changes two scenarios above its "a later unit adds
  below" line: "what an environment should run" hands `desired` a head,
  and "converged and reported" is not applicable where no marker is
  held. The conformance-suite delta modifies both, so the change is the
  spec's.
- Record 0026 is proposed, for the user to accept with the change.
