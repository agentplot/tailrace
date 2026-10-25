# registry-store Specification

## Purpose
The store seam bound to an OCI registry over the distribution protocol:
where a registry holds bytes, artifacts, indexes, attestations and
names, what each store operation sends and answers, and the registries
the profile is proven against (B.1, C.1).

## Requirements

### Requirement: One declared repository holds the store

The registry store SHALL hold every blob, manifest, attestation and
name of the store in the one repository the host names, addressed by
digest, on every registry (C.1, ADR 0003). It SHALL never create a
repository of its own accord; a registry that creates repositories on
push may create the named one on the first put.

#### Scenario: Everything in one repository

- **WHEN** two artifacts of two ecosystem names, an index naming both, an attestation on each and a name are put into a registry store
- **THEN** the registry's one named repository holds every one of them, and no other repository is written

#### Scenario: A repository that is not there

- **WHEN** the named repository does not exist on a registry that does not create repositories on push, and bytes are put
- **THEN** the put is refused carrying the registry's own words verbatim, and nothing is written elsewhere

### Requirement: Bytes are blobs, put once

put-bytes SHALL upload bytes as a blob under their own digest, after
asking the registry whether it holds that blob; when it does, nothing
SHALL be uploaded and the answer SHALL say held already (5, 66,
core.5). read SHALL answer a blob's bytes by digest, and bytes whose
digest is not the one asked for SHALL be refused naming both (2).

#### Scenario: The same bytes put twice

- **WHEN** the same bytes are put twice
- **THEN** both answers name the bytes' own digest, the second says held already, and the second put uploads nothing

#### Scenario: A registry that answers other bytes

- **WHEN** the registry answers a blob read with bytes whose digest is not the digest asked for
- **THEN** read is refused naming the digest asked for and the digest of the bytes answered

### Requirement: An artifact is a manifest in the layout's own bytes

put-artifact SHALL put an artifact as an OCI image manifest and an index
as an OCI image index, by digest alone, in exactly the bytes the layout
store writes for the same artifact, so one artifact has one digest in
every store (1, 67). A manifest the registry holds already SHALL be
answered held already and not put again (5).

#### Scenario: One artifact, one digest, in a layout and a registry

- **WHEN** the same artifact is put into a layout and into a registry
- **THEN** both answer the same digest, and the registry's manifest bytes are the layout's blob bytes

#### Scenario: An artifact and an index read back

- **WHEN** an artifact and an index naming it are put and each digest is read
- **THEN** read answers the artifact's ecosystem, name, version and layers in order, and the index's component, version and artifacts in order

#### Scenario: A manifest naming bytes not held

- **WHEN** an artifact naming a digest the repository does not hold is put
- **THEN** the put is absent naming that digest, and no manifest is put

### Requirement: An attestation is a referrer of its subject

attach SHALL push a referrer of tailrace's envelope type whose subject
is the digest and whose one layer is the envelope, with the statement's
predicate type annotated, and answer the envelope's own digest;
claims-of SHALL answer every envelope so attached, byte for byte (8,
68, 69, C.1, core.27). Both SHALL answer absent for a digest the
repository does not hold, and attach SHALL write nothing then.

#### Scenario: Two attestations read back

- **WHEN** two attestations are attached to one artifact
- **THEN** claims-of answers both envelopes byte for byte, and the registry lists two referrers of the artifact, each of the envelope type and annotated with its statement's predicate type

#### Scenario: The same attestation attached twice

- **WHEN** one attestation is attached twice to one digest
- **THEN** the second attach answers held already, and claims-of answers it once

#### Scenario: Another tool's referrer

- **WHEN** a digest has a referrer of another artifact type, such as a signature another tool pushed, beside tailrace's attestations
- **THEN** claims-of answers only tailrace's attestations, and the other referrer is left as it was

### Requirement: The referrers tag where the registry has no referrers API

Where a registry answers no referrers API, attach SHALL record the
referrer in the referrers tag the OCI distribution specification
defines for the subject, adding it once and keeping every entry already
there, and claims-of SHALL read that tag (C.1). A referrers tag that
lost the entry to another writer SHALL be reported, never retried.

#### Scenario: Attestations on a registry with no referrers API

- **WHEN** two attestations are attached to one artifact in the reference distribution
- **THEN** the subject's referrers tag lists both, and claims-of answers both envelopes byte for byte

#### Scenario: An entry another writer removed

- **WHEN** the referrers tag read back after an attach no longer lists the attestation just pushed
- **THEN** attach is answered unreadable naming the referrers tag and the attestation, and nothing is pushed again

### Requirement: A name is a tag moved from where it is expected and read back

name SHALL read the tag first and refuse a move from anywhere but the
expected digest, naming where it is; after the put it SHALL read the
tag back, again while it answers the former place up to the profile's
bound, and refuse naming a third digest as a lost race, or the former
place once the bound passes, never putting it again (70, 71, 109, C.1,
core.6). A name SHALL be escaped into a valid tag, one name to one tag.

#### Scenario: A name made, moved and resolved

- **WHEN** a name is made from nothing at one manifest and moved from there to another
- **THEN** resolve answers the second digest, and the repository holds one tag for the name

#### Scenario: A move from where the name is not

- **WHEN** a name at one digest is moved with an expected digest it is not at
- **THEN** the move is refused naming the digest the name is at, and the tag is not written

#### Scenario: A race lost after the put

