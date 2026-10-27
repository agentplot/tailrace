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

The batch 18 rulings at agentplot/blueprints 53f6d44 settle what C.10
left open: a held given is switched on by writing its table, empty or
with lines, with no key for it and nothing that turns `beta` or `stable`
off; a channel that follows each publish takes the releases the version
rule makes, and prereleases too only where its line says `prereleases =
true`, which `alpha`'s given does; the givens are declared, so the
channels answer always carries `beta` and `stable`; and who may ask for
a channel moved by an asking is any asker the gate verifies (129),
unless `approvers` under it names signers, `lts`'s given naming no rule
so that only an asking moves it.

The batch 19 rulings at agentplot/blueprints b1267d7 amend C.10 so the
policy command shows the table in force and, beneath it, each given
held off with the line that would switch it on, read from the fragment,
so a reader sees that a switched-off `alpha` is one line away. On the
branch `tailrace policy` shows the channels in force and nothing held
off, since no answer of the policy seam carries a held given.

The batch 20 rulings at agentplot/blueprints 088430c have the fragment
mark the givens it holds off with a mark of its own, which a
repository's file never carries and the profile refuses, and have a
repository write `approvers` on the environment or on the channel it
means, never on a channel the core made for an environment, which takes
its environment's. 131 as amended at 1ae1cc5 has the channels answer
carry, apart from the channels in force, each given the repository has
not switched on, marked held and naming the line that switches it on,
so a surface shows both from one answer; and 136 (d50535a) gives the
policy seam three operations, policy-for, channels and environments,
and no other.

## Goals / Non-Goals

**Goals:**

- One answer, the policy seam's, that every reader of channels uses.
- A channel carried as component and name on every seam that takes one.
- No spelling changes for a repository that declares no component.
- The given channels as data: no Rust source names one.

**Non-Goals:**

- Moving an `after` channel by rule, or converging an environment
  (phase 2's target).
- Persons as approvers: `approvers` names signers until an identity
  provider stands (129, phase 2), and the asker is the host's own
  signer.
- The core's reading of `[[component]]` and `[version]`
  (`Declared::read`), which versions need (123) and which is no
  channel; it stays as it is, and neither moves into the fragment.
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
    EachPublish { prereleases: bool },
    Promotion,
    Bound { to: ChannelName },
}
pub struct DeclaredChannel { pub channel: ChannelId, pub moves: Moves }
```

`Moves` serializes as `{"by": "each_publish", "prereleases": false}`,
`{"by": "promotion"}` or `{"by": "bound", "to": "beta"}`;
`prereleases` is whether a following channel takes prereleases as well
as the releases the version rule makes (C.10), so it lives on the one
case it can be true of; `DeclaredChannel::bound_to()`
answers the channel it is bound to, within the same component. The
answer is ordered by name, then component, so the one-component
answers keep the order they had.

Alternative: keep `bound_to` and add `on_each_publish: bool`. Rejected:
two fields that can say a channel both follows each publish and is
bound, which 37 forbids; the enum cannot.

### D3. The policy for a channel names its promoters and its approvers

`Required::Channel { claims, bound_to, promoted_by: Vec<NamedSigner>,
approvers: Vec<NamedSigner> }`, with `Required::promoted_by()` and
`Required::approvers()` empty for acceptance and a layout. The
policy, not the core, says whose promotion of the channel counts (32,
116, core.7). The file profile answers:

- for a channel the givens hold that the table in force lists, and a
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

`approvers` are who may ask for the channel to move (32, C.10), by
their names in `[signers]`, whatever each may claim: for a given or
hand-declared channel, the signers its table in force names under
`approvers`; for a channel a manual environment makes, the signers the
environment's table names, since a repository writes `approvers` on the
environment or on the channel it means and never on a channel the core
made for an environment, which takes its environment's (C.10, as ruled
at 088430c). Empty means any asker the gate verifies, the host's own
signer while no identity provider stands (129). A list the file writes
empty is unreadable naming the table, since it would read as naming no
one and answer anyone. The approvers never join the promoters: they
narrow who asks, never who signs (C.10).

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
held = true
on_each_publish = true
prereleases = true

[channels.lts]
held = true
bound_to = "stable"
```

