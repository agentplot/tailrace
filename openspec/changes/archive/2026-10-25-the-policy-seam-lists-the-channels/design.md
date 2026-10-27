# Design

## Context

The policy seam has one operation, `policy_for(revision, audience)`
(`crates/tailrace-seams/src/policy.rs`), implemented by the policy file
(`crates/tailrace/src/profiles/policy/file/mod.rs`), the fake
(`crates/tailrace-fakes/src/policy.rs`), the seams' own `Nowhere`
(`crates/tailrace-seams/src/lib.rs`) and two `Watched` test policies
(`crates/tailrace-core/tests/verify.rs`,
`crates/tailrace-conformance/src/core/verify.rs`). The policy file reads
`[signers]`, `[acceptance]`, `[layout]` and `[channels.<name>]`'s
`require` and `bound_to`; it leaves every other key to its reader.

Two readers already give a channel's table more meaning. The core's
`Declared` (`crates/tailrace-core/src/declared.rs`) reads
`[[component]]` and names the one component of a repository that
declares none `repository`. The gate (`crates/tailrace-core/src/gate.rs`,
`following`; its design D6) reads `component` and `on_each_publish`
from `[channels.<name>]` itself, unreadable where a following channel
names no component among several or one not declared. So the file
form of a channel's component is settled: `component = "<name>"`,
optional where one component is declared.

The suite's list of clauses (`crates/tailrace-conformance/src/statement.rs`)
was read at agentplot/blueprints 5b00e15 and holds 1 to 125, while the
statement at 673d334 holds 126 to 131 as well and names B.8 the ledger.
The coverage check finds a scenario citing a number outside the list
(`CitesNoClause`), so no scenario can cite 131 until the list is
brought up to the statement.

## Goals / Non-Goals

**Goals:**

- One seam question, `channels`, answered alike by every policy
  profile and held to by the suite.
- The file profile's reading of a channel's component matching the
  gate's, so the two cannot disagree on one file.

**Non-Goals:**

- Changing the gate to ask the seam. Its own reading stays; the two
  agree by the rules below, and moving it is a later change.
- Pattern channels (127) and anything of `require`.

## Decisions

### D1. The operation and its type

```rust
/// 131: a channel the policy declares, the one component it belongs to
/// (125), and the channel it is bound to where it is one (37, 83).
pub struct DeclaredChannel {
    pub name: ChannelName,
    pub component: ComponentName,
    pub bound_to: Option<ChannelName>,
}

pub trait Policy {
    fn policy_for(&self, revision: &Revision, audience: &Audience) -> Answer<Required>;
    /// 131: every channel the policy at `revision` declares, by name.
    fn channels(&self, revision: &Revision) -> Answer<Vec<DeclaredChannel>>;
}
```

`DeclaredChannel` derives what the seams' types derive and serialises
with `bound_to` as `null` where unbound. The answer is ordered by
channel name, so every profile answers one list for one policy; the
file's own order is the gate's to keep, not the listing's. No default
body: a policy profile that does not answer it does not build.

Alternatives: the channels as a field of a whole-policy answer, which
would make every reader parse every audience; an `Audience::Channels`
variant of `policy_for`, which would make `Required` answer something
that is not a requirement.

### D2. The one component's name lives with the seams

`pub const REPOSITORY_COMPONENT: &str = "repository";` sits in
`crates/tailrace-seams/src/artifact.rs` beside `ComponentName`, and
`crates/tailrace-core/src/declared.rs` keeps `REPOSITORY` as
`pub use tailrace_seams::REPOSITORY_COMPONENT as REPOSITORY;`, so every
caller of `declared::REPOSITORY` is untouched. The fake and the file
profile name the one component by it, never by their own literal, and
no profile imports the core (core.2).

### D3. The file profile's reading

