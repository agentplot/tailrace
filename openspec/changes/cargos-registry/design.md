## Context

The projection seam (B.6, `crates/tailrace-seams/src/projection.rs`)
has pack, render and serve. The file and image projections pack and
render nothing; the moon cache renders cache entries by reading bytes
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
- An index line's `registry` null means the registry holding the index;
  the reference says a registry other than crates.io writes crates.io's
  dependencies as `https://github.com/rust-lang/crates.io-index`.

## Goals / Non-Goals

**Goals:** pack, render and serve for Cargo, bound and held to the whole
projection group; `cargo build` proven against a rendered layout.

**Non-Goals:** choosing versions, the policy, the per-lockfile place and
`tailrace sync` (the render-under-a-policy unit); the declared registry
address convention; a consumer's configuration.

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

### D4. Render writes the registry exactly, from the store alone

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

### D5. Bound with a store of its own

`profiles::projection::bound` takes a second store for Cargo; the
bindings bind the store once more for it, as they bind `cache` for the
moon cache, since a `Box<dyn Store>` is not shared (core.2).

### D6. Tests

Unit tests in the profile build crates in memory with `flate2` and
`tar` and check the index line and the layout. The conformance group
and the proof run cargo, so they sit in `tests/cargo_projection.rs`
behind the `tools` feature, in the `tools` task (check-tools), with a
sealed `CARGO_HOME` per test.

## Risks / Trade-offs

- A pack needing the network to write the crate's `Cargo.lock` fails on
  an offline host → unreadable with cargo's words; the build has just
  run, so the index cache is warm.
- In-place edits of a shared checkout → D2's check refuses rather than
  offers a wrong crate.

## Open Questions

For the design agent:

- How a family repository declares a crate as an output: D1 asks for the
  crate's `Cargo.toml` among the task's declared outputs, which moon
  calls files a task creates.
- The registry address a family crate is declared under, the same on
  every host and naming no host (49), and whether one local registry
  stands for several family repositories' registries. The tests use
  `sparse+https://tailrace.invalid/<owner>/<repo>/`.
- Crates of one component depending on each other by path: whether the
  edit also sets those requirements to the version.
