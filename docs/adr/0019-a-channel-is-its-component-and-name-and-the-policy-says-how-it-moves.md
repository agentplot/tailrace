---
number: 19
title: A channel is its component and name, and the policy says how it moves
status: proposed
date: 2026-10-25
decision-makers:
- Chuck Swanberg
---

# A channel is its component and name, and the policy says how it moves

Status: proposed to the user on 2026-10-27, as `briefs.15` has it; accepted only on their yes. Revised on the user's question on its first card and on their note on its second, both quoted below.

The user's question, on the record's card (2026-10-27): "I don't know if we need it here but is it possible to require additional checks for channel gates? In addition to maybe having a person approve, we have to run a full end-to-end integration test or something"

The user's note, on the record's second card (2026-10-27): "i can't tell if alpha/* and lts/* are considered here still. these are not singular channels." Their words settle the shape of the two held givens, families and not channels; the rest of this record's words are madswan-design's.

Deciders: proposed by the-channels-answer-says-how-each-moves unit of phase 1's bolt 4, which ships the given channels as a policy fragment laid beneath the repository's file, makes the channels an environment's words call for, has the gate and the ledger learn every channel from the policy seam's one answer, and adds `tailrace policy` to show the table in force and the givens held off, for the user to accept with the change. Every choice below is the statement's as ruled at 53f6d44, b1267d7, 088430c, 1ae1cc5 and 9e537bb; none is left to this unit's reading.
Sources: statement clauses 32, 37, 95, 102, 125, 129, 131, 136, C.10, C.13, C.14, as amended at `agentplot/blueprints` 7749369, e18ba89, 53f6d44, b1267d7, 088430c, 1ae1cc5 and 9e537bb, with 67f95da, 85bbf54 and d50535a; ADRs 0009, 0010, 0016; rules core.1, core.2, core.7, core.31, core.32, core.37; the change's design, D1 to D6, D11 and D13; the user's question and note on the record's cards, 2026-10-27; clauses 125, 127 and C.10 as amended at `agentplot/blueprints` on 2026-10-27; `crates/tailrace-conformance/src/register.rs` (127 pending, phase 2) and `crates/tailrace/src/profiles/store/registry/tag.rs` (design D8) as origin/main held them that day.

## Context and problem statement

The policy seam listed the channels written as `[channels.<name>]` tables, each naming its component with a `component` key, and the channel it is bound to. Nothing else read that answer: the gate parsed the policy file itself to find the channels that follow each publish and the signer it accepts as, and the ledger's promotion read the channel's policy for its binding and let a signer promote where the channel's `require` named it for a promotion. Every seam that takes a channel took its name alone, so two components' `beta` shared one chain, one ref and one store name, which 125 forbids (ADR 0010, core.37).

The statement as amended at 7749369 makes a channel's identity its component and its name, fixes how it is spelled in the ledger's ref (C.13), the store (C.14) and the policy file (C.10), gives every component `beta` and `stable`, and has a given channel promoted by the gate's own signer (32); 85bbf54 gives an environment `policy` and `after`, which call for channels of their own. e18ba89 makes the givens a policy fragment in the file's own form, shipped with the core and laid beneath the repository's file, which the policy command shows as the one table in force; adds `alpha` and `lts`, held until one line switches each on; lets a repository override a given by writing the same line; and has the core know the three ways a channel moves and no channel's name (C.10). 53f6d44 rules that writing a held given's table is the one switch, that a following channel takes prereleases only where its line says so, and that `approvers` names who may ask. b1267d7 has the policy command show each given held off with the line that switches it on. 088430c declares the givens though no channel line is written (102), puts `approvers` on the environment or the channel meant and never on a channel the core made for an environment, and makes the fragment's hold mark its own, refused in a repository's file. 1ae1cc5 has the channels answer carry the givens held off, marked held and naming their line, apart from the channels in force, with no further operation for them (131). 9e537bb has the file refuse `approvers` on an environment whose policy is `auto`, which names no one who could be asked, naming the field (C.10).