with a comment above each table citing C.10, the one on `lts` saying
it names no rule, so only an asking moves it. `PolicyFile::new` lays
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
  so writing either replaces both and the fragment's `prereleases` with
  them, unless the repository writes its own `prereleases` (37);
- `held = true` is the fragment's mark alone: a held table is in force
  only where the repository writes the channel's table, so
  `[channels.alpha]`, empty, switches `alpha` on, the one line C.10
  names, and the merged table carries no `held`;
- a `require` the repository writes is the channel's claims after the
  gate's acceptance, and its `approvers` who may ask (D3).

A held channel the repository does not write is no channel: not
listed, its policy absent, and not one a binding or an environment may
name. `held` or `on` written in the repository's file is unreadable
naming the table and the key, saying that writing a channel's table is
the switch and nothing turns `beta` or `stable` off (C.10); the
fragment's comment says the same.

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
   none, keys of a channel table other than these, `prereleases`,
   `approvers`, `held` and `on` stay their readers'.
3. Each channel table in force, given or declared by hand, moves as it
   says: `on_each_publish = true` on each publish, taking prereleases
   where `prereleases = true`, `bound_to` bound, otherwise by a
   promotion alone; both keys together is unreadable (37), and so is
   `prereleases` on a table that does not move on each publish. A table
   is given where the fragment holds its name. Its `approvers` each name
   a `[signers]` entry, unreadable naming the table and the name
   otherwise.
4. The environments: `[environments.<name>]` or
   `[environments.<component>.<name>]`, whose keys are C.10's words and
   no others: `channel`, `policy` (`auto` or `manual`, `auto` unless
   written), `after`, `target`, and `approvers` where the environment
   is manual. The profile reads all but `target`, which it leaves to the
   target's reader, reading nothing of its value. `approvers` is who may
   say yes, each a `[signers]` name and the list never empty, as on a
   channel's table; on an environment that is not manual it is
   unreadable naming the table and the key, since no one says yes to
   it. Any other key is unreadable naming the table and the key, since
   a target's own settings sit in the target's table (C.10). `manual` makes a
   channel named for the environment, bound to the channel it names,
   whose approvers are the environment's;
   `after` makes one bound to the channel the named environments follow
   (their own where they have one, else the one they name), all named
   agreeing; where both words are written, `after`'s binding holds.
   Environments are resolved in dependency order, and a cycle through
   `after`, a name not declared or of another component, or named
   environments that follow different channels, is unreadable naming
   the table. A made channel named as a channel the fragment holds,
   held or not, or as another made channel, is unreadable.
5. A table under a made channel is a line under it: read only where it
   restates how the channel moves, its `require` the claims after the
   gate's acceptance; another way is unreadable naming the table, and
   so is `approvers` under it, saying that who may ask for a channel an
   environment makes is written on the environment (C.10).
6. Bindings: every `Bound { to }` names a channel of the same component
   in force after steps 3 and 4, and an environment's `channel` names
   one too; a cycle of bindings is unreadable naming a table in it,
   since no version could ever reach it (37).

`policy_for` of a channel answers from the resolved entry: absent where
none is resolved, otherwise `claims`, `bound_to`, `promoted_by` and `approvers` as
D3 says. The module's doc and TOML example change to the per-component
form, a given switched on, a line under one, approvers, and a manual
environment naming its approvers.

Alternatives: the givens as Rust constants. Rejected: C.10 has the core
know no channel's name, and a reader could not see them in the file's
form. Two fragments, one of the givens declared and one of the held:
rejected, since C.10 names one fragment, and one file shows a reader
every given together. Marking the held channels by their absence from a
list in the profile's Rust: rejected, since that Rust would name them. A rebinding of a given refused: rejected, since C.10 lets
the same line override it.

