# environment-status Specification

## Purpose
TBD - created by archiving change the-notes-between-what-an-environment-runs-and-should. Update Purpose after archive.

## Requirements

### Requirement: Status shows every environment the policy declares

Beside the channels and the record, status SHALL answer every
environment the policy declares at the branch's landed head (136), of
the component asked where status names one, in the policy seam's order,
each with its name, its component, the channel it follows, whether it
waits, and its approvers (41, 97, 98). Where the landed head or the
policy's environments cannot be read, the part SHALL be that answer,
the channels and the record shown as they are (58).

#### Scenario: Two environments of one component

- **WHEN** the policy declares `srv`, which moves on its own on `stable`, and `desk`, which waits on `stable` with `ops` its approver
- **THEN** status names both, `srv` on `stable` not waiting and `desk` on the channel made for it, waiting, with `ops` its approver

#### Scenario: One component asked

- **WHEN** status is asked for one of two declared components
- **THEN** only that component's environments are shown, and the counts are of those alone

### Requirement: A policy declaring no environment answers an empty list

Where the policy at the landed head declares no environment, the
environments' part SHALL answer an empty list with four counts of none,
the policy's own answer, never absent (58).

#### Scenario: No environment declared

- **WHEN** the policy at the landed head declares no environment
- **THEN** the environments' part answers an empty list with four counts of none

### Requirement: What an environment should run is its channel's head as status shows it

What an environment should run SHALL be the head of the chain of the
channel it follows, as the channels part of the same status answers
it: a version and its index, empty, or that chain's own answer, read
once for both parts (40, 88, rule core.50).

#### Scenario: The head the channels part shows

- **WHEN** `stable` is at 0.4.0 and `srv` follows it
- **THEN** `srv` should run 0.4.0 at the index the channels part names for `stable`'s head

#### Scenario: A chain that cannot be read

- **WHEN** the ledger answers the chain of the channel `srv` follows unreadable with its words
- **THEN** what `srv` should run carries that answer verbatim, and its state is unread naming it

### Requirement: What an environment runs is its newest counted report

What an environment runs SHALL be its newest counted report among the
report claims of that environment and component attached to the
indexes its channel's chain moved to, with its version, its index, the
time the target says it converged, the apply's moment, the claim's
digest and its signer (43, 91, 98). It SHALL never be asked of a target
(60). Where no report counts, it SHALL be absent naming the
environment, saying so where the policy lets no signer claim one (58).

#### Scenario: Two reports, the later counted

- **WHEN** `srv` reported 0.3.0 from an apply at 10:00 and 0.4.0 from an apply at 11:00, both by a signer whose kinds hold `report`
- **THEN** `srv` runs 0.4.0, with the converged time its report names, and the fake target records no call

#### Scenario: A report by a signer that may not report

- **WHEN** the only report of `srv` is signed by a signer whose kinds lack `report`
- **THEN** what `srv` runs is absent naming `srv`, and its state is unread with that reason

#### Scenario: A policy that lets no signer report

- **WHEN** the policy lists no signer with `report` among its kinds
- **THEN** what each environment runs is absent, saying the policy lets no signer claim a report

### Requirement: A report counts without reaching a signer

A report SHALL count only through a signer the policy's answer for a
layout lists with `report` among its kinds, told by the signer
profiles bound verify-only, none of which reaches a key service (116,
rule core.7, 60, C.7). Status SHALL tell the reports newest first and
stop at the first that counts. A report whose signer cannot be told
before one counts SHALL make what the environment runs unreadable
naming what could not be told, never passed over (rule core.42).

#### Scenario: A report only a key service could tell

- **WHEN** `srv`'s newest report carries a P-256 signature and the policy lists its signer by a KMS key ARN
- **THEN** what `srv` runs is unreadable naming the key, saying status reaches no key service, and no request reaches KMS

#### Scenario: An older report that cannot be told

