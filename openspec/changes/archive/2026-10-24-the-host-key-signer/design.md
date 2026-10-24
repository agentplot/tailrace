# Design

## Context

The signer seam is one trait, `tailrace_seams::Signer`, with `sign(claim,
identity) -> Answer<Envelope>` and `verify(envelope, signers) ->
Answer<Made>` (B.4, 79 to 81). An `Envelope` is tailrace's own DSSE
shape with exactly one signature: the payload type, the payload (the
claim as an in-toto statement, `Claim::payload`), and a `Signature` of
the signer's name and opaque `sig` bytes. A `NamedSigner` is a signer as
the policy names it: its id, `verified_by`, "a text its signer profile
reads, a public key or a key's name", and what it may claim. `Made` is
the signer named and whether it has expired. The core asks every signer
bound in turn and goes on to the next on a refusal; any other `Not` ends
verify (`tailrace-core/src/verify.rs`, `tell`). The fake signer
(`tailrace-fakes/src/signer.rs`) answers absent `identity <id>` for an
identity it does not hold and refused `Why::NoNamedSigner { checked }`
when none of the signers given made the envelope.

C.8 says only: a key file on the host, as sluice's signers hold theirs;
verification uses the public key the policy lists. sluice's way, on its
main today (`crates/sluice/src/signing/ssh.rs`, `instance.rs`; sluice ADR
0032), is:

- the host's config names `[signing] signer` and `key`, a path with `~`
  expanded, refused when its mode lets anyone but its owner read it,
  with the words `which lets others read it: give it mode 0600 (chmod 600
  <path>)`;
