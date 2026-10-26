# host-bindings Specification

## Purpose
The operator's bindings for a host: one file beside the landing's own
that names the store, the landing and the signer, and the binding
`tailrace publish` and `tailrace verify` are bound to from it, so a
person on a laptop publishes with the installed binary and a second
machine verifies what it published.

## Requirements

### Requirement: The host's bindings are one file beside the landing's own

The operator's bindings for a host SHALL be kept in one TOML file at
`$XDG_CONFIG_HOME/tailrace/config.toml`, or `~/.config/tailrace/config.toml`
where that variable is unset or empty, beside sluice's own
`config.toml` under the same directory and in the same form, readable
with a file reader and with no tailrace process running (118, 62).

#### Scenario: The file beside sluice's

- **WHEN** `XDG_CONFIG_HOME` names a directory holding `sluice/config.toml` and `tailrace/config.toml`
- **THEN** `tailrace publish` binds the seams the second names, and both are TOML files a plain file reader opens

#### Scenario: No variable set

- **WHEN** `XDG_CONFIG_HOME` is unset and `~/.config/tailrace/config.toml` names a layout, a landing and a signer
- **THEN** `tailrace verify` is bound to what that file names

### Requirement: The file names the store, the landing and the signer

The file SHALL bind the store, the landing and the signer for the host,
and nothing the repository's authors bind (92): `[store]` naming a
layout by its path (C.2) or a registry by its repository (C.1);
`[landing]` naming sluice or a plain branch,
each with an optional remote (C.5, C.6); and `[signing]` naming either
the host's signer and the path of its key file, in sluice's `[signing]`
shape (C.8), or a key in a key service by its key ARN alone, `kms =
"<key ARN>"`, whose ARN is the signer's name (C.7). A path SHALL be
absolute or begin with `~/`.

#### Scenario: A layout, sluice and a host key

- **WHEN** the file names `[store] layout`, `[landing] sluice` with remote `origin`, and `[signing]` with a signer and a key path beginning `~/`
- **THEN** publish writes into that layout, reads sluice's records from `origin`, and signs as that signer with the key under the home directory

#### Scenario: A key in a key service

- **WHEN** the file's `[signing]` names `kms` with a KMS key ARN and nothing else
- **THEN** publish signs as that ARN through the key service signer, and the file names no credential

#### Scenario: A key service key beside a host key

- **WHEN** the file's `[signing]` names `kms` together with `signer` or `key`
- **THEN** publish and verify answer unreadable, naming the file and `[signing]`, and saying a key service signer's name is its key's and the table names `kms` alone

#### Scenario: A registry

- **WHEN** the file names `[store] registry` with a repository such as `localhost:5000/tailrace`
- **THEN** publish writes into that repository and verify reads from it, neither makes a repository, and the file names no credential

#### Scenario: A registry named by something that is not a repository

- **WHEN** `[store] registry` names a value that is not a registry host followed by a repository path
- **THEN** publish and verify answer unreadable, naming the file, `[store] registry` and the value, and nothing is read or written

#### Scenario: A plain branch

- **WHEN** the file names `[landing] branch`
- **THEN** a revision is landed when the branch's history holds it, and publish attaches no verdicts (78, C.6)

#### Scenario: A relative path

- **WHEN** the file names a layout or a key by a path that is neither absolute nor begins with `~/`
- **THEN** publish and verify answer unreadable, naming the file and the key, and nothing is read or written

### Requirement: A missing or broken file is answered, never guessed

Where the host holds no bindings file, publish and verify SHALL answer
absent naming the file's path. A file that is not well formed, names an
unknown table or profile, or names more than one profile for one seam,
SHALL be answered unreadable naming the file, with the parser's own
words (58, 59, core.19). Neither SHALL offer, read or write anything.

#### Scenario: No file

- **WHEN** `tailrace publish HEAD` runs on a host with no bindings file
- **THEN** it exits 4 naming the file's path, and no tag, file or record is written

#### Scenario: Two landings named

- **WHEN** the file's `[landing]` names both sluice and a branch
- **THEN** publish and verify exit 3 naming the file and the table

#### Scenario: Not well formed

- **WHEN** the file is not well formed TOML
- **THEN** publish and verify exit 3, printing the parser's words verbatim

### Requirement: The repository binds the build and the projections

The binding SHALL take the build and the projections from the
repository, not the host (92): moon is bound as the build, its tasks
named per output in the policy file at the landed revision (C.3), and
the file and image projections are bound by their ecosystem names, the
policy file naming each output's (C.10, C.11). The policy seam SHALL be
bound to the policy file at the landed revision (C.10).

#### Scenario: The bindings file names no build

- **WHEN** the bindings file names a store, a landing and a signer and nothing else
- **THEN** publish asks moon to ensure the outputs the policy file declares, and packs each with the projection the policy file names for it

#### Scenario: A build named in the host's file

- **WHEN** the bindings file holds a `[build]` table
- **THEN** publish and verify answer unreadable naming the table, since the build is the repository's to bind

### Requirement: Only publish makes the layout

publish and push SHALL make the layout the file names when its path is
missing or an empty directory, and write into it thereafter; push SHALL
need no `[signing]`, since it signs nothing. verify, status, sync and
rebuild SHALL never make or write a layout (60, core.32): verify over a
layout that is not there SHALL answer unreadable with the operating
system's words.

#### Scenario: The first publish on a laptop

- **WHEN** the file names a layout path that does not exist and a landed revision is published
- **THEN** the layout is made there and holds the version's artifacts and index

