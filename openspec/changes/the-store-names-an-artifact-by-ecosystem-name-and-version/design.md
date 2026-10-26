## Context

A publish today puts each artifact, signs its provenance, puts the index
and moves one name, the component's tag `<prefix><version>`, from
nothing to the index (`crates/tailrace-core/src/publish.rs`,
`Run::component`). A repeat publish resolves that tag, reads the index
back and compares packs (`Run::held`, `Run::again`). Index metadata is
the component, the version and the artifacts
(`crates/tailrace-seams/src/artifact.rs`, `IndexMeta`); both store
profiles write it as an OCI image index whose annotations are the
component and the version (`crates/tailrace/src/profiles/store/oci.rs`).
The registry escapes every name into a tag by one rule
(`profiles/store/registry/tag.rs`): bytes outside `[A-Za-z0-9.-]` become
`_` and two hex digits, a leading `.` or `-` too, and a tag past 128
characters is refused naming it. A channel's name, `channel/<name>` on
bolt tailrace-the-gate-and-the-channels, goes through the same rule, so
"spelled as a channel's" needs no new spelling. The layout store keeps a
name as written, in a ref-name annotation.

This change builds on two pieces of work that are not on this bolt's
base, and its code starts once both are:

- **render-under-a-policy** (built on its branch, archived there). It
  defines the name's spelling, `ARTIFACT = "artifact/"` and
  `artifact_name(&Ecosystem, &EcosystemName, &Version) -> Name` in
  `crates/tailrace-core/src/render.rs`, which `locate` resolves a pin
  by; and it attaches the publisher's layout-policy claim to the index
  in `Run::layout_policy`, after the index is put and before the
  version's tag is moved.
- **bolt tailrace-the-gate-and-the-channels** (on its own bolt, not yet
  on `main`). It holds `RepositoryName { owner, name }` with
  `from_remote(url)` in `crates/tailrace-core/src/served.rs`, and
  `History::remote_url(remote)`, which reads `remote.<r>.url` as written,
  no `insteadOf` applied. npm-rendered-and-served waits on the same.

The statement is `agentplot/blueprints` e267a30 for 138, read with 3,
13, 49, 60, 109, 113, 137, B.6 and C.1 as they stand on `main`.

## Goals / Non-Goals

**Goals:** every artifact a publish offers is named
`artifact/<ecosystem>/<name>@<version>` at its version's index before
the version's tag is made; a repeat publish fills in a missing name; the
index names its revision and its repository; no component's tag enters a
reserved space; the registry's spelling proven.

**Non-Goals:** resolving a pin or judging it (render-under-a-policy);
listing a name's versions (cargos-registry); making these names in a
copy (store-copy); a conformance scenario citing 138.

## Decisions

### D1. One spelling of the name, render-under-a-policy's

Publish calls `artifact_name` from `render.rs` for each artifact's
metadata. There is one function that spells the name, and both the
writer (publish) and the reader (`locate`) use it, so they cannot drift.
The name is made from the artifact's ecosystem, name and version only;
the component, prefix, store and repository are never arguments (138,
49, core.18). Alternative: a second constant in `publish.rs`; refused,
two spellings of one name.

### D2. The order: index, layout policy, artifact names, the version's tag

After the index is put, `Run::component` calls `layout_policy(&index)`
(as render-under-a-policy has it), then `name_artifacts(&index,
&artifacts)`, then moves the version's tag, then writes the record.
`name_artifacts` first resolves every artifact's name. A name absent is
to be made; a name at this index is left as it is; a name at any other
digest refuses the version before any name is made:
`Not::refused(&self.ask.store, Why::NameAt { at: Some(other) },
Some("<name> names <other>, and this publish's index is <index>"))`.
Then each name to be made is made with `store.name(name, index, None)`;
a refusal from the store is answered as the store gave it, and nothing
is asked again (core.6).

Why the artifact names before the version's tag: the tag is what a
repeat publish takes as "this version is in the store". Made last, it
means every version the store names by its tag can also be found by
ecosystem, name and version, and a publish stopped or refused at an
artifact's name leaves the tag unmade, so the next publish takes the
first-publish path, where every put answers held already and the names
at this index are left alone. Alternative: names after the tag; a
refused name would leave a version named by its tag that no consumer
can find by name.

Why the layout-policy claim before the names: a consumer that resolves a
name reads the claims on the index it lands on (113); attaching the
policy first means a resolvable name never lands on an index missing it.

A name is never moved once made. The version's name stays with its first
index (18), so the only index an artifact's name could rightly name is
that one.

### D3. A repeat publish fills in missing names

In the repeat path, after `again` finds every pack the same,
`name_artifacts(&held_index, &artifacts)` runs as in D2. Every name
already there: nothing is written, and S2 holds. A version published
before this change: each name is made from nothing at the held index.
The record and the answer are as before.

### D4. Index metadata names its revision and repository

`IndexMeta` gains `revision: Option<Revision>` and `repository:
Option<RepositoryName>`, with accessors and a builder,
`IndexMeta::published_from(self, revision, repository) -> IndexMeta`, so
`IndexMeta::new` keeps its three arguments and every caller that does
not publish (the copy, the conformance world, the tests) is unchanged.
Both fields are `#[serde(default, skip_serializing_if =
"Option::is_none")]`, so index metadata naming neither has the
machine-readable form it has today, and the fake store, which digests
that form, gives such an index the digest it gave before. They are
optional because an index put before this change names neither and must
still be read (the repeat path reads it). A publish always sets the
revision; the repository is set where D5 finds one.

