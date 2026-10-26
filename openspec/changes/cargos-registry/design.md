## Context

The projection seam (B.6, `crates/tailrace-seams/src/projection.rs`) has
pack, render, serve and pinned. Three units this change comes after shape
what it implements:

- **render under a policy** adds pinned (137), the core's render that
  reads a checkout's pins, resolves each through
  `artifact/<ecosystem>/<name>@<version>` to the version's index and the
  artifact found in it by its metadata (138), judges it under the
  layout's policy (113, C.10), and hands the projection exactly the
  satisfied artifacts at `<working tree>/.tailrace/layouts/<ecosystem>/`;
  `tailrace sync` runs it.
- **the store-naming unit** makes a publish leave that name.
- **npm, rendered and served** (its design, D1 to D4, D10, D11) makes
  serve answer one request:

  ```rust
  pub struct Request { pub method: String, pub path: String, pub base: String }
  pub struct Response { pub status: u16, pub content_type: String, pub body: Vec<u8> }
  pub type Miss<'a> = &'a dyn Fn(&Pinned) -> Answer<()>;
  fn serve(&self, layout: &Layout, request: &Request, miss: Miss<'_>) -> Answer<Response>;
  ```

  `path` is below the layout's prefix, `base` the address of that prefix
  as the client reached it with no trailing `/`, and `miss` asks the core
  to render one version into the layout. The host's one process routes
  `/<owner>/<name>/<ecosystem>/<rest>` to the projection bound for that
  ecosystem in the checkout that last named the repository; `tailrace
  sync` probes each layout's serve with `path` empty and a miss that
  answers absent, reading `Err(absent "serving")` as serving none, and
  starts the process where one serves; the process stays while a served
  layout exists (114). `served::layout_address(repository, ecosystem)`
  spells `http://127.0.0.1:7420/<owner>/<name>/<ecosystem>/`.

What is built here already, from the change's first version: the
profile, its binding with a store of its own, render writing a crate per
version and an index file per name, the index line, the race check after
pack, and the conformance rig. C.11's Cargo row now reads: pack the crate
file with `cargo package --no-verify` after the component's version is
set on every crate of the component, a path dependency on a sibling
rewritten to that version's requirement as cargo does (110); render a
sparse registry under the served address, the same on every host (49,
114), named `tailrace` in the checkout's cargo configuration; serve from
the host's one process.

