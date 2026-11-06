# Tasks

## 1. The Nix cache target's machinery

- [x] 1.1 Bring `change/the-nix-cache-target`'s built work into this branch before any code that reads `Policy::environments`, `Head`, `core::apply`, `timber apply`, the target family's `bound` or `nix_closure`. Take it from the bolt it landed on, or from its branch where it has not landed (design D1). Verify that `moon run ':#check-fast'` passes on the merged tree with nothing of this change written.

## 2. Reports and after channels in the seams and the policy file

- [x] 2.1 Make three changes to the seams (seam-contracts spec, design D9), and verify with `moon run seams:test`:
  - Add `ClaimKind::Report` to `crates/timber-seams/src/claim.rs`, under `https://agentplot.github.io/timber/claim/report/v1`.
  - Add `environment: Option<EnvironmentName>` to `RequiredClaim`, written only where it is set.
  - Add `after: Vec<EnvironmentName>` to `Moves::Bound` in `policy.rs`, skipped in JSON where it is empty.
  Give each a JSON round-trip test, and add a test that a bound channel written with no `after` reads back the same.
- [x] 2.2 Let the fake policy script a bound channel that comes after the environments a test names, and a policy for it requiring a report of each, with each call recorded and faultable (seam-fakes spec). Verify with a case in `crates/timber-fakes/src/policy.rs` under `moon run fakes:test`.
- [x] 2.3 In `crates/timber/src/profiles/policy/file/`, take `report` among a signer's `kinds`. List a channel an `after` makes with its environments in `after`'s order. Make its policy require, after acceptance and its line's claims, a report of each environment by every signer whose kinds hold `report`. Give the one host 108 admits no report (policy-file spec, design D7, D9). Verify with `tests.rs` cases for `eu` after `us`, the deployer that may report, nobody may report, and the one host, under `moon run cli:test`, and with the policy group over the profile.

## 3. A converged apply records a report

- [x] 3.1 Make `judge` in `crates/timber-core/src/ledger.rs` count a report requirement only through a report claim on the index that names the requirement's environment, verified against the signers the requirement names. A requirement naming no signer is refused naming it (environment-reports spec, design D9). Verify with `crates/timber-core/tests/` cases for a report of the right environment, one of another environment, one by a signer that may not report, and no signer, under `moon run core:test`.
- [x] 3.2 Add `newest` to a new `crates/timber-core/src/after.rs` (environment-reports spec, design D8). It reads the environment's channel chain, asks `claims_of` of each index the chain moved to, keeps the counted reports of that environment and component, and answers the one with the latest `at`. It answers absent where none counts, and unreadable naming the chain or the index it could not read. Verify with core tests for a rollback, no report, a report by an uncounted signer, and an unreadable `claims_of`, under `moon run core:test`.
- [x] 3.3 Make three changes to `crates/timber-core/src/apply.rs` (environment-reports spec, design D6), and verify with `crates/timber-core/tests/apply.rs` cases for converged, the report naming another digest, an unread report, already reported, a lost report made again, and a refused attach, under `moon run core:test`:
  - Once the target's report names the index applied, sign a report claim as the bound signer and attach it beside the index. Its body is environment, component, channel, version, index, runs, converged at and the ask's `at`.
  - Where current already runs the head, ask the target's report, and sign only where the newest report names another index or none.
  - Add `report: Answer<Digest>` to `Applied`.
- [x] 3.4 Bind the host's signer to sign for `For::Apply` in `crates/timber/src/bindings/mod.rs`, as publish and promote bind it, with the given host key made on first use where `[signing]` is absent, and every other signer profile verify-only (host-bindings spec). Verify with `bindings/tests.rs` cases under `moon run cli:test`.
- [x] 3.5 Make `timber verify` list a report claim with its signer and its environment, and add its line to `crates/timber/src/render.rs` (environment-reports spec). Verify with a `crates/timber/tests/cli.rs` case over the fakes under `moon run cli:test`.

## 4. A channel an after makes follows the reports

