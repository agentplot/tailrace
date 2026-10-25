# Design

## Context

The signer seam is one trait, `tailrace_seams::Signer`: `sign(claim,
identity) -> Answer<Envelope>` and `verify(envelope, signers) ->
Answer<Made>` (B.4, 79 to 81). An `Envelope` is tailrace's DSSE shape;
a claim of tailrace's form holds one `Signature { signer, keyid, sig }`.
A `NamedSigner`'s `verified_by` is "a text its signer profile reads, a
public key or a key's name"; the policy file fills it from the one key
of a `[signers.<name>]` table besides `kinds` and `checks` (ADR 0013),
so a policy lists a KMS signer as `[signers.fleet] kms = "<key ARN>"`.
The core asks every signer bound in turn, goes on to the next on a
refusal, and stops on any other `Not` (`tailrace-core/src/verify.rs`,
`tell`); publish signs with the first signer bound
(`tailrace-core/src/publish.rs`). An expired maker is reported and still
counts (81, 117).

The host-key profile (`profiles/signer/host_key/`) is the pattern this
one follows: bound by a name and a reference, verify-only when bound to
nothing, verify refusing an envelope it cannot read so the core asks
the next signer, and the host-key spec already names "a signer verified
by a key service key's name" as one it passes over. The bindings file
reads `[signing] signer` and `key` in `bindings/mod.rs` and hands them to
`profiles::signer::bind`, which binds the host key alone.

The suite's signer group (`tailrace-conformance/src/seams/signer.rs`)
asks a rig for: a signer, the identity at hand, a named signer for it, a
stranger, `no_identity`, `expire` and `unreachable`. The host key plays
every scenario but the unreachable one; this profile plays all seven.

What AWS KMS does, from its API reference (Sign, GetPublicKey, the key
state table, read 2026-10-25):

- `Sign` takes a key id or ARN, a message of at most 4096 bytes or a
  digest with `MessageType: DIGEST`, and a signing algorithm; for
  `ECDSA_SHA_256` it answers a DER-encoded signature and the key's ARN.
  It needs `kms:Sign` and fails `DisabledException` on a disabled key and
  `KMSInvalidStateException` on one pending deletion.
- `GetPublicKey` answers the public half as DER `SubjectPublicKeyInfo`,
  with `KeySpec`, `KeyUsage` and `SigningAlgorithms`. It needs
  `kms:GetPublicKey`, works across accounts by ARN, **succeeds on a
  disabled key**, and fails on a key pending deletion.
- `DescribeKey` succeeds in every key state and answers `KeyState`,
  `KeySpec`, `KeyUsage` and `Origin` (`AWS_KMS`, `EXTERNAL`,
  `AWS_CLOUDHSM`, `EXTERNAL_KEY_STORE`).
- Errors arrive as a JSON `__type` such as `AccessDeniedException`,
  `NotFoundException`, `DisabledException`, `KMSInvalidStateException`,
  with a message.

## Goals / Non-Goals

**Goals:**

- A signature only a holder of the role's right to the key can make,
  with no secret on the host's disk and nothing kept by tailrace.
- Every answer KMS or the SDK gives read as exactly one of signed,
  made, absent, refused or unreadable, the source's words carried.
- The group proven on every landing with no credential, and against a
  real key by hand.

**Non-Goals:**

- Verifying with a public half held anywhere but KMS (see Open
  Questions).
- Any key spec but `ECC_NIST_P256`, any cloud but AWS.
- Making keys or granting roles.

## Decisions

### D1. Bound by the key ARN alone; the signer's name is the ARN

The profile is `Kms { signing: Option<KeyArn>, reach: Reach }`. `KeyArn`
parses `arn:<partition>:kms:<region>:<account>:key/<id>` and nothing
else; the region the client reaches is the ARN's. The identity sign
signs as, the envelope's `signer` and its `keyid` are all the ARN
string. Alternatives: a `signer` name beside the key, as the host key
has, which C.7 rules out ("the signer's name is the key's"); an alias,
which can be moved to another key, so a signer named by it would change
identity without a word.

### D2. ECDSA P-256 over SHA-256 of the DSSE encoding, sent as a digest

The one key spec is `ECC_NIST_P256`, signing `ECDSA_SHA_256`. sign
computes `pae(IN_TOTO, payload)` (reuse `host_key::pae`), hashes it
with SHA-256 locally and sends the 32 bytes with `MessageType: DIGEST`,
so a claim of any size signs, and the bytes KMS sees are a digest, not
the claim. `sig` holds KMS's DER signature bytes as answered. verify
recomputes the encoding and checks with `p256`'s verifier, which hashes
with SHA-256 itself, so a signature made over the digest verifies over
the message. P-256 is the spec cosign and Sigstore use with KMS, so one
verifier outside tailrace reads the same envelope. Alternatives: Ed25519
(`ECC_NIST_EDWARDS25519`), newer in KMS and supported by fewer tools;
RSA, larger signatures and keys for nothing gained; sending the message
`RAW`, which fails over 4096 bytes.