Ten things were to be decided: how a channel's identity crosses the seams; how the answer says a channel moves; who signs a channel's promotion and who may ask for it; where the givens live and how the fragment holds some back; how a held given is switched on and a given overridden; where a manual environment's approvers are written; how the policy file resolves its channels; how the gate learns them; how a prerelease moves them; and how the policy command shows the table in force and what is held off.

## Considered options

For a channel's identity:

1. One seam type, a component that is none where the repository declares no component, and a name; spelled `<name>` or `<component>/<name>`.
2. A component always, `repository` meaning none is declared.
3. The spelled string itself.

For how a channel moves:

1. One enum: on each accepted publish, saying whether it takes prereleases; by a promotion alone; or bound to a channel of its component.
2. The listing's `bound_to` kept, beside a flag that the channel follows each publish.

For who signs a channel's promotion and who may ask for it:

1. The policy for the channel names its promoters and, apart from them, its approvers, none meaning any asker the gate verifies.
2. The given channel's answer adds `promotion` to the kinds of the gate's signer, and approvers join the promoters.
3. The channel says it is given, and the core works out who the gate is from the signers.

For where the givens live and how some are held back:

1. One fragment in the policy file's own form, `givens.toml` in the policy profile's directory, compiled in and laid beneath the repository's file, marking each given it holds back with `held = true`, a key of the fragment alone.
2. Two fragments, one of the givens declared and one of those held back.
3. The held givens named by a list in the profile's Rust.
4. Constants in the profile's Rust.

For switching a held given on and overriding a given:

1. The given's table written by the repository, empty or with lines, is the one switch; each key it writes replaces the fragment's, `on_each_publish` and `bound_to` together and with them the fragment's `prereleases` unless the repository writes its own.
2. An `on` key written under the given's table, `on = true` to switch one on and `on = false` to switch one off.
3. A line under a given that would change how it moves refused.

For a manual environment's approvers:

1. Written on the environment, the made channel taking its environment's; never written on the made channel.
2. Written under the made channel, `[channels.<environment>]`.

For the policy file's channels:

1. One resolution of the table in force, shared by the channels listed, one channel's policy and the givens held off.
2. Each operation reading the tables it needs.

For the gate's channels:

1. The policy seam's channels answer, and the policy for each channel listed.
2. The policy file's tables, as the gate read them before.

For a prerelease:

1. The gate moves, of the channels that follow each publish, every one with a release and only those that take prereleases with a prerelease, the version a prerelease where its semantic version carries a prerelease part.
2. Every following channel with every version.

For the policy command and the givens held off:

1. A question to the core answering, at the branch's landed head, the policy for acceptance and a layout and every channel the channels answer lists with its policy, each its own answer, and beneath them the givens held off that the same channels answer carries, marked held, each with the line that switches it on.
2. A third policy seam operation answering the givens held off.
3. The command reading the fragment itself.
4. A policy seam operation answering the merged file in its TOML form.

## Decision outcome

A channel's identity is option 1 (125). `ChannelId` is defined with the seams beside `ChannelName`, and every seam that takes a channel takes it: the policy's channel audience, the keeper's append and chain, the store name `channel/<spelled>`, the refusals a promotion gives, and a target's environment. It is spelled by its name where the repository declares no component, so nothing a repository of one component has written or landed moves, and by its component, a slash and its name where it declares components: `refs/tailrace/ledger/<component>/<channel>` (C.13) and `channel/<component>/<channel>` in the store (C.14), which fills the component ADR 0016 left to the policy seam's list. Channels are ordered by name, then component. Option 2 spells a declared component named `repository` as none and collides with nothing to tell the two apart; option 3 has every reader parse the identity back. The promotion claim's body keeps `channel` and `component` as two fields, as ADR 0016 made it.

