# Proposal

## Why

A person who writes nothing about channels should still get `beta`,
moved by the gate on every accepted release, and `stable`, moved by
promotion from what `beta` has held; one who wants `alpha`, which takes
prereleases too, or `lts` should write one line and have it; a person
who marks an environment `manual` should get a channel of its own that
waits on their yes; and a person should be able to say who may ask for
a channel to move. Today a channel exists only where a `[channels.<name>]`
line is written, the gate finds the channels that follow each publish
by parsing the policy file itself, and two components' channels of one
name share one chain and one store name. This unit makes the policy
seam's channels answer say how each channel moves, ships the given
channels as a policy fragment in the file's own form laid beneath the
repository's file, makes the channels an environment's words call for,
and gives a channel its component wherever it travels, so the gate and
the ledger learn every channel from that one answer and the core knows
no channel's name (131, 125, 37, 83, 95, C.10, C.13, C.14, as amended
at agentplot/blueprints 67f95da, 85bbf54, 7749369, e18ba89, 061bfa8,
b64d367, 53f6d44 and b1267d7).

## What Changes

- **The answer says how each channel moves.** Each channel the policy
  lists says one of three things: it moves on each accepted publish,
  as the gate's own act (95); it moves by a promotion alone; or it is
  bound, naming the channel it is bound to (37, 83). The channel's
  binding leaves the listing's own field and becomes this third case
  (131). A channel that moves on each publish also says whether it
  takes prereleases (C.10).
- **A channel's identity is its component and its name.** Every seam
  that takes a channel takes its component with it: the policy's
  channel audience, the ledger's append and chain, the store name a
  channel takes, and a target's environment (125, 82). A repository
  that declares no component spells every channel by its name alone,
  so nothing it has written or landed moves; one that declares
  components spells it with the component:
  `refs/tailrace/ledger/<component>/<channel>` (C.13),
  `channel/<component>/<channel>` in the store (C.14) and
  `[channels.<component>.<name>]` in the policy file (C.10). **BREAKING**
  for a file that declares components and names a channel's component
  with `component = "<name>"` inside `[channels.<name>]`, the form only
  this bolt's `the-policy-seam-lists-the-channels` wrote: it becomes
  unreadable naming the table and the form to use. Nothing of it is on
  main.
- **The given channels are a fragment of the policy file, shipped as
  data.** A file in the policy profile's directory, written in the
  policy file's own form, declares `beta`, moving on each accepted
  publish with releases alone, and `stable`, bound to `beta`, for every
  component; and holds back, each marked `held = true`, `alpha`, moving
  on each accepted publish with prereleases too, and `lts`, bound to
  `stable` and moved by no rule, so only an asking moves it (C.10). A
  repository that writes no channel line still has `beta` and `stable`,
  and the channels answer carries them, never "not declared". The profile lays it beneath the
  repository's file and reads the two as one table in force, through
  one path that the channels, each channel's policy and the policy
  command all read. No Rust source names a given channel.
- **One line switches a held channel on, and the same line overrides a
  given.** Writing `[channels.alpha]` (or `[channels.<component>.alpha]`),
  empty or with lines, switches `alpha` on, since its keys are already
  written beneath. That is the only switch: no key switches a channel on
  or off, nothing turns `beta` or `stable` off, and an `on` or `held`
  written in the repository's file is unreadable naming the key. A key
  the repository writes under a given replaces the fragment's: a
  `bound_to` or `on_each_publish` changes how it moves, `prereleases`
  whether it takes prereleases, a `require` adds claims after the
  gate's acceptance, and `approvers` narrows who may ask (C.10).
- **A following channel takes releases, and prereleases only where it
  says so.** The gate moves each channel that follows each publish with
  an accepted release, and with an accepted prerelease only where the
  channel's line says `prereleases = true`, as `alpha`'s given does
  (C.10, 95). The acceptance stands whichever channels move.
