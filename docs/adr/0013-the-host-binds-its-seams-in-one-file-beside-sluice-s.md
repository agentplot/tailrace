---
number: 13
title: The host binds its seams in one file beside sluice's
status: proposed
date: 2026-10-24
decision-makers:
- Chuck Swanberg
---

# The host binds its seams in one file beside sluice's

Deciders: proposed by the-laptop unit of phase 1's bolt 2, which lets a person on a laptop publish with the installed binary and a second machine verify what it published (`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 2), for the user to accept with the change.
Sources: statement clauses 58, 59, 62, 64, 65, 79, 80, 82, 83, 92, 116, 118, C.2, C.3, C.5, C.6, C.8, C.10; ADRs 0005, 0008; rules core.1, core.3, core.19, core.21, core.22; sluice's `crates/sluice/src/instance.rs` and `crates/sluice-core/src/policy.rs`; the change's design, D1 to D3.

## Context and problem statement

Every profile the laptop needs is built: the OCI layout on disk, moon, sluice and a plain branch, and the host key. The installed binary binds none of them, so `tailrace publish` and `tailrace verify` answer that nothing is bound, and no profile of the policy seam exists, so no verify on any machine can name a signer. The statement settles who binds what: the operator binds the store, the signers, the landing and the ledger for a host, and the authors bind the build, the projections and the targets in the repository (92); the operator's bindings are one file beside the landing's own, in the same form, readable with a file reader (118); the policy is a file in the repository (C.10). Where is the operator's file, what form does it take, what does a binding hold, and what form does the policy file give its signers and audiences?

## Considered options

For the host's file:

1. `$XDG_CONFIG_HOME/tailrace/config.toml`, else `~/.config/tailrace/config.toml`, beside sluice's own `config.toml` under one directory, in TOML, each seam's table naming its profile by its one key.
2. A `tailrace.toml` inside sluice's configuration directory.
3. A flat string per seam, such as `store = "layout:<path>"`.
4. A path in `$TAILRACE_CONFIG`.

For what a binding holds:

1. The host's file names the store, the landing and the signer; the build, the projections and the policy come from the repository the command runs in.
2. The host's file names moon as the build as well, as the bolt plan's table reads.

For the policy file's signers and audiences:

1. sluice's `[signers.<name>]`, one verifier key beside `kinds` and `checks`, and an ordered `require` per audience: `[acceptance]`, `[layout]`, `[channels.<name>]` with its `bound_to`.
2. The reconstruction's `predicate from [signer]` lines.
3. Each signer listed inline in each `require` entry.

## Decision outcome

The host's file is option 1. It sits where sluice keeps its own host settings, with `tailrace` for `sluice`, so the two are one directory apart and read alike (118). `[store] layout = "<path>"` names the layout (C.2), `[landing]` names `sluice` or `branch`, each with an optional `remote` (C.5, C.6), and `[signing]` takes sluice's own table, `signer` and `key` (C.8). A later profile is a new key of its seam's table, not a new shape. A table naming no profile or two is unreadable naming it, and so are `[build]` and `[projections]`, which the repository binds (92), `[ledger]`, which names no profile built yet, and any other table. A path is absolute or begins `~/`, since a relative one would name another file in each working directory. No file is absent naming its path; a broken one is unreadable with the parser's words (58, 59, core.19). The file names a key's path and never holds a key (64, core.21). Option 2 puts tailrace's file in a directory sluice owns; option 3 is a form sluice's file does not use; option 4 is a variable no tool of the family reads and 118 does not need.

A binding is option 1. The store, the landing and the signers come from the host's file; moon, the one build profile, the policy file and every projection built come from the repository, whose policy file names each output's task and projection (92, C.3, C.10). The ledger is bound to nothing that answers until the ledger unit, and no target is bound. Only publish makes the layout it names, the first time; verify and status never make or write one (60, core.32), and status reads no bindings file at all. A host whose file names no `[signing]` binds the host key with no key, which verifies by the keys the policy lists and signs nothing (79, 80), and its publish answers absent naming `[signing]` before anything is read. The code that reads the file names no tool (core.1): each seam's table is handed to its profile family, which alone matches the key to a profile. Option 2 is rejected because 92 gives the build to the authors: a build named by the host would bind one repository's build for every repository on it.

The policy file is option 1. A signer's one key besides `kinds` and `checks` is its verifier, passed whole to the signer profiles so each reads its own (82, 116); `require` keeps the order the authors wrote, which fixes the first claim found wanting (58); only a channel names `bound_to` (83). It is read with git from the landed revision's tree and never from a checkout (65, core.22). Option 2 keeps a form no TOML reader keeps; option 3 repeats a key per audience and lets two entries disagree about one signer.

The policy file is the policy seam's one profile, and one is enough (core.3): C.10 is the one member of Part C for the policy, the statement names the policy as a file in the repository, and the components, the version rule and the projections the core reads are in the same file. A second profile waits on a member of Part C that names a policy held elsewhere.

### Consequences

- `tailrace publish` and `tailrace verify` work from the installed binary on a host with a bindings file, and a second machine with a copy of the layout, a clone and no key verifies what the first published.
- One host binds one store, one landing kind and one signer (92). A repository landed otherwise is a bindings file away, on another host or under another configuration directory.
- The ledger's binding waits on bolt 4's ledger unit; until then a `[ledger]` table is refused.
- The policy file's signer and audience form is provisional, and the question of whether it is the one C.10 means is with the design agent; a different form changes the policy file profile and its tests alone.
