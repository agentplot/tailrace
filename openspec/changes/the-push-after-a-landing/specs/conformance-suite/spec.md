# Spec Delta

## ADDED Requirements

### Requirement: The push over the fakes

The cache's group SHALL hold the push to 52, 53, 54 and 77 over the
fakes, below the scenarios the cache already holds: a landing's passing
builds pushed, a revision not landed, a push repeated, an archive not on
the host, and a push that offers and keeps nothing.

#### Scenario: Pushed on one host, synced on another

- **WHEN** one world pushes a landed revision and a second world, sharing only its store, syncs that revision
- **THEN** the second world's projection is asked to render exactly the build identities the landing's passing verdicts name, with the digests the first world wrote

#### Scenario: A push offers and keeps nothing

- **WHEN** a landed revision is pushed
- **THEN** the world's writes hold one put of bytes and one name per build identity and no manifest or attachment, no signer or policy was asked, and the host's record is as it was

### Requirement: S12 over the fakes

The record's group SHALL hold the core to 63 over the fakes, below its
marker: everything kept deleted and rebuilt to the same state, playing
S12; a claimed version with no index; a rebuild that writes only the
record; and a rebuild that cannot read the store. Clause 63 SHALL leave
the conformance register in the commit whose scenario first cites it.

#### Scenario: S12, the record deleted and rebuilt

- **WHEN** a world publishes two components' versions, its host's record is deleted, and the record is rebuilt
- **THEN** the record's files hold the bytes they held before, and status answers what it answered before

#### Scenario: The register after this unit

- **WHEN** the coverage check runs
- **THEN** 63 is cited by the record's group and is not in the register
