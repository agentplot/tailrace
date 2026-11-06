# Verify: the-clan-push-profile

This report checks the work on `change/the-clan-push-profile` at
`11faad0` against this change's proposal, design, nine delta specs and
tasks. The branch was cut from `bolt/tailrace-the-nix-fleet-pushed`,
and it took that bolt at `43d3350` (design D1).

**It falls short, so the change is not archived.** Every task is ticked
and every check the tasks name passes. Every requirement is met in the
code but one: S16's world does not declare `first` as the
conformance-suite delta says it is. One module document also still
says that apply signs nothing, which contradicts the host-bindings
delta. Both go back to the code stage, as listed under "What fails".

## The checks

| check | answer |
|---|---|
| `openspec validate the-clan-push-profile --strict` | valid |
| `moon run ':#check-fast'` | exit 0, 20 tasks, 15 replayed from moon's cache |
| `moon run ':#check-fast' --force` | exit 0, 20 tasks run uncached, 1536 tests passed and 0 failed: format, lint, toolchain pins, the records' numbers, every crate's tests, the conformance suite over the fakes, the coverage check and the site's build |
| `moon run ':#check-tools' --force` | exit 0, `cli:tools` run uncached, 101 tests passed and 0 failed; `nix_fleet_push` is not among them |
| `moon run cli:fleet --force`, with `TIMBER_TOOLS_FLEET` unset | exit 101; the run panics at `crates/timber/tests/nix_fleet_push.rs:65` before any command, with the words "the fleet run names no machine: set env.TIMBER_TOOLS_FLEET in devenv.local.nix to `<checkout>#<machine>`, …" |
| the coverage check, `conformance:coverage` | 140 clauses: 129 cited, 11 pending, read at agentplot/blueprints `7f60e57`. 41 and 97 are cited and appear in no register entry, and no entry names the clan push profile. 44 and 126 stay with `phase 2`; 98 and the parts of 132 and 134 stay with `phase 2 · environments in status` |

The run against a machine, `moon run ':#check-fleet'` with the setting,
has not been run. It is the operator's by hand before the change lands
(tasks, workflow follow-up; rule core.38).

## The tasks

Tasks 1.1 to 7.1 are all ticked, and each one's Verify holds, with the
exceptions below:

- 1.1: the bolt merged at `cb66b3b`.
- 2.1: the seams' round trips, including an old bound channel read back
  byte for byte.
- 2.2: the fake policy's `after`, recorded and faultable.
- 2.3: the policy file's `eu` after `us`, the deployer that may report,
  nobody may report, the one host, and a `require` naming a report.
- 3.1 and 3.2: `judge` and `newest` in the core's tests.
- 3.3: apply's six report cases in `timber-core/tests/apply.rs`.
- 3.4: `For::Apply` binding the host's signer.
- 3.5: `timber verify` listing a report.
- 4.1: `ledger::moved` is the one half that both `promote` and `follow`
  call.
- 4.2: `follow`'s seven cases in `timber-core/tests/follow.rs`.
- 4.3: the command's gate, job and unread report in `cli.rs`.
- 4.4: the `after` group. 41, 97 and the register's constant all left
  in `60c01ac`, the commit that adds S16.
- 4.5: `Why::MovesOnReports` before the chain is read, in the core,
  in S16's step 4 and in `cli.rs`.
- 5.1 to 5.4: the profile's unit tests over `Scripted`, and the target
  group over it.
- 5.5: the `fleet` task, tagged `check-fleet` alone.
- 6.1 and 6.2: record 0027, proposed; rules core.71 to core.73; the
  site's targets page and the integrations index.

The exceptions are 4.4, whose world falls short of its spec (below),
and 3.4, whose module document was left as it was.

## The specs

