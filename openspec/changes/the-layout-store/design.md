# Design

## Context

The store seam is `tailrace_seams::Store` (`crates/tailrace-seams/src/store.rs`):
seven synchronous methods taking `&self`, each answering
`Answer<T> = Result<T, Not>`, with `Not` one of absent, refused or
unreadable naming a `Source` (seam and profile). `put_artifact` takes a
`Manifest` (an artifact's `ArtifactMeta` and layer digests, or an
`IndexMeta`), and `read` answers `Held::Bytes` or `Held::Manifest`, the
one place the signature says more than clause 72 (the seams' design
D4). `attach` answers the attestation's own digest, `Envelope::digest()`,
the digest of the envelope's canonical JSON. The fake store
(`crates/tailrace-fakes/src/store.rs`) is the behaviour the core is
tested against.

The conformance suite (`the-conformance-suite-over-the-fakes`, in code
beside this unit) gives the store group in
`tailrace_conformance::seams::store`: `GROUP`, `scenarios()`, and the
`StoreRig` trait a profile's tests implement (`store()` and
`unreachable()`, which makes the store unreachable and answers the
words every operation must then carry), run with
`tailrace_conformance::run(..).assert()`. Its register names no clause
for this unit: 66 to 73 are cited by the store group over the fakes
already, so this change adds no scenario and takes nothing out of the
register.

There is no `crates/tailrace/src/profiles/` yet; this is the first
profile. The OCI image layout specification 1.1 was read for this
design: `oci-layout` must exist and hold `imageLayoutVersion`;
`index.json` must exist and be an image index; the content of
`blobs/<alg>/<encoded>` must match its digest; blobs no entry names are
allowed; unknown media types in `index.json` must not be an error; ref
names are the `org.opencontainers.image.ref.name` annotation on
`index.json` descriptors, with no grammar; other files may sit in the
layout directory.

## Goals / Non-Goals

**Goals:**

- A directory any OCI tool reads as an image layout, so bolt 3 can sync
  it to a registry and a second machine can read a copy.
- Every write either whole or absent on disk, and every move of a name
  a compare-and-swap that two processes cannot both win.
- The suite's store group passing over the layout with no change to the
  suite, the seam or the fakes.

**Non-Goals:**

