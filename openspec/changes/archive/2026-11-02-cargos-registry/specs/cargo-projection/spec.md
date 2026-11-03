## ADDED Requirements

### Requirement: A crate packed by Cargo's own packager

The Cargo projection SHALL pack an output given a version from the one
crate file the output's packaging task left: the package that crate
names, at the place it was packed from, SHALL be packed by `cargo
package --no-verify` without the crate's own lockfile, after the version
is set on it and on every crate of its own component it reaches by path
dependency, each such dependency rewritten to that version's requirement
in the registry `tailrace`, so a path dependency on a sibling crate
becomes a dependency of the registry as Cargo does for any publish (84,
110, C.11). A crate SHALL be of the component whose paths hold its
manifest, as the core matches them (123). A sibling that none of the
component's declared outputs packs SHALL be refused naming it and the
component, since a consumer could resolve nothing for it (C.11, 84). It SHALL answer the package's crate with metadata naming
the ecosystem `cargo`, the package's name and the version, and
`same_again` true, and SHALL never open the archive to edit it (87).

#### Scenario: Packed the same again

- **WHEN** one crate output is packed twice at one version
- **THEN** both packs answer the same bytes and the same metadata, and `same_again` true, the conformance group's "packed the same again" passing

#### Scenario: A sibling becomes a registry dependency

- **WHEN** a crate depending by path on a crate of its own component is packed at a version
- **THEN** the crate's own manifest names the sibling at that version's requirement in the registry the checkout's cargo configuration names `tailrace`, and no path, and the crate holds no lockfile

#### Scenario: A sibling no output publishes

- **WHEN** a crate depending by path on a crate of its own component is packed, and none of the component's outputs packs that crate
- **THEN** the pack is refused naming the sibling and the component, and no cargo package ran

### Requirement: A crate of another component is depended on as the manifest writes

A path dependency on a crate of another component SHALL be packed as a
dependency on the version requirement the manifest writes for it, in the
registry `tailrace`, with no version of it picked: the packed crate SHALL
name the requirement alone, and that crate's manifest SHALL never be
given this component's version (C.11, 123, 124, 137). Where the store's
listing of that crate's versions holds none within the requirement, the
empty listing of a name the store never held among them, the pack SHALL
be refused naming the crate, the requirement and the versions the store
holds, or that it holds none; a path dependency on a crate of another component that writes no
version requirement SHALL be refused naming the crate (C.11, 138).

#### Scenario: Another component's crate the store holds within the requirement

- **WHEN** a crate depending by path on a crate of another component with the requirement `0.3` is packed, and the store holds that crate at 0.3.1
- **THEN** the packed crate's manifest names the dependency at `0.3` in the registry `tailrace` and no path, the other crate's manifest holds its bytes, and the answer's `same_again` is true

#### Scenario: The same bytes whatever the store holds within the requirement

- **WHEN** the same crate is packed again at the same version once the store also holds that crate at 0.3.2
- **THEN** the pack answers the same bytes as before

#### Scenario: Another component's crate the store holds no version of

- **WHEN** the same crate is packed and the store holds that crate only at 0.4.0
- **THEN** the pack is refused naming the crate, the requirement `0.3` and the version 0.4.0, and no cargo package ran

#### Scenario: Another component's crate the store names at no version

- **WHEN** the same crate is packed and the store's listing of that crate answers the empty list
- **THEN** the pack is refused naming the crate, the requirement `0.3` and that the store holds no version of it, and no cargo package ran

### Requirement: A pack leaves the checkout as it was and refuses what it cannot promise

The Cargo projection SHALL restore every manifest it edited to its own
bytes before pack answers, whatever cargo answered. A crate whose own
manifest names another name or version than the one given SHALL be
refused naming both. An output with no crate file among its files, or
several, SHALL be refused naming the output and its files, and a crate
file that names no place in the repository SHALL be refused naming it.
Cargo that cannot be run SHALL answer unreadable with the operating
system's own words, and cargo that fails, on a checkout whose cargo
configuration names no registry `tailrace` among others, unreadable with
cargo's own words (58, 59).

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

### Requirement: The pins a Cargo lockfile holds and the ranges its manifests declare

