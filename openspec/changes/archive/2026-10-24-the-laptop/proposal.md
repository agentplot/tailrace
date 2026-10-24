# Proposal

## Why

Every profile bolt 2 needs is built: the OCI layout on disk, moon, sluice
and a plain branch, and the host key. But the installed binary binds
none of them, so `tailrace publish` and `tailrace verify` answer that
nothing is bound, and the repository's policy can name no signer for
verify to check against. This change lets a person on a laptop publish a
landed revision into a layout on disk with the installed binary, and lets
a second machine verify a copy of that layout (phase-1-bolts.md, Bolt 2,
the laptop).

## What Changes

- **The host binds its seams in one file beside sluice's.** The
  operator's bindings live in `~/.config/tailrace/config.toml` (under
  `$XDG_CONFIG_HOME` where set), next to sluice's own
  `~/.config/sluice/config.toml` and in the same TOML form, readable with
  any file reader (118). It names the layout the host publishes into, the
  landing it reads (sluice, or a plain branch), and the host key it signs
  with, in sluice's own `[signing]` table shape (92, C.2, C.5, C.6, C.8).
- **The commands are bound to it.** `tailrace publish` and `tailrace
  verify` read the file and bind the seams it names; with no file they
  answer absent naming it, and with a broken one unreadable with the
  parser's words (58, 59). `tailrace status` still needs no seam and no
  file: it answers from the host's record, which names the layout each
  version went into (60). verify and status write nothing; only publish
  makes the layout, the first time it is used.
- **The host's record names the layout.** Each version publish records
  names the store it went into as the host's bindings file names it, the
  layout's path, in place of the word `bound` the core has written since
  nothing was bound; two layouts on one host are two names in status's
  answer (61, 118).
- **The repository binds the build and the projections, not the host.**
  moon is bound as the build in every repository, the tasks each output
  comes from named in the repository's policy file as today, and each
  output's projection is the one the policy file names for it (92, C.3).
- **The policy file names the signers and what each audience requires.**
  A profile of the policy seam reads `.config/tailrace.toml` at the
  landed revision: each signer with the key that verifies it and the
  kinds of claim and checks it may make, and the claims acceptance, a
  layout or a channel requires, in order (82, 83, 116, C.10). The signer
  table takes sluice's `[signers.<name>]` shape. Without this profile no
  verify on any machine could name a signer.
- **The file and image projections pack.** A file output is packed as
  its own bytes; an image output, an OCI image archive as `docker save`
  writes one, is packed as those bytes. Neither puts the version in the
  bytes, so the same output packs to the same bytes under any version,
  and both promise the same bytes again (84, 87, 24, C.11). Their render
  and serve are not built and answer absent.
- **A host that holds no key can still verify.** The host-key profile
  gains a verify-only form, so a second machine whose bindings name no
  key verifies by the keys the policy lists and signs nothing (79, 80).
- **The proofs.** S1 and S2 played by the built binary against a scratch
  repository gated by real sluice and moon, then a second clone with a
  copy of the layout verifying it, in the tools layer. A second machine
  is, to tailrace, a remote carrying sluice's refs, a clone of it, a
  copied layout and a config directory of its own; the tools layer gives
  it exactly those, so the end-to-end test is the unit's proof. A
  conformance scenario cites 118, which leaves the register; the policy
  file profile passes the policy group; the file and image projections
  pass the projection group's pack scenarios.

Out of scope, and what each waits on:

- Render and serve for the file and image projections: render under a
  policy is bolt 5's, and these two ecosystems' layouts are phase 2's.
  Until then they fail the projection group's render scenario, a stated
  departure from core.33 (see the design).
- Counting sluice's carried verdicts at verify: no signer profile
  verifies sluice's own run-record signatures yet. verify reports each
  carried verdict as carried, with its signer as far as a bound profile
  can tell. Which profile verifies them is an open question already with
  the design agent (the host-key signer's open questions).
- A media type declared for a file output (C.11): an artifact's metadata
  has no field for it. It waits on a change to the shared artifact type,
  a unit of its own.
- The ledger binding: the host's file names none until bolt 4's ledger
  unit; a binding holds a ledger that answers absent.
- Channels, the gate and promotion (bolt 4); a registry store (bolt 3);
  a second build profile (C.4, phase 2).
- The run against a family repository and the installed binary: the
  bolt's acceptance, done by hand outside this change. Today it can be
  run on a family repository with the second machine's bindings naming
  `[landing] branch` (landed by history, no verdicts, C.6); in the
  plan's full form once the family's remotes carry sluice's records,
  which waits on sluice's peers work and each host's signing key (sluice
  C.6, 78), since no family policy names signers yet. That run is also
  what proves the install: no gate task builds the flake's package. The
  forge's CI builds it and runs `--version`, which proves the package
  builds and nothing of publish or verify; a flake build in the tools
  layer would rebuild the workspace from scratch in nix's sandbox on
  every change, minutes in a layer meant to take under one.

## Capabilities

### New Capabilities

- `host-bindings`: the host's bindings file: where it is, its form, what
  it binds, how publish and verify are bound to it, and the laptop's
  publish and a second machine's verify end to end.
- `policy-file`: the policy seam over `.config/tailrace.toml` at a landed
  revision: signers, what each may claim, and the claims each audience
  requires.
- `file-projection`: the projection that packs a file output as its own
  bytes.
- `image-projection`: the projection that packs an OCI image archive
  output as its own bytes.

### Modified Capabilities

- `command-line`: an unbound host is now one with no bindings file, and
  publish and verify are bound to the file where there is one.
- `host-key-signer`: the profile can be bound to verify only, with no key
  of its own.
- `host-record`: an entry names the store as the host's bindings name
  it.

## Impact

- `crates/tailrace/src/bindings/`: new, the file's reader and the
  binding it makes.
- `crates/tailrace/src/profiles/policy/file/`,
  `crates/tailrace/src/profiles/projection/file/`,
  `crates/tailrace/src/profiles/projection/image/`: new profiles.
- `crates/tailrace/src/lib.rs`, `main.rs` and `commands/`: the binary
  loads the bindings and the commands take the binding's answer.
- `crates/tailrace/src/profiles/signer/host_key/mod.rs`: one constructor.
- `crates/tailrace-core/src/publish.rs`: the ask carries the store's
  name, and the entry and the answer name it; the core's tests and the
  conformance world give their asks one.
- `crates/tailrace-conformance/`: a host group with one scenario citing
  118; 118 leaves the register.
- `docs/adr/0013`: the host's bindings file and the policy seam's one
  profile (core.3). The site's integration pages and self-hosting guide.
- No new crate dependency.

## Touches

```
crates/tailrace/src/bindings/**
crates/tailrace/src/commands/**
crates/tailrace/src/lib.rs
crates/tailrace/src/main.rs
crates/tailrace/src/profiles/mod.rs
crates/tailrace/src/profiles/*/mod.rs
crates/tailrace/src/profiles/policy/**
crates/tailrace/src/profiles/projection/**
crates/tailrace/src/profiles/signer/host_key/mod.rs
crates/tailrace/src/profiles/signer/host_key/tests.rs
crates/tailrace/tests/**
crates/tailrace/moon.yml
crates/tailrace-core/src/publish.rs
crates/tailrace-core/tests/**
crates/tailrace-conformance/src/world.rs
crates/tailrace-conformance/src/lib.rs
crates/tailrace-conformance/src/seams/**
crates/tailrace-conformance/src/register.rs
docs/adr/**
site/src/content/docs/**
```
