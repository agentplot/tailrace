---
number: 9
title: The ledger is a signed chain and its keeper is a seam
status: accepted
date: 2026-10-22
decision-makers:
- Chuck Swanberg
---

# The ledger is a signed chain and its keeper is a seam

Deciders: the user, 2026-10-22, ruling on interview 1 (`agentplot/blueprints:design/tailrace/models/interview-1-answers.md`), with the instruction that the family's habit of keeping records in git is not a reason and that tailrace is designed as a standalone tool whose backend may be anyone's.
Sources: statement clauses 33 to 39, 41, 97, 101, 105, 106, 108, 109, 120 to 122, C.13, C.14; ADR 0007, whose two open consequences this record settles.

## Context and problem statement

Every move of a channel must be in the ledger before anyone reads it, the ledger is append-only, and a channel moves only from the version it is at (33 to 35). The OCI distribution protocol has no conditional tag put, and a registry's retention rules can expire untagged artifacts by age. Where does the ledger live so that these hold for every store, and so that a person who promotes from a phone never commits anything?

## Considered options

1. A git ref is the ledger; the tag and a claim in the store are projections of it.
2. The store is the ledger: promotion claims attached to the versions they move to, the tag moved by expected digest where the registry can and read back elsewhere.
3. The ledger is the chain itself: each promotion a claim the gate signs that names its predecessor's digest, ordered and tamper-evident wherever it is held; where it is held is a seam, the keeper, bound by the operator per host.

## Decision outcome

Option 3. The chain is the truth and the keeper holds it (105, 106, 120 to 122). The gate alone appends, so it serializes promotions and no registry compare-and-swap is needed; a fork, two claims naming one predecessor, is reported and never resolved by choosing. The same claims are attached in the store to the versions they move to (39), and where the store disagrees with the keeper the store is rewritten from the keeper. The standalone tool's default keeper is a git ref per channel that the gate's own identity pushes (C.13), because a fast-forward push refuses a stale predecessor for every store alike and git does not expire what it holds; a store keeps the chain alone only where the operator can trust it to retain untagged claims (C.14). A hosted backend names its own append-only log as its keeper and git does not appear. An approval is a promotion (97): an environment that waits follows a channel of its own, bound to the one it waits on, so every yes is in the ledger. A solo host may offer and accept under a policy that names it as the gate, and its acceptance claim says so (108).

### Consequences

- The core names no git in its ledger code; the keeper is reached through its seam like the store (core.1, core.2), and the git ref is a profile.
- A promotion is appended only after its predecessor matches the chain's head, and a fork is an answer, never a choice (core.34).
- No code path converges a waiting environment except by its own channel moving (core.35).
- ADR 0007's two open consequences, the solo host and the ledger's record, are settled here.
