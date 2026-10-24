# Proposal

## Why

Bolt 3 is usable when a task that passed on one host replays from ECR
on another (phase-1-bolts.md, Bolt 3). Today the only store a host can
bind is an OCI layout on disk, so nothing one host publishes reaches
another except by copying a directory. This change binds the store seam
(B.1) to an OCI registry, the profile C.1 names, proves it against zot
and against a real ECR repository, and lets a laptop copy what its
layout holds into a registry with every attestation going with it. The
cache and the push after a landing, the two other units of the bolt,
write into the store this change binds.

## What Changes

- **A registry is a store.** A new profile under
  `crates/tailrace/src/profiles/store/registry/` implements the one
  store interface over the OCI distribution protocol: bytes as blobs, an
  artifact or an index as a manifest put by its own digest, a name as a
  tag (C.1, ADR 0003). It implements the interface and nothing of the
  core (core.2), and it is the only code that names a registry (core.1).
- **One declared repository holds everything.** The host names one
  repository, such as `<account>.dkr.ecr.<region>.amazonaws.com/tailrace`
  or `localhost:5000/tailrace`, and every blob, manifest, attestation
  and tag of the store is held there, addressed by digest. A name is
  escaped into a valid tag. This is the mode C.1 gives a registry that
  does not create repositories on push; this change uses it on every
  registry, including zot, because the store's operations name only a
  digest and a digest alone cannot say which per-name repository holds
  it (design D2).
- **The same bytes put twice are held once.** A put asks the registry
  first whether it holds the blob or manifest, and writes nothing when
  it does, answering held already (5, 66). A manifest is written in
  exactly the bytes the layout writes, from one shared module, so an
  artifact has the same digest in a layout and in a registry (1).
- **Attestations are referrers, with the tag-schema fallback.** attach
  pushes a manifest whose subject is the digest and whose one layer is
  the envelope; claims-of lists the subject's referrers of that type
  and reads each envelope back, byte for byte (8, 68, 69). Where the
  registry has no referrers API, both use the referrers tag the OCI
  distribution specification defines in its place.
- **A tag is moved from where it is expected, and read back.** No
  registry this change binds can refuse a tag put by the digest it is
  expected to be at, so name reads the tag first and refuses a move
  from anywhere else, naming where it is; after the put it reads the
  tag back, and a tag found at another digest is a lost race, refused
  naming that digest, never retried (70, 71, 109, core.6).
- **A lost race on a channel's name is repaired from the ledger.** A
  new core function moves a channel's name to the digest the ledger's
  head promotion is about, and when the store answers that the name is
  elsewhere, it reports the lost race and moves the name back once, from
  where it was found, to what the ledger says (109). The gate of bolt 4
  calls it; this change proves it over the fakes and against zot.
- **A layout can be synced to a registry.** A new core function copies
  a version from one store to another through the store seam alone:
  every artifact's bytes, the artifacts, every attestation attached to
  each digest, the index last, and the version's name; a channel is
  never copied (103). Copying a layout into zot answers the same
  digests and the same claims.
- **ECR's open issues are answered by a push against a real
  repository.** The store's conformance scenarios run against a real
  ECR repository in the tools layer, and an ADR records what ECR
  answered on each open issue (aws/containers-roadmap 2306 and 2783):
  whether it takes tailrace's manifests, a referrer with a subject, the
  referrers list filtered by type, and a manifest put by digest alone.
- **The store's conformance scenarios pass against ECR and zot.** The
  profile's tests run the suite's store group against zot, against the
  reference distribution (a registry with no referrers API, for the
  fallback), and against ECR, all in the tools layer, which the gate
  already requires; zot, the reference distribution and ECR's
  credential helper are pinned in `devenv.nix`.
- **Credentials are the host's.** The profile reaches a registry's
  credential at the moment of use through the host's Docker
  configuration and its credential helpers, ECR's among them, and keeps
  none (64, core.21). The bindings file names a repository, never a
  credential.
- **The host binds it.** `[store] registry = "<repository>"` in the
  host's bindings file binds the profile, beside `[store] layout`.
