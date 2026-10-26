---
number: 19
title: A channel is its component and name, and the policy says how it moves
status: proposed
date: 2026-10-25
decision-makers:
- Chuck Swanberg
---

# A channel is its component and name, and the policy says how it moves

Deciders: proposed by the-channels-answer-says-how-each-moves unit of phase 1's bolt 4, which ships the given channels as a policy fragment laid beneath the repository's file, makes the channels an environment's words call for, has the gate and the ledger learn every channel from the policy seam's one answer, and adds `tailrace policy` to show the table in force, for the user to accept with the change. The `on` key, the empty table that switches a held channel on, and the environment table's form are this unit's reading of C.10, named for madswan-design.
Sources: statement clauses 32, 37, 95, 125, 131, C.10, C.13, C.14, as amended at `agentplot/blueprints` 7749369 and e18ba89, with 67f95da and 85bbf54; ADRs 0009, 0010, 0016; rules core.1, core.2, core.7, core.31, core.32, core.37; the change's design, D1 to D5 and D11.

## Context and problem statement

The policy seam listed the channels written as `[channels.<name>]` tables, each naming its component with a `component` key, and the channel it is bound to. Nothing else read that answer: the gate parsed the policy file itself to find the channels that follow each publish and the signer it accepts as, and the ledger's promotion read the channel's policy for its binding and let a signer promote where the channel's `require` named it for a promotion. Every seam that takes a channel took its name alone, so two components' `beta` shared one chain, one ref and one store name, which 125 forbids (ADR 0010, core.37).

The statement as amended at 7749369 makes a channel's identity its component and its name, fixes how it is spelled in the ledger's ref (C.13), the store (C.14) and the policy file (C.10), gives every component `beta` and `stable`, and has a given channel promoted by the gate's own signer (32); 85bbf54 gives an environment `policy` and `after`, which call for channels of their own. e18ba89 makes the givens a policy fragment in the file's own form, shipped with the core and laid beneath the repository's file, which the policy command shows as the one table in force; adds `alpha` and `lts`, held until one line switches each on; lets a repository override a given by writing the same line; and has the core know the three ways a channel moves and no channel's name (C.10).

Eight things were left open: how a channel's identity crosses the seams; how the answer says a channel moves; who the policy names as a channel's promoters; where the givens live; how a held given is switched on and a given overridden; how the policy file resolves its channels; how the gate learns them; and how the policy command shows the table in force.

## Considered options

For a channel's identity:

1. One seam type, a component that is none where the repository declares no component, and a name; spelled `<name>` or `<component>/<name>`.
2. A component always, `repository` meaning none is declared.
3. The spelled string itself.

For how a channel moves:

1. One enum: on each accepted publish, by a promotion alone, or bound to a channel of its component.
2. The listing's `bound_to` kept, beside a flag that the channel follows each publish.

For who may sign a channel's promotion:

1. The policy for the channel names its promoters.
2. The given channel's answer adds `promotion` to the kinds of the gate's signer.
3. The channel says it is given, and the core works out who the gate is from the signers.

For where the givens live:

1. A fragment in the policy file's own form, `givens.toml` in the policy profile's directory, compiled in and laid beneath the repository's file.
2. Constants in the profile's Rust.

For switching a held given on and overriding a given:

1. The given's table written by the repository, empty or not, is on unless it writes `on = false`; each key it writes replaces the fragment's, `on_each_publish` and `bound_to` together.
2. `on = true` written under the given's table to switch it on.
3. A line under a given that would change how it moves refused, as before.

For the policy file's channels:

1. One resolution of the table in force, shared by the channels listed and one channel's policy.
2. Each operation reading the tables it needs.

For the gate's channels:

1. The policy seam's channels answer, and the policy for each channel listed.
2. The policy file's tables, as the gate read them before.

For the policy command:

1. A question to the core answering, at the branch's landed head, the policy for acceptance and a layout and every channel listed with its policy, each its own answer.
2. A policy seam operation answering the merged file in its TOML form.

## Decision outcome

A channel's identity is option 1 (125). `ChannelId` is defined with the seams beside `ChannelName`, and every seam that takes a channel takes it: the policy's channel audience, the keeper's append and chain, the store name `channel/<spelled>`, the refusals a promotion gives, and a target's environment. It is spelled by its name where the repository declares no component, so nothing a repository of one component has written or landed moves, and by its component, a slash and its name where it declares components: `refs/tailrace/ledger/<component>/<channel>` (C.13) and `channel/<component>/<channel>` in the store (C.14), which fills the component ADR 0016 left to the policy seam's list. Channels are ordered by name, then component. Option 2 spells a declared component named `repository` as none and collides with nothing to tell the two apart; option 3 has every reader parse the identity back. The promotion claim's body keeps `channel` and `component` as two fields, as ADR 0016 made it.

How a channel moves is option 1 (131, 37, 83, 95). `Moves` is `{"by": "each_publish"}`, `{"by": "promotion"}` or `{"by": "bound", "to": "<channel>"}`, and the channel's binding leaves the listing's own field for its third case. Option 2 has two fields that can say a channel both follows each publish and is bound, which 37 forbids and the enum cannot.

Who may sign a channel's promotion is option 1 (32, core.7). `Required::Channel` gains `promoted_by`, the signers whose promotion of the channel counts, each with what it may claim; acceptance and a layout name none. The policy file names the gate's signers, every `[signers]` entry that may claim acceptance, as the promoters of every channel the fragment holds that the table in force has on and every channel an environment's words make, whatever a line under the channel names, so a line never widens who signs (C.10); such a channel requires the gate's acceptance first, then the line's claims. A channel declared by hand is promoted by the signers its `require` names that may claim a promotion, as the ledger's rule stood. Option 2 rewrites what a signer may claim, and a line's own promotion signer would still widen who signs; option 3 puts the policy's judgment of who the gate is in the core.

