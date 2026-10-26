# Proposal

## Why

Today `tailrace publish`, `verify`, `sync`, `push`, `rebuild` and
`promote` answer absent on a host with no bindings file, and a file that
leaves out `[store]`, `[landing]`, `[ledger]` or `[signing]` is answered
absent for the table it left out. So a person on a laptop has to write a
file before anything works, and so does every plugin that runs a command
for them. The statement says otherwise: a host with no such file is
bound to a layout under the user's own data directory, a host key made
on first use, the branch or sluice, and the git ref, and the file is for
pointing a host at a registry, a key service or a shared ledger (118,
C.2, C.5, C.6, C.8, C.13). This change makes that true.

## What Changes

- **No file binds the given profiles.** A host with no bindings file is
  bound to:
  - the store: an OCI layout at `$XDG_DATA_HOME/tailrace/layout`, or
    `~/.local/share/tailrace/layout` where that variable is unset or
    empty (C.2);
  - the landing: sluice, where the repository is set up for it, which
    is where its `HEAD` commit holds sluice's own policy file,
    `.config/sluice.toml`, read with git and never from the working
    tree; a plain branch otherwise (C.5, C.6); with no remote either way,
    so the repository's own refs are read;
  - the ledger: the git ref, `refs/tailrace/ledger/...`, in the
    repository the command runs in, with no remote (C.13);
  - the signer: a host key at `signing` beside where the bindings file
    would be, `$XDG_CONFIG_HOME/tailrace/signing` or
    `~/.config/tailrace/signing`, made the first time a claim is signed
    (C.8).
- **A file that names some seams leaves the rest at these.** Each of
  `[store]`, `[landing]`, `[ledger]` and `[signing]` the file names
  binds what it names, exactly as today; each it leaves out binds the
  given profile above. An empty file binds all four given profiles. A
  broken file, or a table that names no profile or two, is still
  unreadable naming it (58, 59, core.19): nothing about a file that is
  there is guessed.
- **The host key is made on first use.** The first `publish` or
  `promote` on a host whose file names no `[signing]` makes an Ed25519
  key in OpenSSH's format with no passphrase, readable by its owner
  alone, the form sluice signs with, and writes its public half beside
  it as `signing.pub`, a line a policy can list as it stands. The
  signer's name is the host's own name, its first label in lower case,
  fixed when the key is made and read back from `signing.pub` after
  that. Nothing else makes the key: `verify`, `sync`, `rebuild` and
  `push` sign nothing, and bind the host key verify-only as today. Two
  first uses at once end with one key, and both sign with it.
- **What the record names stays the same.** The given layout is named in
  the host's record as a written one is, `layout <path>` with the path in
  full (61).
- **The docs say it.** ADR 0013's answers for a missing file and missing
  tables are replaced by a new ADR, and the site's self-hosting, store,
  landing, signer and ledger pages describe the given bindings.

Out of scope:

- `tailrace status` still reads no bindings file (60); nothing changes
  for it.
- `[gate]`, what tells the gate, is the-publishs-host-tells-the-gate
  unit's, and 118 gives it no default. A file with no `[gate]` answers
  as that unit says.
- Telling the person, in the publish's own answer, that a key was just
  made and that their policy must list it before its claims count. The
  site's signer page says how, and no command's answer changes. A
  later change can add that line to the answer if the user wants it.
- A registry, a key service or a shared ledger as a default. The
  statement keeps those for the file.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `host-bindings`: a missing file and a missing table bind the given
  profiles instead of answering absent; the landing is chosen by
  whether the repository is set up for sluice; a laptop with nothing
  written publishes from the installed binary.
- `host-key-signer`: the host's given key is made on first use, with its
  public half beside it, and its signer's name is the host's.

## Impact

- `crates/tailrace/src/bindings/`: `read` answers an empty set of
  bindings for a missing file; `bind` takes each missing table's given
  binding from its profile family.
- `crates/tailrace/src/profiles/{store,landing,ledger,signer}/mod.rs`:
  each family gives its given binding, so the bindings module still
  names no tool (core.1).
- `crates/tailrace/src/profiles/signer/host_key/`: a host key that makes
  its key on first sign.
- `Cargo.toml`, `crates/tailrace/Cargo.toml`, `Cargo.lock`: `libc`, for
  the host's name. It is already in the lockfile through other crates.
- `crates/tailrace/tests/laptop.rs`: a laptop with no bindings file
  publishes a sluice landing and verifies it.
- `docs/adr/`: a new ADR amending 0013, and 0013's status line naming
  it.
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
crates/tailrace/Cargo.toml
Cargo.toml
Cargo.lock
docs/adr/0013-the-host-binds-its-seams-in-one-file-beside-sluice-s.md
docs/adr/*-a-host-with-no-bindings-file-is-bound-to-the-given-profiles.md
site/src/content/docs/guides/self-hosting.mdx
site/src/content/docs/guides/integrations/stores.mdx
site/src/content/docs/guides/integrations/landings.mdx
site/src/content/docs/guides/integrations/signers.mdx
site/src/content/docs/guides/integrations/ledgers.mdx
```
