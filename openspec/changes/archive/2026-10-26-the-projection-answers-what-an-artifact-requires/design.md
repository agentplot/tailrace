## Context

The projection trait, in `crates/tailrace-seams/src/projection.rs`, has
four operations: pack, render, serve and pinned. pinned (137) came with
the render-under-a-policy unit, and the shape here follows it:
- the value types sit beside the trait;
- each profile with nothing to answer says why in its doc comment;
- the fake answers what a test sets;
- the conformance group gains scenarios below its marker, and each rig
  says whether its ecosystem has the thing in question.

Every projection built so far has no manifest: the file and image
projections (C.11) and the moon cache. npm and Cargo, which have
manifests, are later units. The trait is implemented eleven times in the
tree, and every implementation gains the two operations in this change:
- the three profiles;
- the fake;
- `Nowhere`, the unbound projection in `tailrace-seams`;
- `Watched` in `tailrace-core/tests/world` and in
  `tailrace-conformance/src/world.rs`;
- `Offering` and `Metaless` in the core's tests;
- `Carrying` in the suite's cache group.

The suite's statement list holds clauses 1 to 125, read at blueprints
`5b00e15`. Clause 140 came in with blueprints `4c14235`, so a scenario
cannot cite it yet (the coverage check refuses a citation of a clause
the list does not hold).

## Goals / Non-Goals

**Goals:**
- The seam's two new operations, their types, and an answer from every
  implementation.
- Conformance scenarios that a later npm or Cargo profile passes or
  fails by its own ordering and its own manifest.

**Non-Goals:**
- Any change to `tailrace-core/src`. The loop that uses these answers
  belongs to the units that resolve ranges.
- Choosing which kinds of dependency an ecosystem's manifest counts
  (runtime, development, optional). That belongs to each ecosystem's
  projection unit (see D4).

## Decisions

### D1 One clause, two operations: requires and satisfying

```rust
pub struct Requirement { pub name: EcosystemName, pub range: Range }

fn requires(&self, held: &Artifact) -> Answer<Vec<Requirement>>;
fn satisfying(&self, range: &Range, versions: &[Version]) -> Answer<Vec<Version>>;
```

`Range` is a new text type, made with the crate's `text!` macro,
holding the range as the manifest spells it. Both types derive what
`Pinned` derives.

140 asks for two things: what a held version requires, and whether a
version satisfies a range. They take different inputs, so they are two
trait methods. Splitting them does not split the clause: both cite 140,
and the spec names them as a pair.

satisfying takes many versions and answers the ones that satisfy,
newest first, rather than answering yes or no for one version. 137 ends
"the ranges and their order the projection's", and 23 forbids tailrace
from ordering versions by spelling or by listing. The core has to pick
"the newest version the policy admits within" a range, so it needs the
ecosystem's ordering, and this one answer carries it. The core then
walks the list and holds the first version the policy admits.

Alternatives considered:
- `satisfies(version, range) -> bool`, with the core ordering versions
  itself, by semver. That puts an ecosystem's rule in the core (core.1)
  and gets ordering wrong for ecosystems whose versions are not semver
  (Maven, Python).
- A third operation, `newer(a, b)`. That is one more call for each pair
  of versions, for no gain over the ordered answer.

### D2 requires takes the held artifact, not its bytes

The core passes the artifact the layout holds, its digest and metadata,
as render already takes them. A projection with a manifest reads the
artifact's bytes through the store seam it was bound with, as the moon
cache reads its entries (`profiles/projection/mod.rs` `bound`; rule
core.2). It never reads them from the layout it rendered (48: a layout
is no source of truth).

Alternative: the core reads the bytes and passes them in. That reads
every held artifact, an image's included, for ecosystems that have no
manifest to find in them.

### D3 None is an empty list, not absent

For pinned, 137 says "absent"; for these operations, 140 says "none",
and the two words differ on purpose. A projection with no manifest has
read nothing and failed nothing. Its true answer is that the artifact
requires nothing, and an empty list is that answer. It is also the
answer that ends the core's loop. core.19 rules out an empty list for a
failed read, and none of these is a read.

The three profiles answer `Ok(Vec::new())` to both operations, with a
doc comment saying their ecosystem has no manifest. They reach no store
and no file. No helper is added, since there is no `Not` to build.

### D4 Every name of the ecosystem, not only the family's

