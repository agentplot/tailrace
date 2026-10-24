# Design

## Context

Two units land before this one in bolt 1. `the-seams-and-their-fakes`
gives the eight traits, the types they pass, `Answer<T>` whose `Not`
keeps absent, refused and unreadable apart (its D1), the `Seams` binding
(D8) and a fake per seam that records every call (D7). Its `Claim` is an
in-toto statement with a JSON `body` the later units fix per kind, and
`Claim::from_envelope` answers `None` for another party's envelope (D5).
`a-landed-revision-takes-its-version` gives `Landed`, made only by the
landing seam's yes (its D2); `History` over the repository (D3);
`Declared` read from `.config/tailrace.toml` at the landed revision
(D4); `derive`, `touched` and `claim` (D5, D6), where a revision already
tagged derives its claimed version and a second claim at the same
revision is not an error.

This unit is the last core unit of bolt 1 before the conformance suite.
The binary's `crates/tailrace/src/` is the scaffold's `main.rs` and
`lib.rs` (ADR 0011); bolt 2's laptop unit binds real profiles under
`crates/tailrace/src/bindings/` and fills `commands/` against them
(phase-1-bolts.md, Bolt 2). sluice's binary is the model for the
command line: clap derive, a `--json` per command, the core called and
its answer printed.

The site's guide for Claude Code already shows the commands as
`tailrace publish <revision> --json`, `tailrace status --json` and
`tailrace verify <ref> --policy <audience> --json`; this unit takes
those shapes.

## Goals / Non-Goals

**Goals:**

- One publish path whose order is the statement's: landed, declared,
  ensured, judged, claimed, packed, put, attested, indexed, named,
  recorded, answered (14 to 19, 22).
- Every seam's absent, refused and unreadable answer passed up as it
  came, with the source's words (58, 59).
- Status a function of files alone, with no binding in its signature
  (60, core.32).
- Commands that only parse, call and render (core.31).

**Non-Goals:**

- The gate's verdict over a policy, acceptance and channels (bolt 4):
  verify tells signers and whether each claim counts, never whether a
  version is accepted.
- Rebuilding the record from the store and git (63, bolt 3).
- Real bindings and the host's bindings file (118, bolt 2).
- Concurrency between two publishers beyond the claim's refusal (22);
  a refused claim ends the publish (core.6).

## Decisions

### D1. Publish is one function over the binding, per component

`publish.rs`: `publish(&Seams, &History, &Kept, &Ask) -> Answer<Publish>`,
where `Ask { branch, revision, component: Option<ComponentName>, signer:
SignerId }` and `Publish { components: Vec<(ComponentName,
Answer<Published>)> }`. The steps, in order, each ending the component's
publish at its first `Not`:

1. `Landed::check(seams.landing, branch, revision)` (14). Its `Not` ends
   the whole publish; nothing else is asked.
2. `Declared::read(history, &landed)`, then the components: the one
   named, or `touched(...)` (124). A named component that derives
   `Unchanged` and holds no tag at the revision is answered
   `Published::Unchanged { last }`, with nothing ensured.
3. Each output's projection (D3), refused or absent before the build is
   asked.
4. `verdicts_of(revision)` once for the publish (78).
5. `ensure(revision, &outputs)` in one asking (74); then `report` per
   output (75).
6. Judge (D2): every output a pass, its identity named where verdicts
   exist (15, 16, 111).
7. `derive` then `claim` (22, core.13). This is after judging and before
   packing, so a refused version claims no name and no pack runs
   without a claimed name.
8. `resolve` the version's store name (D5). Held: read the index; this
   is a repeat (D6).
9. Per output: `pack(output, version)` (84); on a repeat, compare (D6);
   `put_bytes`, checked against `Digest::of` (D4); `put_artifact`.
10. Per artifact: the provenance claim (D7), `sign`, `attach` (8).
11. `put_artifact(Manifest::Index(..))` (6, 17), then `name(version's
    name, index, expected None)` (70, core.6).
