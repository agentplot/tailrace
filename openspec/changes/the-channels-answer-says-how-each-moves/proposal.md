# Proposal

## Why

A person who writes nothing about channels should still get `beta`,
moved by the gate on every accepted publish, and `stable`, moved by
promotion from what `beta` has held; and a person who marks an
environment `manual` should get a channel of its own that waits on
their yes. Today none of that happens: a channel exists only where a
`[channels.<name>]` line is written, the gate finds the channels that
follow each publish by parsing the policy file itself, and two
components' channels of one name share one chain and one store name.
This unit makes the policy seam's channels answer say how each channel
moves, gives every component its two channels, makes the channels an
environment's words call for, and gives a channel its component
wherever it travels, so the gate and the ledger learn every channel
from that one answer (131, 125, 37, 83, 95, C.10, C.13, C.14, as
amended at agentplot/blueprints 67f95da, 85bbf54 and 7749369).

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
- **Every component has `beta` and `stable` with no line written.**
  `beta` moves on each accepted publish; `stable` is bound to `beta`
  (C.10). They are listed like any channel declared by hand, and the
  policy for each answers like any channel's.
- **A given channel is promoted by the gate's own signer.** With no line
  written, `beta` and `stable` require the gate's acceptance and are
  promoted by the signer the policy lets claim acceptance, the gate
  (32). A line under one adds claims its versions must carry and never
  widens who signs; one that would change how a given channel moves is
  unreadable naming the table (C.10). So `tailrace promote` moves a
  given channel when the host holds the gate's signer.
- **An environment's words make its channels.** An environment's
  `policy = "manual"` gives it a channel of its own, named for the
  environment and bound to the channel it names; `after = [...]` gives
  it a channel of its own bound to the channel the named environments
  follow (C.10, 41, 97). These are listed like a channel declared by
  hand, are promoted by the gate's signer as a given channel is, and a
  reader may write a line under one as under a given channel.
- **The gate and the ledger read no key of the policy file.** The gate
  finds the channels that follow each publish, and the signer it accepts
  as, from the channels answer and the policy for each, and no longer
  parses `.config/tailrace.toml`; `tailrace promote` finds the channel,
  its binding and who may sign its promotion the same way (131).
- **The fakes answer it and the suite holds every profile to it.** The
  fake policy lists the channels scripted for it with how each moves;
  the fake keeper keeps a chain per component and channel. The policy's
  conformance group gains how each channel moves and what a profile
  gives with nothing declared; the keeper's group, two components'
  chains of one channel name kept apart.
- **ADR 0019** records the channel's identity and its three spellings,
  the given channels and who signs their promotion, the environment's
  table, and the gate's reading through the seam; the self-hosting,
  ledgers and how-it-works pages say what is built.

Out of scope, and what each waits on:

- The rule that moves an `after` channel once every environment named
  reports it runs the version (37, 43, 91), and an environment
  converging on its channel or on a person's yes (41, 97): the first
  target, phase 2. This unit makes and lists those channels; nothing
  moves them yet but `tailrace promote`. 41 and 97 stay in the register
  with the target's unit.
- The later word that names who may say yes to a `manual` environment,
  and a line that narrows who may ask for a given channel (C.10): no
  key is named for either, and the asker is the host's own signer in
  phase 1 (129). They wait on a design ruling and the hosted gate's
  approvals.
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

- `channel-movement`: how each channel moves, the channels every
  component is given, the channels an environment's words make, and
  who signs a promotion of a given or made channel, as every policy
  profile answers them and the gate and the ledger read them.

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
  and what a profile gives with nothing declared; the keeper's group
  holds two components' chains apart.
- `policy-file`: the file's channel form per component, its given
  channels, its environments' channels, and `on_each_publish` read by
  the profile.
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
- `crates/tailrace/src/profiles/policy/file/`, `profiles/ledger/`,
  `commands/promote.rs`, `commands/verify.rs`, and their tests.
- `crates/tailrace-conformance/src/seams/policy.rs`, `seams/keeper.rs`,
  `core/gate.rs`, `core/ledger.rs`, `fakes.rs`, `world.rs` and the rest
  that name a channel.
- ADR 0019 and three site pages.
- No crate, dependency, language or file type is added.
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
crates/tailrace-core/src/lib.rs
crates/tailrace-core/tests/**
crates/tailrace/src/profiles/policy/file/**
crates/tailrace/src/profiles/ledger/**
crates/tailrace/src/commands/promote.rs
crates/tailrace/src/commands/verify.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/tests/**
crates/tailrace-conformance/src/**
crates/tailrace-conformance/tests/fakes.rs
docs/adr/0019-*.md
site/src/content/docs/guides/self-hosting.mdx
site/src/content/docs/guides/integrations/ledgers.mdx
site/src/content/docs/how-it-works.mdx
```
