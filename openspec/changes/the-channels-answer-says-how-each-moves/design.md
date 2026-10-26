# Design

## Context

This unit builds on the bolt as it stands after
`the-ledger-and-promotion` landed (bolt 04ceec2), which this worktree's
branch does not yet hold: the code stage takes the bolt in first.

What the bolt has: the policy seam answers `channels` as a list of
`DeclaredChannel { name, component, bound_to }` read from
`[channels.<name>]` tables, each naming its component with a
`component` key (`the-policy-seam-lists-the-channels`). The gate does
not use that answer: `crates/tailrace-core/src/gate.rs` parses the
policy file from history itself (`channel_tables`, `ChannelKeys`,
`following`) to find `on_each_publish` channels and, in `gate_named`,
the signer it accepts as. The ledger's `promote`
(`crates/tailrace-core/src/ledger.rs`) reads the channel's policy for
claims and binding and lets a signer promote only where the channel's
`require` names it with kind `promotion`. The keeper seam, the store
name `channel/<channel>`, the git ref `refs/tailrace/ledger/<channel>`
and `Audience::Channel` all take a `ChannelName` alone, so two
components' `beta` would share one chain.

The statement as amended at agentplot/blueprints 7749369 (125, C.10,
C.13, C.14) makes a channel's identity its component and its name,
fixes the three spellings, gives every component `beta` and `stable`,
and has a given channel promoted by the gate's signer. 85bbf54 and
bd772be give an environment's `policy` and `after` words; the book's
`environments.md` names the waiting environment's channel for the
environment ("`prod` follows `prod`, bound to `stable`").

## Goals / Non-Goals

**Goals:**

- One answer, the policy seam's, that every reader of channels uses.
- A channel carried as component and name on every seam that takes one.
- No spelling changes for a repository that declares no component.

**Non-Goals:**

