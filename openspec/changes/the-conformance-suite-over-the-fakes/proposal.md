# Proposal

## Why

Bolt 1 is done when one conformance suite holds the core and every
fake to Parts A and B, and every later unit is done when its profile
passes the same suite (93, core.33; phase-1-bolts.md, Bolt 1 and How
the units stay apart). The seams, the fakes, the version and publish,
status and verify are planned; nothing yet says which clause each test
proves, or notices a clause nothing proves. Bolt 1 builds no gate,
channel, ledger, environment, layout or cache, so about forty clauses
have nothing to prove them until bolts 2 to 5. Record 0012 and the
amended core.33 settle how the suite treats those clauses: it names
them in a register instead of failing them.

## What Changes

- **One suite, in groups.** `tailrace-conformance` holds every scenario,
  each with the clauses of Parts A and B it proves. Scenarios come in
  groups: one per seam (the store, the build, the landing, the signer,
  the policy, a projection, a target, the keeper), one for the binding,
  and one for each part of the core: version, publish, verify, status,
  the surfaces, and the gate, the ledger, the cache, the record and the
  layouts that later bolts build. Each group ends at a marker. A later
  unit adds its scenarios below its group's marker and changes nothing
  above it.
- **Every profile gets the same report, a fake included.** A seam's
  group runs over whatever the caller binds: the fakes here, a real
  profile in that profile's own tests. Every scenario runs on a fresh
  rig. A profile that fails gets one report naming the profile, the
  seam, each scenario that failed, the clauses it cites, and why. The
  report is the same for a fake and a real profile, so a fake that
  breaks its contract fails the suite like any profile (93).
- **The seams' contracts over the fakes.** Each seam's group plays its
  Part B clauses (66 to 91, 120 to 122) and the Part A clauses those
  operations keep: same bytes held once, a name moved only from where
  it is, attestations kept beside their digest, an expired signer told
  as expired, acceptance naming no channel, an undeclared policy absent
  naming its audience, a pack that cannot promise
  the same bytes saying so, an empty channel answered empty, a stale
  predecessor refused naming the head, a fork reported and never
  chosen. Each group also checks that a source that cannot be reached
  answers unreadable in its own words (58, 59).
- **The statement's scenarios over the fakes.** S1, S2, S5 and S13 play
  over the fakes and an in-memory store, with a real git history in a
  temporary directory. S1: a landed revision published, two packages at
  the derived version, the index last, provenance and verdicts
  attached, the answer naming every digest, no channel moved. S2: the
  same revision again writes nothing and ends at the same index. S5 and
  S13 go as far as verify's answer, as record 0012 has it: S5's verdict
  from a signer the policy does not name is listed and not counted,
  nothing moves, and the version reads back by digest. S13's revision
  is published with no verdicts; under a policy that requires verdicts,
  no verdict counts, and under one that requires provenance alone, the
  provenance counts. S15 plays over a two-component fixture: a commit
  under one component's paths versions that component alone, under its
  own prefix, and the other gets no version, index or name (123, 124).
- **Clause coverage, with a register of what waits.** A coverage check,
  its own moon task in the gate, reads every scenario's citations
  against the clauses of Parts A and B and against the register. A
  clause neither cited nor registered fails. A clause both cited and
  registered fails. A citation that names no clause fails, and so does
  a scenario with no citation or a register entry missing its unit or
  its reason. Each registered clause is printed as pending, with its
  unit, on every run, pass or fail (section 12, record 0012, core.33).
- **The register.** It names 47 clauses, each with the unit that will
  cite it and why. These are the 43 clauses record 0012 lists, plus
  four that bolt 1 has nothing to prove: 42 (a reconciler verifies
  before it applies), 53 and 100 (a cache entry is never an artifact),
  and 64 (no credential kept). Six of them, 42, 44, 45, 119 (the
  reconciler) and 55, 112 (the verifier's rebuild), have no unit in
  phase 1's plan, which leaves every target and the verifier's rebuild
  to phase 2. They name phase 2, as core.33 and section 12 allow while a
  phase has no units carved; the planner names the unit when it carves
  phase 2.
- **The command line as a surface.** The surfaces group asks publish,
  verify and status through a surface and holds its answer to the
  core's, value for value (56, 57). It runs over the command line in
  the `tailrace` crate's tests.

Out of scope, and what each waits on:

- Every real profile's run of its seam's group: the profile units of
  bolts 2 to 5, each in its own tests.
- S3, S6 to S12 and S14, and the gate's half of S5 and S13, which needs
  acceptance and refusal: bolts 2 to 5. Bolt 4's gate unit adds the
  gate's refusal and acceptance below the gate's marker and takes 25 to
  30 out of the register (record 0012).
- Taking a clause out of the register: always the commit that first
  cites it.
- A second surface to compare with the command line: bolt 5's Claude
  Code plugin.

## Capabilities

### New Capabilities

- `conformance-suite`: scenarios in groups per seam and per part of the
  core, each citing the clauses it proves; groups that later units only
  add to; one report for any profile that fails, a fake included; the
  seams' contracts, S1, S2, S5, S13, S15 and the surfaces, played over
  the fakes.
- `clause-coverage`: every clause of Parts A and B cited by a scenario
  or named in the register with its unit and reason, checked on every
  run, with registered clauses reported as pending.

### Modified Capabilities

None.

## Impact

- `crates/tailrace-conformance/`: the harness, the groups, the rigs over
  the fakes, a git fixture, the list of clauses, the register, the
  coverage check as a binary, tests that run every group over the
  fakes, and a `coverage` moon task tagged `check-fast`.
- `crates/tailrace/tests/conformance.rs`: the surfaces group over the
  command line. The crate already takes the suite as a dev-dependency.
- `Cargo.lock`: `tempfile`, already a workspace dependency, becomes a
  dependency of the conformance crate for its git fixture.
- No trait, fake or core function changes. A scenario that finds one
  short is reported with `crew needs` and becomes its own unit
  (phase-1-bolts.md).

## Touches

```
Cargo.lock
crates/tailrace-conformance/**
crates/tailrace/tests/conformance.rs
```
