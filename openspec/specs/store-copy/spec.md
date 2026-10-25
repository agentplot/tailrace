# store-copy Specification

## Purpose
A version copied from one store to another through the store seam
alone, carrying every attestation attached to its digests, so a layout
on a developer's host can be synced to a registry (103, C.2, section 9).

## Requirements

### Requirement: A version is copied whole, the index last

Copying a version SHALL resolve its name in the source store to its
index, then put into the target every bytes each artifact names, each
artifact, and the index last, reading each from the source by digest
(3, 6, core.4, core.12). Each digest the target answers SHALL be the
source's; another digest SHALL be refused naming both (1, 2).

#### Scenario: A version copied

- **WHEN** a version of two artifacts held in one store is copied to an empty store
- **THEN** the target holds the index and both artifacts under the source's digests, and reading each answers what the source answers

#### Scenario: A target that names another digest

- **WHEN** the target answers an artifact's put with a digest other than the source's
- **THEN** the copy is refused naming both digests, and the index is not put

### Requirement: Every attestation goes with its digest

Copying a version SHALL attach in the target every attestation the
source holds on the index and on each artifact, byte for byte, before
the version's name is made there (8, 103, core.27).

#### Scenario: Claims carried

- **WHEN** a version whose index and artifacts carry a provenance, a verdict and an acceptance is copied
- **THEN** claims-of in the target answers each of them on its digest, byte for byte

### Requirement: The version's name is made, and no channel is copied

Copying a version SHALL make its name in the target from nothing, last.
A name the target holds at the same index SHALL be answered as already
there; at another digest it SHALL be refused naming that digest, with
nothing retried (70, core.6). A channel's name SHALL never be copied,
since a channel is the repository's and not a store's (103).

#### Scenario: Copied again

- **WHEN** a version already copied is copied again
- **THEN** nothing is written, and the answer says each digest and the name were held already

#### Scenario: A name held elsewhere in the target

- **WHEN** the target holds the version's name at another digest
- **THEN** the copy is refused naming that digest, after the artifacts, the index and the attestations are held, and the name is not moved

### Requirement: A copy reads what it cannot find as the source answered

A digest the source does not hold SHALL make the copy absent naming
it, and a source or target that cannot be read SHALL make it
unreadable in that store's words; the copy SHALL never answer a
version as copied that the target does not hold whole (58, 59).

#### Scenario: A source missing an artifact

- **WHEN** the source's index names an artifact the source does not hold
- **THEN** the copy is absent naming that artifact's digest, and the target holds no index for the version

### Requirement: A layout synced to a registry

A version published into a layout SHALL copy into a registry and read
back there with the same digests and the same claims (103, C.2).

#### Scenario: A layout to zot

- **WHEN** a version published into a layout is copied into zot
- **THEN** zot answers the version's name at the layout's index digest, every artifact under the layout's digest, and every attestation the layout holds on each, byte for byte
