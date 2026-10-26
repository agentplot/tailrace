# Design

## Context

The bolt now holds everything one host needs for `beta` to move. The
gate judges a version told of by a publish and moves each channel that
follows each publish (`crates/tailrace-core/src/gate.rs`, `accept`,
`AcceptAsk`, `ToldBy::Publish`). The bindings file binds a ledger
(`[ledger]`, ADR 0016). `tailrace publish` signs as the host's
`[signing]` signer (`crates/tailrace/src/commands/publish.rs`). Nothing
connects these: no command calls `accept`, and the archived gate change
left "what tells the gate" to a later unit (130, C.16).

Constraints:

- The core starts nothing after an offer, and what tells the gate is
  named in the host's bindings file and nowhere in the core (130, C.16).
  Where the host names nothing, the publish's own host tells the gate
  as its next step (118, as amended at agentplot/blueprints ab0dfea).
- Rule core.10 rules out a publish that moves `beta`, which is the
  gate's act (95). Rule core.14 lets one host hold both the offering
  and the accepting credential where the policy names it as both (108).
- ADR 0013 fixes the file's form: one TOML file, each table naming its
  profile by one key, read generically by a module that names no tool
  (core.1).
- `gate.rs` imports nothing of publish, and publish imports nothing of
  the gate (the gate's design D5); this change keeps that.

## Goals / Non-Goals

**Goals:**

- On a host whose file binds `[gate] publish = {}`, or holds no
  `[gate]`, one run of `tailrace publish` offers, then tells the gate,
  and the gate accepts and moves `beta`.
- A file that names a job leaves the publish as it is today and says
  what tells the gate.

**Non-Goals:**

- Running anything a separated gate needs: no job, no event listener,
  no `tailrace accept`.
- Any change to how the gate judges, which channels follow each publish,
  or how a promotion is written.
- Binding what runs a publish after a landing (130's first half).
- A host with no bindings file, and the host as the one 108 admits where
  the policy names no gate: the unit
  `a-host-with-no-bindings-file-is-bound-to-the-defaults` (118).

## Decisions

### D1. `[gate]` names the teller by one key

```toml
[gate]
publish = {}        # this host's publish tells this host's gate (108)
# job = "the release job a push to main wakes"   # a separated gate
```

The table is read by the same generic reader as `[store]`, `[landing]`
and `[ledger]` (`seam` in `bindings/mod.rs`): one key, else unreadable
naming the table. `publish` takes an empty table, as `[landing] sluice =
{}` does. `job` takes a string, the operator's own words for what
tells the gate. tailrace reads it only to carry it in the publish's
answer, since the core calls none of these (C.16). An empty or
whitespace-only string is unreadable: words that say nothing don't
name a job. The names are the statement's words: C.16's list starts
with a step on the landing's host, which is the publish step here, and a
job that an event wakes covers its forge and store events.

A file with no `[gate]` binds `publish`, the host's default (118): the
publish's own host tells the gate as its next step. It is the same
teller as `publish = {}`, so the answer, the exit code and every proof
are the same for both. `publish = {}` stays a key so an operator can
write down what the host does.

Alternatives:

- No `[gate]` binds no teller, the publish saying so. Rejected: 118
  gives the default, and 130 leaves no host where nothing tells the
  gate; a host whose gate is elsewhere says so with `job`.

Alternatives to the key's form:

- `told_by = "publish"`, one string key. Rejected: it is the flat form
  ADR 0013 turned down for seams, and a separated teller then needs a
  second key for its words.
- One key per C.16 item (`forge`, `store`, `asker`). Rejected for now:
  tailrace behaves the same for each one, since it starts none of them,
  so separate keys would only be labels. A later teller that tailrace
  does build becomes a new key of the same table (ADR 0013's rule).
- Putting it in `[signing]` (whether the host is the gate). Rejected:
  whether this host may accept is the policy's question (108), and who
  tells the gate is a separate fact about the host.

### D2. The telling is the command's, after the core's publish answers

`commands/publish.rs` calls `tailrace_core::publish`, and only after it
answers calls `tailrace_core::accept` for each `Published::Version`.
The core's publish function and the gate share no code path. The step
that calls both is the host's binding of what tells the gate (130),
which is why it sits in the binary that reads the bindings file.

This is the reading of core.10 the change adds to the rule's details as
a "Not ruled out" line: the host's publish step telling the host's own
gate of the publish's answer, where the bindings file names it, since
the judgment and every move after it are the gate's (95, 108, 130). The
rule's "Rules out" stays as it is: the core's publish moves no channel,
and the `publishing` spec's "A publish offers and does nothing more"
still holds of it.

Alternatives:

- A core function that publishes and then judges. Rejected: the core
  would start something after an offer (130).
- A second command run after `publish` (`tailrace accept`). Rejected:
  that is a second process for the landing step to run, and the
  bindings file would then name a step rather than the telling.

### D3. A publish's telling carries the index it offered

`ToldBy::Publish` becomes `ToldBy::Publish { index: Digest }`. It loses
`Copy` and keeps `Serialize`. `accept` resolves the name at step 2, as
it does today (core.4). Where the resolved digest is not the told one,
it answers unreadable before step 3, with the store as the source,
naming the tag, the resolved digest and the told one, so nothing is
read further and nothing is written. `ToldBy::Asked` carries nothing.
Every place that builds an `AcceptAsk` changes mechanically:
`crates/tailrace-core/tests/gate.rs`,
`crates/tailrace-conformance/src/world.rs` and the gate's group.

Alternative: the command compares `Judged`'s index with the publish's
afterwards. Rejected: by then the gate has written its acceptance on
whatever the name resolved to.

### D4. The binding carries the teller

`Binding` (`crates/tailrace/src/lib.rs`) gains `gate: Teller`, defined
in `bindings/mod.rs`:

```rust
pub enum Teller {
    /// `[gate] publish = {}`, or no `[gate]` (118): this host's
    /// publish tells its own gate.
    Publish,
    /// `[gate] job = "<words>"`: a job outside tailrace tells it.
    Job(String),
}
```

`bind` sets it for every command; only publish reads it. The test
bindings in `tests/cli.rs` and `tests/conformance.rs` set
`Teller::Job` with words of their own, so their publish answers are
unchanged apart from the new `gate` member and no test there asks the
gate.

### D5. The answer and the exit code

Publish's answer is the core's `Publish`, rendered as today. For each
component whose answer is `Published::Version`, the command adds the
gate's part:

- `Publish`: `Answer<Judged>` from `accept` with `ToldBy::Publish {
  index }`, the component and version from the answer, the branch the
  publish was asked on, `gate` the binding's signer, and `when` read
  from the clock as `promote` reads it.
- `Job(words)`: `{ "told_by": "job", "job": words }`; the gate is not
  asked.

In JSON, each component's entry gains `gate`: for `Publish`,
`answer_json` of the gate's answer with `told_by: "publish"` added; for
`Job`, the object above. A component with any other answer carries no
`gate` member. In text, an indented `gate:` line follows the version's
lines: accepted with the acceptance's digest, and "one host" where it
is one; each channel's move (promoted from → to, there, held naming the
claim, or its `Not`); refused naming the artifact and the claim; or told
by the job's words.

Exit code: the worst (`render::worst`) of each component's publish code
and, where the gate was asked, its code. `Judged::Refused` and
`Moved::Held` count as `REFUSED`, and a channel's `Not` counts by its
kind. `Job` adds `OK`, because the publish did what it was asked to do.
A landing step that runs `tailrace publish` on the one host still fails
when `beta` did not move, and says why.

### D6. The one host signs and appends with what it already binds

Telling the gate needs nothing more of the file. The gate's identity is
the `[signing]` signer that publish already requires. The ledger is
whatever `[ledger]` binds. Where none is bound, each following
channel's answer is the ledger's absence naming `[ledger]` (the gate
reads an absent chain as empty, and the append is then absent), and the
version stays accepted (94). Whether this host may accept is the
policy's question: where its signer may not claim acceptance, the gate
refuses `OfferedByGate` and writes nothing (27, 96, S9).

### D7. Proof

- **Bindings** (`bindings/tests.rs`, check-fast): each `[gate]` form of
  the host-bindings spec, and that a broken `[gate]` makes no layout.
- **The command over the fakes** (`tests/publish_tells_the_gate.rs`,
  check-fast): a host binding built over the fakes with each `Teller`.
  It checks the run's answer, the JSON `gate` member value for value
  against `accept`'s own answer, the exit codes, unchanged components
  left untold, and the fake keeper's and store's calls (no acceptance or
  append with `Job`). The teller a file with no `[gate]` binds is
  read through `bindings::bind` and gives the same run as `Publish`.
- **The one host end to end** (`tests/one_host.rs`, tools layer, added
  to the `tools` task): the built binary, a scratch repository on a
  plain branch (C.6) with moon, a host key the policy names for
  provenance, acceptance and promotion, an OCI layout, the git ref
  ledger, and `[gate] publish = {}`. Two landings, each followed by one
  `tailrace publish`; `beta`'s chain in `refs/tailrace/ledger/` holds two
  promotions, the second naming the first, the layout's name for `beta`
  is at the second index, and each acceptance says one host. A second
  run of the same test with no `[gate]` shows the same: `beta` moved on
  each landing, the default teller being the host's publish (118).
- **Conformance** (the gate's group, below its marker): the two
  scenarios of the conformance-suite spec, citing 95 and 130. The
  register loses 130.

### D8. Records

ADR 0019 records D1 to D3: the table, its keys, its default, why the
command and not the core holds the telling, and why the told index is
checked. It extends ADR 0013's table list as ADR 0016 did for
`[ledger]`. `docs/architecture/core.md` gains core.10's "Not ruled out"
line (D2). The self-hosting guide's laptop section shows `[gate]
publish = {}` beside the one-host policy and says a file without
`[gate]` does the same, and its team section shows `[gate] job` on the
hosts that only offer, saying such a host must name it.

## Risks / Trade-offs

- [The command calls both publish and the gate, which reads close to
  core.10's "Rules out"] → the rule gains its "Not ruled out" line in
  this change, citing 95, 108 and 130, so a reviewer has a rule to point
  at rather than a reading.
- [`job` is free text that tailrace never checks] → it is named, not
  bound (C.16). The words are carried as written so the person reading
  the publish's answer knows where to look. A teller tailrace builds
  later gets its own key.
- [ADR number 0019 may be taken by a unit that lands first] → the
  records' numbers check in check-fast catches it; the later unit takes
  the next number at its merge.
- [A host that only offers and has a file with no `[gate]` now asks its
  own gate, which refuses where the policy does not let its signer
  accept, and the publish exits refused] → the refusal names the claim
  found wanting, and the guide's team section says such a host names
  its `job`. The version stays offered either way (20).
- [`ToldBy` loses `Copy`] → its only uses are matches and builders in
  the gate's own tests and the suite, all listed in the touches.
