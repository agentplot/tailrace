---
number: 19
title: A channel is its component and name, and the policy says how it moves
status: proposed
date: 2026-10-25
decision-makers:
- Chuck Swanberg
---

# A channel is its component and name, and the policy says how it moves

Deciders: proposed by the-channels-answer-says-how-each-moves unit of phase 1's bolt 4, which gives every component `beta` and `stable` with no line written, makes the channels an environment's words call for, and has the gate and the ledger learn every channel from the policy seam's one answer, for the user to accept with the change; the environment table's form is this unit's reading of C.10, named for madswan-design.
Sources: statement clauses 32, 37, 95, 125, 131, C.10, C.13, C.14, as amended at `agentplot/blueprints` 7749369, with 67f95da and 85bbf54; ADRs 0009, 0010, 0016; rules core.1, core.2, core.7, core.37; the change's design, D1 to D5.

## Context and problem statement

The policy seam listed the channels written as `[channels.<name>]` tables, each naming its component with a `component` key, and the channel it is bound to. Nothing else read that answer: the gate parsed the policy file itself to find the channels that follow each publish and the signer it accepts as, and the ledger's promotion read the channel's policy for its binding and let a signer promote where the channel's `require` named it for a promotion. Every seam that takes a channel took its name alone, so two components' `beta` shared one chain, one ref and one store name, which 125 forbids (ADR 0010, core.37). The statement as amended at 7749369 makes a channel's identity its component and its name, fixes how it is spelled in the ledger's ref (C.13), the store (C.14) and the policy file (C.10), gives every component `beta` and `stable`, and has a given channel promoted by the gate's own signer (32); 85bbf54 gives an environment `policy` and `after`, which call for channels of their own. Five things were left open: how a channel's identity crosses the seams; how the answer says a channel moves; who the policy names as a channel's promoters; how the policy file resolves its channels; and how the gate learns them.

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

For the policy file's channels:

1. One resolution of the file, shared by the channels listed and one channel's policy.
2. Each operation reading the tables it needs.

For the gate's channels:

1. The policy seam's channels answer, and the policy for each channel listed.
2. The policy file's tables, as the gate read them before.

## Decision outcome

A channel's identity is option 1 (125). `ChannelId` is defined with the seams beside `ChannelName`, and every seam that takes a channel takes it: the policy's channel audience, the keeper's append and chain, the store name `channel/<spelled>`, the refusals a promotion gives, and a target's environment. It is spelled by its name where the repository declares no component, so nothing a repository of one component has written or landed moves, and by its component, a slash and its name where it declares components: `refs/tailrace/ledger/<component>/<channel>` (C.13) and `channel/<component>/<channel>` in the store (C.14), which fills the component ADR 0016 left to the policy seam's list. Channels are ordered by name, then component. Option 2 spells a declared component named `repository` as none and collides with nothing to tell the two apart; option 3 has every reader parse the identity back. The promotion claim's body keeps `channel` and `component` as two fields, as ADR 0016 made it.

How a channel moves is option 1 (131, 37, 83, 95). `Moves` is `{"by": "each_publish"}`, `{"by": "promotion"}` or `{"by": "bound", "to": "<channel>"}`, and the channel's binding leaves the listing's own field for its third case. Option 2 has two fields that can say a channel both follows each publish and is bound, which 37 forbids and the enum cannot.

Who may sign a channel's promotion is option 1 (32, core.7). `Required::Channel` gains `promoted_by`, the signers whose promotion of the channel counts, each with what it may claim; acceptance and a layout name none. The policy file names the gate's signers, every `[signers]` entry that may claim acceptance, as the promoters of `beta`, `stable` and every channel an environment's words make, whatever a line under the channel names, so a line never widens who signs (C.10); a channel declared by hand is promoted by the signers its `require` names that may claim a promotion, as the ledger's rule stood. Option 2 rewrites what a signer may claim, and a line's own promotion signer would still widen who signs; option 3 puts the policy's judgment of who the gate is in the core.

The policy file is option 1 (83, 131, C.10). One `resolve` reads, in order, each step unreadable naming its table on the first break: the components, each `[[component]]`'s `name` or none; the channels' tables, `[channels.<name>]` where none is declared and `[channels.<component>.<name>]` where some are, a table of the other form, under a component not declared, or naming the `component` key of before unreadable naming the form to use; `beta`, moving on each publish, and `stable`, bound to `beta`, given to every component; the environments, `[environments.<name>]` or `[environments.<component>.<name>]`, of which it reads `channel`, `policy` and `after` and leaves every other key to its target, `manual` making a channel named for the environment bound to the channel it names, and `after` one bound to the channel the environments it names follow, their own where they have one, the named environments following one channel, none of them it again, and a made channel never named `beta` or `stable`; each table under a given or made channel read as a line under it, adding its claims after the gate's acceptance and read only where it restates how the channel moves; each other table a channel declared by hand, moving on each publish with `on_each_publish = true`, bound with `bound_to`, by a promotion alone otherwise, and unreadable with both; and every binding, and every environment's channel, within its component (37, 125). A revision with no policy file lists `beta` and `stable` of the one component. Option 2 lets the channels listed and one channel's policy disagree, which 83 forbids. A repository declaring components that wrote `component = "<name>"` inside `[channels.<name>]`, the form only this bolt's `the-policy-seam-lists-the-channels` wrote, rewrites it as `[channels.<component>.<name>]`.

The gate is option 1 (95, 131, core.1, core.2). It reads the channels answer at the branch's landed head, keeps the channels of the component published that move on each publish, and looks for the signer it accepts as in the policy for acceptance and then in the policy for each channel listed; it names no key of the policy file. A following channel moves only where the gate's identity is among its promoters, and is refused naming a promotion and those promoters otherwise, that channel's answer alone, the acceptance standing (9, 32). The ledger's promotion finds its channel the same way, among the channels listed for the component chosen, absent naming both where none is, takes the channel's binding from how it moves, and checks the host's signer against its promoters. Option 2 puts the file's form in the core, which 131 and core.1 rule out, and the gate and the ledger would each read channels the seam does not list.

### Consequences

- A person who writes nothing about channels gets `beta`, moved by the gate on every accepted publish, and `stable`, moved by `tailrace promote` from what `beta` held, on a host holding the gate's signer.
- Every accepted publish now asks the ledger of `beta`; a host with no `[ledger]` answers `beta` absent naming the table while the acceptance stands.
- A channel an environment's words make is listed and promoted like a given channel; nothing moves an `after` channel by rule, or converges an environment, until phase 2's first target.
- `stable` is always bound to `beta`, so a version reaches `stable` only once `beta` has held it.
- The environment table's form is this unit's reading of C.10; who may say yes to a manual environment, and a line narrowing who may ask for a given channel, wait on a later word, and the asker stays the host's own signer (129).
- The fake policy gives no channel a test did not script; the given channels are the policy file's, which the policy's conformance group holds every profile to listing beside the declared like any of them.
