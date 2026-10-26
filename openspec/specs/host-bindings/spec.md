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
each with an optional remote (C.5, C.6); and `[signing]` naming the
host's signer and the path of its key file, in sluice's `[signing]`
shape (C.8). A path SHALL be absolute or begin with `~/`.

#### Scenario: A layout, sluice and a host key

- **WHEN** the file names `[store] layout`, `[landing] sluice` with remote `origin`, and `[signing]` with a signer and a key path beginning `~/`
- **THEN** publish writes into that layout, reads sluice's records from `origin`, and signs as that signer with the key under the home directory

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

### Requirement: The ledger is not yet bound

The file SHALL name no ledger; the binding SHALL hold a ledger that
answers absent naming the ledger's binding for every operation, and no
target (92).

#### Scenario: A ledger table

- **WHEN** the bindings file holds a `[ledger]` table
- **THEN** publish and verify answer unreadable naming the table

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

### Requirement: The host trusts its own signer

For reading, the host SHALL trust its own signer, the one `[signing]`
names, verified by its key's public half, the file beside the key that
ssh-keygen names `<key>.pub`, read when sync binds and never the key
itself, so a host consumes what it published itself (118, C.10). The
binding SHALL carry the trusted signers to sync alone, the host's own
before those `[trust]` names; a file naming no `[signing]` and no
`[trust]` SHALL trust no signer.

#### Scenario: The host's own signer trusted

- **WHEN** the file names `[signing]` for the signer `laptop` with a key whose public half lies beside it, and `[trust.gate]`
- **THEN** a sync's binding trusts `laptop`, verified by that public half, then `gate`, neither with a claim of its own, and the key itself was not read

#### Scenario: No trust named

- **WHEN** the file names `[store]`, `[landing]` and `[signing]` and no `[trust]`
- **THEN** a sync's binding trusts the host's own signer alone, and publish and verify answer as before

#### Scenario: Neither signing nor trust named

- **WHEN** the file names `[store]` and `[landing]`, no `[signing]` and no `[trust]`
- **THEN** a sync's binding trusts no signer

### Requirement: The file names the other signers the host trusts

The file SHALL name the other signers the host trusts, one
`[trust.<signer>]` table each, holding exactly one key whose string
value is the text the signer's profile verifies it by, in the policy
file's `[signers]` form and with no `kinds` or `checks` (section 2,
118, C.10), and never a credential (64, core.21). No trusted signer
SHALL carry a claim of its own to make: its reader allows it the one
claim it reads through it (C.10).

#### Scenario: A signer trusted

- **WHEN** the file names `[trust.gate]` with `ssh = "ssh-ed25519 AAAA… gate@box"`
- **THEN** the binding trusts `gate`, verified by that key, with no claim of its own

### Requirement: A broken trust table is unreadable
A trust table naming no key, two keys, or a value that is not a string
SHALL be unreadable naming the file and the table; a public half beside
the `[signing]` key that is missing or holds no OpenSSH public key line
SHALL be unreadable naming its file, in the operating system's or the
parser's words; and sync SHALL then read and write nothing (58, 59,
core.19).

#### Scenario: A trust table with two keys

- **WHEN** `[trust.gate]` names both `ssh` and `kms`
- **THEN** sync answers unreadable naming the file and `[trust.gate]`, and nothing is read or written

#### Scenario: No public half beside the signing key

- **WHEN** `[signing]` names a key with no `<key>.pub` beside it, and sync runs
- **THEN** sync answers unreadable naming the public half's file, and nothing is read or written
