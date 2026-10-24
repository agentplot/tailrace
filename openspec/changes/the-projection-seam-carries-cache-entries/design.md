# Design

## Context

The projection seam today (`crates/tailrace-seams/src/projection.rs`):

```rust
fn pack(&self, output: &Output, version: &Version) -> Answer<Packed>;
fn render(&self, artifacts: &[Artifact], at: &str) -> Answer<Rendered>;
fn serve(&self, layout: &Layout) -> Answer<Serving>;
```

`Packed` is `{ bytes, meta: ArtifactMeta, same_again }` and `Rendered`
is `{ layout, holds: Vec<ArtifactMeta> }`. Implementers are the fake
projection, the file and image projections, the conformance world's and
the core tests' watched projections, and the seams crate's own
`Nowhere`. The callers are publish (step 9, which reads `packed.meta`
throughout steps 10 and 11), the conformance projection and binding
groups, and the fakes' own tests.

The statement now has pack given an output at a build identity, with
the version only where the bytes are an artifact's, and render given
artifacts or cache entries by build identity (84, 85, 87 as amended).
madswan-design ruled the shape: one case of the three operations, no
twin operations, the inputs widened with a cache entry of
`{build identity, digest}`, and a projection that packs no cache
entries answering absent for one.

The conformance register keeps 52 to 54 and 100 under bolt 3's cache;
this change cites only 84, 85 and 87, which the group already cites,
so the register and the coverage check are untouched (core.33).

## Goals / Non-Goals

**Goals:**

- The seam carries a cache entry through pack and render, and nothing
  that takes an artifact can be handed one (core.30).
- Every implementer compiles against the widened trait and keeps its
  behavior for artifacts.
- The suite tells a projection that packs cache entries from one that
  packs none, and holds each to its case.

**Non-Goals:**

- Any store operation on cache entries, the core's cache module, or
  moon's cache projection (`moons-cache-namespace`).
- Changing what publish offers, in what order, or how it judges.

## Decisions

### D1. pack takes the build identity and an optional version

```rust
fn pack(&self, output: &Output, at: &BuildId, version: Option<&Version>) -> Answer<Packed>;

pub struct Packed {
    pub bytes: Vec<u8>,
    /// Present exactly where a version was given (84, 100).
    pub meta: Option<ArtifactMeta>,
    pub same_again: bool,
}
```

`Some(version)` is the artifact case; `None` is the cache-entry case.
The build identity is given in both, as 84 now reads; the file and
image projections ignore it for an artifact, so their bytes under every
version stay the same (87). `same_again` covers the same outputs and
version, or the same build identity (87).

Alternatives: an input enum `Pack::Artifact(Version) | Pack::CacheEntry`
says the same as `Option<&Version>` with one more type to name; an
answer enum `Artifact(meta) | CacheEntry` makes every caller match where
publish only needs to refuse the one case it never asks for. Twin
operations (`pack_entry`, `fill`) were ruled out by design.

### D2. A cache entry is `{ build, digest }` and no artifact

```rust
/// 52, 53: bytes a build left, named by their build identity and their
/// digest, which is never an artifact: it has no ecosystem, name or
/// version, and no conversion makes one an `Artifact`.
pub struct CacheEntry {
    pub build: BuildId,
    pub digest: Digest,
}
```

Defined in `projection.rs`, serde in its JSON form like every seam type
(57), exported from the crate root. It names what the caller packed:
the bytes pack answered at that build identity and their digest. No
`From`/`Into` between it and `Artifact` or `ArtifactMeta`.

### D3. render is given one of two contents, and holds one of two

```rust
pub enum Contents<'a> {
    Artifacts(&'a [Artifact]),
    CacheEntries(&'a [CacheEntry]),
}

fn render(&self, contents: Contents<'_>, at: &str) -> Answer<Rendered>;

pub enum Holds {
    Versions(Vec<ArtifactMeta>),
    BuildIds(Vec<BuildId>),
}

pub struct Rendered {
    pub layout: Layout,
    pub holds: Holds,
}
```

`Contents` derives `Clone, Copy, Debug, Serialize` (the fake records
what it was given as JSON); `Holds` round-trips through JSON,
`#[serde(rename_all = "snake_case")]`. A layout holds one kind, so the
answer says which rather than carrying two lists one of which is always
empty.

### D4. One shared answer for a projection that packs none

```rust
/// 84, 85: what a projection that packs no cache entries answers for
/// one: absent, never refused or unreadable (58).
pub fn no_cache_entries(source: &Source) -> Not {
    Not::absent(source, "cache entries")
}
```