The Cargo projection SHALL answer pinned for a checkout with the name
and version of each package its root `Cargo.lock` pins from a registry,
as the lockfile spells them, and, for each crate a workspace member's
manifest declares in the registry `tailrace`, by its own table or
inherited from the workspace's, that no lockfile pins at a version from
a registry, the requirement the manifest writes as its range, as
written, `*` where it writes none (137). A checkout with a root
`Cargo.toml` and no `Cargo.lock` SHALL answer its manifests' ranges, so
a first install needs no lockfile written by hand. A package with no
source, or from git, and a dependency with a path or a git source, SHALL
be left out. It SHALL read the files alone, run no cargo and write
nothing. A checkout with no `Cargo.lock` whose manifests declare nothing
in `tailrace`, or that has no root `Cargo.toml`, SHALL answer absent
naming `Cargo.lock`, and a lockfile or manifest that cannot be read as
one SHALL be refused naming the file with the parser's own words (58,
59).

#### Scenario: A lockfile's registry pins

- **WHEN** a checkout's `Cargo.lock` holds a workspace member, a crate from a sparse registry, a crate from crates.io and a crate from git
- **THEN** pinned answers the sparse registry's crate and crates.io's crate by name and version as written, and neither the member nor the git crate

#### Scenario: A crate the manifest declares and no lockfile pins

- **WHEN** a workspace's member declares `fam-b = { version = "1", registry = "tailrace" }`, another member inherits `fam-c` from `workspace.dependencies` where it is written `{ version = "0.3", registry = "tailrace" }`, a third declares `fam-d` in `tailrace` with a `path`, and the `Cargo.lock` pins none of them
- **THEN** pinned answers `fam-b` by the range `1` and `fam-c` by the range `0.3`, and nothing of `fam-d`

#### Scenario: A declared crate the lockfile pins

- **WHEN** a member declares `fam-b` in `tailrace` by `1` and the `Cargo.lock` pins `fam-b` at 1.1.0 from the served address
- **THEN** pinned answers `fam-b` at 1.1.0, and no range of it

#### Scenario: A manifest and no lockfile

- **WHEN** pinned is asked of a checkout with no `Cargo.lock` whose manifest declares `fam-a = { version = "1", registry = "tailrace" }`
- **THEN** it answers `fam-a` by the range `1`, and nothing is written

#### Scenario: No lockfile and nothing of the family

- **WHEN** pinned is asked of a checkout with no `Cargo.lock` whose manifest declares nothing in `tailrace`, or with no `Cargo.toml`
- **THEN** it answers absent naming `Cargo.lock`, and nothing is written

### Requirement: What a crate requires of the family

The Cargo projection SHALL answer requires for a held crate with each
dependency its own normalized manifest names whose registry is a family
repository's served Cargo address, in its normal or build dependencies
or a target's of them, optional or not, by the crate's name and the
requirement the manifest writes, as written, each name and range once;
a dev dependency, a dependency of crates.io and one of another registry
SHALL never be answered (140, 49). It SHALL read the crate through the
store seam it is handed, checked against its digest, and a crate that
cannot be read SHALL be unreadable naming its digest.

#### Scenario: A crate's family dependencies

- **WHEN** requires is asked of a crate depending at the served address on `fam-b` by `1.1.0`, optionally on `fam-c` by `0.3`, on `fam-d` for one target by `2`, on `fam-e` as a dev dependency, on a crate of another registry and on a crate of crates.io
- **THEN** it answers `fam-b` by `1.1.0`, `fam-c` by `0.3` and `fam-d` by `2`, and nothing else

#### Scenario: A crate that requires nothing of the family

- **WHEN** requires is asked of a crate whose only dependencies are of crates.io
- **THEN** it answers none

### Requirement: The versions a range admits, by Cargo's rules

The Cargo projection SHALL answer satisfying, given a range and
versions, with those versions that satisfy the range by Cargo's own
requirement rules, newest first by Cargo's version order, a version
Cargo cannot read satisfying no range (140, 23). A range Cargo's rules
cannot read SHALL be refused naming the range and the parser's own
words.

#### Scenario: The versions a range admits

