---
number: 4
title: The version is derived from history after the landing, and the build never knows it
status: accepted
date: 2026-10-22
decision-makers:
- Chuck Swanberg
---

# The version is derived from history after the landing, and the build never knows it

Deciders: the user, 2026-10-21, in the exploration of how a publish fits a merge queue ("build creates the files, but we don't push until we get a version").
Sources: `agentplot/blueprints:design/tailrace/requirements.md` clauses 21 to 24, 15, 18; git-cliff over conventional commits as the family's version rule (section 9); the family's record that a release is versioned by git-cliff (switchboard-kit ADR, record 0074).

## Context and problem statement

The gate builds and tests a candidate before it lands, and the version is not known then: it depends on which landings came before. Most ecosystems write the version into the bytes. When does a version exist, who decides it, and does the build have to run again once it does?

## Considered options

1. Stamp a version into the build before the gate runs it, from the branch's state at submission.
2. Derive the version from history after the landing, tag it first, and pack the gate's cached, version-free outputs with the version at publish.
3. Rebuild at publish with the version known.

## Decision outcome

Option 2. The statement fixes that the version is history's and that the build never knows it (21, 24); the choice this record makes is how the name is claimed: as a git tag pushed before anything is packed, so the repository's shared history is the one place every publisher sees (22), and a layout with no shared history has one publisher. The version is a pure function of the landed history under the authors' rule (21), claimed by pushing the tag before anything is offered (22), so two publishers cannot both succeed and the loser derives the next. The build target stays version-free and expensive, cached by its hash and verified by the gate; the pack target is cheap and takes the version as its argument, writing it only where the ecosystem's form requires it (24). The build identity that sluice's verdict carries is the build target's hash, which the version never enters, so the same build is the same build whatever version it becomes, and no rebuild is ever needed to make the bytes that passed into the bytes that run.

### Consequences

- A publish refuses a revision the branch has not landed, before anything else (14, core.11).
- A second publish of the same revision ends at the same index provided packing is reproducible, and the publish says which artifact differed when it is not (18).
- Two stages fall out: a build artifact per landing, and a release when a version exists, which can run anywhere the store is reachable.
