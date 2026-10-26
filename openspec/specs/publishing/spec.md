# publishing Specification

## Purpose
How one landed revision becomes a version of each component it
publishes: the landing first, every output ensured and judged, each
artifact packed, put and attested, the index last, and an answer a
program reads.

## Requirements

### Requirement: A publish refuses a revision not landed, before anything else

A publish SHALL ask the landing seam whether the branch has landed the
revision before it reads the policy file, asks the build, packs, puts or
signs anything. A revision the branch has not landed SHALL be refused
naming the branch and the revision, and nothing SHALL be read or offered
(14, core.11). Where the landing answered not landed from the host's
copy as last fetched, the refusal SHALL also name when the copy was
fetched, carry the remote's words and say to ask again once the remote
answers (77). A revision the landing answered landed from such a copy
SHALL be published as one it answered landed from its remote.

#### Scenario: A revision the branch has not landed

- **WHEN** a revision the fake landing has not landed on the branch is published
- **THEN** the answer is refused naming the branch and the revision, and the fake build, projections, signer and store record no call

#### Scenario: The landing cannot be read

- **WHEN** the fake landing answers unreadable for the branch and revision
- **THEN** the publish answers that unreadable answer with the landing's words, and nothing is offered

#### Scenario: Not landed as last fetched

- **WHEN** the fake landing answers not landed from its copy as last fetched, with a time and the remote's words
- **THEN** the publish is refused naming the branch, the revision, that time and those words verbatim, saying to ask again, the command line exits as for any refusal, and the fake build, projections, signer and store record no call

#### Scenario: Landed as last fetched

- **WHEN** the fake landing answers landed from its copy as last fetched
- **THEN** the publish goes on as for a revision landed on the remote, and its answer equals that one's

### Requirement: A publish names its components

A publish SHALL publish the one component it is asked for, or, when it
names none, every component a commit since that component's last
release tag touched, and SHALL give no other component a version, an
index or a record (124, core.37). A component asked for that no commit
touched and that holds no version at the revision SHALL be answered
unchanged, naming its last tag, with nothing offered.

#### Scenario: Two components, one touched

- **WHEN** a repository declares two components, the commits since their last tags touch only the first, and a publish names no component
- **THEN** the answer holds the first component's version alone, and the store holds no index for the second

#### Scenario: A component nothing touched

- **WHEN** a publish names a component no commit since its last tag touched
- **THEN** the answer says the component is unchanged since its last tag, and nothing is ensured, packed or put

### Requirement: Each output names the projection that packs it

Each output a component declares SHALL name, in the policy file at the
landed revision, the ecosystem of the projection that packs it (C.10).
An output that names none SHALL refuse the component naming the output,
and an ecosystem no projection is bound for SHALL be answered absent
naming it, before the build is asked anything.

#### Scenario: An output with no projection

- **WHEN** a component declares an output the policy file names no projection for
- **THEN** the publish of that component is refused naming the output, and the fake build records no call

#### Scenario: A projection not bound

- **WHEN** an output names an ecosystem the binding holds no projection for
- **THEN** the publish of that component is absent naming the ecosystem, and the fake build records no call

### Requirement: Every output is ensured in one asking and its outcome read

A publish SHALL ask the build seam to ensure every output the component
declares in one asking, then read each output's build identity, outcome
and files, before anything is packed. An output the build could not
make SHALL refuse the version whole, naming each such output (74, 75,
16).

#### Scenario: One asking for every output

- **WHEN** a component declaring two outputs is published
- **THEN** the fake build records one ensure naming both outputs, then a report of each, before any pack

#### Scenario: An output the build could not make

- **WHEN** the fake build cannot make one of a component's two outputs
- **THEN** the publish is refused naming that output, and nothing is packed or put

### Requirement: An output is a pass only by the build and the landing together

An output's outcome SHALL be a pass only when the build reports a pass
and, where the landing carries verdicts for the revision, a verdict with
a pass names the output's build identity (111). A version any of whose
outputs is not a pass SHALL be refused whole, naming the output, with
nothing offered (16).

#### Scenario: The build reports a failure

- **WHEN** the fake build reports one output's outcome as a failure
- **THEN** the publish is refused naming that output and its outcome, and the store records no put

#### Scenario: Only a failing verdict names the build

- **WHEN** the landing carries verdicts and the only verdict naming an output's build identity failed
- **THEN** the publish is refused naming that output, and nothing is offered

#### Scenario: A landing with no verdicts

- **WHEN** the landing carries no verdicts for the revision and the build reports every output a pass
- **THEN** the publish offers the version, and each provenance claim carries no verdict (S13)

### Requirement: An output no verdict names is refused naming both

Where the landing carries verdicts for the revision, an output whose
build identity no verdict names SHALL be refused, naming the output and
every build identity the verdicts carry, and nothing of the version
SHALL be offered (15, I3).

#### Scenario: A build the landing never checked

- **WHEN** the fake build reports an output from a build identity that none of the landing's verdicts names
- **THEN** the publish is refused naming the output, its build identity and the identities the verdicts carry, and the store records no put

### Requirement: The name is claimed after the outputs are judged and before anything is packed

A publish SHALL claim the version's name only after every output is
ensured and judged, and before any output is packed, so a version
refused for its outputs claims no name, and no build is handed the
version (22, core.12, core.13).

#### Scenario: A refused version claims nothing

