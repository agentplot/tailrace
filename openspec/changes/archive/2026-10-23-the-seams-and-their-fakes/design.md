# Design

## Context

The scaffold (ADR 0011) left five crates as sluice's, each a doc comment
and nothing else. `tailrace-seams` depends on `anyhow` and `serde`;
`tailrace-fakes` on `tailrace-seams`, `tailrace-core`, `anyhow` and
`serde_json`. Each crate is a moon project whose `test` task is tagged
`check-fast`; format and lint are the root project's tasks over the whole
tree. sluice's `sluice-seams` and `sluice-fakes` are the model this
follows where the statement does not differ: synchronous traits taking
`&self`, every crossing type defined in the seams crate, and fakes that
are clonable views of shared in-memory state.

The seams are Part B (66 to 91, 120 to 122) and the binding is 92. This
unit is the first of bolt 1's four after the OpenSpec root; the three
after it code against what it lands, and no later profile unit may
change a trait (phase-1-bolts.md, How the units stay apart).

## Goals / Non-Goals

**Goals:**

- Every operation Part B names, and no other, as a trait method whose
  signature carries what the clause says it is given and answers.
- Absent, refused and unreadable kept apart by the type system, so no
  code path can turn one into another or into an empty value without
  writing it out (58, core.19).
- Fakes good enough that the conformance suite holds them as it will
  hold a real profile, and that the core's tests can script every
  failure the core must answer.

**Non-Goals:**

- What a claim of each kind says (provenance, acceptance, promotion):
  the units that make those claims fix their bodies.
- How the core chooses among bound signers, or orders versions (23).
- Real signatures: the fake signer proves only that a claim was not
  changed and which named signer made it.

## Decisions

### D1. The answer is `Result<T, Not>`, and `Not` is not an error

Every seam method returns `Answer<T>`, an alias for
`Result<T, Not>`, where `Not` is `Absent(Absent)`, `Refused(Refusal)` or
`Unreadable(Unreadable)`. `?` works in the core over `Answer`, and the
compiler holds every caller to the three arms. `Not` deliberately does
not implement `std::error::Error`, so `?` cannot fold it into an
`anyhow::Error` and lose which of the three it was. `anyhow` leaves the
seams crate's dependencies.

Each of the three names its `Source`: the seam (an enum of the eight)
and the profile's name. `Refusal` carries `why`, the refusal Part B
names, and `said`, the source's own words verbatim where it gave any
(59). `Unreadable` carries `what` could not be read and `said`.
`Absent` carries `what` was not there.

`Why` holds the refusals the seams answer, each with what its clause
says it names: `NameAt { at: Option<Digest> }` (70), `NotHead { head:
Option<Digest> }` (120; none when the chain is empty), `Unmade {
outputs }` (74), `NoNamedSigner { checked }` (80), `Fork { predecessor,
claims }` (105), and `Said` for a refusal that names nothing beyond the
source's words. The core's units after this one add the refusals the
core makes (2, 4, 14, 15, 22, 28); a profile unit adds none.

Alternatives: sluice's `anyhow::Result` with refusals as values, which
leaves the core free to answer `Err` as "unknown" or swallow it into an
empty list, the two things core.19 rules out; a four-armed enum of its
own, which loses `?`.

### D2. One module per seam, the shared types beside them

`tailrace-seams/src/` holds `answer.rs` (D1), `digest.rs`, `claim.rs`
(claim, envelope, signer identities), `artifact.rs` (revision, branch,
component, output, build identity, version, ecosystem, artifact and
index metadata), then `store.rs`, `build.rs`, `landing.rs`,
`signer.rs`, `policy.rs`, `projection.rs`, `target.rs`, `keeper.rs`,
each holding its trait and the types only it passes, and `bind.rs`
(D8). `lib.rs` re-exports all and carries the seam table, as
`sluice-seams` does. Every type derives `Serialize` and `Deserialize`
(57). The names are the statement's: `Store`, `Build`, `Landing`,
`Signer`, `Policy`, `Projection`, `Target`, `Keeper`.

Traits are synchronous and take `&self`, with no `Send` or `Sync`
bound, as sluice's. A profile that needs threads (a served layout, 86)
wraps itself; a trait change for it would be a unit of its own.

### D3. A digest is `sha256:` and the hex of the bytes

