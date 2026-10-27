## ADDED Requirements

### Requirement: The given host key is made on first use

A host key bound as the given one, at `tailrace/host/signing.key` in the
user's data directory (`$XDG_DATA_HOME`, or `~/.local/share` where that
variable is unset or empty), SHALL, on a sign that finds no key file
there, make its directory with mode 0700 where it makes it, make an
Ed25519 key in OpenSSH's format with no passphrase, mode 0600, write its
public half beside it as `signing.key.pub`, one line naming the signer,
and sign with it (118, C.8). Only sign makes it. A key file that
is there SHALL be read as any bound key is, never replaced.

#### Scenario: The first sign

- **WHEN** the given host key is asked to sign a claim as the host's name and no file is at its path
- **THEN** `tailrace/host` is made under the data directory with mode 0700, the key and `signing.key.pub` are made in it, the key's mode is 0600, the envelope verifies against the public half `signing.key.pub` holds, and that line, read as a policy listing, tells the envelope as made by the host's name

#### Scenario: The second sign

- **WHEN** the given host key signs a second claim
- **THEN** it signs with the key the first sign made, and neither file is written again

#### Scenario: A key others may read at the given path

- **WHEN** a key file of mode 0644 is at the given path
- **THEN** sign is refused naming the file, its mode and `chmod 600`, and the file is not replaced

#### Scenario: Two first signs at once

- **WHEN** two processes sign with the given host key at once and no key is there
- **THEN** one key is made, and both envelopes verify against it

#### Scenario: Verify makes nothing

- **WHEN** the host key bound verify-only verifies an envelope on a host with no key at the given path
- **THEN** no file is made

#### Scenario: A directory that cannot be written

- **WHEN** the given key's directory cannot be made or written
- **THEN** sign answers unreadable naming the path, carrying the operating system's words, and no envelope is answered

### Requirement: The given host key signs as the host's name

The given host key's signer SHALL be the name `signing.key.pub` gives where
that file is there, and otherwise the host's own name as the operating
system answers it, its first label in lower case, which becomes the
name `signing.key.pub` gives once the key is made (11, 104). A host name
that is empty or cannot be read SHALL be answered unreadable naming the
host's name, carrying the operating system's words where it has them.

#### Scenario: A fresh host

- **WHEN** the host's name is `Laptop.local` and no `signing.key.pub` is there
- **THEN** the given signer is `laptop`, and the key made on first use names `laptop` in `signing.key.pub`

#### Scenario: The host renamed after the key was made

- **WHEN** `signing.key.pub` names `laptop` and the host is now named `desk`
- **THEN** the given signer is still `laptop`

#### Scenario: A public half that cannot be read

- **WHEN** `signing.key.pub` is there and holds no OpenSSH public key line naming a signer
- **THEN** a publish answers unreadable naming `signing.key.pub`, carrying the parser's words, and nothing is signed

## MODIFIED Requirements

### Requirement: The key is read at the moment of use and kept nowhere

sign SHALL read the key file each time it signs and SHALL keep nothing
of the key afterwards: not in the profile between signatures, not in the
envelope, and not in any file it writes but the given key's own (64,
core.21). A key file a bindings file names that is not there SHALL be
answered absent naming the identity and the path: no identity is at
hand (79). The given host key's missing file is made instead, as its
own requirement says.

#### Scenario: The key replaced between two signatures

- **WHEN** a claim is signed, the key file replaced by another key, and a claim signed again
- **THEN** the second envelope verifies with the new key's public half and not the old one's

#### Scenario: The key file removed after a signature

- **WHEN** a claim is signed with a key a bindings file names, the key file removed, and a claim signed again
- **THEN** the second sign answers absent naming the identity and the key file's path

#### Scenario: Nothing of the key leaves it

- **WHEN** a claim is signed with a key in a temporary directory
- **THEN** the envelope's bytes hold neither the private key's file text nor its secret bytes, and the directory holds only the files the test wrote
