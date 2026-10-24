# layout-store Specification

## Purpose
The store seam bound to an OCI image layout on disk, read and written
with no server: where a layout holds bytes, artifacts, indexes,
attestations and names, what each store operation writes and answers,
and how a layout that cannot be read is told apart from a digest it
does not hold (B.1, C.2).

## Requirements

### Requirement: A layout is a directory in the OCI image layout

The layout store SHALL hold everything in one directory laid out as the
OCI image layout specification says: an `oci-layout` file naming its
layout version, an `index.json` that is an OCI image index, and every
blob under `blobs/sha256/` at the hex of its own digest (C.2, ADR 0003).
A layout SHALL be made only when asked to make one, never by an
operation that finds it missing.

#### Scenario: A layout made

- **WHEN** a layout is made in an empty directory
- **THEN** the directory holds an `oci-layout` file whose `imageLayoutVersion` is `1.0.0`, an `index.json` that is an image index naming no manifest, and an empty `blobs/sha256/` directory

#### Scenario: A layout is never made by an operation

- **WHEN** a layout's directory has been removed and bytes are put
- **THEN** the put is answered unreadable, and no directory, `oci-layout` file or blob is written

#### Scenario: A layout another tool wrote

- **WHEN** a layout is opened whose `index.json` holds entries of media types the store does not write, and blobs no entry names
- **THEN** every operation answers as it would without them, and none of them is changed or removed

### Requirement: Bytes are written under their own digest, once

put-bytes SHALL write bytes only to the path their own digest names
and answer that digest (66, core.5). Bytes the layout holds already
SHALL be answered held already and SHALL NOT be written again (5). A
blob SHALL appear at its path whole or not at all, so no reader finds
part of one.

#### Scenario: The same bytes put twice

- **WHEN** the same bytes are put twice
- **THEN** both puts answer the bytes' digest, the second answers held already, and the blob's file is unchanged by the second put

#### Scenario: Where bytes are held

- **WHEN** bytes are put
- **THEN** a file at `blobs/sha256/<hex>` holds exactly those bytes, where `<hex>` is the hex of their digest, and no other file under `blobs/` holds them

### Requirement: Bytes on disk are never rewritten

The layout store SHALL never rewrite or remove a blob it holds (73,
core.5). Where a blob's bytes on disk are not those its digest names,
a put or a read of that digest SHALL be refused naming the digest
expected and the digest of the bytes found, and the blob SHALL be left
as found (2).

#### Scenario: A blob altered on disk is read

- **WHEN** a blob's file is changed on disk after it was put, and its digest is read
- **THEN** the read is refused naming the digest asked for and the digest of the bytes found, and the file is left as it is

#### Scenario: A blob altered on disk is put again

- **WHEN** a blob's file is changed on disk after it was put, and the same bytes are put again
- **THEN** the put is refused naming both digests, and the file is not overwritten

### Requirement: An artifact is an OCI manifest held under its digest

put-artifact SHALL write an artifact as an OCI image manifest whose
artifact type names the ecosystem, whose config is the artifact's
metadata and whose layers are its bytes, and an index as an OCI image
index naming each artifact with its metadata, each under its own
digest, and answer that digest (4, 6, 67). The same manifest SHALL
always write the same bytes, so a second put answers the same digest,
held already (5). read SHALL answer what was put (72).

#### Scenario: An artifact read back

- **WHEN** an artifact is put and its digest is read
- **THEN** the read answers the artifact's ecosystem, name and version, and the digests of its bytes in the order they were given

#### Scenario: An index read back

- **WHEN** an index naming two artifacts is put and its digest is read
- **THEN** the read answers the index's component, its version, and each artifact by digest and metadata in the order they were given

#### Scenario: The same artifact on two hosts

- **WHEN** the same artifact is put into two layouts in two directories
- **THEN** both answer the same digest, and the manifest's file holds the same bytes in each

#### Scenario: An artifact a registry can read

- **WHEN** an artifact is put
- **THEN** its manifest's file is an OCI image manifest whose media type is the image manifest's, whose `artifactType` names the artifact's ecosystem, whose config and layers each name a blob the layout holds by digest and size, and `index.json` holds one entry naming the manifest by digest, media type and size

