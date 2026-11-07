## Why

A person looking at status or at the host's page learns where every
channel is and the chain that put it there, but nothing of the
environments: the part is answered unreadable, saying no target is
read. Beside an environment that is behind, they should see the notes
of every version between what it runs and what its channel says it
should run (98, 132, 134). The clan push profile (record 0028) made
what a pushed environment runs a report claim its host signs and
attaches beside the version's index, so what runs is in the store; and
status reached no store (60, core.32).

madswan-design ruled on 2026-11-06, on this change's question, that
status reads what an environment runs from its reports in the store. It
fetches the reports attached to the versions its channel's chain moved
to into the host's copy where the store answers, as it fetches a chain
the store holds (C.14). It answers the newest report the policy counts,
from the copy and saying when where the store does not. It reaches no
signer and no target and writes nothing but that copy, and an
environment no counted report names is unread with the reason. Asking
every target for current (89) is out: it needs every target's credential
on the reader's host (45, 64) and reaches every machine at every status
and page request. Leaving 60 as it was is out: 98, 132 and 134 want what
runs, the phase 1 herdr pane shows it, and phase 2's list in the bolts
model holds no environments in status. The statement is amended at
agentplot/blueprints `90cf023` (clauses 60, 61, 98).

## What Changes

- **Status shows every environment the policy declares** at the
  branch's landed head, of the component asked (136). For each it
  answers:
  - the channel it follows, whether it waits and on whom (41, 97);
  - what it should run, its channel's head as the channels part reads
    it (40, 88);
  - what it runs, its newest counted report, with when the target says
    it converged and when the apply reported it (43, 91, 98);
  - its state: running what it should, behind, waiting on a person, or
    unread with the reason;
  - beside one that is behind, the notes of every version between what
    it runs and what it should, each rendered from the landed history
    as the changelog's are (134).
- **The four counts are counted.** The environments part and each
  repository's row carry how many run what they should, how many are
  behind, how many wait on a person and how many cannot be read (132).
- **Reports are fetched into the host's copy.** Status asks the store
  the claims of each index an environment's chain moved to, keeps that
  environment's report claims in a copy under the repository's common
  git directory, and answers from the copy, saying when and with the
  store's words, where the store does not answer (60, 61, S10). Deleting
  the copy loses nothing: the next status fetches it again (63, S12).
- **A report counts without reaching a signer.** It counts only through
  a signer the policy lets claim a report (116, core.7), and status
  binds every signer profile to verify with what the policy carries and
  never reach a key service (60, C.7). Until a policy can carry a
  key-service signer's public half (`docs/architecture/_open.md`), a
  report that only a key service could verify leaves its environment
  unread, naming the key.
- **Status binds the store** on every host, opened and never made, for
  the reports alone. A store that cannot be opened leaves the channels
  read and each environment answered from its copy.
- **Every surface shows the environments from the one answer**: the
  command line's text and JSON, the repository's page and the root's
  rows, and the herdr pane (56, 57, 132, core.31, core.56).
- **The suite** plays S11 and holds environments in status over the
  fakes, S10 and S12 among them. 98 leaves the register, and so do the
  waiting parts of 132 and 134. The suite reads the statement at
  `90cf023`: its clauses hold 1 to 143, 141 to 143 named in the
  register with the unit or phase that will cite them.
- **Records.** Record 0029, decided and accepted by madswan-design,
  holds the ruling and the construction's choices. Rule core.32 lets
  status fetch the reports into the host's copy as it fetches a chain,
  and the details of core.27 and core.71 say the copy is no keeping of
  a report apart from its index. Record 0028's line leaving environments
  in status to phase 2 points to 0029.

## Capabilities

### New Capabilities

- `environment-status`: the environments part of status: every
  environment the policy declares with what it should run, what it
  runs, its state, what it waits on and the notes between; the reports'
  copy; the four counts.

### Modified Capabilities

- `channel-status`: asking reaches no signer or target and writes
  nothing but the host's copy; the environments are no longer answered
  unreadable for want of a target; everything kept deleted, the reports'
  copy among it, answers the same; a row carries the counted
  environments.
- `environment-reports`: an environment's newest report has two
  readers, the gate's, which reads no copy, and status's, which reads
  the chain as last read and the reports' copy.
- `host-bindings`: status opens the store, never made, and binds the
  signers to verify reaching no service.
- `key-service-signer`: the profile can be bound to reach no key
  service, for status.
- `serving-process`: the page renders every environment with the notes
  between; the page and the root write nothing but the host's copy.
- `herdr-plugin`: a row shows the counts as counted, and an opened
  repository shows every environment with the notes between.
- `conformance-suite`: environments in status over the fakes, S11, and
  the register after this change.
- `clause-coverage`: the suite's clauses as the statement holds them at
  `90cf023`.

## Out of scope, and what each waits on

- A policy carrying a key-service signer's public half (C.7): unit
  `a-policy-carries-a-key-service-signers-public-half`. Until it lands,
  status reads such a signer's report as unread naming the key.
- Environments a target's roster holds (126): phase 2. Reading a roster
  reaches a target (60), and no target of phase 1 holds one.
- Keyless signing and another system's trusted root (142, 143): phase
  2. The catalogued kinds (141): unit
  `every-claim-takes-the-catalogued-kind-that-carries-it`.
- Taking `main` into the bolt and renumbering its records 0026 and 0027
  to 0027 and 0028: a fix on `bolt/tailrace-the-nix-fleet-pushed`, which
  this change's build takes in before it writes record 0029.

## Impact

- The core: `status.rs` gains the environments part and the notes
  between; `after.rs` splits the newest report into reading the reports
  and counting them, the gate's path unchanged; a new module keeps the
  reports' copy; `apply.rs`'s `reporters` is shared.
- The command line: `bindings/mod.rs` binds the store and the signers
  for status; the key-service profile gains a mode that reaches no
  service; `commands/status.rs`, `commands/serve/page.rs` and the
  pane's `lib/words.jq` render the part.
- The suite: the status group, the register and `statement.rs`.
- The site's surfaces page; record 0029; rules core.27, core.32 and
  core.71; `docs/architecture/_open.md`.
- No crate, dependency, language or file type is added.
- Rules kept: core.4, since what runs is compared by index digest;
  core.7, since a report counts only through a signer the policy lets
  claim one; core.19, since every environment, its reports and each
  version's notes carry their own answer; core.20, since the copy is
  rebuilt from the store; core.21, since nothing reached needs a
  credential beyond the store's; core.31, since every surface renders
  the core's answer; core.45, since the gate's move reads no copy;
  core.50, since what an environment should run is read from its chain;
  core.56, since the page escapes the notes and runs no script.

## Touches

```
crates/timber-core/src/status.rs
crates/timber-core/src/after.rs
crates/timber-core/src/apply.rs
crates/timber-core/src/reports.rs
crates/timber-core/src/lib.rs
crates/timber-core/tests/**
crates/timber-fakes/src/**
crates/timber-conformance/src/core/status.rs
crates/timber-conformance/src/core/surfaces.rs
crates/timber-conformance/src/register.rs
crates/timber-conformance/src/statement.rs
crates/timber/src/bindings/mod.rs
crates/timber/src/bindings/tests.rs
crates/timber/src/profiles/signer/**
crates/timber/src/commands/status.rs
crates/timber/src/commands/serve/**
crates/timber/tests/**
plugins/herdr/**
site/src/content/docs/guides/integrations/surfaces.mdx
site/src/content/docs/guides/herdr.mdx
docs/adr/0028-*.md
docs/adr/0029-*.md
docs/architecture/core.md
docs/architecture/_open.md
```
