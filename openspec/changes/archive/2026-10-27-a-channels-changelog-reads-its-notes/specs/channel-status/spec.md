## ADDED Requirements

### Requirement: A channel's changelog is its chain read with each promotion's notes

Beside each channel whose chain holds a promotion, status SHALL answer
the notes of every version the chain moved the channel to, once per
version, by version and index, so each promotion of the chain is shown
with who asked and the notes of the version it moved to (134, 33). A
channel that is empty, or whose chain is not answered, SHALL answer no
notes.

#### Scenario: Two promotions, two versions' notes

- **WHEN** two versions are published and `stable` is promoted to the first and then the second
- **THEN** status answers `stable`'s notes for both versions, each byte for byte the bytes of the `notes` artifact that version's index names in the fake store, and each promotion of the chain names its asker

#### Scenario: A move back

- **WHEN** `stable` is promoted to 0.1.0, then 0.2.0, then back to 0.1.0
- **THEN** status answers notes for 0.1.0 and 0.2.0 once each, and the three promotions are each shown with the notes of the version they moved to

#### Scenario: An empty channel

- **WHEN** the policy declares `beta` and its chain holds no promotion
- **THEN** status answers `beta` empty with no notes

#### Scenario: On the command line

- **WHEN** `tailrace status` is asked as text and with `--json` after a promotion
- **THEN** the text prints the notes of the version the promotion moved to indented beneath the promotion's line, and the JSON carries each version's notes beside the channel in the one answer form

### Requirement: The changelog's notes are rendered from git on any machine

Status SHALL render a version's notes in the changelog from the
repository's history alone: the version's tag that the branch's landed
head reaches, its revision landed, the commits the version rule derives
there under the policy file that revision holds, rendered by the rule's
tool as a publish renders them (21, 63, 134). It SHALL ask no store,
signer or target for them and write nothing (60, core.32), so every
reader of one ledger and one history is shown the same notes (36).

#### Scenario: A host that never published

- **WHEN** one machine publishes and promotes a version, and a second machine whose record holds nothing asks status of the same repository
- **THEN** the second machine answers the same notes for that version as the first

#### Scenario: The store unreachable

- **WHEN** every operation of the store is unreadable and status is asked of a channel promoted twice
- **THEN** status answers the same notes it answered before, and the store and signer record no call

### Requirement: Each version's notes in the changelog are their own answer

A version whose notes cannot be read SHALL answer them unreadable naming
what could not be read: its tag where the history the landed head
reaches holds none, a commit the history no longer holds, or the
landing's or the policy file's own words; it SHALL never answer them
empty or absent (58, 59, core.19). The channel's version, its chain and
every other version's notes SHALL be shown as they are.

#### Scenario: A tag the history does not hold

- **WHEN** a channel's chain moved it to two versions and the local history no longer holds the tag of the first
- **THEN** the first version's notes are unreadable naming its tag, the second's are answered, and the channel is shown at its version with its whole chain

#### Scenario: A commit the history no longer holds

- **WHEN** a version's notes would be rendered from a commit the local history does not hold
- **THEN** that version's notes are unreadable naming the commit, and status exits 0
