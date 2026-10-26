## Why

A family repository's `cargo build` cannot yet take a family crate from
anything tailrace holds: no projection packs a crate, reads what a
consumer's `Cargo.lock` pins, renders a registry Cargo reads, or answers
Cargo over its protocol. Bolt 5's Cargo unit closes that gap
(`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 5,
"Cargo's registry"), on the Cargo row of C.11 as the statement has it at
blueprints `fbecdbf` (`design/tailrace/requirements.md`, 84, 85, 87, 99,
123, 138, C.1, C.11): a crate published by one family repository is
built against by another from a sparse registry the host's one serving
process answers at `sparse+http://127.0.0.1:7420/<owner>/<name>/cargo/`,
the same address on every host, named `tailrace` in the checkout's cargo
configuration; the lockfile names that address and no path; and a
version the policy refuses never reaches Cargo with only Cargo's own
words.

## What Changes

- **A Cargo projection**, bound under the ecosystem `cargo`, under
  `crates/tailrace/src/profiles/projection/cargo/` (C.11).
- **Pack is told the output's component.** Beside the output, its build
  identity and the version, pack is given the component the output
  belongs to, with its declared outputs as the build left them, and a way
  to ask whether the component's paths hold a path of the repository
  (84, 123). Every projection takes it; only Cargo's reads more than the
  version.
- **A crate is packed by Cargo's own packager, with its component's
  version.** The family repository declares a crate as a build output of
  its component: the packaging task that leaves the crate file. The
  projection reads the package's name and place from that crate file,
  sets the component's one version on the package and on every crate of
  the same component it reaches by path dependency, and packs the package
  with `cargo package --no-verify --exclude-lockfile`, so each path
  dependency on a sibling crate becomes a dependency on that version in
  the registry `tailrace`, as Cargo does for any publish (84, 110, C.11).
  A sibling none of the component's outputs packs is refused naming it,
  since that version of it would never be published. Every edited
  manifest gets its own bytes back.
- **A crate of another component is depended on as the manifest
  writes.** A crate belongs to the component whose paths hold its
  manifest (123). A path dependency on a crate of another component keeps
  the version requirement the manifest writes for it and is packed as a
  dependency on that requirement in the registry `tailrace`. Pack picks
  no version of it: which version a build uses is the consumer's
  lockfile's. Where the store holds no version within the requirement,
  the pack is refused naming the crate, the requirement and the versions
  held (C.11, 138).
- **The same outputs and version pack to the same bytes.** Cargo writes
  fixed times and owners into the archive, and packed without its own
  lockfile a crate is a function of the tree, the version and the
  manifests' text, so the projection promises the same bytes again (87).
  It packs no cache entries.
- **The store lists the versions a name holds** (138). The store seam
  answers, for an ecosystem and a name, the versions it holds under
  `artifact/<ecosystem>/<name>@<version>`: the registry from the tag list
  of the repository C.1 maps that name to, each tag read back to its
  version; the layout store from its index's names; the fake from its
  names. Cargo's pack asks it of the store it is bound with.
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
  crate of another component packs as the statement says, the same bytes
  however many versions of the other crate the store holds; and a
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
- **the store names an artifact by ecosystem, name and version** makes
  `artifact/<ecosystem>/<name>@<version>` at each publish, and on the
  registry makes it in the repository C.1 maps it to, with the version as
  its tag (138, C.1). This change's listing reads what that unit makes,
  by the same spelling.

Out of scope, and what each waits on:

- Which versions a layout holds, the policy it is rendered under and the
  claims read for it, where a layout lives, and `tailrace sync`: render
  under a policy (47, 51, 99, 113, C.10).
- The serving process, its route, its lifetime and `tailrace sync`
  starting it: npm, rendered and served, and the gate-and-the-channels
  bolt's process it extends (114, 115, core.36).
- Making `artifact/<ecosystem>/<name>@<version>` at publish, and on the
  registry its repository and tag (138, C.1): the store-naming unit.
- A copy of a version to another store carrying its artifacts' names
  (103, 138): later work; the clause comes with the unit that does it.
- A name the checkout's manifest declares and no lockfile pins yet
  (137): pinned answers the lockfile's pins alone, as npm's does; the
  listing this change adds is what such a range resolves by (138), and
  the planner places the rest.
- A conformance scenario citing 138: it comes with the store-naming
  unit. The suite's statement list stops at blueprints `5b00e15`, before
  126 to 138, so the register cannot name 138 until the commit adopting
  the statement past it, which registers it with that unit. Each store
  profile's listing is proved by that profile's own tests here.
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

- `seam-contracts`: the store lists the versions a name holds; pack is
  given, with the version, the output's component, its declared outputs
  and a way to ask whether the component's paths hold a path.
- `registry-store`: lists a name's versions from its repository's tags.
- `layout-store`: lists a name's versions from its index.
- `seam-fakes`: the fake store lists a name's versions.
- `publishing`: a publish hands each pack its component, the component's
  outputs as the build left them, and its paths' match.

## Impact

- New profile directory `crates/tailrace/src/profiles/projection/cargo/`
  and its binding in `profiles/projection/mod.rs` and `bindings/mod.rs`.
- The store seam gains its listing; every store implementation answers
  it: the registry and layout profiles, the fake store, and the stores
  tests wrap.
- The projection seam's pack takes what it packs at; every projection
  implementation and every caller follows: the file, image, moon cache
  and npm profiles, the fake, the conformance rigs, the core's tests.
- Publish hands pack the component, its outputs and its paths' match.
- Dependencies `flate2`, `tar` and `toml_edit`, appended (phase-1 plan,
  "Two shared files"); `Cargo.lock` follows.
- The `tools` task of `crates/tailrace/moon.yml` gains the new tests.
- The site's projections page and ADR 0006's consequences, which say
  Cargo needs no server.
- Rules kept: core.1 and core.2 (all Cargo naming inside the profile; the
  store reached only through the store seam; the component's paths
  matched by the core), core.4 and core.5 (bytes read by digest and
  checked, the packager's crate never rewritten), core.13 (the version is
  written into the manifests by the projection after the build ran,
  never handed to the build), core.18 (the address the lockfile records
  names no host), core.19 (absent, refused and unreadable are three
  responses, cargo's words carried verbatim; a store that holds no
  version of a name answers none, a listing that fails answers
  unreadable), core.21 (no credential written into a cargo
  configuration), core.26 (cargo is the devenv's), core.33 (the whole
  projection group runs), core.37 (a sibling is a crate of the packed
  one's component, by its paths).

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