- **WHEN** `srv`'s newest report counts through a host key and an older one carries a signature only a key service could tell
- **THEN** `srv` runs the newest report's version, and the older one is never told

### Requirement: The reports are fetched into the host's copy

Status SHALL fetch each environment's reports from the store, the
claims of each index its channel's chain moved to, asked once per
status whichever environments' chains moved to it. Once every index's
claims are read, it SHALL keep that environment's report claims, each
with its index and uncounted, in the host's copy under the repository's
common git directory, with the moment it fetched them, and write
nothing else (60, 61).

#### Scenario: One index asked once

- **WHEN** `srv` and `desk` follow channels whose chains both moved to one index
- **THEN** the store records one call of `claims_of` for that index in the one status

#### Scenario: The copy read with a file reader

- **WHEN** `srv`'s copy is read as plain JSON from its file
- **THEN** it names the moment of the fetch and each report claim with the index it is attached to, and nothing else

### Requirement: The copy answers where the store does not

Where the store answers any index's claims unreadable, status SHALL
write nothing and answer from the copy, saying when it was fetched and
carrying the store's words verbatim (59, 60, S10). With no copy, what
the environment runs SHALL be unreadable naming its reports with the
store's words. An index the store answers absent, and a copy that
cannot be read or written, SHALL be unreadable naming the index or the
file, the copy left as it was (58).

#### Scenario: The store unreachable after a fetch

- **WHEN** status is asked once, every operation of the store is then faulted unreadable, and status is asked again
- **THEN** the second answer shows each environment running what the first showed, saying the reports were read as last fetched at the first's moment with the store's words, nothing refused

#### Scenario: Never fetched, the store unreachable

- **WHEN** status is asked on a host that never fetched `srv`'s reports, with the store unreadable
- **THEN** what `srv` runs is unreadable naming its reports with the store's words, its state is unread, and status exits 0

#### Scenario: An index the store does not hold

- **WHEN** the store answers absent for an index `srv`'s channel's chain moved to
- **THEN** what `srv` runs is unreadable naming that index, and the copy holds what it held before

### Requirement: Each environment's state

An environment SHALL be unread, carrying the reason, where what it
should run or what it runs is not answered; behind where its newest
counted report names another index than its channel's head (rule
core.4); waiting where it waits and the channel its own is bound to is
at another index than its own; and running what it should otherwise
(98). A waiting environment that is behind SHALL be behind.

#### Scenario: Behind

- **WHEN** `stable` is at 0.4.0 and `srv`'s newest counted report names 0.3.0
- **THEN** `srv` is behind

#### Scenario: Waiting on a person

- **WHEN** `desk` waits on `stable`, its own channel is at 0.3.0, `stable` is at 0.4.0, and `desk`'s newest counted report names 0.3.0
- **THEN** `desk` is waiting, naming `stable` at 0.4.0 and its approver `ops`

### Requirement: An empty channel is unread and an after is never waiting

An environment whose channel is empty SHALL be unread, saying no
counted report names it and its channel is empty, and one that comes
after others SHALL never be waiting on a person, since its channel
moves on reports (37, 98).

#### Scenario: An empty channel

- **WHEN** `srv` follows a channel whose chain holds no promotion
- **THEN** `srv` is unread, saying no counted report names it and its channel is empty, and is counted among those that cannot be read

#### Scenario: After another, its channel apart from the one it is bound to

- **WHEN** `second` comes after `first`, runs its channel's head, and the channel its own is bound to is at another index
- **THEN** `second` is running what it should, never waiting

### Requirement: What an environment waits on

Beside its state, status SHALL answer what an environment's channel
moves on: for one that waits, its approvers and the channel its own is
bound to with that channel's head; for one that comes after others, the
environments its channel comes after; and nothing for one that moves on
its own (37, 41, 97, 132).

#### Scenario: An environment after another

- **WHEN** `second` comes after `first`
- **THEN** `second` names `first` as what its channel moves on, and is never waiting on a person