- **Who may ask is the policy's to say.** A channel moved by an asking,
  by a promotion alone, bound, or made by a manual environment, moves at
  the asking of any asker the gate verifies, the host's own signer
  today (129), unless `approvers` under its table names signers, and
  then only at theirs; a refused asking names the asker and who may
  ask. The same key under a given narrows who asks for it. A manual
  environment's approvers are written under its made channel,
  `[channels.<environment>]`. Approvers never widen who signs, and the
  gate's own move on a publish is no asking (32, 95, C.10).
- **A given channel is promoted by the gate's own signer.** Every
  channel the fragment holds and the table in force has on requires the
  gate's acceptance and is promoted by the signers the policy lets claim
  acceptance, the gate (32), whatever a line under it names: a line
  never widens who signs (C.10). So `tailrace promote` moves a given
  channel when the host holds the gate's signer. The gate and
  `tailrace promote` read a channel's promoters from the policy's answer
  alone and decide nothing of who they are (core.7).
- **`tailrace policy` shows the table in force.** At the governed
  branch's landed head it prints the acceptance and layout policies and
  every channel the channels answer lists, the givens beside the
  repository's own, each with its component, how it moves, whether it
  takes prereleases, its claims, its promoters and who may ask; and
  beneath that table each given held off, with how it would move and
  the line that would switch it on, so a switched-off `alpha` shows as
  one line away. It asks the core like every command (core.31), and the
  held givens come from the policy seam's new held answer, which the
  file profile reads from the fragment (C.10, as amended at b1267d7).
- **An environment's words make its channels.** An environment's
  `policy = "manual"` gives it a channel of its own, named for the
  environment and bound to the channel it names; `after = [...]` gives
  it a channel of its own bound to the channel the named environments
  follow (C.10, 41, 97). These are listed like a channel declared by
  hand and promoted by the gate's signer as a given channel is; a line
  under one adds claims or names its approvers and never changes how it
  moves, since the
  environment's words say that. An environment's table holds C.10's
  words and no others: `channel`, `policy` (`auto` or `manual`, `auto`
  unless written), `after` and `target`; any other key is unreadable
  naming the table and the key, since a target's own settings sit in the
  target's table and nowhere else (C.10).
- **An empty channel read as last read says when.** Where the ledger
  does not answer and the host's copy holds a channel with no promotion,
  the chain is answered empty with the time the copy was read and the
  ledger's words, like any other chain as last read, never as an
  absence that has lost its time (60, as amended at agentplot/blueprints
  b64d367).
- **The core knows three ways a channel moves and no channel's name.**
  Nothing outside the fragment names `beta`, `stable`, `alpha` or `lts`
  as a channel the code treats apart: the core, the seams and the fakes
  name none, and the file profile reads every given from the fragment
  (C.10).
- **The gate and the ledger read no key of the policy file.** The gate
  finds the channels that follow each publish, and the signer it accepts
  as, from the channels answer and the policy for each, and no longer
  parses `.config/tailrace.toml`; `tailrace promote` finds the channel,
  its binding and who may sign its promotion the same way (131).
- **The fakes answer it and the suite holds every profile to it.** The
  fake policy lists the channels scripted for it with how each moves;
  the fake keeper keeps a chain per component and channel; the fake
  policy holds off only what a test scripts. The policy's
  conformance group gains how each channel moves and what a profile
  gives with nothing declared, and what it holds off kept apart from
  its channels, and names no channel a profile gives: the policy file's
  rig reads what it gives and holds from the shipped fragment.
  The keeper's group gains two components' chains of one channel name
  kept apart.
- **ADR 0019** records the channel's identity and its three spellings,
  the givens as a fragment beneath the file, switching on by writing a
  given's table and overriding it by the same line, with no `on` key,
  as ruled at 53f6d44, who signs a given's promotion, who may ask, the
  environment's table, the gate's reading through the seam, and the
  policy command with the givens held off; it puts to the user only
  what is still this unit's reading, the fragment's `held` mark and a
  manual environment's approvers under its made channel. The
  self-hosting, ledgers and how-it-works pages say what is built.

