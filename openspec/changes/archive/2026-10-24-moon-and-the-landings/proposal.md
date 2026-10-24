# Proposal

## Why

Bolt 2 is usable when `tailrace publish`, after a sluice landing on a
family repository, writes a version a second machine can verify
(phase-1-bolts.md, Bolt 2). Today the build and the landing exist only
as fakes, so nothing tells tailrace what a real landing landed, which
verdicts it signed, or which build left the outputs. This change binds
the build seam to moon and the landing seam to sluice's records and to
a plain git branch, the profiles C.3, C.5 and C.6 name, joined by moon's
task hash, which sluice's verdicts carry and tailrace's provenance names
(13, 15; ADR 0005).

## What Changes

- **moon is a build.** A profile under
  `crates/tailrace/src/profiles/build/moon/` implements the build seam
  (B.2) and nothing of the core (core.2). `ensure` is one moon run of
  the tasks the declared outputs name, in a checkout of exactly that
  revision, which replays what moon has cached (74, C.3). `report`
  reads moon's own record of that run: the task's hash is the build
  identity, its status the outcome, and the task's declared outputs
  where the files are (75, 76). The repository names, in its policy
  file at the revision, the moon task that makes each output.
- **sluice is a landing.** A profile under
  `crates/tailrace/src/profiles/landing/sluice/` implements the landing
  seam (B.3) over the records sluice already keeps, and asks sluice to
  do nothing (ADR 0005). A revision is landed when the branch's log
  holds its landing, read from the remote's `refs/sluice/state/<branch>`
  or, for a branch sluice keeps on one host, from the host's own files
  (77, C.5). `verdicts-of` reads the signed run records under every
  `refs/sluice/runs/<signer>` for the combination that landed the
  revision: one verdict per result, its check the task, its build
  identity moon's hash, its outcome, its signer the name sluice gives
  it, and the whole envelope as sluice wrote it (78, 104, C.5). A
  result sluice marks replayed is carried as the earlier signed pass for
  the same key and system, from the combination that ran it (C.5;
  sluice 108).
- **A plain branch is a landing.** A profile under
  `crates/tailrace/src/profiles/landing/branch/` answers landed when the
  branch's history holds the revision, and an empty set of verdicts for
  every revision, which is neither absent nor unreadable (77, 78, C.6).
- **A carried envelope is carried whole.** The seams' envelope holds
  every signature of the envelope it carries, each with its key id, as
  the landing wrote it; tailrace's own claims still hold exactly one
  signature (7, 12; C.5). Each verdict says where the landing found
  it, and the provenance that carries it says so too (12). This is the
  design agent's ruling for this unit.
- **An output no verdict names is refused naming both, against the real
  thing.** The core already refuses it (15); this change proves it with
  moon and sluice: an output whose task the landing's gate never
  checked is refused naming the output and every identity the verdicts
  carry, and nothing is offered.
- **The suite runs over the real profiles.** The build group runs over
  moon and the landing group over sluice and over a plain branch, each
  in a scratch repository. The landing group gains one scenario below
  its marker, two signers each read back by its own name, which takes
  104 out of the register (core.33).
- **A tools layer.** The tests that need moon and sluice run in a new
  `check-tools` layer the gate requires on `main` and every bolt, as
  sluice's own gate does. devenv pins sluice's release binary and
  OpenSSH beside moon (core.26). S13 runs there too: a revision on a
  branch sluice does not run, built by moon, published with no verdict;
  under a policy requiring verdicts none counts, under one requiring
  provenance alone the provenance counts.
- **The site says moon, sluice and the plain branch are built.**

Out of scope, and what each waits on:

- Binding these profiles for a host, in the host's bindings file, and
  `tailrace publish` over them from the installed binary: the laptop
  unit of bolt 2.
- Verifying the signature of a sluice verdict, an SSH signature over
  the envelope: the host-key signer unit of bolt 2. This change carries
  the envelope; it never verifies or re-signs it (core.7, core.8).
