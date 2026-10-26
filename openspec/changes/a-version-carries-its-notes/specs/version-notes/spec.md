## Purpose

A version's notes: rendered from the commits the version was derived
from by the version rule's own tool, carried as an artifact of the
version named in its index and claimed with the rest, and never written
again once offered (134).

## ADDED Requirements

### Requirement: The notes are rendered from the commits the version is derived from

When the version rule derives a component's version, the same run SHALL
render the version's notes from the same commits: those the landed
revision reaches since the component's last release tag that touch its
paths, by the rule's own tool, git-cliff (21, 123, 134, section 9). A
prerelease's notes hold every commit since the last release.

#### Scenario: A release's notes

- **WHEN** a component whose last release tag is `v1.2.3` is published at a revision reached by a `feat:` commit and a `fix:` commit touching its paths and a `docs:` commit touching another component's paths
- **THEN** the notes name the version `1.3.0`, hold the feature under the features and the fix under the fixes, each with its subject and short revision, and hold nothing of the `docs:` commit

#### Scenario: A prerelease's notes

- **WHEN** a component whose last release tag is `v1.2.3` is published at a revision reached by two `chore:` commits touching its paths
- **THEN** the version is `1.2.4-pre.2` and its notes hold both commits under the rest

#### Scenario: A commit that is not conventional

- **WHEN** a commit whose message follows no conventional form touches the component's paths since its last release tag
- **THEN** the notes hold it under the rest, by its subject, and drop no commit the version was derived from

### Requirement: The same commits render the same notes

The notes SHALL be a function of the version, the last release tag and
the commits alone: no date, clock, host or path of the publisher's SHALL
appear in them, so the same revision under the same rule renders the
same bytes on every host (18, 21).

#### Scenario: Two hosts, one revision

- **WHEN** the same landed revision's notes are rendered in two clones of the repository at different paths, at different times
- **THEN** the two renders are the same bytes

### Requirement: The notes are an artifact of the version named in its index

A publish SHALL put the notes' bytes in the store, put an artifact of
the ecosystem `notes` naming them, named by the component at the
version, and name it in the version's index beside the outputs'
artifacts, after the version's name is claimed and before the index is
put (4, 6, 17, 22, 134, core.12, core.13).

#### Scenario: The index names the notes

- **WHEN** a component with two outputs is published
- **THEN** the index names three artifacts, the two outputs' and one of ecosystem `notes` named by the component at the version, whose bytes are the rendered notes under their own digest

#### Scenario: Nothing of the notes before the claim

- **WHEN** the version's name is refused because another revision holds it
- **THEN** the fake store records no put of the notes' bytes and no artifact of ecosystem `notes`

#### Scenario: A publish that stops keeps no notes as a version

- **WHEN** a publish stops after the notes are put and before the index
- **THEN** no index names the notes and the version's name resolves to nothing (17)

### Requirement: The notes carry the publisher's own claim

The notes SHALL be attached one provenance claim signed in the identity
of the party running the publish, about the notes' digest, naming the
revision, the component, the version, the last release tag and each
commit the notes were rendered from, and no build identity, since no
build made them (8, 11, 13, core.8, core.9).

#### Scenario: The notes' claim

- **WHEN** a version is published
- **THEN** claims-of the notes' digest answers one provenance claim signed by the host's signer, naming the revision and every commit the notes hold, and naming no build identity

#### Scenario: No identity at hand

- **WHEN** the fake signer has no identity at hand
- **THEN** the publish answers absent naming the signer, and no index is put and no version's name moved

### Requirement: Nothing writes a version's notes after it is offered

A publish of a revision whose version's index is held SHALL render the
notes again and compare their digest with the bytes of the `notes`
artifact the index names. The same, nothing SHALL be written; different,
the publish SHALL be refused naming the notes and both digests, and
nothing SHALL be offered. An index that names no notes, one published
before notes were carried, SHALL be left as it is and no notes added
(18, 134).

#### Scenario: The same notes again

- **WHEN** a revision is published twice
- **THEN** the second publish puts and attaches nothing, and its answer names the same notes' digest as the first

#### Scenario: Notes that render differently

- **WHEN** a revision is published, the notes the index names are replaced in the fake store's index by other bytes' artifact, and the revision is published again
- **THEN** the second publish is refused naming the notes, the digest the index names and the digest rendered now, and the store records no put

#### Scenario: A version published before notes

- **WHEN** a revision whose index names no `notes` artifact is published again
- **THEN** the publish writes nothing, answers the same index, and answers no notes

### Requirement: No output is packed in the notes' ecosystem

The policy file SHALL be refused when it is read where any output of any
component names `notes` as the ecosystem that packs it, naming the
component and the output, before the build is asked anything (C.10).

#### Scenario: An output packed as notes

- **WHEN** a policy file declares `[projections] changelog = "notes"`
- **THEN** reading it is refused naming the component and the output `changelog`, and the fake build records no call
