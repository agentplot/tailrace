---
number: 6
title: Sync renders each ecosystem's own layout under the policy; serve only where a client reads no files
status: accepted
date: 2026-10-22
decision-makers:
- Chuck Swanberg
---

# Sync renders each ecosystem's own layout under the policy; serve only where a client reads no files

Deciders: the user, 2026-10-21 ("is serve the only way? what about syncing a local cache compatible with each ecosystem's local registry concept").
Sources: `agentplot/blueprints:design/tailrace/requirements.md` clauses 46 to 51, 84 to 87, C.11; the ecosystems' own local-registry forms: a `file://` Maven repository, `GOPROXY=file://`, Cargo's local registry and source replacement, a NuGet folder feed, a PEP 503 `file://` index, and npm, which reads no files.

## Context and problem statement

A consumer's client wants a registry it already understands. A localhost server speaking each protocol serves every ecosystem but is a daemon, and a daemon is the only way for none but npm. How does a consumer read, and where is the policy applied?

## Considered options

1. A localhost server for every ecosystem, applying the policy per request.
2. Sync renders each ecosystem's own local-registry layout from the store under the policy, every file a link into one blob store; serve is a static server over the layout for ecosystems whose clients read no files, and a miss runs the same render on demand.
3. A client plugin per ecosystem.

## Decision outcome

Option 2. The policy moves from serve to sync: a version has a layout entry only after its verdict is satisfied (47), so the layouts hold nothing a client should not have, and a refusal is printed where someone is looking, with its reason, instead of a status code a client cannot explain. Serve then needs no policy. Eager sync reads what the repository's lockfiles pin and fetches exactly those versions (50); lazy serve fetches one miss the same way. The recorded address is a fixed name or path on every host (49), so lockfiles carry no host.

### Consequences

- Maven, Go, Cargo, NuGet and Python need no server; npm does, and images serve the layout as a registry (C.11).
- A consumer layout is never a source of truth and can be deleted (48, 63).
- Plain pip beside PyPI is the dependency-confusion hole; uv's per-package index closes it, and whether plain pip is a supported client is open.
