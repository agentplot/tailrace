## ADDED Requirements

### Requirement: Status is bound with nothing written

Where the host holds no bindings file, status SHALL be bound as 118
binds such a host, of what status asks: the landing, sluice where the
repository's own tree holds `.config/sluice.toml` and the plain branch
otherwise, and the git ref ledger, each with the remote `origin` where
the repository has one and no remote where it has none; the policy from
the repository; and no store and no signer, whose every operation
answers absent naming the binding (60, 92, 118, C.5, C.6, C.13). It
SHALL write nothing to bind them.

#### Scenario: A clone with no bindings file

- **WHEN** status is bound on a host with no bindings file, in a clone with a remote `origin`
- **THEN** the landing is the plain branch read from `origin`, the ledger the git ref fetched from `origin`, and no file or directory is made

#### Scenario: A repository set up for sluice

- **WHEN** the repository's tree holds `.config/sluice.toml` and the host has no bindings file
- **THEN** status's landing is sluice's

### Requirement: Status needs only what it asks

Where the host's bindings file is there, status SHALL be bound to its
`[landing]` and, where named, its `[ledger]` and `[store]`, opening the
store only for a store ledger and never making it; it SHALL need no
`[signing]`, and a file naming no `[ledger]` SHALL bind a ledger whose
every chain is absent naming `[ledger]` in the file, so each channel is
shown absent naming it (60, 118).

#### Scenario: A file naming a landing and no ledger

- **WHEN** status is bound by a file naming `[landing]` and no `[ledger]`, `[store]` or `[signing]`
- **THEN** the binding is made, and each channel's chain is absent naming `[ledger]` in the file
