## ADDED Requirements

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

## MODIFIED Requirements

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
