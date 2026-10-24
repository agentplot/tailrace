# Proposal

## Why

Bolt 2 is usable when `tailrace publish` on a laptop writes a signed
version into a layout on disk and `tailrace verify` on a second machine
says every claim holds (phase-1-bolts.md, Bolt 2). Today the only signer
is the fake, whose signature is a digest anyone can make, so nothing
tailrace signs proves who signed it. This change binds the signer seam
(B.4) to a key file on the host, held the way sluice's signers hold
theirs (C.8), so a publish signs as the host that ran it and any machine
that reads the policy can tell which listed host made a claim, with no
service and no credential kept by tailrace.

## What Changes

- **A key file on the host is a signer.** A new profile under
  `crates/tailrace/src/profiles/signer/host_key/` implements the one
  signer interface and nothing of the core (core.2). It is bound with the
  signer's name and the path of an SSH private key file, the form sluice
  signs its run records with: an Ed25519 key in OpenSSH's format, with no
  passphrase, that only its owner may read (C.8; sluice ADR 0032).
- **sign signs as the host and only as the host.** sign reads the key
  file at the moment it signs, signs the envelope's DSSE
  pre-authentication encoding with an SSH signature in tailrace's own
  namespace, and keeps nothing of the key afterwards (79, 64, core.21).
  Asked to sign as any identity but its own, or with the key file gone,
  it answers that no identity is at hand, naming the identity (79, 11,
  core.8). A key others may read, or one with a passphrase, is refused
  with sluice's words for each; a key file that cannot be read or parsed
  is unreadable, carrying the operating system's or the parser's words
  (58, 59, core.19).
- **verify tells a claim by the public key the policy lists.** verify
  reads no key file and reaches nothing: it checks the signature against
  each signer the policy names whose verifier is an SSH public key, and
  answers the first whose key made a valid signature over exactly that
  envelope (80, C.8). The envelope's own word for who signed it decides
  nothing (core.7).
- **A claim from a signer the policy does not name is refused naming the
  signer.** When no listed key made the signature, verify refuses,
  naming the signers it checked and, in its words, the key that did make
  the signature by its fingerprint, so the gate and `tailrace verify` can
  say whose claim was found wanting (S5, 9, 80). An envelope another
  profile signed, or a signature in any other namespace, is refused the
  same way, never unreadable, so the core goes on to the next signer
  bound.
- **An expired listing is reported expired.** A listed key may carry
  OpenSSH's own `valid-before` option; once that time has passed, a
  signature by the key is answered as made by its signer and expired,
  never as valid and never as absent (81).
- **The signer's conformance scenarios pass against a key in a temporary
  directory.** The profile's own tests run the suite's signer group over
  a fresh key per scenario, and test what the group does not: the key
  read anew at each signing, nothing of it in the envelope or on disk
  afterwards, the key's mode and passphrase refused, and S5's signer
  half with a second host's key.
- **The suite asks every signer to keep no credential.** One scenario is
  added below the signer group's marker: after a signature, the
  identity's credential taken away, the next sign answers no identity at
  hand. It cites 64, and 64 leaves the register in the same commit
  (core.33).
- **The site says the host key is built.** The signers page marks the
  host key built and says what key it takes and what the policy lists to
  verify it.

Out of scope, and what each waits on:

- Naming the key file and the host's signer in the host's bindings file,
  expanding `~` in its path, and `tailrace publish` signing through this
  profile: the laptop unit of bolt 2.
- How `.config/tailrace.toml` lists a signer and its public key, which
  the policy file profile reads into what this profile is handed (C.10):
  no unit of bolt 2 names it (see the design's open questions).
- Verifying sluice's own run records, signed in sluice's namespace, when
  a provenance carries them as verdicts: the moon-and-the-landings unit
  reads them, and whether this profile verifies them waits on a design
  decision (see the design's open questions).
- The key service signer: bolt 4. Keyless signing: phase 2.
- Making a key: the operator makes one with `ssh-keygen`; tailrace adds
  no command for it in phase 1.
- The signer group itself, its rig interface and the register: the
  conformance suite unit, `the-conformance-suite-over-the-fakes`, whose
  signer group this change runs and adds one scenario to below its
  marker. This change's code needs that unit's work in its place first.

## Capabilities

### New Capabilities

- `host-key-signer`: the signer seam bound to an SSH key file on the
  host: how it signs as the host's identity, how it tells which listed
  signer made an attestation, how an expired listing is reported, and
  how a key it cannot use is told apart from no identity at hand.

### Modified Capabilities

- `conformance-suite`: the signer group gains a scenario that no
  credential is kept from one signature to the next (64). The capability
  is added by `the-conformance-suite-over-the-fakes`, ahead of this
  change.

## Impact

- `crates/tailrace/src/profiles/signer/host_key/`: the profile, new.
- `crates/tailrace/src/profiles/signer/mod.rs`, one `pub mod signer;`
  line in `crates/tailrace/src/profiles/mod.rs`, and the `pub mod
  profiles;` line in `crates/tailrace/src/lib.rs` where the layout
  store's unit has not added it yet.
- `crates/tailrace/tests/host_key_signer.rs`: the signer group over the
  profile, and the profile's own tests.
- `crates/tailrace-conformance/src/seams/signer.rs`, below the marker,
  and one line out of `crates/tailrace-conformance/src/register.rs`.
- One dependency, `ssh-key` 0.6, the crate sluice signs with: a line in
  the workspace `Cargo.toml`, a line in `crates/tailrace/Cargo.toml`,
  and `Cargo.lock`.
- `site/src/content/docs/guides/integrations/signers.mdx`.
- No trait, shared type, fake or core module changes.

## Touches

```
crates/tailrace/src/lib.rs
crates/tailrace/src/profiles/mod.rs
crates/tailrace/src/profiles/signer/mod.rs
crates/tailrace/src/profiles/signer/host_key/**
crates/tailrace/tests/host_key_signer.rs
crates/tailrace/Cargo.toml
crates/tailrace-conformance/src/seams/signer.rs
crates/tailrace-conformance/src/register.rs
Cargo.toml
Cargo.lock
site/src/content/docs/guides/integrations/signers.mdx
```
