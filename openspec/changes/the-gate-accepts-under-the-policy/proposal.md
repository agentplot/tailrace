# Proposal

## Why

A publish offers a version and nothing more (20): nothing yet accepts
one, so no version is ever accepted and no channel ever moves. Bolt 4
needs the gate: the one party that judges an offered version under the
policy and, where the policy says `beta` follows each publish, moves
`beta` itself, so a family repository's `beta` moves on every landing
and a stolen build credential still moves nothing (S3, S9; ADR 0007).

## What Changes

- **The gate judges an offered version.** Asked with the branch, the
  component and the version, and handed the identity it signs as, the
  gate resolves the version's name to its index (core.4), checks the
  landing landed the version's revision, and reads the policy for
  acceptance at the governed branch's landed head at that moment, never
  at the version's own revision and never from a checkout (107, 29, 65).
- **Accepted whole or refused whole.** Every artifact the index names is
  judged against each required claim in the policy's order, counting
  only claims a signer the policy names may make (9, 116). One artifact
  found wanting refuses the version whole, naming the artifact and the
  first claim found wanting and why (28, 58). An accepted version gets
  one acceptance claim, signed in the gate's identity and attached to its
  index, and no byte of it changes: the index and every artifact keep
  the digests they were offered with (26, 7). A refused version stays
  offered and readable by digest, and nothing is written (S5).
- **The accepting credential.** The gate signs only through a signer that
  holds its identity at hand; where none does, it answers absent naming
  the identity before anything is written. A version whose provenance
  the gate's own identity made is refused, the gate never accepting what
  it offered (27, 96), apart from the one host 108 admits.
- **The one host.** A policy names one host as both offerer and gate by
  letting one signer claim both provenance and acceptance (108, 116).
  When that host accepts what it offered, its acceptance claim says so,
  so a reader tells a one-host acceptance from a separated one.
- **An independent rebuild is counted, never made.** A required rebuild
  claim counts only from a signer other than the one whose provenance the
  artifact carries, and only where it says the digests matched (30).
- **Nothing accepted is un-accepted.** A version already holding the
  gate's acceptance is answered accepted again with that claim, and
  nothing is attached (94).
- **`beta` follows each publish as the gate's act.** When the gate is
  told of a version by a publish's answer and accepts it, it moves every
  channel of the version's component that the policy file marks as
  following each publish: the version judged under that channel's own
  policy, a promotion claim signed by the gate naming its predecessor
  appended to the channel's chain through the ledger seam, attached to
  the index, and only then the channel's name moved in the store (95,
  32, 33, 39). A publisher never moves it (20, core.10): publish is
  unchanged and calls no part of the gate.
- **A host with no accepting credential moves nothing.** A gate run where
  the gate's identity is not at hand, or a forged acceptance signed by a
  build host's key, leaves every channel where it was; what was offered
  sits unaccepted and attributable to the signer that offered it (S9).
- The conformance suite's gate group gains its scenarios, and clauses
  25 to 30, 94 to 96, 107 and 108 leave the register (core.33).

Out of scope, and what each waits on:

- **A command.** No `tailrace accept`, and nothing that runs the gate
  after a publish, is added; the core starts nothing after an offer
  (130), and what tells the gate is the host's to bind (C.16). It waits
  on a unit the planner names.
- **Promotion at a person's asking, rollback and bound channels.**
  `tailrace promote`, a move back, and a channel bound to another (34,
  37) are the ledger and promotion unit's of this bolt; a following
  channel that is also bound is refused here, never promoted from a
  publish (37).
- **The ledger's profiles.** The git ref and the store as ledgers (C.13,
  C.14) are that unit's too; the gate appends through the ledger seam
  and its fake alone.
- **A channel requiring a separated acceptance** (108, last clause) waits
  on the ledger and promotion unit, which judges a channel's policy at a
  person's asking; the gate writes what that policy would read.
- **Who asked, as a verified identity** (129) waits on promotion at a
  person's asking; the rule that follows each publish is named as the
  asker here.
- **The key service signer** (C.7) is its own unit; the gate signs
  through whatever signer the host binds.

## Capabilities

### New Capabilities

- `acceptance`: the gate's judgment of an offered version under the
  policy for acceptance at the governed branch's landed head, the
  acceptance claim it attaches, the one-host acceptance, and the move of
  each channel that follows each publish.

### Modified Capabilities

- `conformance-suite`: the gate's group holds the gate's scenarios
  rather than waiting empty for this unit.

## Impact

- `crates/tailrace-core`: a new module, `gate`, exported from the core,
  reusing verify's telling of who made an attestation; no change to
  publish, verify's answer or any seam trait or type.
- `crates/tailrace-conformance`: the gate group's scenarios below its
  marker, the gate's block taken out of the register, and the suite's
  table and its empty-group test brought up to date.
- The policy file gains two channel keys the core reads, `on_each_publish`
  and `component`, which the policy file profile already leaves to their
  reader; the self-hosting guide shows them and the one-host signer.

## Touches

```
crates/tailrace-core/src/gate.rs
crates/tailrace-core/src/lib.rs
crates/tailrace-core/src/verify.rs
crates/tailrace-core/tests/gate.rs
crates/tailrace-conformance/src/core/gate.rs
crates/tailrace-conformance/src/register.rs
crates/tailrace-conformance/src/lib.rs
crates/tailrace-conformance/src/world.rs
site/src/content/docs/guides/self-hosting.mdx
```
