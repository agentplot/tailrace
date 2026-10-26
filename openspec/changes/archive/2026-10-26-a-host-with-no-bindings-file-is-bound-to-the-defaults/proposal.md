# Proposal

## Why

On this bolt, `tailrace publish`, `verify`, `sync`, `push`, `rebuild` and
`promote` answer absent on a host with no bindings file, and a file that
leaves out `[store]`, `[landing]` or `[signing]` is answered absent for
the table it left out. So a person on a laptop has to write a file
before anything works, and so does every plugin that runs a command for
them; and even with a file, a laptop's publish cannot move `beta` unless
the policy names the laptop as its own gate. The statement says
otherwise (118, as amended at blueprints ab0dfea, 85bbf54 and b64d367;
C.10 at 061bfa8 and 53f6d44): a host with no such file is bound to a layout under
the user's own data directory, a host key made on first use, the branch
or sluice, and the git ref on the landing's own remote, so two machines
of one repository read one ledger with nothing written; its landing
remote is the checkout's `origin`, or its one remote; its publish tells
its own gate; and where the policy
file names no signer that may claim acceptance, the policy answers the
host's own signer as the one host 108 admits, so `beta` moves on a
laptop with nothing written; where it names several publishers and no
gate, nothing is accepted and the refusal says so. The file is for pointing a host at a
registry, a key service or a shared ledger. This change makes that
true.

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
  - the ledger: the git ref, `refs/tailrace/ledger/...`, on the
    landing's own remote, fetched from it before each read and pushed to
    it on each append as the branch is, so a second machine of the same
    repository reads the chain the first appended; where the landing has
    no remote, the repository's own refs, where it keeps its branch, and
    nothing is pushed (C.13);
  - the signer: a host key in the user's data directory,
    `$XDG_DATA_HOME/tailrace/host/signing.key`, or
    `~/.local/share/tailrace/host/signing.key` where that variable is
    unset or empty, made the first time a claim is signed (C.8, as the
    design agent ruled in batch 18);
  - what tells the gate: the host's own publish, as `[gate] publish =
    {}` binds it (130).
- **A file that names some seams leaves the rest at these.** Each of
  `[store]`, `[landing]`, `[ledger]`, `[signing]` and `[gate]` the file
  names binds what it names, exactly as written, a `[landing]` without a
  `remote` still reading the repository's own refs; each it leaves out
  binds the given binding above, a `[ledger]` left out following
  whatever remote the landing is bound to, written or given. An empty file binds them all. A broken
  file, or a table that names no profile or two, is still unreadable
  naming it (58, 59, core.19): nothing about a file that is there is
  guessed.
- **The host key is made on first use.** The first `publish` or
  `promote` on a host whose file names no `[signing]` makes an Ed25519
  key in OpenSSH's format with no passphrase, readable by its owner
  alone, the form sluice signs with, and writes its public half beside
  it as `signing.key.pub`, a line a policy can list as it stands. The
  signer's name is the host's short name, its first label in lower case,
  fixed when the key is made and read back from `signing.key.pub` after
  that. Nothing else makes the key: `verify`, `sync`, `rebuild`, `push`
  and `status` sign nothing, and bind the host key verify-only. Two
  first uses at once end with one key, and both sign with it.
- **Where the policy file names no gate, its answer names the host.**
  Where `[signers]` names no signer that may claim acceptance and one
  signer that may claim provenance, the policy file profile answers that
  signer as one that may claim acceptance too: the repository's gate for
  acceptance and for the given channels' promotion, the one host 108
  admits (C.10). Every audience and every verdict read that one
  answer. The gate decides nothing of it: it already
  admits a signer the policy names for both provenance and acceptance,
  and already says one host in that acceptance claim (108, core.7), so
  the gate's code does not change. Where the file names a signer that
  may claim acceptance, its answer is the file as written, and a build
  host running the gate is still refused (27, 96, S9).
- **Several publishers and no gate is refused in words.** Where
  `[signers]` names several signers that may claim provenance and none
  that may claim acceptance, 108 admits no host: the policy file profile
  answers the policy for acceptance, and for each given or made channel,
  refused, wanting acceptance by no signer, in its own words `several
  publishers and no gate, name an accepting signer` (58, C.10). So the
  gate accepts nothing there and its answer carries those words, no
  channel moves, and `verify` shows the same refusal.
- **So a laptop with nothing written moves `beta`.** A person who has
  written no bindings file, in a repository whose policy lists their
  host's key for provenance only and writes no channel, runs `tailrace
  publish` after a landing, and the same run offers the version, accepts
  it as one host, and moves the given `beta` through the git ref.
- **The command line says it.** The command-line requirement that an
  unbound host answers absent goes: `publish`, `verify`, `sync`, `push`
  and `promote` on a host with no file are bound to the defaults, and
  answer absent or unreadable only where a default cannot be made: no
  home and no configuration directory, or several remotes and no
  `origin` among them.
- **What the record names stays the same.** The given layout is named in
  the host's record as a written one is, `layout <path>` with the path in
  full (61).
- **The docs say it.** ADR 0021 records the given bindings and the
  policy file's given gate, and amends ADR 0013; rule core.14's details
  admit the host the policy file answers as the gate where it names
  none; the site's self-hosting, store, landing, signer and ledger pages
  describe the given bindings.

Out of scope:

- What runs a publish after a landing (118's last sentence: the
  landing's step after its push where the landing is sluice, a person
  where it is a branch, C.5, C.16). Here a person or the landing host's
  own step runs `tailrace publish`; binding that step waits on a unit
  the planner names.
- The given channels, `beta` and `stable` with no line written, and the
  fragment that gives them, are the `the-channels-answer-says-how-each-moves`
  unit's. This unit comes after it and adds only the given gate to the
  policy file profile it leaves.
- Telling the person, in the publish's own answer, that a key was just
  made and that their policy must list it before its claims count. The
  site's signer page says how. A later change can add that line to the
  answer if the user wants it.
- A registry, a key service, or a ledger held anywhere but the
  landing's remote, as a default. The statement keeps those for the
  file.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `host-bindings`: a missing file and a missing table bind the given
  bindings instead of answering absent; the landing is chosen by
  whether the repository is set up for sluice and reads the checkout's
  one remote or `origin`; the given ledger is kept on the landing's
  remote; status and the serving name are bound the same way; a laptop
  with nothing written publishes and moves `beta`.
- `host-key-signer`: the host's given key is made on first use, with its
  public half beside it, and its signer's name is the host's.
- `policy-file`: where `[signers]` names no signer that may claim
  acceptance, the profile answers the one signer it names for
  provenance as the gate, and refuses in words where it names several.
- `command-line`: an unbound host is bound to the defaults, and sync,
  push and promote on a host with no file are bound like publish.

## Impact

- `crates/tailrace/src/bindings/`: `read` answers an empty set of
  bindings for a missing file; `bind` takes each missing table's given
  binding from its profile family, for every command, status included;
  the second, status-only binding of a host with no file goes.
- `crates/tailrace/src/profiles/{store,landing,ledger,signer}/mod.rs`:
  each family gives its given binding, so the bindings module still
  names no tool (core.1); the landing family gives the given remote,
  and the ledger family's given git ref takes the landing's remote.
- `crates/tailrace/src/profiles/signer/host_key/`: a host key that makes
  its key on first sign.
- `crates/tailrace/src/profiles/policy/file/`: the given gate, where
  `[signers]` names no signer that may claim acceptance and one that
  may claim provenance, and the refusal where it names several.
- `crates/tailrace/src/commands/serve/host.rs`: the serving address's
  repository named from the given landing remote.
- `crates/tailrace-core/src/gate.rs` is not changed: the code this
  unit's first build put there, and the tests of it in the core's and
  the conformance suite's gate tests, are taken out again.
- `Cargo.toml`, `crates/tailrace/Cargo.toml`, `Cargo.lock`: `libc`, for
  the host's name. It is already in the lockfile through other crates.
- Tests: the bindings', signers' and policy file's unit tests, the
  command line's, host's and promote's tests, the publish-tells-the-gate
  and one-host tests, `tests/policy_file.rs` and `tests/laptop.rs`.
- `docs/adr/0021-*`, amending 0013, and 0013's status line naming it;
  `docs/architecture/core.md`, core.14's details.
- `site/src/content/docs/guides/`: the self-hosting page and the store,
  landing, signer and ledger integration pages.
- `openspec/specs/`: the archive writes this change's deltas into the
  host-bindings, host-key-signer, policy-file and command-line specs.
- Order: this unit comes after `the-channels-answer-says-how-each-moves`,
  whose policy file profile and given fragment it builds on, and after
  `the-publishs-host-tells-the-gate` and `status-on-any-machine`, whose
  bindings and command-line requirements it edits.

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
crates/tailrace/src/profiles/policy/file/mod.rs
crates/tailrace/src/profiles/policy/file/tests.rs
crates/tailrace/src/commands/serve/host.rs
crates/tailrace/tests/cli.rs
crates/tailrace/tests/host.rs
crates/tailrace/tests/promote.rs
crates/tailrace/tests/laptop.rs
crates/tailrace/tests/one_host.rs
crates/tailrace/tests/policy_file.rs
crates/tailrace/tests/publish_tells_the_gate.rs
crates/tailrace/tests/status_across_machines.rs
crates/tailrace-core/src/gate.rs
crates/tailrace-core/tests/gate.rs
crates/tailrace-conformance/src/core/gate.rs
crates/tailrace/Cargo.toml
Cargo.toml
Cargo.lock
docs/adr/0013-the-host-binds-its-seams-in-one-file-beside-sluice-s.md
docs/adr/0021-a-host-with-no-bindings-file-is-bound-to-the-given-profiles.md
docs/architecture/core.md
site/src/content/docs/guides/self-hosting.mdx
site/src/content/docs/guides/integrations/stores.mdx
site/src/content/docs/guides/integrations/landings.mdx
site/src/content/docs/guides/integrations/signers.mdx
site/src/content/docs/guides/integrations/ledgers.mdx
openspec/specs/host-bindings/spec.md
openspec/specs/host-key-signer/spec.md
openspec/specs/policy-file/spec.md
openspec/specs/command-line/spec.md
```
