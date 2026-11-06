# environment-reports Specification

## Purpose
TBD - created by archiving change the-clan-push-profile. Update Purpose after archive.

## Requirements

### Requirement: A converged apply signs one report

When an apply converges and the target's report names the index that
was applied, the core SHALL sign one report claim as the host's signer:
what the host observed the target answer (11, 43, 91, rule core.8).
When the report is unread, or names another digest, no report SHALL be
signed, and apply's answer SHALL carry the target's answer in its
place.

#### Scenario: srv converged

- **WHEN** apply converges `srv` on index `sha256:aa…` and the clan push profile's report answers `sha256:aa…`
- **THEN** one report claim about `sha256:aa…` is signed by the host's signer, and apply's answer names its digest

#### Scenario: A report that names another digest

- **WHEN** the target converges but its report answers `sha256:bb…` for an apply of `sha256:aa…`
- **THEN** no report is signed, and apply's answer is unread naming both digests

### Requirement: An environment already running the head is reported once

Where apply was not asked because the environment already runs the
head, the core SHALL ask the target's report, and sign one only where
the environment's newest report names another index or none. A report
the store failed to hold is made again, and one already held is never
made twice.

#### Scenario: Already running the head, reported

- **WHEN** current already answers the channel's head, and the environment's newest report names that head
- **THEN** no report is signed

#### Scenario: Already running the head, its report lost

- **WHEN** current already answers the channel's head, and the attach of the report of the apply that converged it was refused
- **THEN** the target's report is asked, and one report claim about the head is signed and attached

### Requirement: A report claim says what runs where, and when

A report claim SHALL be about the index digest applied, and name the
revision the index names (13, rule core.9). Its body SHALL name the
environment and its component, the channel it follows, the version, the
digest the target reported, the time the target says it converged, and
the moment of the apply that made it, read from the asker as a
promotion's is.

#### Scenario: The report's body

- **WHEN** the host reports `srv` of `swancloud-srv` on `srv`'s channel at `0.4.0`
- **THEN** its body names `srv`, `swancloud-srv`, the channel, `0.4.0`, the digest, the convergence time and the apply's moment

### Requirement: A report is of timber's own kind

A report claim SHALL be of timber's own kind, `report`, since the
in-toto catalogue carries no claim that an environment runs a digest
(141, rule core.57, section 9).

#### Scenario: The report's type

- **WHEN** a report claim is read
- **THEN** its predicate type is timber's `report`

### Requirement: A report is attached beside its index

The report claim SHALL be attached to the index digest it is about,
through the store, and kept nowhere else (91, rule core.27). A store
that refuses the attach SHALL make apply's answer for the report carry
the store's refusal verbatim, with the convergence reported as made
(59).

#### Scenario: Read back from the store

- **WHEN** a second machine with the same store asks claims-of `sha256:aa…`
- **THEN** the report claim is among the attestations, as signed

### Requirement: A report counts only through a signer the policy lets claim one

A report SHALL count toward a verdict only where its signer is listed
in the policy with `report` among the kinds it may claim (116, rule
core.7). A report signed by any other signer SHALL be kept attached and
never counted.

#### Scenario: A host the policy does not let report

- **WHEN** the host `desk`, whose `kinds` lack `report`, signs a report of `srv`
- **THEN** the report stays attached, and no policy counts it

### Requirement: An environment's newest report

The core SHALL read an environment's newest report from its channel's
chain: the claims attached to every index the chain moved to, the
report claims of that environment that count under the policy, and of
those the one whose apply came last by the moment each names. The
target's convergence time SHALL order nothing (40, rule core.50).

#### Scenario: srv rolled back

- **WHEN** `srv` reported `0.4.0` from an apply at 10:00 and `0.3.0` from one at 11:00
- **THEN** its newest report names `0.3.0`

### Requirement: No newest report is absent or unreadable, never a guess

Where no report of the environment counts, its newest report SHALL be
absent naming the environment. A chain, or claims of an index, that
cannot be read SHALL make it unreadable naming what could not be read
(58).

#### Scenario: No report yet

- **WHEN** no counted report of `srv` is attached to any version its channel has held
- **THEN** its newest report is absent naming `srv`

### Requirement: Verify lists reports as it lists every claim

`timber verify` of a version SHALL list each report claim attached to
its index, with the signer that made it and the environment it names,
as it lists every attestation (69, 80).

#### Scenario: Verify after an apply

- **WHEN** a version applied to `srv` is verified
- **THEN** the answer lists the report claim, its signer and `srv`
