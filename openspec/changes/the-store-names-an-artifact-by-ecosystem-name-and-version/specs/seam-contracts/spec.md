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
version the store names that name at, an empty list where it names
none, or unreadable where the listing cannot be read (138, core.19).

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
- **THEN** the first answer is 1.0.0 and 1.1.0 and no other, and the second is the empty list, neither absent nor refused
