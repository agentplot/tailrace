## Why

A family repository's `cargo build` cannot yet take a family crate from
anything tailrace holds: no projection packs a crate or renders a layout
Cargo reads. Bolt 5's Cargo unit closes that gap
(`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 5,
"Cargo's registry"), so a crate published by one family repository is
built against by another from a layout rendered from the store, with
Cargo's own configuration naming it and the lockfile naming no host.

## What Changes

- A Cargo projection, bound under the ecosystem `cargo`, under
  `crates/tailrace/src/profiles/projection/cargo/` (C.11).
- **pack** (84, 87): the crate's manifest is edited to carry the version,
  `cargo package --no-verify --allow-dirty` packs it into a scratch
  target directory, the edit is undone, and the `.crate` file's bytes are
  answered with metadata naming `cargo`, the package's name and the
  version. Cargo writes fixed times and owners into the archive, so the
  same output and version pack to the same bytes and the projection
  promises so; a crate whose own manifest does not carry the version is
  refused. It packs no cache entries.
- **render** (85, 48): a local registry at the place given, holding
  exactly the crates given: each `.crate` file read through the store
  seam and checked against its digest, and an index line per version
  written from the crate's own normalized manifest. Rendering the same
  crates again gives the same layout.
- **serve** (86): absent; Cargo reads a local registry through source
  replacement and needs no server.
- The bindings bind the projection with a store of its own, as the moon
  cache is bound.
- The conformance suite's whole projection group runs over it, and a
  tools-layer test proves a consumer's `cargo build --offline` resolves a
  family crate from the rendered layout through source replacement, its
  lockfile naming the declared registry address and no path or host.

Out of scope, and what each waits on:

- Which versions a layout holds, the policy it is rendered under, where a
  lockfile's layout lives, and `tailrace sync` writing it: bolt 5's
  "render under a policy" unit (47, 51, 99, 113).
- The registry address a family crate's consumers declare, the same on
  every host (49), and a consumer's `.cargo/config.toml`: the test
  declares one of its own; the convention waits on the design (see
  design, Open questions).
- Refusing `cargo publish` in a tailrace repository: bolt 5's Claude Code
  plugin unit.
- Crates of one workspace that depend on each other by path carry the
  requirement their manifest writes; rewriting it to the version waits on
  the design.

## Capabilities

### New Capabilities

- `cargo-projection`: the Cargo projection's pack, render and serve, and
  its binding.

### Modified Capabilities

None.

## Impact

- New profile directory `crates/tailrace/src/profiles/projection/cargo/`
  and its binding in `profiles/projection/mod.rs` and `bindings/mod.rs`.
- New dependencies, appended (phase-1 plan, "Two shared files"):
  `flate2` and `tar` to read a crate's manifest, `toml_edit` to edit one
  value of a manifest; `Cargo.lock` follows.
- The `tools` task of `crates/tailrace/moon.yml` gains the new test.
- Rules kept: core.1 and core.2 (all Cargo naming inside the profile; the
  store reached only through the store seam it is handed), core.4 and
  core.5 (bytes read by digest and checked), core.19 (cargo's words
  carried verbatim), core.33 (the whole projection group runs).

## Touches

```
crates/tailrace/src/profiles/projection/cargo/**
crates/tailrace/src/profiles/projection/mod.rs
crates/tailrace/src/bindings/mod.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/tests/cargo_projection.rs
crates/tailrace/moon.yml
crates/tailrace/Cargo.toml
Cargo.toml
Cargo.lock
```
