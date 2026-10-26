## Why

A family repository's `cargo build` cannot yet take a family crate from
anything tailrace holds: no projection packs a crate, reads what a
consumer's `Cargo.lock` pins, renders a registry Cargo reads, or answers
Cargo over its protocol. Bolt 5's Cargo unit closes that gap
(`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 5,
"Cargo's registry"), on the Cargo row of C.11 as the statement has it at
blueprints `d91401e` (`design/tailrace/requirements.md`, C.11, 99, 123,
138): a crate published by one family repository is built against by
another from a sparse registry the host's one serving process answers at
`sparse+http://127.0.0.1:7420/<owner>/<name>/cargo/`, the same address
on every host, named `tailrace` in the checkout's cargo configuration;
the lockfile names that address and no path; and a version the policy
refuses never reaches Cargo with only Cargo's own words.

## What Changes

- **A Cargo projection**, bound under the ecosystem `cargo`, under
  `crates/tailrace/src/profiles/projection/cargo/` (C.11).
- **A crate is packed by Cargo's own packager, with its component's
  version.** The family repository declares a crate as a build output of
  its component: the packaging task that leaves the crate file. The
  projection reads the package's name and place from that crate file,
  sets the component's one version on the package and on every crate of
  the same component it reaches by path dependency, and packs them with
  `cargo package --no-verify`, so each path dependency on a sibling crate
  becomes a dependency on that version in the registry `tailrace`, as
  Cargo does for any publish (84, 110). Every edited manifest gets its
  own bytes back.
- **A crate of another component is depended on as the manifest
  writes.** A crate belongs to the component whose paths hold its
  manifest (123). A path dependency on a crate of another component keeps
  the version requirement the manifest writes for it and is packed as a
  dependency on that requirement in the registry `tailrace`, resolved
  against the versions the store holds of that crate; where the store
  holds none within the requirement, the pack is refused naming the
  crate, the requirement and the versions held (C.11, 138).
- **The same outputs and version pack to the same bytes, or the
  projection says not.** Cargo writes fixed times and owners into the
  archive, so a crate whose dependencies are all of its own component or
  third-party packs to the same bytes again, and the projection promises
  so. A crate that reaches a crate of another component carries, in its
  own lockfile, the newest version the store held within the requirement
  when it was packed, so the projection says it cannot promise the same
  bytes (87). It packs no cache entries.
- **The store lists the versions a name holds.** The store seam answers
  the names it holds under a prefix, the registry from its repository's
  tags and the layout store from its index's names, and the core answers
  a projection the artifacts the store holds of one ecosystem name, each
  found in its version's index (138). A publish hands each pack its
  component's paths and that listing.
- **Pinned** (137): the names and versions a checkout's `Cargo.lock`
  pins from a registry, spelled as the lockfile spells them, read from
  the file alone; absent where the checkout holds no `Cargo.lock`.
- **Render** (85, 48): the sparse registry's files at the place given,
  holding exactly the crates given, each read through the store seam and
  checked against its digest, and an index file per name written from
  each crate's own normalized manifest. A dependency on a crate of any
  family repository's served registry is written as one of this registry,
  so a consumer resolves it here (49, 138). Rendering the same crates
  again gives the same layout, and the layout names no address.
- **Serve** (86, 115): Cargo's sparse protocol, answered inside the
  host's one serving process under the consuming repository's path: the
  registry's configuration, each name's index file from the layout, and
  each crate's bytes, a crate the layout lacks asked of the core once.
- **A refusal reaches Cargo three ways** (99). A version the policy
  refuses is left out of the name's index file Cargo lists; a request for
  its bytes answers `403` with the version and the claim found wanting,
  which Cargo prints under its error; and `tailrace sync` printed the
  same refusal when it rendered the layout. Cargo's "no matching version"
  is never the only words.
- The bindings bind the projection with a store of its own, as the moon
  cache is bound.
- The conformance suite's whole projection group runs over it, and two
  tools-layer tests prove it: a crate depending on a sibling and on a
  crate of another component packs as the statement says; and a
  consumer's `Cargo.lock` pinning a publisher's two sibling crates is
  synced, and `cargo build --locked` resolves both from the host's
  serving process, the lockfile's checksums matching the crates packed,
  its source naming the served address and no path, and a refused
  version printed by sync and answered with its claim.
- The site's projections page says how a family repository publishes a
  crate and how a consumer names the registry; ADR 0006's consequences
  say Cargo is served.

This change comes after units it builds on and implements their seams as
they land them:

- **render under a policy** adds pinned to the projection seam, the
  core's render that resolves each pin through
  `artifact/<ecosystem>/<name>@<version>` (138) and chooses a layout's
  versions under the policy, the layout's place in the checkout, and
  `tailrace sync`, which prints each refusal (137, 113, 99, C.10).
- **npm, rendered and served** reshapes serve to answer one request with
  a way to ask the core for a version the layout lacks, routes
  `/<owner>/<name>/<ecosystem>/…` on the host's one process to the
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
- Writing the `artifact/<ecosystem>/<name>@<version>` pointer at publish
  (138): the store-naming unit. Until it lands, the listing finds only
  names a test or a store already holds. The store's listing itself,
  which render under a policy leaves to that unit, is this change's,
  since a Cargo pack is refused by what the listing answers (C.11); the
  store-naming unit writes its pointers beside it.
- A name the checkout's manifest declares and no lockfile pins yet
  (137): pinned answers the lockfile's pins alone, as npm's does. Render
  under a policy leaves the range to the ecosystems' units, and this
  unit's sources (84, 85, 87, C.11) do not carry it: the pin on the seam
  carries no range and the core's render chooses no newest version. The
  listing this change adds is what such a range would resolve by (138);
  the planner places the rest.
- A conformance scenario of the store's listing: the suite's statement
  list stops at blueprints `5b00e15`, before 138, so no scenario can
  cite it; the commit adopting the statement past it adds one. Each
  store profile's listing is proved by that profile's own tests here.
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

- `seam-contracts`: the store answers the names it holds under a prefix;
  pack is given, with the version, the component's paths and the
  artifacts the store holds of a name.
- `registry-store`: lists names from the repository's tags.
- `layout-store`: lists names from its index.
- `seam-fakes`: the fake store lists names.
- `publishing`: a publish hands each pack its component's paths and the
  store's artifacts of a name.

## Impact

- New profile directory `crates/tailrace/src/profiles/projection/cargo/`
  and its binding in `profiles/projection/mod.rs` and `bindings/mod.rs`.
- The store seam gains its listing; every store implementation answers
  it: the registry and layout profiles, the fake store, and the stores
  tests wrap.
- The projection seam's pack takes what it packs at; every projection
  implementation and every caller follows: the file, image, moon cache
  and npm profiles, the fake, the conformance rigs, the core's tests.
- The core answers the artifacts the store holds of a name, beside its
  pin resolution, and publish hands both to pack.
- Dependencies `flate2`, `tar` and `toml_edit`, appended (phase-1 plan,
  "Two shared files"); `Cargo.lock` follows.
- The `tools` task of `crates/tailrace/moon.yml` gains the new tests.
- The site's projections page and ADR 0006's consequences, which say
  Cargo needs no server.
- Rules kept: core.1 and core.2 (all Cargo naming inside the profile; the
  store reached only through the store seam, the store's listing and
  pin resolution the core's, the component's paths matched by the core),
  core.4 and core.5 (bytes read by digest and checked, the packager's
  crate never rewritten), core.13 (the version is written into the
  manifests by the projection after the build ran, never handed to the
  build), core.18 (the address the lockfile records names no host),
  core.19 (absent, refused and unreadable are three responses, cargo's
  words carried verbatim; a store that holds no name under a prefix
  answers none, a listing that fails answers unreadable), core.21 (no
  credential written into a cargo configuration), core.26 (cargo is the
  devenv's), core.33 (the whole projection group runs), core.37 (a
  sibling is a crate of the packed one's component, by its paths).

## Touches

```
crates/tailrace/src/profiles/projection/**
crates/tailrace/src/profiles/store/**
crates/tailrace/src/bindings/mod.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/tests/**
crates/tailrace/moon.yml
crates/tailrace/Cargo.toml
crates/tailrace-seams/src/projection.rs
crates/tailrace-seams/src/store.rs
crates/tailrace-seams/src/lib.rs
crates/tailrace-core/src/render.rs
crates/tailrace-core/src/publish.rs
crates/tailrace-core/src/copy.rs
crates/tailrace-core/src/repair.rs
crates/tailrace-core/tests/**
crates/tailrace-fakes/src/store.rs
crates/tailrace-fakes/src/projection.rs
crates/tailrace-conformance/src/**
crates/tailrace-conformance/tests/**
Cargo.toml
Cargo.lock
site/src/content/docs/guides/integrations/projections.mdx
docs/adr/0006-sync-renders-each-ecosystem-s-own-layout-under-the-policy-serve-only-where-a-client-reads-no-files.md
```
