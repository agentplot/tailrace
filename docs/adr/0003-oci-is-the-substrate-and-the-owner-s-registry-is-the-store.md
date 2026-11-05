---
number: 3
title: OCI is the substrate and the owner's registry is the store
status: accepted
date: 2026-10-22
decision-makers:
- Chuck Swanberg
---

# OCI is the substrate and the owner's registry is the store

Status: accepted on 2026-10-22; amended on 2026-11-02 to follow C.12 as amended at `agentplot/blueprints` 3d7827b, as the amendment below says.

Amendment (2026-11-02, following C.11 and C.12 as `agentplot/blueprints` 3d7827b leaves them, on the user's word of 2026-10-26): Nix is no projection, and C.11 names none for it; Nix is reached as two of C.12's targets. A Nix fleet is bound by the environment to two profiles (44): first a push, whose apply runs the fleet's own deploy tool against one machine from a host that holds that machine's key and whose report is what the machine answers after, its current the closure the machine runs; then a pull, replacing the push machine by machine, in which each machine on its own timer reads its channel's tag in the store with its own credential, verifies the gate's claim, realises the version's store paths from the cache and switches, its system profile its marker (119) and its report a claim it signs. A Nix binary cache is pushed the version's store paths. How a closure of many digests maps onto artifacts and the cache stays section 10's question. The rest of the record stands as accepted, its projections npm, Maven, PyPI and templates.

Deciders: the user, 2026-10-22 (inception S1, leaning, and S7, raised by the user); the exploration of 2026-10-21, on a friend's reading that a package manager of our own is quaint beside an OCI registry.
Sources: `agentplot/blueprints:design/timber/requirements.md` clauses 1 to 5, 8, 66 to 73, C.1, C.2; the OCI image and distribution specifications 1.1, whose referrers carry attestations with a subject; ECR's referrers support, with aws/containers-roadmap 2306 and 2783 open on 2026-10-21.

## Context and problem statement

A package manager per ecosystem, an image registry, a bucket for templates and a deploy tool each store bytes under their own names, with their own accounts and copies, and none carries a signed claim beside the bytes. Where do artifacts live, and whose store is it?

## Considered options

1. A package manager per ecosystem (CodeArtifact, npm, PyPI) as the source of truth, with OCI for images alone.
2. One OCI registry as the substrate for everything; npm, Maven, PyPI, Nix and templates are projections rendered out of it; attestations are referrers on the digest.
3. One OCI registry timber standardises on (ECR), with no file layout and no second registry.

## Decision outcome

Option 2. OCI gives content addressing, immutability, a manifest with an artifact type and metadata, and referrers that carry a signed claim with the digest it is about; every registry the owner already runs speaks it, and a directory on disk does too. The store is always the owner's: a registry they run or rent, or a layout on their disk, never one timber holds for them, so every byte, signature and ledger entry stays in their account and a user can leave timber with nothing to migrate. Parts A and B name no registry; C.1 and C.2 bind the store.

### Consequences

- Every artifact is an OCI manifest whose artifact type names its ecosystem and whose config is the ecosystem's metadata (C.1), so an index can be built without pulling layers.
- A package manager, CodeArtifact among them, is at most a target an environment pushes to (C.12), never the source of truth.
- The ECR profile needs a reading of its open referrer issues before it is a profile; the OCI layout on disk needs none.
