## ADDED Requirements

### Requirement: A host with no bindings file is bound to the given profiles

A host with no bindings file SHALL be bound, for every command that
reads one, as if its file were empty: the store, the landing, the ledger
and the signer each bound to its given profile, and nothing answered
absent for the file or any table of it (118). A host where neither
`XDG_CONFIG_HOME` nor `HOME` is set SHALL still answer absent naming
both, since no given path can be placed.

#### Scenario: Publish with nothing written

- **WHEN** `tailrace publish <revision>` runs on a host with no bindings file, in a repository whose branch has landed the revision
- **THEN** the version is offered into the given layout, signed by the given host key, and the host's record names the store as `layout <path>` with the given layout's full path

#### Scenario: An empty file

- **WHEN** the bindings file is there and holds nothing
- **THEN** every command is bound exactly as on a host with no file

#### Scenario: No home and no configuration directory

- **WHEN** neither `XDG_CONFIG_HOME` nor `HOME` is set
- **THEN** publish answers absent naming both variables, and nothing is read or written

### Requirement: A table the file leaves out is bound to its given profile

Each of `[store]`, `[landing]`, `[ledger]` and `[signing]` the file names
SHALL bind what it names; each the file leaves out SHALL bind its given
profile (118). The given store SHALL be an OCI layout at
`$XDG_DATA_HOME/tailrace/layout`, or `~/.local/share/tailrace/layout`
where that variable is unset or empty (C.2); the given ledger the git
ref in the repository the command runs in, with no remote (C.13); the
given signer the host key of the host-key-signer capability (C.8).

#### Scenario: A registry and nothing else

- **WHEN** the file names `[store] registry` and no other table
- **THEN** publish writes into that registry, reads the landing the repository is set up for, signs with the given host key, and a promote appends to `refs/tailrace/ledger/...` in the repository

#### Scenario: A key service and nothing else

- **WHEN** the file names `[signing] kms` and no other table
- **THEN** publish signs through that key and writes into the given layout, and no host key is made

#### Scenario: The data directory set

- **WHEN** `XDG_DATA_HOME` names a directory and the file names no `[store]`
- **THEN** the store is the layout at `tailrace/layout` under that directory

#### Scenario: A promote with no ledger table

- **WHEN** the file names no `[ledger]` and `tailrace promote` runs
- **THEN** the promotion is appended to the channel's git ref in the repository the command runs in, read and written with no remote

### Requirement: The given landing is sluice where the repository is set up for it

Where the file names no `[landing]`, the landing SHALL be sluice where
the repository's `HEAD` commit holds sluice's policy file,
`.config/sluice.toml`, read with git from that commit and never from the
working tree, and a plain branch otherwise, with no remote either way
(C.5, C.6). A repository git cannot read SHALL be answered unreadable
with git's words (58, 59).

#### Scenario: A repository sluice gates

- **WHEN** the repository's `HEAD` commit holds `.config/sluice.toml` and the file names no `[landing]`
- **THEN** a revision is landed when sluice's log for the branch holds its landing, and publish carries sluice's verdicts

#### Scenario: A repository with no sluice

- **WHEN** the repository's `HEAD` commit holds no `.config/sluice.toml` and the file names no `[landing]`
- **THEN** a revision is landed when the branch's history holds it, and publish attaches no verdicts (78)

#### Scenario: sluice's file only in the working tree

- **WHEN** `.config/sluice.toml` is in the working tree and not in the `HEAD` commit
- **THEN** the landing is a plain branch

#### Scenario: A repository with no commit

- **WHEN** the repository's `HEAD` names no commit yet
- **THEN** the landing is a plain branch, which answers every revision not landed

### Requirement: A laptop with nothing written publishes and verifies

The built binary, run on a host with no bindings file against a
repository sluice gates and moon builds, SHALL publish a landed revision
into the given layout with a host key it makes, and SHALL verify that
version once the repository's policy lists the key's public half as it
stands in `signing.pub` (S1, 80, 118).

#### Scenario: The first publish on a fresh laptop

- **WHEN** a host with a home and no bindings file, no layout and no key publishes a revision sluice landed
- **THEN** the given layout and the key are made, the answer names the version and its index, and each provenance is signed as the host's name

