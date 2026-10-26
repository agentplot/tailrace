---
number: 19
title: The host names what tells its gate, and its publish step tells its own
status: proposed
date: 2026-10-25
decision-makers:
- Chuck Swanberg
---

# The host names what tells its gate, and its publish step tells its own

Deciders: proposed by the-publishs-host-tells-the-gate unit of phase 1's bolt 4, which lets a laptop that is both the offerer and the gate move `beta` on every landing in the one run of `tailrace publish` (`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 4), for the user to accept with the change.
Sources: statement clauses 20, 58, 59, 95, 108, 118, 130, C.16; ADRs 0013, 0016; rules core.1, core.4, core.10, core.14, core.19; the change's design, D1 to D3.

## Context and problem statement

The gate judges a version a publish told it of and moves each channel that follows each publish, and nothing tells it: `tailrace publish` offers and stops, and no command asks the gate. The statement says the core starts nothing after an offer, and that what tells the gate is named in the host's bindings file and nowhere in the core (130, C.16). Three things were open: how the bindings file names what tells the gate; where the one teller a single host needs, its own publish step, calls the gate from, given rule core.10 rules out a publish that moves `beta`; and what the gate judges when it is told, since a name read twice may name two digests (core.4).

## Considered options

For the bindings file:

1. A `[gate]` table naming its teller by one key, as `[store]`, `[landing]` and `[ledger]` name their profiles: `publish = {}`, this host's publish step, or `job = "<words>"`, a job outside tailrace that a store's or a forge's event wakes, in the operator's words.
2. One string key, `told_by = "publish"`.
3. One key per item of C.16: `forge`, `store`, `asker`.
4. A key of `[signing]` saying whether the host is the gate.

For where the telling sits:

1. In the command, which calls the core's publish and, once it has answered, the core's gate for each version it offered.
2. A core function that publishes and then judges.
3. A second command, `tailrace accept`, run after `publish`.

For what the gate judges when a publish tells it:

1. The telling carries the index the publish's answer named; the gate resolves the version's name as always and, where it resolves to another digest, answers unreadable naming both before anything is read further or written.
2. The command compares the gate's answer with the publish's afterwards.

## Decision outcome

The bindings file is option 1 (118, 130, C.16). `[gate]` extends the list of tables ADR 0013 gave the file, as ADR 0016 filled `[ledger]`; 0013 stands as written. The table is read by the generic one-key reader the seams' tables use: `publish` takes an empty table, as `[landing] sluice = {}` does, and `job` a string with words in it. tailrace runs nothing for `job` and reads its words only to carry them in the publish's answer as written, since the core calls none of C.16's tellers. A `[gate]` naming no key, two keys, any other key, options on `publish` or a `job` with no words is unreadable naming the file and the table, read before anything is made (58, 59, core.19). A file with no `[gate]` binds no teller: each version a publish offers says that nothing on this host tells the gate, naming `[gate]`, and the publish's exit is its own. The module that reads the file names no tool (core.1). Option 2 is the flat form ADR 0013 turned down, and a separated teller would need a second key for its words; option 3 names three tellers tailrace behaves alike for, since it starts none of them, and a teller tailrace later builds becomes a new key of the same table; option 4 asks the bindings a question the policy answers, whether this host may accept (108), beside the separate fact of who tells the gate.

The telling is option 1 (130). The core's publish and the gate share no code, and the core's publish still moves no channel, so `publishing`'s "a publish offers and does nothing more" holds of it (20). The step that calls one and then the other is the host's binding of what tells the gate, which is why it sits in the binary that reads the bindings file. The gate signs as the host's `[signing]` signer and moves each following channel through the ledger `[ledger]` binds; whether that signer may accept is the policy's question, and where it may not the gate refuses the version as offered by the gate and writes nothing (27, 96, core.14). Rule core.10 gains a line saying this telling is not ruled out, since the judgment and every move after it are the gate's own (95, 108). Option 2 would have the core start something after an offer (130); option 3 is a second process for the landing step to run, and the bindings file would then name a step rather than the telling.

The gate's judgment is option 1 (core.4). `ToldBy::Publish` carries the index digest; a version the gate is asked about by anyone else carries none and is judged by what its name resolves to. Option 2 checks too late: by then the gate has signed and attached its acceptance on whatever the name resolved to.

### Consequences

- On a host whose file binds `[gate] publish = {}` and whose policy lets its signer claim both provenance and acceptance, one `tailrace publish` offers, accepts and moves `beta`, and the answer says so beside each version; its exit is the worst of the publish's and the gate's, a refusal or a channel held counting as refused, so a landing step that runs it fails when `beta` did not move.
- A host whose file names a job, or nothing, publishes as it did and says what tells the gate, asking the gate nothing.
- What runs a publish after a landing, the job a separated gate runs, the events that wake it and a command that runs the gate on request are named and not built; each waits on a unit the planner names.
- A host with no bindings file binds no teller, as 118 lists none among its defaults, so its publish says nothing tells the gate.
