## ADDED Requirements

### Requirement: A crate packed by Cargo's own packager

The Cargo projection SHALL pack an output given a version from the one
crate file the output's packaging task left: the package that crate
names, at the place it was packed from, SHALL be packed by `cargo
package --no-verify` after the version is set on it and on every crate of
its workspace it reaches by path dependency, each such dependency
rewritten to that version's requirement in the registry `tailrace`, so a
path dependency on a sibling crate becomes a dependency of the registry
as Cargo does for any publish (84, 110, C.11). It SHALL answer the
package's crate with metadata naming the ecosystem `cargo`, the package's
name and the version, SHALL answer `same_again` true (87), and SHALL
never open the archive to edit it.

#### Scenario: Packed the same again

- **WHEN** one crate output is packed twice at one version
- **THEN** both packs answer the same bytes and the same metadata, the conformance group's "packed the same again" passing

#### Scenario: A sibling becomes a registry dependency

- **WHEN** a crate depending by path on a sibling crate of its workspace is packed at a version
- **THEN** the crate's own manifest names the sibling at that version's requirement in the registry the checkout's cargo configuration names `tailrace`, and no path

### Requirement: A pack leaves the checkout as it was and refuses what it cannot promise

The Cargo projection SHALL restore every manifest it edited to its own
bytes before pack answers, whatever cargo answered. A crate whose own
manifest names another name or version than the one given SHALL be
refused naming both. An output with no crate file among its files, or
several, SHALL be refused naming the output and its files, and a crate
file that names no place in the repository SHALL be refused naming it.
Cargo that cannot be run, or that fails, SHALL answer unreadable with the
operating system's or cargo's own words (58, 59).

#### Scenario: The checkout is left as it was

- **WHEN** a crate whose version is inherited from its workspace, and which depends by path on a sibling, is packed
- **THEN** the crate's own manifest names the version given, and the workspace's and both crates' manifests hold their original bytes afterwards

#### Scenario: Cargo cannot be reached

- **WHEN** the projection is bound to a cargo that does not exist
- **THEN** pack answers unreadable carrying the operating system's words, the conformance group's "a projection that cannot be reached" passing

### Requirement: No cache entries

The Cargo projection SHALL answer absent naming cache entries for a pack
with no version and a render of cache entries, reading and writing
nothing (84, 85).

#### Scenario: A projection that packs no cache entries

- **WHEN** pack is asked with no version, or render is asked for cache entries
- **THEN** each answers absent naming cache entries and nothing is written

### Requirement: The pins a Cargo lockfile holds

The Cargo projection SHALL answer pinned for a checkout with the name
and version of each package its root `Cargo.lock` pins from a registry,
as the lockfile spells them, reading the file alone and running no
cargo (137). A package with no source, or from git, SHALL be left out.
A checkout with no `Cargo.lock` SHALL answer absent naming it, and a
lockfile that cannot be read as one SHALL be refused naming the file
with the parser's own words (58, 59).

#### Scenario: A lockfile's registry pins

- **WHEN** a checkout's `Cargo.lock` holds a workspace member, a crate from a sparse registry, a crate from crates.io and a crate from git
- **THEN** pinned answers the sparse registry's crate and crates.io's crate by name and version as written, and neither the member nor the git crate

#### Scenario: No lockfile

- **WHEN** pinned is asked of a checkout with no `Cargo.lock`
- **THEN** it answers absent naming `Cargo.lock`, and nothing is written

### Requirement: A sparse registry rendered from the store

The Cargo projection SHALL render the artifacts given as the files of a
Cargo sparse registry at the place given: each crate's bytes read
through the store seam it is handed, and an index file per name holding
a line per version written from the crate's own normalized manifest,
ordered by Cargo's version rules (23, 85). A dependency on a crate of
any family repository's served Cargo registry SHALL be written as one of
the registry serving the line, so a consumer resolves it from its own
served address (49, 138). The layout SHALL hold exactly the versions
given, SHALL name no address, and rendering the same artifacts again
SHALL give the same layout (48). It SHALL answer the versions it holds.

#### Scenario: A layout of the versions given

- **WHEN** two versions of a crate held in the store are rendered twice
- **THEN** the layout holds exactly those two versions, and both renders answer the same, the conformance group's "a layout of the versions given" passing

#### Scenario: A dependency's registry in the index

- **WHEN** a crate depending on a crate at a family repository's served Cargo address, on a crate of another registry, and on a crate of crates.io is rendered
- **THEN** its index line names the first dependency as of the registry serving the line, the second by its index URL, and the third by crates.io's index URL

### Requirement: A render refuses what it cannot hold before writing

The Cargo projection SHALL check every artifact before render writes
anything, and SHALL refuse, naming it, an artifact of another ecosystem,
a manifest naming other than one layer, bytes of another digest than
the layer's, or a crate whose own manifest names another name or
version than its metadata (core.4, core.5).

#### Scenario: Bytes of another digest

- **WHEN** the store answers bytes other than the layer's digest
- **THEN** render is refused naming both digests and writes no file

### Requirement: Cargo's sparse protocol served from the layout

The Cargo projection SHALL serve a rendered layout over Cargo's sparse
registry protocol under the base the client reached (86, 114): the
registry's configuration naming where its crates download from under
that base, each name's index file from the layout, and each crate's
bytes. A crate the layout lacks SHALL be asked of the core once and
answered once the core renders it (115, 50). A crate the core answers
absent, refused or unreadable SHALL be answered with a not-found, a
forbidden or a bad-gateway status, the response carrying the answer's
own words verbatim (58, 59, 99). Every request SHALL be answered with a
response, so the projection is read as serving.

#### Scenario: A crate held, served

- **WHEN** a layout holding two crates is asked for one crate's download, for its name's index file, and for the registry's configuration under a base
- **THEN** the download answers the crate's bytes, the index file answers the layout's lines, the configuration names downloads under that base, and the core was asked nothing

#### Scenario: A crate lacked and refused

- **WHEN** a layout is asked for a crate it lacks and the core answers it refused with the words `acceptance wanting`
- **THEN** the core was asked once for that crate's name and version, and the response is a forbidden status carrying `acceptance wanting` verbatim

### Requirement: Cargo builds from the host's serving process

A consumer whose cargo configuration names the registry `tailrace` at
its served address, `sparse+http://127.0.0.1:7420/<owner>/<name>/cargo/`,
the same on every host (49, 114), and whose `Cargo.lock` pins family
crates, each resolved through the store's name for it to the version's
index and the crate found in that index by its metadata (138), SHALL
build against them with `cargo build --locked` once `tailrace sync` has
rendered its layout, the host's one process serving it. Its lockfile
SHALL name that address and the checksum of each crate packed, and no
layout path or store (46, 49).

#### Scenario: cargo build resolves family crates from the served registry

- **WHEN** a publisher's crate and the sibling it depends on by path are packed and named in the store, and a consumer's `Cargo.lock` pinning both at the served address is synced
- **THEN** `cargo build --locked` succeeds, reading both crates from the host's serving process, and `Cargo.lock` records that address and each crate's checksum as written
