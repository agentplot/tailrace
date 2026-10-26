---
number: 21
title: A host with no bindings file is bound to the given profiles
status: proposed
date: 2026-10-25
decision-makers:
- Chuck Swanberg
---

# A host with no bindings file is bound to the given profiles

Deciders: proposed by the a-host-with-no-bindings-file-is-bound-to-the-defaults unit of phase 1's bolt 4, which lets a person on a laptop publish with nothing written (`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 4), for the user to accept with the change.
Sources: statement clauses 108, 118, C.2, C.5, C.6, C.8, C.13; ADR 0013; rules core.1, core.7, core.14, core.19, core.21, core.22, core.32; the change's design, D1 to D7.

## Context and problem statement

ADR 0013 put the host's bindings in `$XDG_CONFIG_HOME/tailrace/config.toml` and decided that a host with no file is answered absent naming its path, that a file naming no `[store]` or `[landing]` is answered absent naming the table, and that a host naming no `[signing]` signs nothing. So every command on a laptop asked for a file before it did anything, and even with one, a laptop's publish moved no channel unless the policy named the laptop as its own gate. Clause 118, as the statement now reads it, binds a host with no file to a layout under the user's data directory (C.2), a host key made on first use (C.8), the branch or sluice where the repository is set up for it (C.6, C.5) and the git ref (C.13); its landing remote is the checkout's `origin` or its one remote; its publish tells its own gate; and where the policy names no gate the host is the one 108 admits. Seven things were open: what a missing file and a missing table bind and by which path through the code; where each default lives, given rule core.1 keeps every tool's name out of the bindings module; how the given landing tells sluice from a branch; which remote it reads; where the given key is made and what name it signs as; what the given ledger reads and writes; and what the gate does where the policy names no gate.

## Considered options

For a missing file:

1. An empty file: `read` answers every table missing, and `bind` gives each missing table its binding, one path for no file, an empty file and a file naming some tables, status among the commands.
2. `read`'s absent answer kept, and caught in `bind` to bind the defaults.

For where each default lives:

1. In each seam's profile family, which gives the binding it makes where its table is missing, the places it needs passed in.
2. In the bindings module, beside the reading of the file.

For the given landing:

1. sluice where the repository's `HEAD` commit holds `.config/sluice.toml`, read with git from that commit, and a plain branch otherwise, an unborn `HEAD` among them.
2. sluice where the working tree holds the file.

For the given landing's remote:

1. The checkout's `origin` where it has one, its one remote where it has one of another name, none where it has none, and unreadable naming every remote and `[landing]` where it has several and none is `origin`; read only where the file names no `[landing]`.
2. The same rule for a written `[landing]` that names no remote.

For the given key:

1. Made by its first sign at `signing` beside the bindings file, an Ed25519 key in OpenSSH's form with no passphrase, mode 0600, its public half beside it as `signing.pub`, by a temporary file hard-linked into place; signing as the comment of `signing.pub` where it is there, else as the host's name, its first label in lower case.
2. Made when the host is bound.
3. Named by running `hostname`, or by a crate for the host's name.

For the given ledger:

1. The git ref in the repository the command runs in, with no remote.
2. The git ref on the landing's remote.

For a policy that names no gate:

1. The gate reads its own identity, as the policy names it for provenance, as the one host 108 admits: its own provenance counts rather than refusing the version as offered by the gate, its acceptance says one host, and a second telling finds that acceptance already; nothing else of the gate changes, and no verdict but the gate's reads the policy so.
2. A key in the policy file naming the host as its gate.

## Decision outcome

A missing file is option 1 (118). There is one way to be bound to the defaults, and no answer of `read` that nothing reads. A host where neither `XDG_CONFIG_HOME` nor `HOME` is set is still answered absent naming both, since no given path can be placed. A file that is there is never guessed past: one that is not well formed, or a table naming no profile or two, is unreadable naming it and binds nothing given in its place (58, 59, core.19).

Each default lives in its family, option 1 (core.1). The store family gives the layout at `tailrace/layout` under `$XDG_DATA_HOME`, or `~/.local/share` where that is unset or empty, as a `[store] layout` table the file left out, so it is bound, named in the host's record and made by a publish or a push exactly as a written one is (C.2, 61); verify, sync, rebuild and status never make it (core.32), and status, which asks no store, answers why it could not be opened only where something asks. A given table names itself as given, `[store] layout (given)`, in every answer about it. Option 2 would put each tool's name in the module rule core.1 keeps clean.

The given landing is option 1 (C.5, C.6, core.22). A commit is what a landing is set up by; the working tree may hold a file nobody committed, and rule core.22 already keeps the policy out of the checkout.

The given remote is option 1 (118). A written `[landing]` binds the remote it names or none, exactly as written, so a written file means what it meant and a host can still say "read my own refs" in a clone; option 2 would change what every written file with no `remote` binds.

The given key is option 1 (118, C.8, core.21). The key is made by the sign that needs it, so a publish that answers nothing is landed makes none; the hard link makes two first uses end with one key, both signing with it; a key file that is there is never replaced, whatever it holds. The name is fixed in `signing.pub` once the key is made, because a laptop's name can change with the network it is on and the policy lists the name. Only publish and promote bind the given key; every other command binds both signer profiles verify-only. Option 2 makes a key for a publish that then refuses; option 3 needs a tool not every shell holds, or a dependency for one call, where `libc`'s `gethostname` is already in the lockfile.

The given ledger is option 1 (C.13). 118 names a remote for the landing alone and keeps a shared ledger for the file; a laptop's chain is its own until its file names a remote for it.

A policy that names no gate is option 1 (108, 118, core.7, core.14). A policy names a gate where some signer it requires a claim of, for acceptance or for any channel, may claim acceptance; the gate asks this once per judgment. Where it names none, the identity the gate signs as is admitted only as the policy names it for provenance: an identity the policy does not name is admitted by nothing, and its version is refused with its provenance found wanting. Where the policy names a gate, a build host running the gate is refused as offered by the gate, as before (27, 96). Rule core.14's details admit this host beside the one host the policy names for both. Option 2 asks for something written where 118 asks for nothing, and the policy already names the host's signer for provenance.

### Consequences

- ADR 0013's decisions for a missing file, a missing table and a host with no `[signing]` are amended: each now binds the given profiles. Its file, its location, its form and its tables stand.
- A first `publish` on a laptop makes the layout and the key, and its provenance is signed by the host's name; the gate's answer beside it says the version is not accepted until the policy lists that name, with the line `signing.pub` holds.
- A checkout with several remotes and no `origin` answers every command unreadable, naming the remotes and `[landing] remote`, the one line that fixes it.
- Where the policy names no gate, every version its host offers is accepted as one host, and a channel's own policy may require a separated acceptance (108).
- Which channels such a host moves is the channel's policy's to say. A given channel requires the gate's acceptance by the signers that may claim acceptance and is promoted by them, so where the policy names no gate it names no promoter; how a given channel moves on such a host is open.
- What runs a publish after a landing, and a registry, a key service or a shared ledger as a default, are left to the file and to later units.
