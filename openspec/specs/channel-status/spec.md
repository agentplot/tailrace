# channel-status Specification

## Purpose
TBD - created by archiving change status-on-any-machine. Update Purpose after archive.

## Requirements

### Requirement: Status shows every channel the policy declares

Status SHALL answer, for the branch asked (by default `main`), the
landed revision at the branch's head that its policy was read at, and
every channel the policy at that revision declares, as the policy seam
lists them (131): each with the component it belongs to (125), how it
moves, and the channel it is bound to where it is one (37, 83), and
either the one version its chain's head moved it to, by version and
index, or that it is empty (31, 101). A channel's version SHALL be read
from the ledger alone, never from a name in the store (101, 106). Asked
for one component, status SHALL show that component's channels alone.

#### Scenario: Two channels, one promoted

- **WHEN** the policy declares `beta` and `stable` of one component, and the ledger holds two promotions of `stable` and none of `beta`
- **THEN** status names both, `beta` empty and `stable` at the version and index of its second promotion, each with its component and how it moves

#### Scenario: A host that never published

- **WHEN** status is asked on a host whose record holds nothing, of a repository whose ledger holds a promotion
- **THEN** the channel is shown at its version, and the record's part of the answer is absent naming the record, apart from the channels

#### Scenario: A policy that declares no channel

- **WHEN** the policy at the landed head declares no channel
- **THEN** status answers an empty list of channels, never absent

#### Scenario: A head not landed

- **WHEN** the local branch's head is a revision the landing has not landed
- **THEN** the channels' part is refused naming the branch and the revision, and the record's part still answers

### Requirement: Each channel shows the chain that put it there

Beside a channel's version, status SHALL show every promotion of its
chain, newest first, each with the version and index it moved to and
from (none for the first), its claim's digest and its predecessor's,
who asked, the policy revision it was judged under with the claims that
satisfied it, and when (33, 105).

#### Scenario: A move and a move back

- **WHEN** `stable` was promoted to 0.1.0, then 0.2.0, then back to 0.1.0
- **THEN** status shows `stable` at 0.1.0 with three promotions, newest first, the last moving from 0.2.0, each naming its claim and predecessor

### Requirement: Every reader gets one answer as of when it read

Status SHALL say when it asked, in seconds since the Unix epoch and as
a person reads a time in UTC, and every reader of one ledger SHALL see
one version per channel (36). Where the ledger answers a chain as this
host last read it, status SHALL show that chain, saying when it was
read and carrying the remote's or store's own words verbatim, and SHALL
never show it as refused or as empty (36, 60, 61, C.13, C.14). Status
SHALL take the landing's answer as the landing seam gives it: where the
landing cannot be read, the channels' part SHALL be unreadable carrying
the landing's words, never refused, and the record's part SHALL still
answer (58, 59, 77).

#### Scenario: Two machines, one answer

- **WHEN** one machine promotes and a second machine, which never published, asks status of the same repository
- **THEN** both answer the same version and chain for every channel

#### Scenario: The ledger's remote cannot be reached

- **WHEN** the landing answers the branch's head landed, and the ledger's remote cannot be reached after this host last fetched the chain
- **THEN** the channel is shown at the version of the chain as last fetched, with when it was fetched and the remote's words, and nothing is shown refused

#### Scenario: The landing cannot be read

- **WHEN** the landing answers unreadable with its source's words
- **THEN** the channels' part is unreadable carrying those words verbatim, never refused, the record's part answers, and status exits 0

### Requirement: Each channel's answer is its own

Status SHALL answer each channel apart: a chain the ledger cannot read
SHALL be that channel's unreadable answer with the ledger's words, a
fork the ledger refuses that channel's refusal naming the claims, and a
channel never promoted empty, and SHALL never answer one as another
(58, 59, 105, core.19). One channel's answer SHALL leave every other
channel shown as it is.

#### Scenario: One chain cannot be read

- **WHEN** the ledger answers `beta`'s chain unreadable with the words `fatal: unable to access` and `stable`'s chain as held
- **THEN** `beta` is shown unreadable carrying those words verbatim, `stable` is shown at its version, and status exits 0

#### Scenario: A forked chain

- **WHEN** the ledger refuses `stable`'s chain as a fork
- **THEN** `stable` is shown refused naming both claims, and never empty