- **WHEN** a publish is refused because an output is not a pass
- **THEN** no tag of the version's name exists in the local or the shared history

#### Scenario: The claim before the first pack

- **WHEN** a version is published
- **THEN** the tag of its name exists before the first pack is asked, and no ensure or report the fake build records names a version

### Requirement: Each artifact is packed, put and checked by its digest

Each output SHALL be packed with the version by its projection, its
bytes put in the store, and its metadata put as an artifact naming
those bytes (84, 66, 67). A store that answers a digest for the bytes
other than the bytes' own SHALL be refused naming both, and the publish
SHALL stop before the index (2, core.5).

#### Scenario: Two artifacts put

- **WHEN** a component with two outputs is published
- **THEN** the store holds each output's packed bytes under their own digest and an artifact for each naming its ecosystem, name and the version

#### Scenario: A store that names bytes wrongly

- **WHEN** the store answers put-bytes with a digest that is not the digest of the bytes put
- **THEN** the publish is refused naming both digests, and no index is put

### Requirement: Each artifact carries one provenance claim in the host's identity

Each artifact SHALL be attached one provenance claim, signed in the
identity of the party running the publish, naming the artifact's
digest, the revision, the output, the output's build identity and the
build's outcome (11, 13, 79, core.8, core.9). Where no identity is at
hand, the publish SHALL stop before the index and answer absent naming
the signer (17).

#### Scenario: A provenance claim attached

- **WHEN** a version is published
- **THEN** claims-of each artifact's digest answers one provenance claim signed by the host's signer, naming the revision and the output's build identity

#### Scenario: No identity at hand

- **WHEN** the fake signer has no identity at hand
- **THEN** the publish answers absent naming the signer, and no index is put and no version's name moved

### Requirement: The landing's verdicts are carried as made

A provenance claim SHALL carry every verdict of the landing that names
the artifact's build identity, each envelope byte for byte as the
landing recorded it with every signature and key id it holds, and say
where it was found: the landing, the revision it was asked for, and
where the landing says it found the verdict. tailrace SHALL never
re-sign another party's verdict as its own (12, core.8, C.5).

#### Scenario: A verdict carried byte for byte

- **WHEN** a version is published from a revision the landing recorded two verdicts for, both naming the build identity of one output
- **THEN** that output's provenance claim carries both verdicts' envelopes byte for byte with the landing named as where they were found, and no envelope the store holds is signed by the host over a verdict's payload

#### Scenario: Where the landing found it

- **WHEN** the landing answers a verdict found in an earlier run's record
- **THEN** the provenance claim that carries it names that record as the landing named it

### Requirement: Attestations are held beside their digest

Every claim a publish makes SHALL be attached to the digest it is about
in the store the artifact is offered to, and kept nowhere apart from
that digest (8, core.27).

#### Scenario: Nothing attested elsewhere

- **WHEN** a version is published
- **THEN** every envelope the publish signed is answered by claims-of the digest it names as its subject, and the host's record holds no envelope

### Requirement: The index is offered last, then the version is named

The index SHALL be put only after every artifact it names is put and its
claims attached, and the version's name SHALL then be moved in the store
from nothing to the index's digest. A version exists only once its index
is held, and a publish that stops before its index has published nothing
(6, 17, core.12, core.6).

#### Scenario: The order of puts

- **WHEN** a component with two outputs is published
- **THEN** the fake store records both artifacts and their attachments before the index, and the name after the index

#### Scenario: A stopped publish

- **WHEN** a publish stops after one artifact is put
- **THEN** no index is held, the version's name resolves to nothing, and the host's record holds no entry for the version

### Requirement: A second publish of a revision ends at the same index

A publish of a revision whose version's name already names an index
SHALL compare each packed artifact's digest with the digest the index
names for that ecosystem name. All the same, nothing SHALL be written
and the answer SHALL name the same index (5, 18, S2). One different, the
publish SHALL be refused naming the artifact and both digests, nothing
SHALL be offered, and the name SHALL stay with the first index (18, 87).

#### Scenario: The same revision again

- **WHEN** a revision is published twice with projections that pack the same
- **THEN** the second answer names the same index and digests, and the store, the history and the host's record hold after it exactly what they held before it

#### Scenario: A projection that packs differently

- **WHEN** a revision is published twice and the fake projection packs different bytes the second time
- **THEN** the second publish is refused naming the artifact, the digest the index names and the digest packed now, the store records no put, and the version's name still names the first index

### Requirement: A publish answers what a program reads

A publish SHALL end with an answer readable by a program that names, for
each component, the version, the tag, the revision, the index's digest,
each artifact's digest, ecosystem and name, whether its projection
promises the same bytes again, and each claim with its kind, its signer,
its envelope's digest and the digest it is attached to (19, 57).

#### Scenario: The answer names every digest

- **WHEN** a component with two outputs is published and its answer is read as JSON
- **THEN** it names the index's digest, the version, both artifacts' digests and ecosystem names, and for each artifact the provenance claim's envelope digest and the artifact digest it is attached to, each equal to what the fake store holds

### Requirement: A publish offers and does nothing more

A publish SHALL never accept, promote, move a channel or reach a target,
and SHALL move no name in the store but its version's (20, core.10,
I2).

#### Scenario: No channel moved

- **WHEN** a version is published with a target and a keeper bound
- **THEN** the fake keeper and the fake target record no call, and the fake store records one name moved, the version's
