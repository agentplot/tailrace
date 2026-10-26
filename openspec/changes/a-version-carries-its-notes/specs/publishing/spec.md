## MODIFIED Requirements

### Requirement: Each artifact carries one provenance claim in the host's identity

Each output's artifact SHALL be attached one provenance claim, signed in
the identity of the party running the publish, naming the artifact's
digest, the revision, the output, the output's build identity and the
build's outcome (11, 13, 79, core.8, core.9). The version's notes carry
a provenance claim of their own, naming no build (134). Where no
identity is at hand, the publish SHALL stop before the index and answer
absent naming the signer (17).

#### Scenario: A provenance claim attached

- **WHEN** a version is published
- **THEN** claims-of each output's artifact digest answers one provenance claim signed by the host's signer, naming the revision and the output's build identity

#### Scenario: No identity at hand

- **WHEN** the fake signer has no identity at hand
- **THEN** the publish answers absent naming the signer, and no index is put and no version's name moved

### Requirement: A second publish of a revision ends at the same index

A publish of a revision whose version's name already names an index
SHALL compare each packed artifact's digest with the digest the index
names for that ecosystem name, and the notes rendered again with the
notes the index names, where it names any. All the same, nothing SHALL
be written and the answer SHALL name the same index (5, 18, S2). One
different, the publish SHALL be refused naming the artifact and both
digests, nothing SHALL be offered, and the name SHALL stay with the
first index (18, 87, 134).

#### Scenario: The same revision again

- **WHEN** a revision is published twice with projections that pack the same
- **THEN** the second answer names the same index, digests and notes, and the store, the history and the host's record hold after it exactly what they held before it

#### Scenario: A projection that packs differently

- **WHEN** a revision is published twice and the fake projection packs different bytes the second time
- **THEN** the second publish is refused naming the artifact, the digest the index names and the digest packed now, the store records no put, and the version's name still names the first index

### Requirement: A publish answers what a program reads

A publish SHALL end with an answer readable by a program that names, for
each component, the version, the tag, the revision, the index's digest,
each artifact's digest, ecosystem and name, whether its projection
promises the same bytes again, and each claim with its kind, its signer,
its envelope's digest and the digest it is attached to; and, where the
index names notes, the notes' digest, their bytes' digest, their claim
and their text (19, 57, 134).

#### Scenario: The answer names every digest

- **WHEN** a component with two outputs is published and its answer is read as JSON
- **THEN** it names the index's digest, the version, both artifacts' digests and ecosystem names, and for each artifact the provenance claim's envelope digest and the artifact digest it is attached to, each equal to what the fake store holds

#### Scenario: The answer shows the notes

- **WHEN** a version is published and its answer is read as JSON and as text
- **THEN** the JSON names the notes' digest, their claim's envelope digest and their text equal to the bytes the fake store holds, and the text answer prints the notes beneath the version's line
