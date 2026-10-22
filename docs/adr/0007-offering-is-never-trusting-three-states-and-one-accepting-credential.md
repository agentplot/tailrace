---
number: 7
title: Offering is never trusting: three states and one accepting credential
status: accepted
date: 2026-10-22
decision-makers:
- Chuck Swanberg
---

# Offering is never trusting: three states and one accepting credential

Deciders: the user, 2026-10-22 (inception T1, T2, P2: centralize trust, not execution; edges can never promote; a gate promotes by copying the same digest).
Sources: `agentplot/blueprints:design/tailrace/requirements.md` clauses 20, 25 to 30, 32 to 39, I2; the inception pack's threat model (`05-trust-and-security.md`) and gate (`06-gate-and-promotion.md`).

## Context and problem statement

Builds run on machines the developer controls, and a stolen build credential or a poisoned upload must not reach production. The forge bundles where code runs, where it is stored and who may do what; the enterprise needs only the third centralized. What is the smallest trusted center that keeps execution at the edge?

## Considered options

1. Two states, offered and released, with a trusted build farm the only party that may offer.
2. Rebuild everything in a trusted farm before release.
3. Three states, offered, accepted and released, where any host may offer, only the gate accepts by judging attached claims against the policy, and only a promotion the gate makes moves a channel; the offering credential and the accepting credential are never the same.

## Decision outcome

Option 3. The statement fixes the three states and the one accepting party (25, 27, 32); the choices this record makes are that the gate runs in the owner's trusted place and never on a host that builds (96), and that acceptance is a claim attached in place, never a copy to a second store (26). The gate is a verifier, never a builder: it reads claims and a policy, changes no bytes, and its acceptance is one more claim on the digest (26, 29). Where a policy asks for an independent rebuild, a second signer the first builder does not control makes it and attaches a claim; the gate counts the claim (30). A thief with a build credential can offer and write cache entries, and what it offered sits unaccepted and attributable (S9). Nothing accepted is ever un-accepted; a channel moves away (25, 38), and the ledger holds every move before anyone reads it (33, 34).

### Consequences

- No host that builds holds the accepting credential (27, core.14); the gate runs in the owner's trusted place.
- Whether a solo user may run both on one host, with one person's two credentials, is open on the book's open page.
- The ledger's source of truth where the store has no append-only primitive is an open question (101); the lean is git, with a referrer per promotion as the projection (39).
