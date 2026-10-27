---
number: 16
title: The ledger's git ref holds one claim per commit, and a channel is named in the store
status: accepted
date: 2026-10-25
decision-makers:
- Chuck Swanberg
---

# The ledger's git ref holds one claim per commit, and a channel is named in the store

Status: accepted by the user on 2026-10-27, on their card.

Amendment (proposed to the user on 2026-10-27, on their card; accepted only on their yes): the binding of `[ledger]` this record decided, `tailrace promote` needing `[ledger]` and a file naming none binding a ledger that answers absent, is superseded by record 0021 and clause 118 as amended at `agentplot/blueprints` b64d367, under which a file naming no `[ledger]` binds the git ref on the landing's remote. The ruling that put the component into the ledger ref and the store name was `agentplot/blueprints` 7749369, not 67f95da. The rest of the record stands as accepted.

Deciders: proposed by the-ledger-and-promotion unit of phase 1's bolt 4, which builds the ledger seam's two profiles and the promotion itself, so a person promotes a version with one command, undoes a wrong move by promoting back, and a second machine reads the move from the ledger alone (`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 4), with how a chain is read where its source does not answer, and the asker where no identity provider stands, ruled by madswan-design, for the user to accept with the change.
Sources: statement clauses 9, 31 to 35, 37 to 39, 58 to 61, 64, 83, 101, 105, 106, 109, 120 to 122, 125, 129, C.13, C.14, as amended at `agentplot/blueprints` 673d334 and 67f95da; ADRs 0009, 0013; rules core.1, core.2, core.6, core.7, core.15, core.21, core.34; git 2.54's `push`, `fetch`, `update-ref`, `hash-object`, `mktree` and `reflog`, as run for the change; the change's design, D2, D3, D5, D7 and D9 to D11.

## Context and problem statement

ADR 0009 made the ledger a signed chain per channel and where it is held a seam, with a git ref (C.13) and the store (C.14) among its members. The seam had a trait and a fake and nothing more, and ADR 0013 held a `[ledger]` table in the host's bindings file for the ledger's unit while refusing it. To build both profiles and the promotion, five things were left open: what a commit of the git ref holds and how an append reaches the remote; what name a channel takes in the store, which the core moves and the store ledger walks from; how a chain read where its source does not answer says so (60, 61); what a promotion is judged by and who it records as asking (9, 31, 32, 129); and what `[ledger]` names.

## Considered options

For the git ref's commits:

1. One commit per claim on `refs/tailrace/ledger/<channel>`, its tree one blob, `promotion.json`, the envelope's canonical JSON, its one parent the commit of the claim before; append a push that only fast-forwards.
2. One commit per claim with the envelope in the commit message and an empty tree.
3. A file per claim on one branch, appended by a commit that adds it.

For a channel's name in the store:

1. `channel/<channel>`, beside `cache/` for a cache entry.
2. The channel's own name, `stable`.
3. No name: the store ledger finds the head by scanning every index's claims.

For a chain read where its source does not answer:

1. The chain answers how it was read, from the source or as last read, saying when and with the source's words; an append is never held by the copy.
2. Unreadable, always, with the source's words.
3. A second operation that reads the copy.

For the asker:

1. The host's own signer, the gate that signs the promotion, until an identity provider stands.
2. A text the asker gives on the command line.

## Decision outcome

The git ref is option 1 (C.13, 105). Each commit is made with `hash-object -w`, `mktree` and a commit object written by `hash-object -t commit`, by `tailrace <tailrace@localhost>` at the claim's own time, so no index, working tree or identity of the host's is touched and two hosts making one claim on one head make one commit. The chain is `rev-list --reverse --parents` of the ref; a commit with two parents, or whose tree holds no claim, is unreadable naming it, never answered in part. Every read with a remote lists the remote's ref, fetches it into a scratch ref made anew at each fetch, and fast-forwards the repository's own copy to it; a remote that holds less than the copy is unreadable naming the claims it lost, and one that parts from the copy is a fork naming the claim both sides follow and the first of each side, the copy kept (105, 122). Append is the read, the predecessor given against the head, then `push <remote> <commit>:<ref>` and never `+`; a refused push is read again and answered naming the remote's head where it moved, and with git's words where it did not (59). With no remote the repository's own ref is the ledger, moved by `update-ref` only from the commit it was read at. The push uses the host's own git credentials at the moment of use, and the profile keeps none (64, core.21). Option 2 makes a claim unreadable by any tool that reads trees; option 3 lets one commit hold two claims and a merge join two chains, which 105 forbids.

A channel's name in the store is option 1 (C.14). It is no version's name, a component's prefix and a version, nor a cache entry's, and the core's promotion and every ledger profile that walks from it use the one function the seam defines, `channel_name`. A promotion moves it only after the ledger holds the claim, from the index the channel moved from, a race lost on it repaired from the ledger (33, 109, core.6, core.15). Option 2 could collide with a version's name in a repository whose prefix is empty; option 3 reads every index for every chain and names no head when two claims race. Where a repository declares several components the name carries the component, as madswan-design ruled at 67f95da; the channel's component comes from the policy seam's list of channels (125, 131), which a later unit of this bolt builds, and until it does a channel's name is `channel/<channel>`.

The store ledger walks back from the index the channel's name names: of the channel's promotion claims attached there, the one whose walk by `from` and `predecessor` holds every other is the head; two that part, or two met on one index naming one predecessor, are a fork (105). Append attaches the claim to its index and then moves the name from the head's index; a move the store refuses is answered naming the head as walked again, and the claim attached before it stays, which a later walk that meets it reports as the fork it is. This is for a store the operator trusts to retain what it holds (C.14).

A chain's read is option 1, as madswan-design ruled at 673d334 (60, 61, C.13, C.14). `Keeper::chain` answers the claims and `Read::Source` or `Read::Kept { at, said }`. The git ref answers the repository's copy as last fetched or appended, `at` the newest entry of its reflogs; the store ledger keeps each chain it reads, with the time, in `tailrace/ledger/<channel>.json` under the repository's common git directory beside the record of published versions (61, 62), holding the chain and nothing else (64). Nothing moves a channel or a name from a chain read so: a promotion and the store's repair from the ledger refuse it unreadable with the source's words, and an append is unreadable. A bound channel's chain read so still serves its binding, since a chain only grows (122). Option 2 leaves a machine nothing to answer when its remote is away, which 60 asks of a read; option 3 is a second contract for one question.

A promotion is judged under the channel's policy at the branch's landed head. Each attestation on the version's index is told as verify tells it, under the signers the policy names, merged one each (core.7); the version needs an acceptance that counts whether or not the channel lists one (31), then each claim the policy requires, in order, the first found wanting refused naming its kind, its check and the signers that may make it (58), all before anything is signed. Only the index's claims are judged, a claim about a version being a claim about its index (7). The host's signer must be one the policy lets claim a promotion (9, 32), so the promotion is signed by the gate's identity, which on a solo host is the host's (108).

The asker is option 1, as madswan-design ruled at 67f95da (129, amended): until an identity provider stands, the promotion records as `asked_by` the host's own signer, which names a host and no person, and an asker is always a signer, never a phrase. The amended 129 also asks the record to say that it names a host; the body as built names the signer alone, and saying so is left to the change that revises the body. Option 2 records the asker's own word, which 129 forbids.

`[ledger]` fills the table ADR 0013 held for the ledger and lifts its refusal of it; 0013 itself stands as written. It names its profile by its one key, as `[store]` and `[landing]` do: `git_ref`, with an optional `remote`, the repository the command runs in holding the refs where it names none; or `store`, with no option, the chain held in the store `[store]` names through a binding of its own that never makes it. A table naming no profile, two, or an option its profile does not read is unreadable naming it. `tailrace promote` needs `[signing]` and `[ledger]`, absent naming each before anything is read, and never makes the layout; every other command binds the ledger where the file names it, and a host whose file names none binds one that answers absent naming `[ledger]`.

### Consequences

- A person promotes with `tailrace promote <version> --to <channel>`, undoes a wrong move with `--back`, and names where they expect the channel with `--from`; another machine with a clone and an empty store reads the channel from the git ref alone, and as last fetched when the remote is away.
- The ledger's code in the core names no git and reaches the chain through the ledger seam alone (core.1, core.2); the git ref and store ledgers each pass the keeper's conformance group.
- The gate's move of a channel that follows each publish should make the same claim, under the same name, through the same promotion; how the merged gate is brought to these forms is with the planner.
- A store ledger whose refused move left a claim attached reports a fork once a walk meets it; an operator who runs two gates against one store with no conditional names is to use the git ref.
- The time a git ref's chain is said to be read is the last time this repository fetched or wrote it, at a second's resolution.
