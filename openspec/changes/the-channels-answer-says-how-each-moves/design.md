# Design

## Context

The branch holds the bolt through `the-ledger-and-promotion` and
`status-on-any-machine`, and this unit built to the statement as
amended at agentplot/blueprints 7749369: `ChannelId` and `Moves` on the
seams, the fakes, the gate and the ledger reading the channels answer,
the ledger profiles spelling the component, the suite's scenarios and
ADR 0019. The file profile
(`crates/tailrace/src/profiles/policy/file/mod.rs`) gives every
component `beta` and `stable` from two constants in its Rust (`BETA`,
`STABLE`), refuses a line that would change how either moves, and
refuses a made channel by those two names.

The statement as amended at e18ba89 makes the givens a policy fragment
in the file's own form, shipped with the core and laid beneath the
repository's file, which the policy command shows as the one table in
force; adds `alpha` and `lts`, held and switched on by one line; lets a
repository override a given by writing the same line; and has the core
know the three ways a channel moves and no channel's name (C.10). 131
is unchanged: each publish, a promotion alone, or bound to a named
channel. 85bbf54 and bd772be give an environment's `policy` and
`after` words; the book's `environments.md` names the waiting
environment's channel for the environment ("`prod` follows `prod`,
bound to `stable`").

No `tailrace policy` command exists; C.10 names one.

## Goals / Non-Goals

**Goals:**

- One answer, the policy seam's, that every reader of channels uses.
- A channel carried as component and name on every seam that takes one.
- No spelling changes for a repository that declares no component.
- The given channels as data: no Rust source names one.

**Non-Goals:**

