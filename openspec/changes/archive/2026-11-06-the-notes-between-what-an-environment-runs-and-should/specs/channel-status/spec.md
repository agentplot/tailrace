## ADDED Requirements

### Requirement: Asking reaches no signer or target and writes nothing but the host's copy

Status SHALL ask the landing, the policy, the ledger and the host's
record, and the store only for the claims of the indexes an
environment's channel's chain moved to; it SHALL ask no signer and no
target anything (60, rule core.32). It SHALL write nothing but the
host's copy of a chain a store ledger holds and of each environment's
reports. With the store unreachable it SHALL answer the same channels,
chains and record (S10).

#### Scenario: The store unreachable

- **WHEN** every operation of the store is unreadable and status is asked
- **THEN** status answers the same channels, chains and record it answered before, each environment from its copy saying when, nothing is refused, and the signers and targets record no call

#### Scenario: Nothing written but the copy

- **WHEN** status is asked
- **THEN** every file of the host's record holds what it held before, the ledger records no append, the store records no write, and only files of the host's copy changed

## MODIFIED Requirements

### Requirement: Everything kept deleted answers the same

With everything timber keeps deleted but the record of claimed
versions, the reports' copy among it, status SHALL answer the same
channels, chains and environments from the ledger, the store and git,
and the same record once rebuilt (63, S12).

#### Scenario: The record deleted

- **WHEN** the host's record and the ledger's kept copies are deleted and status is asked again
- **THEN** the channels and chains equal those answered before, and after a rebuild the record's part equals it too

#### Scenario: The reports' copy deleted

- **WHEN** status is asked, the reports' copy is deleted, and status is asked again with the store answering
- **THEN** the environments' part of both answers is equal, and the copy is written again

### Requirement: A repository's row is made from its status

The core SHALL make one repository's row from its status answer alone:
the repository's owner and name, and for each channel shown its
component, its name and its head, the version it is at, empty, or that
channel's own answer as status gives it; for the environments, the
environments' part's four counts, how many run what they should, how
many are behind, how many wait on a person and how many cannot be read,
or the environments' part's own answer where it gives no counts (132,
58). Where the channels' part is not answered, the row SHALL carry that
answer in place of the heads. A surface SHALL compute no row of its own
(56, core.31).

#### Scenario: A row of heads

- **WHEN** a repository's status names `beta` empty and `stable` at 0.2.0, with one environment running what it should and one behind
- **THEN** its row names `beta` empty, `stable` at 0.2.0 with its component, and the counts one, one, none and none

#### Scenario: A row whose channels cannot be read

- **WHEN** a repository's status answers its channels' part unreadable
- **THEN** its row carries that unreadable answer, with the source's words, in place of the heads

## REMOVED Requirements

### Requirement: Asking reaches no store, signer or target and writes nothing

**Reason**: Status reads what an environment runs from its reports in
the store, fetched into the host's copy (60 and 98 as amended at
agentplot/blueprints `90cf023`; record 0029).

**Migration**: "Asking reaches no signer or target and writes nothing
but the host's copy" holds status to what it may ask and write.

### Requirement: Status names the environments it cannot yet read

**Reason**: Status reads every environment the policy declares from its
channel's chain and its reports (98, 132; record 0029).

**Migration**: The `environment-status` capability answers the
environments' part.
