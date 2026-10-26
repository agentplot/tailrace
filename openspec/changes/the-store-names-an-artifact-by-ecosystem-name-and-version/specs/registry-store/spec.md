## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: One declared repository holds the store

The registry store SHALL hold every blob, manifest, attestation and
name of the store in the one repository the host names, addressed by
digest, on every registry, save each artifact's name, which is a tag of
its own repository under the named one holding what its index names,
and each cache entry, a tag of its build profile's cache repository
under the named one holding its bytes (C.1, ADR 0003). It SHALL never
create a repository of its own accord; a registry that creates
repositories on push may create the named one, an artifact name's and a
cache repository on the first put.

#### Scenario: Everything in one repository

- **WHEN** two artifacts of two ecosystem names, an index naming both, an attestation on each and a name that is neither an artifact's nor a cache entry's are put into a registry store
- **THEN** the registry's one named repository holds every one of them, and no other repository is written

#### Scenario: A repository that is not there

- **WHEN** the named repository does not exist on a registry that does not create repositories on push, and bytes are put
- **THEN** the put is refused carrying the registry's own words verbatim and naming the repository, and nothing is written elsewhere
