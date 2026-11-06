## MODIFIED Requirements

### Requirement: The repository binds the build and the projections

The binding SHALL take the build, the projections and the targets from
the repository, not the host (92): moon is bound as the build, its tasks
named per output in the policy file at the landed revision (C.3), and
the file and image projections are bound by their ecosystem names, the
policy file naming each output's (C.10, C.11). Every target profile built
SHALL be bound by its profile name, each environment's `target` in the
policy file picking among them (C.10, C.12); a target that reads the
store SHALL read it through a store binding of its own, opened and never
made. The policy seam SHALL be bound to the policy file at the landed
revision (C.10).

#### Scenario: The bindings file names no build

- **WHEN** the bindings file names a store, a landing and a signer and nothing else
- **THEN** publish asks moon to ensure the outputs the policy file declares, and packs each with the projection the policy file names for it

#### Scenario: A build named in the host's file

- **WHEN** the bindings file holds a `[build]` table
- **THEN** publish and verify answer unreadable naming the table, since the build is the repository's to bind

#### Scenario: The targets are the repository's

- **WHEN** the bindings file names a store, a landing and a signer and nothing else, and `timber apply` runs
- **THEN** the binding holds the target `nix_cache`, reading through a store binding of its own that is not made

### Requirement: The file names the ledger

The file SHALL bind the ledger for the host with `[ledger]`, naming its
profile by its one key (92, 118): `git_ref`, with an optional `remote`
holding the refs, the repository the command runs in holding them where
none is named (C.13); or `store`, holding the chain in the store the
host binds, through a binding of its own that never makes the store
(C.14). `[ledger]` naming no profile, two, or an option its profile does
not read SHALL be unreadable naming the table. A file that names no
`[ledger]` SHALL bind the git ref on the remote the landing is bound
to, written or given, or with no remote where the landing has none
(118). A `[targets]` table, or any table naming a target, SHALL be
unreadable naming it, since the targets are the repository's to bind.

#### Scenario: A git ref ledger with a remote

- **WHEN** the file names `[ledger] git_ref = { remote = "origin" }`
- **THEN** a promotion pushes its claim to `refs/timber/ledger/<channel>` on `origin`, and the file names no credential

#### Scenario: A store ledger

- **WHEN** the file names `[store] layout` and `[ledger] store = {}`
- **THEN** a promotion's chain is walked from the channel's name in that layout, and the layout is not made by the ledger's binding

#### Scenario: A store ledger over the given layout

- **WHEN** the file names `[ledger] store = {}` and no `[store]`
- **THEN** the chain is held in the given layout, and the ledger's binding does not make it

#### Scenario: A ledger table naming two profiles

- **WHEN** `[ledger]` names both `git_ref` and `store`
- **THEN** every command bound by the file answers unreadable naming `[ledger]`, and nothing is read or written

#### Scenario: No ledger table

- **WHEN** the file names no `[ledger]` and `timber promote` runs, the landing bound to the remote `origin`
- **THEN** the promotion is pushed to `refs/timber/ledger/<channel>` on `origin`, after the chain is fetched from it

#### Scenario: Targets named in the host's file

- **WHEN** the bindings file holds a `[targets]` table
- **THEN** every command bound by the file answers unreadable naming `[targets]`, since the targets are the repository's to bind