- **clan-push-target.** The profile lives under
  `profiles/target/nix_fleet/push/`, bound as `clan`.
  - **Table.** `machine` and `closure` are required, and any other key
    is unreadable naming it and the two keys the table reads. `machine`
    must be letters, digits, `-` and `_`, beginning with a letter or a
    digit.
  - **Apply.** It reads the index and its revision and repository,
    refusing another publishing repository. It reads the closure
    through `nix_closure` and refuses anything but exactly one root. It
    then makes the revision's worktree under `<common>/timber/fleet/`,
    checks the evaluated toplevel against the root, and realises the
    root with `nix-store --realise … --option max-jobs 0`. Last it runs
    `clan machines update <machine> --flake <worktree> --build-host
    localhost --host-key-check strict`. Each step's refusal is tested
    over `Scripted`, asserting which fleet commands ran.
  - **Marker.** It is written by `clan ssh` to a temporary file renamed
    into `${XDG_STATE_HOME:-$HOME/.local/state}/timber/markers/<owner>/<name>/<component>/<environment>.json`.
  - **current and report.** Both read `/run/current-system` and the
    marker in one `clan ssh`. They answer as the spec says, unread
    where the machine runs something else or holds no marker.
  - **Rule core.1.** No source outside `profiles/` names clan, Nix's
    commands or ssh.
- **environment-reports.** A converged apply signs one report claim as
  the bound signer and attaches it beside the index.
  - **The claim.** Its kind is timber's own `report/v1`, and it names
    the index's revision. Its body holds environment, component,
    channel, version, index, runs, converged_at and the ask's `at`.
  - **When nothing is signed.** An unread report signs nothing, and so
    does one naming another digest, whose answer names both.
  - **Head already running.** Apply reads the newest counted report and
    signs only where that report names another index or none.
  - **Refused attach.** The store's words are carried, and the
    convergence is answered as made.
  - **Newest report.** It is read across the channel's chain and
    ordered by the apply's `at`. It is absent where none counts, and
    unreadable naming the chain or the index.
  - **verify.** `timber verify` lists a report with its signer and its
    environment.
- **after-channels.**
  - **Telling the gate.** `commands/apply.rs` tells the gate under
    `Teller::Publish` and carries a job's words under `Teller::Job`,
    moving nothing. The core's apply starts nothing.
  - **The host's signer.** `follow` refuses a host that is not among a
    channel's promoters with `Why::NotPromoter` before any report is
    read.
  - **Held.** A channel is held, naming each environment's answer,
    where the environments disagree or one has no counted report.
  - **The move.** Otherwise the channel moves through `ledger::moved`
    under its own policy. The host's signer is recorded as the asker,
    and the reports are among the counted claims. It follows back the
    same way and is never moved ahead (`held_by`, newest by `at`).
  - **An asked promotion.** It is refused with `Why::MovesOnReports`
    before the chain is read, with nothing signed or appended, the
    channel's own promoter included.
  - **Each channel.** Each is answered on its own, and the exit is the
    worst of the parts, held counting as refused.
- **seam-contracts.**
  - `Moves::Bound { to, after }` skips `after` when it is empty, so an
    old bound channel reads and writes back the same.
  - `RequiredClaim.environment` is written only where it is set.
  - `ClaimKind::Report` is in the seams, and a report on the index
    satisfies a requirement for every artifact.
- **seam-fakes.** The fake policy's `after` scripts a bound channel
  coming after the environments a test names, with a report
  requirement of each. Both calls are recorded and faultable.
- **policy-file.**
  - `report` is among a signer's `kinds`, and the one host 108 admits
    gets no report.
  - A `require` naming a report is unreadable, naming its table.
  - An `after` channel is listed with its environments in order. Its
    policy requires acceptance, its line's claims, then a report of
    each environment by the signers that may report. Where nobody may
    report, the requirement names no signer and the judge refuses it.
- **host-bindings.**
  - `For::Apply` binds the host's signer first, the given host key
    where `[signing]` is absent, and every other signer verify-only.
  - With no `[gate]`, this host's own apply tells its gate.
  - Nothing is signed where apply is refused before the target is
    asked.
- **command-line.**
  - Apply's text and JSON carry the target's report, the report claim's
    digest and each `after` channel's answer, or a job's words.
  - The modified "an environment applied" prints the report and its
    digest, asserted line by line.
  - Apply still makes no store with the signer bound.
- **conformance-suite.**
  - The `after` group's three report scenarios cite 11, 43 and 91, and
    43 and 91 twice.
  - S16 cites 37, 41, 43, 83, 91 and 97, asserts `never_ahead` after
    every step, and refuses step 4's asked move.
  - `never_ahead` is shown failing, naming the step and both indexes.
  - The target group passes over the clan push profile's scripted
    fleet, holding a marker.

## What fails