How a channel moves is option 1 (131, 37, 83, 95, C.10). `Moves` is `{"by": "each_publish", "prereleases": <bool>}`, `{"by": "promotion"}` or `{"by": "bound", "to": "<channel>"}`; the channel's binding leaves the listing's own field for its third case, and `prereleases` lives on the one case it can be true of. Option 2 has two fields that can say a channel both follows each publish and is bound, which 37 forbids and the enum cannot.

Who signs and who may ask is option 1 (32, 129, core.7). `Required::Channel` gains `promoted_by`, the signers whose promotion of the channel counts, and `approvers`, the signers that may ask for it to move, each with what it may claim; acceptance and a layout name neither. The policy file names the gate's signers, every `[signers]` entry that may claim acceptance, as the promoters of every given the table in force lists and every channel an environment's words make, whatever a line under the channel names, so a line never widens who signs (C.10); such a channel requires the gate's acceptance first, then the line's claims. A channel declared by hand is promoted by the signers its `require` names that may claim a promotion. A channel's `approvers` are the signers its table in force names under `approvers`, each one `[signers]` holds and the list never empty; where none is named any asker the gate verifies may ask, the host's own signer while no identity provider stands (129). `tailrace promote` refuses an asker the approvers do not name, naming the asker and who may ask, before anything is signed or appended; approvers never join who signs. The gate's move on a publish is its own act, naming its own signer as who asked, so a channel's approvers do not bind it. Option 2 rewrites what a signer may claim and lets a line widen who signs; option 3 puts the policy's judgment of who the gate is in the core.

