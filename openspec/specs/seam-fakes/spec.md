# seam-fakes Specification

## Purpose
One in-memory fake for each seam, keeping the contract its seam states
and scriptable per test, so tailrace's core and its conformance suite
run with no registry, task runner, merge queue, key or target on the
host.

## Requirements

### Requirement: One fake per seam

There SHALL be exactly one fake for each of the eight seams, each
implementing that seam's interface and no other (core.2), holding its
state in memory, and needing nothing outside the process: no network,
no file outside a test's own, no key and no tool.

#### Scenario: The core bound to fakes alone

- **WHEN** a test binds every seam to its fake and asks each fake every one of its operations
- **THEN** each answers with no network, no task runner, no merge queue and no key reached

### Requirement: A fake keeps its seam's contract

A fake's answers SHALL keep what Part B says of its seam, so that a
fake passes the conformance suite as a profile must (93): bytes held by
digest and never altered (73), the same bytes put twice held once (66,
5), a name moved only from where it is expected (70), a chain never
reordered (122).

#### Scenario: The same bytes put twice

- **WHEN** the fake store is given the same bytes twice
- **THEN** the second answer names the same digest and says the bytes were held already, and the store holds them once

#### Scenario: A name moved from where it is not

- **WHEN** the fake store is asked to move a name from a digest it is not at
- **THEN** it refuses naming the digest the name is at, and the name stays

#### Scenario: Attestations travel with their digest

- **WHEN** an attestation is attached to a digest the fake store holds
- **THEN** claims-of that digest answers it as attached, and claims-of a digest the store does not hold answers absent

#### Scenario: The keeper refuses a stale predecessor

- **WHEN** two promotions are appended to one channel naming the same predecessor
- **THEN** the first is held at the head and the second is refused naming it

### Requirement: Scriptable per test

