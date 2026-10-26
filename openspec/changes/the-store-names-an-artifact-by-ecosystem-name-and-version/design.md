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
absent.

The registry profile holds everything in the one repository the binding
declares and escapes every name into a tag of it
(`profiles/store/registry/tag.rs`: bytes outside `[A-Za-z0-9.-]`, and a
leading `.` or `-`, become `_` and two hex digits; past 128 characters
refused), a cache entry's bytes named through a cache manifest (ADR
0014). C.1 at 4c14235 maps the store's names into three spaces a
credential can be scoped to apart: a component's indexes and channels
under the component's own repository, the channel's name a tag of it;
cache entries under `cache/<build>`, one repository per build profile,
the build identity the tag; and an artifact's pointer as the repository
`artifact/<ecosystem>/<name>` with the version as its tag. On a registry
that makes a repository on push only by a creation template, ECR among
them, the operator makes the repositories or a template covering them,
and a push refused for a missing repository is reported naming it. The
layout store keeps a name as written, in a ref-name annotation of
`index.json`. A channel's store name is `channel/<name>` where the
repository declares no component and `channel/<component>/<name>` where
it does (`crates/tailrace-seams/src/keeper.rs`, `channel_name`); the
gate moves it through the store seam, and the ledger held in the store
(C.14) walks a chain back from the index it resolves to. The core writes a cache entry only from a push
(`crates/tailrace-core/src/push.rs`, through `cache::write`: put-bytes,
then name `cache/<build identity>`); a sync reads one by resolve and
read, through the moon cache projection's second binding of `[store]`
(`crates/tailrace/src/bindings/mod.rs`). Moon is the one build profile
(`profiles/build/mod.rs`).

The statement is `agentplot/blueprints` 4c14235: 138, read with 3, 13,
49, 60, 109, 113, 125, 135, 137, B.6 and C.1, with the design agent's
batch 25, 26 and 27 rulings and the planner's settlement of batch 26's
collisions, proposal 153, which puts the component's own repository
into this change on batch 27's ruling (a3efbd8), and the design agent's
ruling on the dependency closure at 4c14235: the listing of a name's
versions, `Store::versions`, is this unit's alone, empty where the store
holds nothing, and this unit lands first; cargos-registry reads it and
owns the core's loop (137: list through the store's versions, judge
under the policy, hold, ask what each held version requires again until
nothing held requires a name the layout lacks) and the range types; B.6's
requires (140) is built by the-projection-answers-what-an-artifact-requires;
cache entries leave the declared repository for `cache/<build>`; 138's
scenario is this unit's.

This change's code starts once two things are on its branch:

- **bolt tailrace-the-gate-and-the-channels**: `RepositoryName { owner,
  name }` with `from_remote(url)` in `crates/tailrace-core/src/served.rs`,
  `History::remote(branch)` and `History::remote_url(remote)`, which
  reads `remote.<r>.url` as written with no `insteadOf` applied; and the
  conformance register's parts, and the statement list at 157c828.
- **the fix to render-under-a-policy** that adopts the statement through
  138 in the suite's list and registers 138, and 137's range and closure
  sentences as parts.

## Goals / Non-Goals

**Goals:** every artifact a publish offers is named
`artifact/<ecosystem>/<name>@<version>` at its version's index, after
the layout policy and before the version's tag; a repeat publish fills
in a missing name; the index names its revision and repository; the
store lists a name's versions on every profile; a registry holds a
declared component's channels as tags of its own repository, which
holds its indexes, each artifact name as a repository with its versions
as tags, and each cache entry in its build profile's cache repository,
written there by a push; a component's name names one repository;
a missing repository refused naming it; no component's tag enters a
reserved space; 138 cited by the suite.

