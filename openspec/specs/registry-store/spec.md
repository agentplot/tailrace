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
digest, on every registry, save three spaces under the named one: a
declared component's versions and channels, tags of its own repository
holding its indexes and what they name; each artifact's name, a tag of its own
repository holding what its index names; and each cache entry, a tag of
its build profile's cache repository holding its bytes (C.1, ADR 0003).
It SHALL never create a repository of its own accord; a registry that
creates repositories on push may create the named one, a component's,
an artifact name's and a cache repository on the first put.

#### Scenario: Everything in one repository

- **WHEN** two artifacts of two ecosystem names, an index of the undeclared component naming both, an attestation on each and a name that is neither an artifact's, a cache entry's nor a declared component's version or channel are put into a registry store
- **THEN** the registry's one named repository holds every one of them, and no other repository is written

#### Scenario: A repository that is not there

- **WHEN** the named repository does not exist on a registry that does not create repositories on push, and bytes are put
- **THEN** the put is refused carrying the registry's own words verbatim and naming the repository, and nothing is written elsewhere

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

attach SHALL push a referrer of timber's envelope type whose subject
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

- **WHEN** a digest has a referrer of another artifact type, such as a signature another tool pushed, beside timber's attestations
- **THEN** claims-of answers only timber's attestations, and the other referrer is left as it was

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
manifest of timber's cache-entry type whose one layer is those bytes,
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
- **THEN** the store asks the helper for the credential when the registry asks for one, and no file timber writes holds it

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
answered on each open issue the profile rests on: timber's artifact
manifest and image index, a referrer with a subject, the referrers
list filtered by artifact type, a manifest put by digest alone, the
cache manifest, a media type with a parameter, and tag immutability
(ADR 0003, ADR 0014).

#### Scenario: The record read

- **WHEN** a reader opens the ADR
- **THEN** each open issue names what ECR answered, the date and the region, and what the profile does because of it

### Requirement: A declared component's versions and channels are tags of its own repository

The registry SHALL hold a declared component's version's name
`<component>:<tag>` as the tag `<tag>`, and its channel's name
`channel/<component>/<name>` as the tag `<name>`, of the component's
repository under the one the binding declares, the component escaped
into one path component, each moved with the digest expected and read
back there. A version's bare tag and `channel/<name>`, of a repository
that declares no component, SHALL stay tags of the declared repository
(C.1, 109, 125, core.6).

#### Scenario: A declared component's version

- **WHEN** a publish of the component `lib` makes `lib:lib-v1.2.0` at its index
- **THEN** the repository `lib` under the declared one has the tag `lib-v1.2.0` at the index, resolve of `lib:lib-v1.2.0` answers it, and the declared repository holds no tag `lib-v1.2.0`

#### Scenario: Two components' channels of one name

- **WHEN** a promotion moves `channel/lib/beta` to one index and another moves `channel/app/beta` to another
- **THEN** the repository `lib` under the declared one has the tag `beta` at the first index, the repository `app` has the tag `beta` at the second, each resolves to its own, and the declared repository holds neither tag

#### Scenario: A repository that declares no component

- **WHEN** a publish makes the version `v1.2.0` and a promotion moves `channel/stable` to its index
- **THEN** the declared repository holds the version's and the channel's tags as it did before this change, and no other repository is written for them

#### Scenario: A channel moved from where it is not

- **WHEN** `channel/lib/beta` is at one index and a move to another names a third as the digest expected
- **THEN** the move is refused naming the digest the tag is at in the repository `lib`, and the tag does not move

### Requirement: A declared component's repository holds its indexes

When the registry puts an index whose component is a declared one, it
SHALL hold the index, and every manifest and blob the index names, in
that component's repository as well as the declared one, mounted or,
where the registry declines the mount, uploaded under their own
digests; and before a version's or a channel's tag is put there it
SHALL hold the index the tag names, writing only what the repository
lacks.
Attestations SHALL stay in the declared repository (8, C.1, core.5).

#### Scenario: An index published

- **WHEN** a publish of the component `lib` puts its index into a registry store
- **THEN** the repository `lib` under the declared one holds the index and every artifact manifest and blob it names, untagged, and the declared repository holds them and every attestation

#### Scenario: An index put before this change

- **WHEN** the repository `lib` lacks an index the declared repository holds, and a promotion moves `channel/lib/beta` to it
- **THEN** the index and what it names are held in `lib` before the tag `beta` is put there

### Requirement: An artifact's name is a tag of a repository of its own

The registry SHALL hold the name `artifact/<ecosystem>/<name>@<version>`
as the tag `<version>` of the repository `artifact/<ecosystem>/<name>`
under the one the binding declares, the ecosystem and the name each
lowercased and escaped into one path component the distribution
protocol allows, and the version escaped into a tag one way and read
back. A repository path past 255 characters SHALL be refused naming the
name, and nothing written (70, 71, 138, C.1).