### D3. sign asks DescribeKey, then Sign

In order: an identity other than the bound ARN is absent with nothing
reached; credentials are resolved (D5), and none is absent; DescribeKey
answers the key's usage, spec, origin and state; a usage other than
`SIGN_VERIFY`, a spec other than `ECC_NIST_P256`, or an origin other
than `AWS_KMS` or `AWS_CLOUDHSM` is refused with `Why::Said` naming the
key and the value; then Sign. Key material from outside KMS (`EXTERNAL`,
`EXTERNAL_KEY_STORE`) existed outside the service, so it is not a key
"the host may use and never export" (C.7). DescribeKey first costs one
call per signature and keeps a wrong key from signing anything at all.
The Sign answer's `KeyId` must equal the bound ARN, else unreadable.

### D4. How each answer is read

| what happens | sign | verify (for that signer) |
|---|---|---|
| no credentials found | absent, identity, SDK's words | unreadable, signer and key, SDK's words |
| `AccessDeniedException` | absent, identity, KMS's words | unreadable, signer and key, KMS's words |
| `NotFoundException` | absent, identity, KMS's words | unreadable, signer and key, KMS's words |
| `DisabledException`, or key state `Disabled` | absent, identity, KMS's words | made, **expired** |
| key state `PendingDeletion`, `KMSInvalidStateException` | absent, identity, KMS's words | unreadable, signer and key, KMS's words |
| wrong usage, spec or origin | refused, `Why::Said` | unreadable naming the spec or usage |
| connection failed, timeout, 5xx, `KMSInternalException`, `KeyUnavailableException`, throttled after retries | unreadable, key, SDK's or KMS's words | unreadable, signer and key, words |

Absent carries the words in its `what`, after the identity, since
`Not::absent` holds no `said`. Absent is right for sign's failures of
standing because 79 has exactly one answer for them: no identity is at
hand. verify never answers absent for a key (81): when it cannot read a
key's public half it is unreadable, since it cannot tell whether that
key made the signature, and never reports a forged envelope as made and
expired, which would count (117).

### D5. Credentials: the host's chain, resolved at each call, held by nothing

`Reach` is how the profile reaches KMS: `Reach::Host`, the default, and
`Reach::Given { credentials, http, endpoint }` for tests. Under `Host`,
each sign or verify builds a current-thread Tokio runtime, loads
`aws_config::defaults(BehaviorVersion::latest())` with the ARN's region,
resolves its credentials provider once (`provide_credentials`), builds a
KMS client with them, makes its calls and drops all of it before
answering. Resolving explicitly, rather than letting the client resolve
inside a call, gives one place where "no credentials" is told from a
failed request. Nothing is a field of the profile but the ARN and the
reach (64, core.21). The chain is the AWS tools' own: environment,
`AWS_PROFILE` and its SSO session, web identity, the container's and the
instance's role, so "the host's own role" is whatever role the host's
AWS tools already use. Alternatives: shelling out to the `aws` CLI,
another tool to pin and parse; hand-written SigV4 over `ureq`, which
would have to reimplement SSO and instance roles.

### D6. The SDK on rustls with ring, a connect timeout, standard retries

`aws-config` and `aws-sdk-kms` with default features off and
`rt-tokio`, `behavior-version-latest`, and on `aws-config` `sso` and
`credentials-process`; the HTTPS client built by `aws-smithy-http-client`
with rustls on ring, the crypto `ureq` already uses, and passed to both
the config loader and the client, so the build needs no CMake and the
flake no new input. A connect timeout of 3 s and the SDK's standard
retry mode, so a loopback port nothing listens on is answered in a few
seconds and a throttled call is retried as the SDK retries it. A retried
Sign is harmless: a signature changes nothing in KMS.

### D7. verify reaches only for a signature of its form and a KMS signer

In order: collect the signers given whose `verified_by` parses as a key
ARN (D1); a verifier that begins `arn:` and names `:kms:` but is no key
ARN (an alias ARN, a malformed one) is unreadable naming the signer, so
a broken listing is never passed over; any other verifier is passed
over. Parse each envelope signature as a DER ECDSA signature. If no
signer names a key, or no signature parses, refuse `NoNamedSigner`
naming every signer given, reaching nothing. Otherwise, per KMS signer
in order: resolve credentials (once per verify), GetPublicKey, check
`KeyUsage` and `KeySpec`, decode the SPKI, and test every parsed
signature over `pae(payload_type, payload)`; on the first that verifies,
DescribeKey, and answer `Made { signer, expired: state == Disabled }`
(a state other than `Enabled` or `Disabled` here is unreadable with the
state). If none verifies, refuse `NoNamedSigner` with words naming each
signature's `keyid` and `signer` as the envelope gives them. Reading the
public half each time keeps nothing (D5) and follows a key re-enabled
or disabled between two verifies. `p256` accepts high-S signatures only
after `normalize_s`; normalize before verifying, since KMS does not
promise low-S.

