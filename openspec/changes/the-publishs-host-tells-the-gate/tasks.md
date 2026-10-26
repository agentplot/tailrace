# Tasks

Every cargo and moon command runs inside `devenv shell`. The fast layer
is `moon run ':#check-fast'`, the tools layer `moon run ':#check-tools'`.

## 1. The gate judges the index a publish told it of

- [x] 1.1 In `crates/tailrace-core/src/gate.rs`, make `ToldBy::Publish` carry `index: Digest` (design D3), and in `accept` answer unreadable naming the tag, the resolved digest and the told one where they differ, before the tag's revision is read; update the module's and `ToldBy`'s doc comments. Verify: `cargo test -p tailrace-core --test gate` passes with two new tests, one told the digest the name resolves to and one told another, which writes nothing (the fake store records no `attach`, the fake keeper no `append`).
- [x] 1.2 Update every builder of `AcceptAsk` to the new `ToldBy::Publish { index }`: `crates/tailrace-core/tests/gate.rs`, `crates/tailrace-conformance/src/world.rs`, `crates/tailrace-conformance/src/core/gate.rs`. Verify: `moon run core:test conformance:test` passes with no scenario's outcome changed.
- [x] 1.3 Below the gate group's marker in `crates/tailrace-conformance/src/core/gate.rs`, add "the core starts nothing after an offer" and "a told index the name does not resolve to" (conformance-suite spec), each citing 95 and 130. Remove 130's entry from `crates/tailrace-conformance/src/register.rs`. Verify: `moon run conformance:test` passes, and the coverage check reports 130 cited and not pending.

## 2. The bindings file names what tells the gate

- [x] 2.1 In `crates/tailrace/src/bindings/mod.rs`, read `[gate]` through the generic one-key reader into `Teller` (`Publish`, `Job(String)`, design D1, D4): `publish` takes only an empty table, `job` a string with words in it, any other key is `Table::unknown` naming `publish` and `job`. Add `gate: Teller` to `Binding` in `crates/tailrace/src/lib.rs`, set by `bind` for every command, and update the module doc's example and table list. Verify: `cargo test -p tailrace bindings` passes with a test for each scenario of the host-bindings spec, including that a broken `[gate]` leaves the layout unmade.
- [x] 2.2 Set `gate` in the test bindings of `crates/tailrace/tests/cli.rs` and `crates/tailrace/tests/conformance.rs`. Verify: `moon run cli:test` passes.

## 3. Publish tells the host's gate

- [x] 3.1 In `crates/tailrace/src/commands/publish.rs`, after the core's publish answers, ask `accept` for each `Published::Version` where the binding's teller is `Publish`, with `ToldBy::Publish { index }`, the binding's signer as `gate`, and `when` from the clock as `promote` reads it; carry the job's words where it is `Job` (design D2, D5). Render each version's `gate` member in JSON and its `gate:` line in text, and compute the exit code as D5 says. The core's `publish` stays unchanged. Verify: the tests of 3.2 pass.
- [x] 3.2 Add `crates/tailrace/tests/publish_tells_the_gate.rs` (check-fast, over the fakes through `tailrace::run`), one test per scenario of the command-line spec that the fakes can play: one host accepted with `beta` promoted; the gate refusing what the host offered; a channel held; accepted with no ledger; unchanged not told; the JSON `gate` member value for value against `accept`'s own answer; a job; each with its exit code and the fake keeper's and store's calls. Verify: `moon run cli:test` passes.
- [x] 3.3 Add `crates/tailrace/tests/one_host.rs` behind the `tools` feature (design D7): the built binary on a plain-branch scratch repository built by moon, a host key named for provenance, acceptance and promotion, an OCI layout, the git ref ledger and `[gate] publish = {}`; two landings each followed by one `tailrace publish`, then `beta`'s chain, its predecessor link, the layout's name for `beta` and each acceptance saying one host. Add `--test one_host` to the `tools` task in `crates/tailrace/moon.yml`. Verify: `moon run ':#check-tools'` passes.

## 4. Records and the guide

- [x] 4.1 Write `docs/adr/0019-the-host-names-what-tells-its-gate-and-its-publish-step-tells-its-own.md` (or the next free number when 0019 is taken) recording design D1 to D3 and extending ADR 0013's table list, in the form `adrs.toml` sets. Add to `docs/architecture/core.md`, in core.10's details, "Not ruled out: the host's publish step telling the host's own gate of the publish's answer where the host's bindings file names it, whose judgment and moves are the gate's (95, 108, 130)", with the new ADR in its source. Verify: `moon run ':#check-fast'` passes, the records' numbers check among it.
- [x] 4.2 In `site/src/content/docs/guides/self-hosting.mdx`, show `[gate] publish = {}` in the laptop's bindings beside the one-host policy, saying `beta` then moves on each publish in the same run; and in the team section, show `[gate] job = "…"` on hosts that only offer. Verify: `moon run site:build` passes.

## 5. A file naming no teller tells the host's own gate

- [ ] 5.1 In `crates/tailrace/src/bindings/mod.rs`, make `Teller::of` answer `Teller::Publish` where the file holds no `[gate]`, the host's default (118, design D1, D4), and remove `Teller::Nothing` and the absence it carried; update `Teller`'s and the module's doc comments to say a file with no `[gate]` binds the host's publish. In `crates/tailrace/src/bindings/tests.rs`, the no-`[gate]` test asserts the binding is `Teller::Publish`, the same as `publish = {}`. Verify: `cargo test -p tailrace bindings` passes.
- [ ] 5.2 In `crates/tailrace/src/commands/publish.rs`, remove the path that renders the absence of a teller, in JSON and in text (design D5). In `crates/tailrace/tests/cli.rs` and `crates/tailrace/tests/conformance.rs`, set the test bindings' `gate` to `Teller::Job` with words of their own, and make the CLI test that compares publish's JSON with the core's take the `gate` member as the job's object. Verify: `moon run cli:test` passes.
- [ ] 5.3 In `crates/tailrace/tests/publish_tells_the_gate.rs`, replace the tests of a file with no `[gate]` by one that binds the teller a file with no `[gate]` binds (read through `bindings::bind` from a file holding none) and checks the run's answer, exit code, acceptance and `beta`'s chain match those under `publish = {}`; and make the JSON test of a host not telling the gate the job's (command-line spec, "The JSON answer says so"). Verify: `moon run cli:test` passes.
- [ ] 5.4 In `crates/tailrace/tests/one_host.rs`, make the no-`[gate]` test show `beta` moving on each of two landings exactly as with `publish = {}`, each acceptance saying one host (design D7), and update the module's doc comment. Verify: `moon run ':#check-tools'` passes.
- [ ] 5.5 In `docs/adr/0019-the-host-names-what-tells-its-gate-and-its-publish-step-tells-its-own.md`, say that a file with no `[gate]` binds the host's publish to tell its own gate (118), give the rejected alternative of design D1, and say in the consequences that a host that only offers names its `job`, and that a host with no bindings file is left to the change that binds 118's defaults, this teller among them; remove the consequences that said a file or a host naming nothing tells no gate. In `site/src/content/docs/guides/self-hosting.mdx`, say a laptop's file needs no `[gate]` for `beta` to move, and that a host that only offers must name its `job`, since without one its publish asks its own gate. Verify: `moon run ':#check-fast'` passes, the records' numbers check and the site's build among it.

## 6. The whole

- [ ] 6.1 Run `moon run ':#check-fast'` and `moon run ':#check-tools'` and verify both pass.
