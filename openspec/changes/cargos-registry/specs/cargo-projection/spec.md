## ADDED Requirements

### Requirement: A crate packed by Cargo's own packager

The Cargo projection SHALL pack an output given a version by setting the
version in the manifest that declares it for the output's package,
running `cargo package --no-verify --allow-dirty`, and answering the
`.crate` file's bytes with metadata naming the ecosystem `cargo`, the
package's name and the version (84, C.11). It SHALL answer `same_again`
true (87) and SHALL never open the archive to edit it.

#### Scenario: Packed the same again

- **WHEN** one crate output is packed twice at one version
- **THEN** both packs answer the same bytes and the same metadata, the conformance group's "packed the same again" passing

### Requirement: A pack leaves the checkout as it was and refuses what it cannot promise

The Cargo projection SHALL restore the edited manifest's bytes before
pack answers, whatever cargo answered. A crate whose own manifest names
another version than the one given SHALL be refused naming both. An
output with no `[package]` manifest among its files, or several, SHALL
be refused naming the output and its files. Cargo that cannot be run,
or that fails, SHALL answer unreadable with the operating system's or
cargo's own words (58, 59).

#### Scenario: The checkout is left as it was

- **WHEN** a crate whose version is inherited from its workspace is packed
- **THEN** the crate's own manifest names the version given, and the workspace's manifest holds its original bytes afterwards

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

### Requirement: A local registry rendered from the store

The Cargo projection SHALL render the artifacts given as a Cargo local
registry at the place given: each crate's bytes read through the store
seam it is handed, and an index line per version written from the
crate's own normalized manifest, ordered by Cargo's version rules (23,
85). The layout SHALL hold exactly the versions given, and rendering
the same artifacts again SHALL give the same layout (48). It SHALL
answer the versions it holds.

#### Scenario: A layout of the versions given

- **WHEN** two versions of a crate held in the store are rendered twice
- **THEN** the layout holds exactly those two versions, and both renders answer the same, the conformance group's "a layout of the versions given" passing

#### Scenario: A dependency's registry in the index

- **WHEN** a crate depending on a crate of its own registry, recorded by `registry-index`, and on a crate of crates.io is rendered
- **THEN** its index line names the first dependency's registry by that index URL and the second by crates.io's index URL

### Requirement: A render refuses what it cannot hold before writing

The Cargo projection SHALL check every artifact before render writes
anything, and SHALL refuse, naming it, an artifact of another ecosystem,
a manifest naming other than one layer, bytes of another digest than
the layer's, or a crate whose own manifest names another name or
version than its metadata (core.4, core.5).

#### Scenario: Bytes of another digest

- **WHEN** the store answers bytes other than the layer's digest
- **THEN** render is refused naming both digests and writes no file

### Requirement: Cargo builds from the layout

A consumer whose `.cargo/config.toml` replaces its family registry's
source with the rendered local registry SHALL build against a family
crate with `cargo build --offline`, and its lockfile SHALL name the
registry's declared address and no layout path or host (46, 49). Serve
SHALL answer absent, since Cargo reads the layout itself (86).

#### Scenario: cargo build resolves a family crate from the layout

- **WHEN** a crate is packed, put in a layout store, rendered, and a consumer depending on it through a registry replaced by the layout runs `cargo build --offline`
- **THEN** the build succeeds, and `Cargo.lock` records the crate's source as the declared registry address with the crate's checksum
