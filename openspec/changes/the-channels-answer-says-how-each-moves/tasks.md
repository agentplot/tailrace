# Tasks

Every cargo and moon command runs inside `devenv shell --`. A task's
verification is the moon task named, which the gate runs under
`check-fast`: `seams:test`, `fakes:test`, `core:test`, `cli:test`,
`conformance:test`, `conformance:coverage` and `site:build`. The
workspace builds only once every implementor and caller of the policy,
keeper and target seams takes the new types, so groups 2 to 5 land in
one commit.

## 1. The bolt

- [x] 1.1 Take `bolt/tailrace-the-gate-and-the-channels` into this branch (it holds `the-ledger-and-promotion`, which this unit builds on; design, Context), committing the merge by itself; verify with `moon run ':#check-fast'` passing before anything of this unit is changed

## 2. The seam

- [x] 2.1 Add `ChannelId` to `crates/tailrace-seams/src/artifact.rs` as design D1 gives it, with `spelled()`, `Display` and `component_name()`; test in `seams:test` that `beta` of no component and of `lib` spell `beta` and `lib/beta`, are unequal, and round-trip (spec `seam-contracts`, a channel spelled)
- [x] 2.2 In `crates/tailrace-seams/src/policy.rs`, make `Audience::Channel` take a `ChannelId`, add `Moves` and make `DeclaredChannel { channel, moves }` with `bound_to()` (design D2), and add `promoted_by` to `Required::Channel` with `Required::promoted_by()` (design D3); doc comments cite 131, 125, 37, 83, 32; replace the module's tests so each `Moves` round-trips and its JSON is `{"by": ...}` as D2 writes it, and a declared channel names its component or null; verify with `seams:test`
- [x] 2.3 In `crates/tailrace-seams/src/keeper.rs`, make `append` and `chain` take `&ChannelId` and `channel_name` spell `channel/<spelled>`; in `answer.rs` make `Why::ChannelAt` and `Why::NeverHeld` name `ChannelId`s, rendered spelled; in `target.rs` make `Environment.channel` a `ChannelId`; update `lib.rs`'s `Nowhere` and its every-seam test; test in `seams:test` that `stable` of `lib` is named `channel/lib/stable` and of no component `channel/stable`, and that each refusal round-trips (spec `seam-contracts`, a channel's name in the store)

## 3. The fakes

- [x] 3.1 In `crates/tailrace-fakes/src/policy.rs`, drop `belongs` and `belongs_at`, add `follows` and `follows_at`, and list channels as design D8 says; replace the channels tests with the spec's scripted channels (each publish, promotion alone, bound, at one revision and every revision) and keep the faulted-and-recorded test; verify with `fakes:test` (spec `seam-fakes`, the fake policy's channels)
- [x] 3.2 Key `crates/tailrace-fakes/src/keeper.rs`'s chains, answered copies and `fork` by `ChannelId`, and `target.rs`'s environments' channels; add a test that `beta` of `lib` appended leaves `beta` of `web` and of no component absent; verify with `fakes:test` (spec `seam-fakes`, the fake keeper's two components)

## 4. The core

- [x] 4.1 In `crates/tailrace-core/src/gate.rs`, remove `ChannelKeys`, `ChannelTables`, `channel_tables` and the use of `POLICY_FILE`; find following channels and the gate's named signer through `channels_at` and the policy for each listed channel, and check the gate's identity against `promoted_by` before a following channel moves, as design D5 says; update `crates/tailrace-core/tests/gate.rs`, whose file-parsing tests become tests over the fake policy: `beta` given by the fake moving on publish, two components' `beta` with only the published one moving, a following channel whose promoters exclude the gate refused while the acceptance stands, an unreadable channels answer accepting nothing; verify with `core:test` (spec `acceptance`, the channels that follow each publish come from the policy seam)
- [x] 4.2 In `crates/tailrace-core/src/ledger.rs`, find the channel in the channels answer for the chosen component, absent naming both where none is, and key the policy read, the chain, the append, `mirror`, `channel` and `held_by` by its `ChannelId`; take the binding from `Moves::Bound` and the promoters from `promoted_by` (design D6); take `ChannelId` in `repair.rs`; give `verify.rs` a channel audience the version's component; update `crates/tailrace-core/tests/ledger.rs`, `verify.rs` and the rest of `tests/` to the new types, adding: a channel of another component absent, a given `stable` promoted by a signer named only for acceptance, a line's promotion signer refused; verify with `core:test` (spec `promotion`)