**What a channel may require before it moves, the user's question.** Yes, and both at once. A channel's line may require, beside who may ask, the claims a version must carry before it moves into the channel: C.10 says a line under a channel narrows who may ask or adds a claim its versions must carry, and `require` is that key. A claim is any signer's signed statement about the version's digest, that a test ran and passed among them (the statement's claim, 29), so a full end-to-end run is a signer the file names, which attaches its verdict to the version it tested, and `require` under `[channels.stable]` names that claim; `approvers` under the channel, or `policy = "manual"` on the environment, names the persons whose yes is also needed (97, 129). A promotion whose version lacks a required claim is refused naming the claim, and nothing moves; scenario S6 is the same shape, with a second signer's rebuild as the claim required. The gate itself runs nothing (29): the run happens wherever it runs and signs what it found. Nothing in phase 1 builds the signer that runs a test and attaches its verdict; a repository that wants `stable` gated this way stands it up as a signer of its own, and the policy file does the rest.

The givens are option 1 (C.10, 102). `givens.toml` holds `[channels.beta]` moving on each accepted publish with releases alone and `[channels.stable]` bound to `beta`, both declared, so a repository that writes no channel line still has them and the channels answer carries them, never as undeclared; and `[channels.alpha]` moving on each accepted publish with `prereleases = true` and `[channels.lts]` bound to `stable` and naming no rule, so only an asking moves it, both marked `held = true`; each under a comment citing C.10. `held` is the fragment's own mark: a repository's file never carries it, and `held` written there, or any key written to switch a channel on or off, is unreadable naming the table and the key, saying that writing a channel's table is the switch and nothing turns `beta` or `stable` off. The profile lays the fragment beneath the repository's file for every component it declares, as `[channels.<name>]` where none is declared and `[channels.<component>.<name>]` where some are; one function makes that table in force and the resolution alone reads it. A fragment that does not parse, or holds any table but a channel's, makes every answer unreadable naming the givens, and a test of the shipped one keeps that from happening; `PolicyFile::with_givens` takes another fragment, so a test proves nothing but the fragment names a given. No Rust source of the profile names a channel, and the core, the seams and the fakes name none. Option 2 splits what one file shows a reader together; options 3 and 4 have Rust name channels, which C.10 rules out, and a reader could not see the givens in the file's own form.

**`alpha/*` and `lts/*` are families, the user's note.** Clause 127, the user's ruling of 2026-10-24, declares a channel by a pattern over branch names, `alpha/<branch>`, whose channel for a branch exists from its first promotion and is retired with the branch; no unit of phase 1 carves it, and the conformance register holds it pending for phase 2. This unit wrote the fragment's `alpha` and `lts` as two single channels held off, which the user's note says they are not. On their words the statement holds the two givens as patterns (C.10 and 127 as amended on 2026-10-27): `alpha/*`, each branch's channel following every accepted version that branch publishes, prereleases included, retired with its branch; and `lts/*`, each line's channel made at the promotion that first names it, moved by a person alone from among what `stable` has held, retired by the authors' word in the file, its chain readable after. Each branch's or line's channel is a channel of its own under 125, with one chain, one store name and one policy, spelled `alpha/<branch>` and `lts/<line>`; it reads after a component without doubt, since a component's name holds no slash (125), and the store already holds such a name as one tag (ADR 0015, design D8). What this record built stands for the channels a policy names singly, `beta`, `stable` and every channel written by hand or made for an environment. The single `alpha` and `lts` leave the fragment in the first change after this record is accepted, so no held given invites a reader to switch on a channel the design does not have, and the two patterns come with the change that builds 127, whose ways of moving gain the branch a pattern's channel follows and the line a promotion names.

Switching on and overriding is option 1 (C.10). Writing `[channels.alpha]`, empty or with lines, switches `alpha` on, since its keys are already written beneath, and a held given the repository does not write is no channel: not listed, its policy absent, named by no binding or environment. No key switches a channel on or off, and nothing turns `beta` or `stable` off. A key the repository writes replaces the fragment's: `on_each_publish` or `bound_to` how the channel moves, the two together and the fragment's `prereleases` with them; `prereleases` whether a following channel takes prereleases, unreadable on a channel that does not move on each publish; `require` the claims after the gate's acceptance; and `approvers` who may ask. Option 2 takes two lines where one is enough and gives a repository a way to turn `beta` off, which C.10 does not; option 3 refuses the override C.10 grants.

A manual environment's approvers are option 1 (C.10). A repository writes `approvers` on the environment, or on the channel it means, and never on a channel the core made for an environment; the made channel takes its environment's, and `approvers` written under a made channel is unreadable naming its table. An environment's table holds C.10's words and no others: `channel`, `policy`, `auto` or `manual`, `auto` unless written, `after`, `target`, whose value the policy profile leaves to the target's reader, and `approvers` where the environment is manual, each a `[signers]` name and the list never empty; `approvers` on an environment whose policy is `auto` is unreadable naming the table and the key, since it names no one who could be asked (C.10, as ruled at 9e537bb), and any other key is unreadable naming the table and the key, since a target's own settings sit in the target's table and nowhere else. Option 2 puts who may say yes to an environment on a channel the repository did not write.

The policy file is option 1 (83, 131, C.10). One `resolve` reads the table in force in six steps, each unreadable naming its table on the first break: the components, each `[[component]]`'s `name` or none; the channels' tables, a table of the other form, under a component not declared, or naming the `component` key of before unreadable naming the form to use; how each table in force moves, with its `prereleases` and its `approvers`; the environments, `[environments.<name>]` or `[environments.<component>.<name>]`, `manual` making a channel named for the environment bound to the channel it names and `after` one bound to the channel the environments it names follow, their own where they have one, the named environments following one channel and none of them it again, a made channel never named as a channel the fragment holds, held or not; a table under a made channel read as a line under it, adding its claims after the gate's acceptance and read only where it restates how the channel moves; and every binding, and every environment's channel, naming a channel of its component in force, no binding a cycle (37, 125). The givens held off are read from the same table in force, so the channels in force and the givens held off never disagree and no channel is in both. A revision with no policy file lists the givens the fragment declares, of the one component. Option 2 lets the channels listed and one channel's policy disagree, which 83 forbids. A repository declaring components that wrote `component = "<name>"` inside `[channels.<name>]`, the form only this bolt's `the-policy-seam-lists-the-channels` wrote, rewrites it as `[channels.<component>.<name>]`.

The gate is option 1 (95, 131, core.1, core.2). It reads the channels answer at the branch's landed head, keeps the channels of the component published that move on each publish, and looks for the signer it accepts as in the policy for acceptance and then in the policy for each channel listed; it names no key of the policy file. A following channel moves only where the gate's identity is among its promoters, and is refused naming a promotion and those promoters otherwise, that channel's answer alone, the acceptance standing (9, 32). The ledger's promotion finds its channel the same way, among the channels listed for the component chosen, absent naming both where none is, takes the channel's binding from how it moves, and checks the asker against its approvers and the host's signer against its promoters. Option 2 puts the file's form in the core, which 131 and core.1 rule out, and the gate and the ledger would each read channels the seam does not list.

A prerelease is option 1 (C.10, 95, 21). The version is a prerelease where it parses as a semantic version with a prerelease part, the test the version rule makes of a tag; the acceptance stands whichever channels move. Option 2 moves a channel that takes releases alone, `beta` among them, to a prerelease.

The policy command is option 1 (C.10, 58, 131, core.31, core.32). `tailrace policy [--branch main] [--json]` asks the core's `in_force`, which resolves the branch's head, checks the landing landed it, and answers the policy for acceptance, for a layout and the channels in force, each channel with its own policy, each part its own answer, so an absent layout or one unreadable channel leaves the rest shown. Beneath the table it shows each given held off, from the same channels answer, which carries them marked held and apart from the channels in force (1ae1cc5): the channel spelled, how it would move, and `switch on with <line>`, the line in the profile's own words, `[channels.alpha]` or `[channels.<component>.alpha]`, which the core and the command carry and never parse. The policy seam gains no operation for them: 136 gives it `policy-for`, `channels` and `environments` and no other. A channel row shows how it moves, `each publish`, `each publish, prereleases too`, `promotion` or `bound to <channel>`, its claims, its promoters, and who may ask, `any asker` where it names no approvers. It is bound as status is, to the landing and the policy, with no store and no signer, and writes nothing. Option 2 adds an operation B.5 does not name; option 3 has a surface read a profile's data (core.31), and a profile with other givens would be shown the file's; option 4 carries the file profile's whole form across the seam.

### Consequences

- A repository that wants a channel to move only after an end-to-end run and a person's yes writes `require`, naming the run's claim and its signer, and `approvers`, naming the person, under that channel's table; the run's signer is the repository's to stand up, since the gate judges claims and runs nothing (29).
- A person who writes nothing about channels gets `beta`, moved by the gate on every accepted release, and `stable`, moved by `tailrace promote` from what `beta` held, on a host holding the gate's signer; `alpha/*` and `lts/*` come as patterns with 127, each one table away, and until then `tailrace policy` shows no held given.
- Every accepted publish now asks the ledger of `beta`; a host with no `[ledger]` answers `beta` absent naming the table while the acceptance stands.
- A repository may now rebind a given, and so make a cycle of bindings, which is unreadable naming a table in it, since no version could reach the channel (37).
- A channel declared by hand under a name the fragment holds now overrides that given, its own keys over the fragment's; until the patterns come, `[channels.alpha]` or `[channels.lts]` written by hand declares a channel of the repository's own, as any other name does, and no single given answers to it.
- A file writing `held` under a channel, or a key this bolt read to switch one on or off, is unreadable naming the key; writing the channel's table is the switch.
- A channel an environment's words make is listed and promoted like a given channel; nothing moves an `after` channel by rule, or converges an environment, until phase 2's first target.
- While no identity provider stands the asker is the host's own signer, which must also be a promoter; `approvers` naming no gate signer leaves the channel movable by none, and `tailrace policy` shows both lists side by side.
- The fake policy gives and holds off no channel a test did not script; the given channels are the policy file's, which the policy's conformance group holds every profile to listing beside the declared like any of them, its rig for the file reading them from the shipped fragment.
