---
number: 21
title: A host with no bindings file is bound to the given profiles
status: accepted
date: 2026-10-25
decision-makers:
- Chuck Swanberg
---

# A host with no bindings file is bound to the given profiles

Status: accepted by the user on 2026-10-27, on their card.

Deciders: proposed by the a-host-with-no-bindings-file-is-bound-to-the-defaults unit of phase 1's bolt 4, which lets a person on a laptop publish with nothing written (`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 4), for the user to accept with the change.
Sources: statement clauses 58, 108, 118, C.2, C.5, C.6, C.8, C.10, C.13, as amended at blueprints ab0dfea, 85bbf54, 061bfa8, b64d367 and 53f6d44; the design agent's batch 18 rulings; ADR 0013; rules core.1, core.7, core.14, core.19, core.21, core.22, core.32; the change's design, D1 to D7a.

## Context and problem statement

ADR 0013 put the host's bindings in `$XDG_CONFIG_HOME/tailrace/config.toml` and decided that a host with no file is answered absent naming its path, that a file naming no `[store]` or `[landing]` is answered absent naming the table, and that a host naming no `[signing]` signs nothing. So every command on a laptop asked for a file before it did anything, and even with one, a laptop's publish moved no channel unless the policy named the laptop as its own gate. Clause 118, as the statement now reads it, binds a host with no file to a layout under the user's data directory (C.2), a host key made on first use (C.8), the branch or sluice where the repository is set up for it (C.6, C.5), and the git ref (C.13) on the landing's own remote, so two machines of one repository read one ledger with nothing written; its landing remote is the checkout's `origin` or its one remote; its publish tells its own gate. C.10 places the gate of a policy that names none in the policy's answer: where the file names no signer that may claim acceptance, the policy answers the host's own signer, the one it names for provenance, as the repository's gate, and the gate reads this from the answer and decides nothing of it (core.7); where the file names several signers for provenance and none for acceptance, 108 admits no host, nothing is accepted, and the refusal says so in words C.10 gives. Eight things were open: what a missing file and a missing table bind and by which path through the code; where each default lives, given rule core.1 keeps every tool's name out of the bindings module; how the given landing tells sluice from a branch; which remote it reads; where the given key is made and what name it signs as; where the given ledger keeps its chain; where the gate of a policy that names none is decided; and what a policy with several publishers and no gate answers.

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

1. Made by its first sign at `tailrace/host/signing.key` under `$XDG_DATA_HOME`, or `~/.local/share` where that is unset or empty, an Ed25519 key in OpenSSH's form with no passphrase, mode 0600, its public half beside it as `signing.key.pub`, each by a temporary file hard-linked into place; signing as the comment of `signing.key.pub` where it is there, else as the host's short name, its first label in lower case, which the public half fixes once the key is made.
2. Made at `signing` beside the bindings file.
3. Made when the host is bound.
4. Named by running `hostname`, or by a crate for the host's name.

For the given ledger:

1. The git ref on the landing's remote, as the file writes it or as the checkout gives it, fetched from it before each read and pushed to it on each append as the branch is; the repository's own refs, with nothing pushed, where the landing has no remote.
2. The git ref in the repository the command runs in alone, with no remote.

For a policy that names no gate:

1. The policy file's given gate, at the policy seam: where `[signers]` names no signer that may claim acceptance and exactly one that may claim provenance, the profile answers that signer as one that may claim acceptance too, in every audience's claims, and never rewrites the file; the gate reads the answer as it reads any policy.
2. The gate reading a policy that names no gate as naming its own identity, as the policy names it for provenance, the one host 108 admits.
3. A key in the policy file naming the host as its gate.

For a policy that names several publishers and no gate:

1. The policy file's refusal: the policy for acceptance, and for every given or made channel, refused wanting an acceptance no signer may make, `Why::Wanting` with no signers, in the words C.10 gives, `several publishers and no gate, name an accepting signer`; `[layout]` and a channel declared by hand answered as written.
2. A new `Why` variant for it.

## Decision outcome

A missing file is option 1 (118). There is one way to be bound to the defaults, and no answer of `read` that nothing reads. A host where neither `XDG_CONFIG_HOME` nor `HOME` is set is still answered absent naming both, since no given path can be placed. A file that is there is never guessed past: one that is not well formed, or a table naming no profile or two, is unreadable naming it and binds nothing given in its place (58, 59, core.19).

Each default lives in its family, option 1 (core.1). The store family gives the layout at `tailrace/layout` under the user's data directory as a `[store] layout` table the file left out, so it is bound, named in the host's record and made by a publish or a push exactly as a written one is (C.2, 61); verify, sync, rebuild and status never make it (core.32), and status opens a store only for a ledger held in it. A given table names itself as given, `[store] layout (given)`, in every answer about it. Option 2 would put each tool's name in the module rule core.1 keeps clean.

