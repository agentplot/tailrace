## Why

A family repository that depends on another family repository's npm
packages still installs them from wherever its `.npmrc` points, because
tailrace cannot pack an npm package, cannot say what a checkout's
`package-lock.json` pins or its `package.json` asks for, cannot say what
a held package requires of the family, cannot render an npm layout, and
npm, which reads no files, has nothing to read one from. This unit gives
npm all of it: a publish packs with npm's own packager, `tailrace sync`
renders the family's versions the checkout pins or asks for, and every
family package those require, under the policy, and the host's one
serving process answers npm at one fixed address, with the consuming
repository in the path. `npm install` then reads the family's packages
from that layout and from nowhere else, with no lockfile written by
hand, and the lockfile npm writes names no host
(`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 5,
"npm, rendered and served"; `design/tailrace/requirements.md` at
`4c14235`, 137 and 140; ADR 0006).

## What Changes

- **npm packs with npm's own packager.** The npm projection copies the
  files the build left under the output's `package.json` into a scratch
  directory, sets the version there with `npm pkg set version=<v>`, and
  runs `npm pack --ignore-scripts`; the tarball is the artifact's bytes,
  named in npm by the package's own `name` (C.11, 84, 4). It never edits
  the tarball, never edits the build's own files, and runs no script of
  the package, since tailrace builds nothing (15). It says it cannot
  promise the same bytes again, since npm does not promise it (87).
- **npm answers what a checkout pins and what it asks for.** pinned
  reads the checkout's `npm-shrinkwrap.json` or `package-lock.json` and
  answers every package it pins by its npm name and version, as the
  lockfile spells them (137). Beside those, for each family package the
  checkout's `package.json`, or a workspace's, declares and no lockfile
  pins yet, it answers the name and the range as `package.json` spells
  it, so a first `npm install` needs no lockfile written by hand (137).
  A family package is one whose scope the checkout's own `.npmrc` sends
  to the host's serving process. A checkout with neither a lockfile nor a
  family range is absent.
- **npm answers what a held package requires of the family** (140).
  Given a held version's artifact, the projection reads the
  `package.json` inside its tarball and answers each family package it
  requires, from `dependencies`, `optionalDependencies` and the
  `peerDependencies` npm installs, with its range as written; and it
  answers whether a version falls within a range, and which of two is
  newer, by npm's own rules, through the `nodejs-semver` crate (23). The
  core runs the loop: it lists a name's versions through the store,
  judges them under the policy and asks again of what it held, until
  nothing held requires a name the layout lacks (137).
- **npm renders a registry read layout.** Given the versions the policy
  admits, the projection writes each tarball, read from the store by
  digest, and one packument per package name listing the versions held,
  under the checkout's `.tailrace/layouts/npm/` (85, C.11). A refused
  version is never listed: a packument carries no words for a version it
  lacks, so npm's refusal reaches the developer through the tarball's
  response and through the refusal `tailrace sync` prints (99).
- **The host's one process serves it.** The serving process
  `tailrace serve` gains a route for any served layout,
  `/<owner>/<name>/<ecosystem>/…`, the owner and name the consuming
  repository's landing remote gives it (114, 49). It answers from the
  layout of the checkout that last named the repository and hands each
  request to the projection bound for that ecosystem, so npm's protocol
  stays in the npm profile (core.1, core.2). Every checkout of the
  repository on the host reaches the same process at
  `http://127.0.0.1:7420/<owner>/<name>/npm/`, and none starts one of its
  own (114, core.36).
- **The projection seam's serve answers one request.** serve is given a
  layout, one request of the ecosystem's own client and a way to ask the
  core for a version the layout lacks, and answers the response; a
  projection whose clients read files answers absent for every request
  (86). This replaces serve's answer of an address, which one process
  per host leaves nothing to do.
- **A version the layout lacks is rendered on the request.** When npm
  asks for a tarball the layout does not hold, the process asks the core
  to render that one version into the layout from the store, under the
  policy `tailrace sync` would use in that checkout, found as render
  finds it: the consuming repository's own where its file writes one,
  else the publisher's layout policy the version's index carries, else
  the given policy of accepted versions (113, C.10). Like any version the
  layout holds, its family dependencies are then held by the core's loop
  (137). The process answers from the layout (115, 50). A refused version
  is answered in the response to npm's own request, `403` with the
  version and the claim found wanting, which npm prints after the status
  line; an absent one `404`, and an unreadable one `502` with the
  source's own words (99, 58, 59).
- **`tailrace sync` makes a served layout reachable.** After the layouts
  are rendered, each layout whose projection serves names its address in
  the answer, and sync names the checkout's repository to the host and
  finds the serving process or starts it, as `tailrace status --page`
  does (114, 132).
- **The process stays while a layout is served.** npm cannot start the
  process, so an install long after a sync must still find it: the
  process ends after its idle time only where no checkout the host's
  file names holds a rendered layout its projection serves, and ends
  once the last such layout, or its checkout, is gone (114).
- **The address npm records names no host.** Every tarball address in a
  packument, which npm writes into `package-lock.json` as `resolved`, is
  the core's fixed name and port with the consuming repository in the
  path, the same on every host and in every checkout (49, core.18).
- **npm is a pinned tool.** Node.js and its npm are in the devenv shell,
  pinned by `devenv.lock`, and the tests that run npm are in the tools
  layer (core.26).

Out of scope, and what each waits on:

- Writing the `artifact/npm/<name>@<version>` pointer at publish, and
  the store's listing of a name's versions (138): the unit
  the-store-names-an-artifact-by-ecosystem-name-and-version, which this
  unit comes after.
- The core's render under a policy, pinned on the seam, the layout's
  place and its record: render-under-a-policy, which this unit comes
  after. This unit adds only the render of one missed version and the
  lock that keeps two renders of one layout apart.
- requires on the projection seam, with the fake's answer and the absent
  answer of a projection with no manifest: render-under-a-policy's fix on
  the bolt, which this unit's code waits on.
- The range a pin carries on the seam, and the core's loop that lists,
  judges and holds through every level (137): cargos-registry, which owns
  them in `render.rs` and runs beside this unit. npm implements its side
  of both, and its range and requires work waits on their landing.
- The serving process itself, its lock, the host's file of repositories
  and `tailrace status --page`: bolt tailrace-the-gate-and-the-channels'
  status across machines, on which this unit's code waits.
- `npm publish` refused in a tailrace repository: the Claude Code
  plugin unit.
- Third-party packages, which stay the public registry's (section 8).

## Capabilities

### New Capabilities

- `npm-projection`: the npm profile of the projection seam: pack with
  `npm pkg set` and `npm pack`, pinned from `package-lock.json` and the
  family ranges `package.json` declares, requires from a held tarball's
  `package.json` with npm's own range rules, render as tarballs and
  packuments, and serve as npm's registry protocol, with a missed version
  asked of the core and a refusal answered in npm's own response.

### Modified Capabilities

- `seam-contracts`: a projection's serve answers one request of its
  client and may ask the core for a version the layout lacks; the
  contract's pinned and requires read as 137 and 140 now say.
- `seam-fakes`: the fake projection serves requests and records the
  versions it asked for.
- `consumer-layouts`: a version a served layout lacks is rendered into
  it under the policy, with the family packages it requires, and one
  layout is rendered by one party at a time.
- `serving-process`: the process answers each served layout under the
  repository's path, from the layout of the checkout that last named it,
  and stays past its idle time while any such layout is served.
- `command-line`: `tailrace sync` names each served layout's address and
  finds or starts the host's serving process.
- `conformance-suite`: the projection group holds a projection to serve
  and the npm projection runs it; the layouts group holds the core to a
  render on a miss; 46, 49, 114 and 115 leave the register; the group's
  range and requires scenarios, cargos-registry's, run over npm's rig.
- `tools-layer`: npm's tests run in the tools layer against the pinned
  npm.

## Impact

- New profile directory `crates/tailrace/src/profiles/projection/npm/`,
  bound beside the others in `profiles/projection/mod.rs` with a store of
  its own to read tarballs through and the host's serving address, by
  which it tells a family scope in the checkout's `.npmrc`.
- The projection trait's serve, and every implementation: the fake, the
  file, image and moon cache profiles, and the conformance rigs.
- `crates/tailrace/src/commands/serve/` routes served layouts and answers
  bytes; `crates/tailrace/src/commands/sync.rs` names and starts the
  process; `tailrace-core`'s `render.rs` gains the miss, which holds a
  version and closes over it by cargos-registry's loop, and the layout's
  lock; `served.rs` the served layout's address and the process's.
- New dependencies: `flate2` and `tar`, to read a tarball's
  `package.json` for the packument and for requires; `sha1`, already in
  the lockfile, for the packument's `shasum`; `nodejs-semver` 4.2, npm's
  own range and version rules ported to Rust, for requires; `tempfile`
  becomes a dependency of the binary. `devenv.nix` gains Node.js.
- The register loses 46, 49, 114 and 115.
- The site's projections page says how npm reads a family package.
- Rules kept: core.1 and core.2 (only the npm profile names npm; the
  process routes by the bound ecosystem's name and the projection never
  reaches the store but through the seam it is handed, nor the policy),
  core.5 (the packager's tarball is never rewritten), core.17 (a
  miss, a range's version and a required one are each judged under the
  policy before it is held, and the core judges, never the projection),
  core.18 and core.36 (one address on every host, one process), core.19 (absent, refused and
  unreadable are three responses), core.21 (no credential written into
  `.npmrc` or kept), core.25 and core.26 (npm pinned and its tests a
  moon task), core.31 (the process renders the core's answer), core.33.

## Touches

```
crates/tailrace/src/profiles/projection/npm/**
crates/tailrace/src/profiles/projection/mod.rs
crates/tailrace/src/profiles/projection/file/mod.rs
crates/tailrace/src/profiles/projection/image/mod.rs
crates/tailrace/src/profiles/projection/moon_cache/mod.rs
crates/tailrace/src/profiles/projection/moon_cache/tests.rs
crates/tailrace/src/bindings/mod.rs
crates/tailrace/src/commands/serve/**
crates/tailrace/src/commands/sync.rs
crates/tailrace/src/commands/status.rs
crates/tailrace/Cargo.toml
crates/tailrace/moon.yml
crates/tailrace/tests/**
crates/tailrace-seams/src/projection.rs
crates/tailrace-seams/src/lib.rs
crates/tailrace-core/src/render.rs
crates/tailrace-core/src/served.rs
crates/tailrace-core/src/lib.rs
crates/tailrace-core/tests/**
crates/tailrace-fakes/src/projection.rs
crates/tailrace-conformance/src/**
crates/tailrace-conformance/tests/**
Cargo.toml
Cargo.lock
devenv.nix
site/src/content/docs/guides/integrations/projections.mdx
```