- **The register and the site.** Clauses 103 and 109 leave the
  register in the commits whose scenarios cite them. The stores page
  marks ECR, zot and the reference distribution built and says how the
  registry holds what it holds; the self-hosting page shows the
  registry binding.

Out of scope, and what each waits on:

- A command that syncs a layout to a registry: the copy is in the core
  and proven in the tests, and the command line belongs to the push
  after a landing, which owns `crates/tailrace/src/commands/` in this
  bolt, or to a later unit the planner names.
- A repository per ecosystem name on a registry that creates them on
  push: a design decision C.1 leaves open, named for the design agent.
- The gate itself, promotion, the ledger's profiles and the chain's
  form: bolt 4. The repair here reads the chain through the ledger seam
  and its fakes, and writes nothing to it.
- Cache entries in the registry: the cache unit of this bolt, which
  writes through the store this change binds.
- GitHub's registry and Harbor: the profile speaks the protocol they
  speak, and they stay designed until something runs the suite against
  them.
- Removing anything from a registry, and lifecycle rules: no unit of
  phase 1. The ADR names the rule an operator must not set (expiring
  untagged manifests).

## Capabilities

### New Capabilities

- `registry-store`: the store seam bound to an OCI registry over the
  distribution protocol: where blobs, manifests, attestations and names
  are held in one declared repository, what each operation sends and
  answers, the referrers fallback, the tag read back, the host's
  credentials, and the runs against zot, the reference distribution and
  ECR.
- `store-copy`: a version copied from one store to another through the
  store seam, with every attestation attached to its digests, and a
  layout synced to a registry.
- `name-repair`: a channel's name moved in the store from the ledger's
  head, and a lost race reported and repaired from the ledger.

### Modified Capabilities

- `host-bindings`: `[store]` names a registry by its repository as well
  as a layout by its path.
- `tools-layer`: the layer runs the registry store against zot, the
  reference distribution and ECR, each pinned or named on the host.

## Impact

- `crates/tailrace/src/profiles/store/registry/`: the profile, new.
- `crates/tailrace/src/profiles/store/oci.rs`: the OCI forms, moved up
  from the layout's directory so both profiles write one form;
  `profiles/store/layout/` changes only its imports.
- `crates/tailrace/src/profiles/store/mod.rs`: the `registry` key.
- `crates/tailrace-core/src/copy.rs` and `src/repair.rs`, new, and two
  lines of `src/lib.rs`.
- `crates/tailrace-conformance/`: a `copy` group, one scenario below
  the ledger group's marker, and two register entries removed.
- `crates/tailrace/tests/registry_store.rs`, `registry_ecr.rs` and the
  rig they share; `crates/tailrace/moon.yml`'s tools task and a
  `tools-ecr` task.
- `Cargo.toml` and `crates/tailrace/Cargo.toml`: `ureq` for HTTP and
  `docker_credential` for the host's Docker credentials; `Cargo.lock`.
- `devenv.nix`: zot, the reference distribution and
  `amazon-ecr-credential-helper`. The ECR repository the tools layer
  pushes to is named per host in `devenv.local.nix`, which git ignores.
- `docs/adr/`: one record, the registry store and what ECR answered.
- The site's stores and self-hosting pages.
- No seam trait or shared type changes.

## Touches

```
Cargo.toml
Cargo.lock
devenv.nix
devenv.lock
docs/adr/*
crates/tailrace/Cargo.toml
crates/tailrace/moon.yml
crates/tailrace/src/bindings/mod.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/src/profiles/store/**
crates/tailrace/tests/registry_store.rs
crates/tailrace/tests/registry_ecr.rs
crates/tailrace/tests/registry/**
crates/tailrace-core/src/lib.rs
crates/tailrace-core/src/copy.rs
crates/tailrace-core/src/repair.rs
crates/tailrace-conformance/src/lib.rs
crates/tailrace-conformance/src/register.rs
crates/tailrace-conformance/src/core/mod.rs
crates/tailrace-conformance/src/core/copy.rs
crates/tailrace-conformance/src/core/ledger.rs
site/src/content/docs/guides/integrations/stores.mdx
site/src/content/docs/guides/self-hosting.mdx
```
