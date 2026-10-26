## ADDED Requirements

### Requirement: An artifact's store name is spelled once

The store name of an artifact's version SHALL be spelled
`artifact/<ecosystem>/<name>@<version>` by one function of the seams,
and read back into its ecosystem, name and version by its one inverse,
which every writer and reader of the name uses (138).

#### Scenario: A scoped name read back

- **WHEN** the store name of ecosystem `npm`, name `@acme/site` and version `1.3.0+b.1` is spelled and read back
- **THEN** it is `artifact/npm/@acme/site@1.3.0+b.1`, and reading it back answers that ecosystem, name and version

#### Scenario: A name outside the artifact space

- **WHEN** `cache/b7e1`, `artifact/npm` or `artifact/npm/site@` is read back as an artifact's store name
- **THEN** each answers none

## MODIFIED Requirements

### Requirement: An artifact says what its bytes are

Artifact metadata SHALL name the ecosystem whose form it takes, its
name in that ecosystem and its version (4). Index metadata SHALL name
one component and one version, and an index SHALL name every artifact
of that component at that version by digest (6, 110). Index metadata
SHALL also name the revision it was published from and the publishing
repository by its remote's owner and name, each where it has one, and
index metadata naming neither SHALL still be read (13, 138).

#### Scenario: Metadata with a part missing

- **WHEN** artifact metadata is made with an empty ecosystem, name or version
- **THEN** it is rejected, naming the part that is missing, and no metadata is made

#### Scenario: An index names its artifacts

- **WHEN** index metadata is made for a component, a version and a set of artifacts
- **THEN** it names each artifact's digest and metadata, and every artifact it names carries that version

#### Scenario: An index names where it came from

- **WHEN** index metadata naming a revision and the repository `acme/site` is written in its machine-readable form and read back
- **THEN** it is the same value, naming that revision and that owner and name

#### Scenario: An index written before it named where it came from

- **WHEN** index metadata in the machine-readable form that names no revision and no repository is read
- **THEN** it is read, naming neither, and written again it gives the same form

### Requirement: The store's contract

The store SHALL answer put-bytes with the digest and whether it held
the bytes already (66, 5); put-artifact with the artifact's digest
(67); attach with the attestation held beside its digest (68, 8);
claims-of with every attestation as attached, or absent (69); name
with the name moved, or refused naming where it is (70); resolve with
a digest, or absent (71); read with what the digest holds, or
absent (72); and the listing, given an ecosystem and a name, with every
version the store names that name at, or absent where it names none
(138).

#### Scenario: A name moved only from where it is expected

- **WHEN** name is asked to move a name from a digest it is not at
- **THEN** the answer is refused, naming the digest the name is at, and the name does not move

#### Scenario: A name made from nothing

- **WHEN** name is asked to move a name that names nothing, from nothing
- **THEN** the name is made

#### Scenario: An artifact read back

- **WHEN** read is asked for the digest put-artifact answered
- **THEN** it answers the artifact's metadata and the digests it names

#### Scenario: A name's versions listed

- **WHEN** the store names `artifact/eco/site@1.0.0`, `artifact/eco/site@1.1.0` and `artifact/eco/site-cli@1.0.0`, and the versions of `eco` `site` and of `eco` `web` are asked
- **THEN** the first answer is 1.0.0 and 1.1.0 and no other, and the second is absent, never an empty list

### Requirement: A projection's contract
A projection SHALL answer pack, given an output at a build identity and
the version where the bytes are an artifact's, with the ecosystem's own
bytes, their metadata only where a version was given, and whether the
same outputs and version, or the same build identity, pack to the same
bytes (84, 87, 100); render, given artifacts and their metadata or cache
entries by build identity, with a consumer layout and the versions or
build identities it holds, and given besides a range and the artifacts
of its name, holding the newest of them within the range by the
ecosystem's own rules or none (85, 137, 23); serve, given a layout, with
where its clients read it (86); and pinned, given a checkout, with the
ecosystem names and versions its lockfile pins as the lockfile spells
them, and, for a name its manifest declares and no lockfile pins, the
manifest's range as the manifest spells it, or absent where the checkout
holds no lockfile of that ecosystem (137).

#### Scenario: A projection that cannot promise the same bytes
- **WHEN** pack answers bytes it cannot promise to make again
- **THEN** its answer says so

#### Scenario: An output packed as an artifact
- **WHEN** pack is given an output, a build identity and a version
- **THEN** its answer carries metadata naming the ecosystem, the name and that version

#### Scenario: An output packed as a cache entry
- **WHEN** a projection that packs cache entries is given an output and a build identity and no version
- **THEN** its answer carries the bytes and no artifact metadata, so nothing about it makes an offer

#### Scenario: A layout of cache entries
- **WHEN** a projection that packs cache entries renders a layout from cache entries
- **THEN** the answer names the build identities the layout holds, and no version

#### Scenario: A projection that packs no cache entries
- **WHEN** a projection that packs no cache entries is asked to pack an output with no version, or to render cache entries
- **THEN** each answer is absent naming cache entries, never refused and never unreadable

#### Scenario: A checkout with no lockfile
- **WHEN** pinned is asked of a checkout holding no lockfile of the projection's ecosystem
- **THEN** the answer is absent naming the lockfile, never refused and never an empty list of pins

#### Scenario: A pin written and read back
- **WHEN** a pin, an ecosystem name and a version, and a range, an ecosystem name and a range's text, are each written in its machine-readable form and read back
- **THEN** each is the same as written

#### Scenario: A projection with no ranges asked to render one
- **WHEN** the file, image or moon cache projection is asked to render artifacts with a range
- **THEN** the answer is refused naming that its ecosystem declares no range, and nothing is written