#### Scenario: A scoped package's name

- **WHEN** the name `artifact/npm/@acme/site@1.3.0` is made at an index and resolved, and so is `artifact/npm/acme-site@1.3.0` at another
- **THEN** each resolves to its own index, the first in the repository `artifact/npm/0_40acme_2fsite` and the second in `artifact/npm/acme-site` under the declared one, each tagged `1.3.0`, and the declared repository holds every attestation

#### Scenario: A version a tag cannot hold as written

- **WHEN** the name `artifact/cargo/site@1.0.0+build.1` is made at an index, and its name's versions are listed
- **THEN** the tag escapes the `+`, and the listing answers `1.0.0+build.1` as written

#### Scenario: A name moved from where it is not, in its own repository

- **WHEN** an artifact's name is made at one index, then made again from nothing at another
- **THEN** the second is refused naming the first index, and the name still resolves to it

### Requirement: An artifact name's repository holds what its index names

Before an artifact name's tag is put, the index it names and every
manifest and blob the index names SHALL be held in that name's
repository, mounted from the declared repository or, where the registry
declines the mount, uploaded under their own digests; attestations SHALL
stay in the declared repository, where the seam reads them (8, 138,
C.1, core.5).

#### Scenario: A mount declined
- **WHEN** the registry declines the mount of a blob the index names, and the name is made
- **THEN** the blob is uploaded under its own digest into the name's repository, the tag is put, and the name resolves to the index

### Requirement: A name's versions are its repository's tags

The registry SHALL answer the listing of an ecosystem and a name from
the tag list of that name's repository, every page of it, each tag read
back as a version; a repository the registry does not hold, or one with
no tags, SHALL be the empty list, and a tag that does not read back
SHALL be unreadable naming the repository and the tag (138, C.1,
core.19).

#### Scenario: Versions listed by tags

- **WHEN** two versions of `artifact/npm/@acme/site` are named and its versions are asked
- **THEN** the answer is the two versions and no other

#### Scenario: A name the registry never held

- **WHEN** the versions of a name whose repository the registry does not hold are asked
- **THEN** the answer is the empty list, neither absent nor refused

### Requirement: A cache entry is a tag of its build profile's cache repository

The registry SHALL hold the name `cache/<build identity>` as the tag of
the build identity in the repository `cache/<build>` under the one the
binding declares, `<build>` the bound build profile's name, with its
cache manifest and the bytes it names held there, mounted from the
declared repository or uploaded under their own digest where they are
not (52, 135, C.1, core.5).

#### Scenario: An entry named from bytes the declared repository holds
- **WHEN** a store bound for a publish puts bytes and makes `cache/b7e1` at their digest
- **THEN** the bytes are mounted into `cache/moon`, or uploaded there under their digest where the mount is declined, before the cache manifest and the tag are put

### Requirement: A push writes into the cache repository alone

A store bound for a push SHALL put bytes into the cache repository, and
every other binding into the declared one. A read by digest SHALL ask
the declared repository and, where it does not hold the digest, the
cache repository (54, 135, core.14).

#### Scenario: A push's cache entry
- **WHEN** a store bound for a push puts a cache entry's bytes and makes `cache/b7e1` at their digest, with the moon build profile bound
- **THEN** the repository `cache/moon` under the declared one holds the bytes, the cache manifest and the tag `b7e1`, the declared repository holds none of them, and resolve of `cache/b7e1` answers the bytes' digest

#### Scenario: An entry's bytes read back
- **WHEN** a store bound for a sync resolves `cache/b7e1` and reads the digest it answers
- **THEN** the read answers the entry's bytes from the cache repository

### Requirement: A repository nobody made is refused naming it

The registry store SHALL never make a repository by an API of its own.
Where a registry refuses a put because the repository it is for does not
exist, as one that makes repositories only by a creation template does
where none applies, the put SHALL be refused carrying the registry's
words verbatim and naming the repository, and nothing SHALL be retried
(C.1, 58, 59).

#### Scenario: An artifact name's repository missing
- **WHEN** a registry that makes no repository on push holds the declared repository and not `artifact/eco/site`, and the name `artifact/eco/site@1.3.0` is made
- **THEN** the make is refused carrying the registry's words and naming `artifact/eco/site` under the declared repository, and no tag is put

#### Scenario: A component's repository missing
- **WHEN** a registry that makes no repository on push holds the declared repository and not `lib`, and a publish of the component `lib` makes `lib:lib-v1.2.0`
- **THEN** the make is refused carrying the registry's words and naming `lib` under the declared repository, nothing is asked again, and the version's tag is not made

#### Scenario: A cache repository missing
- **WHEN** a registry that makes no repository on push holds the declared repository and not `cache/moon`, and a store bound for a push puts a cache entry's bytes, with the moon build profile bound
- **THEN** the put is refused carrying the registry's words and naming `cache/moon` under the declared repository, nothing is asked again, and the declared repository is not written