1. **S16's `first` is not declared manual.** This falls short of the
   conformance-suite requirement "S16 over the fakes" and design D10.
   The spec says the group plays S16 "with `first` a manual environment
   on `stable` whose `approvers` name `desk`". `s16_world` in
   `crates/timber-conformance/src/core/after.rs` declares `first`
   through `declares`, which hard-codes `waits: false, approvers:
   vec![]` (`after.rs:137-151`). The policy's environments answer
   therefore says `first` moves on its own and waits on no one.
   - Only `first`'s channel policy is scripted as manual: bound to
     `stable`, with `desk` as its approver.
   - The doc comment of `s16_world` says "`first` declared manual on
     `stable`", which the code contradicts.
   - Nothing in the core reads `DeclaredEnvironment.waits` today:
     apply carries it to the target, and the move reads the channel's
     policy. So the scenario passes, but its world is not the one the
     spec names and clauses 41 and 97 describe.
   - The fix: declare `first` with `waits: true` and `approvers =
     [desk]`. Make `declares` take both, or add a declaration for a
     manual environment.
2. **The bindings module still says apply signs nothing.** This
   contradicts the host-bindings requirement "Apply binds the host's
   signer to sign its report". The module document of
   `crates/timber/src/bindings/mod.rs`, at lines 107 to 110, still
   reads "Apply is bound as status is, signing nothing, with the store
   opened and never made". `For::Apply` now binds the host's signer to
   sign (`mod.rs:614-631`). The `For` enum's own document (`mod.rs:486-488`)
   is right. The fix: say that apply binds the host's signer to sign
   its report and the gate's moves on it, every other profile
   verify-only, with the store opened and never made.

## Observations, none blocking

- **clan-push-target, "An environment names the clan profile".** No
  test sends an environment written `target.clan = {…}` through apply
  to the profile. `bindings/tests.rs` shows `clan` bound beside
  `nix_cache`. The core hands each environment its own table's settings
  and dispatches by the table's profile, which is tested for
  `nix_cache`. So the scenario holds by construction.
- **clan-push-target, "A machine named as an option".** `-swancloud` is
  tested at `Table::read`, not through apply. Every operation reads the
  table first (`push/mod.rs`), so nothing runs.
- **clan-push-target, "An index naming no revision".** The test asserts
  "names no revision", not the index's digest, which the message
  carries.
- **The real `Clan` commands.** `Clan`'s command lines are tested by
  no `check-fast` test, since `Scripted` stands in for them. They match
  design D3 and D4 as read. Only the by-hand run exercises them.
- **No person asked.** `--host-key-check strict` keeps ssh from asking
  about a host key. Nothing passes `BatchMode`, so on a terminal ssh
  could still ask for a key's passphrase. A deployer's loop with no
  terminal cannot be asked.
- **Untested side paths.** Four are untested, and each code path reads
  as the design says:
  - in the core's apply, the revision of an index that names none,
    taken from the version's tag;
  - also in apply, a newest report that cannot be read on the
    already-running path;
  - in `follow`, a channel held on an unreadable report;
  - also in `follow`, the never-held check on the gate's path.
- **A host whose `kinds` lack `report`.** Its reports never count. So
  each apply that finds the head running signs and attaches another
  one. This follows design D6 and rule core.71's "newest counted
  report", and the policy file's own reporters would be listed with
  `report`.
- **Telling the gate with nothing signed.** The command tells the gate
  whenever the report is answered, including one already held, and
  `follow` then answers that the channel is already there. Task 4.3
  says "answers a report digest", so this is as tasked.
- **S16's last moves.** S16's step 3 checks that `second` moved to
  `first`'s index from `0.2.0`, not that the move counted the step's
  report. The move could not pass `judge` without the report, so it
  holds. The chain's length is checked after step 3 but not after
  step 4, where the writes are checked unchanged.
- **`desk` in S16.** `desk` is both `first`'s approver and promoter, and
  it signs its own move. The fake may script that. A policy file would
  list `desk` as a promoter only by its own `kinds`, never by being an
  approver (rule core.44).
- **Records.** Record 0027 is proposed, for the user to accept with
  the change. The statement's amendment at blueprints `c118a95` is not
  yet landed. The coverage check reads the statement at `7f60e57`,
  and adopting the newer statement is the follow-up's own chore.
