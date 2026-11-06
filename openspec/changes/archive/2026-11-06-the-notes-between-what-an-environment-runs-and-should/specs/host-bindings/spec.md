## MODIFIED Requirements

### Requirement: Every host binds every signer profile for verify

The binding SHALL bind both signer profiles on every host: the one
`[signing]` names first, signing as its signer, and the other verify-only,
so a claim made by either kind of signer is told on any host whatever it
signs with (80, 92). A host whose file names no `[signing]` SHALL bind
the given host key first, signing as the host's name, for publish and
promote, and both profiles verify-only for every other command. For
status, both profiles SHALL be bound verify-only and the key service
profile bound to reach no service, whatever `[signing]` names, since
status reaches no signer (60).

#### Scenario: A host key host

- **WHEN** a host whose `[signing]` names a host key is bound for verify
- **THEN** the signers bound are the host key, signing as its signer, and then the key service signer, verify-only

#### Scenario: A key service host

- **WHEN** a host whose `[signing]` names a KMS key ARN is bound for publish
- **THEN** the binding's signer is that ARN, and the signers bound are the key service signer bound to that key, and then the host key, verify-only

#### Scenario: A host that names no signer, publishing

- **WHEN** a host whose file names no `[signing]` is bound for publish
- **THEN** the binding's signer is the host's name, and the signers bound are the given host key, signing as it, and then the key service signer, verify-only, and no key is read or made in binding them

#### Scenario: A host that signs nothing

- **WHEN** a host whose file names no `[signing]` is bound for verify
- **THEN** both signer profiles are bound verify-only, and nothing is read, made or reached in binding them

#### Scenario: A key service host asking status

- **WHEN** a host whose `[signing]` names a KMS key ARN is bound for status
- **THEN** both signer profiles are bound verify-only, the key service profile reaching no service, and the binding names no signer

### Requirement: Status is bound with nothing written

Where the host holds no bindings file, status SHALL be bound as 118
binds such a host, of what status asks: the landing, sluice where the
repository's own tree holds `.config/sluice.toml` and the plain branch
otherwise, and the git ref ledger, each with the remote `origin` where
the repository has one and no remote where it has none; the policy from
the repository; the given layout as the store, opened and never made;
and the signer profiles verify-only, reaching no service (60, 92, 118,
C.5, C.6, C.13). A layout that is not there SHALL be bound as a store
whose every operation answers unreadable with the opening's words. It
SHALL write nothing to bind them.

#### Scenario: A clone with no bindings file

- **WHEN** status is bound on a host with no bindings file, in a clone with a remote `origin`
- **THEN** the landing is the plain branch read from `origin`, the ledger the git ref fetched from `origin`, and no file or directory is made

#### Scenario: A repository set up for sluice

- **WHEN** the repository's tree holds `.config/sluice.toml` and the host has no bindings file
- **THEN** status's landing is sluice's

#### Scenario: A host that never published

- **WHEN** status is bound on a host with no bindings file whose given layout is not there
- **THEN** the binding is made, the store answers every operation unreadable naming the layout, and no layout is made

### Requirement: Status needs only what it asks

Where the host's bindings file is there, status SHALL be bound to its
`[landing]`, its `[store]` or the given layout, opened for the
environments' reports and for a store ledger and never made, and, where
named, its `[ledger]`; it SHALL need no `[signing]`, and a file naming
no `[ledger]` SHALL bind a ledger whose every chain is absent naming
`[ledger]` in the file, so each channel is shown absent naming it (60,
118). A store that cannot be opened SHALL be bound as a store whose
every operation answers unreadable with the opening's words, so the
channels are read and each environment is answered from its copy (61).

#### Scenario: A file naming a landing and no ledger

- **WHEN** status is bound by a file naming `[landing]` and no `[ledger]`, `[store]` or `[signing]`
- **THEN** the binding is made, and each channel's chain is absent naming `[ledger]` in the file

#### Scenario: A store that cannot be opened

- **WHEN** status is bound by a file whose `[store]` names a layout that is not there and whose `[ledger]` is the git ref
- **THEN** the binding is made, the channels are read from the git ref, and the store answers every operation unreadable naming the layout
