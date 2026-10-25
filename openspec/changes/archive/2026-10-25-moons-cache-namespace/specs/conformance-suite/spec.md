# Spec Delta

## ADDED Requirements

### Requirement: The cache over the fakes

The cache's group SHALL hold the core to 52, 53, 54 and 100 over the
fakes, below its marker: an entry written on one host and rendered on
another, an entry held already, bytes an artifact and an entry share, an
entry removed, and S14. Each clause SHALL leave the conformance register
in the commit whose scenario first cites it.

#### Scenario: Written on one host, read on another

- **WHEN** one world writes the cache entries of a landed revision's passing builds and a second world, sharing only its store, syncs that revision
- **THEN** the second world's projection is asked to render exactly those build identities with their digests

#### Scenario: An entry is never an artifact

- **WHEN** an entry is written and synced
- **THEN** the store was asked to put no manifest, attach nothing and move no name but the entry's own, and no policy was asked

#### Scenario: One store object for an artifact and an entry

- **WHEN** a version is published and then an entry is written whose bytes are one of its artifacts' bytes
- **THEN** the entry's put answers held already, and the version verifies as before

#### Scenario: S14 over the fakes

- **WHEN** a revision's entry is removed from the fake store, the next landing's verdicts name a new build identity for that output, the old revision is synced and the new landing is published
- **THEN** the sync names the removed build identity as absent and refuses nothing, the publish offers the new outputs, and every name the store held before names the digest it named

#### Scenario: The register after the cache

- **WHEN** the coverage check runs after this group's scenarios are added
- **THEN** 52, 53, 54 and 100 are cited and none of them is in the register, and 55 is still pending for the verifier's rebuild
