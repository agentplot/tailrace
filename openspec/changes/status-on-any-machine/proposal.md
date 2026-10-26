# Proposal

## Why

Bolt 4 is usable when `tailrace status` on any machine, one that never
published included, shows a repository's channels and why each is where
it is. Today status shows only the versions this host published, read
from its own record, so a second machine sees nothing, and no channel
appears anywhere. With the ledger and the policy seam's list of channels
built ahead of this unit, status can now read each channel from the
ledger and every channel the policy declares, and the host's one serving
process can show the same answer as a page: every repository the host
knows on one page, one row each, and each repository whole on a page of
its own
(`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 4,
status across machines; 31, 36, 56 to 63, 114, 118, 131, 132, S10 to
S12, as amended at `agentplot/blueprints` 85bbf54 and 157c828).

## What Changes

- **Status shows every channel.** `tailrace status` names the branch
  and the landed revision its policy was read at, then, for each channel
  the policy declares (131), its component, how it moves, the channel
  it is bound to where it is one, and either the one version it is at
  or that it is empty (31). Beside each version is the chain of
  promotions that put it there, newest first: from which version, who
  asked, under which policy revision, and when (33). The given `beta`
  and `stable` the policy file now holds without a line (C.10) appear
  as any declared channel does, since status reads only the seam's
  answer.
- **Every reader gets one answer, as of when it read.** Status says
  when it asked, and a chain the ledger could only answer as this host
  last read it says when that was, with the remote's or store's own
  words (36, 60, 61, C.13, C.14).
- **Three answers kept apart, per channel.** A chain that cannot be
  read, a fork the ledger refuses, and a channel never promoted are
  three answers, each carrying the ledger's own words verbatim beside
  tailrace's, and one channel's failure leaves the others shown (58,
  59). The host's record of what it published stays beside the
  channels as its own answer: absent on a machine that never published,
  which is not an error of status.
- **Asking writes nothing and reaches no store, signer or target.**
  Status asks the landing, the policy and the ledger, and nothing else;
  with the store unreachable it answers the same (60, S10). With
  everything tailrace keeps deleted but the record of claimed versions,
  the channels answer the same from the ledger and git (63, S12).
- **Status is bound on a host with no bindings file.** Such a host is
  bound as 118 now gives it: the plain branch, or sluice where the
  repository is set up for it, and the git ref ledger, so status shows
  the channels with nothing written. Status needs no store or signer,
  so it binds neither.
- **Status names what it cannot yet read of the environments.** The
  answer has a third part beside the channels and the record: the
  environments, answered unreadable, saying what each environment should
  run and runs is read from a target and no target is read yet, never
  empty and never counted as running what it should (58, 98). Phase 2's
  first target fills it; every surface already shows it.
- **`tailrace serve` is the host's one serving process.** It answers at
  `127.0.0.1:7420`, a constant of the core no setting moves (114), and
  writes nothing on a request. Each page is rendered on each request
  from the core's answer, with the same answer as JSON beside it, and no
  timer renders one (57, 132):
  - `/` shows every repository the host knows together, one row each:
    every channel's head version, and how many of its environments run
    what they should, are behind, wait on a person or cannot be read;
    `/status.json` is that answer.
  - `/<owner>/<name>/status` shows that repository whole, its channels, their
    chains, its environments and what this host published, the owner
    and name being those its landing's remote gives it;
    `/<owner>/<name>/status.json` is the same answer `tailrace status
    --json` prints in its checkout.
  - `/serving.json` names the process, its revision and its id, so an
    asker tells it from anything else on the port without asking every
    repository.
- **The first asker that finds none starts it, and the lock keeps it
  one.** `tailrace status --page` names the checkout's repository to
  the host, finds the serving process or starts one when none answers,
  and prints the page's address. The process holds the host's one lock
  for as long as it runs, so a second exits saying the first answers,
  and it ends when nothing has asked for ten minutes. A host may also
  run it as a service of its own; nothing tells a person to start one
  (114, as sluice's one reader per host is, sluice 129). The address it
  prints is the repository's own page.
- **The process answers only repositories a checkout named.** The
  repositories named live in one file of the host,
  `~/.local/state/tailrace/repositories`, owner and name with the
  checkout's path, read on each request; one no checkout named answers
  absent saying so, and the root page shows every one it names, a row
  whose checkout is gone showing that, beside the others (132).
- **Environments and notes wait, and the register says so.** What each
  environment should run, runs and waits on is phase 2's first target:
  98 moves to its block, and the register gains a way to name the part
  of a clause this unit cites that still waits: 132's environments and
  the four counts of each row, for the first target, and its notes, for
  the notes unit (ruling 4). 114
  stays registered for bolt 5's first served projection, whose served
  layout the suite will cite it with; this unit proves the process in
  the command line's own tests.
- **ADR 0018** records the serving process: its address, its paths, the
  root and each repository's page, the lock, the file of named
  repositories, how an asker starts it and when it ends.

Out of scope, and what each waits on:

- What each environment should run, runs and waits on, on the command
  line and on the pages, and the four counts of each row on the root
  page (98, 132, the environment half of S11): phase 2's first target.
  No target is asked, and the policy seam lists no environment yet.
- Each version's notes on the page (132): the notes unit, which the
  user has not yet said yes to.
- The rest of 118's bindings for a host with no file, a layout under
  the user's data directory and a host key made on first use, so that
  publish and promote work with nothing written: not this unit; status
  needs neither. The planner places it.
- A layout served at the same address (114, 115, 49): bolt 5's first
  served projection, which shares this unit's address and process.
- A layout answering what it holds and when it was rendered (51, the
  layout half of S10): bolt 5's render under a policy.
- The herdr pane and the Claude Code plugin reading the page: bolt 5's
  plugin and phase 2.
- A channel named by its component and its name together, in the
  ledger's refs and the store's names (125, C.13, C.14 as amended at
  7749369): not this unit; status asks the ledger by the channel the
  policy seam answers, and shows its component beside it. The planner
  places it.
- How each channel moves, in the policy seam's answer (131 as widened
  at 67f95da): `the-policy-seam-lists-the-channels`, as amended by the
  plan's proposal 92. This unit shows what that answer carries.

## Capabilities

### New Capabilities

- `channel-status`: every channel a repository's policy declares, at
  its version or empty, with the chain that put it there, read from the
  ledger on any machine, the same for every reader as of when it read,
  each channel's absent, refused and unreadable kept apart; the
  environments' part; one repository's row of heads and counts.
- `serving-process`: the host's one process at a fixed address,
  started by the first asker that finds none and kept one by a lock,
  answering from the one core every repository a checkout named
  together at its root and each whole at its own path, each as a page
  and as JSON.

### Modified Capabilities

- `host-record`: status's answer from the host's record is one part of
  status, absent on a host that never published while the channels
  answer.
- `command-line`: status is bound to the landing, the policy and the
  ledger, takes `--branch` and `--page`, and answers on a host with no
  bindings file; `tailrace serve` is a command.
- `host-bindings`: status is bound with no bindings file, and with one,
  needs no store, signer or ledger to answer.
- `conformance-suite`: the status group holds channels across machines,
  S10 and S12 for them, and the environments not yet read; the surfaces group holds the page beside the
  command line, the channels' answer among the questions.
- `clause-coverage`: the suite's clauses run to 132, read at 157c828,
  and the register names the part of a cited clause that waits.

## Impact

- `crates/tailrace-core`: `status.rs` gains the channels, the
  environments' part, the whole status answer and a repository's row; `served.rs` (new) holds the address and the
  repository's name from a remote's address; `history.rs` reads a
  remote's address; `lib.rs`'s exports.
- `crates/tailrace`: `commands/status.rs`, `commands/serve/` (new: the
  process, the page, the host's lock and file of repositories),
  `commands/mod.rs`, `lib.rs`, `main.rs`, `render.rs` (a time rendered
  in UTC), `bindings/` for status's binding, and the landing and ledger
  profiles' `mod.rs` for the binding of a host with no file.
- `crates/tailrace-conformance`: the status and surfaces groups below
  their markers, the register and its parts, the coverage check, the
  statement's list.
- Tests: `crates/tailrace-core/tests/status.rs`,
  `crates/tailrace/tests/cli.rs`, `conformance.rs`, and two new,
  `status_across_machines.rs` and `serve.rs`, all check-fast, needing
  git alone.
- ADR 0018, and the surfaces, self-hosting, how-it-works and roadmap
  pages.
- No crate, dependency, language or file type is added: the process
  speaks HTTP/1.1 over the standard library's sockets, and the lock is
  the standard library's file lock.

## Touches

```
crates/tailrace-core/src/status.rs
crates/tailrace-core/src/served.rs
crates/tailrace-core/src/history.rs
crates/tailrace-core/src/lib.rs
crates/tailrace-core/tests/status.rs
crates/tailrace/src/commands/status.rs
crates/tailrace/src/commands/serve/**
crates/tailrace/src/commands/mod.rs
crates/tailrace/src/lib.rs
crates/tailrace/src/main.rs
crates/tailrace/src/render.rs
crates/tailrace/src/bindings/mod.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/src/profiles/landing/mod.rs
crates/tailrace/src/profiles/ledger/mod.rs
crates/tailrace/tests/cli.rs
crates/tailrace/tests/conformance.rs
crates/tailrace/tests/status_across_machines.rs
crates/tailrace/tests/serve.rs
crates/tailrace-conformance/src/core/status.rs
crates/tailrace-conformance/src/core/surfaces.rs
crates/tailrace-conformance/src/world.rs
crates/tailrace-conformance/src/register.rs
crates/tailrace-conformance/src/coverage.rs
crates/tailrace-conformance/src/bin/coverage.rs
crates/tailrace-conformance/src/statement.rs
docs/adr/0018-*.md
site/src/content/docs/guides/integrations/surfaces.mdx
site/src/content/docs/guides/self-hosting.mdx
site/src/content/docs/how-it-works.mdx
site/src/content/docs/roadmap.mdx
```
