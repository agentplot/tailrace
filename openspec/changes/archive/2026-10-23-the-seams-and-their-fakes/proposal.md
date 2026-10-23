# Proposal

## Why

tailrace's core has to reach every outside system the same way, and each
of the bolt's later units, the version, publish, status and verify, the
conformance suite, codes against that reach. Until the eight seams of
Part B exist as interfaces, with the values that cross them and fakes
that stand in for every one, nothing of the core can be written or
tested without a registry, a task runner, a merge queue or a key on the
host (core.1, core.2; ADR 0002, 0005, 0009, 0011).

## What Changes

- `tailrace-seams` gains one trait per seam of Part B, each holding that
  seam's operations and nothing else: the store (66 to 73), the build
  (74 to 76), the landing (77, 78), the signer (79 to 81), the policy
  (82, 83), a projection (84 to 87), a target (88 to 91) and the keeper
  (120 to 122).
- It gains the types those operations pass, defined there and nowhere
  else: digest, revision, component, version, artifact and index
  metadata (4, 6), claim and signed envelope (7, 8, 13), a verdict as
  the landing recorded it (78), the policy's requirements for an
  audience (82, 83), and the binding of one store, one build, one
  landing, one policy, one keeper, the signers, and any number of
  projections and targets (92).
- Every seam operation answers in one of four ways: what was asked,
  absent, refused, or unreadable. Absent, refused and unreadable are
  kept apart and each names the source that answered; a refusal and an
  unreadable answer carry that source's own words verbatim (58, 59,
  core.19). No seam operation answers with an error of any other kind.
- `tailrace-fakes` gains one in-memory fake per trait, holding state the
  way a real profile would, recording every call it is asked, and
  scriptable per test to refuse, to be unreadable, to hold other bytes
  under a digest, to report an output unmade, to expire a signer or to
  hold a fork, so the core and the suite run with no registry, runner,
  queue or key.

Out of scope, and what each waits on:

- The core's own modules, the version rule, the policy file and
  components read at a landed revision: `a-landed-revision-takes-its-version`.
- Publish, status, verify, provenance and the machine-readable answer:
  `publish-status-and-verify-from-one-core`.
- The conformance suite and its scenarios (93, core.33):
  `the-conformance-suite-over-the-fakes`, which runs over these fakes.
- Every real profile of Part C and the host's bindings file (118):
  bolts 2 to 5.

## Capabilities

### New Capabilities

- `seam-contracts`: the eight traits of Part B, the types they pass, the
  four answers every operation gives, and the binding the core is handed.
- `seam-fakes`: one in-memory fake per trait, scriptable per test, that
  keeps the contract its trait states.

### Modified Capabilities

None.

## Impact

- `crates/tailrace-seams/`: the traits and types, with their tests.
- `crates/tailrace-fakes/`: the fakes, with their tests.
- `Cargo.toml` and `Cargo.lock`: one workspace dependency, `sha2`, for
  the digest of bytes (1).
- Nothing outside the two crates calls them yet; the core, the binary
  and the conformance crate are unchanged.

## Touches

```
Cargo.toml
Cargo.lock
crates/tailrace-seams/**
crates/tailrace-fakes/**
```
