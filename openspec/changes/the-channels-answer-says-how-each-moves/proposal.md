# Proposal

## Why

A person who writes nothing about channels should still get `beta`,
moved by the gate on every accepted publish, and `stable`, moved by
promotion from what `beta` has held; one who wants `alpha` or `lts`
should write one line and have it; and a person who marks an
environment `manual` should get a channel of its own that waits on
their yes. Today a channel exists only where a `[channels.<name>]`
line is written, the gate finds the channels that follow each publish
by parsing the policy file itself, and two components' channels of one
name share one chain and one store name. This unit makes the policy
seam's channels answer say how each channel moves, ships the given
channels as a policy fragment in the file's own form laid beneath the
repository's file, makes the channels an environment's words call for,
and gives a channel its component wherever it travels, so the gate and
the ledger learn every channel from that one answer and the core knows
no channel's name (131, 125, 37, 83, 95, C.10, C.13, C.14, as amended
at agentplot/blueprints 67f95da, 85bbf54, 7749369 and e18ba89).

## What Changes

- **The answer says how each channel moves.** Each channel the policy
  lists says one of three things: it moves on each accepted publish,
  as the gate's own act (95); it moves by a promotion alone; or it is
  bound, naming the channel it is bound to (37, 83). The channel's
  binding leaves the listing's own field and becomes this third case
  (131).
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
  policy file's own form, holds `beta`, moving on each accepted publish,
  and `stable`, bound to `beta`, both on for every component, and
  `alpha`, moving on each accepted publish, and `lts`, bound to
  `stable`, both held off (C.10). The profile lays it beneath the
  repository's file and reads the two as one table in force, through
  one path that the channels, each channel's policy and the policy
  command all read. No Rust source names a given channel.
- **One line switches a held channel on, and the same line overrides a
  given.** Writing `[channels.alpha]` (or `[channels.<component>.alpha]`)
  switches `alpha` on, since its keys are already written beneath. A
  key the repository writes under a given replaces the fragment's: a
  `bound_to` or `on_each_publish` changes how it moves, `on = false`
  switches it off, and a `require` adds claims after the gate's
  acceptance (C.10).
- **A given channel is promoted by the gate's own signer.** Every
  channel the fragment holds and the table in force has on requires the
  gate's acceptance and is promoted by the signers the policy lets claim
  acceptance, the gate (32), whatever a line under it names: a line
  never widens who signs (C.10). So `tailrace promote` moves a given
  channel when the host holds the gate's signer.
- **`tailrace policy` shows the table in force.** At the governed
  branch's landed head it prints the acceptance and layout policies and
  every channel the channels answer lists, the givens beside the
  repository's own, each with its component, how it moves, its claims
  and its promoters, asked of the core like every command (core.31).
- **An environment's words make its channels.** An environment's
  `policy = "manual"` gives it a channel of its own, named for the
  environment and bound to the channel it names; `after = [...]` gives
  it a channel of its own bound to the channel the named environments
  follow (C.10, 41, 97). These are listed like a channel declared by
  hand and promoted by the gate's signer as a given channel is; a line
  under one adds claims and never changes how it moves, since the
  environment's words say that.
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
  the fake keeper keeps a chain per component and channel. The policy's
  conformance group gains how each channel moves and what a profile
  gives with nothing declared, and names no channel a profile gives:
  the policy file's rig reads what it gives from the shipped fragment.
  The keeper's group gains two components' chains of one channel name
  kept apart.
- **ADR 0019** records the channel's identity and its three spellings,
  the givens as a fragment beneath the file, switching on and
  overriding by one line, who signs a given's promotion, the
  environment's table, and the gate's reading through the seam; the
  self-hosting, ledgers and how-it-works pages say what is built.

Out of scope, and what each waits on:

- The rule that moves an `after` channel once every environment named
  reports it runs the version (37, 43, 91), and an environment
  converging on its channel or on a person's yes (41, 97): the first
  target, phase 2. This unit makes and lists those channels; nothing
  moves them yet but `tailrace promote`. 41 and 97 stay in the register
  with the target's unit.
- The later word that names who may say yes to a `manual` environment,
  a line that narrows who may ask for a given channel, and `lts`'s
  "by a person alone" (C.10): no key is named for any of them, and the
  asker is the host's own signer in phase 1 (129). They wait on a
  design ruling and the hosted gate's approvals.
- `alpha` taking prereleases where `beta` does not: C.10 says `alpha`
  follows every accepted version including prereleases, and nothing
  says `beta` leaves them out or how a channel that moves on each
  publish would. Here `alpha` moves on each accepted publish exactly as
  `beta` does; it waits on a design ruling.
- The other givens C.10 lists, the one component and the version rule,
  moving into the fragment: the core's reading of `[[component]]` and
  `[version]` stays as it is.
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

- `channel-movement`: how each channel moves, the given channels as a
  fragment beneath the file, switched on and overridden by one line,
  the channels an environment's words make, and who signs a promotion
  of a given or made channel, as every policy profile answers them and
  the gate and the ledger read them.

### Modified Capabilities

- `seam-contracts`: the policy's channels answer says how each channel
  moves; a channel's identity is its component and name across the
  policy, keeper and target seams; the channel's store name carries its
  component where components are declared; the policy for a channel
  names who may sign its promotion.
- `seam-fakes`: the fake policy scripts how a channel moves, its
  component carried by the channel itself; the fake keeper keeps a
  chain per component and channel.
- `conformance-suite`: the policy's group holds how each channel moves
  and what a profile gives with nothing declared, naming no channel a
  profile gives; the keeper's group holds two components' chains apart.
- `policy-file`: the file's channel form per component, the givens'
  fragment laid beneath it as the table in force, its environments'
  channels, and `on_each_publish`, `bound_to` and `on` read by the
  profile.
- `command-line`: `tailrace policy` shows the policy in force at the
  branch's landed head.
- `acceptance`: the gate learns the channels that follow each publish,
  and the signer it accepts as, through the policy seam alone.
- `promotion`: a promotion finds its channel through the channels
  answer and is signed by whom the channel's policy names, the gate's
  signer for a given or made channel.
- `git-ref-ledger`: the ref carries the component where components are
  declared.
- `store-ledger`: the walk starts from the channel's name with its
  component.

## Impact

- `crates/tailrace-seams/src/policy.rs` (how a channel moves, the
  promotion's signers), a channel identity type in `artifact.rs`,
  `keeper.rs` (append, chain and the store name by identity),
  `answer.rs` (the promotion refusals naming the identity),
  `target.rs`, `lib.rs`.
- `crates/tailrace-fakes/src/policy.rs`, `keeper.rs`, `target.rs`.
- `crates/tailrace-core/src/gate.rs` (no file parsing), `ledger.rs`,
  `repair.rs`, `verify.rs`, `landed.rs`, and their tests.
- `crates/tailrace/src/profiles/policy/file/` (the resolution, and the
  fragment `givens.toml` beside it), `profiles/ledger/`,
  `commands/promote.rs`, `commands/verify.rs`, a new
  `commands/policy.rs` with its binding in `bindings/mod.rs`, `lib.rs`
  and `main.rs`, and their tests.
- `crates/tailrace-core/src/in_force.rs`, the policy in force asked of
  the core.
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