### Requirement: Asking reaches no store, signer or target and writes nothing

Status SHALL ask only the landing, the policy and the ledger, and the
host's record, and SHALL ask the store, a signer or a target nothing,
and write nothing to the record or the repository (60, core.32).
Status SHALL answer the same with the store unreachable (S10).

#### Scenario: The store unreachable

- **WHEN** every operation of the store is unreadable and status is asked
- **THEN** status answers the same channels, chains and record it answered before, nothing is refused, and the store, signer and targets record no call

#### Scenario: Nothing written

- **WHEN** status is asked
- **THEN** every file of the host's record holds what it held before, and the ledger records no append

### Requirement: Everything kept deleted answers the same

With everything tailrace keeps deleted but the record of claimed
versions, status SHALL answer the same channels and chains from the
ledger and git, and the same record once rebuilt (63, S12).

#### Scenario: The record deleted

- **WHEN** the host's record and the ledger's kept copies are deleted and status is asked again
- **THEN** the channels and chains equal those answered before, and after a rebuild the record's part equals it too

### Requirement: Every surface gets the same status from the one core

The command line's `--json` answer and the served page's JSON SHALL be
the core's status answer, value for value, and the page SHALL render
that answer and compute nothing of its own (56, 57, core.31).

#### Scenario: The command line and the page

- **WHEN** status is asked through the command line with `--json` and through the served page's JSON over one binding
- **THEN** each equals the core's answer to the same question, value for value but for the moment each asked, which lies between the moments before and after the asking

### Requirement: Status names the environments it cannot yet read

Beside the channels and the record, status SHALL answer the
environments as a part of its own, and, while no target is read, SHALL
answer it unreadable, naming the environments and saying what each
should run and runs is read from a target and none is read; it SHALL
never answer the part empty, as a count of none, or as running what
they should, and SHALL ask no target to answer it (58, 60, 98). The
part SHALL leave the channels and the record answered as they are.

#### Scenario: A target bound and never asked

- **WHEN** status is asked over a binding that holds a target for an environment
- **THEN** the environments' part is unreadable saying no target is read, the channels are shown, and the target records no call

### Requirement: A repository's row is made from its status

The core SHALL make one repository's row from its status answer alone:
the repository's owner and name, and for each channel shown its
component, its name and its head, the version it is at, empty, or that
channel's own answer as status gives it; for the environments, how many
run what they should, how many are behind, how many wait on a person
and how many cannot be read, or the environments' part's own answer
where it gives no counts (132, 58). Where the channels' part is not
answered, the row SHALL carry that answer in place of the heads. A
surface SHALL compute no row of its own (56, core.31).

#### Scenario: A row of heads

- **WHEN** a repository's status names `beta` empty and `stable` at 0.2.0
- **THEN** its row names `beta` empty, `stable` at 0.2.0 with its component, and the environments' unreadable answer in place of the counts

#### Scenario: A row whose channels cannot be read

- **WHEN** a repository's status answers its channels' part unreadable
- **THEN** its row carries that unreadable answer, with the source's words, in place of the heads

### Requirement: Status says when the landing was fetched

The channels' part of status SHALL say how the landing of the branch's
head was read: from the landing itself, or from the host's copy as last
fetched, with when it was fetched, as seconds since the Unix epoch and
as a person reads a time in UTC, and the remote's words verbatim; the
channels SHALL then be shown as for a landing read from its remote, on
the command line, in its JSON and on the host's page alike (36, 56, 57,
60, 77).

#### Scenario: The landing's remote cannot be reached

- **WHEN** the landing answers the branch's head landed from its copy as last fetched, with a time and the remote's words
- **THEN** status shows every channel the policy declares, says the landing was read as last fetched at that time in UTC with those words verbatim, shows nothing refused, and exits 0

#### Scenario: Cut off from both remotes

- **WHEN** a machine that read the landing and the ledger from one remote is cut off from it, and asks status
- **THEN** every channel is shown at the version of its chain as last read, the landing and each chain saying when they were fetched, and the page and `--json` give the same answer as the command line

#### Scenario: The head not landed as last fetched

- **WHEN** the landing answers the branch's head not landed from its copy
- **THEN** the channels' part is refused naming the copy's time and the remote's words, saying to ask again, and the record's part answers

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
