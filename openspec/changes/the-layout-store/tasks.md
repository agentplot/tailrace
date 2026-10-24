# Tasks

Every cargo and moon command runs as `devenv shell -- <command>`. This
change runs the conformance suite's store group as
`the-conformance-suite-over-the-fakes` lands it: read
`crates/tailrace-conformance/src/seams/store.rs`, `src/scenario.rs` and
`src/lib.rs`'s section on running a seam's group over a profile before
task 5, and take the names that landed where this design's differ. Read
`crates/tailrace-seams/src/store.rs`, `answer.rs`, `artifact.rs`,
`claim.rs` and `digest.rs`, and the fake store, before starting. It
adds no crate, language, file type or dependency, so no project,
formatter entry or `Cargo.toml` line is added; the profile's tests run
in the `tailrace` project's existing `test` task. No trait, shared
type, fake, core module or conformance scenario changes; a scenario of
the group that cannot pass without one is reported with `crew needs`.

## 1. The layout and its handle

- [x] 1.1 Add `pub mod profiles;` to `crates/tailrace/src/lib.rs`, and write `crates/tailrace/src/profiles/mod.rs` (its doc comment and `pub mod store;`) and `profiles/store/mod.rs` (`pub mod layout;`) (design D7). Verify `devenv shell -- cargo build -p tailrace` passes.
- [x] 1.2 Write `crates/tailrace/src/profiles/store/layout/mod.rs`: `Layout`, its source, `Layout::create` and `Layout::open`, and the check of `oci-layout` every operation begins with (design D1). Verify with unit tests in the module: a layout made holds the `oci-layout`, `index.json` and `blobs/sha256/` the spec names; create refuses a directory holding a file; open of a removed directory is unreadable carrying `File::open`'s words verbatim; an `oci-layout` naming version `2.0.0`, or holding text that is not JSON, is unreadable naming the file.

## 2. Blobs

- [x] 2.1 Write `profiles/store/layout/blobs.rs`: the write through a temporary file and a hard link, held already checked against the bytes on disk, and the read with its digest check (design D2, D3); implement `put_bytes` and the bytes half of `read` on them. Verify with unit tests: the same bytes put twice answer one digest, the second held already, with the file's modification time unchanged; a file altered on disk is refused `Why::OtherBytes` naming both digests on read and on put, and left as found; a digest never put is absent naming it; no temporary file remains after a put.

## 3. Artifacts and indexes

- [x] 3.1 Write `profiles/store/layout/oci.rs`: the image manifest, the image index, the descriptor and the media types and annotation keys of design D4, each to and from the seam's `Manifest`. Verify with unit tests: an artifact's and an index's JSON round-trip to the `Manifest` they came from; the same `Manifest` writes the same bytes twice; an ecosystem with a space in its name is refused naming it.
- [x] 3.2 Implement `put_artifact` and the manifest half of `read` over `blobs.rs`, `oci.rs` and the manifest entry of design D5. Verify with unit tests: an artifact read back answers its metadata and layers in order; an index read back answers its component, version and artifacts in order; a manifest naming a digest not held is absent naming it and writes no blob or entry; the config blob of an artifact reads back as bytes.

## 4. `index.json`: attestations and names

- [x] 4.1 Write `profiles/store/layout/index.rs`: reading `index.json`, and the locked rewrite through a temporary file and a rename that keeps every entry it does not change (design D5). Verify with unit tests: an entry of a media type tailrace never writes survives a rewrite unchanged; a malformed `index.json` is unreadable naming it with `serde_json`'s words and is left as found.
- [x] 4.2 Implement `attach` and `claims_of` over the attestation entry. Verify with unit tests: two attestations answer in the order attached, byte for byte; the same attestation attached twice answers held already and is answered once; attach and claims-of for a digest not held are absent naming it and write nothing.
- [x] 4.3 Implement `name` and `resolve` over the name entry, the check of the expected digest made under the lock. Verify with unit tests: a name made, moved, and held in exactly one entry; a move from where the name is not, and from nothing when it names a digest, refused `Why::NameAt` naming where it is with nothing written; a move to a digest not held absent; a name never made absent naming it.

## 5. The profile's own tests

- [x] 5.1 Write `crates/tailrace/tests/layout_store.rs`: `LayoutRig` and the test that runs the suite's store group over it (design D6). Verify `devenv shell -- cargo test -p tailrace --test layout_store` passes with the group's report naming the layout's profile and every scenario passed.
- [x] 5.2 In the same file, one test per scenario of spec `layout-store` the group does not play (design D6): a layout made; never made by an operation; another tool's entries and blobs untouched; where bytes are held; a blob altered on disk, read and put again; an index read back; the same artifact in two layouts, byte for byte; an artifact's manifest as a registry reads it, with its `index.json` entry; a manifest naming bytes not held; an attestation attached twice; an attestation and claims-of for a digest not held; claims-of in a copy of the layout; a name made and moved with one entry; two handles in two threads moving one name, exactly one succeeding; a name moved to a digest not held; a malformed index. Verify `devenv shell -- cargo test -p tailrace --test layout_store` passes.
- [x] 5.3 Verify `grep -rn 'tailrace_core' crates/tailrace/src/profiles` finds nothing (core.2), and `grep -rniwE 'oci|ecr|ghcr|oras|cosign|moon|sluice|kms|npm|maven|cargo|flux|gh' crates/tailrace/src --exclude-dir=profiles` finds nothing (core.1).

## 6. The site

- [x] 6.1 In `site/src/content/docs/guides/integrations/stores.mdx`, mark the OCI layout on disk `built` and say in its row that an artifact is an OCI manifest, an attestation an entry of `index.json` annotated with its subject, and a name an entry with a ref name moved only from where it is; in `guides/integrations/index.mdx`, replace "Nothing is built yet; every member below is designed" with a sentence saying the OCI layout on disk is built and every other member is designed. Verify the site's moon build task passes in the gate (7.1) and `grep -n 'Nothing is built yet' site/src/content/docs/guides/integrations/index.mdx` finds nothing.

## 7. The gate

- [ ] 7.1 Verify `devenv shell -- moon run ':#check-fast'` passes: format, lint, toolchain, ADR numbers, the conformance suite's coverage, every crate's tests and the site's build.
