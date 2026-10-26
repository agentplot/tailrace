## Why

A family repository's `cargo build` cannot yet take a family crate from
anything tailrace holds: no projection packs a crate, reads what a
consumer's `Cargo.lock` pins, renders a registry Cargo reads, or answers
Cargo over its protocol. Bolt 5's Cargo unit closes that gap
(`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 5,
"Cargo's registry"), on the Cargo row of C.11 as the statement now has
it (`design/tailrace/requirements.md`, C.11, landed at blueprints
`aa591ce`): a crate published by one family repository is built against
by another from a sparse registry the host's one serving process answers
at `sparse+http://127.0.0.1:7420/<owner>/<name>/cargo/`, the same address
on every host, named `tailrace` in the checkout's cargo configuration,
and the lockfile names that address and no path.

## What Changes

- A Cargo projection, bound under the ecosystem `cargo`, under
  `crates/tailrace/src/profiles/projection/cargo/` (C.11).
- **pack** (84, 87, 110): the family repository declares a crate as a
  build output of its component, the packaging task that leaves the
  crate file. The projection reads the package's name and place from
  that crate file, sets the component's one version on the package and
  on every crate it reaches by path dependency in its workspace, rewrites
  each such path dependency to that version's requirement in the
  registry `tailrace`, and runs `cargo package --no-verify --allow-dirty
  --offline --registry tailrace` over them together, so Cargo turns the
  path dependency into a registry one as it does for any publish. Every
  edited manifest gets its own bytes back, and the output's crate is
  answered with metadata naming `cargo`, the package's name and the
  version. Cargo writes fixed times and owners into the archive, so the
  same outputs and version pack to the same bytes and the projection
  promises so. It packs no cache entries.
- **pinned** (137): the names and versions a checkout's `Cargo.lock`
  pins from a registry, spelled as the lockfile spells them, read from
  the file alone; absent where the checkout holds no `Cargo.lock`. The
  core resolves each through the name the store holds for it (138); the
  projection resolves nothing.
- **render** (85, 48): the sparse registry's files at the place given,
  holding exactly the crates given: each `.crate` file read through the
  store seam and checked against its digest, and an index file per name
  written from each crate's own normalized manifest. A dependency on a
  crate of any family repository's served registry is written as one of
  this registry, so a consumer resolves it here (49, 138). Rendering the
  same crates again gives the same layout, and the layout names no
  address.
- **serve** (86, 99, 115): Cargo's sparse protocol, answered inside the
  host's one serving process under the consuming repository's path:
  `config.json` naming where crates download from under the address the
  client reached, each name's index file from the layout, and each
  crate's bytes. A crate the layout lacks is asked of the core once and
  answered once rendered; a refused one is answered `403` with the
  refusal's own words, which Cargo prints under its error.
- The bindings bind the projection with a store of its own, as the moon
  cache is bound.
- The conformance suite's whole projection group runs over it, serve's
  scenarios included, and a tools-layer test proves the loop: a
  publisher's two sibling crates are packed, a consumer's `Cargo.lock`
  pinning them is synced, and `cargo build --locked` resolves both from
  the host's serving process, the lockfile's checksums matching the
  crates packed and its source naming the served address and no path.
- The site's projections page says how a family repository publishes a
  crate and how a consumer names the registry.

This change comes after units it builds on and implements their seams as
they land them:

- **render under a policy** adds `pinned` to the projection seam, the
  core's render that chooses a layout's versions from a lockfile's pins
  under the policy, the layout's place in the checkout and
  `tailrace sync` (137, 113, C.10).
- **the store names an artifact by ecosystem, name and version** writes
  the pointer a publish leaves at `artifact/<ecosystem>/<name>@<version>`
  (138); the proof resolves its pins through it.
- **npm, rendered and served** reshapes the seam's serve to answer one
  request with a way to ask the core for a version the layout lacks,
  routes `/<owner>/<name>/<ecosystem>/…` on the host's one process to the
  projection bound for that ecosystem, keeps the process while a layout
  is served, and has `tailrace sync` start it (114, 115); this change
  implements serve for Cargo on that route.

Out of scope, and what each waits on:

- Which versions a layout holds, the policy it is rendered under and the
  claims read for it, where a layout lives, and `tailrace sync`: render
  under a policy (47, 51, 99, 113, C.10).
- The serving process, its route, its lifetime and `tailrace sync`
  starting it: npm, rendered and served, and the gate-and-the-channels
  bolt's process it extends (114, 115, core.36).
- A name the checkout's manifest declares and no lockfile pins yet
  (137): pinned answers the lockfile's pins alone, as npm's does, until
  the design says how a range is resolved against a store that names a
  crate only by its version.
- Writing a checkout's `.cargo/config.toml`: the repository's authors
  commit the one `[registries.tailrace]` line, as a consumer writes its
  own `.npmrc`; tailrace writes none (core.21).
- Refusing `cargo publish` in a tailrace repository: bolt 5's Claude Code
  plugin unit.

## Capabilities

### New Capabilities

- `cargo-projection`: the Cargo projection's pack, pinned, render and
  serve, and its binding.

### Modified Capabilities

None.

## Impact

- New profile directory `crates/tailrace/src/profiles/projection/cargo/`
  and its binding in `profiles/projection/mod.rs` and `bindings/mod.rs`.
- Dependencies `flate2`, `tar` and `toml_edit`, appended (phase-1 plan,
  "Two shared files"); `Cargo.lock` follows.
- The `tools` task of `crates/tailrace/moon.yml` gains the new tests.
- The site's projections page, `site/src/content/docs/guides/integrations/projections.mdx`,
  and ADR 0006's consequences, which say Cargo needs no server.
- Rules kept: core.1 and core.2 (all Cargo naming inside the profile; the
  store reached only through the store seam it is handed; a missed crate
  asked of the core, never read from the store by the profile), core.4
  and core.5 (bytes read by digest and checked, the packager's crate
  never rewritten), core.13 (the version is written into the manifests
  by the projection after the build ran, never handed to the build),
  core.18 (the address the lockfile records names no host), core.19
  (absent, refused and unreadable are three responses, cargo's words
  carried verbatim), core.21 (no credential written into a cargo
  configuration), core.26 (cargo is the devenv's), core.33 (the whole
  projection group runs).

## Touches

```
crates/tailrace/src/profiles/projection/cargo/**
crates/tailrace/src/profiles/projection/mod.rs
crates/tailrace/src/bindings/mod.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/tests/cargo_projection.rs
crates/tailrace/tests/cargo_served.rs
crates/tailrace/moon.yml
crates/tailrace/Cargo.toml
Cargo.toml
Cargo.lock
site/src/content/docs/guides/integrations/projections.mdx
docs/adr/0006-sync-renders-each-ecosystem-s-own-layout-under-the-policy-serve-only-where-a-client-reads-no-files.md
```