### Requirement: The versions between what an environment runs and what it should

Beside an environment that is behind, status SHALL answer the versions
between what it runs and what it should, ahead where what it should run
is higher by semantic precedence and back where it is lower (23, 134):
the component's tags the landed head reaches above the lower and at or
below the higher, releases only, with the higher where it is a
prerelease, in ascending order, each by version and its tag's revision.

#### Scenario: Two versions ahead

- **WHEN** tags 0.2.0, 0.3.0 and 0.4.0 are landed, `stable` is at 0.4.0, and `srv` runs 0.2.0
- **THEN** `srv`'s between is ahead with 0.3.0 and 0.4.0

#### Scenario: Back

- **WHEN** `stable` was moved back to 0.2.0 and `srv` still runs 0.4.0
- **THEN** `srv`'s between is back with 0.3.0 and 0.4.0, the versions whose changes it leaves

#### Scenario: A version the channel skipped

- **WHEN** `stable` moved from 0.2.0 straight to 0.4.0, and `srv` runs 0.2.0
- **THEN** `srv`'s between holds 0.3.0's notes as well as 0.4.0's

#### Scenario: Prereleases between

- **WHEN** tags 0.3.0, 0.4.0-rc.1 and 0.4.0 are landed, `srv` runs 0.3.0, and its channel is at 0.4.0
- **THEN** `srv`'s between holds 0.4.0 alone

### Requirement: Each version between carries its notes

Each version between SHALL carry its notes rendered from the landed
history as the channels' changelog renders them, once per status and
asking no store (21, 60, 134). An environment that is not behind SHALL
answer no versions between.

#### Scenario: The notes of the versions between

- **WHEN** `srv` is behind by 0.3.0 and 0.4.0, and `stable`'s changelog holds 0.4.0
- **THEN** each version between carries its notes, 0.4.0's byte for byte the changelog's, and the store records no call for them

#### Scenario: Not behind

- **WHEN** `srv` runs what it should, waits, or cannot be read
- **THEN** status answers no versions between for it

### Requirement: Each version between is its own answer

A version between whose notes cannot be rendered SHALL be that
version's unreadable answer, every other version shown (rule core.19).
Tags that cannot be read, or a version that does not parse as a
semantic version, SHALL make the versions between unreadable naming it,
never empty. The environment's state, what it should run and what it
runs SHALL be shown as they are.

#### Scenario: One version's notes cannot be rendered

- **WHEN** one version between is tagged at a revision whose notes' template the version rule's tool cannot read
- **THEN** that version's notes are unreadable naming the template with the tool's words, the other versions' notes are shown, and status exits 0

#### Scenario: A commit the history no longer holds

- **WHEN** a commit between what `srv` runs and what it should is no longer in the local history
- **THEN** the versions between are unreadable naming the tags the head reaches, with git's words naming the commit, and `srv`'s state, what it should run and what it runs are shown as they are

### Requirement: The four counts are of the environments shown

The environments' part SHALL carry how many of the environments it
lists run what they should, how many are behind, how many wait on a
person and how many cannot be read, each environment in exactly one
count, beside the list (132).

#### Scenario: One of each

- **WHEN** four environments are shown, one running what it should, one behind, one waiting and one unread
- **THEN** the part's counts are one, one, one and one

### Requirement: The environments on the command line

`timber status` SHALL print the four counts under `environments`, then
each environment on its own line with its state, what it should run and
what it runs with when it converged, saying where the reports were read
as last fetched, and the notes of each version between indented beneath
it; `--json` SHALL carry the part in the one answer form (57).

#### Scenario: An environment behind, as text and JSON

- **WHEN** `timber status` is asked as text and with `--json` while `srv` is behind by two versions
- **THEN** the text prints `srv` behind with both versions and their notes indented beneath, and the JSON carries the same part, counts and versions, value for value