12. `Kept::write` (D9), then the `Published` answer (D8).

Components are published in the order the policy file declares them, and
one component's `Not` does not stop the next (124). `Publish` maps to an
exit code by its worst answer (D10).

Alternatives: claiming the name first, as the earliest reading of 22
allows, which leaves a tag on a version whose outputs are then refused;
ensuring after the claim gains nothing, since the build never needs the
version (24).

### D2. Judging an output

For each output, with its `Report { build, outcome, files }` and the
landing's verdicts:

- `outcome` not a pass: refused `Why::NotPassed { output, build,
  outcome }` (16).
- Verdicts non-empty and none names `build`: refused `Why::Unnamed {
  output, build, carried }`, `carried` every build identity the verdicts
  name, deduplicated in the landing's order (15).
- Verdicts name `build` but none with a pass: refused `Why::NotPassed`,
  naming the verdicts' outcome (111).
- Verdicts empty: the build's pass alone (S13, C.6).

The first output that fails refuses the component (16); outputs are
judged in declared order so the answer is the same every time. The
verdicts kept for each output are every verdict naming its build, pass
or not, to be carried in its provenance (D7).

### D3. Each output names its projection in the policy file

`declared.rs` gains, per component, a `projections` table from output
name to ecosystem, and at top level the same for the one component the
repository is when it declares none (C.10):

```toml
outputs = ["site"]
[projections]
site = "file"

[[component]]
name = "lib"
outputs = ["lib-pack"]
paths = ["crates/lib/**"]
prefix = "lib-v"
projections = { lib-pack = "npm" }
```

`Component` gains `projection_of(&OutputName) -> Option<&Ecosystem>`.
An output with none is refused `Why::NoProjection { component, output }`;
an ecosystem `seams.projections` lacks is absent naming the ecosystem
with `Source::host`. The rest of `declared.rs` and its tests stand; a
projections key naming an output the component does not declare is
unreadable naming it, as an ill-typed key is (its D4).

### D4. Bytes are checked against their own digest

After `put_bytes(bytes)` answers `Put { digest, .. }`, publish compares
`digest` with `Digest::of(&bytes)`; a difference is refused
`Why::OtherBytes { expected, answered }` sourced to the store (2,
core.5). The artifact's digest is the store's (seams D4) and is not
recomputed.

### D5. The version's name in the store is its tag's

The version is named in the store by the component's tag name (prefix
and version, as `Derived` gives it), through the seams' `Name` for a
version. It points at the index (3) and is what verify resolves, what a
repeat publish reads (18), and what bolt 3 rebuilds the record from
(63). It is moved with `expected: None`, so it is made from nothing;
`NameAt` from the store on a first publish is passed up as refused
(core.6), never retried.

### D6. A repeat publish compares before it puts

When step 8 resolves the name, publish reads the index and, for each of
its artifacts, reads the artifact's manifest (`read` answers
`Held::Manifest`, seams D4). After each pack it compares, for the
artifact of the same ecosystem and ecosystem name, the held layer's
digest with `Digest::of(&packed.bytes)` and the held metadata with the
packed metadata, computing both without writing; an artifact's own
digest is the store's to compute, so it is not compared. One different:
refused `Why::Differed { artifact, first, now }`, naming the bytes'
digests, before any put, and nothing is written (18). An output the
index has no artifact for is different in the same way. All the same:
the puts are skipped, claims are
read with `claims_of` instead of signed again (D7), the index is the
held one, the name is not moved, and `Kept::write` finds its entry
equal and writes nothing (S2). The answer is built the same way as a
first publish's, so the two answers are equal.

Alternatives: putting again and relying on the store's idempotence (5),
which writes nothing for bytes but would sign and attach a second
provenance where a signer's signatures are not deterministic.

### D7. The provenance claim