The file and image projections answer `Err(no_cache_entries(&source))`
for `pack(_, _, None)` and for `render(Contents::CacheEntries(_), _)`.
Their render of artifacts stays absent as not built. A trait default
body cannot carry this, since pack's artifact case must be written by
every projection; the shared function is the one body they all give.

### D5. publish gives the build identity and refuses a pack with no metadata

Step 9 calls `pack(&files, &output.report.build, Some(&versioned.version))`
and turns each answer at once into publish's own record of an artifact
pack (bytes, `ArtifactMeta`, `same_again`), so steps 10 and 11 and
`answered` read the metadata as they do today. An answer with `meta:
None` is refused before anything is offered (core.12): source
`Seam::Projection` with the output's ecosystem as profile, `Why::Said`,
words `"{output} was packed at {version} and the answer names no
metadata"`. No conforming projection answers so; the refusal keeps a
broken one from reaching put-artifact.

### D6. The fake packs cache entries by default and can be scripted to pack none

`ProjectionState` gains `caches: bool`, `true` from `new`;
`FakeProjection::packs_no_cache_entries()` sets it false. pack with
`None`: when caching, the bytes are the canonical JSON of
`{ "build", "output", "files", "pack" }` (no ecosystem, name or
version; `pack` as the unsteady counter is today), `meta: None`; when
not, absent by D4. pack with `Some(version)` is unchanged and ignores
the build identity. render of cache entries: when caching, a layout at
`at` holding `Holds::BuildIds` in the order given; when not, absent.
The recorded `given` JSON names `output`, `build` and `version` (null
for none), or `contents` and `at`.

### D7. The rig says which case; the scenarios use `not_applicable`

`ProjectionRig` gains `fn caches(&self) -> bool`. Three scenarios below
the group's marker, `plays: None`:

| name | cites | runs |
|---|---|---|
| a cache entry packed with no version | 84, 87 | not applicable unless `caches`; packs `rig.output()` twice at `b7e1` with `None`; both `meta` are `None`; where both promise `same_again`, the bytes are equal |
| a layout of the build identities given | 85 | not applicable unless `caches`; packs at `b7e1` and `c3d4`, makes each `CacheEntry` from the build identity and `Digest::of` the bytes, renders them twice at the group's `AT`; both hold `BuildIds` of exactly those two (sorted), and the two renders are equal |
| a projection that packs no cache entries | 84, 85 | not applicable when `caches`; pack at `b7e1` with `None` and render of one cache entry each answer `Not::Absent` |

A not-applicable call names the rig's case, e.g. "the projection packs
no cache entries". Above the marker the scenarios change only in their
calls: pack at `b7e1` with `Some(&version)`, `packed.meta` read through
`expect` with a message naming the scenario, render with
`Contents::Artifacts`, and `holds` matched as `Holds::Versions`, any
other kind failing the scenario. The binding group's pack call changes
the same way.

`FakeProjectionRig` gains `packing_no_cache_entries()`, its projection
so scripted; `caches()` answers the fake's state. `tests/fakes.rs` runs
the group over both rigs. `tests/projections.rs` adds `"a projection
that packs no cache entries"` to the scenarios it runs over the file
and image projections, whose rigs answer `caches()` false; the module
doc's departure from core.33 (render and serve not run) is unchanged.

### D8. The watched projections pass the widened inputs through

`crates/tailrace-conformance/src/world.rs`'s `Watched` records a
`Packing` only for an answer that carries metadata (the artifact case),
reading the ecosystem from it; the core tests' `Watched` records what
it sees at every pack, as now. Both hand render's `Contents` through.

## Risks / Trade-offs

- [Every implementer and test of pack and render changes in one unit] →
  the change is mechanical outside the seams, fakes and suite, and the
  gate's `check-fast` runs every crate's tests; the tasks list each
  call site found on the bolt.
- [`meta: Option` lets a projection answer no metadata to a versioned
  pack] → the suite fails it (packed the same again reads the version
  from the metadata) and publish refuses it (D5).
- [A unit landed between this bolt's cut and code adds a projection or
  a pack call] → the coder greps for `impl Projection for`, `.pack(`
  and `.render(` across `crates/` before the last group and updates
  any found.

## Open Questions

None that change the specs or tasks. Whether moon's cache projection
renders a layout's build identities in a given order is the next unit's;
the suite here compares them sorted.
