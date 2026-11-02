---
number: 22
title: Every claim tailrace signs takes the catalogued kind that carries it
status: accepted
date: 2026-11-02
decision-makers:
- Chuck Swanberg
---

# Every claim tailrace signs takes the catalogued kind that carries it

Status: accepted by the user on 2026-11-02, approving that the clauses of `agentplot/blueprints` 7beb5fa come into this repository's rules and records; the choice is their ruling of 2026-11-01, which that commit carries into the statement.

Deciders: the user, 2026-11-01, ruling on a reading of the public catalogues of claim kinds, carried into the statement at `agentplot/blueprints` cfe9e04 and 7beb5fa.
Sources: statement clauses 7, 9, 10, 11, 12, 13, 30, 116, 141, 142, 143, sections 3 (trusted root) and 9, as amended at `agentplot/blueprints` cfe9e04 and 7beb5fa; ADRs 0005, 0014; rules core.1, core.7, core.8, core.51; in-toto's predicate catalogue, in-toto/attestation v1.2.0, `spec/predicates`; SLSA 1.2's build requirements, which recommend SLSA Provenance; SLSA's Verification Summary Attestation, v1.1; Sigstore's bundle; `crates/tailrace-seams/src/claim.rs` as origin/main holds it on 2026-11-02.

## Context and problem statement

Every attestation is an in-toto statement in a DSSE envelope, the form sluice signs its verdicts in, so one verifier reads both (section 9). A statement names the type of its predicate, and phase 1 gave every kind of claim tailrace's form writes a type of tailrace's own, `https://agentplot.github.io/tailrace/claim/<kind>/v1`, a provenance, an acceptance and a rebuild among them. in-toto keeps a public catalogue of predicate types, SLSA's Provenance, SLSA's Verification Summary Attestation and in-toto's Test Result among them, which verifiers outside tailrace already read. No record here ruled on a predicate type: cfe9e04 read core.7, core.8 and records 0005 and 0014 and found none that does. Which type does each claim tailrace signs take?

## Considered options

1. A type of tailrace's own for every kind, as phase 1 built.
2. The type the catalogue defines for what the claim says, wherever one does, and a type of tailrace's own only for a claim no catalogued type carries.
3. The catalogued type for the gate's verdict alone, as cfe9e04 first wrote 141, and tailrace's own for every other kind.

## Decision outcome

Option 2 (141, section 9). A provenance is SLSA's Provenance, `https://slsa.dev/provenance/v1`; an acceptance and a rebuild are SLSA's Verification Summary Attestation, `https://slsa.dev/verification_summary/v1`; a test is in-toto's Test Result, `https://in-toto.io/attestation/test-result/v0.1`. A review, a promotion and a carried claim are tailrace's own, since the catalogue carries none of them (7beb5fa). A claim another party made keeps the type its signer gave it, carried as made inside tailrace's own carried claim (12, core.8).

A claim in a catalogued type is read by any verifier that reads the catalogue, with nothing of tailrace's, and tailrace's verify reads another system's claims of the same kinds the same way: 142 has it verify a bundle of claims under a trusted root it is given, the open verifier's clause, carried because that verifier is to be a mode of tailrace's verify (cfe9e04), and 143 lets a policy name another system's trusted root as a signer. SLSA 1.2 recommends its Provenance for what a build produced. Option 1 leaves every claim legible to tailrace alone. Option 3 is the first reading of 141, which 7beb5fa widened to every claim tailrace signs, not only its verdicts, on the user's ruling.

A policy binds a signer to the kinds of claim it may make by the names the landing gives them (116), never by a predicate type, so a policy file names the same kinds whichever type a claim takes, and a claim counts through its signer and what that signer may claim, never through its type (core.7).

### Consequences

- A provenance, an acceptance, a rebuild and a test take the catalogued types, a change to the signer seam's form that no unit carves yet (`docs/architecture/_open.md`); a review, a promotion and a carried claim keep `https://agentplot.github.io/tailrace/claim/<kind>/v1`.
- The registry profile writes a claim's predicate type in the `in-toto.io/predicate-type` annotation and its envelope type as the artifact type (ADR 0014), so no form the store writes changes, and none carries a parameter (core.51).
- The catalogued types are named in the core beside the in-toto and DSSE forms, outside any profile (core.1).
- 142 and 143 come with it: a bundle of claims is verified under a trusted root tailrace is given, its certificates, inclusion proofs and passkey assertions, with no call to any service, and a claim under another system's root that a policy names counts as any named signer's, checked against the root as it stood when the claim was made (core.58, core.59). Neither is built; keyless signing (C.9) and the open verifier wait on the planner.
- A claim already signed under tailrace's own type for a catalogued kind stays attached and readable as made (10); what a verify answers for it waits on the change that moves the types (`docs/architecture/_open.md`).