- [x] 4.1 Split `ledger::promote` so that its half after the asking, the chain head, `held_by`, `judge`, `may_promote`, sign, append and mirror, is one function that promote calls (design D9). Verify that `moon run core:test` and `moon run conformance:test` pass with every promotion scenario unchanged in what it asserts.
- [x] 4.2 Add `follow` and `FollowAsk` to `after.rs` (after-channels spec, design D9). For each channel in force bound and coming after the environment, it reads each named environment's newest report. It answers `Held` with each one's answer where any is absent or unreadable, or where they disagree. Otherwise it moves the channel to the one index through the shared function, as the gate's own act with the host's signer as asker and no `may_ask`. Verify with core tests for forward, back, two environments that disagree, an older report of a newer version, a host that is not a promoter, a claim the channel's line wants, and two channels with one refused, under `moon run core:test`.
- [x] 4.3 In `crates/timber/src/commands/apply.rs`, after the core's apply answers a report digest, read the bound `Teller` (after-channels and command-line specs, design D9). For `Publish`, call `follow` and put each channel's answer in `Applied.after`. For `Job`, carry the words. Render both in text and JSON, and take the worst exit, a held channel counting as refused. Verify with `cli.rs` cases over the fakes for the host's own gate, a job, and an unread report asking no gate, under `moon run cli:test`.
- [x] 4.4 Add the core group `after` in `crates/timber-conformance/src/core/after.rs` (conformance-suite spec, design D10). It holds:
  - "a converged apply reports", citing 11, 43 and 91;
  - "an unread report signs nothing", citing 43 and 91;
  - "the newest report across a rollback", citing 43 and 91;
  - "S16", playing `S16` and citing 37, 41, 43, 83, 91 and 97, over `first` manual on `stable` and `second` after `first`, with the never-ahead check after each step.
  Take 41 and 97 and the `bolt 6 · the clan push profile` constant out of `register.rs` in the same commit. Verify with `moon run conformance:test` and `moon run conformance:coverage`: 41 and 97 cited and registered nowhere, and no clause newly uncited.
- [x] 4.5 Refuse an asked move of a channel an `after` makes in `ledger::moved` (after-channels spec, design D9 and D12, rule core.72): where `Asked::By` meets a `Moves::Bound` whose `after` is not empty, answer refused `Why::MovesOnReports { channel, after }`, naming the channel and the environments whose reports the gate moves it on, before the chain is read and with nothing signed or appended. Verify with a `crates/timber-core/tests/follow.rs` case where `srv` reported 0.2.0 and then went back and reported 0.1.0, and an asked promotion of `second` to 0.2.0 by its own promoter is refused, under `moon run core:test`; with a fourth step in S16, the host asking `second` to 0.2.0 after `first` went back, refused and `never_ahead` after it, under `moon run conformance:test`; and with a `cli.rs` case where `timber promote 0.1.0 --to second` exits 1 naming `second` and `srv`, under `moon run cli:test`.

- [x] 4.6 Declare S16's `first` as the conformance-suite delta and design D10 name it, a manual environment on `stable` whose `approvers` name `desk`: in `crates/timber-conformance/src/core/after.rs`, `s16_world` declares `first` through `declared` with `waits` true and `approvers` `[desk]`, and `declares` keeps every other environment moving on its own. In the same commit, reword the module document of `crates/timber/src/bindings/mod.rs` to say that apply binds the host's signer to sign its report and the gate's moves on it, every other profile verify-only, with the store opened and never made (host-bindings delta). Verify with `moon run conformance:test` and `moon run ':#check-fast'`.

## 5. The clan push profile

- [x] 5.1 Create `crates/timber/src/profiles/target/nix_fleet/mod.rs` and `push/` (clan-push-target spec, design D2, D5), and verify with unit tests in the profile under `moon run cli:test`. It holds:
  - the `clan` table, `machine` and `closure` required, any other key unreadable naming it and the two the table reads;
  - the private fleet trait (`eval_toplevel`, `realise`, `update`, `read`, `mark`);
  - `Clan`, running `clan`, `nix` and `nix-store` from `PATH` and carrying their stderr verbatim;
  - a `Scripted` fleet for tests.
  Add the profile to the family's `bound` under `clan`. The unit tests cover a missing key, an extra key and both keys.
