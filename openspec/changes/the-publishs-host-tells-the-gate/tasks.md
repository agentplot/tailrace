# Tasks

Every cargo and moon command runs inside `devenv shell`. The fast layer
is `moon run ':#check-fast'`, the tools layer `moon run ':#check-tools'`.

## 1. The gate judges the index a publish told it of

- [ ] 1.1 In `crates/tailrace-core/src/gate.rs`, make `ToldBy::Publish` carry `index: Digest` (design D3), and in `accept` answer unreadable naming the tag, the resolved digest and the told one where they differ, before the tag's revision is read; update the module's and `ToldBy`'s doc comments. Verify: `cargo test -p tailrace-core --test gate` passes with two new tests, one told the digest the name resolves to and one told another, which writes nothing (the fake store records no `attach`, the fake keeper no `append`).
- [ ] 1.2 Update every builder of `AcceptAsk` to the new `ToldBy::Publish { index }`: `crates/tailrace-core/tests/gate.rs`, `crates/tailrace-conformance/src/world.rs`, `crates/tailrace-conformance/src/core/gate.rs`. Verify: `moon run core:test conformance:test` passes with no scenario's outcome changed.
- [ ] 1.3 Below the gate group's marker in `crates/tailrace-conformance/src/core/gate.rs`, add "the core starts nothing after an offer" and "a told index the name does not resolve to" (conformance-suite spec), each citing 95 and 130. Remove 130's entry from `crates/tailrace-conformance/src/register.rs`. Verify: `moon run conformance:test` passes, and the coverage check reports 130 cited and not pending.

## 2. The bindings file names what tells the gate

- [ ] 2.1 In `crates/tailrace/src/bindings/mod.rs`, read `[gate]` through the generic one-key reader into `Teller` (`Publish`, `Job(String)`, `Nothing(Not)`, design D1, D4): `publish` takes only an empty table, `job` a string with words in it, any other key is `Table::unknown` naming `publish` and `job`. Add `gate: Teller` to `Binding` in `crates/tailrace/src/lib.rs`, set by `bind` for every command, and update the module doc's example and table list. Verify: `cargo test -p tailrace bindings` passes with a test for each scenario of the host-bindings spec, including that a broken `[gate]` leaves the layout unmade.
- [ ] 2.2 Set `gate: Teller::Nothing(…)` in the test bindings of `crates/tailrace/tests/cli.rs` and `crates/tailrace/tests/conformance.rs`. Verify: `moon run cli:test` passes.

## 3. Publish tells the host's gate

- [ ] 3.1 In `crates/tailrace/src/commands/publish.rs`, after the core's publish answers, ask `accept` for each `Published::Version` where the binding's teller is `Publish`, with `ToldBy::Publish { index }`, the binding's signer as `gate`, and `when` from the clock as `promote` reads it; carry the job's words or the absence otherwise (design D2, D5). Render each version's `gate` member in JSON and its `gate:` line in text, and compute the exit code as D5 says. The core's `publish` stays unchanged. Verify: the tests of 3.2 pass.
- [ ] 3.2 Add `crates/tailrace/tests/publish_tells_the_gate.rs` (check-fast, over the fakes through `tailrace::run`), one test per scenario of the command-line spec that the fakes can play: one host accepted with `beta` promoted; the gate refusing what the host offered; a channel held; accepted with no ledger; unchanged not told; the JSON `gate` member value for value against `accept`'s own answer; no `[gate]`; a job; each with its exit code and the fake keeper's and store's calls. Verify: `moon run cli:test` passes.
- [ ] 3.3 Add `crates/tailrace/tests/one_host.rs` behind the `tools` feature (design D7): the built binary on a plain-branch scratch repository built by moon, a host key named for provenance, acceptance and promotion, an OCI layout, the git ref ledger and `[gate] publish = {}`; two landings each followed by one `tailrace publish`, then `beta`'s chain, its predecessor link, the layout's name for `beta` and each acceptance saying one host; and the same host without `[gate]` leaving the version offered with nothing attached and no ledger ref. Add `--test one_host` to the `tools` task in `crates/tailrace/moon.yml`. Verify: `moon run ':#check-tools'` passes.

## 4. Records and the guide

- [ ] 4.1 Write `docs/adr/0018-the-host-names-what-tells-its-gate-and-its-publish-step-tells-its-own.md` (or the next free number when 0018 is taken) recording design D1 to D3 and extending ADR 0013's table list, in the form `adrs.toml` sets. Add to `docs/architecture/core.md`, in core.10's details, "Not ruled out: the host's publish step telling the host's own gate of the publish's answer where the host's bindings file names it, whose judgment and moves are the gate's (95, 108, 130)", with the new ADR in its source. Verify: `moon run ':#check-fast'` passes, the records' numbers check among it.
- [ ] 4.2 In `site/src/content/docs/guides/self-hosting.mdx`, show `[gate] publish = {}` in the laptop's bindings beside the one-host policy, saying `beta` then moves on each publish in the same run; in the team section, show `[gate] job = "…"` on hosts that only offer; and say that a file with no `[gate]` leaves each version offered and says so. Verify: `moon run site:build` passes.

## 5. The whole

- [ ] 5.1 Run `moon run ':#check-fast'` and `moon run ':#check-tools'` and verify both pass.
