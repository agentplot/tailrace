# Spec Delta

## Purpose

The signer seam (B.4) bound to a key in AWS KMS that the host uses
through its own role and can never export (C.7): claims signed as the
key, told apart by the key's public half, and a key switched off
reported as expired.

## ADDED Requirements

### Requirement: A key service signer is a KMS key named by its key ARN

The profile SHALL be bound with one value, the key's ARN,
`arn:<partition>:kms:<region>:<account>:key/<key id>`, and the signer's
identity SHALL be that ARN (C.7). It SHALL reach the key in the ARN's
own region. A value that is an alias, a bare key id or no KMS key ARN
SHALL be unreadable naming the value, since an alias can be moved to
another key.

#### Scenario: Bound by its key ARN

- **WHEN** the profile is bound with `arn:aws:kms:us-west-2:111122223333:key/1234abcd-12ab-34cd-56ef-1234567890ab`
- **THEN** it signs as that ARN and reaches KMS in `us-west-2`

#### Scenario: Bound by an alias

- **WHEN** the profile is bound with `arn:aws:kms:us-west-2:111122223333:alias/fleet`
- **THEN** the binding is unreadable, naming the value and saying a key is named by its key ARN

### Requirement: sign signs as the key alone, through the host's role

sign SHALL sign a claim only as the bound key's ARN; asked to sign as
any other identity it SHALL answer absent naming that identity, and
reach nothing (79, 11, core.8). The envelope SHALL hold the claim's
payload, its payload type and one signature, naming the ARN as its
signer and its key id, made by KMS with the key over the envelope's
DSSE pre-authentication encoding.

#### Scenario: Signed as the key

- **WHEN** the profile bound to a key signs a provenance claim as that key's ARN
- **THEN** the envelope holds that claim and names the ARN, and its signature verifies against the key's public half over the envelope's pre-authentication encoding

#### Scenario: Asked to sign as another

- **WHEN** the profile bound to a key is asked to sign as `gate`
- **THEN** the answer is absent from the signer naming `gate`, and no request reaches KMS

### Requirement: Credentials are the host's, reached at each signature

sign and verify SHALL take credentials from the host as the AWS tools
find them (the environment, the AWS profile and its SSO session, the
container's or instance's role) at each call, and SHALL keep none
between calls, in the profile, the envelope or any file (64, core.21).
The bindings file SHALL name no credential.

#### Scenario: The credentials taken away after a signature

- **WHEN** a claim is signed, the host's credentials then taken away, and a claim signed again
- **THEN** the second sign answers absent naming the identity, carrying the SDK's words for finding no credentials

#### Scenario: Nothing of the credentials in the envelope

- **WHEN** a claim is signed with a known access key and session token
- **THEN** the envelope's bytes hold neither

### Requirement: A key the host cannot sign with is no identity at hand

sign SHALL answer absent naming the identity, carrying KMS's words
verbatim, when the host has no credentials, when its role may not use
the key, when the key is disabled or pending deletion, and when no key
has the ARN: in each, no identity is at hand (79). Nothing SHALL be
signed.

#### Scenario: A role the key does not let sign

- **WHEN** KMS answers the sign with `AccessDeniedException`
- **THEN** sign answers absent naming the identity and carrying KMS's message

#### Scenario: A disabled key

- **WHEN** the bound key is disabled and a claim is signed
- **THEN** sign answers absent naming the identity and carrying KMS's words that the key is not enabled

### Requirement: A key of the wrong kind is refused

sign SHALL be refused, naming the key and what it is, for a key whose
usage is not signing, whose key spec is not `ECC_NIST_P256`, or whose
key material came from outside KMS, since such a key was exported once
(C.7). Nothing SHALL be signed.

#### Scenario: An RSA key

- **WHEN** the bound key's spec is `RSA_2048`
- **THEN** sign is refused naming the key and `RSA_2048`, and no envelope is answered

#### Scenario: Imported key material

- **WHEN** the bound key's origin is `EXTERNAL`
- **THEN** sign is refused naming the key and saying its key material was imported

### Requirement: verify checks the signature with the key's public half

verify SHALL, for each signer given in order whose verifier is a KMS key
ARN, read that key's public half from KMS and check the envelope's
signatures with it on this machine, and SHALL answer the first signer
whose key made a valid signature over exactly the envelope's payload
and payload type (80, C.7). The name the envelope gives its signer
SHALL decide nothing (core.7).

#### Scenario: Verified on another host

- **WHEN** an envelope one host signed is verified on a host bound to no KMS key, against signers naming the key's ARN
- **THEN** verify names that signer, not expired

#### Scenario: The envelope names another signer