`provenance.rs`: `ClaimKind::Provenance` (added to the seams' `claim.rs`
where the seams unit has not), with body:

```json
{
  "output": "lib-pack",
  "ecosystem": "npm", "name": "@acme/lib", "version": "1.3.0",
  "outcome": "pass",
  "carried": [
    { "found": { "seam": "landing", "profile": "fake", "revision": "…" },
      "check": "…", "build": "…", "outcome": "pass", "signer": "…",
      "envelope": { … as the landing recorded it … } }
  ]
}
```

The claim's `subject` is the artifact's digest, `revision` the landed
revision and `build` the report's build identity (13, core.9). `carried`
holds every verdict naming that build (D2), each envelope unchanged
(12): the core never decodes it, since its form is the landing
profile's (core.1). The check, build, outcome and signer beside it are
the landing seam's reading, carried as the landing answered them. The
claim is signed by `seams.signers[0]` with the ask's `SignerId` (79);
absent passes up and ends the component before its index (17). The
envelope is attached to the artifact's digest (68).

One claim carrying the verdicts, rather than a carrier claim of its own
per verdict, is 12's "inside the carrier's own claim": what tailrace
observed of the build and where it found the verdicts are one
observation of one publish (11).

### D8. The answer

`Published::Version { component, version, tag, revision, branch, index,
artifacts: Vec<PublishedArtifact>, store: Source }`, each
`PublishedArtifact { digest, bytes, ecosystem, name, output, build,
same_again, claims: Vec<Attached> }` and `Attached { kind, signer,
envelope: Digest, on: Digest }` (19). `Published::Unchanged { component,
last }` (D1). Everything derives `Serialize`; the JSON of an `Answer`
is `{"answer": "ok" | "absent" | "refused" | "unreadable", ...}` with the
value or the `Not` beside it, written once in `tailrace-core` as
`answer_json` so every command renders the same form (57).

### D9. The host's record lives in the repository's common git directory

