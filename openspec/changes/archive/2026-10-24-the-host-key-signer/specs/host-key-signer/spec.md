# Spec Delta

## Purpose

The signer seam (B.4) bound to a key file on the host, held as sluice's
signers hold theirs (C.8): claims signed as the host running tailrace,
told apart by the public key the policy lists, with no credential kept.

## ADDED Requirements

### Requirement: A host key is a key file only its owner reads

The profile SHALL be bound with the signer's name and the path of an
Ed25519 private key in OpenSSH's format with no passphrase, the form
sluice signs with (C.8). A key file whose mode lets anyone but its owner
read it SHALL be refused with the words `which lets others read it` and
`chmod 600`, and nothing signed; a key with a passphrase SHALL be
refused saying tailrace signs with none to type.

#### Scenario: A key others may read

- **WHEN** a claim is signed with a key file of mode 0644
- **THEN** sign is refused from the signer naming the file, its mode and `chmod 600`, and no envelope is answered

#### Scenario: A key with a passphrase

- **WHEN** a claim is signed with a key file encrypted under a passphrase
- **THEN** sign is refused naming the file and saying tailrace signs with no passphrase to type

#### Scenario: A key of another algorithm

- **WHEN** a claim is signed with an RSA key file
- **THEN** sign is refused naming the key's algorithm, and no envelope is answered

### Requirement: sign signs as the host's own identity alone

sign SHALL sign a claim only as the signer the profile is bound with:
asked to sign as any other identity, it SHALL answer absent naming that
identity, and nothing is signed (79, 11, core.8). The envelope SHALL
hold the claim's payload, its payload type, the bound signer's name and
one SSH signature over the DSSE pre-authentication encoding of the two,
in tailrace's own signature namespace.

#### Scenario: Signed as the host

- **WHEN** the profile bound as `host` signs a provenance claim as `host`
- **THEN** the envelope holds that claim, names `host`, and its signature is an armored SSH signature that verifies against the key's public half in tailrace's namespace over the envelope's pre-authentication encoding, and in no other namespace

#### Scenario: Asked to sign as another

- **WHEN** the profile bound as `host` is asked to sign as `gate`
- **THEN** the answer is absent from the signer naming `gate`, and no key file is read

### Requirement: The key is read at the moment of use and kept nowhere

sign SHALL read the key file each time it signs and SHALL keep nothing
of the key afterwards: not in the profile between signatures, not in the
envelope, and not in any file it writes (64, core.21). A key file that
is not there SHALL be answered absent naming the identity and the path:
no identity is at hand (79).

#### Scenario: The key replaced between two signatures

- **WHEN** a claim is signed, the key file replaced by another key, and a claim signed again
- **THEN** the second envelope verifies with the new key's public half and not the old one's

#### Scenario: The key file removed after a signature

- **WHEN** a claim is signed, the key file removed, and a claim signed again
- **THEN** the second sign answers absent naming the identity and the key file's path

#### Scenario: Nothing of the key leaves it

- **WHEN** a claim is signed with a key in a temporary directory
- **THEN** the envelope's bytes hold neither the private key's file text nor its secret bytes, and the directory holds only the files the test wrote

### Requirement: A key file that cannot be read is unreadable

A key file that is there and cannot be read, or that does not parse as
an OpenSSH private key, SHALL be answered unreadable naming the file and
carrying the operating system's or the parser's words verbatim, never
absent and never refused (58, 59, core.19).

#### Scenario: A key file that is a directory

- **WHEN** the bound key path is a directory and a claim is signed
- **THEN** sign answers unreadable from the signer naming the path, carrying the operating system's words

#### Scenario: A key file that is not a key

- **WHEN** the bound key file holds text that is no OpenSSH private key
- **THEN** sign answers unreadable naming the file, carrying the parser's words

### Requirement: verify tells a claim by the public key the policy lists

verify SHALL answer the first signer given, in order, whose verifier is
an SSH public key that made a valid signature in tailrace's namespace
over exactly the envelope's payload and payload type (80, C.8). It SHALL
read no key file and reach nothing. The name the envelope gives its
signer SHALL decide nothing (core.7).

