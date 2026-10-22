---
number: 2
title: One tool organized around artifacts, channels and environments, with publish standing alone
status: accepted
date: 2026-10-22
decision-makers:
- Chuck Swanberg
---

# One tool organized around artifacts, channels and environments, with publish standing alone

Deciders: the user, 2026-10-22 (inception D4, D9, D5; the layering requirement of `03-tailrace-core-model.md`).
Sources: `agentplot/blueprints:design/tailrace/requirements.md` clauses 6, 17, 20, 31, 40, 92; the exploration of 2026-10-21, where the verbs were drawn as push, attest, publish, verify, sync, serve.

## Context and problem statement

What a build left has three lives: a cache entry, an artifact a consumer installs, a release an environment runs. Each is a product today, and a tool could be built for any one of them. Is tailrace one tool or three, and what must stand alone?

## Considered options

1. Three tools: a remote cache, a package publisher, a release manager.
2. One tool around three concepts, artifacts, channels and environments, where publish stands alone and channels and environments are optional layers over it.
3. One tool that is only useful whole.

## Decision outcome

Option 2. One content-addressed store is what makes a cache entry, an artifact and a release the same bytes at three levels of trust, and three tools would each hold a copy. Publish stands alone: a publish offers and does nothing more (20), its answer is machine-readable (19), and a deployment that owns its own channels and promotion, Switchboard's, consumes that answer with tailrace's channel layer off. A repository turns on channels and environments by declaring them, and nothing in publish changes when it does.

### Consequences

- The core knows artifacts, claims, versions, channels, environments and nothing of any ecosystem, registry, build or target: those are seams (ADR 0003, 0005, 0006).
- `publish` never reaches a channel or a target (core.10); the Switchboard hybrid is a consequence, not a special case.
- The cache is in the same store under the least-trusted name, never a product of its own (52, 53).
