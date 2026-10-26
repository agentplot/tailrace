## MODIFIED Requirements

### Requirement: The host keeps which versions it published from which revisions

After a publish moves a version's name to its index, the host SHALL
keep one entry for that version naming its component, version, tag,
revision, branch, the index's digest and the store it was offered to,
the store as the host's bindings name it, and, where the index names
notes, the last release tag and the revisions of the commits its notes
were rendered from; and nothing the store or git holds besides: no
artifact, no claim, no notes' text and no credential (61, 64, 118, 134,
core.20, core.21).

#### Scenario: An entry after a publish

- **WHEN** a version is published
- **THEN** the host's record holds one entry for that component and version naming the revision, the branch, the index's digest, the store and the revisions its notes were rendered from, and no envelope, artifact digest, notes' text or credential

#### Scenario: The store as the bindings name it

- **WHEN** the built binary publishes a version on a host whose bindings file names `[store] layout` with a path beginning `~/`, and status is asked with `--json`
- **THEN** the version's entry names the store as `layout` and that path with `~/` expanded

#### Scenario: Nothing kept of a refused publish

- **WHEN** a publish is refused or stops before its index
- **THEN** the host's record holds no entry for that version

#### Scenario: An entry written before notes

- **WHEN** the record holds an entry written before entries named the notes' commits, and the same version, whose index names no notes, is published again
- **THEN** the entry reads back naming no notes and is answered held, its file unchanged

### Requirement: Status answers from the record alone

`tailrace status` SHALL answer, for each component, the versions this
host published, each with its revision, branch, index's digest and
store, and its notes where its entry names them, from the host's record
and the repository's own history alone, ordered by the version rules
(23), and SHALL reach no store, signer or target and write nothing (60,
132, core.32).

#### Scenario: Status after a publish

- **WHEN** a version is published and status is asked
- **THEN** status answers that component's version, revision, index's digest and notes, and the fake store, signer and targets record no call made by status

#### Scenario: Status writes nothing

- **WHEN** status is asked
- **THEN** every file of the host's record holds after it what it held before it, and no file is added

#### Scenario: Which version a revision became

- **WHEN** status is asked for one revision this host published
- **THEN** it answers the version each component took at that revision, and no other

### Requirement: The record is rebuilt from the store and git

A rebuild SHALL write the host's record from what the store and git
hold: for each component the policy file declares at a landed revision,
each tag carrying its prefix that names a revision the branch landed,
and whose name the store resolves to that component's index at that
version, one entry, the same file a publish of it writes. Where that
index names notes, the entry's commits SHALL be those the version rule
derives the version from again at the tag's revision, under the policy
file at that revision (21, 61 to 63, 134).

#### Scenario: S12, everything kept is deleted and rebuilt

- **WHEN** two components' versions are published, the host's record is deleted, and the record is rebuilt
- **THEN** every file of the record holds the same bytes it held before the deletion, the notes' commits among them, and status answers the same as before

#### Scenario: A claimed version with no index

- **WHEN** a version's tag is claimed but the store names no index under it, and the record is rebuilt
- **THEN** the record holds no entry for that version, as a publish that stopped before its index keeps none

#### Scenario: A version another branch landed

- **WHEN** a tag carrying a component's prefix names a revision the branch asked for did not land, and the record is rebuilt for that branch
- **THEN** the record holds no entry for that tag

#### Scenario: A version published before notes

- **WHEN** a tag's index names no notes and the record is rebuilt
- **THEN** that version's entry names no notes' commits

## ADDED Requirements

### Requirement: Status shows each version's notes from git

For each version whose entry names the commits its notes were rendered
from, status SHALL render those commits again from the repository's own
history with the rule's tool and the same form a publish renders, and
show them beside the version. A version whose entry names none SHALL
show no notes; a commit the history no longer holds SHALL make that
version's notes unreadable naming the commit, and the rest of the
answer SHALL stand (58, 60, 132, 134, core.19).

#### Scenario: The notes status shows are the notes carried

- **WHEN** a version is published and status is asked
- **THEN** the notes status shows are byte for byte the bytes of the `notes` artifact the version's index names in the fake store

#### Scenario: No notes, and notes that cannot be read

- **WHEN** status is asked with one entry naming no notes and another naming a commit the history does not hold
- **THEN** the first version is answered with no notes, the second with its notes unreadable naming that commit, and both versions are answered with their revision and index

#### Scenario: Notes on the command line

- **WHEN** `tailrace status` is asked as text and with `--json` after a publish
- **THEN** the text prints the notes indented beneath the version's line, and the JSON carries the notes beside the version in the one answer form, `answer` naming `ok` with the text
