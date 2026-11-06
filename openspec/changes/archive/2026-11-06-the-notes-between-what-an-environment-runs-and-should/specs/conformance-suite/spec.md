## ADDED Requirements

### Requirement: Environments in status over the fakes

The status group SHALL hold status's environments to 43, 58 to 61, 63,
91, 98, 132 and 134 over the fakes, with reports made by the fakes'
apply and no target asked: running what it should, behind with the
notes between, moved back, waiting, after another, no counted report,
an uncounted signer, S10 and S12 for the environments, a chain as last
read, and the four counts of the part and the row.

#### Scenario: Behind, with the notes between

- **WHEN** a world publishes three versions, `stable` is promoted to the third, and the fake apply reported `srv` running the first
- **THEN** status names `srv` behind, running the first, should run the third, with the second's and the third's notes, and the fake target records no call

#### Scenario: S10 for the environments

- **WHEN** status is asked, the fake store is then faulted unreadable on every operation, and status is asked again
- **THEN** each environment is shown as the first answer showed it, its reports read as last fetched at the first's moment with the store's words, and no part is refused

#### Scenario: S12 for the environments

- **WHEN** status is asked, the reports' copy is deleted, and status is asked again
- **THEN** the environments' part of both answers is equal

#### Scenario: No counted report

- **WHEN** a world declares `srv` and no apply has reported it
- **THEN** `srv` is unread saying no counted report names it, and is counted among those that cannot be read

### Requirement: S11 over the fakes

The status group SHALL play S11 over the fakes, citing 56, 57, 98 and
132: the overview names each environment with what it should run, what
it runs and whether they differ, and the row made from it carries the
part's four counts. The surfaces group SHALL hold the command line and
the served page to the core's answer with environments in it, value for
value but for the moment asked (56, 57, rule core.31).

#### Scenario: S11

- **WHEN** a world declares two environments, one running what it should and one behind, and status is asked of the core, through the command line with `--json`, and through the page's JSON
- **THEN** the three answers are equal but for the moment asked, each naming both environments with what each should run and runs and whether they differ, and the row's counts are one running what it should and one behind

### Requirement: The register after environments in status

98 SHALL leave the register, and the parts of 132 and 134 that show
environments SHALL leave its parts, in the commit whose scenarios first
cite them, and no entry SHALL name `phase 2 · environments in status`
after this change (rule core.33, ADR 0012).

#### Scenario: The coverage check after this change

- **WHEN** `moon run conformance:coverage` runs
- **THEN** 98 is cited and appears in no register entry, the register names no part of 132 or 134, and no entry names environments in status

### Requirement: Status's channels over the fakes, beside the environments

The status group SHALL hold status's channels to 31, 36, 58 to 61, 63,
101 and 131 over the fakes, below its marker, each scenario asking the
landing, the policy and the keeper, the store only for an environment's
reports, and writing nothing but the host's copy: every channel at its
version or empty; S10 for the channels; one chain unreadable and one
forked beside a channel shown; and S12 for the channels.

#### Scenario: Every channel at its version or empty

- **WHEN** the fake policy declares `beta` and `stable` of the world's component and no environment, and the fake keeper holds two promotions of `stable`
- **THEN** status names `beta` empty and `stable` at its head's version with both promotions newest first, and the store, signer and targets record no call

#### Scenario: S10 for status

- **WHEN** the fake store faults every operation, the fake keeper is made unreachable after one answer, and status is asked
- **THEN** each channel is shown as last read with its time and the keeper's words, no part is refused, and the store records no write

#### Scenario: One channel's answer beside another's

- **WHEN** the fake keeper's chain is faulted unreadable once with the words `connection reset` and another channel is forked
- **THEN** the first channel asked is unreadable carrying `connection reset`, the forked one refused naming both claims, and every other channel shown

#### Scenario: S12 for the channels

- **WHEN** status is asked, the host's record is deleted, and status is asked again
- **THEN** the channels' part of both answers is equal, and after a rebuild so is the record's

## MODIFIED Requirements

### Requirement: Status over the fakes

The status group SHALL hold status to 58, 60, 61 and 62: answered from
what the host keeps and what the ledger and the store answer, with no
signer or target asked, and nothing written but the host's copy.

#### Scenario: Status after a publish

- **WHEN** status is asked after a publish, in a world whose policy declares no environment
- **THEN** it answers the component, version, revision and index published with the notes the index names, byte for byte, no fake records a call after the publish's last, and no file of the record changes

#### Scenario: The record read with a file reader

- **WHEN** an entry of the host's record is read as plain JSON from its file
- **THEN** it names the component, version, revision, branch, index, store and the revisions the notes were rendered from, and nothing the store or git holds besides

#### Scenario: Nothing kept, and an entry that cannot be read

- **WHEN** status is asked with no record, and then with an entry that does not parse
- **THEN** the first is absent naming the directory, and the second unreadable naming the file

### Requirement: The register after the clan push profile

41 and 97 SHALL leave the register in the commit whose S16 scenario
first cites them, and no entry SHALL name `bolt 6 · the clan push
profile` after this change (rule core.33, ADR 0012). 44 and 126 SHALL
stay named by the phase the Nix cache target's change names them by.

#### Scenario: The coverage check after this change

- **WHEN** `moon run conformance:coverage` runs
- **THEN** 41 and 97 are cited by S16, appear in no register entry, and no entry names the clan push profile

## REMOVED Requirements

### Requirement: Channels in status over the fakes

**Reason**: Status reads every environment's reports from the store, so
the status group no longer holds the environments unreadable with a
target bound (98 as amended at agentplot/blueprints `90cf023`; record
0029).

**Migration**: "Status's channels over the fakes, beside the
environments" holds the channels' scenarios, and "Environments in status
over the fakes" the environments'.
