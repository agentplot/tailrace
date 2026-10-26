## 1. Dependencies and binding

- [ ] 1.1 Append `flate2`, `tar` and `toml_edit` to `[workspace.dependencies]` in `Cargo.toml` and to `crates/tailrace/Cargo.toml`, changing no existing line; `Cargo.lock` follows. Verify: `moon run ':#check-fast'` passes
- [ ] 1.2 Add `profiles/projection/cargo/mod.rs` with `ECOSYSTEM = "cargo"`, a `CargoProjection` holding the checkout root, a store and the cargo program (a constructor taking the program for tests), and bind it in `profiles::projection::bound`, the bindings binding a second store for it (design D5). Verify: `bindings/tests.rs` asserts `cargo` is among the bound projections, in `check-fast`

## 2. Pack

- [ ] 2.1 Find the output's one `[package]` manifest and refuse none or several naming the output and its files (design D1). Verify: unit tests for none, two, and one
- [ ] 2.2 Read the package's name and workspace root with `cargo metadata`; write the version into `package.version` or `workspace.package.version` with `toml_edit`; run `cargo package --no-verify --allow-dirty --manifest-path <it> --target-dir <temp>`; restore the manifest's bytes on every path; answer the crate's bytes, `cargo` metadata and `same_again` true (design D2, D3). Verify: a tools-layer test in `tests/cargo_projection.rs` packs a workspace-inherited crate in a scratch git repository and checks the manifest bytes are unchanged and the crate's manifest names the version
- [ ] 2.3 Refuse a crate whose own manifest names another version, naming both; answer unreadable with cargo's or the operating system's words when cargo fails or cannot start. Verify: unit test of the check on an in-memory crate; the conformance group's unreachable scenario
- [ ] 2.4 Answer absent naming cache entries for a pack with no version and a render of cache entries, reading nothing. Verify: unit test, and the group's "a projection that packs no cache entries"

## 3. Render and serve

- [ ] 3.1 Read a crate's normalized `Cargo.toml` from its bytes with `flate2` and `tar`, and write its index line as design D4 lists, `registry` from `registry-index` or crates.io's index URL, `features2` and `v` 2 where needed. Verify: unit tests over in-memory crates with a renamed, an optional, a dev, a target and a crates.io dependency and a `dep:` feature
- [ ] 3.2 Render: check every artifact first (ecosystem, one layer, digest, name and version), then write each `.crate` and each whole index file by rename, remove files of versions not given, sort lines by `semver`, and answer the versions held (design D4). Verify: unit tests over a layout store for exactly-the-given, a second render equal, a version dropped, bytes of another digest refused with nothing written
- [ ] 3.3 Serve answers absent naming serve of the cargo projection. Verify: unit test

## 4. Conformance and proof

- [ ] 4.1 `tests/cargo_projection.rs` (feature `tools`): a rig over a scratch git repository holding a crate and a layout store, running every scenario of the projection group. Verify: added to the `tools` task's command in `crates/tailrace/moon.yml`; `moon run ':#check-tools'` passes
- [ ] 4.2 The proof: pack a crate, put it in a layout store, render it, and in a consumer replacing `sparse+https://tailrace.invalid/<owner>/<repo>/` with the layout run `cargo build --offline` under a sealed `CARGO_HOME`; assert success and that `Cargo.lock` records that address and the checksum, and no layout path. Verify: the same tools-layer test file, in `check-tools`
