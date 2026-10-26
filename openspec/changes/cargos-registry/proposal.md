## Why

A family repository's `cargo build` cannot yet take a family crate from
anything tailrace holds: no projection packs a crate, reads what a
consumer's `Cargo.lock` pins or its manifests declare, renders a
registry Cargo reads, or answers Cargo over its protocol. Bolt 5's Cargo
unit closes that gap
(`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 5,
"Cargo's registry"), on the Cargo row of C.11 as the statement has it at
blueprints `b720017` (`design/tailrace/requirements.md`, 84, 85, 87, 99,
123, 137, 138, C.1, C.11): a crate published by one family repository is
built against by another from a sparse registry the host's one serving
process answers at `sparse+http://127.0.0.1:7420/<owner>/<name>/cargo/`,
the same address on every host, named `tailrace` in the checkout's cargo
configuration; the lockfile names that address and no path; a crate a
manifest declares and no lockfile pins yet resolves to the newest
version the policy admits within the manifest's requirement; and a
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
  A sibling the component does not declare as an output is refused
  naming it, since a consumer could resolve nothing for it (C.11). Every
  edited manifest gets its own bytes back.
- **A crate of another component is depended on as the manifest
  writes.** A crate belongs to the component whose paths hold its
  manifest (123). A path dependency on a crate of another component keeps
  the version requirement the manifest writes for it and is packed as a
  dependency on that requirement in the registry `tailrace`. Pack picks
  no version of it. Where the store's listing of that crate's versions,
  which the store-naming unit adds, holds none within the requirement,
  the pack is refused naming the crate, the requirement and the versions
  held (C.11, 138).
- **The same outputs and version pack to the same bytes.** Cargo writes
  fixed times and owners into the archive, and packed without its own
  lockfile a crate is a function of the tree, the version and the
  manifests' text, so the projection promises the same bytes again (87).
  It packs no cache entries.
- **Pinned** (137): the names and versions a checkout's `Cargo.lock`
  pins from a registry, spelled as the lockfile spells them; and, for
  each crate the checkout's manifests declare in the registry `tailrace`
  that the lockfile pins no version of, the requirement the manifest
  writes, as its range. Read from the files alone; absent where the
  checkout holds no `Cargo.lock`.
- **Render** (85, 137, 48): the sparse registry's files at the place
  given, holding exactly the crates given, and of each range's
  candidates the newest within the range by Cargo's own requirement
  rules, or none; each crate read through the store seam and checked
  against its digest, and an index file per name written from each
  crate's own normalized manifest. A dependency on a crate of any family
  repository's served registry is written as one of this registry, so a
  consumer resolves it here (49, 138). Rendering the same crates again
  gives the same layout, and the layout names no address. So a consumer
  that adds a family crate to its manifest syncs, builds, and Cargo
  writes the pin.
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
  its source naming the served address and no path, a refused version
  printed by sync and answered with its claim, and a crate the manifest
  adds by range resolved to the newest admitted version and pinned by
  Cargo.
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
  `artifact/<ecosystem>/<name>@<version>` at each publish, adds the
  store's listing of the versions a name holds on every store (138,
  C.1), reshapes pinned to answer a manifest's range beside a lockfile's
  pins, and has the core list a range's name, judge each version under
  the layout's policy and hand render the admitted ones as candidates
  (137). This change reads that listing in pack and implements the
  range in Cargo's pinned and render.

Out of scope, and what each waits on:

- Which versions a layout holds, the policy it is rendered under and the
  claims read for it, where a layout lives, and `tailrace sync`: render
  under a policy (47, 51, 99, 113, C.10).
- The serving process, its route, its lifetime and `tailrace sync`
  starting it: npm, rendered and served, and the gate-and-the-channels
  bolt's process it extends (114, 115, core.36).
- The store's listing of a name's versions, on every store profile, and
  making `artifact/<ecosystem>/<name>@<version>` with, on the registry,
  its repository and tag; listing a range's name and judging each
  version; and 138's and 137's range scenarios in the suite: the
  store-naming unit (138, C.1, 137).
- A copy of a version to another store carrying its artifacts' names
  (103, 138): later work.
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

- `seam-contracts`: pack is given, with the version, the output's
  component, its declared outputs and a way to ask whether the
  component's paths hold a path.
- `publishing`: a publish hands each pack its component, the component's
  outputs as the build left them, and its paths' match.

## Impact

- New profile directory `crates/tailrace/src/profiles/projection/cargo/`
  and its binding in `profiles/projection/mod.rs` and `bindings/mod.rs`.
- The projection seam's pack takes what it packs at; every projection
  implementation and every caller follows: the file, image, moon cache
  and npm profiles, the fake and the fakes' own tests, the conformance
  rigs, the core's cache push and tests.
- Publish hands pack the component, its outputs and its paths' match.
- Dependencies `flate2`, `tar` and `toml_edit`, appended (phase-1 plan,
  "Two shared files"); `Cargo.lock` follows.
- The `tools` task of `crates/tailrace/moon.yml` gains the new tests.
- The site's projections page and ADR 0006's consequences, which say
  Cargo needs no server.
- Rules kept: core.1 and core.2 (all Cargo naming inside the profile; the
  store reached only through the store seam; the component's paths
  matched by the core; which versions are within a range decided by the
  projection, which may be held by the core under the policy), core.4
  and core.5 (bytes read by digest and checked, the packager's crate
  never rewritten), core.13 (the version is written into the manifests by
  the projection after the build ran, never handed to the build),
  core.18 (the address the lockfile records names no host), core.19
  (absent, refused and unreadable are three responses, cargo's words
  carried verbatim; a crate the store lists no version of answers the
  pack refused naming it), core.21 (no credential written into a cargo
  configuration), core.26 (cargo is the devenv's), core.33 (the whole
  projection group runs), core.37 (a sibling is a crate of the packed
  one's component, by its paths).

## Touches

```
crates/tailrace/src/profiles/projection/**
crates/tailrace/src/bindings/mod.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/tests/**
crates/tailrace/moon.yml
crates/tailrace/Cargo.toml
crates/tailrace-seams/src/projection.rs
crates/tailrace-seams/src/lib.rs
crates/tailrace-core/src/publish.rs
crates/tailrace-core/src/cache.rs
crates/tailrace-core/tests/**
crates/tailrace-fakes/src/projection.rs
crates/tailrace-fakes/src/lib.rs
crates/tailrace-conformance/src/**
crates/tailrace-conformance/tests/**
Cargo.toml
Cargo.lock
site/src/content/docs/guides/integrations/projections.mdx
docs/adr/0006-sync-renders-each-ecosystem-s-own-layout-under-the-policy-serve-only-where-a-client-reads-no-files.md
```
