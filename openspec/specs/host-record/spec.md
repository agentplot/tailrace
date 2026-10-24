# host-record Specification

## Purpose
What a host keeps of the versions it published, readable with a file
reader and no tailrace process, and `tailrace status` answered from it
alone, reaching no store, signer or target and writing nothing.

## Requirements

### Requirement: The host keeps which versions it published from which revisions

After a publish moves a version's name to its index, the host SHALL
keep one entry for that version naming its component, version, tag,
revision, branch, the index's digest and the store it was offered to,
the store as the host's bindings name it, and nothing the store or git
holds besides: no artifact, no claim and no credential (61, 64, 118,
core.20, core.21).

#### Scenario: An entry after a publish

- **WHEN** a version is published
- **THEN** the host's record holds one entry for that component and version naming the revision, the branch, the index's digest and the store, and no envelope, artifact digest or credential

#### Scenario: The store as the bindings name it

- **WHEN** the built binary publishes a version on a host whose bindings file names `[store] layout` with a path beginning `~/`, and status is asked with `--json`
- **THEN** the version's entry names the store as `layout` and that path with `~/` expanded

#### Scenario: Nothing kept of a refused publish

- **WHEN** a publish is refused or stops before its index
- **THEN** the host's record holds no entry for that version

### Requirement: The record is a file a reader opens

Each entry SHALL be a file of its own on the host, in a form a program
reads, shared by every checkout of the repository on the host and
readable with no tailrace process running (62, core.20). A publish
whose entry already holds the same content SHALL write nothing.

#### Scenario: Read with a file reader

- **WHEN** a version is published and its entry is opened as a file and parsed as JSON
- **THEN** it names the component, version, revision and index's digest the publish answered

#### Scenario: Two checkouts, one record

- **WHEN** a version is published from one worktree of a repository and status is asked in another worktree of it on the same host
- **THEN** status answers that version

### Requirement: Status answers from the record alone

`tailrace status` SHALL answer, for each component, the versions this
host published, each with its revision, branch, index's digest and
store, from the host's record alone, ordered by the version rules (23),
and SHALL reach no store, signer or target and write nothing (60,
core.32).

#### Scenario: Status after a publish

- **WHEN** a version is published and status is asked
- **THEN** status answers that component's version, revision and index's digest, and the fake store, signer and targets record no call made by status

#### Scenario: Status writes nothing

- **WHEN** status is asked
- **THEN** every file of the host's record holds after it what it held before it, and no file is added

#### Scenario: Which version a revision became

- **WHEN** status is asked for one revision this host published
- **THEN** it answers the version each component took at that revision, and no other

### Requirement: Status keeps absent and unreadable apart

Status SHALL answer absent when the host keeps nothing for the
repository, and unreadable naming the file, with the parser's words,
when an entry cannot be read; it SHALL never answer an unreadable entry
as absent or as an empty list (58, 59, core.19).

#### Scenario: Nothing kept

- **WHEN** status is asked in a repository this host never published from
- **THEN** the answer is absent, naming the record it looked in

#### Scenario: An entry that cannot be read

- **WHEN** one entry of the record is not well formed
- **THEN** the answer is unreadable naming that file with the parser's words verbatim