- Moving an `after` channel by rule, or converging an environment
  (phase 2's target).
- A key naming who may ask, or who may say yes to a manual
  environment; none is designed (C.10's "later word").
- The core's reading of `[[component]]` (`Declared::read`), which
  versions need (123) and which is no channel; it stays as it is.
- The promotion claim's body: it keeps `channel` and `component` as two
  fields, `repository` where none is declared, so what
  `status-on-any-machine` reads does not change.

## Decisions

### D1. A channel's identity is one seam type

`ChannelId { component: Option<ComponentName>, name: ChannelName }` in
`crates/tailrace-seams/src/artifact.rs`, beside `ChannelName`.
`component` is `None` where the repository declares no component.
`ChannelId::spelled()` (and `Display`) answers `name` or
`component/name`; `component_name()` answers the component or
`REPOSITORY_COMPONENT`. It serializes as
`{"component": null | "<c>", "name": "<n>"}`.

Every seam that takes a channel takes a `ChannelId` (125):
`Audience::Channel(ChannelId)`; `Keeper::append(&ChannelId, ..)` and
`Keeper::chain(&ChannelId)`; `channel_name(&ChannelId)` answers
`channel/<spelled>`; the promotion refusals `Why::ChannelAt` and
`Why::NeverHeld` name `ChannelId`s and render them spelled; a target's
`Environment.channel` is a `ChannelId`.

Alternatives: a `ComponentName` always, with `repository` meaning none
declared. Rejected: a repository may declare a component named
`repository`, whose channels would then be spelled without it and
collide with nothing to tell them apart. A spelled string as the
identity: rejected, since every reader would parse it back.

### D2. How a channel moves is one enum

```rust
pub enum Moves {
    EachPublish,
    Promotion,
    Bound { to: ChannelName },
}
pub struct DeclaredChannel { pub channel: ChannelId, pub moves: Moves }
```

`Moves` serializes as `{"by": "each_publish"}`, `{"by": "promotion"}`
or `{"by": "bound", "to": "beta"}`; `DeclaredChannel::bound_to()`
answers the channel it is bound to, within the same component. The
answer is ordered by name, then component, so the one-component
answers keep the order they had.

Alternative: keep `bound_to` and add `on_each_publish: bool`. Rejected:
two fields that can say a channel both follows each publish and is
bound, which 37 forbids; the enum cannot.

### D3. The policy for a channel names its promoters

`Required::Channel { claims, bound_to, promoted_by: Vec<NamedSigner> }`,
with `Required::promoted_by()` empty for acceptance and a layout. The
policy, not the core, says whose promotion of the channel counts (32,
116, core.7). The file profile answers:

- for `beta`, `stable` and a channel an environment's words make: the
  gate's signers, every `[signers]` entry whose `kinds` hold
  `acceptance`, whatever a line under the channel names (C.10, "never
  widens who signs");
- for a channel declared by hand: the signers its `require` names that
  may claim `promotion`, the ledger's rule as it stands.

`claims` for a given or made channel is an acceptance by the gate's
signers, then the line's `require` in order.

Alternatives: append `promotion` to the gate signer's `kinds` in a
given channel's answer. Rejected: it rewrites what a signer may claim,
and a line's own promotion signer would still widen who signs. A
`given: bool` on the channel: rejected, since the core would then work
out who the gate is from the signers, which is the policy's to say.

### D4. The file profile resolves the channels in one pass

`crates/tailrace/src/profiles/policy/file/mod.rs` keeps one `resolve`
shared by `channels` and `policy_for`, so the two never disagree (83).
Over the file at the revision, in order, each step unreadable naming
the table on the first break:

1. The components: each `[[component]]`'s `name`, or none.
2. The channel tables: `[channels.<name>]` where none is declared;
   `[channels.<component>.<name>]` where components are, the component
   one declared. A flat table holding `require`, `bound_to`,
   `on_each_publish` or `component` in a repository that declares
   components is unreadable naming the form to use; so is a nested
   table under a component not declared. In a repository that declares
   none, keys of a channel table other than `require`, `bound_to` and
   `on_each_publish` stay their readers'.
3. The given channels: `beta` (each publish) and `stable` (bound to
   `beta`) for every component.
4. The environments: `[environments.<name>]` or
   `[environments.<component>.<name>]`, reading `channel`, `policy` and
   `after` and leaving every other key to its reader. `manual` makes a
   channel named for the environment, bound to the channel it names;
   `after` makes one bound to the channel the named environments follow
   (their own where they have one, else the one they name), all named
   agreeing; where both words are written, `after`'s binding holds.
   Environments are resolved in dependency order, and a cycle through
   `after`, a name not declared or of another component, or named
   environments that follow different channels, is unreadable naming
   the table. A made channel named `beta` or `stable`, or the same as
   another made channel, is unreadable.
5. The lines: a table under a given or made channel is a line under
   it. A line naming `on_each_publish` or `bound_to` is read only where
   it restates how the channel moves; another way is unreadable naming
   the table (C.10). A hand-declared channel moves as its table says:
   `on_each_publish = true`, bound by `bound_to`, otherwise by a
   promotion alone; both keys together is unreadable (37).
6. Bindings: every `Bound { to }` names a channel of the same component
   known after steps 2 to 4; an environment's `channel` names one too.

`policy_for` of a channel answers from the resolved entry: absent where
none is resolved, otherwise `claims`, `bound_to` and `promoted_by` as
D3 says. The module's doc and TOML example change to the per-component
form, a given channel, a line under one and a manual environment.

### D5. The gate reads the channels answer

`gate.rs` loses `ChannelKeys`, `ChannelTables`, `channel_tables` and
its use of `POLICY_FILE`. `following` becomes: the channels answer at
the landed head (`channels_at`), filtered to the component published
and `Moves::EachPublish`, each as its `ChannelId`. `gate_named` walks
the policy for each listed channel instead of each file table. A
following channel's move (`Judging::promote`) checks that the gate's
identity is among the channel's `promoted_by` before it counts, and
answers refused `Why::Wanting` a promotion naming the promoters
otherwise, that channel's answer alone; the acceptance stands. The
`bound_to` check in `Judging::promote` goes, since the answer cannot
list a bound channel as following.

### D6. Promote finds its channel in the answer

`ledger::promote` reads the channels answer at the head after choosing
the component, finds the entry whose name is the one asked and whose
`component_name()` is the chosen component's, and answers absent naming
the channel and the component where none is. Its `ChannelId` then
keys the policy read, the keeper's chain and append, `mirror` and the
store name; `Moves::Bound { to }` gives the bound channel, as a
`ChannelId` of the same component, for `held_by`. `may_promote` checks
`required.promoted_by()`. `channel()` and `repair.rs` take a
`ChannelId`. The CLI's `promote` is unchanged in its arguments.

`verify` with `--policy channel:<name>` passes a `ChannelId` with no
component, and the core gives it the component of the version it
verifies before asking the policy.

### D7. The ledger profiles spell the channel

`git_ref`: `refs/tailrace/ledger/<spelled>` and its fetched copy
`refs/tailrace/fetched/<spelled>`; a slash in the spelling is a ref
path segment, which git allows. `store`: walks from
`channel_name(id)`, keeps its last-read copy under a path of the
spelled id, and takes the claims on an index whose body names both the
channel's name and `component_name()`. A component and a channel are
both names git and OCI tags accept, so no escaping is added.

### D8. The fakes script what the profile derives

The fake policy keeps scripting policies by audience. Its `channels`
lists each channel audience held, at the revision or every revision,
its component from the `ChannelId`, and moves `Bound` where the policy
names `bound_to`, `EachPublish` where the test scripted it with
`follows(&ChannelId)` (or `follows_at`), and `Promotion` otherwise.
`belongs` and `belongs_at` go: the identity carries the component.
The fake gives no channel a test did not script; givens are the file
profile's (C.10). The fake keeper keys its chains by `ChannelId`.

### D9. The suite holds every profile to the answer

`PolicyRig::declaring(components, channels: &[DeclaredChannel])`
declares each channel moving as given, and a new
`PolicyRig::given(components) -> Vec<DeclaredChannel>` answers what the
profile lists with nothing declared: none for the fake, `beta` and
`stable` per component for the file. The channels scenarios expect the
declared and the given together, which holds every profile to "listed
like any channel declared by hand" without the suite knowing C.10. The
declared names avoid `beta` and `stable`. The policy file's rig writes
the per-component form. Below the marker the group gains "each way a
channel moves" (131, 37, 83, 95). The keeper's group gains "two
components' channels of one name" (125, 120, 121). The core's gate and
ledger groups gain the two scenarios of spec `conformance-suite`, the
given channels over the fakes, citing 95, 125, 131 and 9, 32. The
register changes nothing: 41 and 97 stay pending with the target.

| scenario | group | cites |
|---|---|---|
| each way a channel moves | policy | 131, 37, 83, 95 |
| two components' channels of one name | keeper | 125, 120, 121 |
| beta of one component | gate | 95, 125, 131 |
| stable promoted by the gate's signer | ledger | 32, 9, 125 |

### D10. ADR 0019

`docs/adr/0019-a-channel-is-its-component-and-name-and-the-policy-says-how-it-moves.md`
records D1, D2, D3, D4's environment table and made-channel names, and
D5. It is numbered 0019 because two units in flight on this bolt
propose an 0018; whichever lands later renumbers.

## Risks / Trade-offs

- [Every repository now has `beta`, so every accepted publish asks the
  ledger] → a host with no `[ledger]` answers `beta` absent naming the
  ledger while the acceptance stands, as each channel is answered on its
  own; `a-host-with-no-bindings-file-is-bound-to-the-defaults` binds the
  git ref by default.
- [The keeper's and the policy's signatures change under
  `status-on-any-machine`, in construct now] → the conductor orders the
  two; whichever is second takes `ChannelId` where it reads a chain.
- [`gate.rs` is also edited by `the-publishs-host-tells-the-gate`] →
  this unit changes only how channels are found and moved; the told-by
  path is untouched.
- [The environment table's form is this unit's reading of C.10, not a
  ruling] → it is recorded in ADR 0019 for the user to accept with the
  change, and named for the design agent.
- [102 says a repository may declare no channel, and C.10 now gives
  every component two] → read as: givens are not declared, and nothing
  of a publish changes; named for the design agent.

## Migration Plan

A repository that declares no component keeps every spelling: its refs,
store names and `[channels.<name>]` tables. One that declares
components rewrites `[channels.<name>] component = "<c>"` as
`[channels.<c>.<name>]`; nothing of that form, nor any per-component
ref or store name, is on main. Rollback is the bolt without this unit.
