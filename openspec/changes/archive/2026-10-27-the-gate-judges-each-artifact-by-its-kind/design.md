## Context

Two pieces of work meet in this change, on the tree this unit's branch
holds: this bolt, with main 293b4a8 taken into it by da0abc6.

- The gate, `crates/tailrace-core/src/gate.rs`, from main
  (the-gate-accepts-under-the-policy). `accept` reads the index, then
  `Judging::count` takes every artifact the index names against every
  claim `Required::claims()` lists, in order, and refuses at the first
  wanting (28). `Judging::promote` runs the same `count` under each
  following channel's policy. A verdict counts only carried in a parsed
  `Provenance` body whose build names the verdict's (13, 111).
- The notes, `crates/tailrace-core/src/notes.rs` and `publish.rs`, from
  this bolt (a-version-carries-its-notes). A publish names an artifact of
  ecosystem `notes` in the index after the outputs' and attaches it one
  provenance claim signed by the publisher, `build: None`, whose body is
  the notes' own and no `Provenance` (134, core.9).

On that tree every version a policy asks a verdict of is refused naming
its notes: the notes' claim parses as no `Provenance`, so no verdict is
found for them. Sluice #62 on the bolt shows it: the gate's tests in
`core:test`, the gate group in `conformance:test` and the publish that
tells the gate in `cli:test` all refuse with the verdict of `test`
missing for the notes' digest.

The policy seam answers `Required` with a flat `Vec<RequiredClaim>` per
audience (`crates/tailrace-seams/src/policy.rs`); the policy file profile
(`crates/tailrace/src/profiles/policy/file/mod.rs`) reads each audience's
`require` entries, `kind`, `check` and `signers`, with
`deny_unknown_fields`. C.10 names the claims each audience requires and
writes no form for which artifacts a claim is asked of.

## Goals / Non-Goals

**Goals:**

- The policy's answer says, per required claim, which artifacts it is
  asked of; the gate reads that and decides none of it (core.7, core.31).
- The gate tells each artifact's making from the claims attached alone
  (29), and judges acceptance and every following channel by it.
- No change to what a refusal names: the artifact, the claim found
  wanting and why (28, 58).

**Non-Goals:**

- A key in the policy file for the authors to choose which artifacts a
  claim is asked of. C.10 does not write one; the profile reads it from
  the kind until the design does.
- Consumer layouts and `verify`. A layout holds one ecosystem's artifact
  and never the notes (`render.rs` judges the artifact of its ecosystem),
  and `verify` reports which claims count without refusing; neither reads
  the new field.
- Changing the notes' claim, its body or what publish writes.

## Decisions

### D1. The required claim names the artifacts it is asked of

`RequiredClaim` gains `asked_of: AskedOf`, defined with the seams:

```rust
/// 134: the artifacts a required claim is asked of: every artifact a
/// version names, or a built artifact alone, which leaves a rendered
/// artifact, one no build made, unasked.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AskedOf {
    #[default]
    Every,
    Built,
}
```

Its serde form is `#[serde(default, skip_serializing_if = "AskedOf::is_every")]`,
so a required claim asked of every artifact writes exactly what it wrote
before. That keeps the publisher's layout-policy claim body
(`render::layout_policy_body`, signed and attached at publish) byte for
byte what it was, and a body written before reads back the same.

The gate's own reading of an artifact is a second enum in the core,
`Made { Built, Rendered }`, with `AskedOf::asks(made)`: `Every` asks both,
`Built` asks `Built` alone.

*Alternatives.* Two lists on `Required::Acceptance` and
`Required::Channel`, one for built artifacts and one for rendered ones:
it reads closest to 134's sentence, but the two lists can disagree on a
signer (`named_in`, `gate_named` walk `claims()`), and the policy's order,
which fixes the first claim found wanting, would be kept twice. A method
on `ClaimKind` deciding it in the seams: it takes the choice from the
policy, which 134 gives it, and puts it in a type every profile shares.

