# Proposal

## Why

Bolt 2 is usable when `tailrace publish` on a laptop writes a version
into an OCI layout on disk and a second machine verifies a copy of that
layout (phase-1-bolts.md, Bolt 2). Today the only store is the fake in
memory, so nothing a publish offers outlives the process. This change
binds the store seam (B.1) to a directory on disk, the profile C.2
names, with no server, so the laptop unit has a store to write into and
a copy of the directory carries every artifact and attestation with it.

## What Changes

- **A layout on disk is a store.** A new profile under
  `crates/tailrace/src/profiles/store/layout/` implements the one store
  interface over a directory in the OCI image layout: the `oci-layout`
  file, `index.json`, and every blob at `blobs/sha256/<hex>`, read and
  written with no server (C.2, ADR 0003). It implements the interface
  and nothing of the core (core.2).
- **Every operation is a file write under the digest's path.** Bytes
  are written to the path their own digest names, and to no other
  (core.5). An artifact is an OCI image manifest whose artifact type
  names the ecosystem, whose config is the artifact's metadata and whose
  layers are its bytes; an index is an OCI image index naming each
  artifact. Both are written under their own digest like any bytes, so
  what the layout holds is a real OCI layout another tool can copy to a
  registry (C.2, ADR 0003's consequences).
- **The same bytes put twice are held once.** A second put of bytes, of
  an artifact or of an attestation writes nothing and answers the same
  digest, held already (5, 66). A blob is never overwritten: bytes on
  disk whose digest is not their path's are refused naming both, never
  rewritten (2, 73, core.5).
- **Attestations are index entries with a subject annotation.** attach
  writes the envelope under its own digest and adds one entry to
  `index.json` naming it, annotated with the digest it is about;
  claims-of reads the entries annotated with a digest, in the order
  they were attached. An attestation is never held for a digest the
  layout does not hold (8, 68, 69, core.27).
- **A name is an index entry with a ref name, moved only from where it
  is.** name reads where the name is and moves it only from the
  expected digest, under a lock on the layout, so two writers cannot
  both move it; a refused move is reported naming where it is, never
  retried (70, 71, core.6).
- **Absent, refused and unreadable kept apart.** A digest or name the
  layout does not hold is absent; a layout that is gone, unreadable or
  malformed is unreadable, carrying the operating system's or the
  parser's words verbatim (58, 59, core.19). The layout is never made
  by an operation that finds it missing.
- **The store's conformance scenarios pass against a layout in a
  temporary directory.** The profile's own tests run the suite's store
  group over a fresh layout per scenario, and test what the group does
  not: a blob altered on disk, two handles on one layout, a layout
  other tools wrote, and the layout's files as the OCI specification
  lays them out.
- **The site says the layout is built.** The stores page marks the OCI
  layout on disk built and says how it holds artifacts, attestations and
  names; the integrations index no longer says nothing is built.

Out of scope, and what each waits on:

- Binding the layout for a host, by its path in the host's bindings
  file, and `tailrace publish`, `status` and `verify` over it: the
  laptop unit of bolt 2.
- The registry store, and a layout synced to a registry, with the
  referrers an attestation becomes there (C.1, 103, 109): the registry
  store unit of bolt 3.
- The store group itself, its rig interface and the register: the
  conformance suite unit, `the-conformance-suite-over-the-fakes`, whose
  store group this change runs and does not change.
- Removing anything from a layout, a garbage collection of unreferenced
  blobs: no unit of phase 1.

## Capabilities

### New Capabilities

- `layout-store`: the store seam bound to an OCI image layout on disk:
  where bytes, artifacts, indexes, attestations and names are held,
  what each operation writes and answers, and how a layout that cannot
  be read is told apart from a digest it does not hold.

### Modified Capabilities

None.

## Impact

- `crates/tailrace/src/profiles/store/layout/`: the profile, new.
- `crates/tailrace/src/profiles/mod.rs`, `profiles/store/mod.rs` and
  one `pub mod profiles;` line in `crates/tailrace/src/lib.rs`: the
  module path to the first profile. The other bolt 2 units add their
  seams beside it in the same two files.
- `crates/tailrace/tests/layout_store.rs`: the store group over the
  layout, and the profile's own tests.
- No dependency is added: digests come from `tailrace_seams::Digest`,
  JSON from `serde_json`, and the lock and the atomic writes from the
  standard library.
- `site/src/content/docs/guides/integrations/stores.mdx` and
  `guides/integrations/index.mdx`.
- No trait, shared type, fake, core module or conformance scenario
  changes.

## Touches

```
crates/tailrace/src/profiles/mod.rs
crates/tailrace/src/profiles/store/mod.rs
crates/tailrace/src/profiles/store/layout/**
crates/tailrace/src/lib.rs
crates/tailrace/tests/layout_store.rs
site/src/content/docs/guides/integrations/stores.mdx
site/src/content/docs/guides/integrations/index.mdx
```
