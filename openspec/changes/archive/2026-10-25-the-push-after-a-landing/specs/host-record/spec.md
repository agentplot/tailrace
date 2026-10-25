# Spec Delta

## ADDED Requirements

### Requirement: The record is rebuilt from the store and git

A rebuild SHALL write the host's record from what the store and git
hold: for each component the policy file declares at a landed revision,
each tag carrying its prefix that names a revision the branch landed,
and whose name the store resolves to that component's index at that
version, one entry, the same file a publish of it writes (61 to 63).

#### Scenario: S12, everything kept is deleted and rebuilt

- **WHEN** two components' versions are published, the host's record is deleted, and the record is rebuilt
- **THEN** every file of the record holds the same bytes it held before the deletion, and status answers the same as before

#### Scenario: A claimed version with no index

- **WHEN** a version's tag is claimed but the store names no index under it, and the record is rebuilt
- **THEN** the record holds no entry for that version, as a publish that stopped before its index keeps none

#### Scenario: A version another branch landed

- **WHEN** a tag carrying a component's prefix names a revision the branch asked for did not land, and the record is rebuilt for that branch
- **THEN** the record holds no entry for that tag

### Requirement: A rebuild writes only the record

A rebuild SHALL ask the landing first and refuse a revision its branch
has not landed before anything else is read; it SHALL ask the store
only to resolve names and read digests, sign nothing, and write nothing
but the record's entries (60, 63).

#### Scenario: Nothing written to the store

- **WHEN** the record is rebuilt
- **THEN** the store was asked only to resolve and read, and the signer was asked nothing

#### Scenario: A rebuild of a revision not landed

- **WHEN** a rebuild is asked for a revision its branch has not landed
- **THEN** it is refused naming the branch and the revision, and the store is asked nothing

### Requirement: A rebuild never rewrites what the record holds

A rebuild SHALL leave an entry the record already holds with the same
content untouched and answer it held, SHALL answer an entry held with
other content refused naming its file, leaving it as it is, and SHALL
delete nothing (63).

#### Scenario: A rebuild over a whole record

- **WHEN** the record is rebuilt while it holds every entry already
- **THEN** every entry is answered held, and no file of the record is written

#### Scenario: An entry that disagrees

- **WHEN** one entry of the record names another index than the store names for its tag, and the record is rebuilt
- **THEN** that entry is answered refused naming its file, the file is unchanged, and every other entry is rebuilt

### Requirement: A rebuild that cannot read is unreadable, and writes nothing

A rebuild SHALL answer unreadable, writing no entry, when the store or
the shared history cannot be read, or when a tag's name resolves to a
digest that holds no index of that component at that version, naming
what could not be read with the source's own words (58, 59, rule
core.19).

#### Scenario: The store unreachable

- **WHEN** the store's resolve is unreadable with the words `connection refused` and the record is rebuilt
- **THEN** the answer is unreadable carrying `connection refused` verbatim, and no file of the record is written

#### Scenario: A name at something else

- **WHEN** a version's tag name in the store resolves to a digest that holds bytes rather than that component's index
- **THEN** the answer is unreadable naming the tag and the digest, and no file of the record is written
