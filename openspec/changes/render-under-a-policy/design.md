## Context

`tailrace sync` today is `cache::sync`: the build identities a landed
revision's passing verdicts name, each rendered into moon's outputs
directory, reading no policy (`crates/tailrace-core/src/cache.rs`,
`crates/tailrace/src/commands/sync.rs`). The projection seam has pack,
render and serve (`crates/tailrace-seams/src/projection.rs`); render
already takes `Contents::Artifacts` and answers `Holds::Versions`, but
no projection renders artifacts yet. The policy file profile answers
`policy_for(revision, Audience::Layout)` from `[layout]`, absent where
the file has none (ADR 0013). Verify tells each attestation on an index
and its artifacts by the named signer that made it (`verify.rs`,
`tell`), and reaches no verdict. The host's record of published
versions lives under the common git directory (`published.rs`).

The statement this rests on: 47 to 51, 61 to 65, 85, 99, 113 and S4 at
`agentplot/blueprints` e267a30, with the design agent's rulings there:
the pointer names the version's index and is spelled
`artifact/<ecosystem>/<name>@<version>` (138); pinned is the projection
seam's (137); the publisher's layout policy rides the index as a claim
(113). ADR 0006 puts the policy at render, not at serve.

Two other units of this bolt build on this one: cargos-registry
implements pinned and render for Cargo, npm-rendered-and-served for npm
with the serving process. Bolt 4 (the gate and the channels) is not on
`main` and so not on this bolt; its gate has a judgment of claims under
a policy of its own.

## Goals / Non-Goals

**Goals:** pinned on the seam; a core render that resolves pins,
judges each version under the layout's policy and renders what passes;
`tailrace sync` filling layouts; the record of rendered layouts; the
publisher's layout-policy claim written at publish.

**Non-Goals:** any ecosystem's own pinned or render; writing the
`artifact/` pointer; serving; showing layouts in status.

## Decisions

### D1. pinned answers pins, and the core does the rest

`fn pinned(&self, checkout: &str) -> Answer<Vec<Pinned>>`, where
`Pinned { name: EcosystemName, version: Version }` is a seam type,
spelled as the lockfile spells it. A checkout with no lockfile of the
ecosystem is `Not::absent(source, "lockfile")`; the file, image and moon
cache profiles answer that for every checkout. The projection never
reaches the store, the policy or the version's claims (core.2): the
core resolves, judges and hands render the artifacts. Alternative: a
projection renders from its own lockfile reading; refused because the
policy would then be applied in eight profiles instead of one core.

### D2. A pin resolves through its index

The core spells the reserved space as it spells `cache/`:
`pub const ARTIFACT: &str = "artifact/"` and
`artifact_name(&Ecosystem, &EcosystemName, &Version) -> Name` giving
`artifact/<ecosystem>/<name>@<version>`, in `render.rs`. A pin is
resolved (`store.resolve`), the digest read (`store.read`) and must
hold `Manifest::Index`, whose artifacts must hold exactly one of that
ecosystem, name and version; anything else is unreadable naming the
name and the digest (core.4, as `record::indexed` does). Resolve's
absent is the pin's answer: absent naming the name. Most of a
lockfile's pins are third-party, so absent is the common answer and
never counts against the sync.

### D3. Which policy