**Non-Goals:** the core's loop that resolves a manifest's range by the
listing and closes a layout over what its versions require, and the
range types (cargos-registry); requires (140, the projection unit); a
version's tag in its component's repository (D10);
pack given the component (cargos-registry); a
copy carrying the names (later work); serving.

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
/// core relies on (23); empty where it holds none.
fn versions(&self, ecosystem: &Ecosystem, name: &EcosystemName) -> Answer<Vec<Version>>;
```

An empty list where the store names no version of it, as the planner
settled the contract for cargos-registry, which reads it: the read
succeeded and found nothing, and only a read that fails is unreadable,
never an empty list (core.19). The fake store and the layout store
answer from the names they hold beginning `artifact_name(ecosystem,
name, "")`, read back by `Name::as_artifact` and kept where ecosystem
and name match exactly; the layout store reads `index.json`'s ref
names, as "a layout its index's names" says (138). The registry answers
from a tag list (D11). The core orders nothing by the listing: which
versions are newest is the ecosystem's (23), and no core path of this
change reads it; the core's loop over a manifest's range, which
cargos-registry builds on this listing, does (137). Alternative: list by an
arbitrary name prefix; refused, a registry can list one repository's
tags but no prefix across repositories, and only the artifact space
needs listing.

### D8. A registry repository per artifact name

`artifact/<ecosystem>/<name>@<version>` is held as the tag of the
version in the repository `<declared path>/artifact/<e>/<n>`, where `<e>`
and `<n>` are the ecosystem and the name each written as one path
component (D12). Under the declared repository's path, so the store
stays in the owner's namespace on every registry (GitHub's registry
names the owner first) and names no second owner. Blobs, manifests and
referrers stay in the declared repository, and the seam's reads by
digest go there first (D9).

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
is put in, so the index cannot be tagged there alone. This is the one
way the registry holds an index in a repository other than the declared
one, `hold(repository, index)`, which D10 uses too. Attestations are
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

### D9. A cache entry in its build profile's cache repository

`cache/<build identity>` is held as the tag of the build identity, by
`tag::of`, in the repository `<declared path>/cache/<build>`, where
`<build>` is the bound build profile's name (`cache/moon`), so cache
entries live apart from every component and a credential scoped to
`<declared path>/cache/` writes entries and nothing else (C.1, 135). The
core's name is unchanged, `cache/<build identity>` on every profile
(52), so `cache.rs`, the layout store and the fakes are untouched; only
the registry splits it. Every other name that is no artifact's, no
cache entry's and no declared component's channel (D10), a version's
tag and the channel of a repository that declares no component among
them, stays a tag of the declared repository as `tag::of` writes it
today, a name at bytes
through a cache manifest there as ADR 0014 has it.

The cache manifest of a cache entry is put in the cache repository, and
its one layer must be there: where a HEAD of the bytes' digest in the
cache repository answers not held, the blob is mounted from the declared
repository, or uploaded from its bytes there where the mount is
declined, as D8 does, before the manifest and the tag are put. Resolve
of a cache name reads the tag in the cache repository and answers the
layer's digest; a repository or tag not held is absent, and an entry
gone is the answer it always was (54).

A store bound for a push puts bytes into the cache repository, since a
push writes cache entries and nothing more (135): `profiles::store::bind`
takes what it is bound for, and `bindings::bind` passes `For::Push`'s,
so a job holding only a `cache/` credential puts the bytes, the cache
manifest and the tag where it may write, and never touches the declared
repository. Every other binding puts bytes into the declared repository,
where an artifact's manifest needs its layers.

A read by digest, of bytes or of a manifest, asks the declared
repository and, where it answers not held, the cache repository, so a
sync's read of an entry's bytes and a publish's read of bytes a push
left (135) both find them with no map of digests to repositories, which
ADR 0014 refused as kept state (61, core.20). claims-of asks the
declared repository alone: no cache entry carries a claim (53).

The build profile's name reaches the registry from the build family:
`profiles::build` gives it beside `bind`, and the store binding takes
it, so `bindings/mod.rs` names no tool (core.1). Alternative: the core's
name `cache/<build>/<identity>`; refused, it would change 52's name on
every profile and make the core carry the build profile's name for a
registry's credential scoping alone.

### D10. A declared component's channels and indexes in its own repository

`channel/<component>/<name>`, the store name of a declared component's
channel, is held as the tag `<name>`, by `tag::of`, of the repository
`<declared path>/<component>`, the component written as one path
component (D12): `channel/lib/beta` is the tag `beta` of
`<declared path>/lib`. `channel/<name>`, the channel of the one
component of a repository that declares none, stays the tag of the
declared repository it is today: that component's own repository is the
declared one, and nothing changes for it (125). The registry reads a
channel's name back through one seams function beside `channel_name`,
`Name::as_channel(&self) -> Option<ChannelId>`: after `channel/`, the
component up to the first `/` and the channel's name after it, or the
channel's name alone with no component where no `/` follows; none for a
name outside `channel/` or with an empty part. D14 keeps a component's
name free of `/`, so the reading is `channel_name`'s inverse.

The component's indexes live there. When the registry puts an image
index whose metadata names a component other than `repository`
(`REPOSITORY_COMPONENT`, the undeclared one's), it holds it in the
component's repository after putting it in the declared one, by D8's
`hold`. Before a channel's tag is put, `hold` runs again for the index
the channel moves to, writing only what a HEAD finds missing, so an
index put before this change is held there before a tag names it. The
tag is then put with the expected digest and read back as `name` does
today (109, ADR 0015), in the component's repository; resolve reads it
there, and a repository or tag the registry does not hold is absent.
The gate's promotion and the ledger held in the store reach all of it
through the store seam unchanged: the ledger walks from the index the
channel resolves to and reads claims by digest, in the declared
repository where they were attached.

A version's tag, `<prefix><version>`, stays a tag of the declared
repository. C.1 places a component's indexes and channels in its
repository and does not place the version's tag, and the tag's spelling
does not carry its component, so the registry could not tell which
repository it belongs in without the components the policy declares,
which no store is bound with (core.2, 65). Alternative: spell a
declared component's version in the store with its component and hold
it in the component's repository; refused here, it renames every
version in every store for a placement the statement does not ask, and
is named for the design agent.

A publish then writes the declared repository and its components'
repositories, and a promotion writes the channel's component's
repository; the self-hosting guide says so for an operator scoping
credentials (D13).

### D11. A registry lists a name's versions by its repository's tags

`versions` reads `GET /v2/<repository>/tags/list` of the artifact name's
repository, following each `Link: rel="next"` page as the referrers list
does (`next`), and answers each tag read back by `tag::read`.
`NAME_UNKNOWN` (404) and an empty tag list are the empty list; a tag
that does not read back is unreadable naming the repository and the
tag, never left out (core.19).

### D12. A repository path component as the distribution protocol allows

An ecosystem, a name, a component and a build profile are each written
as one path component by the rule C.1 calls "lowercased and escaped as the
distribution protocol allows", whose path component grammar is
`[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*`: the text lowercased; each byte of
`[a-z0-9]` kept; a `.` or `-` kept where the byte before it was kept as
a letter or digit and the byte after it is one; every other byte, `_`
among them, written `_` and two lower-case hex digits; and a first byte
that is not `[a-z1-9]` escaped and the component begun with `0`.
`@acme/site` is `0_40acme_2fsite`, `tailrace-core` is itself, `0x` is
`0_30x`. Every output is a valid component, and two names that differ
once lowercased never share one (a leading `0` before `_` is always the
added one). `lib` is itself, so a component's repository is named as
its people read it. Names differing only by case share a repository, as C.1's
lowercasing says; their versions then share its tags, and a name
already at another index is refused at the make (D2), never the wrong
bytes. A repository path, host included, past 255 characters is refused
naming the name and its length before anything is written, the limit
the distribution specification gives clients.

### D13. A repository nobody made

The store never makes a repository by an API of its own: a registry that
creates repositories on push (zot, the reference distribution, GitHub's
registry) makes `<component>`, `artifact/<e>/<n>` and `cache/<build>`
on the first mount or put. One that makes them only by a creation template, ECR among
them, refuses a put into a repository no one made or no template
covers; the refusal carries the registry's words verbatim and names the
repository the put was for, which `NAME_UNKNOWN` on a put, or a body
naming a repository not found, gives, so an operator reads which one to
make (58, 59). A refused artifact name leaves the version's tag unmade
(D2); a refused cache entry is the push's answer for that output, as any
store refusal is, naming `cache/<build>`; a refused channel's tag is
the promotion's answer, as any refused move is, naming the component's
repository; and a push bound for the cache
space never falls back to the declared repository, which its credential
may not write (135). Nothing is retried. The self-hosting guide says what
an ECR operator makes before the first publish: the declared repository,
and repository creation templates applying on push to
`<declared path>/artifact/`, `<declared path>/cache/` and each declared
component's `<declared path>/<component>`, or one applying to
`<declared path>/` that covers all three, or each repository by hand.

### D14. Reserved spaces, and a repository per component

`declared.rs` holds `RESERVED: [&str; 3] = [ARTIFACT, CACHE,
"channel/"]`, `ARTIFACT` from the seams and `CACHE` from `cache.rs`.
`Declared::read` refuses a component's prefix, or the rule's prefix for
the one undeclared component, that begins with any of them, with a new
`Why::Reserved { component, what, space }` from the history's source,
before anything is derived. A prefix is followed by a version, which
begins with a digit, so a prefix only enters a space by beginning with
it; `artifact-v` is fine. 138 says no component's tag may enter
`artifact/`, beside `cache/` and `channel/`; the three are kept by one
check. On a registry, a version's tag is a tag of the declared
repository, so a reserved prefix would also have reached the tag the
artifact or cache space is not.

`Declared::read` refuses too, before anything is derived, a declared
component whose name would not name a repository of its own on a
registry (D10): a name holding `/`, which `as_channel` could not read
back; `artifact` or `cache`, whose repository would sit at the root of a
reserved space; `repository`, the undeclared component's name, whose
indexes the registry leaves in the declared repository; each with
`Why::Reserved { component, what, space }`, `what` the prefix or the
name spelled as `DeclaredTwice` spells it (`prefix "cache/"`, `name
"cache"`) and `space` what it would enter. Two components whose names
are equal once lowercased are refused as `Why::DeclaredTwice`, `what`
naming both spellings, since a registry's repository names are lower
case and the two would share one. The core names no registry for this
(core.1): it keeps a component's name a single path segment apart from
the store's own spaces, which every store profile can then place.

### D15. The answer names each artifact's store name

`PublishedArtifact` gains `store_name: Name`, the name D1 spells,
serialized beside `ecosystem` and `name`. The command's text answer
prints it on each artifact's line.

### D16. The suite

The store group gains a scenario citing 138: two versions of one name
and one of another named, `versions` of the first answers its two and
not the other's, and a name never named answers the empty list. It runs
over the fake, the layout store and, in the tools layer, the registry
against zot and the reference distribution, and in `check-ecr` against
ECR, each run's names apart from every earlier run's. The publish
group's S1 checks each artifact's name made at the index after the
layout policy and before the version's, and the index's revision,
citing 138 beside what it cites. A store-naming group over the fakes
publishes a component of two outputs and syncs a checkout that pins one
of them by version: the pin is held from the index the publish
answered, the layout-policy claim read from that index, and nothing in
the checkout's ask or the answer names a component, a prefix or a
repository; it cites 138. 138 leaves the register in the commit whose
scenarios cite it, and the parts of 137 about a range and the closure
are named in the register as cargos-registry's, which cites them.

### D17. Tests

Core: `crates/tailrace-core/tests/publish.rs` over the fakes for each
scenario of the publishing delta, and `tests/declared.rs` for the
reserved spaces. A repository with a remote is staged by giving the
scratch repository's `origin` the URL `https://example.com/acme/site.git`
and `url.<fixture path>.insteadOf` that URL, so git still reaches the
fixture while `remote_url` reads the URL as written. Seams: round trips
of `IndexMeta` with and without the new fields and the form without them
unchanged; `artifact_name` and `as_artifact` inverse on names with `@`
and `/`; `as_channel` the inverse of `channel_name` with and without a
component, and none for `cache/x`, `channel/` and `channel/lib/`. Store: layout unit tests for the annotations, the unchanged
bytes and the listing; registry unit tests for the component escape and
`tag::read`, and, against a stub registry in the style of `client.rs`'s
tests, the declined mount, the push-bound put, the read that falls
through to the cache repository, and a declared component's index held
in its repository at put and its channel's tag put there; the
registry's mount, put, tag, resolve, listing, component repository and
cache repository in
`crates/tailrace/tests/registry_store.rs` against zot and the reference
distribution, in check-tools; bindings tests that a push binds the store
for the cache space and every other command does not. Everything else
runs in check-fast through the crates' `test` tasks.

## Risks / Trade-offs

- [ECR makes no repository on push unless a repository creation
  template applies to it, and whether it takes a cross-repository mount
  is to be read] → a put into a missing repository is refused naming it
  and the version's tag is not made, so nothing is half published; a
  declined mount uploads instead. `check-ecr` needs the templates (or
  the repositories) on its host before it passes; the team's ops read
  both before code.
- [Each artifact name's repository holds a mount of the index and every
  artifact it names, so a version of n artifacts makes n repositories
  each holding all n manifests] → mounts copy no bytes where the
  registry allows them; the declared repository stays the one place
  claims are read.
- [A read by digest the declared repository does not hold asks a second
  repository] → only for a cache entry's bytes, which a sync and a
  publish from the cache read; one request more, and no state kept.
- [A declared component's channel tags a gate wrote into the declared
  repository before this change are not read from its component's
  repository] → the self-hosting guide says to put each one, at the
  digest it names, as the channel's tag of its component's repository
  before the first promotion after this change; until then the channel
  reads as empty in the store, and where the ledger is held in the store
  (C.14) its chain does too, so the step comes before any promotion. A
  carried tag is a name made from nothing at the digest the old one
  names, by hand, and tailrace writes nothing to carry it.
- [Every index of a declared component is held twice, in the declared
  repository and its component's] → a mount copies no bytes where the
  registry allows it, and the declared repository stays the one place
  claims are read.
- [A version's tag is not in its component's repository, so a
  credential scoped to one component cannot make that component's
  versions] → C.1 does not place the tag; D10 names it for the design
  agent.
- [Cache entries a push wrote into the declared repository before this
  change are not found under `cache/<build>`] → an entry gone is a build
  again and no worse (54); the next push writes it where it now lives.
- [Two components that pack the same ecosystem name at the same version
  collide on one artifact name] → the second publish is refused naming
  the name and both indexes, never chosen between.
- [cargos-registry's change, as it stands, adds a listing of its own or
  leaves the range loop to this unit] → the design agent ruled at
  4c14235 that it reads this listing and owns the loop and the range
  types, and this change lands first; the conductor sees the overlap at
  the store seam's file.
- [Code waits on the gate bolt and render's fix] → the first task checks
  both and stops through crew where either is missing.
- [A version published before this change has no artifact names until
  it is published again] → D3; nothing but a publish makes them, by
  design (20).

## Migration Plan

Indexes already in a store keep their bytes and digest, and a repeat
publish of their revision adds the layout policy where the file writes
one and the artifact names (D3). On a registry, cache entries move to
`cache/<build>` by being pushed again; the ones left in the declared
repository are read by nothing and may be removed at any time (54). An
ECR operator makes the creation templates before the first publish after
this change (D13). A declared component's channel tags already in the
declared repository are put, by the operator, as tags of the component's
repository at the digests they name before the first promotion after
this change (Risks); its indexes put before this change are held
there by the first promotion to each (D10).
