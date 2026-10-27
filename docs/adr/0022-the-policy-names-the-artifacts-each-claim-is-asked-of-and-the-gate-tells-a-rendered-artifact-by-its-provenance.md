---
number: 22
title: The policy names the artifacts each claim is asked of and the gate tells a rendered artifact by its provenance
status: proposed
date: 2026-10-27
decision-makers:
- Chuck Swanberg
---

# The policy names the artifacts each claim is asked of and the gate tells a rendered artifact by its provenance

Deciders: proposed by the the-gate-judges-each-artifact-by-its-kind unit of phase 1's bolt 5, which takes main's gate into the bolt that carries a version's notes (`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 5), for the user to accept with the change.
Sources: statement clauses 7, 13, 25, 28, 29, 82, 95, 134 and C.10, as amended at blueprints 955c25a; ADR 0007; rules core.7, core.28, core.31; the change's design, D1 to D4.

## Context and problem statement

Every version now carries its notes, an artifact the version rule renders from history with no build behind it, attached one provenance claim the publisher signs naming no build (134, core.9). The gate asked every artifact of a version for every claim the acceptance policy listed, so on a tree holding both the gate and the notes every version a policy asked a verdict of was refused, naming its notes: no build, so no verdict. Clause 134 now says the acceptance policy names what a built artifact requires and what a rendered one requires, the publisher's provenance alone, and the gate judges each artifact against the claims required of artifacts made its way (25, 28). Three things were open: where the policy says which artifacts a claim is asked of; how the policy file, whose C.10 form writes nothing for it, answers it; and how the gate tells that an artifact was rendered.

## Considered options

For where the policy says it:

1. Each required claim names the artifacts it is asked of, every artifact or a built artifact alone, in the policy seam's answer; a claim naming none reads as asked of every artifact, and one so asked is written as it was before.
2. Two lists on the acceptance and channel answers, one for built artifacts and one for rendered ones.
3. A method on the claim kind, in the seams, deciding it for every profile.

For how the policy file answers it:

1. From each claim's kind alone, with no key added: a provenance, an acceptance and a promotion of every artifact; a verdict, a test, a review, a rebuild and a carried claim of a built artifact alone; `[layout]`'s claims of every artifact.
2. A key on each `require` entry naming the artifacts it is asked of.

For how the gate tells a rendered artifact:

1. By its counted provenance: an artifact whose provenance of the version's revision, attached to it alone, counts through a signer the acceptance policy at the head names for a provenance, and names no build, where none that counts names one, is rendered; every other artifact is built. Told once per judgment, before any claim is counted, and read by acceptance and each following channel alike.
2. By the index's ecosystem `notes`.
3. By a mark in the index's metadata.

## Decision outcome

Where the policy says it is option 1 (82, 134, core.7). The choice is the policy's, so the gate reads it from the answer and decides none of it. One list keeps the policy's order, which fixes the first claim found wanting (58), once; option 2 keeps it twice and lets the two lists disagree on a signer the gate looks up across them; option 3 takes the choice from the policy that 134 gives it. Because a claim asked of every artifact is written as before, the publisher's layout-policy claim, signed and attached at publish, keeps its bytes, and every body written before reads back the same.

The policy file is option 1 (C.10). C.10 writes no form for it, so the file gains no key and an entry naming one is unreadable naming its table, as any unknown key is. A provenance is the publisher's word, which 134 lets stand alone for a rendered artifact; an acceptance and a promotion are claims about the version as a whole, which a claim on the index satisfies (7), and so ask nothing of the notes themselves. A review is a build's claim here because 134's "alone" leaves it out. Option 2 writes a form into the file that the design has not written; it waits on C.10.

The gate tells a rendered artifact by option 1 (13, 29, 134, core.7). The making is read from the claims attached alone and only from a provenance that counts, so a build-less claim from a signer the policy does not name for provenance makes nothing rendered, and a policy that asks no provenance judges every artifact as built, the stricter reading. A following channel is judged by the making told under acceptance, since a channel's policy rarely asks a provenance itself and telling it again there would judge the notes as built (95). Options 2 and 3 take the index's word for how an artifact was made, not a signer's, and option 2 names one kind of rendered artifact rather than the way it was made.

### Consequences

- A version whose outputs carry their verdicts is accepted beside notes that carry none, and its acceptance names the notes' provenance among the claims that satisfied the policy; `beta` moves on it where its own claims of built artifacts count.
- A rendered artifact still needs every claim asked of every artifact: notes the store answers no provenance of refuse the version whole, naming the notes (28).
- `tailrace policy` shows a claim asked of built artifacts alone as such, and `--json` carries it as the core answers it (core.31).
- The publisher's provenance decides that an artifact is asked no build's claims. A stolen publisher key could mark an artifact rendered and so offer it without a verdict; it could already offer any bytes with the same word, and 134 trusts that signer alone for a rendered artifact. Only its counted word counts, and every verdict a built artifact needs is still asked of each output.
- Consumer layouts and `verify` do not read the making: a layout holds one ecosystem's artifact and never the notes, and `verify` reports which claims count without refusing.
- A key with which the authors choose for themselves which artifacts a claim is asked of waits on C.10's form for it.