### D5. The gate reads the channels answer

`gate.rs` loses `ChannelKeys`, `ChannelTables`, `channel_tables` and
its use of `POLICY_FILE`. `following` becomes: the channels answer at
the landed head (`channels_at`), filtered to the component published
and `Moves::EachPublish`, each as its `ChannelId`, leaving out one whose
`prereleases` is false where the version accepted is a prerelease. A
version is a prerelease where it parses as a semantic version with a
prerelease part, the test the version rule's own `derive` makes of a
tag (`crates/tailrace-core/src/version.rs`, 21); `following` takes the
version for it. `gate_named` walks
the policy for each listed channel instead of each file table. A
following channel's move (`Judging::promote`) checks that the gate's
identity is among the channel's `promoted_by` before it counts, and
answers refused `Why::Wanting` a promotion naming the promoters
otherwise, that channel's answer alone; the acceptance stands. The
`bound_to` check in `Judging::promote` goes, since the answer cannot
list a bound channel as following. The gate's move is its own act, recording
its own signer as who asked and the publish as its cause (129), so a
channel's `approvers` do not bind it.

### D6. Promote finds its channel in the answer

`ledger::promote` reads the channels answer at the head after choosing
the component, finds the entry whose name is the one asked and whose
`component_name()` is the chosen component's, and answers absent naming
the channel and the component where none is. Its `ChannelId` then
keys the policy read, the keeper's chain and append, `mirror` and the
store name; `Moves::Bound { to }` gives the bound channel, as a
`ChannelId` of the same component, for `held_by`. `may_promote` checks
`required.promoted_by()`, and a new `may_ask` checks, before it, that
the asker, `ask.signer` while no identity provider stands (129), is
among `required.approvers()` where any are named, refused
`Why::NotApprover { asker, approvers }` naming both otherwise (32,
C.10). The refusal is a new `Why` in `answer.rs`, rendered as `<asker>
may not ask for <channel>; ask one of <approvers>`. `channel()` and `repair.rs` take a
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
`follows(&ChannelId, prereleases)` (or `follows_at`), and `Promotion`
otherwise.
`belongs` and `belongs_at` go: the identity carries the component.
The fake gives no channel a test did not script; givens are the file
profile's (C.10). It holds off only what a test scripts with `holds`
(D13). The fake keeper keys its chains by `ChannelId`.

### D9. The suite holds every profile to the answer

`PolicyRig::declaring(components, channels: &[DeclaredChannel])`
declares each channel moving as given, and a new
`PolicyRig::given(components) -> Vec<DeclaredChannel>` answers what the
profile lists with nothing declared: none for the fake; for the file,
each channel the shipped `givens.toml` declares, not held, per component, moving as
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
| a prerelease and the channels that follow | gate | 95, 131 |
| an asker the approvers do not name | ledger | 32, 33 |
| the givens held off | policy | 131, C.10 |

The "each way a channel moves" scenario declares a second following
channel taking prereleases. The approvers scenario cites 32, not 129,
which stays pending with phase 2: an asker named by an identity
provider is not what it exercises.

### D10. ADR 0019

`docs/adr/0019-a-channel-is-its-component-and-name-and-the-policy-says-how-it-moves.md`
records D1, D2 (with `prereleases`), D3 (with `approvers`), D4 (the
fragment and its `held` mark, the table in force, switching on by
writing the table and overriding by one line, the environment table and
made-channel names), D5, D11 and D13. How a held given is switched on
is the statement's, not this unit's reading (53f6d44): writing its
table is the switch, there is no `on` key, and nothing turns `beta` or
`stable` off. So the record's outcome and consequences name no `on` key, its considered
options keep an `on` key only as an option weighed and rejected, its
Deciders line names no reading of this unit's, the fragment's own hold
mark and where approvers are written being ruled (088430c), and its
policy command shows the givens held off with their lines, carried by
the channels answer (b1267d7, 1ae1cc5). It is numbered 0019 because two units in
flight on this bolt propose an 0018; whichever lands later renumbers.

