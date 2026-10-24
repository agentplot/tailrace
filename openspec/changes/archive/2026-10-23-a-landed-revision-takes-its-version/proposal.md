# Proposal

## Why

Nothing can be packed until a landed revision has a version, and the
version has to come from the repository's history alone, per declared
component, and be claimed where every publisher sees it before anything
is offered (21, 22, 123; ADR 0004, 0010). The seams are in place; this
is the first piece of the core that uses them, and publish, the next
unit, starts from the version this one claims.

## What Changes

- A revision counts as landed on a branch only when the landing seam
  says the branch holds it (77). Until it does, nothing of the
  repository is read at that revision, and the answer is a refusal
  naming the branch (14, 65).
- The policy file, `.config/tailrace.toml`, is read from the
  repository's history at the landed revision, never from a checkout's
  working tree, so an edit a publisher has not landed changes nothing
  (65, C.10, core.22). From it the core takes the authors' components,
  each with its outputs, the paths whose changes version it and its tag
  prefix, and the version rule (123). A repository with no file, or a
  file that declares no component, is one component over the whole
  repository with the rule's default prefix.
- The policy for an audience is asked of the policy seam only for a
  revision the landing seam has said is landed (65, 82, 83).
- The version rule runs per component: over the commits since the
  component's last release tag that touch its paths, against the tags
  that carry its prefix. A feature, a fix or a breaking change since
  that tag makes a release, numbered by git-cliff's conventional bump;
  anything else makes a prerelease. The same revision under the same
  rule always derives the same version (21, section 9; core.37).
- A commit under a path two components name versions both, and a
  component whose paths no commit since its last tag touched gets no
  new version (124, S15). The core answers which components a landed
  revision touched.
- The version's name, the component's prefix and the version, is
  claimed as a tag on the revision in the shared history before
  anything is packed. Claiming a name the same revision already holds
  is not an error; a name another revision holds refuses the publish,
  naming both revisions, in git's own words beside tailrace's (22, 18,
  59).
- Deriving and claiming ask nothing of the build seam: the build never
  learns the version (24, core.13).

Out of scope, and what each waits on:

- Packing, offering, the index and the publish answer, status and
  verify: `publish-status-and-verify-from-one-core`, which calls this
  unit's derive and claim for each component it publishes.
- The conformance suite's scenarios, S15 among them, over the fakes:
  `the-conformance-suite-over-the-fakes`.
- The policy seam's own profile over the file, which answers the claims
  each audience requires, its signers, channels and environments (82,
  83, C.10): no unit of this bolt holds it, and the fake policy stands
  in. A channel belonging to one component (125) arrives with the
  channels, bolt 4.
- A host's bindings file naming which remote is the shared history
  (118): bolt 2.

## Capabilities

### New Capabilities

- `landed-policy`: what the core reads of a repository at a revision
  the landing seam says is landed: the components and the version rule
  from the policy file at that revision, and the policy for an audience.
- `component-versions`: the version a landed revision takes per
  component, which components it touched, and the claim of that
  version's name in the shared history.

### Modified Capabilities

None.

## Impact

- `crates/tailrace-core/`: new modules for the landed revision, the
  repository's history, the declared components and rule, and the
  version, with their tests over temporary git repositories and the
  fakes.
- `crates/tailrace-seams/src/answer.rs`: the refusals this unit makes
  (not landed; a name claimed by another revision) and the repository's
  history as a source that answers, as the seams' design leaves to the
  core's units.
- `Cargo.toml`, `Cargo.lock`: workspace dependencies on
  `git-cliff-core`, `git2`, `glob`, `regex` and `semver`, and
  `tempfile` for tests.

## Touches

```
Cargo.toml
Cargo.lock
crates/tailrace-core/**
crates/tailrace-seams/src/answer.rs
```