#### Scenario: Verify after listing the made key

- **WHEN** the repository's policy then lists the host's name with the line `signing.pub` holds, a later revision that changes the component lands and is published, and the built binary runs `tailrace verify <that version> --json` on the same host
- **THEN** every provenance is made by the host's name and counts under an acceptance policy requiring provenance from it, read at that version's revision

## MODIFIED Requirements

### Requirement: A missing or broken file is answered, never guessed

Where the host holds no bindings file, every command SHALL be bound to
the given profiles, as a file that names nothing is. A file that is not
well formed, names an unknown table or profile, or names more than one
profile for one seam, SHALL be answered unreadable naming the file, with
the parser's own words (58, 59, core.19), and SHALL offer, read or write
nothing: a file that is there is never guessed past.

#### Scenario: No file

- **WHEN** `tailrace verify <version>` runs on a host with no bindings file
- **THEN** it is bound to the given layout, the landing the repository is set up for, the git ref ledger and the host key verify-only, and no bindings file is written

#### Scenario: Two landings named

- **WHEN** the file's `[landing]` names both sluice and a branch
- **THEN** publish and verify exit 3 naming the file and the table

#### Scenario: Not well formed

- **WHEN** the file is not well formed TOML
- **THEN** publish and verify exit 3, printing the parser's words verbatim, and bind no given profile in its place

### Requirement: Only publish makes the layout

publish and push SHALL make the layout the host binds, the one the file
names or the given one, when its path is missing or an empty directory,
and write into it thereafter; push SHALL need no `[signing]`, since it
signs nothing. verify, status, sync and rebuild SHALL never make or
write a layout (60, core.32): verify over a layout that is not there
SHALL answer unreadable with the operating system's words.

#### Scenario: The first publish on a laptop

- **WHEN** the file names a layout path that does not exist and a landed revision is published
- **THEN** the layout is made there and holds the version's artifacts and index

#### Scenario: The first publish into the given layout

- **WHEN** the host has no bindings file and no given layout yet, and a landed revision is published
- **THEN** the layout is made at the given path, with its parent directories, and holds the version's artifacts and index

#### Scenario: verify over no layout

- **WHEN** verify runs while the layout path the file names does not exist
- **THEN** it exits 3 naming the layout, and no directory is made

#### Scenario: verify over no given layout

- **WHEN** verify runs on a host with no bindings file and no given layout yet
- **THEN** it exits 3 naming the given layout's path, and no directory is made

#### Scenario: The first push on a landing host

- **WHEN** the file names a layout path that does not exist and no `[signing]`, and a landed revision is pushed
- **THEN** the layout is made there and holds the landing's cache entries

#### Scenario: rebuild over no layout

- **WHEN** rebuild runs while the layout path the file names does not exist
- **THEN** it exits 3 naming the layout, and no directory is made

### Requirement: Every host binds every signer profile for verify

The binding SHALL bind both signer profiles on every host: the one
`[signing]` names first, signing as its signer, and the other verify-only,
so a claim made by either kind of signer is told on any host whatever it
signs with (80, 92). A host whose file names no `[signing]` SHALL bind
the given host key first, signing as the host's name, for publish and
promote, and both profiles verify-only for every other command.

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

### Requirement: The file names the ledger

The file SHALL bind the ledger for the host with `[ledger]`, naming its
profile by its one key (92, 118): `git_ref`, with an optional `remote`
holding the refs, the repository the command runs in holding them where
none is named (C.13); or `store`, holding the chain in the store the
host binds, through a binding of its own that never makes the store
(C.14). `[ledger]` naming no profile, two, or an option its profile does
not read SHALL be unreadable naming the table. A file that names no
`[ledger]` SHALL bind the git ref with no remote; the binding SHALL hold
no target.

#### Scenario: A git ref ledger with a remote

- **WHEN** the file names `[ledger] git_ref = { remote = "origin" }`
- **THEN** a promotion pushes its claim to `refs/tailrace/ledger/<channel>` on `origin`, and the file names no credential

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

- **WHEN** the file names no `[ledger]` and `tailrace promote` runs
- **THEN** the promotion is appended to `refs/tailrace/ledger/<channel>` in the repository the command runs in, and nothing is pushed to a remote
