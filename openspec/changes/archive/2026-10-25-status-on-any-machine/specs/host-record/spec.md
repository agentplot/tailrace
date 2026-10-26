## MODIFIED Requirements

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