- the key is an OpenSSH private key read with `ssh-key` 0.6
  (`PrivateKey::read_openssh_file`), refused when encrypted ("sluice
  signs with none to type"), Ed25519 the one algorithm built in;
- a signature is `SSHSIG` over the DSSE pre-authentication encoding
  `PAE(type, payload)`, hash SHA-512, in the namespace `sluice-run`,
  written as the armored PEM text;
- verification checks each allowed-signers line's public key against
  the signature's own public key and then the signature, in that
  namespace.

The layout store's unit adds `crates/tailrace/src/profiles/mod.rs` and
the `pub mod profiles;` line; this unit adds the signer's module beside
the store's. The suite's signer group, its `SignerRig` and the register
come from `the-conformance-suite-over-the-fakes`; the group's six
scenarios and the rig's methods are as that unit's
`crates/tailrace-conformance/src/seams/signer.rs` has them.

## Goals / Non-Goals

**Goals:**

- A signer whose signature only the holder of the key file can make and
  any machine can check from the policy alone, in the form sluice's
  signers already use, so an operator's host key is the same kind of
  thing in both tools (ADR 0005's consequences).
- The key reached at the moment of use and nowhere else (64, core.21).
- Every answer one of the seam's: an envelope or `Made`, absent,
  refused, unreadable, with the source's own words (58, 59).

**Non-Goals:**

- Reading the host's bindings file or `.config/tailrace.toml`: the
  profile is handed its signer name and key path, and verify is handed
  the `NamedSigner`s.
- Verifying sluice's own run records (namespace `sluice-run`).
- Making keys, rotating them, or a key agent.

## Decisions

### D1. The profile is a name and a path, and nothing else

`HostKey { signer: SignerId, key: PathBuf }`, constructed by
`HostKey::new(signer, key)`. It holds no key material and caches nothing:
sign reads the file each time and drops the parsed key before it
returns. The source every answer names is `Source::new(Seam::Signer,
"host key")`. `~` is not expanded here; the path is the bindings'
to resolve, as sluice resolves it when it reads its config (the laptop
unit).

*Alternative:* read the key once at construction, as sluice does when it
reads its config. Rejected: 64 asks the credential be reached at the
moment of use, and a key kept in a long-lived binding is a credential
kept.

### D2. sign follows sluice's order of checks

1. `identity != self.signer` → absent `identity <identity>`, no file
   touched (79, 11).
2. `metadata(key)`: not found → absent `identity <signer>: no key file at
   <path>`; any other error → unreadable, what `the key file <path>`,
   said the operating system's words.
3. A path that is not a regular file → unreadable as in 2, with the
   words of the read that fails on it.
4. Mode `& 0o077 != 0` → refused `Why::Said`, said `the key file <path>
   has mode <mode:04o>, which lets others read it: give it mode 0600
   (chmod 600 <path>)`.
5. `PrivateKey::read_openssh_file` error → unreadable, said `ssh-key`'s
   words. `is_encrypted()` → refused `Why::Said`, said `the key file
   <path> has a passphrase; tailrace signs with none to type`.
   `algorithm() != Ed25519` → refused `Why::Said`, said naming the
   algorithm and that tailrace signs with Ed25519 keys.
6. `payload = claim.payload()`; `sig = key.sign(NAMESPACE, Sha512,
   &pae(IN_TOTO, &payload))`; envelope `{ payload_type: IN_TOTO,
   payload, signature: { signer: self.signer, sig:
   sig.to_pem(LineEnding::LF) bytes } }`.

`Why::Said` is the seam's refusal that names nothing beyond the source's
words; no new `Why` is needed, and a profile unit changes no shared type.

### D3. tailrace signs in its own namespace, `tailrace-claim`

The SSH signature namespace is `tailrace-claim`, not sluice's
`sluice-run`, so a signature one tool made never verifies as the
other's, as sluice's namespace keeps its runs apart from git's.

*Alternative:* sign in `sluice-run`, so one verifier reads both. Rejected:
a host's run signature would then verify as a tailrace claim of
whatever the payload says. Whether this profile should also verify
sluice's records in `sluice-run` when carried is left open (Open
questions).

### D4. The DSSE encoding is written in the profile

`pae(type, payload) = "DSSEv1 <len(type)> <type> <len(payload)> " ||
payload`, as sluice's `dsse.rs` and the DSSE specification have it,
tested against the specification's own example. It lives in the profile
module; the seams crate is not changed by a profile unit. The key
service signer of bolt 4 needs the same function; moving it into
`tailrace-seams` is a change of its own.

### D5. What the policy lists: an allowed-signers entry without its principal

`verified_by` for a host key is ssh-keygen's allowed-signers form with
the principal left out, since the `NamedSigner`'s id is the principal:
`[options] keytype base64 [comment]`, for example `ssh-ed25519 AAAA…
host@laptop`, or `valid-before="20270101Z" ssh-ed25519 AAAA…`. A
`verified_by` with no word naming an SSH key type (`ssh-`, `ecdsa-`,
`sk-`, sluice's `is_key_type`) is another profile's signer and is passed
over. One that names a key type and then fails to parse, or carries an
option other than `valid-before`, makes verify unreadable naming the
signer: a broken listing of a host key must be loud, never a silent
"not made by".

*Alternative:* a bare OpenSSH public key only. Rejected: it leaves 81
with no way to say when a host key's standing ends.

### D6. Expiry is OpenSSH's `valid-before`, in UTC

A listing's `valid-before` is read in ssh-keygen's forms `YYYYMMDD` or
`YYYYMMDDHHMM[SS]`, and only with the trailing `Z`: ssh-keygen reads a
time without `Z` in the host's own zone, and a policy must answer the
same on every host. Converting to seconds since the epoch is a few lines
of civil-date arithmetic; no time crate is added. `Made.expired` is
`now >= valid_before`, `now` from `SystemTime::now()`. `valid-after` is
not read (a refused option, D5): the statement asks only that an expired
identity be told as expired (81).

### D7. verify by key, in order, refusing what it cannot read

1. Parse every `NamedSigner` per D5, in order; one that is this
   profile's and broken → unreadable naming its id.
2. `sig` bytes as UTF-8 and `SshSig::from_pem`; failing → refused
   `NoNamedSigner { checked: every id given }`, said `the signature is
   not an SSH signature`.
3. `sig.namespace() != NAMESPACE` → refused as in 2, said naming the
   namespace it was made in.
4. The first listed key with `key_data() == sig.public_key()` and
   `verify(NAMESPACE, &pae(type, payload), &sig)` ok → `Made { signer:
   id, expired }`. `attestation.signature.signer` is never compared
   (core.7).
5. None → refused `NoNamedSigner { checked }`, said `signed by the key
   <SHA256 fingerprint>, which the envelope names <name>; no signer
   checked lists that key with a signature over this envelope`.

Refusing rather than answering unreadable in 2 and 3 matters: the core
asks each bound signer in turn and only a refusal lets it go on.

### D8. The dependency is the one sluice uses

`ssh-key = { version = "0.6", features = ["ed25519", "getrandom"] }` in
the workspace's `[workspace.dependencies]`, and `ssh-key.workspace =
true` in `crates/tailrace/Cargo.toml`. The default features (`ecdsa`,
`rand_core`, `std`) stay, as sluice keeps them; `getrandom` lets the
tests make keys with `ssh_key::rand_core::OsRng` with no second crate.
`Cargo.lock` takes what this resolves to; sluice locks 0.6.7. The API
used, as in ssh-key 0.6.7's source: `PrivateKey::{read_openssh_file,
is_encrypted, algorithm, sign, public_key, random, write_openssh_file}`,
`PublicKey::{from_openssh, key_data, verify, fingerprint}`,
`SshSig::{from_pem, to_pem, namespace, public_key}`. `PublicKey::verify`
checks the key and the namespace before the signature.

### D9. The rig and the profile's own tests

`crates/tailrace/tests/host_key_signer.rs` holds a `HostKeyRig`: a
`TempDir`, a fresh Ed25519 key written there with mode 0600, a second
key for the stranger, and the profile bound as `host`.

| rig method | host key |
|---|---|
| `identity` | `host` |
| `named(id)` | `id`, the key's public half, every claim kind; with `valid-before="20000101Z"` once `expire` was called |
| `stranger` | `a stranger`, the second key's public half |
| `no_identity` | removes the key file |
| `expire(id)` | marks `id` expired for `named` |
| `unreachable` | `not_applicable("a host key's verify reads only the public keys the policy lists and reaches nothing; an unreadable key file is the profile's own test")` |

The rig calls `not_applicable` from `unreachable`, which unwinds out of
the scenario the way a scenario's own call does; the report names it
not applicable. The tests then cover each scenario of the
`host-key-signer` spec, among them S5's signer half: a verdict claim
signed by a second host's key, refused naming the checked signer with
the second key's fingerprint in the words.

### D10. The suite's new scenario

Below the signer group's marker: `the credential taken away after a
signature`, `plays: None`, `cites: &[64]`, which signs, calls
`rig.no_identity()`, and asserts the second sign absent naming the
identity, using the module's `absent` helper. The fakes' rig passes it
as written (its `no_identity` holds another identity). The line for 64
leaves `register.rs` in the same commit.

## Risks / Trade-offs

- [The unreachable scenario is not applicable to this profile] → Named
  so in the report with its reason, and sign's unreadable key file is
  tested in the profile's own tests; the suite's owner may split that
  scenario per operation later.
- [The bolt's units land `profiles/mod.rs` and `lib.rs` side by side] →
  Each adds one line; crew forecasts the overlap and settle takes both.
- [The mode check is Unix only] → tailrace's hosts are macOS and Linux,
  as sluice's are; the check uses `std::os::unix`.
- [A policy author writes `valid-before` without `Z`] → Unreadable, with
  words saying the time must end in `Z`, never a silent pass.

## Open Questions

- Should the host-key signer also verify sluice's own run records,
  signed in `sluice-run`, when a provenance carries them as verdicts?
  This change verifies tailrace's namespace only; the landing profile's
  mapping of sluice's records decides whether a carried verdict reaches
  any signer profile in a form it can check. For `madswan-design`.
- How `.config/tailrace.toml` lists a signer, and so what text lands in
  `verified_by` (C.10). This change fixes what a host key's
  `verified_by` holds (D5); the policy file profile that writes it is
  named by no unit of bolt 2.