Each fake SHALL be set up by the test that uses it, with what its seam
holds (the revisions a branch has landed and their verdicts, the
outputs a build leaves and their identities and outcomes, the policy
for each audience, the signers and their identities, a projection's
ecosystem, an environment's channel head and what it runs), and SHALL
share that state with every copy the test hands to the core.

#### Scenario: A landing scripted for one test

- **WHEN** a test scripts the fake landing to have landed one revision on a branch, with two verdicts
- **THEN** landed answers yes for that revision and no for any other, and verdicts-of answers the two verdicts as scripted

#### Scenario: State seen through a copy

- **WHEN** a test keeps one handle to a fake store and puts bytes through a copy of it
- **THEN** the test's handle sees the bytes held

### Requirement: Scripted refusals and unreadable answers

A test SHALL be able to make any operation of any fake answer refused
or unreadable, with words of the test's choosing as the source's own,
once or until the test stops it, and the fake SHALL answer those words
verbatim (58, 59).

#### Scenario: A store that cannot be reached

- **WHEN** a test makes the fake store's claims-of unreadable with the words `connection refused`
- **THEN** claims-of answers unreadable carrying `connection refused` verbatim, and never an empty list

#### Scenario: A fault scripted once

- **WHEN** a test makes the next put-bytes refused, and put-bytes is asked twice
- **THEN** the first is refused with the scripted words and the second holds the bytes

### Requirement: Scripted faults the core must catch

The fakes SHALL let a test script what the core is bound to detect: a
store holding other bytes under a digest (2), a build leaving an output
unmade or failed (74, 16), a signer whose identity has expired (81) or
an attestation whose bytes were changed after signing, a projection
whose packing is not the same twice (87), and a keeper holding a fork
(105).

#### Scenario: Other bytes under a digest

- **WHEN** a test makes the fake store hold other bytes under a digest it holds
- **THEN** read of that digest answers those other bytes, so the core can refuse naming both

#### Scenario: An attestation changed after signing

- **WHEN** an envelope the fake signer made has its claim changed, and is verified against that signer
- **THEN** verify refuses it

#### Scenario: A fork held

- **WHEN** a test makes the fake keeper hold two claims naming one predecessor
- **THEN** chain refuses naming the fork and both claims

### Requirement: Every call recorded

Each fake SHALL record every operation it is asked, in order, with what
it was given and what it answered, readable by the test, so a test can
check what the core asked, in what order, and that nothing was written.

#### Scenario: Nothing written the second time

- **WHEN** the same bytes and artifact are put twice into the fake store
- **THEN** the record shows both asks, and the fake store's contents after the second are what they were after the first

#### Scenario: The order of asks

- **WHEN** two artifacts and then an index are put into the fake store
- **THEN** the fake store's record shows the index put after both artifacts

### Requirement: The fake projection packs cache entries or none

The fake projection SHALL pack an output given a build identity and no
version as a cache entry, whose bytes name the build identity and the
output's files and no ecosystem, name or version, and SHALL render
cache entries as a layout holding their build identities (84, 85). A
test SHALL be able to script it to pack no cache entries, after which
both answer absent for one.

#### Scenario: A cache entry packed by the fake

- **WHEN** the fake projection packs one output twice at one build identity with no version, and once at another
- **THEN** the first two answers are the same bytes with no metadata, the third is other bytes, and none of them names an ecosystem, a name or a version

#### Scenario: The fake renders cache entries

- **WHEN** the fake projection renders two cache entries
- **THEN** the layout holds those two build identities and no version

#### Scenario: The fake scripted to pack none

- **WHEN** a test scripts the fake projection to pack no cache entries
- **THEN** a pack with no version and a render of cache entries each answer absent, and a pack with a version answers as before

### Requirement: The fake projection answers the pins a test sets
The fake projection SHALL answer pinned, for a checkout, the ecosystem
names and versions a test set for that checkout, in the order set, and
absent naming the lockfile for a checkout given none; it SHALL render
artifacts as a layout holding their metadata, recording what it was
given, and write nothing (137, 85).

#### Scenario: Pins set by the test
- **WHEN** a test sets two pins for one checkout and asks the fake projection for that checkout's pins and another's
- **THEN** the first answer is the two pins in the order set, and the second is absent naming the lockfile

#### Scenario: The fake renders artifacts
- **WHEN** the fake projection renders two artifacts at a place
- **THEN** the layout holds those two artifacts' metadata and no other, and the test reads back the place and the artifacts it was given

### Requirement: The fake projection answers the requirements a test sets
The fake projection SHALL answer requires, for an artifact, the
requirements a test set for that artifact's digest, in the order set,
and absent naming the manifest of that artifact's name and version for
an artifact given none; it reads no bytes (140). requires SHALL be
recorded and can be faulted, as every operation of a fake is.

#### Scenario: Requirements set by the test
- **WHEN** a test sets two requirements for one artifact and asks the fake projection what that artifact and another require
- **THEN** the first answer is the two requirements in the order set, and the second is absent naming the other artifact's manifest

#### Scenario: requires faulted and recorded
- **WHEN** a test faults requires once and asks it
- **THEN** it answers the fault, and the fake's calls record requires with the artifact it was given

### Requirement: The fake projection orders versions by Semantic Versioning
The fake projection SHALL answer satisfying with the versions given
that satisfy the range by Semantic Versioning, newest first by its
precedence (23, 140). A range it cannot read SHALL be refused carrying
the range parser's own words (59), and a version it cannot read SHALL
satisfy no range. satisfying SHALL be recorded and can be faulted.

#### Scenario: Newest first, not by spelling
- **WHEN** the fake projection is asked which of `1.9.0`, `2.0.0`, `1.10.0`, `1.2.0` and `1.1.9` satisfy `^1.2`
- **THEN** it answers `1.10.0`, `1.9.0`, `1.2.0`, in that order

#### Scenario: A range it cannot read
- **WHEN** the fake projection is asked which versions satisfy a range that is not a Semantic Versioning range
- **THEN** the answer is refused carrying the parser's words, never absent and never an empty list

#### Scenario: satisfying faulted and recorded
- **WHEN** a test faults satisfying once and asks it
- **THEN** it answers the fault, and the fake's calls record satisfying with the range and versions it was given