### D8. The bindings: `[signing] kms = "<key ARN>"`, read by the signer family

`[signing]` reads either `signer` and `key`, the host key, or `kms`
alone, the key service; `kms` with either other key is unreadable naming
`[signing]`. The reading moves from `bindings/mod.rs` into
`profiles::signer` (`read(file, home, table) -> Answer<Signing>`), as
`[store]` and `[landing]` are matched by their families, so the bindings
module names no key service (core.1); `bindings::path_of` becomes
`pub(crate)` for the host key's path. `Signing` is the family's own:
the identity, and which profile with which key. `profiles::signer::plan`
answers, in order, what to bind: the profile `[signing]` names, signing,
then the other verify-only; with no `[signing]`, both verify-only, host
key first. `bind` boxes the plan. The plan is a list of values with
`PartialEq`, so the binding's order is tested without reaching KMS.
ADR 0013's "a later profile is a new key, not a new shape" holds: `kms`
is a new key of `[signing]`.

### D9. Two runs of the group: a stand-in on every landing, a real key by hand

`crates/tailrace/tests/kms/mod.rs` holds a stand-in for KMS's JSON API
as an SDK HTTP client (`infallible_client_fn` of `aws-smithy-http-client`
with `test-util`, a dev-dependency): it reads `X-Amz-Target`
(`TrentService.Sign`, `GetPublicKey`, `DescribeKey`, `EnableKey`,
`DisableKey`) and the JSON body, holds keys by ARN as `p256` signing keys
with a state, spec, usage and origin, answers errors as `__type` and
`message` the way KMS does, counts requests, and can be told to answer
any operation with a given error. A switchable credentials provider
answers static test credentials or, once switched off, "no credentials".
`tests/kms_signer.rs` runs the group over a rig of the stand-in (expire
disables the key; unreachable swaps in a reach at a free loopback port
with the SDK's real client) and holds the profile's own tests; it runs
in the `cli` project's `test` task, `check-fast`.
`tests/kms_account.rs`, behind the `tools` feature, runs the group over
`Reach::Host` against the key `TAILRACE_TOOLS_KMS_KEY` names, with
`TAILRACE_TOOLS_KMS_STRANGER` as the stranger; each scenario's rig
enables the key and waits until DescribeKey says `Enabled`, expire
disables it and waits for `Disabled`, no_identity switches off the
credentials wrapping the host's chain, and the test enables the key
again when the group ends, pass or fail. It refuses to run, naming both
settings, on a host that sets none. Its moon task `kms` is tagged
`check-kms`, outside the gate (core.38), as `tools-ecr` is.

### D10. ADR 0016

The record holds D1 to D4 and D7: the key named by its ARN, P-256 over
the DSSE encoding's digest, a key's standing as its state in KMS
(disabled is expired; pending deletion or deleted, unreadable),
imported key material refused, and verify reaching KMS for the public
half. Status proposed, for the user to accept with the change.

## Risks / Trade-offs

- [verify needs AWS credentials with `kms:GetPublicKey` and
  `kms:DescribeKey` on the key, on every machine that verifies] → the
  key's policy grants the readers' accounts; the signers guide says so.
  A machine without them answers unreadable, naming the key, never a
  pass and never a refusal.
- [A key scheduled for deletion makes every claim it made unreadable,
  including ones accepted before (117)] → the guide says to disable a
  key to retire it and never to schedule its deletion while anything it
  signed is in use; the gap goes to madswan-design (Open Questions).
- [The SDK with rustls on ring may not build without the default
  aws-lc client] → if it cannot, the coder stops and says so with
  `crew needs`: the alternative adds CMake to `devenv.nix` and the
  flake, outside this change.
- [KMS is eventually consistent: a sign just after EnableKey may meet
  `DisabledException`] → the account rig waits for DescribeKey's state,
  and then for one sign to pass, before handing the key to a scenario.
- [The stand-in can drift from KMS] → it is checked against the real
  service by the `check-kms` run, which uses the same group.
- [Two KMS keys cost the test account about $2 a month] → accepted.

## Migration Plan

None: a host key binding reads as before. A host moves to the key
service by replacing `[signing]`'s `signer` and `key` with `kms`, and
the policy lists the new signer beside the old until the old one's
claims are no longer needed.

## Open Questions

- Should the policy be able to carry a KMS key's public half beside its
  ARN, so a claim stays verifiable after its key is deleted (81, 117)?
  The answer adds a form to the verifier and changes no part of this
  change's approach; it goes to madswan-design.