- Removing blobs or entries, compaction, or any garbage collection.
- Digests other than sha256 (the seams' `ALGORITHM`).
- Wire DSSE: the envelope is held in the seam's own canonical form
  (D4), and how it is written for a registry is bolt 3's.
- Making a layout from the bindings, or choosing where it lives on a
  host: the laptop unit.

## Decisions

### D1. One `Layout` handle, opened or made, naming its path

`profiles/store/layout/mod.rs` holds `pub struct Layout { root, source }`
with `Layout::create(path) -> Answer<Layout>`, which makes the layout in
an empty or missing directory (the directory, `oci-layout` with
`{"imageLayoutVersion":"1.0.0"}`, `index.json` as an image index with
no manifests, `blobs/sha256/`) and refuses a directory that already
holds files, and `Layout::open(path) -> Answer<Layout>`, which only
checks `oci-layout`. `impl Store for Layout`. The source is
`Source::new(Seam::Store, format!("OCI layout {}", root.display()))`, so
every answer names the layout by its path. Submodules: `blobs.rs` (D2,
D3), `oci.rs` (D4) and `index.rs` (D5).

Every operation begins by opening `<root>/oci-layout` with
`File::open` and reading it, so a layout that is gone or unreadable is
answered unreadable naming that path, carrying the `io::Error`'s
`to_string()` verbatim, before anything is read or written. A version
other than `1.0.0`, or text that is not that JSON object, is unreadable
naming `oci-layout` with the parser's words. Nothing an operation does
makes a directory or an `oci-layout` (the spec's "A layout is never
made by an operation").

*Alternative:* an operation that makes the layout when it finds none.
Rejected: a layout removed or unmounted would be silently replaced by
an empty one, and the suite's unreachable scenario requires unreadable.

### D2. A blob is written once, whole, under its own digest

`blobs/sha256/<hex>`, the hex of `Digest::of(bytes)`; nothing else is
ever written under `blobs/`. A write goes to a temporary file in the
layout's root named `.tailrace-<pid>-<counter>.tmp` (an extra file the
spec allows), is synced, then `std::fs::hard_link`ed to its path, and
the temporary file removed. A hard link never replaces a file, so two
writers of the same bytes cannot clobber each other and a reader never
sees part of a blob: when the link finds the path taken, the blob was
held already (D3).

*Alternative:* `rename`, which replaces whatever is at the path, so a
second writer would rewrite a blob another reader holds open (core.5).

### D3. Held already means the bytes on disk were checked

When a blob's path is taken, put reads the file and compares its digest
with the digest asked: equal answers `held_already: true` and writes
nothing (5, 66); different is refused
`Why::OtherBytes { expected, answered }` and the file is left as found
(2, core.5). read checks the same way and refuses alike rather than
answer bytes that are not their digest's (73). A missing blob is absent
naming the digest; any other I/O error is unreadable naming the file
with the operating system's words.

*Alternative:* trust the path, or compare sizes. Rejected: a layout is
files a person or a sync can alter, and the specification makes the
match a must.

### D4. The OCI forms, and the keys tailrace names

`oci.rs` maps the seam's types to the OCI JSON and back, each written
through one `serde` struct whose field order is fixed and whose
annotations are a `BTreeMap`, so the same `Manifest` always writes the
same bytes on every host and so the same digest (1, 5):

- **An artifact** is an image manifest: `schemaVersion: 2`, `mediaType`
  `application/vnd.oci.image.manifest.v1+json`, `artifactType`
  `application/vnd.agentplot.tailrace.artifact.<ecosystem>`, `config`
  the canonical JSON of its `ArtifactMeta` written as a blob with media
  type `application/vnd.agentplot.tailrace.meta.v1+json`, and one layer
  per layer digest, in order, with media type
  `application/octet-stream` and the blob's size on disk. read takes
  the metadata from the config blob, never from `artifactType`.
- **An index** is an image index: `schemaVersion: 2`, `mediaType`
  `application/vnd.oci.image.index.v1+json`, `artifactType`
  `application/vnd.agentplot.tailrace.index`, annotations
  `io.github.agentplot.tailrace.component` and
  `org.opencontainers.image.version`, and one descriptor per artifact,
  in order, naming its manifest by digest, media type and size, its
  `artifactType`, and annotations `io.github.agentplot.tailrace.ecosystem`,
  `io.github.agentplot.tailrace.name` and
  `org.opencontainers.image.version`, from which read rebuilds the
  `IndexMeta` through `IndexMeta::new` without opening each artifact.
- **An attestation** is the envelope's canonical JSON
  (`tailrace_seams::canonical`), so its blob's digest is
  `Envelope::digest()`, which attach answers; its media type is
  `application/vnd.agentplot.tailrace.envelope.v1+json`.

A digest a manifest names (a layer, or an index's artifact) that the
layout does not hold is answered absent naming it before anything is
written, since a descriptor needs the size of what it names. An
ecosystem whose name holds a character a media type cannot carry
(outside RFC 6838's restricted name characters) is refused
`Why::Said`, the words naming the ecosystem.

*Alternatives:* hold the seam's canonical `Manifest` JSON as the fake
does. Rejected: no OCI tool reads it, and C.2's layout is the one bolt
3 syncs to a registry (ADR 0003: every artifact is an OCI manifest
whose artifact type names its ecosystem). Write attestations as OCI
referrer manifests with a `subject` field. Rejected: C.2 says index
entries with a subject annotation, and attach must answer the
envelope's own digest; the referrer is the registry profile's form.

### D5. `index.json` holds three kinds of entry, rewritten under a lock

Every entry is a descriptor in `index.json`'s `manifests`, and which
kind it is is read from its annotations:

| entry | annotations | written by |
|---|---|---|
| a manifest held | none | put-artifact, after both blobs |
| a name | `org.opencontainers.image.ref.name` | name |
| an attestation | `io.github.agentplot.tailrace.subject`: the subject's digest | attach, after the envelope's blob |

read answers `Held::Manifest` for a digest a manifest entry names,
parsing its blob by the entry's media type, and `Held::Bytes` for any
other blob. put-artifact answers held already when its manifest entry
exists. A name's entry carries the media type and size of what it
names: the manifest entry's where there is one, else
`application/octet-stream` and the blob's size; a name moved to a
digest not held is absent naming it. A name has exactly one entry,
replaced in place when it moves. attach answers held already when an
entry with that envelope's digest and that subject exists, and adds
none. claims-of reads the attestation entries for the digest in
`index.json`'s order, which is the order attached, and answers each
envelope as its blob holds it; a blob that is not an envelope is
unreadable naming it. Entries of any other shape, another tool's, are
kept as they are on every rewrite and never answered.

Every change to `index.json` takes an exclusive `File::lock` on the
open `oci-layout` file, which nothing rewrites, reads `index.json`,
checks what the operation requires (for name, that the name is at the
expected digest, else refused `Why::NameAt { at }` and nothing written,
never retried: core.6), writes the new index to a temporary file, syncs
it and `rename`s it over `index.json`, and only then unlocks. Readers
take no lock: a rename is atomic, so a reader sees the index before or
after a change, never part of one. A malformed `index.json` is
unreadable naming it with `serde_json`'s words, and is never rewritten.

*Alternatives:* a lock file of tailrace's own beside the layout.
Rejected: a second file to leave behind when a writer dies, where
`oci-layout` is there for every layout. A lock on `index.json` itself.
Rejected: the rename replaces the file the lock is held on.

### D6. The profile's tests, and the rig

`crates/tailrace/tests/layout_store.rs` holds `LayoutRig { dir:
TempDir, layout: Layout }`, a new layout made in a new temporary
directory for each scenario. `profile()` answers the layout's source's
profile. `unreachable()` removes the directory and answers
`File::open(root.join("oci-layout")).unwrap_err().to_string()`, the
words D1 carries. One test runs
`run(store::GROUP, &store::scenarios(), &|| Box::new(LayoutRig::new())).assert()`.
The same file holds one test per scenario of the spec the group does
not play: the files a layout is made with, a layout never made by an
operation, another tool's entries and blobs, where bytes are held and
the blob unchanged by a second put, a blob altered on disk read and put
again, an index read back, the same artifact in two layouts, an
artifact's manifest as a registry reads it, a manifest naming bytes not
held, an attestation attached twice, an attestation and claims-of for a
digest not held, a copy of the layout, a name made and moved with one
entry, two threads moving one name, a name moved to a digest not held,
and a malformed index. They run in the `tailrace` project's existing
`test` task, tagged `check-fast`.

### D7. The module path to the profiles

`crates/tailrace/src/lib.rs` gains `pub mod profiles;`;
`profiles/mod.rs` holds a doc comment saying a profile implements one
seam's interface and nothing of the core (core.2) and `pub mod store;`;
`profiles/store/mod.rs` holds `pub mod layout;`. The other units of
bolt 2 add their seams' lines to `profiles/mod.rs` beside this one.
Nothing under `profiles/` imports `tailrace_core` (core.2), and no word
core.1 lists is added outside `profiles/`.

## Risks / Trade-offs

- [Every put of a blob held already reads the whole file to check it
  (D3)] → a laptop's layout holds packages and small images; a large
  image re-put costs one read. The registry profile answers held
  already from the registry, so the cost stays here.
- [`index.json` is read whole by every operation and rewritten by every
  attach, name and put-artifact] → linear in what the layout holds,
  which on one laptop is hundreds of entries; a layout that outgrows
  it is a registry's job (bolt 3).
- [A writer that dies between a blob and its entry leaves a blob no
  entry names, and one that dies before removing its temporary file
  leaves that file] → both are allowed by the specification and
  harmless to every reader; the next put of the same artifact adds the
  missing entry.
- [`File::lock` is advisory] → every writer of a layout is tailrace;
  another tool writing `index.json` at the same moment could lose an
  entry, which the risk of syncing a layout while publishing into it
  shares (bolt 3).
- [The envelope's blob is the seam's canonical JSON, not DSSE's wire
  JSON] → a verifier outside tailrace reads neither yet; named for the
  design agent, since bolt 3's registry form needs a decision.
