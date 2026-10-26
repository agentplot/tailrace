## Context

`tailrace sync` today is `cache::sync`: the build identities a landed
revision's passing verdicts name, each rendered into moon's outputs
directory, reading no policy (`crates/tailrace-core/src/cache.rs`,
`crates/tailrace/src/commands/sync.rs`). The projection seam has pack,
render and serve (`crates/tailrace-seams/src/projection.rs`); render
already takes `Contents::Artifacts` and answers `Holds::Versions`, but
no projection renders artifacts yet. The policy file profile answers
`policy_for(revision, Audience::Layout)` from `[layout]`, absent where
the file has none; nothing of the givens (C.10) is built yet. The host's
bindings file (`crates/tailrace/src/bindings/`) binds `[store]`,
`[landing]` and `[signing]` and names no signer the host trusts. Verify tells each attestation on an index
and its artifacts by the named signer that made it (`verify.rs`,
`tell`), and reaches no verdict. The host's record of published
versions lives under the common git directory (`published.rs`).

The statement this rests on: 47 to 51, 61 to 65, 85, 99, 113, 116, 118,
137, 138, C.10 and S4 at `agentplot/blueprints` d91401e, with the design
agent's rulings there: the pointer names the version's index and is
spelled `artifact/<ecosystem>/<name>@<version>`, and the store lists the
versions a name holds (138); pinned is the projection seam's (137); the
layout's policy is stacked, the consuming repository's own where its
file writes a layout line, else the publisher's claim where the index
carries one, else the given, a consumer with no file and one with no
layout line alike (113); the given policy for a layout admits a version
the gate accepted and no other, and the givens name no signer, who may
claim what being the repository's file's alone, with the host the
one-host gate where it names none (C.10, 116); a consuming repository
with no policy of its own counts the publisher's layout-policy claim,
and every claim it reads, through the signers its host's bindings trust
(C.10); a sparse index leaves a refused version out and sync prints the
refusal (99). ADR 0006 puts the policy at render, not at serve.

Two other units of this bolt build on this one: cargos-registry
implements pinned and render for Cargo, npm-rendered-and-served for npm
with the serving process. Bolt 4 (the gate and the channels) is not on
`main` and so not on this bolt; its gate has a judgment of claims under
a policy of its own.

## Goals / Non-Goals

**Goals:** pinned on the seam; a core render that resolves pins,
judges each version under the layout's policy and renders what passes;
the given policy for a layout; the signers a host trusts; `tailrace
sync` filling layouts; the record of rendered layouts; the publisher's
written layout policy carried as a claim at publish.

**Non-Goals:** any ecosystem's own pinned or render; a manifest's range
where no pin exists, and the store's listing of a name's versions that
resolves it; writing the `artifact/` pointer; serving; the
givens as a fragment and the given acceptance and channels; showing
layouts in status.

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

### D3. Which policy: the consumer's line, the publisher's, the given