### D2. The policy file reads it from the kind

The profile answers each `[acceptance]` entry, each channel's `require`
entry and the given channels' gate acceptance with `asked_of` from the
entry's kind: `Every` for `provenance`, `acceptance` and `promotion`, the
publisher's word and the claims about the version as a whole, which a
claim on the index satisfies (7); `Built` for `verdict`, `test`,
`review`, `rebuild` and `carried`. `[layout]` entries are `Every`. No key
is added, and `Entry` keeps `deny_unknown_fields`.

134 says what a rendered artifact requires is "the publisher's provenance
alone"; `acceptance` and `promotion` are asked of it only because they are
claims about the version, satisfied once on the index, and never ask
anything of the notes themselves. `review` is a build's claim here
because 134's "alone" leaves it out.

### D3. A rendered artifact is told by its counted provenance naming no build

The gate tells each artifact's making once, in `accept`'s step 5, under
the acceptance policy at the head, and keeps it beside the artifact's
claims in `Judging.artifacts`:

1. The signers: those of the acceptance policy's required claims of kind
   `provenance`, each within its reach.
2. Each envelope attached to the artifact itself (never the index), whose
   claim is a `provenance` naming the version's revision, told through
   those signers with `tell`, counting and not expired.
3. Any such claim naming a build identity: `Built`. At least one and
   none naming a build: `Rendered`. None: `Built`.

The making is read from the claims attached alone (29) and only from a
provenance that counts (core.7): a build-less claim from a signer the
policy does not name for provenance makes nothing rendered. Where the
policy asks no provenance at all, nothing is rendered and every artifact
is asked every claim, the stricter reading, since the publisher's word is
what 134 lets stand for a rendered artifact.

*Alternatives.* The index's ecosystem `notes`: it names one kind of
rendered artifact rather than the way it was made, and is the index's
word, not a signer's. A mark in the index metadata: a new field in a
form the store holds, and again the index's word.

### D4. Acceptance and each following channel judge by the making

`Judging::count` takes, for each artifact, the required claims whose
`asked_of.asks(made)`, in the policy's order; the first wanting refuses
the whole version naming the artifact, the claim and why, as today.
`promote` calls the same `count` under the channel's policy and so
judges by the making told under acceptance: a channel's policy rarely
asks a provenance itself (the given channels ask the gate's acceptance),
and telling the making again under it would judge the notes as built.
The already-accepted path (94) tells the making too, since a following
channel still judges the version.

`Judged` and `Moved` keep their shapes; the acceptance body's `claims`
list names the notes' provenance among the envelopes that satisfied it.

### D5. The policy command marks a claim asked of built artifacts

`commands/policy.rs`'s `claim` renders a `Built` claim with `, of built
artifacts alone` after its signers, and nothing for `Every`. `--json`
prints the core's answer, which now carries `asked_of` for a built-only
claim.

### D6. The record

One ADR, 0022 in `docs/adr/`, the next number on this tree, records D1 to
D3: the policy names per claim which artifacts it is asked of, the file
reads it from the kind until C.10 writes a form for it, and the gate
tells a rendered artifact by its counted provenance naming no build. The
self-hosting guide's section on the gate says the same in a reader's
words.

## Risks / Trade-offs

- [The publisher's provenance decides that an artifact needs no verdict]
  → It is the signer 134 trusts alone for a rendered artifact, and only
  its counted word counts; a stolen publisher key could already offer
  bytes with that same word. The ADR says so.
- [The unit's merge carries main into the bolt, so its diff against the
  bolt holds main's paths beside this change's] → The planner agreed to
  it; `## Touches` names only what this change edits, and the paths main
  brings are read as main's.
- [Every place that makes a `RequiredClaim` must name `asked_of`] → A
  mechanical edit across the fakes, the conformance suite and the tests;
  the compiler finds each.

## Migration Plan

None beyond the code: no policy file changes, and no claim already in a
store is read differently, since a required claim naming nothing reads as
asked of every artifact.
