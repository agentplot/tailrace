## Why

A family repository's `cargo build` cannot yet take a family crate from
anything tailrace holds: no projection packs a crate, reads what a
consumer's `Cargo.lock` pins or its manifests declare, renders a
registry Cargo reads, or answers Cargo over its protocol, and the host's
one serving process answers status alone, so no client of any ecosystem
can read a layout from it. Bolt 5's Cargo unit closes that gap
(`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 5,
"Cargo's registry"), on the Cargo row of C.11 as the statement has it at
blueprints `92afa99` (`design/tailrace/requirements.md`, 84, 85, 86, 87,
99, 114, 115, 123, 137, 138, 140, C.1, C.11): a crate published by one
family repository is built against by another from a sparse registry the
host's one serving process answers at
`sparse+http://127.0.0.1:7420/<owner>/<name>/cargo/`, the same address
on every host, named `tailrace` in the checkout's cargo configuration. A
consumer writes no lockfile by hand: a crate its manifest declares and
no lockfile pins yet is held at the newest version the policy admits
within the manifest's requirement, every family crate a held crate
requires is held by the same rule through every level, the loop the
core's and what a crate requires the projection's, and Cargo's own
resolution then closes within the layout and Cargo writes the lockfile
(137, 140). A version the policy refuses never reaches Cargo with only
Cargo's own words (99).

The host's process answers every ecosystem's client in one shape: a
request, its response, and a miss, a version the layout lacks asked of
the core. npm's unit has written that shape, and Cargo answers in it
rather than writing its own, so the shape is written once (the planner's
case for proposal 231, as the user approved it).

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
  that declares nothing in `tailrace`. Every other projection answers
  pins alone, a range of the same type `requires` answers.
- **Sync closes the layout: the core's one loop** (137, 113, 99). The
  core holds each pinned version the policy admits; then, for each
  range, the newest version the store lists that the projection answers
  within it and the policy admits; then, for each version held, what the
  projection answers it requires, by the same rule, level by level,
  until nothing held requires a name the layout lacks. A range a held
  version already satisfies is met by it. Every version is judged before
  anything is written, and the projection renders the whole held set
  once. Sync's answer and the layout's record name each range, what
  required it (the checkout's manifest, or a held crate), and the
  version held for it, or that none within it was admitted. The loop is
  this change's; npm's unit and any later served projection close their
  layouts by it.
- **Cargo answers what a crate requires, and which versions a range
  admits.** Cargo's answer to the seam's question reads a crate's own
  manifest and names each dependency the layout serves: one whose
  registry is a served Cargo address, as the registry `tailrace` the
  checkout's cargo configuration routes to the host's process is written
  into a packed crate (140), normal and build, optional and target ones
  among them, with the requirement it writes; and it answers which of
  some versions satisfy a requirement, newest first, by Cargo's own
  rules (23).
- **Render writes a sparse registry.** Each crate is checked against its
  digest, and an index file per name is written from each crate's own
  normalized manifest. A dependency on a crate of any family
  repository's served registry is written as one of this registry, so a
  consumer resolves it here (49, 138). Rendering again gives the same
  layout, and the layout names no address (48).
- **Cargo answers in the shape npm's unit wrote** (86, 115). The
  projection seam's one-request serve, the fake's serve, the process's
  route of `/<owner>/<name>/<ecosystem>/…` to the projection the
  checkout binds, the served address, the core's render of a version a
  served layout lacks, the layout's lock, and their conformance
  scenarios are npm, rendered and served's, already committed on its
  branch. Cargo's serve is written to them once they are on this branch,
  and nothing of them is written here.
- **Cargo's serve** speaks Cargo's sparse protocol on the process's
  route: the registry's configuration, each name's index file from the
  layout, and each crate's bytes, a crate the layout lacks asked of the
  core once and answered once the core renders it (115, 50).
- **A refusal reaches Cargo three ways** (99). A version the policy
  refuses is left out of the name's index file Cargo lists; a request for
  its bytes answers `403` with the version and the claim found wanting,
  which Cargo prints under its error; and `tailrace sync` printed the
  same refusal when it rendered the layout. Cargo's "no matching version"
  is never the only words.
- The bindings bind the projection with a store of its own, as the moon
  cache is bound.
- The conformance suite's projection group, serve's scenarios among
  them, runs whole over Cargo; the layouts group proves a range and a
  held crate's dependency over the fakes, and 137's range and closure
  parts leave the register with the scenarios that cite them. Two
  tools-layer tests prove Cargo: a crate depending on a sibling and on a
  crate of another component packs as the statement says, the same
  bytes however many versions of the other crate the store holds; and a
  consumer with no `Cargo.lock`, declaring one family crate by its
  requirement, syncs and `cargo build` resolves that crate and the
  sibling it requires from the host's serving process, a newer version
  within the requirement that the policy refuses left out and answered
  with its claim, and Cargo writes a lockfile pinning both at the served
  address with the packed checksums, which `cargo build --locked` then
  reads after a second sync.
- The site's projections page says how a family repository publishes a
  crate and how a consumer names the registry; ADR 0006's consequences
  say Cargo is served.

This change comes after three units it builds on:

- **render under a policy** (landed) added pinned to the projection
  seam, the core's render that resolves each pin through
  `artifact/<ecosystem>/<name>@<version>` (138) and judges it under the
  layout's policy, the layout's place in the checkout, its record, and
  `tailrace sync`, which prints each refusal (137, 113, 99, C.10). This
  change extends that render with ranges, dependencies and a miss.
- **the projection answers what an artifact requires** (landed) added
  140's question to the projection seam: `requires` and `satisfying`,
  every profile answering, a projection with no manifest none, and the
  fake answering what a test sets. This change answers it for Cargo and
  asks it in the core's loop; it adds nothing to the trait for it, and
  makes `requires`' contract read as 140 now does.
- **the store names an artifact by ecosystem, name and version** (in
  flight) makes `artifact/<ecosystem>/<name>@<version>` at each publish
  and adds the store's listing of the versions a name holds on every
  store, the empty list where it holds none (138, C.1). This change takes
  its work into its branch before code that reads the listing, reads that
  listing in pack and in render, and adds none of its own.

Out of scope, and what each waits on:

- The policy a layout is rendered under, the claims read for it, where a
  layout lives, and judging one version: render under a policy (47, 51,
  99, 113, C.10); this change asks that judgment of each version a range,
  a dependency or a miss names.
- The one shape of request, response and miss, the process's route to a
  served layout, the served address, the render on a miss, the layout's
  lock, `tailrace sync` naming the served address and finding or
  starting the host's process, the process staying past its idle time
  while a layout is served, their scenarios, and 46, 49, 114 and 115
  leaving the register (86, 114, 115): npm, rendered and served, which
  has committed them and merges after this unit. How this branch takes
  them before it merges is its conductor's. Cargo's proof runs the
  host's process itself and names the consumer's repository to it.
- The serving process's lock, its file of repositories and `tailrace
  status --page`: the gate-and-the-channels bolt's status across
  machines, landed.
- The store's listing of a name's versions, on every store profile, and
  making `artifact/<ecosystem>/<name>@<version>`: the store-naming unit
  (138, C.1).
- npm's pack, pinned, requires, render and serve, and a miss closed over
  the loop (137, 140, C.11): npm, rendered and served, on this change's
  `Pin` and loop.
- A copy of a version to another store carrying its artifacts' names
  (103, 138): later work.
- Writing a checkout's `.cargo/config.toml`: the repository's authors
  commit the one `[registries.tailrace]` line, as a consumer writes its
  own `.npmrc`; tailrace writes none (core.21).
- Refusing `cargo publish` in a tailrace repository: bolt 5's Claude Code
  plugin unit, landed.

## Capabilities

### New Capabilities

- `cargo-projection`: the Cargo projection's pack, pinned, requires,
  satisfying, render and serve, and its binding.

### Modified Capabilities

- `seam-contracts`: pack is given, with the version, the output's
  component, its declared outputs and a way to ask whether the
  component's paths hold a path; pinned answers pins and ranges, a range
  of the type requires answers; requires answers the names the layout
  serves; the contract written whole as 84, 86, 115, 137 and 140 read,
  serve as npm's unit words it.
- `publishing`: a publish hands each pack its component, the component's
  outputs as the build left them, and its paths' match.
- `consumer-layouts`: a layout holds the pinned versions, the newest
  admitted version within each range, and the family dependencies each
  held version requires.
- `seam-fakes`: the fake projection answers the pins and ranges a test
  sets.
- `conformance-suite`: a projection's rig names its ecosystem's request
  for a version's bytes, Cargo runs the projection group, and the
  layouts group holds a range and the closure.

## Impact

- New profile directory `crates/tailrace/src/profiles/projection/cargo/`
  and its binding in `profiles/projection/mod.rs` and `bindings/mod.rs`.
- The projection seam's pack takes what it packs at and pinned answers
  `Pin`; every projection implementation and every caller follows: the
  file, image and moon cache profiles, the fake and the fakes' own tests,
  the conformance rigs, the core's cache push, render and tests, and
  whatever of npm's code this branch holds.
- Publish hands pack the component, its outputs and its paths' match.
- The core's render closes a layout over ranges and requirements;
  `tailrace sync` prints each range.
- Cargo's serve is written to the seam's one-request serve, which npm's
  unit committed; the process, the served address and the miss are used
  as that unit wrote them.
- The conformance register loses 137's range and closure parts; the
  layouts group gains their scenarios, and the projection group runs
  over Cargo.
- No new dependency: `flate2`, `tar` and `toml_edit` are already
  appended, and `semver` and `glob` are the workspace's.
- The `tools` task of `crates/tailrace/moon.yml` gains the new tests.
- The site's projections page and ADR 0006's consequences, which say
  Cargo needs no server.
- Rules kept: core.1 and core.2 (all Cargo naming inside the profile; the
  store reached only
  through the store seam; the component's paths matched by the core;
  which versions a range admits, their order and which crates a crate
  requires answered by the projection, the loop, the listing's reading
  and whether a version is admitted the core's, no manifest read by the
  core), core.4 and core.5 (bytes read by digest and checked, the
  packager's crate never rewritten), core.13 (the version is written into
  the manifests by the projection after the build ran, never handed to
  the build), core.17 (a layout holds only admitted versions, a range's,
  a dependency's and a crate a request asks among them), core.18 and core.36 (one
  address on every host, one process; the address the lockfile records
  names no host), core.19 (absent, refused and unreadable are three
  responses, the source's words carried verbatim; an empty listing is a
  store that holds none, never an unreadable one), core.21 (no
  credential written into a cargo configuration), core.26 (cargo is the
  devenv's), core.31 (Cargo's serve words the core's answer to a miss), core.33
  (the whole projection group runs, and each clause leaves the register
  with the scenario that cites it), core.37 (a sibling is a crate of the
  packed one's component, by its paths).

## Touches

```
Cargo.toml
Cargo.lock
crates/tailrace/Cargo.toml
crates/tailrace/src/profiles/projection/**
crates/tailrace/src/bindings/mod.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/src/commands/serve/host.rs
crates/tailrace/src/commands/serve/mod.rs
crates/tailrace/src/commands/sync.rs
crates/tailrace/tests/**
crates/tailrace/moon.yml
crates/tailrace-seams/Cargo.toml
crates/tailrace-seams/src/projection.rs
crates/tailrace-seams/src/lib.rs
crates/tailrace-core/src/publish.rs
crates/tailrace-core/src/history.rs
crates/tailrace-core/src/cache.rs
crates/tailrace-core/src/render.rs
crates/tailrace-core/src/served.rs
crates/tailrace-core/src/lib.rs
crates/tailrace-core/tests/**
crates/tailrace-fakes/src/projection.rs
crates/tailrace-fakes/src/lib.rs
crates/tailrace-conformance/src/**
crates/tailrace-conformance/tests/**
site/src/content/docs/guides/integrations/projections.mdx
docs/adr/0006-sync-renders-each-ecosystem-s-own-layout-under-the-policy-serve-only-where-a-client-reads-no-files.md
```