Vendor facts, checked against cargo 1.98.1 (the devenv's) and Cargo's
reference (registries, registry index, `cargo package`):

- `cargo package` writes every member with the fixed time 2006-07-23 and
  owner 0/0; two packs of one tree gave identical bytes. In a git
  checkout it writes `.cargo_vcs_info.json` with the HEAD sha1 and
  `path_in_vcs`, the package's directory in the repository, and refuses
  an edited manifest without `--allow-dirty`.
- A path dependency with no `version` is refused when packaging ("all
  dependencies must have a version requirement specified when
  packaging"); with one and no `registry`, the packed crate names it
  from crates.io.
- With `registry = "tailrace"` beside `path` and `version`, packing the
  crate alone fails to resolve the sibling from the registry ("no
  matching package named `fam-b` found, location searched: `tailrace`
  index"). Packing it together with the sibling, `cargo package
  --registry tailrace -p fam-a -p fam-b`, resolves the sibling from
  cargo's own overlay of the crates being packed: the packed manifest
  names `[dependencies.fam-b] version = "1.2.3" registry-index =
  "<tailrace's index>"`, and the crate's `Cargo.lock` names the sibling
  with that source and the sibling crate's checksum. Without `--offline`
  cargo first fetches the registry's `config.json` and fails where
  nothing answers; with it, it reaches no registry. `--registry` needs
  `[registries.tailrace]` in cargo's configuration, found from the
  directory cargo runs in.
- A sparse registry: Cargo fetches `<index>/config.json` (`dl`, the
  download template with `{crate}` and `{version}`; `api` absent for a
  registry that takes no publish), then each name's index file at
  `1/<n>`, `2/<n>`, `3/<c>/<n>` or `<ab>/<cd>/<n>`, the name lower-case,
  then each crate at `dl`. A 404 on an index file is "not found". A
  non-success on a download fails the build printing `got <status>`,
  then `body:` and the body. An index line's dependency with `registry`
  null is of the registry serving the line. Run against a test server,
  `cargo build` resolved two crates so and wrote `source =
  "sparse+<index>"` and each crate's SHA-256 as `checksum`, and `cargo
  build --locked` refuses where a checksum differs.
- `Cargo.lock` is TOML at the workspace root: `version` (4 today) and
  one `[[package]]` per package, with `name`, `version`, `source` and
  `checksum`. `source` is `registry+<url>` or `sparse+<url>` for a
  registry, `git+<url>#<sha1>` for git, and absent for a workspace member
  or a path dependency.

## Goals / Non-Goals

**Goals:** pack, pinned, render and serve for Cargo on the seams the
units above land, held to the whole projection group; `cargo build
--locked` proven against the host's serving process after `tailrace
sync`.

**Non-Goals:** choosing versions, the policy, the layout's place and
`tailrace sync` (render under a policy); the process, its route and its
lifetime (npm, rendered and served); writing a checkout's cargo
configuration; a manifest's range where no lockfile pins a name (137).

## Decisions

### D1. The output names its package by the crate file it left

The build output a family repository declares for a crate is its
packaging task, and the output's files hold the `.crate` file the task
left. The projection takes the one file among them ending `.crate`,
reads its normalized `Cargo.toml` and its `.cargo_vcs_info.json` with
`flate2` and `tar`, and packs the package of that name whose manifest is
`<checkout>/<path_in_vcs>/Cargo.toml`, checking with `cargo metadata
--no-deps --format-version 1 --manifest-path <it>` that the package
there has that name, and reading the workspace's root and members from
the same answer. None or several crate files are refused naming the
output and its files; a crate with no `path_in_vcs`, packed outside git,
is refused naming the crate. Alternative: a `Cargo.toml` among the
output's files; refused because the design declares the output as the
packaging task that leaves the crate file.

### D2. The component's version on every crate it reaches by path

From the packed package, the projection follows each dependency in
`dependencies`, `build-dependencies` and each `target.<cfg>` table of
them that names a `path` to a member of the workspace, written in the
package's manifest or inherited from `workspace.dependencies`, and so on
from each member it reaches. These are the siblings: the crates of the
component the packed crate depends on, versioned with it (110). With
`toml_edit`, each edit in the file and the table where its value is
written, the rest of every file keeping its bytes:

- the version is set on the packed package and every sibling, in
  `package.version`, or in `workspace.package.version` for one that
  writes `version.workspace = true`;
- each dependency on a sibling gets `version = "<version>"`, the
  requirement Cargo reads a bare version as, and `registry =
  "tailrace"`, so the packed crate names the sibling in the registry its
  consumers read.

Then `cargo package --no-verify --allow-dirty --offline --registry
tailrace --manifest-path <workspace root>/Cargo.toml -p <package> -p
<sibling>…` runs in the package's directory with `--target-dir` a fresh
temporary directory, and the projection answers
`<target>/package/<name>-<version>.crate`; the siblings' crates are left,
each answered by its own output's pack. Every edited file's original
bytes are written back before pack answers, on every path. Dev
dependencies are left as written: cargo drops a path dev dependency with
no version from the packed crate.

The race check stays: the crate's own manifest is read and a crate
naming another name or version is refused naming both, since every pack
of a revision edits one checkout.

Why `--offline`: the registry `tailrace` is the host's serving process,
which serves consumers' layouts and holds nothing a publish needs; the
siblings come from cargo's overlay of the crates being packed, and the
third-party crates the crate's lockfile names come from the cache the
build just filled. Alternative: rewrite only the dependency and pack the
crate alone; refused because cargo then looks for the sibling in the
registry, where it is not yet.

### D3. same_again is true

The bytes are a function of the tree, the version, the revision, the
checkout's `[registries.tailrace]` address, which is the publishing
repository's own and the same on every host (49), and the cargo that
ran. A host with another cargo may pack other bytes; a repeat publish
there is refused naming the artifact (18), the clause's own remedy. The
promise is the packager's (C.11).

### D4. Pinned reads the lockfile and nothing else

Pinned reads `Cargo.lock` at the checkout's root with `toml_edit` and
answers, for each `[[package]]` whose `source` begins `registry+` or
`sparse+`, its `name` and `version` as written (137). A package with no
source is the checkout's own, and one from git is pinned by commit; both
are left out. crates.io's pins are answered like any other: the core
finds no name for them in the store. No `Cargo.lock` answers absent
naming it; a file that is not TOML, or a package with no name or version,
is refused naming the file and carrying the parser's words (58, 59).
Pinned runs no cargo, since `cargo metadata` may write the lockfile, and
writes nothing. A name the manifest declares and no lockfile pins yet is
not answered, as npm's pinned answers none (proposal).

### D5. Render writes the registry's files, from the store alone

Render first checks every artifact (ecosystem `cargo`, one layer, bytes
of that digest, the crate's manifest naming the metadata's name and
version, no two digests for one name and version), then writes, under
the place given:

```
index/<path>                 each name's index file, <path> as the sparse protocol names it
<name>-<version>.crate       each crate
```

Crates are written beside their place and renamed onto it; each index
file is written whole and renamed. Crate and index files of versions not
given are removed, so the layout holds exactly what it was given. Index
lines are sorted by `semver` (23). A relative place is under the checkout
the projection is bound in.

Index line, from the normalized manifest, as built: `name`, `vers`,
`deps`, `cksum`, `features`, `features2` and `v`, `links`,
`rust_version`, `yanked` false, keys in a fixed order. A dependency's
`registry` is null where its `registry-index` is a served Cargo address,
`sparse+http://127.0.0.1:7420/<owner>/<name>/cargo/` for any owner and
name, its host and port the core's (`served`); the dependency's
`registry-index` where it is another registry; crates.io's index where
it names none. Why null for every family repository's address: the
packed crate names its sibling at the publisher's address, a consumer
reads at its own, and its own resolves any family crate by name and
version (138), so the dependency is one of the registry serving the
line. The layout names no address: `config.json` is the server's (D6),
so one layout serves under any base.

### D6. Cargo's serve

The request's `path`, below `/<owner>/<name>/cargo/`:

- `config.json`: `{"dl":"<base>/crates/{crate}/{version}/download"}`,
  `200` `application/json`, with no `api`, since the registry takes no
  publish.
- an index path (`1/<n>`, `2/<n>`, `3/<c>/<n>`, `<ab>/<cd>/<n>`, `<n>`
  lower-case and its prefix matching it): the layout's
  `index/<path>`, `200` `text/plain`; no such file is `404`. An index
  file never asks the core: the store names a crate only with its
  version (138).
- `crates/<name>/<version>/download`: the layout's
  `<name>-<version>.crate`, `200` `application/octet-stream`; one the
  layout lacks calls `miss` once with `Pinned { name, version }` and, on
  `Ok`, answers the crate now in the layout.
- anything else, `path` empty among them, `404`; a method other than
  `GET` or `HEAD`, `405`.

Every answer that is not the bytes or file asked is `text/plain`, the
`Not`'s own rendering, the source's words verbatim: `404` absent, `403`
refused, `502` unreadable (58, 59, 99); cargo prints it under `body:`.
Every request is answered with a response, so sync's probe reads the
projection as serving (npm's D10); `Err` is kept for a layout whose
files cannot be read for another reason than their absence.

### D7. Bound with a store of its own

`profiles::projection::bound` takes a second store for Cargo; the
bindings bind the store once more for it, as they bind `cache` for the
moon cache, since a `Box<dyn Store>` is not shared (core.2).

### D8. Tests, and the proof as one loop

Unit tests in the profile build crates in memory with `flate2` and
`tar` for the index line, render and serve, and read lockfiles written as
text for pinned; they run in `check-fast` with no cargo needed. The
conformance group and the proof run cargo, so they sit behind the
`tools` feature in the `tools` task (check-tools), with a sealed
`CARGO_HOME` and `HOME` per test:

- `tests/cargo_projection.rs`: the projection group over a scratch git
  repository holding a workspace of two sibling crates, its rig serving
  `crates/<name>/<version>/download` as the request for a version.
- `tests/cargo_served.rs`: the publisher's workspace, `fam-b` and
  `fam-a` depending on it by path, each sibling dependency written with
  a placeholder version and `registry = "tailrace"`, `.cargo/config.toml`
  naming `tailrace` at its served address; the build's packaging task
  stood in for by `cargo package --workspace --no-verify --offline
  --registry tailrace`; both packed at one version, put in a layout store
  with their index, claims and `artifact/cargo/<name>@<version>` names as
  npm's served test puts its package. A consumer checkout whose
  `.cargo/config.toml` names `tailrace` at
  `sparse+<the test's base>/<owner>/<name>/cargo/`, depending on `fam-a`
  in that registry, with a `Cargo.lock` written as text pinning both
  crates at that source with the packed crates' checksums; `tailrace
  sync` renders the layout and the process at the test's address serves
  it; `cargo build --locked` succeeds and leaves the lockfile as written.
  A version the policy refuses fails the build with the refusal's words
  under cargo's `body:`.

## Risks / Trade-offs

- [A crate of another component reached by path] → it is taken as a
  sibling and given this component's version, which its own component's
  publish does not give it (124); the pack is given one output and
  cannot tell a component's crates from the workspace's. Named for the
  design (Open questions).
- [A pack offline on a host whose cargo cache lacks a third-party
  crate] → unreadable with cargo's words; the build has just run on the
  host, so the cache is warm.
- [A crate whose `publish` leaves `tailrace` out, or a checkout naming no
  `tailrace` registry] → cargo refuses, and pack answers in cargo's
  words.
- [A lockfile pin the layout's policy refused] → the index Cargo reads
  holds no line for it, so Cargo's own words say no version matches;
  `tailrace sync` printed the refusal when it rendered, and a download
  of a refused version answers it in Cargo's error. Named for the
  design.
- [In-place edits of a shared checkout] → the race check refuses rather
  than offers a wrong crate.
- [A crates.io crate with a family crate's name and version] → resolved
  as the family's by name and version alone (138); Cargo still reads it
  only for a dependency naming the registry `tailrace`.

## Open Questions

For the design agent, none of which changes what this change builds:

- Which crates are a component's when its outputs share a workspace with
  another component's: D2 takes every crate the packed one reaches by
  path.
- How a refusal of a version a lockfile pins reaches Cargo, which reads
  a name's index file before it asks for any crate, so the request that
  would carry the refusal is never made.
