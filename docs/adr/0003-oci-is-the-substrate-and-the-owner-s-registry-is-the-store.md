---
number: 3
title: OCI is the substrate and the owner's registry is the store
status: accepted
date: 2026-10-22
decision-makers:
- Chuck Swanberg
---

# OCI is the substrate and the owner's registry is the store

Deciders: the user, 2026-10-22 (inception S1, leaning, and S7, raised by the user); the exploration of 2026-10-21, on a friend's reading that a package manager of our own is quaint beside an OCI registry.
Sources: `agentplot/blueprints:design/tailrace/requirements.md` clauses 1 to 5, 8, 66 to 73, C.1, C.2; the OCI image and distribution specifications 1.1, whose referrers carry attestations with a subject; ECR's referrers support, with aws/containers-roadmap 2306 and 2783 open on 2026-10-21.

## Context and problem statement

A package manager per ecosystem, an image registry, a bucket for templates and a deploy tool each store bytes under their own names, with their own accounts and copies, and none carries a signed claim beside the bytes. Where do artifacts live, and whose store is it?

## Considered options

1. A package manager per ecosystem (CodeArtifact, npm, PyPI) as the source of truth, with OCI for images alone.
2. One OCI registry as the substrate for everything; npm, Maven, PyPI, Nix and templates are projections rendered out of it; attestations are referrers on the digest.
3. One OCI registry tailrace standardises on (ECR), with no file layout and no second registry.

## Decision outcome

Option 2. OCI gives content addressing, immutability, a manifest with an artifact type and metadata, and referrers that carry a signed claim with the digest it is about; every registry the owner already runs speaks it, and a directory on disk does too. The store is always the owner's: a registry they run or rent, or a layout on their disk, never one tailrace holds for them, so every byte, signature and ledger entry stays in their account and a user can leave tailrace with nothing to migrate. Parts A and B name no registry; C.1 and C.2 bind the store.

### Consequences

- Every artifact is an OCI manifest whose artifact type names its ecosystem and whose config is the ecosystem's metadata (C.1), so an index can be built without pulling layers.
- A package manager, CodeArtifact among them, is at most a target an environment pushes to (C.12), never the source of truth.
- The ECR profile needs a reading of its open referrer issues before it is a profile; the OCI layout on disk needs none.