- [x] 5.2 Make `desired` answer the core's head, and make apply follow design D3: the index and its revision and repository; the closure artifact read by digest through `nix_closure` with exactly one root; the revision's worktree under `<common>/timber/fleet/`; the evaluated toplevel checked against the root; the root realised with substitution alone; one `clan machines update <machine> --flake <worktree> --build-host localhost --host-key-check strict` (clan-push-target spec). Verify with unit tests over `Scripted` for a closure of two roots, no artifact, a flake evaluating to another path, a path no substituter holds (clan not run), an update clan refuses (its words verbatim) and a converged update, each asserting which fleet commands ran, under `moon run cli:test`.
- [x] 5.3 Write the marker after a converged update, through `clan ssh` to a temporary file renamed into `${XDG_STATE_HOME:-$HOME/.local/state}/timber/markers/<owner>/<name>/<component>/<environment>.json`. Make `current` and `report` read `/run/current-system` and the marker in one `clan ssh`, as the spec answers them (clan-push-target spec, design D4). Verify with unit tests over `Scripted` for the marker's fields, a machine deployed by hand, no marker, an unreachable machine with ssh's words, a marker that cannot be written, and a read-back naming another closure, under `moon run cli:test`.
- [x] 5.4 Run the target group over the profile with `Scripted`, declaring that it holds a marker, in the profile's tests (clan-push-target and conformance-suite specs). Verify that every applicable scenario passes under `moon run cli:test`.
- [x] 5.5 Add `crates/timber/tests/nix_fleet_push.rs` behind the `tools` feature, as design D11 lays it out, and the `fleet` task to `crates/timber/moon.yml`, tagged `check-fleet`, with `$TIMBER_TOOLS_FLEET` among its inputs and a comment saying it runs outside the gate (core.38). The run refuses, naming `TIMBER_TOOLS_FLEET`, where the setting is unset. Verify by running `moon run cli:fleet` with the setting unset and seeing it fail naming the setting. It can only be run by hand: with the setting, it needs a machine and the host's key file to it, which no gate task reads (core.38). Also verify that `moon run ':#check-tools'` does not run it.

## 6. Records and the site

- [x] 6.1 Add to `docs/adr/` and `adrs.toml` record 0027, `a-pushed-machine-s-report-is-a-claim-beside-its-version-and-an-after-channel-follows-the-newest-reports`, in MADR form and proposed. It records design D3, D4, D6, D7, D8 and D9 with the paths not taken, and cites agentplot/blueprints `c118a95` and swancloud record 0043's words. Add rules core.71, core.72 and core.73 with their details to `docs/architecture/core.md`, as design D12 words them. Verify with `moon run root:adrs` and `moon run root:format`.
- [x] 6.2 Add the clan push profile to `site/src/content/docs/guides/integrations/targets.mdx`: its `target.clan` table, what apply runs and checks, the marker, the report and the `report` kind its signer needs, `after` with an example of an environment that waits and one after it, and that `timber apply` runs on the host that holds the machine's key. Mark the Nix fleet row built in the integrations index (design D12). Verify with `moon run site:build`.

## 7. Integration

- [x] 7.1 Run `moon run ':#check-fast'` and `moon run ':#check-tools'` in the devenv shell, and verify that both pass and that the coverage check reports 41 and 97 cited and no clause newly uncited.

## Workflow follow-up

- The operator runs `moon run ':#check-fleet'` by hand against a throwaway machine named in `TIMBER_TOOLS_FLEET`, before the change lands (core.38). The run's answer goes in the landing's description.
- The blueprints branch `design/timber-an-environment-after-another` (`c118a95`) is pushed and landed. Then `crates/timber-conformance/src/statement.rs` adopts the statement past it, in its own chore.
- swancloud adopts the profile in its own change: closure files from its machine tasks, its `.config/timber.toml` (no `after` for a Mac, per its record 0043), and its deployer's loop of `timber apply`.
