# Proposal

## Why

On this bolt, `tailrace publish`, `verify`, `sync`, `push`, `rebuild` and
`promote` answer absent on a host with no bindings file, and a file that
leaves out `[store]`, `[landing]` or `[signing]` is answered absent for
the table it left out. So a person on a laptop has to write a file
before anything works, and so does every plugin that runs a command for
them; and even with a file, a laptop's publish cannot move `beta` unless
the policy names the laptop as its own gate. The statement says
otherwise (118, as amended at blueprints ab0dfea): a host with no such
file is bound to a layout under the user's own data directory, a host
key made on first use, the branch or sluice, and the git ref; its
landing remote is the checkout's `origin`, or its one remote; its
publish tells its own gate; and where the policy names no gate the host
is the one 108 admits, so `beta` moves on a laptop with nothing written.
The file is for pointing a host at a registry, a key service or a
shared ledger. This change makes that true.

## What Changes

- **No file binds the given profiles.** A host with no bindings file is
  bound, for every command, status among them, to:
  - the store: an OCI layout at `$XDG_DATA_HOME/tailrace/layout`, or
    `~/.local/share/tailrace/layout` where that variable is unset or
    empty (C.2);
  - the landing: sluice where the repository is set up for it, which is
    where its `HEAD` commit holds sluice's own policy file,
    `.config/sluice.toml`, read with git and never from the working
    tree; a plain branch otherwise (C.5, C.6);
  - the landing's remote: the checkout's `origin`; its one remote where
    it has no `origin`; the repository's own refs where it has no remote
    at all; and unreadable, naming every remote it has, where it has
    several and none is `origin` (118);
  - the ledger: the git ref, `refs/tailrace/ledger/...`, in the
    repository the command runs in, read and written with no remote, so
    nothing is pushed anywhere (C.13);
  - the signer: a host key at `signing` beside where the bindings file
    would be, `$XDG_CONFIG_HOME/tailrace/signing` or
    `~/.config/tailrace/signing`, made the first time a claim is signed
    (C.8);
  - what tells the gate: the host's own publish, as `[gate] publish =
    {}` binds it (130).
- **A file that names some seams leaves the rest at these.** Each of
  `[store]`, `[landing]`, `[ledger]`, `[signing]` and `[gate]` the file
  names binds what it names, exactly as written, a `[landing]` without a
  `remote` still reading the repository's own refs; each it leaves out
  binds the given binding above. An empty file binds them all. A broken
  file, or a table that names no profile or two, is still unreadable
  naming it (58, 59, core.19): nothing about a file that is there is
  guessed.
- **The host key is made on first use.** The first `publish` or
  `promote` on a host whose file names no `[signing]` makes an Ed25519
  key in OpenSSH's format with no passphrase, readable by its owner
  alone, the form sluice signs with, and writes its public half beside
  it as `signing.pub`, a line a policy can list as it stands. The
  signer's name is the host's own name, its first label in lower case,
  fixed when the key is made and read back from `signing.pub` after
  that. Nothing else makes the key: `verify`, `sync`, `rebuild`, `push`
  and `status` sign nothing, and bind the host key verify-only. Two
  first uses at once end with one key, and both sign with it.
- **Where the policy names no gate, the publishing host is the gate.**
  A policy names a gate where some signer it requires a claim of, for
  acceptance or for any channel, may claim acceptance. Where none may,
  the gate signing as the host's signer reads that signer as the one
  host 108 admits: its own provenance counts rather than refusing the
  version as offered by the gate, its acceptance claim says one host
  offered and accepted, and a second telling of the same version finds
  that acceptance rather than making another (94, 108). Where the policy
  names a gate, nothing changes: a build host running the gate is still
  refused (27, 96, S9).
- **So a laptop with nothing written moves `beta`.** A person who has
  written no bindings file, in a repository whose policy names their
  host's signer for provenance and no gate, runs `tailrace publish`
  after a landing, and the same run offers the version, accepts it as
  one host, and moves each channel that follows each publish through
  the git ref.
- **What the record names stays the same.** The given layout is named in
  the host's record as a written one is, `layout <path>` with the path in
  full (61).
- **The docs say it.** A new ADR records the given bindings and amends
  ADR 0013; rule core.14's details admit the host where the policy names
  no gate beside the one host the policy names; the site's self-hosting,
  store, landing, signer and ledger pages describe the given bindings.

Out of scope:

- What runs a publish after a landing (118's last sentence: the
  landing's step after its push where the landing is sluice, a person
  where it is a branch, C.5, C.16). Here a person or the landing host's
  own step runs `tailrace publish`; binding that step waits on a unit
  the planner names.
- The given channels, `beta` and `stable` with no line written, are the
  `the-channels-answer-says-how-each-moves` unit's. This change moves
  the channels the policy says follow each publish, in whatever form
  the bolt reads them when it is built.
- Telling the person, in the publish's own answer, that a key was just
  made and that their policy must list it before its claims count. The
  site's signer page says how. A later change can add that line to the
  answer if the user wants it.
- A registry, a key service or a shared ledger as a default. The
  statement keeps those for the file.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `host-bindings`: a missing file and a missing table bind the given
  bindings instead of answering absent; the landing is chosen by
  whether the repository is set up for sluice and reads the checkout's
  one remote or `origin`; status and the serving name are bound the
  same way; a laptop with nothing written publishes and moves `beta`.
- `host-key-signer`: the host's given key is made on first use, with its
  public half beside it, and its signer's name is the host's.
- `acceptance`: where the policy names no gate, the gate's own signer is
  the one host 108 admits, and its acceptance says so.

## Impact

- `crates/tailrace/src/bindings/`: `read` answers an empty set of
  bindings for a missing file; `bind` takes each missing table's given
  binding from its profile family, for every command, status included;
  the second, status-only binding of a host with no file goes.
- `crates/tailrace/src/profiles/{store,landing,ledger,signer}/mod.rs`:
  each family gives its given binding, so the bindings module still
  names no tool (core.1); the landing family gives the given remote.
- `crates/tailrace/src/profiles/signer/host_key/`: a host key that makes
  its key on first sign.
- `crates/tailrace-core/src/gate.rs`: the gate reads a policy that names
  no gate as naming its own signer as the one host.
- `Cargo.toml`, `crates/tailrace/Cargo.toml`, `Cargo.lock`: `libc`, for
  the host's name. It is already in the lockfile through other crates.
- Tests: the bindings' and signers' unit tests, the core's gate tests,
  the conformance suite's gate group, the publish-tells-the-gate and
  one-host tests where their policies name no gate, and
  `crates/tailrace/tests/laptop.rs`.
- `docs/adr/`: a new ADR amending 0013, and 0013's status line naming
  it; `docs/architecture/core.md`, core.14's details.
- `site/src/content/docs/guides/`: the self-hosting page and the store,
  landing, signer and ledger integration pages.

## Touches

```
crates/tailrace/src/bindings/mod.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/src/profiles/store/mod.rs
crates/tailrace/src/profiles/landing/mod.rs
crates/tailrace/src/profiles/ledger/mod.rs
crates/tailrace/src/profiles/signer/mod.rs
crates/tailrace/src/profiles/signer/tests.rs
crates/tailrace/src/profiles/signer/host_key/mod.rs
crates/tailrace/src/profiles/signer/host_key/tests.rs
crates/tailrace/tests/laptop.rs
crates/tailrace/tests/one_host.rs
crates/tailrace/tests/publish_tells_the_gate.rs
crates/tailrace/Cargo.toml
crates/tailrace-core/src/gate.rs
crates/tailrace-core/tests/gate.rs
crates/tailrace-conformance/src/core/gate.rs
Cargo.toml
Cargo.lock
docs/adr/0013-the-host-binds-its-seams-in-one-file-beside-sluice-s.md
docs/adr/*-a-host-with-no-bindings-file-is-bound-to-the-given-profiles.md
docs/architecture/core.md
site/src/content/docs/guides/self-hosting.mdx
site/src/content/docs/guides/integrations/stores.mdx
site/src/content/docs/guides/integrations/landings.mdx
site/src/content/docs/guides/integrations/signers.mdx
site/src/content/docs/guides/integrations/ledgers.mdx
```
