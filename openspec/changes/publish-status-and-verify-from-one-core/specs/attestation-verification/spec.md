# Spec Delta

## Purpose

What `tailrace verify` tells of a version: which signer the policy names
made each attestation attached to its index and artifacts, and each
verdict those attestations carry, whether it counts, and that it writes
nothing.

## ADDED Requirements

### Requirement: Verify resolves the version to its index first

Verify SHALL resolve a version's name to the digest it names in the
store, then read the index at that digest and the artifacts it names,
and SHALL name that digest in its answer (3, core.4). A name that names
nothing SHALL be answered absent naming it.

#### Scenario: A published version

- **WHEN** a published version is verified by its version
- **THEN** the answer names the index's digest the version's name resolved to, and every artifact digest the index names

#### Scenario: A version never published

- **WHEN** a version whose name the store does not hold is verified
- **THEN** the answer is absent naming the version's name, never refused

### Requirement: Verify reads the policy at the version's landed revision

Verify SHALL read the signers the policy names for the audience it is
asked for, acceptance unless another is named, at the revision the
version's tag names in the history, once the landing seam says the
branch has landed it (65, 82). An audience with no policy declared
SHALL be answered absent naming the audience and the revision.

#### Scenario: The policy read at the tag's revision

- **WHEN** a version is verified and its tag names a revision the branch has landed
- **THEN** the fake policy records one ask, for acceptance at that revision, after the fake landing's yes

#### Scenario: No policy for the audience

- **WHEN** the fake policy declares nothing for the audience asked
- **THEN** the answer is absent naming the audience and the revision, and no claim is reported as counted

### Requirement: Each attestation is told by the signer that made it

For each attestation attached to the index and to each artifact, and for
each verdict a provenance claim carries, verify SHALL answer which of
the signers the policy names made it, by asking the signer seam with
those signers (80), and the digest it is attached to.

#### Scenario: A provenance and its carried verdicts

- **WHEN** a version whose artifacts each carry a provenance claim holding two verdicts is verified under a policy naming the host's signer and the landing's signer
- **THEN** the answer names, for each artifact, the provenance made by the host's signer and each carried verdict made by the landing's signer

### Requirement: An expired signer is reported as such

A signature by an identity that has expired SHALL be answered as made by
that signer and expired, never as valid and never as absent (81). The
expiry SHALL be reported beside the claim and SHALL not by itself stop
the claim counting (117).

#### Scenario: A signer whose identity expired

- **WHEN** the fake signer marks the landing's signer expired and a version carrying its verdict is verified
- **THEN** the verdict is answered made by the landing's signer, marked expired, and is not reported absent or unsigned

### Requirement: A claim from no named signer is kept and never counted

An attestation that no signer the policy names made, or that its signer
made beyond the kinds and checks the policy lets it claim, SHALL be
listed in the answer as kept and not counted, naming the signers it was
checked against; it SHALL never be removed and never counted (9, 116,
core.7).

#### Scenario: A verdict from a signer the policy does not name

- **WHEN** a provenance carries a verdict signed by a signer the policy names for nothing
- **THEN** the answer lists that verdict as made by no named signer, kept and not counted, names the signers checked, and claims-of the artifact still answers the provenance carrying it

#### Scenario: A signer beyond its reach

- **WHEN** the policy names the host's signer for provenance alone and the host's signer made a claim of another kind
- **THEN** the answer names the host's signer as its maker and that it does not count

#### Scenario: A changed attestation

- **WHEN** an attestation's claim was changed after it was signed
- **THEN** the answer lists it as made by no named signer, not counted

### Requirement: A failed read is not an empty answer

Where the store cannot be read for a digest's claims, or the signer seam
cannot verify, verify SHALL answer unreadable naming what could not be
read with the source's words, never an empty list and never a claim
from no signer (58, 59, core.19).

#### Scenario: The store unreachable for claims

- **WHEN** the fake store's claims-of is unreadable with the words `connection refused`
- **THEN** verify answers unreadable carrying `connection refused` verbatim, and lists no claim

### Requirement: Verify writes nothing

Verify SHALL put, attach, sign, name and record nothing (20, 60).

#### Scenario: Nothing written by a verify

- **WHEN** a published version is verified
- **THEN** the fake store records no put, attach or name, the fake signer records no sign, and the host's record is unchanged
