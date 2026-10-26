## Why

A family repository's `cargo build` cannot yet take a family crate from
anything tailrace holds: no projection packs a crate, reads what a
consumer's `Cargo.lock` pins or its manifests declare, renders a
registry Cargo reads, or answers Cargo over its protocol. Bolt 5's Cargo
unit closes that gap
(`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 5,
"Cargo's registry"), on the Cargo row of C.11 as the statement has it at
blueprints `4c14235` (`design/tailrace/requirements.md`, 84, 85, 87, 99,
123, 137, 138, 140, C.1, C.11): a crate published by one family repository is
built against by another from a sparse registry the host's one serving
process answers at `sparse+http://127.0.0.1:7420/<owner>/<name>/cargo/`,
the same address on every host, named `tailrace` in the checkout's cargo
configuration. A consumer writes no lockfile by hand: a crate its
manifest declares and no lockfile pins yet is held at the newest version
the policy admits within the manifest's requirement, every family crate
a held crate requires is held by the same rule through every level, the
loop the core's and what a crate requires the projection's, and Cargo's
own resolution then closes within the layout and Cargo writes the
lockfile (137, 140). A version the policy refuses never reaches Cargo
with only Cargo's own words (99).

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
  no version of it. It reads the store's listing of that crate's
  versions, which the store-naming unit adds; where the listing holds
  none within the requirement, an empty listing among them, the pack is
  refused naming the crate, the requirement and the versions held, or
  that the store holds none (C.11, 138).
- **The same outputs and version pack to the same bytes.** Cargo writes
  fixed times and owners into the archive, and packed without its own
  lockfile a crate is a function of the tree, the version and the
  manifests' text, so the projection promises the same bytes again (87).
  It packs no cache entries.
- **Pinned answers pins and ranges** (137). A pin is a name and version
  a checkout's `Cargo.lock` pins from a registry, spelled as the lockfile
  spells it. A range is a crate the checkout's manifests declare in the
  registry `tailrace` that no lockfile pins yet, with the requirement the
  manifest writes. A checkout with a `Cargo.toml` and no `Cargo.lock`
  answers its manifests' ranges, so a first install needs no lockfile
  written by hand. Pinned reads the files alone, and answers absent only
  where the checkout holds neither file, or a manifest and no lockfile
  that declares nothing in `tailrace`.
- **The seam carries ranges and what a version requires** (137, 140).
  pinned answers each entry as a pin or a range. The projection gains
  `requires`, the family's crates and ranges a held version's manifest
  requires, as the manifest spells them, and `within`, the versions that
  satisfy a range, newest first by the ecosystem's own order. render is
  unchanged: it holds exactly what the core gives it. The file, image,
  moon cache and npm projections answer both with none.
- **Sync closes the layout** (137, 113, 99). The core holds each pinned
  version the policy admits; then, for each range, the newest version
  the store lists, `within` keeps and the policy admits; then, for each
  version held, what `requires` answers of it, by the same rule, level
  by level, until nothing held requires a name the layout lacks. A range
  a held version already satisfies is met by it. Every version is judged
  before anything is written, and the projection renders the whole held
  set once. Sync's answer and the layout's record name each range, what
  required it (the checkout's manifest, or a held crate), and the
  version held for it, or that none within it was admitted.
- **Cargo's requires, within and render.** `requires` reads a crate's
  own manifest and answers each dependency on a family crate, one whose
  registry is a served Cargo address, normal and build, optional and
  target ones among them, with the requirement it writes. `within` reads
  a requirement by Cargo's own rules, newest first. render writes the
  sparse registry's files at the place given: each crate checked against
  its digest, and an index file per name written from each crate's own
  normalized manifest. A dependency on a crate of any family
  repository's served registry is written as one of this registry, so a
  consumer resolves it here (49, 138). Rendering again gives the same
  layout, and the layout names no address (48).
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
- The conformance suite's whole projection group runs over it; the
  layouts group proves a range and a held crate's dependency over the
  fakes, and 137's range part leaves the register. Two tools-layer tests
  prove Cargo: a crate depending on a sibling and on a crate of another
  component packs as the statement says, the same bytes however many
  versions of the other crate the store holds; and a consumer with no
  `Cargo.lock`, declaring one family crate by its requirement, syncs and
  `cargo build` resolves that crate and the sibling it requires from the
  host's serving process, a newer version within the requirement that the
  policy refuses left out and answered with its claim, and Cargo writes a
  lockfile pinning both at the served address with the packed checksums,
  which `cargo build --locked` then reads after a second sync.
- The site's projections page says how a family repository publishes a
  crate and how a consumer names the registry; ADR 0006's consequences
  say Cargo is served.

This change comes after units it builds on and implements their seams as
they land them:

- **render under a policy** adds pinned to the projection seam, the
  core's render that resolves each pin through
  `artifact/<ecosystem>/<name>@<version>` (138) and judges it under the
  layout's policy, the layout's place in the checkout, its record, and
  `tailrace sync`, which prints each refusal (137, 113, 99, C.10). This
  change extends that render with ranges and dependencies.
- **npm, rendered and served** reshapes serve to answer one request with
  a way to ask the core for a version the layout lacks, routes
  `/<owner>/<name>/<ecosystem>/…` on the host's one process to the
  projection bound for that ecosystem, keeps the process while a layout
  is served, and has `tailrace sync` start it (114, 115); this change
  implements serve for Cargo on that route.
- **the store names an artifact by ecosystem, name and version** makes
  `artifact/<ecosystem>/<name>@<version>` at each publish and adds the
  store's listing of the versions a name holds on every store, the empty
  list where it holds none (138, C.1). This change reads that listing in
  pack and in render and adds none of its own.

Out of scope, and what each waits on:

- The policy a layout is rendered under, the claims read for it, where a
  layout lives, and judging one version: render under a policy (47, 51,
  99, 113, C.10); this change asks that judgment of each version a range
  or a dependency names.
- The serving process, its route, its lifetime and `tailrace sync`
  starting it: npm, rendered and served, and the gate-and-the-channels
  bolt's process it extends (114, 115, core.36).
- The store's listing of a name's versions, on every store profile, and
  making `artifact/<ecosystem>/<name>@<version>`: the store-naming unit
  (138, C.1).
- npm's ranges and what an npm version requires of the family (137 and
  140 bind every ecosystem with a manifest): not carved; npm's projection
  answers no range and requires nothing here.
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
  component's paths hold a path; pinned answers pins and ranges; the
  projection answers requires and within (140).
- `publishing`: a publish hands each pack its component, the component's
  outputs as the build left them, and its paths' match.
- `consumer-layouts`: a layout holds the pinned versions, the newest
  admitted version within each range, and the family dependencies each
  held version requires, and sync answers each range.

## Impact

- New profile directory `crates/tailrace/src/profiles/projection/cargo/`
  and its binding in `profiles/projection/mod.rs` and `bindings/mod.rs`.
- The projection seam's pack takes what it packs at; pinned answers
  `Pin`; the seam gains `requires` and `within`. Every projection implementation and every caller follows: the
  file, image, moon cache and npm profiles, the fake and the fakes' own
  tests, the conformance rigs, the core's cache push, render and tests.
- Publish hands pack the component, its outputs and its paths' match.
- The core's render closes a layout over ranges and requirements;
  `tailrace sync` prints each range.
- The conformance register loses 137's range part; the layouts group
  gains its scenarios.
- No new dependency: `flate2`, `tar` and `toml_edit` are already
  appended, and `semver` and `glob` are the workspace's.
- The `tools` task of `crates/tailrace/moon.yml` gains the new tests.
- The site's projections page and ADR 0006's consequences, which say
  Cargo needs no server.
- Rules kept: core.1 and core.2 (all Cargo naming inside the profile; the
  store reached only through the store seam; the component's paths
  matched by the core; which versions are within a range, which is
  newest and which crates a crate requires answered by the projection,
  the loop, the listing's reading and whether a version is admitted the
  core's, no manifest read by the core), core.4 and core.5 (bytes
  read by digest and checked, the packager's crate never rewritten),
  core.13 (the version is written into the manifests by the projection
  after the build ran, never handed to the build), core.17 (a layout
  holds only admitted versions, a range's and a dependency's among
  them), core.18 (the address the lockfile records names no host),
  core.19 (absent, refused and unreadable are three responses, cargo's
  words carried verbatim; an empty listing is a store that holds none,
  never an unreadable one), core.21 (no credential written into a cargo
  configuration), core.26 (cargo is the devenv's), core.33 (the whole
  projection group runs, and 137's range part leaves the register with
  the scenarios that cite it), core.37 (a sibling is a crate of the
  packed one's component, by its paths).

## Touches

```
crates/tailrace/src/profiles/projection/**
crates/tailrace/src/bindings/mod.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/src/commands/sync.rs
crates/tailrace/tests/**
crates/tailrace/moon.yml
crates/tailrace-seams/src/projection.rs
crates/tailrace-seams/src/lib.rs
crates/tailrace-core/src/publish.rs
crates/tailrace-core/src/cache.rs
crates/tailrace-core/src/render.rs
crates/tailrace-core/src/lib.rs
crates/tailrace-core/tests/**
crates/tailrace-fakes/src/projection.rs
crates/tailrace-fakes/src/lib.rs
crates/tailrace-conformance/src/**
crates/tailrace-conformance/tests/**
site/src/content/docs/guides/integrations/projections.mdx
docs/adr/0006-sync-renders-each-ecosystem-s-own-layout-under-the-policy-serve-only-where-a-client-reads-no-files.md
```
