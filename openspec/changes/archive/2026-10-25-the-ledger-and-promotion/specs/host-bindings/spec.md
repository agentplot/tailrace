## ADDED Requirements

### Requirement: The file names the ledger

The file SHALL bind the ledger for the host with `[ledger]`, naming its
profile by its one key (92, 118): `git_ref`, with an optional `remote`
holding the refs, the repository the command runs in holding them where
none is named (C.13); or `store`, holding the chain in the store
`[store]` names, through a binding of its own that never makes the
store (C.14). `[ledger]` naming no profile, two, or an option its
profile does not read SHALL be unreadable naming the table. A command
that needs the ledger on a host whose file names no `[ledger]` SHALL
answer absent naming `[ledger]` before anything is read; the binding
SHALL hold no target.

#### Scenario: A git ref ledger with a remote

- **WHEN** the file names `[ledger] git_ref = { remote = "origin" }`
- **THEN** a promotion pushes its claim to `refs/tailrace/ledger/<channel>` on `origin`, and the file names no credential

#### Scenario: A store ledger

- **WHEN** the file names `[store] layout` and `[ledger] store = {}`
- **THEN** a promotion's chain is walked from the channel's name in that layout, and the layout is not made by the ledger's binding

#### Scenario: A ledger table naming two profiles

- **WHEN** `[ledger]` names both `git_ref` and `store`
- **THEN** every command bound by the file answers unreadable naming `[ledger]`, and nothing is read or written

#### Scenario: No ledger table

- **WHEN** the file names no `[ledger]` and `tailrace promote` runs
- **THEN** it answers absent naming `[ledger]`, and nothing is read or written

## REMOVED Requirements

### Requirement: The ledger is not yet bound

**Reason**: The ledger's two profiles are built, and the file binds the
ledger with `[ledger]` (record 0013's consequence, settled by this
unit).

**Migration**: A host that promotes adds `[ledger]` naming `git_ref` or
`store`; a host that never promotes needs no change.