The given landing is option 1 (C.5, C.6, core.22). A commit is what a landing is set up by; the working tree may hold a file nobody committed, and rule core.22 already keeps the policy out of the checkout.

The given remote is option 1 (118). A written `[landing]` binds the remote it names or none, exactly as written, so a written file means what it meant and a host can still say "read my own refs" in a clone; option 2 would change what every written file with no `remote` binds.

The given key is option 1 (118, C.8, core.21), as the design agent ruled in batch 18. The key is the host's made state and not its configuration, so it sits under the data directory beside the given layout, and the configuration directory holds nothing tailrace made. It is made by the sign that needs it, so a publish that answers nothing is landed makes none; the hard link makes two first uses end with one key, both signing with it; a key file that is there is never replaced, whatever it holds. The name is fixed in `signing.key.pub` once the key is made, because a laptop's name can change with the network it is on and the policy lists the name; that line is what a policy lists under `[signers.<name>]`. Only publish and promote bind the given key; every other command binds both signer profiles verify-only. Option 2 puts made state among the files a person writes; option 3 makes a key for a publish that then refuses; option 4 needs a tool not every shell holds, or a dependency for one call, where `libc`'s `gethostname` is already in the lockfile.

The given ledger is option 1 (118 at blueprints b64d367, C.13). The ledger is kept where the landing reads, so a second machine of the same repository reads the chain the first appended with nothing written, and a remote that does not answer leaves the chain answered from the copy as last fetched, saying when (60). A written `[ledger]` binds what it names. Option 2 would leave each machine a chain of its own, and a second machine's status would show none of the first's promotions.

A policy that names no gate is option 1 (C.10 at blueprints 061bfa8, 108, 118, core.7, core.14). The admission is the policy's to make, so the gate decides nothing of whom the policy names: it already admits a signer the policy names for both provenance and acceptance, and already says one host in that acceptance claim. Every audience's answer follows from the one reading of `[signers]`: acceptance's required provenance names the host with both kinds, and a given or made channel requires the acceptance of, and is promoted by, the signers that may claim acceptance, the host among them; `verify` reads the same answer (core.31). Where `[signers]` names a signer that may claim acceptance, the answer is the file as written, and a build host running the gate is refused as offered by the gate (27, 96). Rule core.14's details admit the host the policy file answers as the gate beside the one host a policy names for both. Option 2 puts in the gate's code a reading of whom the policy names, which C.10 keeps out of it (core.7); option 3 asks for something written where 118 asks for nothing.

A policy that names several publishers and no gate is option 1 (C.10 at blueprints 53f6d44, 58, 59, core.19). `Why::Wanting` already says that no claim of a kind counts under the policy, which lets the listed signers make it; here the policy lets no signer make an acceptance, and the profile's own words say why and name the fix. The gate's reading of the acceptance policy answers the refusal before any claim is read, so nothing is accepted and no channel moves, and the publish's `gate` member carries the refusal with its words. Option 2 changes the seam's shared type for one sentence the existing variant carries.

### Consequences

- ADR 0013's decisions for a missing file, a missing table and a host with no `[signing]` are amended: each now binds the given profiles. Its file, its location, its form and its tables stand.
- A first `publish` on a laptop makes the layout, at the store's first operation, and the key, at the first sign, both under the data directory, and its provenance is signed by the host's name; the gate's answer beside it says the version is not accepted until the policy lists that name, with the line `signing.key.pub` holds. A publish refused before it asks the store anything, a revision not landed among them, makes neither.
- A checkout with several remotes and no `origin` answers every command unreadable, naming the remotes and `[landing] remote`, the one line that fixes it.
- A host with nothing written pushes its ledger refs to the landing's remote, fast-forward only, with the host's own git credentials at the moment of use (core.21); a team that wants the chain elsewhere names `[ledger]`.
- Where the policy file names its host for provenance and no signer that may claim acceptance, every version that host offers is accepted as one host, and a given channel on such a host is promoted by the host's signer: `beta` moves on each publish with nothing written. Naming a signer that may claim acceptance ends it, and from then on the build host is refused as offered by the gate; a channel's own policy may require a separated acceptance (108).
- Where the policy file names several publishers and no gate, nothing is accepted and every answer says so in the policy's words, until the file names an accepting signer.
- What runs a publish after a landing, and a registry, a key service or a shared ledger elsewhere than the landing's remote as a default, are left to the file and to later units.