## 5. The profiles and the commands

- [x] 5.1 Rewrite `crates/tailrace/src/profiles/policy/file/mod.rs`'s channel reading as design D4's six steps, one `resolve` shared by `channels` and `policy_for`, answering `promoted_by` and given claims as D3 says; update the module doc and its TOML example to the per-component form, a given channel, a line under one and a manual environment; verify with `cli:test`
- [x] 5.2 Add to `crates/tailrace/src/profiles/policy/file/tests.rs` one test for each scenario of spec `policy-file`'s modified and added requirements and spec `channel-movement`'s file-level ones: the per-component form, the flat form and the old `component` key refused naming the form, givens with no file and with no `[channels]`, a line adding a claim, a line rebinding a given channel, a hand-declared channel each way and with both keys, the gate's signers as promoters, a manual and an after environment, after over two channels, a cycle, an unknown `policy`, a made channel named `beta`, an environment's other keys not read; verify with `cli:test`
- [x] 5.3 In `crates/tailrace/src/profiles/ledger/git_ref/mod.rs` and `store/mod.rs`, spell refs, fetched refs, the store name and the kept copy's path by `ChannelId` and match a claim's body by name and component (design D7); add to `crates/tailrace/tests/ledger_git_ref.rs` that `beta` of `lib` is held at `refs/tailrace/ledger/lib/beta` alone, and to `ledger_store.rs` that two components' `beta` walk apart; verify with `cli:test` (specs `git-ref-ledger`, `store-ledger`)
- [x] 5.4 Bring `crates/tailrace/src/commands/promote.rs`, `commands/verify.rs`, `bindings/tests.rs` and `crates/tailrace/tests/` (`promote.rs`, `cli.rs`, `policy_file.rs` and the rest naming a channel) to the new types and the per-component file form, adding to `promote.rs` a given `stable` promoted with no `[channels]` written; verify with `cli:test`

## 6. The suite

- [x] 6.1 In `crates/tailrace-conformance/src/seams/policy.rs`, add `given` to `PolicyRig`, make `declaring` take channels with how each moves, and change the channels scenarios below the marker to expect the declared and the given together, by name then component, with names other than `beta` and `stable`; add "each way a channel moves" below the marker (design D9); implement both on `FakePolicyRig` (`src/fakes.rs`) and the policy file's rig (`crates/tailrace/tests/policy_file.rs`, writing the per-component form); verify with `conformance:test` and `cli:test`, each report naming no scenario not applicable (spec `conformance-suite`, the policy's contract)
- [ ] 6.2 Add "two components' channels of one name" below the keeper group's marker in `src/seams/keeper.rs`, and bring the keeper rigs (`src/fakes.rs`, `crates/tailrace/tests/ledger_git_ref.rs`, `ledger_store.rs`) to `ChannelId`; verify with `conformance:test` and `cli:test` (spec `conformance-suite`, the keeper's contract)
- [ ] 6.3 Add "beta of one component" to `src/core/gate.rs` and "stable promoted by the gate's signer" to `src/core/ledger.rs` below their markers, with design D9's citations, and bring `world.rs`, `core/` and `tests/fakes.rs` to the new types; verify with `conformance:test` and `conformance:coverage`, the register unchanged and no clause both cited and registered (spec `conformance-suite`, the given channels over the fakes)

## 7. The record and the guide

- [ ] 7.1 Write `docs/adr/0019-a-channel-is-its-component-and-name-and-the-policy-says-how-it-moves.md` in MADR form, status proposed, recording design D1 to D5 with the options each weighed and the sources (125, 131, 32, 37, 95, C.10, C.13, C.14 at agentplot/blueprints 7749369; ADRs 0009, 0010, 0016; rules core.1, core.2, core.7, core.37); verify with the root project's `adrs` task (`moon run :adrs`) passing, no two records sharing a number
- [ ] 7.2 In `site/src/content/docs/guides/self-hosting.mdx`, say that every component has `beta` and `stable` with no line written and who promotes them, show a line under `stable`, the per-component `[channels.<component>.<name>]` form, a hand-declared channel each way it moves, and a manual and an after environment with the channels they make; in `guides/integrations/ledgers.mdx` and `how-it-works.mdx`, give the ref and the store name with the component where components are declared; verify with `site:build`

## 8. The gate

- [ ] 8.1 Run the fast layer, `moon run ':#check-fast'`, and verify every task passes, format and lint among them