### D11. `tailrace policy` asks the core for the policy in force

`crates/tailrace-core/src/in_force.rs`:
`in_force(seams, history, ask: InForceAsk { branch }) -> Answer<InForce>`
resolves the branch's head, checks it landed (`Landed::check`), and
answers `InForce { branch, revision, acceptance, layout, channels }`,
each policy an `Answer<Required>` of its own and `channels` an
`Answer<ChannelsInForce { in_force: Vec<ChannelInForce { channel:
DeclaredChannel, policy: Answer<Required> }>, held: Vec<HeldChannel>
}>`, the one channels answer (D13) with each channel in force given its
policy, so an absent layout or one unreadable channel leaves the rest
standing (58). It asks the landing and the policy alone, and writes
nothing (60, core.32).

`crates/tailrace/src/commands/policy.rs`: `tailrace policy [--branch
main] [--json]`, bound as status is (the landing and the policy; no
store, no signer; with no bindings file, as a host with no file is
bound), a new `For::Policy` in `bindings/mod.rs`. Text prints the
branch and revision, then the acceptance and layout claims, then one
row per channel, by name then component: the channel spelled, how it
moves (`each publish`, `each publish, prereleases too`, `promotion`,
`bound to <channel>`), its claims by kind, its promoters, and who may
ask (`any asker` where it names no approvers). Beneath the channels,
under `held off`, it prints one row per held given (D13): the channel
spelled, how it would move, in the same words, and `switch on with
<line>`; the heading and its rows are printed only where the channels
answer holds any off. An unreadable channels answer prints its words
once, in place of both parts, the acceptance and the layout standing.
`--json` prints the core's answer. It exits
0 whenever an answer is made, and the answer's own code where the head
cannot be resolved or has not landed.

This shows the table in force and what is held off through the seam's
answers, so the command reads no file and the core no profile (core.2,
core.31). Alternative: a policy seam operation answering the merged
file in its TOML form. Rejected: B.5 names no such operation, and it
would carry the whole of the file profile's form across the seam, where
the channels answer carries one line of it for each held given, opaque
to the core.

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

### D13. The channels answer carries the givens held off

C.10, as amended at b1267d7, has the policy command show each given
held off with the line that would switch it on, read from the fragment;
131, as amended at 1ae1cc5, has the channels answer carry them, apart
from the channels in force, so a surface shows both from one answer,
and 136 leaves the policy seam no operation for them. The channels
answer becomes:

```rust
pub struct HeldChannel { pub channel: DeclaredChannel, pub switch: String }

pub struct PolicyChannels {
    /// 131: every channel in force, once each, ordered by name and then
    /// component, each with how it moves.
    pub in_force: Vec<DeclaredChannel>,
    /// C.10: every given the repository has not switched on, once each,
    /// ordered by name and then component, each with how it would move
    /// and the line that switches it on; never a channel in force.
    pub held: Vec<HeldChannel>,
}

fn channels(&self, revision: &Revision) -> Answer<PolicyChannels>;
```

Both are defined in `crates/tailrace-seams/src/policy.rs`. The answer
serializes as `{"in_force": [...], "held": [...]}`, a held channel as
the declared channel flattened with `"switch": "[channels.alpha]"`.
Being in `held`, as a `HeldChannel`, is the mark: no reader can take a
held given for a channel in force. `switch` is the profile's own words,
which the core and the command carry and never parse. One answer, so
it is read, unreadable or absent as a whole: the held givens can never
disagree with the channels in force.

- **The file profile** answers `held` from the same `in_force` and
  resolution as the channels in force: for every channel the fragment
  marks `held` and every component (the one component where none is
  declared) whose table the repository does not write, the channel with
  the fragment's table read as step 3 of the resolution reads one (how
  it would move, its `prereleases`), and `switch` the table's header in
  the file's form, `[channels.<name>]` or
  `[channels.<component>.<name>]`. A held given never appears in
  `in_force`, and a switched-on one never in `held`.