`File` gains `component: Vec<ComponentEntry>` (`name` only; serde
ignores the core's other keys), and `AudienceTable` gains
`component: Option<ComponentName>`. One function resolves a channel's
table against the file, used by both operations:

1. The declared components: the `[[component]]` names, or
   `[REPOSITORY_COMPONENT]` where there are none.
2. The channel's component: its `component`, which must be one of
   them; or, absent, the one where exactly one is declared. Otherwise
   unreadable naming `[channels.<name>]` and the components declared,
   in the gate's words where the gate has words for it.
3. Its `bound_to`, where named: a channel of `[channels]`, resolved by
   steps 1 and 2 to the same component; otherwise unreadable naming
   the table and the channel named.

`channels` resolves the revision first, as `policy_for` does (core.4),
answers `Ok(vec![])` where the tree holds no policy file, parses the
whole file as `policy_for` does (so a file one cannot read the other
cannot either, with the same words), and resolves every channel; the
first channel that fails makes the answer unreadable. `policy_for` of a
channel audience resolves that one channel the same way before it
builds the requirement. Acceptance and layout are unchanged. Unreadable
names `{POLICY_FILE} at {revision}` as `what`, as today, with the table
in the words.

A whole answer unreadable for one bad channel, rather than a list of
answers per channel: a reader of the listing (status) has no channel
it can trust from a file whose channels disagree, and the words name
the table to fix.

### D4. The fake's channels

`PolicyOp::Channels` beside `PolicyFor`; `PolicyState` gains
`belongs: BTreeMap<ChannelName, ComponentName>` and
`belongs_at: BTreeMap<(Revision, ChannelName), ComponentName>`, set by
`FakePolicy::belongs(channel, component)` and
`belongs_at(revision, channel, component)`. `channels(revision)`
answers, through `ask` so it is recorded and faultable, one entry for
each channel audience held at every revision or at `revision` (the
revision's policy over every revision's, as `policy_for` reads them),
its component from `belongs_at`, then `belongs`, then
`REPOSITORY_COMPONENT`, and its `bound_to` from the held `Required`.
Existing tests that script channel policies keep their channels in
`repository`, the world's one component. The fake does not check a
binding's component; it answers as scripted, as every fake does.

### D5. The rig and the scenarios

`PolicyRig` gains:

```rust
/// A landed revision at which the repository declares `components`
/// (none: the one component of its own) and each of `channels`, in the
/// component it names, bound where it is bound and requiring no claim.
fn declaring(&self, components: &[ComponentName], channels: &[DeclaredChannel]) -> Revision;
```

The fake rig scripts `require_at` with `Required::Channel { claims:
vec![], bound_to }` and `belongs_at` for each channel. The file rig
writes a `[[component]]` for each component with `name`, `outputs =
[]`, `paths = ["<name>/**"]` and `prefix = "<name>-v"` (every key the
core requires), and each `[channels.<name>]` with `require = []`,
`bound_to` where bound, and `component` only where several components
are declared, so the one-component scenario reads a file that names
none. `unreachable` on the fake rig faults `Channels` as well as
`PolicyFor`.

Below the policy group's marker, in this order:

| scenario | cites |
|---|---|
| the channels a policy declares | 131, 125, 37, 83 |
| one component has every channel | 131, 125, 123 |
| a policy declaring no channel | 131 |
| the channels of a policy that cannot be reached | 131, 58, 59 |

The first declares `lib` and `web`, `beta` and `stable` (bound to
`beta`) of `lib`, `edge` of `web`, asserts the answer equals the three
in name order, and asks `policy_for` of `stable` for the same binding.
The second asks at two revisions, none declared and `lib` alone, each
naming the component the rig passes. The third uses `at` with
acceptance only and asserts `Ok(vec![])`. The fourth uses the group's
`unreadable` helper.

### D6. The statement adopted at 673d334

`READ_AT` becomes `agentplot/blueprints 673d334`, and `PARTS` takes
the statement's order there: 127 and 128 after 37 in A.6; 129 after 60
in A.10; 131 after 83 in B.5; 126 after 91 in B.7; B.8 named "The
ledger"; 130 after 118 in B.9. The statement at 673d334 numbers a
second clause 124, in A.9 (the compilation's cache); the list keeps 124
once, in A.4, where 5b00e15 placed it, and the test asserts 1 to 131
once each in 20 sections. The duplicate number is the statement's to
fix (core.23); see Risks.

The register drops 125, which the first two scenarios cite, and gains:

| clause | unit | why |
|---|---|---|
| 126 | phase 2 · the first target | the roster is a target's, and phase 1 leaves every target to phase 2 |
| 127 | phase 2 | a channel declared by a pattern over branch names, which no unit of phase 1 carves |
| 128 | phase 2 | several promotions in one act, which no unit of phase 1 carves |
| 129 | phase 2 | the asker named by an identity the owner's identity provider issued, and phase 1 binds none |
| 130 | phase 2 | what runs a publish and tells the gate, named in the host's bindings file, which no unit of phase 1 carves |

`TARGET` already names "phase 2 · the first target"; a `PHASE_2`
constant names the rest, as section 12 lets a phase stand while it has
no units carved.

### D7. The core asks at a landed revision

`crates/tailrace-core/src/landed.rs` gains
`channels_at(policy: &dyn Policy, landed: &Landed) -> Answer<Vec<DeclaredChannel>>`
beside `policy_at`, so a reader in the core asks only at a revision the
landing landed (65, core.22). Nothing calls it in this unit; the status
and ledger units do.

## Risks / Trade-offs

- [The ledger unit, built side by side, edits the register and plans to
  move 125 "to this unit's block"] → this unit takes 125 out; whichever
  lands second keeps 125 out and its other edits, and the conductor's
  merge resolves the one line.
- [The statement numbers two clauses 124] → the list keeps the first;
  the duplicate goes to the design agent to renumber, and the list
  changes in the commit that adopts the fix.
- [The gate and the profile read `component` separately] → D3 follows
  the gate's rules and words; one file read both ways answers alike.
  Moving the gate onto the seam removes the second reading later.
- [Every implementor of `Policy` must change at once] → there are five,
  all listed in the proposal's Touches; the workspace does not build
  until each has `channels`.
