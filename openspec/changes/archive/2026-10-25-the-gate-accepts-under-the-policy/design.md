# Design

## Context

The core publishes and verifies; nothing accepts. The seams the gate
needs are all built: the store's `claims-of`, `attach` and `name`, the
signer's `sign` and `verify`, the landing's `landed`, the policy's
`policy-for` per audience, and the ledger seam's `append` and `chain`
(the `Keeper` trait), each with its fake. `verify` already tells, for
each attestation on a version, which named signer made it and whether
its reach lets it count (`crates/tailrace-core/src/verify.rs`, `tell`).
`repair::follow` already moves a channel's store name to the digest the
chain's head promotion is about, reporting a lost race (109). The
conformance suite's gate group is empty, waiting for this unit, and the
register names 25 to 30, 94 to 96, 107 and 108 for it
(`crates/tailrace-conformance/src/register.rs`).

The bolt's four units are built side by side (phase-1-bolts.md, How the
units stay apart). The ledger and promotion unit builds `tailrace
promote`, rollback, bound channels and the ledger's profiles in
`crates/tailrace-core/src/ledger.rs` and `profiles/ledger/`; this unit
owns `crates/tailrace-core/src/gate.rs` and must not reach into either.

## Goals / Non-Goals

**Goals:**

- One core function the host runs as the gate, judging and accepting a
  version and moving the channels that follow each publish, through the
  seams alone (core.1, core.2, core.31).
- Every answer machine-readable through `answer_json` (57).

**Non-Goals:**

- No change to any seam trait or seam type, to publish, or to verify's
  answer: a unit of this bolt changes no shared type (phase-1-bolts.md).
- No command and nothing that runs the gate (130, C.16).

## Decisions

### D1. One module, one entry: `gate::accept`

`accept(seams, history, ask) -> Answer<Judged>`, where the ask is
`AcceptAsk { branch, component: Option<ComponentName>, version, told:
ToldBy, gate: SignerId, when: u64 }`. `ToldBy` is `Publish` (a publish's
answer told the gate) or `Asked` (an asker or a rule asked it), the two
ways 95 names. `gate` is the identity the gate signs as, the host's
`[signing] signer` on the gate's host. `when` is the moment of the act,
in seconds since the Unix epoch, handed in so the core reads no clock
and its tests are fixed.

`Judged` is `Accepted { component, version, index, revision, policy_at,
acceptance, already, one_host, channels }` or `Refused { component,
version, index, revision, policy_at, artifact, wanting, why }`, both
`Serialize`. `channels` is `Vec<(ChannelName, Answer<Moved>)>`, each
following channel answered on its own, as publish answers each
component. `Moved` is `Promoted { from, to, promotion }`, `There { at }`,
or `Held { wanting, why }` for a channel whose own policy refused.

A refusal of a version is a verdict, not a seam's refusal, so it is a
value of `Judged` rather than a `Not` (glossary, verdict; 58). Absent and
unreadable stay `Not`. This keeps the refusal's form in the gate's own
module and adds no `Why` variant to the seam crate.

Alternative: a `Why::Wanting` in `tailrace-seams`. Rejected: it changes
a shared type in a bolt whose units are built apart, and a verdict is not
a seam operation's answer.

### D2. The order of a judgment

1. The branch's head through `History::head`, landed through
   `Landed::check` (107, 77); the components through `Declared::read` at
   that head, the component chosen as verify chooses it.
2. The version's tag resolved in the store to the index (core.4).
3. The tag's revision from the history at the head, landed through
   `Landed::check` (29, 14).
4. The policy for acceptance through `policy_at` at the head (107), not
   at the version's revision. verify reads at the version's revision
   (65, verify's D10) because it answers what held when the version was
   made; the gate answers what binds now.
5. The index read as an index (unreadable otherwise), then the claims of
   the index and of each artifact through `claims-of`.
6. Already accepted? An acceptance claim on the index whose maker the
   signer seam tells is `ask.gate` answers `Accepted { already: true }`
   with that claim's digest, nothing attached (94). Channels are still
   followed when told by a publish (D7), which a repeat publish (18)
   leaves where they are.
7. Each artifact against each required claim, in order (D3); the first
   wanting refuses the version (28).
8. The acceptance claim signed (D5), then attached to the index. Signing
   comes before the one write, so a gate whose identity is not at hand
   writes nothing.
9. Following channels, when told by a publish (D6, D7).

### D3. Counting reuses verify's telling, with the gate's own joins

`verify::tell` becomes `pub(crate)` and is called by the gate unchanged:
it asks each bound signer which named signer made an envelope and
whether that signer's kinds and checks let it count (9, 80, 116,
core.7). The gate then adds what acceptance needs and verify does not
judge:

- **The revision.** A claim counts only when it names the version's
  revision (13); a claim of another form, which carries no revision, is
  never counted.
- **The build.** A verdict counts only carried in the artifact's
  provenance (`Provenance::of`), for the check required, with the
  provenance's build identity (13, 111).
- **Where it is attached.** A claim on the artifact, or on the index,
  which a claim about the version is (7), counts for the artifact.
- **Expired.** A maker reported expired does not count at acceptance
  (81); verify, which reads after, keeps counting it (117).