- **WHEN** the projection is asked which of 1.2.0, 2.0.0, 1.5.0-rc.1, 1.4.0 and `latest` satisfy the range `1`
- **THEN** it answers 1.4.0 then 1.2.0, and no other

#### Scenario: Newest by Cargo's order

- **WHEN** the projection is asked which of 1.9.0, 1.10.0, 1.0.0 and 1.0.0-rc.1 satisfy the range `>=1.0.0-rc.1, <2`
- **THEN** it answers 1.10.0, 1.9.0, 1.0.0 and 1.0.0-rc.1, in that order

#### Scenario: A range Cargo cannot read

- **WHEN** the projection is asked which of 1.0.0 satisfy the range `not-a-range`
- **THEN** it is refused naming `not-a-range` and the parser's words

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
answered once the core renders it (115, 50); an index file SHALL be
answered from the layout alone. A crate the core answers absent, refused
or unreadable SHALL be answered with a not-found, a forbidden or a
bad-gateway status, the response carrying the answer's own words
verbatim (58, 59, 99). Every request SHALL be answered with a response,
so the projection is read as serving, in the one shape of request,
response and miss the host's process serves every ecosystem in.

#### Scenario: A crate held, served

- **WHEN** a layout holding two crates is asked for one crate's download, for its name's index file, and for the registry's configuration under a base
- **THEN** the download answers the crate's bytes, the index file answers the layout's lines, the configuration names downloads under that base, and the core was asked nothing

#### Scenario: A crate lacked and refused

- **WHEN** a layout is asked for a crate it lacks and the core answers it refused with the words `acceptance wanting`
- **THEN** the core was asked once for that crate's name and version, and the response is a forbidden status carrying `acceptance wanting` verbatim

### Requirement: A refused version reaches Cargo three ways

A version the layout's policy refuses SHALL be left out of the index
file Cargo lists for its name, a request for its bytes SHALL be answered
with the refusal naming the version and the claim found wanting, and the
sync that rendered the layout SHALL have answered the same refusal, so
Cargo's own words are never the only ones (99).

#### Scenario: A newer version within a range the policy refuses

- **WHEN** a consumer's manifest declares a family crate by a range within which the store names an admitted version and a newer one the consumer's policy refuses, and the consumer syncs and builds
- **THEN** sync's answer names the newer version and the claim found wanting, the name's index file served lists only the admitted version, a request for the newer version's download answers a forbidden status carrying the same words, and the build resolves the admitted version

### Requirement: Cargo builds from the host's serving process with no lockfile written by hand

A consumer whose cargo configuration names the registry `tailrace` at
its served address, `sparse+http://127.0.0.1:7420/<owner>/<name>/cargo/`,
the same on every host (49, 114), and whose manifest declares a family
crate by a requirement, SHALL build against it with `cargo build` once
`tailrace sync` has rendered its layout, the host's one process serving
it, with no `Cargo.lock` before the first sync: the layout SHALL hold the
newest admitted version within the requirement and the family crates
that version requires, through every level, so Cargo's resolution
closes within the layout and Cargo writes the lockfile (137, 140). That
lockfile SHALL name the served address and the checksum of each crate
packed, and no layout path or store (46, 49), and a second sync and
`cargo build --locked` SHALL read it unchanged.

#### Scenario: A first install with no lockfile

- **WHEN** a publisher's crate `fam-a`, depending by path on its sibling `fam-b`, is packed with it at 1.0.0, 1.1.0 and 2.0.0 and named in the store, and a consumer with no `Cargo.lock` declaring `fam-a` in `tailrace` by `1` syncs and runs `cargo build`
- **THEN** sync's answer names the range `fam-a` `1` holding 1.1.0 and the range of `fam-b` that `fam-a` 1.1.0 requires holding 1.1.0, the build succeeds reading both from the host's serving process, and Cargo writes a `Cargo.lock` pinning both at 1.1.0 from the served address with their packed checksums and no path

#### Scenario: The lockfile Cargo wrote, read locked

- **WHEN** that consumer syncs again and runs `cargo build --locked`
- **THEN** sync answers both as pins held, the build succeeds, and the `Cargo.lock` is as Cargo wrote it
