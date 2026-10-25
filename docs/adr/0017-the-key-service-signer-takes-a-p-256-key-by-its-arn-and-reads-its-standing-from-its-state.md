---
number: 17
title: The key service signer takes a P-256 key by its ARN and reads its standing from its state
status: proposed
date: 2026-10-25
decision-makers:
- Chuck Swanberg
---

# The key service signer takes a P-256 key by its ARN and reads its standing from its state

Deciders: proposed by the-signer-in-a-key-service unit of phase 1's bolt 4, which binds the signer seam to a key in AWS KMS used through the host's own role (`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 4), after the operator ruled that the offering signer on a build fleet is a key in a key service (rulings, 2026-10-24), for the user to accept with the change.
Sources: statement clauses 11, 58, 59, 64, 79, 80, 81, 117, B.4, C.7; ADRs 0005, 0013; rules core.1, core.7, core.8, core.19, core.21, core.38; AWS KMS's API reference for Sign, GetPublicKey and DescribeKey, and its table of key states, read 2026-10-25; the change's design, D1 to D4 and D7.

## Context and problem statement

C.7 binds the signer seam to "a key the host's cloud identity may use and never export, such as a key in AWS KMS used through the host's role; the signer's name is the key's, and verification uses its public half". Four questions are left to the profile.

A KMS key has several names: its key ARN, its bare key id, and any number of aliases, each with an ARN of its own. Which is the signer's name, the one `[signing]` binds and a policy lists?

KMS signs with keys of several specs and algorithms, a message of at most 4096 bytes or a digest. A claim's envelope is signed over its DSSE pre-authentication encoding, of any length. What key, what algorithm, and what does KMS sign?

81 binds a signer's identity to its holder's standing and asks that a signature by an expired identity be reported as such, never valid and never absent; 117 lets a claim that counted at acceptance keep counting after. A KMS key has no expiry of its own. It has a state: enabled, disabled, pending deletion, or deleted. KMS answers GetPublicKey on a disabled key, and on a key pending deletion or deleted it answers nothing. What is a key's standing, and what does verify answer for each state?

A key's material is made in KMS, in a CloudHSM cluster, imported from outside, or held in an external key store. Which of these is a key "the host may use and never export"?

## Considered options

For the name:

1. The key ARN, `arn:<partition>:kms:<region>:<account>:key/<key id>`, alone: the identity sign signs as, the envelope's signer and key id, and the policy's listing; the region the ARN's.
2. A signer's name beside the key, as a host key has one.
3. An alias, or an alias's ARN.
4. The bare key id, with the region from the host's configuration.

For what is signed:

1. An `ECC_NIST_P256` key, `ECDSA_SHA_256`, the SHA-256 of the envelope's pre-authentication encoding sent with `MessageType: DIGEST`, the signature KMS's DER bytes as answered.
2. The same key, the encoding sent `RAW`.
3. An `ECC_NIST_EDWARDS25519` key.
4. An RSA key.

For a key's standing:

1. Its state in KMS when verify runs: enabled is not expired, disabled is expired, and a key whose public half KMS no longer gives, pending deletion or deleted, is unreadable, naming the key and carrying KMS's words.
2. Disabled and pending deletion both expired, the public half read before deletion was scheduled.
3. A validity time in the policy's listing, as a host key's `valid-before`.
4. Any state but enabled answered absent.

For the key's material:

1. Made in KMS, origin `AWS_KMS` or `AWS_CLOUDHSM`; a key whose material was imported, `EXTERNAL`, or is held outside, `EXTERNAL_KEY_STORE`, refused at sign, naming the key and its origin.
2. Any origin.

## Decision outcome

The name is option 1. C.7 says the signer's name is the key's, and the key ARN is the one name of the key that names nothing else and never moves: the identity sign signs as, the envelope's `signer` and `keyid`, and what a policy lists, `[signers.<name>] kms = "<key ARN>"`, are all that one text, and the client reaches KMS in the ARN's own region. `[signing]` binds it as `kms = "<key ARN>"` alone (ADR 0013). A text that begins `arn:` and names `:kms:` and is no key ARN, an alias's ARN or a broken one, is unreadable naming it wherever it is read, in `[signing]` or in a policy's listing, so a broken listing is never passed over (58); any other text is another profile's verifier. Option 2 gives the key two names, which C.7 rules out. Option 3 names whichever key the alias points at today: an alias moved to another key would change who the signer is without a word. Option 4 makes a name mean another key on a host configured for another region.

What is signed is option 1. sign computes the envelope's DSSE pre-authentication encoding, hashes it with SHA-256 on the host and sends KMS the 32 bytes as a digest, so a claim of any size signs and KMS sees a digest, never the claim. verify recomputes the encoding and checks the signature with the key's public half on the host, normalizing S first, since KMS does not promise a low-S signature and the verifier takes only those. P-256 with ECDSA over SHA-256 is the spec cosign and Sigstore use with KMS, so a verifier outside tailrace reads the same envelope. Option 2 fails a claim over 4096 bytes. Option 3 is newer in KMS and read by fewer tools. Option 4 makes larger keys and signatures for nothing gained.

A key's standing is option 1. A key disabled is the key's owner switching it off, and KMS still gives its public half, so verify tells which signer made the signature and answers it made and expired (81), which counts where it counted at acceptance (117); enabled again, the key answers not expired. Verify reads the public half and the state at each call, keeping nothing (core.21), so a key enabled or disabled between two verifies is answered as it stands. A key pending deletion or deleted gives no public half: verify cannot tell whether that key made the signature, so it answers unreadable, naming the signer and its key and carrying KMS's words, never valid, never refused and never absent (58, 59, core.19). Option 2 needs a public half held somewhere but KMS, which the statement does not yet say a policy may carry; it is the open question this change hands madswan-design. Option 3 makes the policy keep a second record of what KMS already holds. Option 4 is what 81 rules out.

At sign, a key the host may not sign with now is no identity at hand (79): the host has no credentials, KMS answers `AccessDeniedException` or `NotFoundException`, or the key is disabled or pending deletion, `DisabledException` or `KMSInvalidStateException`; sign answers absent naming the identity and carrying the SDK's or KMS's words. sign asks DescribeKey before Sign, so a key of the wrong kind signs nothing at all: a usage but `SIGN_VERIFY` or a spec but `ECC_NIST_P256` is refused naming the key and what it is. A service that cannot be reached, or KMS's own failure after the SDK's retries, is unreadable at sign and at verify alike (58, 59).

The key's material is option 1. Imported material existed outside KMS before it was imported, and material in an external key store is held outside it, so neither is a key "the host may use and never export" (C.7). Option 2 lets a key whose private half sits somewhere else sign as if it never had.

The credentials are the host's own AWS chain, found as the AWS tools find them at each call, the environment, the AWS profile and its SSO session, web identity, the container's or the instance's role, and dropped before the call answers (64, core.21). The bindings file names no credential. The SDK reaches KMS over rustls on ring, the crypto the rest of the binary uses.

### Consequences

- A host signs claims with nothing on its disk to steal: the private half never leaves KMS, and tailrace holds no credential between calls.
- Every machine that verifies a KMS signer's claims needs AWS credentials that may `kms:GetPublicKey` and `kms:DescribeKey` on the key, granted by the key's policy to the readers' accounts; a machine without them answers unreadable naming the key, never a pass and never a refusal.
- A key is retired by disabling it: its signatures are reported expired and still count where they counted (81, 117). A key scheduled for deletion makes every claim it signed unreadable, accepted ones among them, until the policy can carry its public half; the signers guide says never to schedule a key's deletion while anything it signed is in use.
- Every host binds the KMS profile as well as the host key, verify-only where its file names no KMS key, so a laptop with a host key verifies a fleet's claims. An envelope with no ECDSA signature, or signers no KMS key lists, is refused without reaching KMS, so a host key's claim is told on a host with no AWS credentials at all.
- The gate proves the profile on every landing against an in-process stand-in for KMS's API under the real SDK, with no credential (core.38); the same signer group runs against two real keys in a test account by hand, `moon run ':#check-kms'` with a live session, and a change to how the profile reads KMS's answers is proven there again.
- Only P-256 signing keys in AWS KMS are bound. Another spec, another cloud's key service, or keyless signing (C.9) is a profile or a decision of its own.