- **The rebuild** (30). A rebuild claim counts only when its maker is not
  the maker of the artifact's provenance and its body holds `matched:
  true`. The rebuild's form is the verifier's, built in phase 2; this is
  the reading the gate takes until then.
- **The offerer** (27, 96, 108). Where the counted provenance's maker is
  `ask.gate`, the gate refuses `OfferedByGate` unless that maker's
  `NamedSigner` lists both `provenance` and `acceptance`.

`why` is one of `Missing`, `Expired { signer }`, `OfferedByGate { signer
}` and `Unmatched { signer }`, read beside `wanting`, the required claim
as the policy answered it.

Alternative: count in the gate from scratch. Rejected: two counters of
one rule could drift, and core.7 is one rule.

### D4. The acceptance claim is on the index alone

One claim, kind `acceptance`, subject the index, `revision` the version's
revision, `build` none (a version has no one build; core.9 is satisfied
by the revision), body:

```json
{
  "component": "repository",
  "version": "0.2.0",
  "policy": { "audience": "acceptance", "revision": "<head>" },
  "claims": ["sha256:…", "sha256:…"],
  "offered": { "by": ["host"], "one_host": false }
}
```

`claims` are the envelope digests that satisfied each required claim,
for every artifact, in order. `offered.by` names the provenance makers;
`one_host` is true when the gate accepted what its own identity offered
under a policy naming it for both (108).

Alternative: an acceptance on every artifact. Rejected: one judgment is
one claim (7), and 28 makes the index's acceptance cover every digest it
names (the reconstruction's recommendation, Where designs tie).

### D5. The accepting credential is the gate's signer

The gate signs by asking each bound signer to `sign` as `ask.gate`,
taking the first that answers; every one answering absent is absent
naming the identity. That is the accepting credential: the key a policy
names for acceptance, held only where the gate runs (95, 96, ADR 0007).
What the store lets the gate's host write is the host's credential (64),
unmodelled here. The gate's path imports nothing of publish, and publish
nothing of the gate, so no code path holds both (core.14).

### D6. A channel follows each publish by a key the core reads

`[channels.<name>]` with `on_each_publish = true` follows each publish
of its component, named by `component = "<name>"`, or the one component
where the repository declares one. The gate reads these keys itself with
`History::blob` at the head it judged at, as `Declared` reads the core's
own tables, since the policy file profile leaves a channel's keys other
than `require` and `bound_to` to their reader (policy-file spec; ADR
0013). A following channel naming `bound_to`, or naming no component
where several are declared, or naming one not declared, is unreadable
naming its table (37, 125). The self-hosting guide shows the keys.

```toml
[channels.beta]
on_each_publish = true
require = [{ kind = "acceptance", signers = ["gate"] }]
```

Alternatives: a field on the policy seam's `Required::Channel`, read by
the policy file profile. Cleaner as a seam, and the likely home once
the bolt has landed, but it changes a seam type the ledger and promotion
unit also reads while both are built apart. The reconstruction's
`asker: after-publish` names the same thing in a form the file does not
use.

### D7. The gate moves a following channel through the ledger seam

For each following channel of the version's component, in the file's
order, only when `told` is `Publish`:

1. The version judged under `Audience::Channel(<name>)` at the head,
   exactly as acceptance is (D3), its claims now including the acceptance
   just attached. Wanting: `Held`, the channel unmoved.
2. `chain(<name>)`: absent is an empty chain (from nothing, no
   predecessor); a fork is the keeper's refusal, answered (105); the
   head's promotion claim gives `from`, its subject, and the predecessor,
   the head envelope's digest. A head already about the index is
   `There`, nothing written.
3. A promotion claim signed as the gate (D5): kind `promotion`, subject
   the index, the version's revision, body `{ channel, from, to,
   asked_by: "each publish", policy: { audience, revision }, claims,
   when, predecessor }` (33, 105).
4. `append(<name>, promotion, predecessor)`; a refusal is the channel's
   answer and nothing after it is done (35, core.34).
5. `attach(index, promotion)` (39).
6. `repair::follow(store, keeper, <name>, Name(<name>), from)`, which
   moves the name from where the previous head left it and reports a
   lost race (109, core.6). The ledger holds the move before the name
   moves (33, core.15).

The channel's store name is its name, as `repair` and the ledger group
already use it; a channel's name is unique in the policy file, so two
components' channels never share one.

The ledger and promotion unit makes promotion claims at a person's
asking. The body above follows the reconstruction's ledger entry, so the
two can be one form; whichever lands second takes the other's.

### D8. Conformance

Scenarios below the gate group's marker, over the `FakeWorld`: S3
(accept, attached, bytes the same, `beta` moved with the ledger first),
S5 (an unnamed signer refused, nothing written), S9 (a thief's
acceptance, a build host running the gate, a host with no gate key, and
a publish alone), and one each for 26 to 30, 94 to 96, 107 and 108. They
cite only those clauses and S-numbers they play, so the gate takes its
block out of the register and touches no other unit's lines. The world
gains append-only helpers to commit and land a policy file and to judge
as a given identity.

## Risks / Trade-offs

- [Two places make promotion claims in one bolt] → D7 uses one body form
  and says so; the second unit to land aligns to the first, which the
  settle or merge stage of the later one catches.
- [The core reads two channel keys from the file beside the policy seam]
  → the same precedent as `Declared`; moving them into `Required::Channel`
  later changes the reader and the guide, not the behaviour.
- [`on_each_publish`, `component` and the rebuild's `matched` are forms
  this change picks] → named to the design agent; a different spelling
  changes `gate.rs`, its tests and the guide alone.
- [An acceptance attached and a following channel then refused] → the
  version stays accepted (94), the channel's answer says why, and a later
  promotion moves it.
