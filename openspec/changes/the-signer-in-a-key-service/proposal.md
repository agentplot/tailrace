# Proposal

## Why

Bolt 4 is usable when a family repository's `beta` moves on every
landing and `stable` moves when a person promotes, with the chain held
in ECR and in a git ref (phase-1-bolts.md, Bolt 4). Today the only
signer that proves who signed is a host key, a file on a disk, and the
operator ruled that the offering signer on a build fleet is a key in a
key service used through the runner's role (rulings, 2026-10-24). This
change binds the signer seam (B.4) to a key in AWS KMS that the host
reaches through its own role, may use and can never export (C.7), so a
host signs claims with nothing on its disk to steal, and any machine
the key's owner lets read the key tells who made a claim, and whether
that key has since been switched off.

## What Changes

- **A key in a key service is a signer.** A new profile under
  `crates/tailrace/src/profiles/signer/kms/` implements the one signer
  interface and nothing of the core (core.2). It is bound with one
  thing: the key's ARN. The signer's name is the key's ARN (C.7), and
  the key's region comes from the ARN.
- **sign signs through the host's own role.** At each signature the
  profile reaches the host's AWS credentials the way the AWS tools do
  (environment, profile and SSO session, the instance's or the task's
  role), asks KMS to sign, and keeps nothing afterwards (64, core.21).
  The private half never leaves KMS. Asked to sign as any other
  identity, with no credentials at hand, or with a key the role may not
  use, sign answers that no identity is at hand, naming the identity
  and carrying KMS's or the SDK's own words (79, 11, core.8). A key of
  the wrong kind, or one whose key material was imported from outside
  KMS, and so was exported once, is refused, saying which. A service
  that cannot be reached is unreadable (58, 59).
- **verify uses the key's public half.** For each signer the policy
  names by a KMS key's ARN, verify reads the key's public half from KMS
  and checks the signature on this machine against it (80, C.7). The
  name the envelope gives its signer decides nothing (core.7). An
  envelope no KMS signature is in, such as a host key's, is refused
  without reaching KMS, so the core goes on to the next signer bound.
- **A switched-off key is reported expired.** The key's standing is its
  state in KMS. A signature by a key that has since been disabled is
  answered as made by its signer and expired, never as valid and never
  as absent (81). Once a key is scheduled for deletion KMS gives out
  its public half no more, so verify answers unreadable, naming the
  key and carrying KMS's words: it can no longer tell whether that key
  made the signature, and says so.
- **The host binds a KMS key in its bindings file.** `[signing]` takes
  `kms = "<key ARN>"` in place of `signer` and `key` (92, 118). Every
  host binds the KMS profile as well, verify-only where its file names
  no KMS key, so a laptop with a host key still verifies a fleet's
  claims. The reading of `[signing]` moves into the signer profiles'
  own module, so the bindings module names no key service (core.1).
- **The signer's conformance scenarios pass against a key in a test
  account, outside the gate.** The suite's signer group runs against a
  real KMS key named in the host's `devenv.local.nix`, by hand with a
  live AWS session, under a tag of its own, `check-kms`, as the ECR run
  does (core.38). The same group runs on every landing against an
  in-process stand-in for KMS's API under the real SDK client, so the
  gate proves the profile's reading of every answer KMS gives without
  holding a credential.
- **A decision record and the site.** ADR 0016 records what key the
  profile takes, how it signs, and what it reads as a key's standing,
  for the user to accept with the change. The signers guide marks the
  key service built and says how to make the key, what the host's role
  needs, and what the policy lists; the self-hosting guide shows
  `[signing] kms`.

Out of scope, and what each waits on:

- Verifying a claim after its key's deletion is scheduled or done:
  this needs the policy to carry the key's public half beside its name,
  which the design does not yet say; it waits on madswan-design (see
  the design's open questions).
- The gate signing its acceptance and the ledger signing promotions
  with a KMS key: the gate unit and the ledger and promotion unit of
  bolt 4, which sign through whatever signer the host binds.
- Making the key or granting the role: the operator does both in AWS;
  tailrace adds no command for either.
- Keys in other clouds' key services, and keyless signing (C.9):
  phase 2.

## Capabilities

### New Capabilities

- `key-service-signer`: the signer seam bound to a key in AWS KMS: how
  it signs through the host's role, how verify reads the key's public
  half, how a disabled key is reported expired, and how each of KMS's
  answers is told apart as absent, refused or unreadable.

### Modified Capabilities

- `host-bindings`: `[signing]` may name a KMS key by its ARN in place of
  a host key's signer and path, and every host binds the KMS profile
  for verify.
- `tools-layer`: a `check-kms` check, outside the gate, runs the signer
  group against a KMS key the host names, and the gate's tools layer
  stays free of any credential.

## Impact

- `crates/tailrace/src/profiles/signer/kms/`: the profile, new.
- `crates/tailrace/src/profiles/signer/mod.rs` and a new `tests.rs`
  beside it: the reading of `[signing]` and the binding of both signer
  profiles.
- `crates/tailrace/src/bindings/mod.rs` and `tests.rs`: `[signing]`
  handed whole to the signer profiles.
- `crates/tailrace/tests/kms/` (the stand-in for KMS's API),
  `crates/tailrace/tests/kms_signer.rs` (the group over the stand-in,
  in `check-fast`) and `crates/tailrace/tests/kms_account.rs` (the group
  against the test account, `check-kms`).
- `crates/tailrace/moon.yml`: the `kms` task under `check-kms`.
- New dependencies: `aws-config`, `aws-sdk-kms`, `tokio` (a
  current-thread runtime to drive the SDK) and `p256`; dev-only, the
  SDK's HTTP test client. Lines in the workspace `Cargo.toml` and in
  `crates/tailrace/Cargo.toml`, and `Cargo.lock`. The flake builds from
  `Cargo.lock` and needs no hash change.
- `docs/adr/0016-…md`, `site/src/content/docs/guides/integrations/signers.mdx`,
  `site/src/content/docs/guides/self-hosting.mdx`.
- No trait, shared type, fake, conformance scenario or core module
  changes.

## Touches

```
crates/tailrace/src/profiles/signer/mod.rs
crates/tailrace/src/profiles/signer/tests.rs
crates/tailrace/src/profiles/signer/kms/**
crates/tailrace/src/bindings/mod.rs
crates/tailrace/src/bindings/tests.rs
crates/tailrace/tests/kms/**
crates/tailrace/tests/kms_signer.rs
crates/tailrace/tests/kms_account.rs
crates/tailrace/Cargo.toml
crates/tailrace/moon.yml
Cargo.toml
Cargo.lock
docs/adr/0016-*.md
site/src/content/docs/guides/integrations/signers.mdx
site/src/content/docs/guides/self-hosting.mdx
```