The givens are option 1 (C.10). `givens.toml` holds `[channels.beta]` moving on each accepted publish and `[channels.stable]` bound to `beta`, both on, and `[channels.alpha]` moving on each accepted publish and `[channels.lts]` bound to `stable`, both `on = false`, each under a comment citing C.10. `alpha` moves exactly as `beta` does: C.10 has it follow every accepted version including prereleases, and nothing yet says how a channel that moves on each publish leaves prereleases out. The profile lays the fragment beneath the repository's file, for every component the file declares, as `[channels.<name>]` where none is declared and `[channels.<component>.<name>]` where some are; one function makes that table in force and the resolution alone reads it. A fragment that does not parse, or holds any table but a channel's, makes every answer unreadable naming the givens, and a test of the shipped one keeps that from happening; `PolicyFile::with_givens` takes another fragment, so a test proves nothing but the fragment names a given. No Rust source of the profile names a channel, and the core, the seams and the fakes name none. Option 2 has the core know channels' names, which C.10 rules out, and a reader could not see the givens in the file's own form.

Switching on and overriding is option 1 (C.10). A table the repository writes under a channel the fragment holds is on unless it writes `on = false`, so `[channels.alpha]` with nothing under it switches `alpha` on, the one line C.10 names, since its keys are already written beneath; `on = false` under `beta` switches `beta` off. A key the repository writes replaces the fragment's: `on_each_publish` and `bound_to` together, since the two say how the channel moves (37), and `require` the claims after the gate's acceptance. A channel the table in force has off is not listed, has no policy, and is named by no binding or environment. Option 2 takes a table header and a key, two lines, where the fragment already holds every key; option 3 refuses the override C.10 grants.

The policy file is option 1 (83, 131, C.10). One `resolve` reads the table in force in six steps, each unreadable naming its table on the first break: the components, each `[[component]]`'s `name` or none; the channels' tables, a table of the other form, under a component not declared, or naming the `component` key of before unreadable naming the form to use; how each table in force moves, on each publish with `on_each_publish = true`, bound with `bound_to`, by a promotion alone otherwise, both keys unreadable; the environments, `[environments.<name>]` or `[environments.<component>.<name>]`, of which it reads `channel`, `policy` and `after` and leaves every other key to its target, `manual` making a channel named for the environment bound to the channel it names and `after` one bound to the channel the environments it names follow, their own where they have one, the named environments following one channel and none of them it again, a made channel never named as a channel the fragment holds, on or off; a table under a made channel read as a line under it, adding its claims after the gate's acceptance and read only where it restates how the channel moves; and every binding, and every environment's channel, naming a channel of its component that is on, no binding a cycle (37, 125). A revision with no policy file lists the givens that are on, of the one component. Option 2 lets the channels listed and one channel's policy disagree, which 83 forbids. A repository declaring components that wrote `component = "<name>"` inside `[channels.<name>]`, the form only this bolt's `the-policy-seam-lists-the-channels` wrote, rewrites it as `[channels.<component>.<name>]`.

The gate is option 1 (95, 131, core.1, core.2). It reads the channels answer at the branch's landed head, keeps the channels of the component published that move on each publish, and looks for the signer it accepts as in the policy for acceptance and then in the policy for each channel listed; it names no key of the policy file. A following channel moves only where the gate's identity is among its promoters, and is refused naming a promotion and those promoters otherwise, that channel's answer alone, the acceptance standing (9, 32). The ledger's promotion finds its channel the same way, among the channels listed for the component chosen, absent naming both where none is, takes the channel's binding from how it moves, and checks the host's signer against its promoters. Option 2 puts the file's form in the core, which 131 and core.1 rule out, and the gate and the ledger would each read channels the seam does not list.

The policy command is option 1 (C.10, 58, core.31, core.32). `tailrace policy [--branch main] [--json]` asks the core's `in_force`, which resolves the branch's head, checks the landing landed it, and answers the policy for acceptance, for a layout and the channels listed, each channel with its own policy, each part its own answer, so an absent layout or one unreadable channel leaves the rest shown. It is bound as status is, to the landing and the policy, with no store and no signer, and writes nothing. The table in force is shown through the seam's answers, the givens beside the repository's own, so the command reads no file and the core no profile. Option 2 would carry the file profile's form across the seam, and B.5 names no such operation; so a held channel switched off is not shown, which is named for madswan-design.

### Consequences

- A person who writes nothing about channels gets `beta`, moved by the gate on every accepted publish, and `stable`, moved by `tailrace promote` from what `beta` held, on a host holding the gate's signer; `alpha` or `lts` is one empty table away.
- Every accepted publish now asks the ledger of `beta`; a host with no `[ledger]` answers `beta` absent naming the table while the acceptance stands.
- A repository may now rebind a given or switch it off, and so bind a channel to one that is off or make a cycle of bindings; each is unreadable naming a table, since no version could reach the channel (37).
- A channel declared by hand under a name the fragment holds now switches that given on, its own keys over the fragment's.
- A channel an environment's words make is listed and promoted like a given channel; nothing moves an `after` channel by rule, or converges an environment, until phase 2's first target.
- `tailrace policy` shows the channels on, never a held one switched off.
- The environment table's form, the `on` key and the empty table are this unit's reading of C.10; who may say yes to a manual environment, a line narrowing who may ask for a given channel, and `lts` promoted by a person alone wait on a later word, and the asker stays the host's own signer (129).
- The fake policy gives no channel a test did not script; the given channels are the policy file's, which the policy's conformance group holds every profile to listing beside the declared like any of them, its rig for the file reading them from the shipped fragment.