- moon's outputs directory shared by every checkout, and filling it from
  the store (C.3's cache): the moon's cache unit of bolt 3. A build here
  replays only what its own checkout has cached.
- The gate's refusal and acceptance in S13: bolt 4's gate unit. S13 here
  stops at verify's answer, as the suite's S13 does.
- The declared-command build (C.4): no unit of phase 1.

## Capabilities

### New Capabilities

- `moon-build`: the build seam bound to moon: which task makes an
  output, ensure as one run in a checkout of the revision, and report
  from moon's own record with the task hash as the build identity.
- `sluice-landing`: the landing seam bound to sluice's records: landed
  from the branch's log, verdicts from the signed run records of the
  combination that landed the revision, replays carried as the pass
  that ran, every envelope whole.
- `branch-landing`: the landing seam bound to a plain git branch:
  landed by the branch's history, and no verdicts.
- `tools-layer`: the gate's layer for tests against real tools, and the
  tools it pins.

### Modified Capabilities

- `seam-contracts`: a signed envelope holds exactly one signature when
  it is tailrace's own claim and every signature, with its key id, when
  it is carried; a verdict says where the landing found it.
- `publishing`: a carried verdict says where the landing found it, in
  the landing's own words.
- `conformance-suite`: the landing group holds a landing to name each
  verdict's signer by the landing's own name for it (104).

## Impact

- `crates/tailrace/src/profiles/build/moon/`,
  `profiles/landing/sluice/`, `profiles/landing/branch/`: the three
  profiles, new; `profiles/mod.rs`, `profiles/build/mod.rs`,
  `profiles/landing/mod.rs` and `crates/tailrace/src/lib.rs` for their
  module paths, beside the layout store's.
- `crates/tailrace-seams/src/claim.rs`, `landing.rs`, `signer.rs`:
  `Envelope` holds a list of signatures, `Signature` a key id, `Verdict`
  where it was found. Every crate that builds an envelope or a verdict
  follows: the fakes, the core's provenance and its tests, the
  conformance suite's rigs and world.
- `crates/tailrace-conformance/src/seams/landing.rs` (one scenario below
  the marker) and `src/register.rs` (104 out).
- `crates/tailrace/tests/`: the rigs and the profiles' own tests, a
  scratch-repository helper, and the end-to-end tests over moon and
  sluice.
- `crates/tailrace/Cargo.toml`: a `tools` feature; `base64` for DSSE's
  encoding, a new workspace dependency in `Cargo.toml` and `Cargo.lock`.
- `crates/tailrace/moon.yml`: a `tools` task tagged `check-tools`; the
  root `moon.yml`'s lint, and the commit hook's, lint the tools tests
  too.
  `.config/sluice.toml` requires `#check-tools` on `main` and `bolt/*`;
  `.moon/workspace.yml` and `CLAUDE.md` name the layer.
- `devenv.nix`: sluice's release binary and OpenSSH.
- `site/src/content/docs/guides/integrations/builds.mdx` and
  `landings.mdx`.

## Touches

```
Cargo.toml
Cargo.lock
CLAUDE.md
devenv.nix
moon.yml
.config/sluice.toml
.moon/workspace.yml
crates/tailrace-seams/src/claim.rs
crates/tailrace-seams/src/landing.rs
crates/tailrace-seams/src/signer.rs
crates/tailrace-seams/src/lib.rs
crates/tailrace-fakes/src/**
crates/tailrace-core/src/provenance.rs
crates/tailrace-core/src/publish.rs
crates/tailrace-core/tests/**
crates/tailrace-conformance/src/**
crates/tailrace-conformance/tests/**
crates/tailrace/Cargo.toml
crates/tailrace/moon.yml
crates/tailrace/src/lib.rs
crates/tailrace/src/profiles/mod.rs
crates/tailrace/src/profiles/run.rs
crates/tailrace/src/profiles/build/**
crates/tailrace/src/profiles/landing/**
crates/tailrace/src/profiles/store/layout/mod.rs
crates/tailrace/tests/scratch/**
crates/tailrace/tests/build_moon.rs
crates/tailrace/tests/landing_sluice.rs
crates/tailrace/tests/landing_branch.rs
crates/tailrace/tests/landings_end_to_end.rs
crates/tailrace/tests/layout_store.rs
site/src/content/docs/guides/integrations/builds.mdx
site/src/content/docs/guides/integrations/index.mdx
site/src/content/docs/guides/integrations/landings.mdx
```
