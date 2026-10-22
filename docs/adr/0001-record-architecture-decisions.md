---
number: 1
title: Record architecture decisions
status: accepted
date: 2026-10-22
decision-makers:
- Chuck Swanberg
---

# Record architecture decisions

Deciders: the user, in the tailrace exploration of 2026-10-21 and 2026-10-22.
Sources: `agentplot/blueprints:design/tailrace/requirements.md`; `agentplot/blueprints:design/requirements-style.md`; sluice's `docs/adr/0001` as the model for the form; the inception pack of 2026-10-22, which tags each claim decided, leaning, proposed or open.

## Context and problem statement

tailrace starts with a statement of invariants and no code. The decisions that shaped the statement were made in two conversations and an inception pack, and a decision in a conversation alone is not recorded. Where do the decisions live, how do the rules cite them, and what is kept out of this public repository?

## Considered options

1. Decisions as prose in the README.
2. One MADR record per decision under `docs/adr/`, with the rules in `docs/architecture/` naming them as sources.
3. Decisions only in the blueprints statement and its proposals.

## Decision outcome

Option 2. The statement in blueprints says what is always true; the ADRs here say why a choice was made where the statement admits several; the rules in `docs/architecture/core.md` say what a diff must keep and cite both. A record is made with the `adrs` CLI, never by hand, and an accepted record's text is never edited: a decision that changes is a new record that supersedes it. Nothing commercial appears in this repository; commercial thinking, the hosted trust plane among it, belongs in agentplot/strategy.

### Consequences

- Every rule in `core.md` names a clause or an ADR as its source.
- A change that departs from a rule names it and says why, in the commit or the design.
- A claim the inception pack tags proposed or leaning is a record here only once the user rules on it; until then it is on the book's open page.
