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

`tailrace status` SHALL answer, as the record's part of its answer and
beside the channels, for each component, the versions this host
published, each with its revision, branch, index's digest and store,
from the host's record alone, ordered by the version rules (23), and
SHALL reach no store, signer or target and write nothing (60,
core.32).

#### Scenario: Status after a publish

- **WHEN** a version is published and status is asked
- **THEN** the record's part answers that component's version, revision and index's digest, and the fake store, signer and targets record no call made by status

#### Scenario: Status writes nothing

- **WHEN** status is asked
- **THEN** every file of the host's record holds after it what it held before it, and no file is added

#### Scenario: Which version a revision became

- **WHEN** status is asked for one revision this host published
- **THEN** the record's part answers the version each component took at that revision, and no other

### Requirement: Status keeps absent and unreadable apart

The record's part of status SHALL answer absent when the host keeps
nothing for the repository, and unreadable naming the file, with the
parser's words, when an entry cannot be read; it SHALL never answer an
unreadable entry as absent or as an empty list (58, 59, core.19). An
absent or unreadable record SHALL leave the channels' part answered as
it is, and status SHALL exit 0 with both parts.

#### Scenario: Nothing kept

- **WHEN** status is asked in a repository this host never published from
- **THEN** the record's part is absent, naming the record it looked in, and the channels are shown

#### Scenario: An entry that cannot be read

- **WHEN** one entry of the record is not well formed
- **THEN** the record's part is unreadable naming that file with the parser's words verbatim

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
