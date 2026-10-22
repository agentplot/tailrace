---
number: 5
title: tailrace reads sluice and moon, and neither knows tailrace
status: accepted
date: 2026-10-22
decision-makers:
- Chuck Swanberg
---

# tailrace reads sluice and moon, and neither knows tailrace

Deciders: the user, 2026-10-21 ("either we build it into sluice or it's another standalone CLI"), resolved as standalone in the exploration and the inception pack (D9).
Sources: `agentplot/blueprints:design/tailrace/requirements.md` clauses 12, 13, 14, 74 to 78, C.3, C.5; sluice's statement clauses 1, 10, 23 and C.6, where a run record is an in-toto statement in a DSSE envelope with the test's key and system as subject and moon's hash as digest; `agentplot/blueprints:design/sluice/proposals/landing-handoff-1.md`.

## Context and problem statement

sluice already signs each check's result as an attestation, and moon already names each task's inputs by a hash. tailrace needs both: whether a revision landed, the verdicts of the landing, and the identity of the build whose outputs it packs. Does tailrace become a sluice feature, does sluice gain a hook that publishes, or does tailrace read what the two leave?

## Considered options

1. Build publishing into sluice as a step after the landing.
2. A hook in sluice that runs a command after a landing.
3. tailrace standalone, reading sluice's records and moon's state, joined by moon's task hash; whoever drives the tool after the landing turns the wheel.

## Decision outcome

Option 3. sluice lands and decides nothing more; its core starts no service (sluice 10), and a hook would make sluice the thing that publishes. tailrace reads the landing and the signed run records through the landing seam (77, 78) and the task hash and outputs through the build seam (74 to 76), and carries sluice's verdicts as sluice made them inside its own claim (12). The task hash is the join: sluice's subject digest and tailrace's provenance name the same number (13). What sluice owes is a stable read, drafted as landing-handoff-1 (clauses 120 to 122), never a hook.

### Consequences

- Code outside a profile names neither sluice nor moon (core.1); C.5 and C.3 bind them, and a plain branch and a declared command are the second profiles (C.6, C.4).
- Whoever runs a publish after a landing is outside tailrace: a watcher of sluice's log, a plan's step, a person, CI.
- A host trusted as a signer of sluice's verdicts is the same signer in tailrace's policy.