`RepositoryName` moves from `tailrace-core/src/served.rs` to
`tailrace-seams/src/artifact.rs`, with `from_remote`, `path`, `Display`
and their tests; `served.rs` keeps `pub use tailrace_seams::RepositoryName;`
so every caller compiles unchanged. The seams cannot depend on the core,
and the type now crosses the store seam.

### D5. The repository, read once per publish before anything is offered

`Run` reads, once and before the first output is packed:
`history.remote(branch)`, then `history.remote_url(remote)`, then
`RepositoryName::from_remote(url)`. A branch with no remote, no URL, or
a URL with no owner and name (a local path, as the test fixtures' remotes
are) gives none, and the publish goes on: 22 lets a repository whose
history no other publisher shares publish, and 138 names a remote's
owner and name, which such a repository has none of. A git configuration
that cannot be read stops the publish as unreadable, before anything is
offered. The URL is read as written, so the owner and name are the same
on every host whatever `insteadOf` a host applies (49, 114).
Alternative: refuse a publish with no remote; refused because local
publishing, the conformance fixtures and a laptop's layout store all
publish without one.

### D6. The OCI encoding carries them as annotations

`oci.rs` writes, on the image index, `org.opencontainers.image.revision`
with the revision, the key the OCI image specification's annotations
define for a source control revision, and
`io.github.agentplot.tailrace.repository` with `owner/name`, each only
where the metadata names it; reading takes each where present and
splits the repository at its first `/`, an annotation with no `/` or an
empty part being unreadable naming the digest. Not
`org.opencontainers.image.source`: that key is a URL, which names a
host (49). Both profiles share this encoding, so one artifact keeps one
digest in a layout and a registry (1), and an index naming neither is
written in exactly today's bytes.

### D7. Reserved spaces for names

`declared.rs` holds `RESERVED: [&str; 3] = [ARTIFACT, CACHE,
"channel/"]`, `ARTIFACT` from `render.rs` and `CACHE` from `cache.rs`.
`Declared::read` refuses a component's prefix, or the rule's prefix for
the one undeclared component, that begins with any of them, with a new
`Why::Reserved { component, prefix, space }` from the history's source,
before anything is derived. A prefix is followed by a version, which
begins with a digit, so a prefix only enters a space by beginning with
it; `artifact-v` is fine. 138 says no component's tag may enter
`artifact/`, beside `cache/` and `channel/`; the three are kept by one
check. Alternative: refuse only `artifact/`; refused, the other two are
the same kind of space and the same one-line check keeps them.

### D8. The answer names each artifact's store name

`PublishedArtifact` gains `store_name: Name`, the name D1 spells,
serialized beside `ecosystem` and `name`. The command's text answer
prints it on each artifact's line.

### D9. The registry needs no new code

`tag::of` already escapes the name: `artifact/npm/@acme/site@1.3.0` is
the tag `artifact_2fnpm_2f_40acme_2fsite_401.3.0`, in the binding's one
repository (C.1). This change adds unit tests in `tag.rs` for that
spelling, for two names differing only in `/` and `@` giving two tags,
and for a name past 128 characters refused, and a tools-layer test in
`crates/tailrace/tests/registry_store.rs` that makes and resolves an
artifact's name against zot and the reference distribution.

### D10. Tests

Core: `crates/tailrace-core/tests/publish.rs` over the fakes for each
scenario of the publishing delta, and `tests/declared.rs` (or the
existing policy-file tests) for the reserved spaces. A repository with a
remote is staged by giving the scratch repository's `origin` the URL
`https://example.com/acme/site.git` and `url.<fixture path>.insteadOf`
that URL, so git still reaches the fixture while `remote_url` reads the
URL as written. An integration test publishes over the fakes and then
calls `render::locate` with a pin of each artifact, which finds it.
Seams: round trips of `IndexMeta` with and without the new fields, and
the form without them unchanged. Store: `layout` unit tests for the
annotations and the unchanged bytes. Conformance: S1's checks gain the
names and the revision; publish-group checks that count names moved
count three. All run in `check-fast` through the crates' `test` tasks,
the registry test in `check-tools`.

## Risks / Trade-offs

- [A name whose tag passes 128 characters cannot be named in a
  registry: `artifact_2f` and the escapes take room, so an npm name of
  about 90 characters or more is refused] → the refusal names the name
  and its length, and the version's tag is not made, so nothing is half
  published; the layout store has no such limit. Named for the design
  agent.
- [Two components that pack the same ecosystem name at the same version
  collide on one artifact name] → the second publish is refused naming
  the name and both indexes, never chosen between.
- [Code waits on two other pieces of work] → the conductor sequences it
  after render-under-a-policy merges and the gate bolt is taken in, as
  npm-rendered-and-served already waits.
- [A version published before this change has no artifact names until
  it is published again] → D3; nothing but a publish makes them, by
  design (20).

## Migration Plan

None beyond D3: indexes already in a store keep their bytes and digest,
and a repeat publish of their revision adds the artifact names.
