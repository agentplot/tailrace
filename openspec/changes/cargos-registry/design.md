## Context

The projection seam (B.6, `crates/tailrace-seams/src/projection.rs`)
has pack, render and serve; render under a policy, which this change
comes after, adds pinned (137) with the fakes and the core's render that
reads a checkout's pins, resolves each through the name the store holds
for it, and renders what it found. The store-naming unit, which this
change also comes after, makes a publish leave that name:
`artifact/<ecosystem>/<name>@<version>`, a pointer to the digest of the
version's index, in which the artifact of that ecosystem and name is
found by its metadata and beside which the claims about the version are
read (138). The core resolves a pin so; the projection is handed the
artifacts it found. The file and image projections pack and render
nothing; the moon cache renders cache entries by reading bytes
through a store it is handed (`profiles/projection/moon_cache/`). An
artifact's digest names a manifest (`Manifest::Artifact { meta, layers
}`), whose layer is the packed bytes. C.11 fixes Cargo's row: pack the
crate file through a manifest edit and `cargo package --no-verify`,
render a local registry per lockfile, serve nothing (source
replacement). ADR 0006 records why.

Vendor facts, checked against cargo 1.98.1 (the devenv's) and Cargo's
reference (source replacement, registry index, `cargo package`):

- `cargo package` writes every member with the fixed time 2006-07-23 and
  owner 0/0; two packs of one tree, with mtimes touched between, gave
  identical bytes.
- In a git checkout with the manifest edited it refuses without
  `--allow-dirty`; with it, `.cargo_vcs_info.json` records the HEAD sha1,
  `"dirty": true` and `path_in_vcs`, all fixed for one revision.
- The normalized manifest in the crate resolves workspace inheritance,
  writes a dependency from another registry with `registry-index =
  "<url>"`, and one from crates.io with no registry key.
- A local registry is a directory of `<name>-<version>.crate` files and
  an `index/` laid out as the crates.io index (`1/`, `2/`, `3/<c>/`,
  `<ab>/<cd>/`, lower-case name). `[source.<x>] registry = "<url>"
  replace-with = "<y>"` with `[source.<y>] local-registry = "<path>"`
  replaces an alternative registry; `cargo build --offline` resolved a
  crate so, and the lockfile recorded `source = "<url>"` and the
  checksum, never the path.
- `Cargo.lock` is TOML at the workspace root: `version` (4 today) and
  one `[[package]]` per package, with `name`, `version`, `source` and
  `checksum`. `source` is `registry+<url>` or `sparse+<url>` for a
  registry, `git+<url>#<sha1>` for git, and absent for a workspace member
  or a path dependency; `checksum` is the `.crate` file's SHA-256 hex.
  `cargo build --locked` refuses where a checksum differs from the crate
  the source gives.
- An index line's `registry` null means the registry holding the index;
  the reference says a registry other than crates.io writes crates.io's
  dependencies as `https://github.com/rust-lang/crates.io-index`.

## Goals / Non-Goals

**Goals:** pack, pinned, render and serve for Cargo, bound and held to
the whole projection group; `cargo build --locked` proven against a
layout rendered from a lockfile's pins.

**Non-Goals:** choosing versions, the policy, the per-lockfile place and
`tailrace sync` (render under a policy); writing the name pointers (the
store-naming unit); the declared registry address convention; a
consumer's configuration.

## Decisions

### D1. The output names its package by its manifest

The output's files hold the package's `Cargo.toml`, the one among them
that declares `[package]`; the projection reads the package's name and
workspace root from `cargo metadata --no-deps --format-version 1
--manifest-path <it>`, the ecosystem's own answer, not a walk of the
tree. The artifact's name is the package's name (4), which a
consumer's dependency names. Alternative: locate the package by the
output's name in the workspace holding any file the output left; refused
because a shared target directory may lie outside the workspace.

### D2. The edit is undone, and the crate is checked for the version

The version is written with `toml_edit` into the one value that holds
it: `package.version`, or `workspace.package.version` in the workspace
root where the package writes `version.workspace = true`; the rest of
the file keeps its bytes. Cargo packs with `--target-dir` a fresh
temporary directory, so nothing is left in the checkout's `target/`.
The original bytes are written back before pack answers, on every path.
The build's checkout is shared by every pack of the revision, so the
projection then reads the crate's own `Cargo.toml` and refuses a crate
that names another version: two packs racing over one manifest are
refused, never offered wrong. Alternative: copy the workspace out of
git; refused because cargo's file list outside git differs from the one
the landing checked.

### D3. same_again is true

The bytes are a function of the tree, the version, the revision and the
cargo that ran (the fact above). A host with another cargo may pack
other bytes; a repeat publish there is refused naming the artifact (18),
which is the clause's own remedy. The promise is the packager's (C.11).