The consumer's policy is read once per sync with `policy_at(landed,
&Audience::Layout)` at the revision sync runs for, which `Landed::check`
made (65, core.22). An answer is the policy for every layout. Absent
means each version is judged under the publisher's: the envelopes
`claims_of(index)` holds that read as `Claim` of kind
`ClaimKind::LayoutPolicy` about that index. Exactly one distinct body
is the policy; none is a refusal of the version naming the publisher's
layout policy as wanting (core.17: no layout rendered with no policy);
two distinct bodies are unreadable naming both envelopes' digests,
never one chosen (as core.34 refuses to choose in a fork). The
consumer's policy unreadable or refused leaves every layout unrefreshed
(D6). The answer and the record name the policy as
`{ "own": { "revision" } }` or `{ "publisher": { "revision" } }` per
version.

The publisher's claim is read as the store holds it: its body is the
policy, and the claims it requires then count only through the signers
it names (core.7). Nothing names a signer the consumer trusts to make
the policy claim itself; see Open Questions.

### D4. The verdict: every required claim, in order, on the index or the artifact

For each `RequiredClaim` in the policy's order, the version satisfies it
when an attestation on the index or on the pinned artifact, or a
verdict carried in that artifact's provenance, is told by `tell` as
made by a signer the claim names, allowed its kind and check (9, 116).
The first required claim with no such attestation is the refusal:
`Why::Wanting { version, index, wanting: RequiredClaim, checked:
Vec<SignerId> }`. An expired signer counts and is reported (117).
Claims are read from the index and the one artifact the layout takes,
which the ruling on 138 names as what a consumer reads ("nothing more
to look up"); the gate's acceptance of the version is on the index
(28). `tell` becomes `pub(crate)` in `verify.rs`; nothing else of verify
changes. Alternative: reuse bolt 4's gate judgment; it is not on this
bolt's base, and the two should be one function once both land.

### D5. A layout's place, and the core's one question for sync

The place is `<working tree>/.tailrace/layouts/<ecosystem>/`; sync
writes `.tailrace/.gitignore` holding `*` when it makes `.tailrace/`,
so the consumer's repository needs no ignore line. The consumer's own
configuration names the place by a path relative to its working tree,
the same on every host (49, core.18). Alternatives: under the git
directory, which a worktree's `.git` file makes a different path per
checkout; under the user's data directory, which names a home
directory per host.

`render::render(seams, &RenderAsk) -> Answer<Refreshed>` renders one
ecosystem's layout from given pins under a given policy source; it is
what a server calls for one pin on a miss (50, 115). `render::layouts`
asks every bound projection for pins at the working tree and calls
render per ecosystem. The command asks the core one question,
`render::sync_checkout(seams, &Layouts, &CheckoutAsk) ->
Answer<CheckoutSynced>`, which runs `cache::sync` unchanged and then
`layouts`, answering `{ branch, revision, entries, layouts }`. `SyncAsk`
and `cache::sync` keep their shapes, so the cache's scenarios above the
conformance marker are untouched. `CheckoutAsk` carries `when`, seconds
since the Unix epoch, the command's clock: the core reads none.

### D6. A layout that cannot be refreshed

Per ecosystem, every read comes before any write: pins, each pin's
resolve, read and claims, and the judgment. If the policy, or any of
those reads for one pin, is unreadable or refused by its source, that
layout is not rendered and not recorded anew; its answer is
`Kept { at, holds, rendered_at, policy, because: Not }` from the
record, or `holds` empty and `rendered_at` none where no record is
(51, S10). A pin answered absent or refused under the policy does not
stop the render. A render that fails answers the same way. Otherwise
render is given exactly the satisfied artifacts, so a version refused
now leaves the layout (47), and the answer is
`Refreshed { at, rendered_at, pins: Vec<(Pinned, Answer<HeldPin>)> }`.

### D7. The record

`Layouts` opens at the checkout's own git directory,
`<git dir>/tailrace/layouts/<ecosystem>.json`, one file per layout,
written beside and renamed as `published.rs` writes: ecosystem, place,
`rendered_at`, and per pin held `{ name, version, index, artifact,
policy }`, per pin refused `{ name, version, index, wanting, policy }`.
Absent pins are not kept: the lockfile and the store answer them again.
`History` gains `git_dir()`, the worktree's own (`git rev-parse
--git-dir`), beside `common_dir()`. Per checkout, because two worktrees
pin differently (50); under git, so no push carries it (62). Deleting
it loses nothing the next sync does not rewrite (63).

### D8. The publisher's claim at publish

After the index is put and before the version's name is moved, publish
asks `policy_at(landed, &Audience::Layout)`; on an answer it signs
`Claim { subject: index, kind: LayoutPolicy, revision, build: None,
body: { "claims": [...] } }` with the host's signer, as provenance is
signed, and attaches it to the index (8, core.8, core.9). Absent attaches
nothing; unreadable stops the publish before the name moves, as any
other read does. Where `claims_of(index)` already holds a layout-policy
claim of the same body, nothing is signed (18). `ClaimKind::LayoutPolicy`
is named `layout_policy`, predicate
`https://agentplot.github.io/tailrace/claim/layout_policy/v1`.

### D9. The answer and the exit

Text names each layout's place and, per pin, held with the index's
digest, refused with the claim found wanting and the signers checked,
and a count of absent pins; `--json` lists every pin. The exit is the
worst code among the entries' and pins' answers that are not absent,
with a layout that cannot be refreshed counting as its cause's code; 0
when everything not rendered is absent.

### D10. Tests

Unit tests in `crates/tailrace-core/tests/render.rs` over the fakes for
D2 to D7, and in `tests/publish.rs` for D8. Conformance: the projection
group gains the two pinned scenarios with a rig method saying whether
the ecosystem has a lockfile and staging a checkout with and without
one; the layouts group gains the scenarios of the spec, with the fake
world holding pins and `artifact/` names written by the test; the
register loses 47, 51, 99 and 113. The command line: `tailrace sync
--json` over the fakes in `crates/tailrace/tests/`. All run in
`check-fast` through the crates' existing `test` tasks.

## Risks / Trade-offs

- [The pointer is not written yet] → until the store-naming unit lands,
  a real sync finds no family pin and answers each absent; tests write
  the names themselves.
- [A layout judged under a publisher's claim the store alone vouches
  for] → a consumer that wants its own trust declares `[layout]`; the
  open question below goes to the design agent.
- [Two judgments of claims under a policy, this one and bolt 4's gate]
  → both use verify's `tell`; folding them into one is a chore once
  bolt 4 is on `main`.
- [Third-party pins make the answer long] → text counts absent pins;
  JSON lists them for a program.

## Open Questions

For the design agent:

- What makes a consumer with no policy trust the publisher's
  layout-policy claim: today it is read as the store holds it, and
  could be held to a signer the host's bindings trust (section 2 gives
  the operator which signers a host trusts).
- Whether the givens (C.10) should hold a policy for layouts, so a
  family repository with no `[layout]` line renders anything at all.
