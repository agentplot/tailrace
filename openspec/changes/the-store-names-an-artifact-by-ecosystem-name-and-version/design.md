## Context

A publish today puts each artifact, signs its provenance, puts the index,
attaches the publisher's layout-policy claim (render-under-a-policy,
`Run::layout_policy`) and moves one name, the component's tag
`<prefix><version>`, from nothing to the index
(`crates/tailrace-core/src/publish.rs`, `Run::component`). A repeat
publish resolves that tag, reads the index back and compares packs
(`Run::held`, `Run::again`). Index metadata is the component, the
version and the artifacts (`crates/tailrace-seams/src/artifact.rs`,
`IndexMeta`); both store profiles write it as an OCI image index whose
annotations are the component and the version
(`crates/tailrace/src/profiles/store/oci.rs`).

Render-under-a-policy spells the artifact space in
`crates/tailrace-core/src/render.rs` (`ARTIFACT`, `artifact_name`) and
resolves a pin by it (`locate`); a name the store does not hold answers
absent. Its pinned answers a lockfile's pins alone (`Pinned { name,
version }`), and its render is given exactly the artifacts held
(`Contents::Artifacts`).

The registry profile holds everything in the one repository the binding
declares and escapes every name into a tag of it
(`profiles/store/registry/tag.rs`: bytes outside `[A-Za-z0-9.-]`, and a
leading `.` or `-`, become `_` and two hex digits; past 128 characters
refused). ADR 0014 chose that one repository and set aside "each
artifact also mounted into a repository named for it". C.1 at fbecdbf
now maps an artifact's pointer to a repository of its own, its version
the tag, so a registry lists a name's versions; channels and cache
entries keep their tags in the declared repository. The layout store
keeps a name as written, in a ref-name annotation of `index.json`.

The statement is `agentplot/blueprints` fbecdbf: 138, read with 3, 13,
49, 60, 109, 113, 137, B.1, B.6 and C.1, and the design agent's batch 25
rulings: the listing of a name's versions and the range resolution
render left out are this unit's, and so is 138's scenario.

This change's code starts once two things are on its branch:

- **bolt tailrace-the-gate-and-the-channels**: `RepositoryName { owner,
  name }` with `from_remote(url)` in `crates/tailrace-core/src/served.rs`,
  `History::remote(branch)` and `History::remote_url(remote)`, which
  reads `remote.<r>.url` as written with no `insteadOf` applied; and the
  conformance register's parts, and the statement list at 157c828.
- **the fix to render-under-a-policy** that adopts the statement through
  138 in the suite's list and registers 138, and 137's range sentence as
  a part, as this unit's.

## Goals / Non-Goals

**Goals:** every artifact a publish offers is named
`artifact/<ecosystem>/<name>@<version>` at its version's index, after
the layout policy and before the version's tag; a repeat publish fills
in a missing name; the index names its revision and repository; the
store lists a name's versions on every profile; a registry holds each
name as a repository with its versions as tags; a range with no pin
resolves to the newest version the layout's policy admits; no
component's tag enters a reserved space; 138 and 137's range cited by
the suite.

**Non-Goals:** npm's and Cargo's range rules and their renders of a
range (their units); pack given the component (cargos-registry); a copy
carrying the names (later work); serving.

## Decisions

### D1. One spelling of the artifact space, in the seams

`ARTIFACT` and `artifact_name(&Ecosystem, &EcosystemName, &Version) ->
Name` move from `render.rs` to `tailrace-seams` (`artifact.rs`), with
their reading back, `Name::as_artifact(&self) -> Option<(Ecosystem,
EcosystemName, Version)>`: after `artifact/`, the ecosystem up to the
first `/`, the version after the last `@`, the name between, none where
a part is empty. `render.rs` re-exports both, so `locate` and every
caller compile unchanged. Publish and `locate` spell the name, and a
store profile reads it back, through the one pair, so no two spellings
can drift. The name is built from the artifact's ecosystem, name and
version only; component, prefix, store and repository are never
arguments (138, 49, core.18). An ecosystem names no `/` and a version
no `@` in any ecosystem tailrace packs; a name holding `@`, as npm's
scoped names do, is split at the last one. Alternative: the registry
profile parsing its own spelling; refused, two spellings of one name.

### D2. The order: index, layout policy, artifact names, the version's tag

After the index is put, `Run::component` calls `layout_policy(&index)`,
then `name_artifacts(&index, &artifacts)`, then moves the version's tag,
then writes the record. `name_artifacts` first resolves every artifact's
name. A name absent is to be made; a name at this index is left as it
is; a name at any other digest refuses the version before any name is
made: `Not::refused(&self.ask.store, Why::NameAt { at: Some(other) },
Some("<name> names <other>, and this publish's index is <index>"))`.
Then each name to be made is made with `store.name(name, index, None)`;
a refusal from the store is answered as the store gave it, and nothing
is asked again (core.6).

Why the artifact names before the version's tag: the tag is what a
repeat publish takes as "this version is in the store". Made last, it
means every version the store names by its tag can also be found by
ecosystem, name and version, and a publish stopped or refused at an
artifact's name leaves the tag unmade, so the next publish takes the
first-publish path, where every put answers held already, the layout
policy is found carried and the names at this index are left alone.
Alternative: names after the tag; a refused name would leave a version
named by its tag that no consumer can find by name.

Why the layout-policy claim before the names: a consumer that resolves a
name reads the claims on the index it lands on (113); attaching the
policy first means a resolvable name never lands on an index missing it.

A name is never moved once made. The version's name stays with its first
index (18), so the only index an artifact's name could rightly name is
that one.

### D3. A repeat publish fills in what is missing

In the repeat path, after `again` finds every pack the same,
`layout_policy(&held_index)` runs, which signs nothing where the index
already carries the claim, and then `name_artifacts(&held_index,
&artifacts)` as in D2. Everything already there: nothing is written, and
S2 holds. A version published before this change: the layout-policy
claim where the file writes one and the index lacks it, then each name
made from nothing at the held index. The record and the answer are as
before.

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
revision; the repository is set where D5 finds one. A copy keeps both.

`RepositoryName` moves from `tailrace-core/src/served.rs` to
`tailrace-seams/src/artifact.rs`, with `from_remote`, `path`, `Display`
and their tests; `served.rs` keeps `pub use tailrace_seams::RepositoryName;`
so every caller compiles unchanged. The seams cannot depend on the core,
and the type now crosses the store seam.

### D5. The repository, read once per publish before anything is offered

`Run` reads, once and before the first output is packed:
`history.remote(branch)`, then `history.remote_url(remote)`, then
`RepositoryName::from_remote(url)`. A branch with no remote, no URL, or
a URL with no owner and name (a local path, as the test fixtures'
remotes are) gives none, and the publish goes on: 22 lets a repository
whose history no other publisher shares publish, and 138 names a
remote's owner and name, which such a repository has none of. A git
configuration that cannot be read stops the publish as unreadable,
before anything is offered. The URL is read as written, so the owner and
name are the same on every host whatever `insteadOf` a host applies (49,
114). Alternative: refuse a publish with no remote; refused because
local publishing, the conformance fixtures and a laptop's layout store
all publish without one.

### D6. The OCI encoding carries them as annotations

`oci.rs` writes, on the image index, `org.opencontainers.image.revision`
with the revision, the key the OCI image specification's annotations
define for a source control revision, and
`io.github.agentplot.tailrace.repository` with `owner/name`, each only
where the metadata names it; reading takes each where present and splits
the repository at its first `/`, an annotation with no `/` or an empty
part being unreadable naming the digest. Not
`org.opencontainers.image.source`: that key is a URL, which names a host
(49). Both profiles share this encoding, so one index keeps one digest in
a layout and a registry (1), and an index naming neither is written in
exactly today's bytes.

### D7. The store lists a name's versions

`Store` gains

```rust
/// 138: the versions the store names `name` of `ecosystem` at, every
/// `artifact/<ecosystem>/<name>@<version>` it holds, in no order the
/// core relies on (23); absent where it holds none.
fn versions(&self, ecosystem: &Ecosystem, name: &EcosystemName) -> Answer<Vec<Version>>;
```

Absent, not an empty list, where the store names no version of it: most
names a manifest declares are third-party, and an empty list would read
as a read that found nothing to say (core.19). The fake store and the
layout store answer from the names they hold beginning
`artifact_name(ecosystem, name, "")`, read back by `Name::as_artifact`
and kept where ecosystem and name match exactly; the layout store reads
`index.json`'s ref names, as "a layout its index's names" says (138).
The registry answers from a tag list (D9). The core orders nothing by
the listing: which versions are newest is the ecosystem's (23, D11).
Alternative: list by an arbitrary name prefix; refused, a registry can
list one repository's tags but no prefix across repositories, and only
the artifact space needs listing.

### D8. A registry repository per artifact name

`artifact/<ecosystem>/<name>@<version>` is held as the tag of the
version in the repository `<declared path>/artifact/<e>/<n>`, where `<e>`
and `<n>` are the ecosystem and the name each written as one path
component. Under the declared repository's path, so the store stays in
the owner's namespace on every registry (GitHub's registry names the
owner first) and names no second owner. Every other name, a channel's
and a cache entry's among them, stays a tag of the declared repository
as `tag::of` writes it today; blobs, manifests and referrers all stay in
the declared repository, and the seam's reads by digest go there.

A component is written by the rule C.1 calls "lowercased and escaped as
the distribution protocol allows", whose path component grammar is
`[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*`: the text lowercased; each byte of
`[a-z0-9]` kept; a `.` or `-` kept where the byte before it was kept as
a letter or digit and the byte after it is one; every other byte, `_`
among them, written `_` and two lower-case hex digits; and a first byte
that is not `[a-z1-9]` escaped and the component begun with `0`.
`@acme/site` is `0_40acme_2fsite`, `tailrace-core` is itself, `0x` is
`0_30x`. Every output is a valid component, and two names that differ
once lowercased never share one (a leading `0` before `_` is always the
added one). Names differing only by case share a repository, as C.1's
lowercasing says; their versions then share its tags, and a name
already at another index is refused at the make (D2), and a listed
version whose index names no artifact of the asked name is unreadable
for that version (D11), never the wrong bytes. A repository path, host
included, past 255 characters is refused naming the name and its length
before anything is written, the limit the distribution specification
gives clients.

The version is the tag, written by `tag::of` (`_` and two hex digits for
`+` and every byte outside `[A-Za-z0-9.-]`), and read back by a new
`tag::read`, its inverse, which answers none for a tag `of` could not
have written. Since only the version is a tag, the 128-character limit
binds versions alone.

Before the tag is put, the repository is made to hold the index: each
blob of each artifact the index names, the configs and layers, is
mounted from the declared repository (`POST
/v2/<repository>/blobs/uploads/?mount=<digest>&from=<declared path>`), and
where the registry answers 202, declining the mount, the bytes read from
the declared repository by digest are uploaded under the same digest
(core.5); then each artifact manifest, then the index, is put by its
digest in its own bytes, read from the declared repository. Anything the
repository holds already, by a HEAD of its digest, is skipped, so a
second make writes nothing but what is missing. A registry validates an
image index's manifests and a manifest's blobs within the repository it
is put in, so the index cannot be tagged there alone. Attestations are
not mounted: the seam reads an index's claims by its digest in the
declared repository, where they were attached (8, 138). The tag is then
put and read back exactly as `name` does today (expected digest, the
bound, 109, ADR 0015), in the artifact's repository; resolve reads the
tag there, and a repository the registry does not hold
(`NAME_UNKNOWN`, 404) or a tag it does not hold is absent.

The client holds one authorization per repository it has been answered
for, so a token scoped to one repository is not thrown away by a request
to another; the mount's challenge names the declared repository's pull
scope beside the artifact repository's push, which the existing
challenge answer covers.

The store never makes a repository by an API of its own: a registry that
creates repositories on push (zot, the reference distribution, GitHub's
registry) makes `artifact/<e>/<n>` on the first mount or put; one that
does not refuses that put in its own words (ADR 0015 records ECR's), and
the version's tag is not made. An ADR records this beside ADR 0014,
whose option 3 C.1 now takes for artifact names alone.

### D9. A registry lists a name's versions by its repository's tags

`versions` reads `GET /v2/<repository>/tags/list`, following each
`Link: rel="next"` page as the referrers list does (`next`), and answers
each tag read back by `tag::read`. `NAME_UNKNOWN` (404) and an empty tag
list are absent; a tag that does not read back is unreadable naming the
repository and the tag, never left out (core.19).

### D10. A projection answers a range, and render holds the newest within it

The projection seam's pinned answers `Vec<Pin>`:

```rust
/// 137: what a checkout wants of a name: the version its lockfile pins,
/// or, for a name its manifest declares and no lockfile pins yet, the
/// manifest's range as the manifest spells it.
pub enum Pin { Version(Pinned), Range(Ranged) }
pub struct Ranged { pub name: EcosystemName, pub range: Range }
```

`Pinned { name, version }` keeps its shape, and `Pin: From<Pinned>`, so
a projection that answers only pins wraps them. `Range` is a text newtype
as the manifest spells it; the core never reads it.

`Contents` gains

```rust
/// 85, 137: `held` held as given, and of each `within`'s candidates,
/// every one of its range's name, the newest by the ecosystem's own
/// rules (23) within its range, or none.
Ranged { held: &'a [Artifact], within: &'a [Within] }
pub struct Within { pub range: Ranged, pub candidates: Vec<Artifact> }
```

and render's answer is unchanged: `Holds::Versions` names every version
the layout holds, which is how the core learns which candidate render
took. Which versions fall within a range, and which is newest, are the
ecosystem's (137, 23), so the projection decides them; whether a
version may be held is the policy's, so the core decides that, and a
projection is given only candidates the policy admitted (core.2). The
file, image and moon cache projections answer `Contents::Ranged`
refused naming that their ecosystem declares no range, which their
pinned, answering absent, never leads to. Alternative: a fifth
projection operation answering which versions satisfy a range; refused,
137 gives that to render and B.6 names no such operation.

### D11. The core resolves a range by the listing

In `render::render`, for each `Pin::Range(ranged)`, before anything is
written (design D6 of render-under-a-policy):

1. `store.versions(ecosystem, name)`: absent is the range's answer,
   absent naming the name and the range, as a third-party pin's is
   (51); unreadable or refused is the outer answer and the layout is
   kept.
2. Each listed version is located and judged as a pin of that name and
   version is (`judged`), its answer kept: held is a candidate, refused
   is answered naming the claim found wanting (99), absent or
   unreadable for that version is answered as it is.
3. The projection renders `Contents::Ranged` with the pins' artifacts
   as `held` and each range's candidates; with no range among the pins,
   `Contents::Artifacts` as today.
4. The range's answer is the candidate whose metadata `Holds::Versions`
   names, held with its index and artifact; or, where render held none,
   absent naming the name and the range and saying how many listed
   versions the policy refused. A projection that holds a version it was
   not given a candidate for is unreadable naming it, and the layout is
   kept.

The answer names, per range, its name, its range, the version held or
the absence, and every listed version's answer; the record keeps the
held version as a held entry with its range, and each refused version as
a refused entry, so the record still says what the layout holds and why
not (61). A range lists the store; a pin never does, so S4's "the store
was asked nothing about the third" holds for pins.

### D12. Reserved spaces for names

`declared.rs` holds `RESERVED: [&str; 3] = [ARTIFACT, CACHE,
"channel/"]`, `ARTIFACT` from the seams and `CACHE` from `cache.rs`.
`Declared::read` refuses a component's prefix, or the rule's prefix for
the one undeclared component, that begins with any of them, with a new
`Why::Reserved { component, prefix, space }` from the history's source,
before anything is derived. A prefix is followed by a version, which
begins with a digit, so a prefix only enters a space by beginning with
it; `artifact-v` is fine. 138 says no component's tag may enter
`artifact/`, beside `cache/` and `channel/`; the three are kept by one
check.

### D13. The answer names each artifact's store name

`PublishedArtifact` gains `store_name: Name`, the name D1 spells,
serialized beside `ecosystem` and `name`. The command's text answer
prints it on each artifact's line.

### D14. The suite

The store group gains a scenario citing 138: two versions of one name
and one of another named, `versions` of the first answers its two and
not the other's, and a name never named answers absent. It runs over
the fake, the layout store and, in the tools layer, the registry against
zot and the reference distribution. The publish group's S1 checks each
artifact's name made at the index after the layout policy and before
the version's, and the index's revision, citing 138 beside what it
cites. A store-naming group over the fakes publishes a component of two
outputs and syncs a checkout that pins one by version and declares the
other by range, with a second, newer version of the second published
and a third refused by the policy: the pin is held, the range holds the
newest admitted version, the refused one is answered naming its claim,
and nothing names a component, a prefix or a repository; it cites 138
and 137. 138 leaves the register, and 137's range part leaves the
register's parts, in the commit whose scenarios cite them.

### D15. Tests

Core: `crates/tailrace-core/tests/publish.rs` over the fakes for each
scenario of the publishing delta, `tests/render.rs` for the
consumer-layouts delta, and `tests/declared.rs` for the reserved spaces.
A repository with a remote is staged by giving the scratch repository's
`origin` the URL `https://example.com/acme/site.git` and
`url.<fixture path>.insteadOf` that URL, so git still reaches the
fixture while `remote_url` reads the URL as written. Seams: round trips
of `IndexMeta` with and without the new fields and the form without them
unchanged; `artifact_name` and `as_artifact` inverse on names with `@`
and `/`. Store: layout unit tests for the annotations, the unchanged
bytes and the listing; registry unit tests for the component escape and
`tag::read`; the registry's mount, put, tag, resolve and listing in
`crates/tailrace/tests/registry_store.rs` against zot and the reference
distribution, in check-tools. Everything else runs in check-fast
through the crates' `test` tasks.

## Risks / Trade-offs

- [ECR makes no repository on push unless a repository creation
  template applies to it, and its support for a cross-repository mount
  is to be read] → the put is refused in ECR's words and the version's
  tag is not made, so nothing is half published; the ECR check, under
  its own tag (core.38), says what an operator sets up. Named for the
  team's ops to read before code.
- [Each artifact name's repository holds a mount of the index and every
  artifact it names, so a version of n artifacts makes n repositories
  each holding all n manifests] → mounts copy no bytes where the
  registry allows them; the declared repository stays the one place
  reads go.
- [Two components that pack the same ecosystem name at the same version
  collide on one artifact name] → the second publish is refused naming
  the name and both indexes, never chosen between.
- [A range judges every listed version of its name] → claims are read
  only for names a manifest declares and no lockfile pins, which a first
  install alone has; a pin lists nothing.
- [cargos-registry and npm-rendered-and-served implement pinned and
  render against the seam as it was] → whichever lands after this
  wraps its pins in `Pin` and matches `Contents::Ranged`; the conductor
  sees the overlap at the projection seam's file.
- [Code waits on the gate bolt and render's fix] → the first task checks
  both and stops through crew where either is missing.
- [A version published before this change has no artifact names until
  it is published again] → D3; nothing but a publish makes them, by
  design (20).

## Migration Plan

None beyond D3: indexes already in a store keep their bytes and digest,
and a repeat publish of their revision adds the layout policy where the
file writes one and the artifact names.