- **WHEN** an envelope signed with the key is changed to name `gate` as its signer and key id, and verified against signers naming `gate` with a second key and `fleet` with the first
- **THEN** verify names `fleet`

#### Scenario: The payload changed

- **WHEN** an envelope's payload is changed after signing and it is verified against the key's signer
- **THEN** verify is refused, saying none of the signers checked made it

### Requirement: An envelope with no signature of this profile's form is refused unreached

verify SHALL be refused as made by none of the signers checked, reaching
nothing, when no signature of the envelope is a DER-encoded ECDSA
signature, or when no signer given names a KMS key, so the core asks the
next signer bound (80). A signer whose verifier is not a KMS key ARN
SHALL be passed over and still named among those checked.

#### Scenario: A host key's envelope

- **WHEN** an envelope signed with a host key is verified by this profile against signers naming a KMS key and a host key
- **THEN** verify is refused naming both signers as checked, and no request reaches KMS

### Requirement: A signature by no listed key is refused naming the key it names

When no signer given has a key that made a signature, verify SHALL be
refused naming the signers it checked, with words naming the signature's
key id and signer as the envelope gives them, so whoever reads the
refusal can tell whose claim it says it was (S5, 9, 80).

#### Scenario: S5's verdict from another key

- **WHEN** a verdict claim signed with a second key is verified against signers naming only the first key
- **THEN** verify is refused naming the first key's signer as checked, and its words carry the second key's ARN as the envelope names it

### Requirement: A disabled key is told as expired

The key's standing SHALL be its state in KMS. A signature by a key that
is disabled when verify runs SHALL be answered as made by its signer
and expired, never as valid and never as absent (81); a key enabled
again SHALL be answered not expired.

#### Scenario: Verified after the key is disabled

- **WHEN** an envelope is signed, the key then disabled, and the envelope verified against the key's signer
- **THEN** verify names the signer and says it has expired

#### Scenario: Verified after the key is enabled again

- **WHEN** the disabled key is enabled and the same envelope verified
- **THEN** verify names the signer, not expired

### Requirement: A key whose public half KMS no longer gives is unreadable

When KMS gives no public half for a signer's key, because the key is
pending deletion, deleted, or the host may not read it, verify SHALL
answer unreadable naming that signer and its key and carrying KMS's
words verbatim, never valid, never refused and never absent (58, 59,
81, core.19).

#### Scenario: A key pending deletion

- **WHEN** the signer's key is scheduled for deletion and an envelope it signed is verified
- **THEN** verify answers unreadable naming the signer and the key, carrying KMS's words on the key's state

#### Scenario: A key the host may not read

- **WHEN** KMS answers the public key's read with `AccessDeniedException`
- **THEN** verify answers unreadable naming the signer, carrying KMS's message

#### Scenario: A listed key of another spec

- **WHEN** the signer's key is an RSA signing key
- **THEN** verify answers unreadable naming the signer and the key's spec

### Requirement: A service that cannot be reached is unreadable

When KMS cannot be reached, or answers with a failure of its own after
the SDK's retries, sign and verify SHALL answer unreadable naming the
key, carrying the SDK's or KMS's words, never absent and never refused
(58, 59). An unreachable service SHALL be answered within seconds.

#### Scenario: No service at the endpoint

- **WHEN** the profile's endpoint is a loopback port nothing listens on
- **THEN** sign and verify each answer unreadable within ten seconds, carrying the SDK's words for the failed connection

### Requirement: The profile can be bound to verify only

The profile SHALL be bindable with no key, for a host that verifies and
signs nothing: verify then tells a claim by the keys the policy names
exactly as a profile with a key does, and sign answers absent for every
identity, saying no identity is at hand, reaching nothing (79, 80).

#### Scenario: Sign on a host with no key

- **WHEN** the verify-only profile is asked to sign a claim as any identity
- **THEN** it answers absent, saying no identity is at hand, and no request reaches KMS

### Requirement: The signer group passes against a stand-in and against a real key

The profile SHALL pass the conformance suite's signer group, every
scenario played, in its own tests twice: on every landing against an
in-process stand-in for KMS's API reached through the SDK, and by hand
against a real KMS key in a test account, outside the gate (93,
core.38).

#### Scenario: The signer group over the stand-in

- **WHEN** the profile's tests run the signer group over a rig whose KMS is the in-process stand-in
- **THEN** every scenario of the group passes, none reported not applicable

#### Scenario: The signer group against the test account

- **WHEN** `moon run ':#check-kms'` runs with a live session on a host whose `devenv.local.nix` names a key and a second key in a test account
- **THEN** every scenario of the group passes against those keys, the expired identity's scenario disabling the key and the key enabled again afterwards