### D4. Pinned reads the lockfile and nothing else

Pinned reads `Cargo.lock` at the checkout's root with `toml_edit` and
answers, for each `[[package]]` whose `source` begins `registry+` or
`sparse+`, its `name` and `version` as written (137). A package with no
source is the checkout's own, and one from git is pinned by commit, not
by a version a registry holds; neither is a pin a layout can answer, so
both are left out. crates.io's pins are answered like any other: the
core finds no name for them in the store and Cargo fetches them as it
always does. No `Cargo.lock` answers absent naming it; a file that is not
TOML, or a package with no name or version, is refused naming the file
and carrying the parser's words (58, 59). Pinned runs no cargo, since
`cargo metadata` may write the lockfile, and writes nothing.
Alternative: answer only pins whose source is the family registry's
address; refused because 138 has a pin resolved by name and version
alone, and the address convention is not yet the design's.

### D5. Render writes the registry exactly, from the store alone

Render first checks every artifact (ecosystem `cargo`, one layer, bytes
of that digest, the crate's manifest naming the metadata's name and
version, no two digests for one name and version), then writes. Crates
are written beside their place and renamed onto it; each index file is
written whole and renamed. Crate and index files of versions not given
are removed, so the layout holds exactly what it was given (47 is kept
by whoever chooses the set). Index lines are sorted by `semver` (23).
A relative place is under the checkout the projection is bound in, as
the moon cache's is. Reading a crate's manifest uses `flate2` and `tar`
to read one member; nothing is extracted to disk.

Index line, from the normalized manifest: `name`, `vers`, `deps` (from
`dependencies`, `dev-dependencies`, `build-dependencies` and each
`target.<cfg>.*`, with `req`, `features`, `optional`,
`default_features`, `target`, `kind`, `package` where renamed, and
`registry`: the dependency's `registry-index`, else crates.io's index
URL), `cksum` (the crate's SHA-256 hex), `features` and `features2` (a
feature naming `dep:` or `name?/` goes to `features2`, and `v` is 2
where it is not empty), `links`, `rust_version`, `yanked` false. The
JSON is written with keys in a fixed order, one line per version.

### D6. Bound with a store of its own

`profiles::projection::bound` takes a second store for Cargo; the
bindings bind the store once more for it, as they bind `cache` for the
moon cache, since a `Box<dyn Store>` is not shared (core.2).

### D7. Tests, and the proof as one loop

Unit tests in the profile build crates in memory with `flate2` and
`tar` and check the index line and the layout, and read lockfiles
written as text for pinned. The conformance group and the proof run
cargo, so they sit in `tests/cargo_projection.rs` behind the `tools`
feature, in the `tools` task (check-tools), with a sealed `CARGO_HOME`
and `HOME` per test.

The proof runs the loop a consumer's build takes: pack a crate, put its
artifact in a layout store under its digest, put a version's index
naming it beside an artifact of another ecosystem, and name the index at
`artifact/cargo/<name>@<version>` as the store-naming unit's publish
does; render the crate once so the consumer, depending on it through a
registry replaced by the layout, writes its `Cargo.lock` offline; remove
the layout; read the consumer's pins with pinned, resolve each with the
core's resolution those units land (the name to the index, the crate
found in it by its metadata), render what resolved; and run `cargo build
--offline --locked`. It passes only where the crate rendered is the
crate whose checksum the lockfile recorded, so it proves 87 and 138
together. The claims beside the index are render under a policy's to
read and prove; the proof renders no policy. The
registry the test declares is `sparse+https://tailrace.invalid/cargo/`,
naming no repository, as 138 has a consumer know nothing of the
publisher's.

## Risks / Trade-offs

- A pack needing the network to write the crate's `Cargo.lock` fails on
  an offline host → unreadable with cargo's words; the build has just
  run, so the index cache is warm.
- In-place edits of a shared checkout → D2's check refuses rather than
  offers a wrong crate.
- A crates.io crate with a family crate's name and version would resolve
  to the family's → the core's resolution by name and version alone is
  the design's (138); Cargo still reads it only through the registry the
  consumer replaced.

## Open Questions

For the design agent:

- How a family repository declares a crate as an output: D1 asks for the
  crate's `Cargo.toml` among the task's declared outputs, which moon
  calls files a task creates.
- The registry name and address a family crate's consumers declare, the
  same on every host and naming no host (49). With a pin resolved by
  name and version alone (138), one address can stand for every family
  repository's crates; the design names neither it nor the registry's
  name in `.cargo/config.toml`. The tests use
  `sparse+https://tailrace.invalid/cargo/`.
- Crates of one component depending on each other by path: whether the
  edit also sets those requirements to the version.