`Digest::of(&[u8])` computes SHA-256 with the `sha2` crate, added as a
workspace dependency, and the digest's text is `sha256:<64 lowercase
hex>`. `Digest::parse` takes only that form and rejects any other,
naming the text. One algorithm is enough for every store Part C names;
a second is an addition to the parse, never a change to `of`. A digest
serializes as its text.

### D4. An artifact and an index are both manifests the store names

`put-artifact` (67) takes a `Manifest`: `Artifact { meta, layers }`,
the artifact's metadata and the digests of its bytes, or `Index(meta)`,
whose metadata names a component, a version and every artifact by
digest and metadata (6). An index is an artifact (section 3) whose
"bytes" are the artifacts it names. `ArtifactMeta::new` rejects an
empty ecosystem, name or version (4); `IndexMeta::new` rejects an
artifact whose version is not the index's (110). The store chooses the
artifact's digest; the fake takes the digest of the manifest's
canonical JSON.

`read` (72) answers `Held`: `Bytes(Vec<u8>)` for what put-bytes held,
or `Manifest(Manifest)` for what put-artifact held. Part B's read says
only bytes; status and verify must read an index back to name its
artifacts, and a seam reached through nothing else must answer it. This
is the one place the signature says more than its clause, named in the
change's final message for the design agent.

### D5. A claim is an in-toto statement, an envelope is DSSE's shape

`Claim { subject: Digest, kind: ClaimKind, revision: Revision, build:
Option<BuildId>, body: serde_json::Value }` (7, 13). `Claim::payload`
writes it as an in-toto statement (subject the digest, predicate type
the kind, predicate the revision, build and body), the form section 9
fixes for every attestation and that core.1 lets the core name, so
every signer profile signs the same bytes. `Envelope { payload_type,
payload, signature: Signature { signer: SignerId, sig } }` is DSSE's
shape with exactly one signature (7). `Envelope::digest` is the digest
of its canonical JSON (105, 120). `Claim::from_envelope` reads a claim
back and answers `None` for an envelope another party made in another
form, such as a landing's verdict, which is carried as made (12) and
never decoded into a tailrace claim. `body` stays JSON so the later
units fix each kind's fields without changing this type.

### D6. Each operation's signature

| seam | operation | given | answers `Ok` |
|---|---|---|---|
| store | `put_bytes` | bytes | `Put { digest, held_already }` |
| | `put_artifact` | `&Manifest` | `Put` |
| | `attach` | subject digest, `&Envelope` | `Put` of the envelope |
| | `claims_of` | digest | `Vec<Envelope>`; absent when not held |
| | `name` | `&Name`, to, expected `Option<&Digest>` | `()`; refused `NameAt` |
| | `resolve` | `&Name` | `Digest`; absent when it names nothing |
| | `read` | digest | `Held` (D4); absent when not held |
| build | `ensure` | revision, `&[OutputName]` | `()`; refused `Unmade` |
| | `report` | revision, output | `Report { build, outcome, files }` |
| landing | `landed` | `&Branch`, revision | `bool` |
| | `verdicts_of` | revision | `Vec<Verdict>`, empty when none |
| signer | `sign` | `&Claim`, `&SignerId` | `Envelope`; absent when no identity |
| | `verify` | `&Envelope`, `&[NamedSigner]` | `Made { signer, expired }`; refused `NoNamedSigner` |
| policy | `policy_for` | revision, `&Audience` | `Required`; absent when none declared |
| projection | `pack` | `&Output`, version | `Packed { bytes, meta, same_again }` |
| | `render` | `&[Artifact]`, where | `Rendered { layout, holds }` |
| | `serve` | `&Layout` | `Serving { address }` |
| target | `desired` | `&Environment` | `Digest`; absent when the channel is empty |
| | `current` | `&Environment` | `Digest`; unreadable with the reason |
| | `apply` | `&Environment`, `&Verified` | `()` |
| | `report` | `&Environment` | `Running { digest, converged_at }` |
| keeper | `append` | channel, `&Envelope`, predecessor `Option<&Digest>` | `Digest` of the claim held; refused `NotHead` |
| | `chain` | channel | `Vec<Envelope>`; absent when none; refused `Fork` |

`Required` is an enum by audience, so acceptance has no field for a
bound channel (83): `Acceptance { claims }`, `Channel { claims,
bound_to: Option<ChannelName> }`, `Layout { claims }`. Each required
claim names its kind, the check it is for where it is a verdict, and
the `NamedSigner`s that may make it; a `NamedSigner` is the signer's id,
what verifies it (an opaque text its profile reads, a public key or a
key's name), and the kinds and checks it may claim (82). `Made.expired`
is set when the identity has expired (81). `Verified` is the index's
digest and version the reconciler verified; only the core makes one
(42, core.29). A when (`converged_at`) is an RFC 3339 text, so the
seams take no clock dependency.

### D7. Fakes are clonable views of shared state, recording every call

Each fake (`FakeStore`, `FakeBuild`, `FakeLanding`, `FakeSigner`,
`FakePolicy`, `FakeProjection`, `FakeTarget`, `FakeKeeper`) is
`Clone` over `Rc<RefCell<…>>`, so a test keeps one handle and binds
another. Each has scripting methods for what its seam holds and a
`calls()` that returns, in order, a typed record of each operation
asked, what it was given and the answer (`Call<Op>` with the fake's own
operation enum).

One script mechanism is shared by every fake (`script.rs`):
`fault(op, Fault, Times)`, where a `Fault` is refused or unreadable with
the source's words, and `Times` is once or until `heal(op)`. A faulted
operation answers the fault before touching state, so a refused put
holds nothing. Every fault names the source as the fake's seam and the
profile `fake`.

What each fake holds and what it can be scripted to do beyond faults:

- store: blobs, manifests, attachments and names in maps; `corrupt(d,
  bytes)` holds other bytes under `d` (2).
- build: per revision and output, a `Report` or unmade; `ensure`
  answers `Unmade` naming every unmade output asked (74).
- landing: landed revisions per branch, verdicts per revision.
- signer: the identity at hand, the identities that have expired. A
  signature is the digest of the signer's id and the payload, so a
  changed payload fails verify and no key is used.
- policy: `Required` per audience, for every revision or one.
- projection: its ecosystem and the name each output takes. Pack is
  the canonical JSON of the ecosystem, name, version, output name and
  file paths, read from no disk; `unsteady()` makes each pack differ
  and answer `same_again: false` (87). Render writes nothing and
  answers the versions given; serve answers a `fake://` address.