#### Scenario: Verified with no key file on the machine

- **WHEN** an envelope the host signed is verified on a machine with no key file, against signers listing the host's public key
- **THEN** verify names the host's signer, not expired

#### Scenario: The envelope names another signer

- **WHEN** an envelope signed by the host's key is changed to name `gate` as its signer and verified against signers listing `gate` with another key and `host` with the host's key
- **THEN** verify names `host`

#### Scenario: The payload type changed

- **WHEN** an envelope's payload type is changed after signing and it is verified against the host's signer
- **THEN** verify is refused, saying none of the signers checked made it

### Requirement: A signature by no listed key is refused naming its key

When no signer given lists the key that made the signature, verify SHALL
be refused, naming the signers it checked, with words naming that key by
its SHA-256 fingerprint and the name the envelope gives itself, so whoever
reads the refusal can tell whose claim it was (S5, 9, 80).

#### Scenario: S5's verdict from another host

- **WHEN** a verdict claim signed with a second host's key is verified against signers listing only the first host's key
- **THEN** verify is refused naming the first host's signer as checked, and its words carry the second key's `SHA256:` fingerprint

### Requirement: An envelope this profile cannot read is refused, not unreadable

An envelope whose signature is no SSH signature, or one made in another
namespace, SHALL be refused as made by none of the signers checked, never
unreadable, so the core asks the next signer bound (80). A signer given
whose verifier is not an SSH public key SHALL be passed over and still
named among those checked.

#### Scenario: Another profile's envelope

- **WHEN** an envelope signed by the fake signer is verified by this profile
- **THEN** verify is refused, saying none of the signers checked made it

#### Scenario: A signature in another namespace

- **WHEN** an envelope's signature was made with the listed key in the namespace `git`
- **THEN** verify is refused, its words naming the namespace the signature was made in

#### Scenario: A signer verified by a key's name

- **WHEN** the signers given are one whose verifier is a key service key's name and one listing the host's public key
- **THEN** verify names the host's signer, and a refusal names both as checked

### Requirement: A listed key that cannot be read is unreadable

A signer given whose verifier names an SSH key type and does not parse
as an OpenSSH public key with options this profile reads SHALL make
verify unreadable, naming that signer and carrying the parser's words,
never a refusal and never passed over (58, 59).

#### Scenario: A listed key with a broken body

- **WHEN** a signer's verifier is `ssh-ed25519` followed by text that is not a key
- **THEN** verify answers unreadable naming that signer and carrying the parser's words

### Requirement: A listing past its valid-before is told as expired

A listed key MAY carry OpenSSH's `valid-before` option, a UTC time ending
in `Z`. Once that time has passed, a signature by the key SHALL be
answered as made by its signer and expired, never as valid and never as
absent (81). A `valid-before` time with no `Z` SHALL be unreadable, so no
host's time zone changes the answer.

#### Scenario: A listing that has expired

- **WHEN** an envelope the host signed is verified against the host's key listed with `valid-before="20000101Z"`
- **THEN** verify names the host's signer and says it has expired

#### Scenario: A listing still standing

- **WHEN** the host's key is listed with a `valid-before` a year from now
- **THEN** verify names the host's signer, not expired

#### Scenario: A time with no zone

- **WHEN** the host's key is listed with `valid-before="20991231"`
- **THEN** verify answers unreadable naming the host's signer and saying the time must end in `Z`

### Requirement: The signer group passes against a key in a temporary directory

The profile SHALL pass the conformance suite's signer group, a fresh key
in a temporary directory per scenario, in the profile's own tests (93).
A scenario that asks verify to be unreachable SHALL be reported not
applicable with the reason that this profile's verify reaches nothing.

#### Scenario: The signer group over the host key

- **WHEN** the profile's tests run the signer group over a rig binding a fresh key in a temporary directory
- **THEN** every scenario passes but the unreachable signer's, which the report names not applicable with its reason
