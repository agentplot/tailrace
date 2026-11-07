# nix-cache-target Specification

## Purpose
TBD - created by archiving change the-nix-cache-target. Update Purpose after archive.

## Requirements

### Requirement: The Nix cache target is a target profile of its own

The Nix cache target SHALL be a target profile under
`crates/timber/src/profiles/target/nix_cache/`, bound by the profile
name `nix_cache`, implementing the target interface and nothing of the
core (C.12, rules core.1, core.2). It SHALL read the version's index and
its artifacts through a store binding of its own, opened and never
made, and SHALL name attic, Nix and store paths in no source outside
`profiles/target/`.

#### Scenario: Bound by its profile name

- **WHEN** a repository's environment names `target.nix_cache = { attic = "fleet:swancloud", closures = ["swancloud-srv"] }`
- **THEN** apply of that environment reaches the Nix cache target, handed the environment with those settings

### Requirement: The target's table names the cache and the closures, and nothing else

The environment's `nix_cache` table SHALL name `attic`, the cache as
attic's own command names it, `<server>:<cache>`, and `closures`, one or
more output names whose artifacts name closures. Any other key, an
empty `closures`, or an `attic` that is not `<server>:<cache>` SHALL be
unreadable naming the environment and the key, and nothing SHALL be
pushed; so no credential sits in the table (45, 58, rule core.21).

#### Scenario: A token in the table

- **WHEN** the environment's `nix_cache` table names `attic`, `closures` and `token`
- **THEN** apply answers unreadable naming the environment and `token`, and attic is not run

#### Scenario: A cache with no server

- **WHEN** the table names `attic = "swancloud"`
- **THEN** apply answers unreadable naming `attic` and saying it is `<server>:<cache>`

### Requirement: A closure's artifact names its roots

A closure SHALL be named by the artifact of an output its component
declares, packed by the file projection as the output's one file: one
absolute store path per line, each a root of the closure, as `nix build
--print-out-paths` prints it (C.11, C.12). The store SHALL hold the
artifact and no store path of the closure; the paths are the cache's
(section 10).

#### Scenario: A machine's closure

- **WHEN** the artifact of `swancloud-srv` at `0.4.0` holds one line, the machine's top-level store path
- **THEN** the closure's roots are that one store path

### Requirement: The closure's form is read in one place

The closure's form SHALL be read by one module the Nix target profiles
share, `profiles/target/nix_closure.rs`, so no target profile calls
another (rule core.2). A file with no line, an empty or repeated line,
or a line that is no store path, `<store dir>/<32 characters of Nix's
base32>-<name>`, SHALL be refused naming the artifact and the line.

#### Scenario: A line that is no store path

- **WHEN** a closure's artifact holds a line `result`
- **THEN** reading it is refused naming the artifact and `result`, and nothing is pushed

### Requirement: desired is the channel's head

`desired` SHALL answer the head the core hands it, and absent naming the
channel where the core answers the channel empty (88, 31).

#### Scenario: The head handed

- **WHEN** desired is asked with a head that answers index `i3`
- **THEN** it answers `i3`

### Requirement: Apply reads each closure the version names by digest

`apply` SHALL read the verified version's index through the store by
digest, find for each output `closures` names the file projection's
artifact of that name, and read its bytes by digest (42, rule core.4);
an output the index carries no such artifact of SHALL be refused naming
the output and the version.

#### Scenario: A closure output the version lacks

- **WHEN** `closures` names `swancloud-srv` and the version's index carries no artifact of it
- **THEN** apply is refused naming `swancloud-srv` and the version, and attic is not run

### Requirement: Apply pushes only from the host that holds the closure

Before anything is pushed, every root SHALL be checked valid in this
host's Nix store; a root it does not hold SHALL refuse the apply naming
the root and the host, and nothing SHALL be pushed. Apply SHALL never
build, substitute or copy a root into this host's store (15, I3).

#### Scenario: A root this host does not hold

- **WHEN** the closure's root is not valid in this host's Nix store
- **THEN** apply is refused naming the root and the host, attic is not run, and nothing is built or fetched

### Requirement: Apply pushes each root with its closure through attic

Once every root is held, apply SHALL run `attic push <server>:<cache>`
with every root, which pushes each root with its closure; attic's own
words on a failure SHALL be the answer, unreadable, verbatim (59).

#### Scenario: Pushed with its closure

- **WHEN** the root is valid in this host's store and refers to one other store path
- **THEN** after the apply the cache holds both store paths

### Requirement: The push uses the environment's own credential

The push SHALL be run with the credential attic reads from its own
configuration on the host, at the push, the environment's own place
(45). timber SHALL pass no token on attic's command line or in its
environment, SHALL read nothing of attic's configuration, and SHALL keep
nothing of it (rule core.21).

#### Scenario: The token stays attic's

- **WHEN** apply pushes to a cache whose token is held only in attic's own configuration
- **THEN** the push succeeds, and neither attic's command line, its environment as timber set it, nor anything timber wrote holds the token

### Requirement: current and report are unread, since a cache keeps no marker

A Nix binary cache holds store paths by their hash and keeps no name a
marker could be held under, so `current` and `report` SHALL answer
unreadable, in words that say so and name the cache, never a digest and
never converged (119, 89, 98).

#### Scenario: Current after an apply

- **WHEN** a version is applied and current is asked
- **THEN** current answers unreadable saying the cache keeps no marker of what was pushed, not the digest applied

### Requirement: The profile passes the target group

The profile SHALL pass the target group with a rig declaring that its
target holds no marker, its foreign settings a `token` key (93).

#### Scenario: The group over the profile

- **WHEN** the target group runs over the Nix cache target's rig
- **THEN** every scenario passes, and "converged and reported" is reported not applicable

### Requirement: A closure built on one host is realised on another from the cache alone

A tools-layer test SHALL push, through the profile, a closure of two
store paths built by the test's host and named by a published version's
artifact, with the token in attic's own configuration alone, to an attic
server started for the test; then a fresh Nix store, trusting only the
cache's key, SHALL realise the root and its leaf from that cache alone
(C.12).

#### Scenario: Realised from the cache alone

- **WHEN** the version is applied to the cache environment from the host that built it
- **THEN** a fresh Nix store with the cache as its only source realises the root, and holds the leaf it refers to