#### Scenario: verify over no layout

- **WHEN** verify runs while the layout path the file names does not exist
- **THEN** it exits 3 naming the layout, and no directory is made

#### Scenario: The first push on a landing host

- **WHEN** the file names a layout path that does not exist and no `[signing]`, and a landed revision is pushed
- **THEN** the layout is made there and holds the landing's cache entries

#### Scenario: rebuild over no layout

- **WHEN** rebuild runs while the layout path the file names does not exist
- **THEN** it exits 3 naming the layout, and no directory is made

### Requirement: A laptop publishes a sluice landing from the installed binary

The built binary, bound by a bindings file naming a layout, sluice and a
host key, SHALL play S1 and S2 against a repository whose landings
sluice makes and whose builds moon runs, declaring two outputs, one
packed as a file and one as an image (S1, S2, 14 to 20).

#### Scenario: S1 from the binary

- **WHEN** sluice lands a revision and the built binary runs `tailrace publish <revision> --json`
- **THEN** the answer names the version the history derives, the index's digest, both artifacts' digests and ecosystem names, the provenance on each signed by the host's key and sluice's verdicts carried in it; the layout holds no name but the version's, so no channel moved

#### Scenario: S2 from the binary

- **WHEN** the built binary publishes the same revision again
- **THEN** it answers the same index, and no file of the layout and no file of the host's record changed

### Requirement: A second machine verifies a copied layout

A machine that never published, holding a clone of the repository, a
copy of the layout and a bindings file naming the copy and a landing
but no key, SHALL verify a version published elsewhere: each provenance
told as made by the publishing host's listed key, and counted where the
policy lets that signer claim it (80, 9, 61).

#### Scenario: verify from a copy

- **WHEN** the layout is copied to another directory, the repository cloned beside it, and the built binary runs `tailrace verify <version> --json` there under a bindings file naming the copy, sluice with the clone's remote, and no `[signing]` table
- **THEN** every provenance on the index's artifacts is made by the publishing host and counts under an acceptance policy requiring provenance from it, and each of sluice's verdicts is reported as carried in the provenance that holds it

### Requirement: Every host binds every signer profile for verify

The binding SHALL bind both signer profiles on every host: the one
`[signing]` names first, signing as its signer, and the other verify-only,
so a claim made by either kind of signer is told on any host whatever it
signs with (80, 92). A host whose file names no `[signing]` SHALL bind
both verify-only.

#### Scenario: A host key host

- **WHEN** a host whose `[signing]` names a host key is bound for verify
- **THEN** the signers bound are the host key, signing as its signer, and then the key service signer, verify-only

#### Scenario: A key service host

- **WHEN** a host whose `[signing]` names a KMS key ARN is bound for publish
- **THEN** the binding's signer is that ARN, and the signers bound are the key service signer bound to that key, and then the host key, verify-only

#### Scenario: A host that signs nothing

- **WHEN** a host whose file names no `[signing]` is bound for verify
- **THEN** both signer profiles are bound verify-only, and nothing is read or reached in binding them

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

### Requirement: The file names what tells the gate

The bindings file SHALL name what tells the host's gate of an offered
version in a `[gate]` table, by one key, in the form every other table
of the file takes (118, 130, C.16): `publish = {}`, the host's own
publish step tells the host's own gate; or `job = "<words>"`, a job
outside tailrace, woken by a store's or a forge's event, tells a gate
elsewhere, and the words say which. The core SHALL name neither (130).
tailrace SHALL run nothing for `job`, and SHALL carry its words as
written. A `[gate]` naming no key, two keys, a key other than these, a
`publish` that is not an empty table, or a `job` that is not a string
with words in it SHALL be unreadable naming the file and the table, and
SHALL leave nothing written for every command that reads the file (58,
59, core.19).

#### Scenario: The host's publish tells its own gate

- **WHEN** the bindings file holds `[gate]` with `publish = {}`
- **THEN** the binding says the host's publish step tells the host's gate

#### Scenario: A job tells a separated gate

- **WHEN** the bindings file holds `[gate]` with `job = "the release job a push to main wakes"`
- **THEN** the binding says a job tells the gate, carrying those words as written, and nothing of tailrace starts or reaches that job

#### Scenario: Two tellers

- **WHEN** `[gate]` holds both `publish = {}` and `job = "…"`
- **THEN** publish and verify answer unreadable naming `[gate]` and both keys, and the layout the file names is not made

#### Scenario: A teller the file does not know

- **WHEN** `[gate]` holds `webhook = "…"`
- **THEN** the answer is unreadable naming `[gate]`, `webhook`, and the keys the table reads, `publish` and `job`

#### Scenario: A publish teller with options

- **WHEN** `[gate]` holds `publish = { now = true }`
- **THEN** the answer is unreadable naming `[gate] publish`, which takes no options

#### Scenario: A job with no words

- **WHEN** `[gate]` holds `job = ""`
- **THEN** the answer is unreadable naming `[gate] job`

### Requirement: A file naming no teller binds the host's own publish to tell its gate

A bindings file with no `[gate]` table SHALL bind the host's default
teller: the host's own publish step tells the host's own gate, as
`[gate] publish = {}` does (118, 130). Every command SHALL bind
otherwise as it does without one.

#### Scenario: No gate table

- **WHEN** the bindings file names a store, a landing and a signer and holds no `[gate]`
- **THEN** the binding binds them as before and says the host's publish step tells the host's gate, the same binding `[gate] publish = {}` makes
