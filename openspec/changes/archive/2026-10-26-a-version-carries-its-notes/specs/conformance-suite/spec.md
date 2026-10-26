## MODIFIED Requirements

### Requirement: S1 over the fakes

The publish group SHALL play S1 over the fakes, an in-memory store and
a git history: a landed revision with two outputs and a landing
verdict naming their build is published.

#### Scenario: S1, a landed revision published

- **WHEN** a revision landed with a verdict naming its build is published, its two outputs read from the build and packed
- **THEN** both artifacts carry the version history derives, the index is put after both artifacts, the notes and their claims, each output's artifact has one provenance in the host's identity naming the revision and build identity with the landing's verdict carried unchanged, the index names the notes as an artifact of ecosystem `notes` at the version with one provenance in the host's identity naming the revision and no build, the version's name is made at the index, the answer names every digest and where each claim is attached, and nothing else moved: no other name, no keeper append and no target call

### Requirement: S2 over the fakes

The publish group SHALL play S2: the same publisher publishes the same
revision again.

#### Scenario: S2, the same revision again

- **WHEN** the revision published in S1 is published again
- **THEN** the store records no put, attach or name move, the history and the host's record are unchanged, and the answer equals the first, its notes among it

### Requirement: Status over the fakes

The status group SHALL hold status to 58, 60, 61 and 62: answered from
what the host keeps, with no seam asked and nothing written.

#### Scenario: Status after a publish

- **WHEN** status is asked after a publish
- **THEN** it answers the component, version, revision and index published with the notes the index names, byte for byte, no fake records a call after the publish's last, and no file of the record changes

#### Scenario: The record read with a file reader

- **WHEN** an entry of the host's record is read as plain JSON from its file
- **THEN** it names the component, version, revision, branch, index, store and the revisions the notes were rendered from, and nothing the store or git holds besides

#### Scenario: Nothing kept, and an entry that cannot be read

- **WHEN** status is asked with no record, and then with an entry that does not parse
- **THEN** the first is absent naming the directory, and the second unreadable naming the file