The consumer's policy is read once per sync with `policy_at(landed,
&Audience::Layout)` at the revision sync runs for, which `Landed::check`
made (65, core.22). Its answer is `Required::Layout { claims, given }`
(D11): `given` false is a layout line the file writes, which judges
every version of every layout; `given` true is the file's given, kept
for the last step below; absent, a revision holding no policy file,
leaves no given of the file's. In both of the last two the consumer
writes no layout policy, and each version is judged as follows (113).

The publisher's policy is read from the envelopes `claims_of(index)`
holds that read as `Claim` of kind `ClaimKind::LayoutPolicy` about that
index, and only those `tell` says a signer the host trusts made
(C.10, 118, core.7): the trusted signers (D12) are passed to `tell` as
the signers that may make a layout-policy claim. A layout-policy claim
by any other signer is kept and never counted (9). Exactly one distinct
body among those counted is the policy; two distinct bodies are
unreadable naming both envelopes' digests, never one chosen (as core.34
refuses to choose in a fork).

Every claim the publisher's policy requires then counts only through a
signer the host trusts too: each required claim keeps, of the signers
the body names, those whose id the host trusts, each verified by the
host's key for it and allowed the kinds and checks the body gives it.
A required claim none of whose signers the host trusts keeps an empty
list, and so is the claim found wanting (C.10: "refuses a layout whose
claims count through none, naming the claim"). The narrowed claims are
judged as D4 judges any policy's.

Where no layout-policy claim counts, the version is judged under the
given (C.10): the file's given where the consumer has a policy file,
its gate resolved by the profile from the file's `[signers]` (D11),
verified by the keys the file names, and not narrowed to the host's
trusted signers, since who may claim what under the given is the
repository's file's alone (116); and where the consumer has no policy
file, acceptance by the signers the host trusts, each allowed
acceptance, since such a consumer names no signer and counts every
claim it reads through its host's trusted signers (C.10, 118). A host
that trusts no signer then refuses every version naming acceptance
with no signer checked. There is always a policy, so no layout is
rendered with none (core.17).

`policy_for(own, trusted, version, index, on_index)` answers the
policy and the claims to judge; the consumer's policy unreadable or
refused leaves every layout unrefreshed (D6). The answer and the record
name the policy per version as `{ "own": { "revision" } }`,
`{ "publisher": { "revision" } }` or `{ "given": { "revision" } }`, the
last's revision the consumer's landed revision where it has a policy
file and null where it has none. Alternatives: count the publisher's
claim as the store holds it, refused, since C.10 holds it to the host's
trusted signers, and a store any host can write to would otherwise
choose a consumer's policy; let the consumer's given win over the
publisher's claim, refused, since 113 stacks the publisher's above the
given for a consumer that writes no layout line.

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
since the Unix epoch, the command's clock, so the core reads none, and
`trusted`, the host's trusted signers (D12); `RenderAsk` carries
`trusted` too.

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
asks `policy_at(landed, &Audience::Layout)`; on an answer the file
writes, `given` false, it signs `Claim { subject: index, kind:
LayoutPolicy, revision, build: None, body: { "claims": [...] } }` with
the host's signer, as provenance is signed, and attaches it to the index
(8, core.8, core.9). The given, or absent, a revision with no policy
file, attaches nothing: 113 has the publisher's claim carry the claims
"as that repository declared them", and a consumer that finds none lays
the given beneath itself (D3). Unreadable stops the publish before the
name moves, as any
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
D2 to D7, and in `tests/publish.rs` for D8. The given (D11) is tested in
the policy file profile's own tests over a scratch repository, and
`[trust]` (D12) in the bindings module's tests. Conformance: the projection
group gains the two pinned scenarios with a rig method saying whether
the ecosystem has a lockfile and staging a checkout with and without
one; the layouts group gains the scenarios of the spec, with the fake
world holding pins and `artifact/` names written by the test; the
register loses 47, 51, 99 and 113. The command line: `tailrace sync
--json` over the fakes in `crates/tailrace/tests/`. All run in
`check-fast` through the crates' existing `test` tasks.

### D11. The given policy for a layout

The policy seam's `Required::Layout` gains `given: bool`, serialized
only when true, so the core can tell a layout line the file writes from
the given beneath it (113); no other audience's answer changes. The
policy file profile answers `Audience::Layout` for a file with no
`[layout]` table with the given (C.10): `Required::Layout { claims: [{
kind: acceptance, check: None, signers: <the gate> }], given: true }`,
where the gate
is every `[signers]` entry whose `kinds` holds `acceptance`, in the
file's order; where none does and exactly one entry's `kinds` holds
`provenance`, that one, as the one host 108 admits; and otherwise none,
so every version is refused naming acceptance with no signer checked,
since 108 admits no gate. The givens name no signer: the gate is read
from the repository's file alone (116, C.10), as the gate's unit reads
the given gate for acceptance. A file's own `[layout]` is answered as
written, `given` false, and the given is not laid beneath it. A
revision holding no
policy file answers absent naming the policy file, as today. The given
is built in the profile, beside the table it stands in for, as one
function of the parsed file; the fragment the policy command shows, and
the given acceptance and channels, are the gate's and the channels'
units (C.10). The policy group's scenario of an absence asks a channel
no file declares, `nightly`, in place of a layout, which the given now
answers; its words and the clause it cites are unchanged.
Alternatives: the core lays the given beneath an absence, refused,
since which signers may claim acceptance is in `[signers]`, which only
the profile reads, and the seam has no operation answering it; the
profile answers absent for a file with no `[layout]`, refused for the
same reason, since the core could then not build the file's given.

### D12. The signers a host trusts

The bindings file gains `[trust.<signer>]`, one table per signer the
host trusts, each naming exactly one key whose string is what the
signer's profile verifies it by, in the policy file's `[signers]` form
without `kinds` and `checks`:

```toml
[trust.gate]
ssh = "ssh-ed25519 AAAA… gate@box"
```

A table naming no key, or two, or anything but a string, is unreadable
naming the file and the table; a file with no `[trust]` trusts no
signer. `Bindings` gains `trust: Vec<NamedSigner>`, each with the
id, its verifier, `kinds = [layout_policy]` and no checks, and
`Binding` carries it to the command, which passes it on the ask (D5).
The core allows a trusted signer acceptance only where it stands as the
gate of a consumer with no policy file (D3).
The key is a public key, so nothing kept is a credential (64, core.21).
No default trust is given, not even the host's own `[signing]`
signer: its public half would be read from the private key file, and
a consumer that has a policy of its own needs no trust at all.
Alternative: trust named in each repository's policy file; refused,
since the consumer this serves has no policy file, and section 2 gives
which signers a host trusts to the operator.

## Risks / Trade-offs

- [The pointer is not written yet] → until the store-naming unit lands,
  a real sync finds no family pin and answers each absent; tests write
  the names themselves.
- [A consumer with no policy file and no `[trust]` refuses every
  version] → the refusal names acceptance and no signer checked, so the
  reader sees what to write; a consumer with a policy file is judged
  under its own file's gate instead.
- [A consumer whose file names its own gate and no layout line refuses a
  family version its publisher's gate accepted, where the publisher
  writes no layout line] → the given counts acceptance by the consumer
  file's gate alone (116); the consumer lists the family's gate in its
  `[signers]`, or the publisher writes a layout line its index carries.
- [The given admits nothing where a file names several provenance
  signers and no acceptance signer] → every version is refused naming
  acceptance, as 108 refuses acceptance itself.
- [Two judgments of claims under a policy, this one and bolt 4's gate]
  → both use verify's `tell`; folding them into one is a chore once
  bolt 4 is on `main`.
- [Third-party pins make the answer long] → text counts absent pins;
  JSON lists them for a program.

## Open Questions

For the design agent:

- Which versions of a listed name fall within a manifest's range is the
  ecosystem's (137), and no projection operation answers it; B.1 names
  no listing operation beside 138's sentence. This unit leaves ranges
  out; the unit that resolves them needs both.
- The given for a consumer with no policy file counts acceptance through
  the signers its host trusts (C.10, 118). Read instead as the host
  being the one-host gate (C.10), it would count the host's own signer,
  whose public key this bolt cannot read.
- The given for a consumer whose file writes no layout line takes its
  gate from that file, not narrowed to the host's trusted signers (116),
  where C.10's sentence on a consumer with no policy of its own could be
  read as narrowing every claim it reads.