`published.rs`: `Kept::at(&History)` is `<git common dir>/tailrace/
published/`, which every worktree of the repository on the host shares
and no push carries (62). One entry per version, `<component>/<tag>.json`,
holding `Entry { component, version, tag, revision, branch, index,
store }` (61). `Kept::write(&Entry)` writes a temporary file beside it
and renames, so a reader never sees half an entry; an entry that exists
with equal content is left untouched, and one with other content is
refused `Why::KeptOther { path }` (it cannot arise unless the store's
name moved behind tailrace's back). `Kept::read(filter) ->
Answer<Vec<Entry>>`: no directory, or no entry, is absent naming the
directory; an entry that does not parse is unreadable naming its path
with serde's words. Entries are ordered by component, then by semantic
version precedence of their version (23).

`status.rs`: `status(&Kept, &StatusAsk) -> Answer<Status>`, with
`StatusAsk { component, revision }`. It takes no `Seams`, so it cannot
reach a store, signer or target (60, core.32), and only reads.

The record is not `record.rs`, which bolt 3 owns for the push after a
landing and the rebuild (63). `Source::host(<path>)` names the record,
and the bindings, as a source.

Alternatives: a directory under the user's state home keyed by the
repository's path, which two clones of one repository on a host would
not share and which needs a key the common directory gives for free.

### D10. Verify

`verify.rs`: `verify(&Seams, &History, &VerifyAsk) -> Answer<Verified>`,
`VerifyAsk { branch, component: Option, version, audience }`:

1. The component: `Declared::read` at the branch's head
   (`History::head`, added to `history.rs`), once `Landed::check` says
   the branch holds it; then the one named, or the only one declared;
   several and none named is refused `Why::WhichComponent { declared }`.
   The tag is the component's prefix and the version.
2. `resolve` the tag name in the store (D5); absent names it.
3. The tag's revision from the history, read local and shared as
   `derive` reads tags; `Landed::check` of that revision; `policy_at`
   for the audience (65). Absent: absent naming audience and revision.
4. The signers: every `NamedSigner` the `Required` names, deduplicated
   by id.
5. `read` the index; for the index and each artifact, `claims_of`; for
   each envelope, ask each bound signer `verify(envelope, &signers)` in
   order until one answers `Made`; all refusing is `NoNamedSigner`; an
   unreadable answer ends verify unreadable. A provenance's `carried`
   envelopes are verified the same way.
6. `counts`: made by a named signer whose kinds include the claim's
   kind and, for a verdict, whose checks include the verdict's check
   (9, 116). Expiry is reported beside and does not change `counts`
   (81, 117).

`Verified { component, version, tag, revision, index, audience,
claims: Vec<Told> }`, `Told { on, envelope, kind, check, made_by:
Option<SignerId>, expired, counts, checked: Vec<SignerId>, carried_in:
Option<Digest> }`. Verify writes nothing; the fakes' records show no
put, attach, name or sign.

The policy is read at the version's own landed revision rather than at
the branch's head, so a verify of one version says the same thing
tomorrow; the gate's judgment at the head (107) is bolt 4's.

### D11. The command line

`crates/tailrace/src/lib.rs`: `Cli` (clap derive) with `publish`,
`verify` and `status`, and `run(&Cli, Option<&Seams>, &Path, &mut dyn
Write) -> i32`, which opens `History` at the path, resolves the
revision argument to a commit with the history (core.4), calls the core
and renders. `commands/publish.rs`, `commands/verify.rs` and
`commands/status.rs` each hold one command's argument struct and its
text rendering; `render.rs` holds the exit code and the JSON. Exit
codes: 0 an answer, 1 refused, 2 arguments (clap's), 3 unreadable, 4
absent; a `Publish` exits by its worst component.

The text form prints what was asked and the answer in the statement's
words; a refusal prints tailrace's reason, then the source's words on a
line of their own, unchanged (59).

`main.rs` passes `None` for the binding, since no host binds seams
until bolt 2: `publish` and `verify` answer absent, sourced to the
host, naming the seams unbound; `status` runs. `--version` keeps the
scaffold's line (ADR 0011). The signer identity of a publish is the
binding's to give; the tests give the fake's.

`--branch` defaults to `main`, the family's governed branch, until the
host's bindings file names one (118). `--policy` takes `acceptance`,
`layout` or `channel:<name>`, the seams' `Audience`.

### D12. Tests

- `crates/tailrace-core/tests/publish.rs`, `verify.rs`, `status.rs`: one
  test per spec scenario, over `Fakes` and the history fixture of the
  previous unit (`tests/common/mod.rs`), a bare shared repository and
  clones.
- `crates/tailrace/tests/cli.rs`: `run` over `Fakes::seams()` for the
  command-line scenarios, comparing printed JSON with the core's answer
  as a `serde_json::Value`; and the built binary
  (`env!("CARGO_BIN_EXE_tailrace")`) for status and the unbound host.

## Risks / Trade-offs

- [The seams' type names differ from this design's] → the design names
  the seams unit's decisions (its D4 to D8); the coder takes the names
  that landed and keeps the behavior.
- [The carried verdict's check name is the landing seam's reading, not
  read from the envelope] → decoding the landing's envelope is the
  landing profile's (core.1); the reading is carried inside tailrace's
  signed claim, and the gate (bolt 4) may re-ask `verdicts-of`.
- [The repeat compares bytes and metadata, not artifact digests] → an
  artifact's digest is the store's function of its metadata and layers,
  so equal bytes and equal metadata are the same artifact in any store
  that keeps 73.
- [The record under the git directory is lost with the clone] → it is
  the host's and rebuilt from the store and git (63, bolt 3).
- [`main` as the default branch] → a default only; every command takes
  `--branch`, and bolt 2's bindings name the host's.
