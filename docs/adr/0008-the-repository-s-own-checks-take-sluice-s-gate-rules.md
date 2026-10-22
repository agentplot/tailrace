---
number: 8
title: The repository's own checks take sluice's gate rules
status: accepted
date: 2026-10-22
decision-makers:
- Chuck Swanberg
---

# The repository's own checks take sluice's gate rules

Deciders: the user, by the family's practice; sluice's ADRs 0029, 0030 and 0040, which the clean-room review of 2026-10-22 asked this repository to adopt as its own record rather than cite across repositories.
Sources: sluice's `docs/architecture/core.md` core.21 and core.24; this repository's `.moon/workspace.yml`, `.prototools` and `devenv.nix`, scaffolded from headwater's.

## Context and problem statement

This repository runs its own checks through moon under a devenv shell, as every member of the family does, and will land through sluice. What binds the shape of those checks, and the pins the tools come from, so that a check here means the same as a check in sluice's or headwater's tree?

## Considered options

1. Cite sluice's rules from this repository's rules and keep no record here.
2. Adopt the two rules as this repository's own, core.25 and core.26, with this record as their source, so a change to sluice's rules does not silently change this repository's.
3. Write check rules of this repository's own from scratch.

## Decision outcome

Option 2. Every tracked file is owned by a formatter the gate runs or excluded by name with its reason; every language with a linter is linted; `site/`, and every crate and plugin directory, is a moon project of its own whose check-fast tasks validate, type-check and test it, one task per check (core.25). Every tool a gate task runs is the version the tree pins, on every host that runs the gate (core.26). The site's build is its one check until it has tests. A seam with a single profile is documented as such in the record that bound it (core.3), so that a second profile is a known gap.

### Consequences

- The first crate adds its project to `.moon/workspace.yml` with its tasks, in the same change.
- A pin lives in one file; a second file restating it carries a check that fails when they differ.