Out of scope, and what each waits on:

- The rule that moves an `after` channel once every environment named
  reports it runs the version (37, 43, 91), and an environment
  converging on its channel or on a person's yes (41, 97): the first
  target, phase 2. This unit makes and lists those channels; nothing
  moves them yet but `tailrace promote`. 41 and 97 stay in the register
  with the target's unit.
- Persons as approvers, and `lts`'s "by a person alone" as a person
  rather than the host's signer: they wait on the owner's identity
  provider (129, phase 2). Until then `approvers` names signers, and an
  asker is the host's own signer.
- The other givens C.10 lists, the one component and the version rule,
  moving into the fragment: the core's reading of `[[component]]` and
  `[version]` stays as it is.
- A file that names no signer that may claim acceptance: C.10 has the
  policy answer the host's own signer as the gate, for acceptance and
  for the given channels' promotion, so `beta` moves on a laptop with
  nothing written. That default lives in the policy file profile and is
  `a-host-with-no-bindings-file-is-bound-to-the-defaults`'s, after this
  unit lands; until then such a file names no gate, as on main, and its
  given channels name no promoter. Neither the gate nor `tailrace
  promote` changes for it, since both read the gate and the promoters
  from the answer. The refusal C.10 gives a file naming several signers
  for provenance and none for acceptance, "several publishers and no
  gate", goes with that default.
- `tailrace status` rendering how each channel moves:
  `status-on-any-machine`, which reads the channels answer and the
  chains as this unit leaves them.
- A host told by its publish's answer that the gate should judge:
  `the-publishs-host-tells-the-gate`; this unit changes only how the
  gate finds its channels once told.
- Channels by branch pattern (127) and several promotions in one act
  (128): no unit of phase 1 carves them.

## Capabilities

### New Capabilities

- `channel-movement`: how each channel moves, prereleases among it, the
  given channels as a fragment beneath the file, switched on and
  overridden by one line, the channels an environment's words make, who
  may ask for a channel and who signs a promotion of a given or made
  channel, and the givens held off with the line that switches each on,
  as every policy profile answers them and the gate and the ledger read
  them.

### Modified Capabilities

- `seam-contracts`: the policy's channels answer says how each channel
  moves, and whether a following one takes prereleases; the policy for
  a channel names its approvers; a channel's identity is its component and name across the
  policy, keeper and target seams; the channel's store name carries its
  component where components are declared; the policy for a channel
  names who may sign its promotion; an empty chain read as last read
  says when; the policy answers the givens held off, each with the line
  that would switch it on.
- `seam-fakes`: the fake policy scripts how a channel moves,
  prereleases among it, its
  component carried by the channel itself, and the givens it holds off
  as a test scripts them; the fake keeper keeps a
  chain per component and channel, and an empty chain as last read.
- `conformance-suite`: the policy's group holds how each channel moves,
  prereleases among it, and what a profile gives with nothing declared,
  naming no channel a profile gives, and the givens a profile holds off
  kept apart from its channels; the gate's group a prerelease
  moving only the channels that take prereleases, and the ledger's an
  asker the approvers do not name; the keeper's group holds two components' chains apart,
  and an empty chain as last read saying when.
- `policy-file`: the file's channel form per component, the givens'
  fragment laid beneath it as the table in force with its `held` mark,
  its environments' channels and the four keys an environment's table
  holds, and `on_each_publish`, `bound_to`, `prereleases` and
  `approvers` read by the profile.
- `command-line`: `tailrace policy` shows the policy in force at the
  branch's landed head and, beneath it, the givens held off with the
  line that switches each on.
- `acceptance`: the gate learns the channels that follow each publish,
  and the signer it accepts as, through the policy seam alone; a
  prerelease moves only those that take prereleases; the gate's move is
  no asking.
- `promotion`: a promotion finds its channel through the channels
  answer, is asked by an approver where the channel names any, and is
  signed by whom the channel's policy names, the gate's
  signer for a given or made channel; an empty channel read as last
  read says when.