- target: per environment, the channel's head or empty, what it runs or
  unreadable; `apply` sets what it runs and stamps a scripted when.
- keeper: per channel a chain; append checks the predecessor against
  the head's envelope digest; `fork(channel, a, b)` holds two claims
  naming one predecessor, and chain then refuses `Fork`.

### D8. The binding is a struct of boxed traits

`Seams` holds `store`, `build`, `landing`, `policy` and `keeper` as one
`Box<dyn …>` each, `signers` as a list, `projections` keyed by ecosystem
and `targets` keyed by environment name (92). It lives in the seams
crate because 92 is a Part B clause; how the host's bindings file fills
it (118) is bolt 2's. `tailrace-fakes` gives `Fakes::new()`, every fake
with a handle kept, and `Fakes::seams()` binding clones of them.

### D9. The fakes stop depending on the core

`tailrace-fakes` drops its `tailrace-core` and `anyhow` dependencies
and its moon project's `dependsOn: core`, since the binding is in the
seams. The core's own tests in the next unit can then use the fakes as
a dev-dependency without the core being compiled twice.

### D10. The fakes are not Part C profiles

core.3 asks an ADR of a seam with a single profile. The fakes stand in
for profiles in tests and pass the suite as one must (93), but bind no
host and are no member of Part C; they count for no seam under core.3.

## Risks / Trade-offs

- [The later units find an operation's signature short] → each later
  core unit in this bolt may add a `Why` variant or a field the
  statement backs; a change to a trait's operations is a unit of its
  own, landed before the profile units that need it.
- [`read` answering a manifest is wider than 72] → named for the design
  agent; if the statement adds an operation for it, the method splits
  without changing what the fakes hold.
- [Synchronous traits for a networked store] → as sluice's; a profile
  blocks on its own client.
- [The fake signer is no cryptography] → it is a fake; signatures are
  the signer profiles' (bolts 2 and 4), held to the suite's signer
  scenarios.