#### Scenario: An artifact naming bytes not held

- **WHEN** an artifact or an index is put naming a digest the layout does not hold
- **THEN** the put is answered absent naming that digest, and nothing is written

### Requirement: An attestation is an index entry with a subject annotation

attach SHALL write the attestation under its own digest and add one
entry naming it to `index.json`, annotated with the digest it is about,
and answer the attestation's digest (68, C.2). claims-of SHALL answer
every attestation whose entry names that digest, as attached, in the
order attached (69). An attestation SHALL NOT be held for a digest the
layout does not hold (8, core.27).

#### Scenario: Two attestations on one digest

- **WHEN** two attestations are attached to one artifact
- **THEN** claims-of answers both, byte for byte as attached, in the order attached, and `index.json` holds one entry for each, annotated with the artifact's digest

#### Scenario: The same attestation attached twice

- **WHEN** one attestation is attached to one digest twice
- **THEN** the second attach answers the same digest, held already, and claims-of answers it once

#### Scenario: An attestation for a digest not held

- **WHEN** an attestation is attached to a digest the layout does not hold
- **THEN** the attach is answered absent naming that digest, and no blob or entry is written

#### Scenario: Claims of a digest not held

- **WHEN** claims-of is asked for a digest the layout does not hold
- **THEN** the answer is absent naming the digest, never an empty list

#### Scenario: Attestations travel with a copy of the layout

- **WHEN** a layout's directory is copied to another path and opened there
- **THEN** claims-of answers in the copy every attestation it answers in the original

### Requirement: A name is moved only from the digest it is expected at

name SHALL move a name, held as the ref name of an entry in
`index.json`, to a digest only when the name is at the expected digest,
or names nothing when nothing is expected; otherwise it SHALL refuse
naming the digest the name is at, and leave the name where it is (70,
core.6). resolve SHALL answer the digest a name names, or absent (71).

#### Scenario: A name made and moved

- **WHEN** a name is made from nothing and then moved from the digest it is at
- **THEN** resolve answers the digest it was moved to, and `index.json` holds exactly one entry with that ref name

#### Scenario: A name moved from where it is not

- **WHEN** a name is moved from a digest it is not at, or from nothing when it names a digest
- **THEN** the move is refused naming the digest the name is at, and resolve still answers that digest

#### Scenario: Two writers move one name

- **WHEN** two handles on one layout, in two threads at once, each move one name to a different digest, both expecting the digest it is at
- **THEN** exactly one move succeeds, the other is refused naming the digest the first moved it to, and the name is where the first moved it

#### Scenario: A name moved to a digest not held

- **WHEN** a name is moved to a digest the layout does not hold
- **THEN** the move is answered absent naming that digest, and the name does not move

#### Scenario: A name naming nothing

- **WHEN** resolve is asked for a name never made
- **THEN** the answer is absent naming the name

### Requirement: Absent and unreadable are kept apart

A digest or a name the layout does not hold SHALL be answered absent.
A layout whose directory is gone, whose files cannot be read, or whose
`oci-layout` or `index.json` cannot be parsed SHALL be answered
unreadable, naming the file and carrying the operating system's or the
parser's words verbatim (58, 59, core.19). Every answer SHALL name the
store seam and the layout's profile with its path.

#### Scenario: A digest not held

- **WHEN** read is asked for a digest never put
- **THEN** the answer is absent naming the digest, from the store, naming the layout

#### Scenario: A layout that cannot be reached

- **WHEN** a layout's directory is removed and each operation is asked
- **THEN** each answers unreadable from the store, carrying the operating system's words for the missing `oci-layout` file unchanged

#### Scenario: A malformed index

- **WHEN** a layout's `index.json` holds text that is not an image index, and a name is resolved
- **THEN** the answer is unreadable naming `index.json` and carrying the parser's words, and the file is left as it is

### Requirement: The layout keeps the store's contract in the conformance suite

The layout store SHALL pass every scenario of the conformance suite's
store group, each on a fresh layout in a temporary directory, in the
profile's own tests that the gate runs (93, core.33).

#### Scenario: The store group over a layout

- **WHEN** the suite's store group runs over the layout store, a new layout in a temporary directory for each scenario
- **THEN** the report names the layout's profile and every scenario passes
