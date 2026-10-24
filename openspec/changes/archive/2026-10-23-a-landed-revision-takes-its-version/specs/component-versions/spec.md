# Spec Delta

## Purpose

The version a landed revision takes for each declared component, derived
from the repository's history alone, and the claim of that version's
name in the history every publisher shares before anything is packed.

## ADDED Requirements

### Requirement: A version is derived from history alone

A version SHALL be derived from the repository's history at a landed
revision by the authors' rule, for one named component, and from
nothing a publisher supplies. The same revision under the same rule
SHALL always derive the same version (21, core.13, core.37).

#### Scenario: The same revision twice

- **WHEN** the version of one component is derived twice at the same landed revision under the same rule
- **THEN** both derivations give the same version

#### Scenario: No version without a component

- **WHEN** a version is derived
- **THEN** the derivation names the component it is for, and its tag carries that component's prefix

### Requirement: The rule runs over the component's own commits and tags

The rule SHALL run over the commits reachable from the landed revision
since the component's last release tag that touch the component's
paths, against the tags that carry the component's prefix in the shared
history as well as the local one. A commit that touches none of its
paths SHALL not version it (22, 123).

#### Scenario: A tag another publisher claimed

- **WHEN** the shared history holds a release tag with the component's prefix, on a revision the landed revision reaches, that the local history lacks
- **THEN** the derivation reads that tag as the component's last release tag

#### Scenario: A commit elsewhere in the repository

- **WHEN** the only commit since a component's last release tag touches no path the component names
- **THEN** that component gets no new version

#### Scenario: Another component's tags are not read

- **WHEN** a second component has a later tag under its own prefix
- **THEN** the first component's version is derived from its own prefix's tags alone

### Requirement: A feature or a fix makes a release, anything else a prerelease

When a commit the rule runs over is a feature, a fix or a breaking
change, the version SHALL be a release, numbered by the conventional
bump from the last release tag. Otherwise it SHALL be a prerelease of
the next patch, numbered by how many of the component's commits
follow the last release tag (section 9, 21).

#### Scenario: A feature since the last release

- **WHEN** a component's last release tag is at 1.2.3 and a feature commit touching its paths follows
- **THEN** the version is 1.3.0 under the component's prefix

#### Scenario: A fix since the last release

- **WHEN** a component's last release tag is at 1.2.3 and only a fix touches its paths since
- **THEN** the version is 1.2.4

#### Scenario: Only housekeeping since the last release

- **WHEN** a component's last release tag is at 1.2.3 and three commits that are neither a feature, a fix nor a breaking change touch its paths since
- **THEN** the version is a prerelease of 1.2.4 numbered 3, which orders after 1.2.3 and before 1.2.4

#### Scenario: A component never released

- **WHEN** no release tag carries the component's prefix
- **THEN** the version is derived from the rule's initial version

### Requirement: Tags are ordered by the version rules, never by time

The component's last release tag SHALL be the highest by semantic
version precedence among the release tags carrying its prefix that the
landed revision reaches, and never chosen by time, by spelling or by the
order git lists them (23).

#### Scenario: A later tag of a lower version

- **WHEN** a tag of 1.10.0 and a tag of 1.9.0 made after it both carry the prefix and are reachable
- **THEN** the last release tag is 1.10.0

### Requirement: A revision already claimed keeps its version

When a tag carrying the component's prefix already names the landed
revision, the version SHALL be that tag's, so a second publish of the
revision derives the version the first claimed (18, 21).

#### Scenario: Deriving after the claim

- **WHEN** a component's version has been claimed at a revision and is derived again at that revision
- **THEN** the version is the one claimed

### Requirement: A publish versions the components a landing touched

The core SHALL answer, for a landed revision, every component a commit
since that component's last release tag touched. A commit under a path
two components name SHALL version both, and a component no such commit
touched SHALL get no version, index or move (124, S15).

#### Scenario: A landing that touches one component

- **WHEN** a repository declares two components and the commits since their last tags touch only the first's paths
- **THEN** only the first component is answered, its version derived from its own tags under its own prefix, and the second gets no version

#### Scenario: A path two components name

- **WHEN** two components both name a path and a commit since their last tags touches only that path
- **THEN** both components are answered, each with a version under its own prefix

### Requirement: The name is claimed in the shared history before anything is packed

The version's name SHALL be claimed as a tag on the landed revision in
the repository's history every publisher shares, before any artifact is
packed. A repository whose history no other publisher shares SHALL claim
in its own history alone (22).

#### Scenario: A new name claimed

- **WHEN** a version's name is claimed at a landed revision and no tag of that name exists
- **THEN** the tag names the revision in the local history and in the shared history

#### Scenario: The same revision claimed again

- **WHEN** the name is claimed again at the revision that already holds it
- **THEN** the claim succeeds, nothing in the shared history changes, and it is not an error

#### Scenario: A history no one else shares

- **WHEN** the branch has no shared history beyond the local repository
- **THEN** the name is claimed as a tag in the local history alone

### Requirement: A name another revision holds refuses the publish

A claim of a name another revision already holds, in the local or the
shared history, SHALL be refused naming the name and both revisions,
carrying git's own words where git refused, and SHALL move no tag (22,
59, core.6).

#### Scenario: Two publishers, one name

- **WHEN** the shared history holds the name at another revision and the name is claimed at this one
- **THEN** the claim is refused naming the name, this revision and the other, with git's words verbatim, and no tag is moved

#### Scenario: The shared history cannot be reached

- **WHEN** the shared history cannot be reached while claiming
- **THEN** the answer is unreadable, naming the shared history and carrying git's words, never refused and never claimed

### Requirement: The build never learns the version

Deriving and claiming a version SHALL ask nothing of the build seam, so
no build is handed a version (24, core.13).

#### Scenario: No build asked

- **WHEN** a version is derived and claimed over the fakes
- **THEN** the fake build records no call
