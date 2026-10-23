---
number: 10
title: A version belongs to a declared component
status: accepted
date: 2026-10-23
decision-makers:
- Chuck Swanberg
---

# A version belongs to a declared component

Deciders: the user, 2026-10-23, ruling through Pending You (`agentplot/blueprints:design/tailrace/models/rulings.md`).
Sources: statement clauses 21, 22, 110, 123 to 125, C.10, the givens, scenario S15.

## Context and problem statement

A version is derived from the repository's history by the authors' rule, git-cliff over conventional commits, and the statement gave every artifact of a revision one version (110). A repository of the shape Switchboard delivers holds several manifests, each its own build with its own history: a library package, a service image, an infrastructure template. Under one version per revision, a landing that touches the template re-versions the package it did not touch, and every landing publishes every artifact. What takes a version?

## Considered options

1. Lockstep: every artifact of a revision takes the one version; an ecosystem with its own history keeps it as a profile under that version.
2. Declared components: the authors declare each component by its outputs, the paths whose changes version it and the prefix its tags carry; the version rule runs per component; a repository that declares none is one component.
3. Leave it open and rule after a measurement.

## Decision outcome

Option 2. A component is what takes a version (123). The rule runs once per component over the commits that touch its paths, against the tags that carry its prefix, which is `git-cliff-core`'s own path filter and tag pattern called once per component from the one binary; a repository that declares no component is one component over the whole tree with the rule's default prefix, so a small repository sees no difference. A publish names one component, or every component a commit since its last tag touched, and one component's publish never gives another a version, an index or a move (124). A version's name is its component's (22), the index is per component (110), and a channel belongs to one component and names only its versions (125). Every component is declared in the one policy file at the landed revision, so a host that runs a separate job per manifest runs a publish per component against the same policy.

### Consequences

- The version rule is a function of a component and a revision, never of a revision alone, and no code path derives a version without naming the component (core.37).
- A channel's name carries its component, and promotion never crosses components.
- Whether an environment may be admitted beyond the policy file, and follow a channel of its own, is open (statement section 10).