- Moving an `after` channel by rule, or converging an environment
  (phase 2's target).
- A key naming who may ask, or who may say yes to a manual
  environment; none is designed (C.10's "later word").
- The core's reading of `[[component]]` and `[version]`
  (`Declared::read`), which versions need (123) and which is no
  channel; it stays as it is, and neither moves into the fragment.
- Telling `alpha` from `beta` by prereleases (C.10's "including
  prereleases"): 131 gives a channel that moves on each publish no word
  for it.
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

- for a channel the givens hold that the table in force has on, and a
  channel an environment's words make: the gate's signers, every
  `[signers]` entry whose `kinds` hold `acceptance`, whatever a line
  under the channel names (C.10, "never widens who signs");
- for a channel declared by hand: the signers its `require` names that
  may claim `promotion`.

The gate and `ledger::promote` read the gate's signers and a channel's
promoters from these answers and decide nothing of them (core.7). Where
the file names no signer that may claim acceptance, C.10 (061bfa8) has
the policy answer the host's own signer as the gate, for acceptance and
for the given channels' promotion; that default is the policy file
profile's, added by `a-host-with-no-bindings-file-is-bound-to-the-defaults`
after this unit, and needs nothing of the gate or the ledger then. Here
such a file names no acceptance signer, as on main, and a given
channel's `promoted_by` is empty.

`claims` for a given or made channel is an acceptance by the gate's
signers, then the `require` of the table in force, in order.

Alternatives: append `promotion` to the gate signer's `kinds` in a
given channel's answer. Rejected: it rewrites what a signer may claim,
and a line's own promotion signer would still widen who signs. A
`given: bool` on the channel: rejected, since the core would then work
out who the gate is from the signers, which is the policy's to say.

### D4. The givens are a fragment beneath the file

`crates/tailrace/src/profiles/policy/file/givens.toml`, compiled in
with `include_str!`, is written in the file's own form for a repository
that declares no component:

```toml
[channels.beta]
on_each_publish = true

[channels.stable]
bound_to = "beta"

[channels.alpha]
on = false
on_each_publish = true

[channels.lts]
on = false
bound_to = "stable"
```

with a comment above each table citing C.10. `PolicyFile::new` lays
this fragment beneath the repository's file; `PolicyFile::with_givens`
takes another fragment's text, so a test proves that nothing but the
fragment names a given. A fragment that does not parse, or that holds
any table but `[channels.<name>]`, makes every answer unreadable naming
the givens; a test of the shipped one keeps that from ever happening.
`BETA` and `STABLE` go, and no Rust source of the profile names a
channel.

**The table in force.** One function, `in_force(file, givens)`, makes
the table every reader reads: the repository's file as written, with,
for every channel the fragment holds and every component (the one
component where none is declared), the fragment's table laid beneath
the repository's table of that channel and component:

- a key the repository writes replaces the fragment's key;
- `on_each_publish` and `bound_to` say together how the channel moves,
  so writing either replaces both (37);
- `on` is true in any table the repository writes, unless it writes
  `on = false`. So `[channels.alpha]`, empty, switches `alpha` on, the
  one line C.10 names, and `on = false` under `beta` switches `beta`
  off;
- a `require` the repository writes is the channel's claims after the
  gate's acceptance (D3).

A channel whose table in force says `on = false` is no channel: not
listed, its policy absent, and not one a binding or an environment may
name.

**The resolution.** `resolve` reads the table in force, shared by
`channels` and `policy_for` so the two never disagree (83). In order,
each step unreadable naming the table on the first break:

1. The components: each `[[component]]`'s `name`, or none.
2. The channel tables: `[channels.<name>]` where none is declared;
   `[channels.<component>.<name>]` where components are, the component
   one declared. A flat table holding `require`, `bound_to`,
   `on_each_publish`, `on` or `component` in a repository that declares
   components is unreadable naming the form to use; so is a nested
   table under a component not declared. In a repository that declares
   none, keys of a channel table other than these stay their readers'.
3. Each channel table in force, given or declared by hand, moves as it
   says: `on_each_publish = true` on each publish, `bound_to` bound,
   otherwise by a promotion alone; both keys together is unreadable
   (37). A table is given where the fragment holds its name.
4. The environments: `[environments.<name>]` or
   `[environments.<component>.<name>]`, whose keys are C.10's words and
   no others: `channel`, `policy` (`auto` or `manual`, `auto` unless
   written), `after` and `target`. The profile reads the first three and
   leaves `target` to the target's reader, reading nothing of its value;
   any other key is unreadable naming the table and the key, since a
   target's own settings sit in the target's table (C.10). `manual` makes a
   channel named for the environment, bound to the channel it names;
   `after` makes one bound to the channel the named environments follow
   (their own where they have one, else the one they name), all named
   agreeing; where both words are written, `after`'s binding holds.
   Environments are resolved in dependency order, and a cycle through
   `after`, a name not declared or of another component, or named
   environments that follow different channels, is unreadable naming
   the table. A made channel named as a channel the fragment holds, on
   or off, or as another made channel, is unreadable.
5. A table under a made channel is a line under it: read only where it
   restates how the channel moves, its `require` the claims after the
   gate's acceptance; another way is unreadable naming the table (C.10).
6. Bindings: every `Bound { to }` names a channel of the same component
   that is on after steps 3 and 4, and an environment's `channel` names
   one too; a cycle of bindings is unreadable naming a table in it,
   since no version could ever reach it (37).

`policy_for` of a channel answers from the resolved entry: absent where
none is resolved, otherwise `claims`, `bound_to` and `promoted_by` as
D3 says. The module's doc and TOML example change to the per-component
form, a given switched on, a line under one and a manual environment.

Alternatives: the givens as Rust constants. Rejected: C.10 has the core
know no channel's name, and a reader could not see them in the file's
form. Switching a held channel on with `on = true`: rejected, since a
table header and a key are two lines, and the fragment already holds
every key. A rebinding of a given refused: rejected, since C.10 lets
the same line override it.

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
profile lists with nothing declared: none for the fake; for the file,
each channel the shipped `givens.toml` has on, per component, moving as
its table says, read by the rig from that file and naming no channel
in its Rust. The channels scenarios expect the declared and the given
together, which holds every profile to "listed like any channel
declared by hand" and to the three ways without the suite knowing a
name (C.10). The declared names are none the fragment holds. The
policy file's rig writes the per-component form. Below the marker the group gains "each way a
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
records D1, D2, D3, D4 (the fragment, the table in force, switching on
and overriding by one line, the environment table and made-channel
names), D5 and D11. It is numbered 0019 because two units in flight on
this bolt propose an 0018; whichever lands later renumbers.

### D11. `tailrace policy` asks the core for the policy in force

`crates/tailrace-core/src/in_force.rs`:
`in_force(seams, history, ask: InForceAsk { branch }) -> Answer<InForce>`
resolves the branch's head, checks it landed (`Landed::check`), and
answers `InForce { branch, revision, acceptance, layout, channels }`,
each policy an `Answer<Required>` of its own and `channels` an
`Answer<Vec<ChannelInForce { channel: DeclaredChannel, policy:
Answer<Required> }>>`, so an absent layout or one unreadable channel
leaves the rest standing (58). It asks the landing and the policy
alone, and writes nothing (60, core.32).

`crates/tailrace/src/commands/policy.rs`: `tailrace policy [--branch
main] [--json]`, bound as status is (the landing and the policy; no
store, no signer; with no bindings file, as a host with no file is
bound), a new `For::Policy` in `bindings/mod.rs`. Text prints the
branch and revision, then the acceptance and layout claims, then one
row per channel, by name then component: the channel spelled, how it
moves (`each publish`, `promotion`, `bound to <channel>`), its claims
by kind and its promoters. `--json` prints the core's answer. It exits
0 whenever an answer is made, and the answer's own code where the head
cannot be resolved or has not landed.

This shows the table in force through the seam's answers, so the
command reads no file and the core no profile (core.2, core.31).
Alternative: a policy seam operation answering the merged file in its
TOML form. Rejected for now: B.5 names no such operation, and it would
carry the file profile's form across the seam; a held channel switched
off is therefore not shown. Named for the design agent.

### D12. An empty channel as last read says when

60, as amended at b64d367: where the ledger does not answer, a chain is
answered from the host's copy, an empty channel as any other answer,
with the time the copy was read. Today every keeper answers an empty
chain absent, from the source or the copy alike, and
`ledger::channel` answers `Channel::Empty` with nothing of how it was
read, so the time is lost.

- `Keeper::chain` answers absent only where the source itself holds
  none (121). Where the source does not answer and the host last read
  the channel holding none, it answers `Chain { claims: [], read:
  Read::Kept { at, said } }`, the time of that read and the source's
  words. A channel this host never read, its source not answering,
  stays unreadable with the source's words: there is no copy to answer
  from.
- The profiles keep an empty read as they keep any other: the store
  profile's `keep` keeps an empty chain too; the git ref profile keeps,
  beside its fetched refs in the repository's git directory, the time
  it last read each channel the remote held none of, which is part of
  the copy the fetch writes (60); the fake keeper keeps an empty answer
  in `answered` as it keeps one with claims.
- `ledger::channel` answers `Channel::Empty { read: Read }`:
  `Read::Source` where the keeper answered absent, the chain's `read`
  where it answered no claims. It serializes as
  `{"state": "empty", "read": ...}`, as `At` carries its `read`.
- `tailrace status` prints an empty channel read from the copy as
  `empty, as last read <time>`, the source's words on a line of their
  own, as it prints a chain at a head; the page's row (`HeadAt`) is
  unchanged, since it carries no read for a head either.

Alternative: answer an empty chain `Chain { claims: [] }` from the
source too, and retire the absence. Rejected: 121 answers "that none is
held", and every caller of `chain` reads that as absent today.

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
- [The `on` key and the empty table that switches a held channel on
  are this unit's reading of C.10, not a ruling] → recorded in ADR 0019
  for the user to accept with the change, and named for the design
  agent.
- [`target`'s value has no form yet: C.10 names the key and no target
  table exists before phase 2] → the profile accepts the key and reads
  nothing of it; the target's unit gives it its form.
- [A repository that put a target's setting, such as `bucket`, in an
  environment's table now reads unreadable] → nothing of that form is on
  main; the words name the key and say a target's settings sit in its
  own table.
- [`alpha` moves exactly as `beta` does until a ruling says how a
  channel leaves prereleases out] → the fragment's comment says so, and
  it is named for the design agent.
- [A repository may now rebind or switch off a given, and so make a
  cycle of bindings] → a cycle is unreadable naming a table in it.
- [102 says a repository may declare no channel, and C.10 now gives
  every component two] → read as: givens are not declared, and nothing
  of a publish changes; named for the design agent.

## Migration Plan

A repository that declares no component keeps every spelling: its refs,
store names and `[channels.<name>]` tables. A line it wrote under
`beta` or `stable` that restates how the channel moves reads as before;
`[channels.alpha]` or `[channels.lts]` it declared by hand becomes the
given switched on, its own keys over the fragment's. One that declares
components rewrites `[channels.<name>] component = "<c>"` as
`[channels.<c>.<name>]`; nothing of that form, nor any per-component
ref or store name, is on main. Rollback is the bolt without this unit.