- `git-ref-ledger`: the ref carries the component where components are
  declared; an empty channel read from the copy says when.
- `store-ledger`: the walk starts from the channel's name with its
  component; an empty channel read from the kept copy says when.

## Impact

- `crates/tailrace-seams/src/policy.rs` (how a channel moves,
  prereleases among it, the promotion's signers and approvers, and the
  held givens' answer), a channel identity type in `artifact.rs`,
  `keeper.rs` (append, chain and the store name by identity, an empty
  chain as last read),
  `answer.rs` (the promotion refusals naming the identity, and the
  asker the approvers do not name),
  `target.rs`, `lib.rs` (`Nowhere` answering the held givens).
- `crates/tailrace-fakes/src/policy.rs` (with the held givens a test
  scripts), `keeper.rs`, `target.rs`.
- `crates/tailrace-core/src/gate.rs` (no file parsing, a prerelease
  moving only the channels that take prereleases), `ledger.rs`
  (an empty channel carrying how it was read), `repair.rs`, `verify.rs`,
  `landed.rs`, `status.rs`, and their tests.
- `crates/tailrace/src/profiles/policy/file/` (the resolution, and the
  fragment `givens.toml` beside it), `profiles/ledger/`,
  `commands/promote.rs`, `commands/verify.rs`, `commands/status.rs`
  (an empty channel as last read), a new
  `commands/policy.rs` with its binding in `bindings/mod.rs`, `lib.rs`
  and `main.rs`, and their tests.
- `crates/tailrace-core/src/in_force.rs`, the policy in force asked of
  the core, with the givens held off, and `landed.rs`'s `held_at`.
- `crates/tailrace-conformance/src/seams/policy.rs`, `seams/keeper.rs`,
  `core/gate.rs`, `core/ledger.rs`, `fakes.rs`, `world.rs` and the rest
  that name a channel.
- ADR 0019 and three site pages.
- No crate, dependency, language or file type is added: the fragment is
  TOML, which taplo already formats.
- Overlaps in flight on this bolt: `status-on-any-machine` reads the
  keeper's chain and the channels answer whose signatures change here,
  and `the-publishs-host-tells-the-gate` edits `gate.rs`.

## Touches

```
crates/tailrace-seams/src/policy.rs
crates/tailrace-seams/src/artifact.rs
crates/tailrace-seams/src/keeper.rs
crates/tailrace-seams/src/answer.rs
crates/tailrace-seams/src/target.rs
crates/tailrace-seams/src/lib.rs
crates/tailrace-fakes/src/policy.rs
crates/tailrace-fakes/src/keeper.rs
crates/tailrace-fakes/src/target.rs
crates/tailrace-fakes/src/lib.rs
crates/tailrace-core/src/gate.rs
crates/tailrace-core/src/ledger.rs
crates/tailrace-core/src/repair.rs
crates/tailrace-core/src/verify.rs
crates/tailrace-core/src/landed.rs
crates/tailrace-core/src/status.rs
crates/tailrace-core/src/in_force.rs
crates/tailrace-core/src/lib.rs
crates/tailrace-core/tests/**
crates/tailrace/src/profiles/policy/file/**
crates/tailrace/src/profiles/ledger/**
crates/tailrace/src/commands/promote.rs
crates/tailrace/src/commands/verify.rs
crates/tailrace/src/commands/policy.rs
crates/tailrace/src/commands/status.rs
crates/tailrace/src/commands/mod.rs
crates/tailrace/src/bindings/mod.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/src/lib.rs
crates/tailrace/src/main.rs
crates/tailrace/tests/**
crates/tailrace-conformance/src/**
crates/tailrace-conformance/tests/fakes.rs
docs/adr/0019-*.md
site/src/content/docs/guides/self-hosting.mdx
site/src/content/docs/guides/integrations/ledgers.mdx
site/src/content/docs/how-it-works.mdx
```