- **WHEN** another writer moves the tag between the store's put and its read back
- **THEN** the move is refused naming the other writer's digest, and the store puts the tag no second time

#### Scenario: A read back at the former place within the bound

- **WHEN** the tag's read back after the put answers the digest it was moved from, as a registry not yet showing the put answers it, and a read again within the profile's bound answers the digest it was moved to
- **THEN** the move is made and is no lost race, and the store puts the tag no second time

#### Scenario: A name no tag can carry as written

- **WHEN** a name holding characters a tag cannot hold, such as `/` and `+`, is made and resolved, and so is a name differing from it only in those characters
- **THEN** each resolves to its own digest, and the repository holds a valid tag for each

### Requirement: A cache entry's bytes are named through a cache manifest

To name a digest held only as bytes, the registry store SHALL tag a
manifest of tailrace's cache-entry type whose one layer is those bytes,
and every reading of that tag, by resolve, by name's check and read
back, and in a refusal, SHALL answer the bytes' digest (52, 70, 71,
100, C.1, ADR 0014). The cache manifest is no artifact: read SHALL
never answer it as one, and claims-of SHALL never list it (53,
core.30).

#### Scenario: A cache entry named and resolved

- **WHEN** bytes are put and a name is made from nothing at their digest
- **THEN** resolve answers the bytes' digest, and the tag is at a manifest of the cache-entry type whose one layer is those bytes

#### Scenario: The same bytes named twice

- **WHEN** two names are made at the digest of the same bytes
- **THEN** both resolve to the bytes' digest, and the repository holds one cache manifest for them

#### Scenario: A cache entry's name moved

- **WHEN** a name at a cache entry's bytes is moved, with those bytes' digest as expected, to other bytes, and then moved with the first bytes' digest as expected
- **THEN** the first move succeeds and resolves to the other bytes' digest, and the second is refused naming the other bytes' digest

#### Scenario: The cache manifest is no artifact

- **WHEN** a name is made at bytes and the cache manifest's own digest is read, and claims-of is asked of the bytes' digest
- **THEN** read answers bytes and never an artifact, and claims-of lists no cache manifest

#### Scenario: Bytes not held

- **WHEN** a name is made at a digest the repository holds neither as a manifest nor as bytes
- **THEN** name is absent naming the digest, and no manifest or tag is put

### Requirement: Absent, refused and unreadable kept apart

A digest or name the repository does not hold SHALL be absent; a
registry that refuses SHALL be refused with its own words verbatim; a
registry that cannot be reached or answers what cannot be read SHALL be
unreadable with the client's or the registry's words (58, 59,
core.19).

#### Scenario: A registry that cannot be reached

- **WHEN** the registry's address answers no connection and bytes are read
- **THEN** the read is unreadable carrying the client's words, and never absent

#### Scenario: A tag the registry will not move

- **WHEN** the registry refuses a tag put, as an ECR repository with immutable tags does
- **THEN** name is refused carrying the registry's error body verbatim, and nothing is retried

### Requirement: The credential is the host's, reached at use

The registry store SHALL reach a registry's credential at the moment of
use through the host's Docker configuration and the credential helper
it names, and SHALL keep no credential, write none into any file, and
name none in the bindings file (64, core.21).

#### Scenario: A registry behind a credential helper

- **WHEN** the host's Docker configuration names a credential helper for the registry and an artifact is put
- **THEN** the store asks the helper for the credential when the registry asks for one, and no file tailrace writes holds it

#### Scenario: No credential for a registry that asks for one

- **WHEN** the registry answers that a credential is required and the host has none for it
- **THEN** the operation is refused carrying the registry's words, and names the registry

### Requirement: The store's conformance scenarios pass against real registries

The suite's store group SHALL pass against zot and against the
reference distribution in the tools layer, which the gate requires,
and against a real ECR repository in a check of its own outside the
gate, tagged `check-ecr` and run by hand with a live session as the
bolt's acceptance, since it needs a credential a person signs in for
(93, core.38). The ECR
check SHALL refuse to run, naming the setting it lacks, on a host that
names no ECR repository, rather than pass without running.

#### Scenario: zot

- **WHEN** the tools layer runs the store group against a zot started for the test
- **THEN** every scenario of the group passes

#### Scenario: The reference distribution

- **WHEN** the tools layer runs the store group against a reference distribution started for the test
- **THEN** every scenario of the group passes, attestations held through the referrers tag

#### Scenario: ECR, by hand, as the bolt's acceptance

- **WHEN** `moon run ':#check-ecr'` runs on a host whose `devenv.local.nix` names an ECR repository, with a live session for it
- **THEN** every scenario of the group passes against that repository, each run's names apart from every earlier run's

#### Scenario: No ECR repository named

- **WHEN** the ECR check starts on a host that names no ECR repository
- **THEN** it refuses to run naming the setting the host must give

### Requirement: What ECR answered is recorded

An ADR SHALL record what a push against a real ECR repository
answered on each open issue the profile rests on: tailrace's artifact
manifest and image index, a referrer with a subject, the referrers
list filtered by artifact type, a manifest put by digest alone, the
cache manifest, a media type with a parameter, and tag immutability
(ADR 0003, ADR 0014).

#### Scenario: The record read

- **WHEN** a reader opens the ADR
- **THEN** each open issue names what ECR answered, the date and the region, and what the profile does because of it