- **The fake policy** carries in `held` what a test scripts with
  `holds(&ChannelId, moves, switch)`, an empty list where none is
  scripted; the call is recorded and faulted as `channels` is today.
  `Nowhere` answers `channels` as it does now, and every wrapping
  `Policy` in the workspace (`Watched` in the suite and the core's
  tests, `OneUnreadable`) delegates it as now, taking the new type.
- **The suite**: `PolicyRig::held(components) -> Vec<HeldChannel>`
  answers what the profile holds off with nothing declared, none for
  the fake; for the file, read by the rig from the shipped `givens.toml`
  per component, naming no channel in its Rust. "the givens held off"
  below the policy group's marker, citing 131 and C.10, holds the
  answer's `held` to the rig's word and apart from its `in_force`; the
  unreachable policy's scenario needs nothing more, the held givens
  being part of the one answer it already asks.
- **The core**: `landed.rs` gains `policy_channels_at`, answering the
  whole answer at the landed head, and `channels_at` answers its
  `in_force` alone, so the gate, the ledger and status read the
  channels in force as before and never see a held given. `in_force`
  alone reads `policy_channels_at` (D11).

Alternatives: a third operation answering the held givens. Rejected:
136 gives the policy seam policy-for, channels and environments and no
other, and two answers could disagree at one revision. A held given
listed among the channels with a mark on each: rejected, since every
reader of the channels in force would have to leave the held ones out.
The command reading the fragment itself: rejected, since a surface
would then read a profile's data (core.31) and a profile with other
givens would be shown the file's.

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
- [`approvers` on an environment that is not manual is refused: the
  ruling at 088430c admits the key where the environment names who may
  say yes, and no one says yes to an `auto` environment] → named for
  the design agent.
- [While no identity provider stands the asker is the host's own
  signer, which must also be a promoter, so `approvers` naming no gate
  signer leaves the channel unmovable] → `tailrace policy` shows both
  lists side by side; nothing refuses the file for it, since a hosted
  gate's askers will differ from its signer.
- [`target`'s value has no form yet: C.10 names the key and no target
  table exists before phase 2] → the profile accepts the key and reads
  nothing of it; the target's unit gives it its form.
- [A repository that put a target's setting, such as `bucket`, in an
  environment's table now reads unreadable] → nothing of that form is on
  main; the words name the key and say a target's settings sit in its
  own table.
- [A repository may now rebind a given, and so make a cycle of
  bindings] → a cycle is unreadable naming a table in it.
- [102 says a repository may declare no channel, which a reviewer may
  read as the givens answered not declared] → the givens are declared,
  by the fragment (ruled at b64d367): a repository that declares no
  channel in 102's sense writes no channel line, its versions offered
  and accepted as 102 says, and the channels answer and the policy for
  each given carry `beta` and `stable` as declared, never "not
  declared"; spec `channel-movement` says so with a scenario.
- [Every reader of the channels answer takes a new type] → `channels_at`
  keeps answering the channels in force, so the gate, the ledger and
  status change only where they call the seam directly.
- [Several provenance signers and no accepting one, which C.10 now
  refuses in words] → that refusal belongs with the host-as-gate
  default, `a-host-with-no-bindings-file-is-bound-to-the-defaults`'s;
  here such a file names no gate, as on main.

## Migration Plan

A repository that declares no component keeps every spelling: its refs,
store names and `[channels.<name>]` tables. A line it wrote under
`beta` or `stable` that restates how the channel moves reads as before;
`[channels.alpha]` or `[channels.lts]` it declared by hand becomes the
given switched on, its own keys over the fragment's, so a hand-declared
`alpha` that moved on each publish now takes prereleases too unless it
writes `prereleases = false`. A file writing `on` under a channel reads
unreadable naming the key; that key was only this bolt's. One that declares
components rewrites `[channels.<name>] component = "<c>"` as
`[channels.<c>.<name>]`; nothing of that form, nor any per-component
ref or store name, is on main. Rollback is the bolt without this unit.
