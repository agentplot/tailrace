# Proposal

## Why

The gate is built and nothing tells it anything. `tailrace publish`
offers a version and stops, so on a laptop that is both the offerer and
the gate (108) `beta` never moves unless something else runs the gate,
and no command does. The statement says the core starts nothing after an
offer: the gate is told by whatever the host binds to tell it, named in
the host's bindings file (130, C.16), and where the host binds nothing
the publish's own host tells the gate as its next step (118). This
change gives the bindings file that place, and builds the one teller a
single host needs: its own publish step, which tells its own gate of the
version it just offered in the same run, whether the file names it or
names no teller at all.

## What Changes

- **The bindings file names what tells the gate.** A `[gate]` table,
  in the form ADR 0013 gives every table, names it by one key (118,
  130):
  - `publish = {}`: this host's publish step tells this host's gate.
    After `tailrace publish` has offered a version, the same run hands
    the gate the publish's answer: the component, the version and the
    index digest it offered. The gate judges that version as it judges
    any version told by a publish (95): under the policy for acceptance
    at the branch's landed head, signing as the host's `[signing]`
    signer, and it moves each channel that follows each publish through
    the ledger the file binds. On a host the policy names as both
    offerer and gate, `beta` moves on every landing in one process
    (108).
  - `job = "<words>"`: a separated gate, told by a job that a store's or
    a forge's event wakes. tailrace runs nothing for it. A publish
    answers that the version is offered and that the gate is told by
    what those words name, carried as written (C.16).
  - A `[gate]` naming no key, two keys, a key not in this list, or a
    value of the wrong form is unreadable, naming the table, as every
    other table is (58, 59, core.19).
- **A file that names no teller falls to the host's default.** Where
  the file holds no `[gate]`, the host's publish step tells the host's
  gate, exactly as `publish = {}` does (118). A host that only offers,
  and whose gate is elsewhere, says so with `job`; a publish there asks
  the gate nothing. There is no binding under which a publish offers and
  nothing tells the gate (130).
- **The gate judges the digest it was told of.** A publish's answer names
  the index. When a publish tells the gate, the gate resolves the
  version's name as it always does (core.4) and, where the name now
  resolves to another digest than the one it was told, answers
  unreadable naming both, and writes nothing.
- **The publish's answer carries the gate's.** Where the publish tells
  the gate, by `publish = {}` or by the default, each
  version offered carries the gate's verdict: accepted, with the
  acceptance claim and each following channel's move, or refused,
  naming the artifact and the first claim found wanting (28, 58). A
  refused version stays offered. The exit code is the worst of the
  publish's answer and the gate's, a refusal or a channel held counting
  as refused.
- **The core's publish still reaches nothing of the gate.** The core's
  publish function and the gate share no code; the host's publish step
  calls one and then the other (130). Rule core.10 gains a line saying
  that this telling is not ruled out, since what the gate then does is
  the gate's own act (95, 108, core.14).
- **ADR 0019** records the `[gate]` table, its two keys, its default,
  and why the telling is in the command and not in the core. The
  self-hosting guide shows it for a laptop and for a separated gate, and
  says a laptop's file needs no `[gate]` for `beta` to move.
- **Conformance.** The gate's group gains a scenario in which the core's
  publish answers, nothing is accepted and nothing is appended, and the
  gate told of that answer accepts the index it names; and one in which
  a told index the name no longer resolves to is unreadable and writes
  nothing. 130 leaves the register.

Out of scope, and what each waits on:

- **What runs a publish after a landing.** 130 names it in the same
  file. Here a person or the landing host's own step runs `tailrace
  publish`; a binding naming what runs it waits on a unit the planner
  names.
- **The job a separated gate runs, and the events that wake it.** A
  forge's push event delivered to a job, or a store's own event delivered
  to the gate (C.16), are named in the file and not built. A command that
  runs the gate when asked (`tailrace accept`) waits on a unit the
  planner names.
- **A host with no bindings file.** The unit
  `a-host-with-no-bindings-file-is-bound-to-the-defaults` binds such a
  host to the defaults 118 lists, the teller among them, and with them
  the rest of 118's defaults for such a host: its landing remote, and,
  where the policy names no gate, the host as the one 108 admits. It
  comes after this unit and completes the gate table this unit adds.
  Here, a host with no file is absent naming the file, as it is today.
- **How a channel follows each publish.** The gate reads it from the
  policy file as it does today; the unit
  `the-channels-answer-says-how-each-moves` changes how, and this change
  relies only on the gate's answer.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `host-bindings`: the file names what tells the gate in a `[gate]`
  table, and a file without one binds the host's own publish to tell it.
- `command-line`: `tailrace publish` tells the host's gate unless the
  file binds a job, and says so where it binds a job.
- `acceptance`: the gate told by a publish's answer judges the index that
  answer names, and refuses to act on a name that resolves elsewhere.
- `conformance-suite`: the gate's group holds the publish's answer
  telling the gate, and 130 leaves the register.

## Impact

- `crates/tailrace/src/bindings/` reads `[gate]`; `Binding` in
  `crates/tailrace/src/lib.rs` carries what tells the gate;
  `crates/tailrace/src/commands/publish.rs` tells it and renders the
  gate's answer beside the publish's.
- `crates/tailrace-core/src/gate.rs`: a publish's telling carries the
  index it offered. Every place that builds a gate's ask is updated:
  the core's gate tests, the conformance world and the gate's group.
- `crates/tailrace/tests/`: the fakes' tests of the command (check-fast)
  and one end-to-end test in the tools layer, added to the tools task
  in `crates/tailrace/moon.yml`.
- `docs/adr/0019-…`, `docs/architecture/core.md` (core.10's details),
  `site/src/content/docs/guides/self-hosting.mdx`.
- No crate, dependency, language or file type is added.

## Touches

```
crates/tailrace/src/bindings/mod.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/src/lib.rs
crates/tailrace/src/commands/publish.rs
crates/tailrace/tests/cli.rs
crates/tailrace/tests/conformance.rs
crates/tailrace/tests/publish_tells_the_gate.rs
crates/tailrace/tests/one_host.rs
crates/tailrace/moon.yml
crates/tailrace-core/src/gate.rs
crates/tailrace-core/tests/gate.rs
crates/tailrace-conformance/src/core/gate.rs
crates/tailrace-conformance/src/world.rs
crates/tailrace-conformance/src/register.rs
docs/adr/0019-*.md
docs/architecture/core.md
site/src/content/docs/guides/self-hosting.mdx
```