requires answers every name of the projection's own ecosystem that the
manifest requires, with its range, in the order the manifest gives
them. A projection cannot know which names the family publishes,
because only the store knows what it holds. So the store's listing
decides that (138): the core lists each name, and a name the store
holds no version of is absent, as most of a lockfile's pins are
(render.rs's own rule). This is how this change reads 140's "the
family's names" (see Open Questions). Which dependency kinds a manifest
counts is the ecosystem's own rule for what an install resolves, and
each profile's unit writes it in its own doc comment.

### D5 The fake: requirements by digest, Semantic Versioning for satisfying

The fake's state gains `manifests: BTreeMap<Digest, Vec<Requirement>>`,
which a test sets with `FakeProjection::manifest(&Digest,
&[Requirement])`.

requires answers what was set for the artifact's digest. For any other
artifact it answers `Not::absent(source, "the manifest of
<name>@<version>")`, following the wording of the fake's "the archive
of <build>". That is the intent's "the fakes answer absent": an
artifact the test gave no manifest is one whose manifest the fake
cannot find.

satisfying parses the range with the workspace's `semver` crate
(`VersionReq::parse`). It refuses an unreadable range with `Why::Said`
and the crate's error text (59). It drops any version `Version::parse`
cannot read, keeps those `req.matches`, and sorts them by `semver`'s
precedence, newest first. With that, a test of the later core loop can
list versions out of order over the fakes and get the right answer.
semver is not one of the names core.1 rules out, and the fakes crate is
not the core.

`ProjectionOp` gains `Requires` and `Satisfying`. Each is recorded with
`given` naming what it was asked (`{"held": ...}`, `{"range": ...,
"versions": ...}`) and can be faulted.

### D6 The conformance rig

`ProjectionRig` gains:

- `has_manifest() -> bool`;
- `held() -> (Artifact, Vec<Requirement>)`: an artifact of the ecosystem
  held where the projection reads it, and what its manifest requires,
  as spelled (none where the ecosystem has no manifest);
- `ranged() -> (Range, Vec<Version>, Vec<Version>)`: a range, versions
  listed in an order that is neither the ecosystem's nor spelling
  order, and those that satisfy the range, newest first.

The fake's rig sets the manifest for its held artifact's digest: a
scoped name with a caret range, and a plain name with a range carrying
a prerelease. It answers `^1.2` over `1.9.0, 2.0.0, 1.10.0, 1.2.0,
1.1.9` with `1.10.0, 1.9.0, 1.2.0`. The file, image and moon cache rigs
answer `false`, their own `held()` artifact, and an empty `ranged()`.

The scenarios, below the marker:
- "required as the manifest spells it", citing 50 and 58;
- "satisfied by the ecosystem's own ordering", citing 23;
- "no manifest, none", citing 58.

The module's doc comment says these cite in place of 140 until the
statement list holds it, as it already says for 137. The register is
not changed, because no clause enters or leaves it.

The cli crate's `PACK` and `CACHE` filters gain "no manifest, none".
The file, image and moon cache tests expect the two manifest scenarios
as not applicable, with the reason "the ecosystem has no manifest". The
fake's two runs in `tailrace-conformance/tests/fakes.rs` expect "no
manifest, none" as not applicable, with the reason "the ecosystem has a
manifest".

### D7 The pass-through and test projections

The two `Watched` projections pass both operations through to their
inner fake. `Offering`, `Metaless` and `Carrying` answer none. `Nowhere`
answers `nothing(Seam::Projection, "requires")` and `"satisfying"`, as
it does for its other operations.

## Risks / Trade-offs

- [The fake's ordering is Semantic Versioning, so a core test over the
  fakes could pass where npm's or Maven's rules would order differently.]
  → The suite's ordering scenario takes its expected answer from each
  profile's rig. A later npm or Cargo profile is held to its own
  ecosystem's order, not the fake's.
- [Adding two required methods to the trait breaks every implementor at
  once.] → All eleven are in the tree, and every one is named in the
  tasks. Nothing outside the workspace implements the trait.
- [Reading "the family's names" as every name of the ecosystem puts more
  names on the core's loop than the family publishes.] → Each extra name
  costs one listing that answers absent. That is the same cost the
  lockfile's third-party pins already pay.

## Open Questions

- What 140's "the family's names" excludes. D4 answers every name of the
  ecosystem and lets the store's listing decide. If the design means the
  projection should filter, for example to the dependencies a Cargo
  manifest marks with the family's registry, that changes the npm and
  Cargo units, not this seam's shape.
