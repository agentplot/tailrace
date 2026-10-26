## ADDED Requirements

### Requirement: Each required claim is answered with the artifacts it is asked of

The profile SHALL answer each claim `[acceptance]` or a channel requires
with the artifacts it is asked of, read from its kind alone: a
provenance, an acceptance and a promotion of every artifact (7, 134); a
verdict, a test, a review, a rebuild and a carried claim of a built
artifact alone, since a rendered artifact has no build (134). The file
SHALL gain no key for it, and `[layout]`'s claims SHALL be asked of
every artifact.

#### Scenario: Acceptance requires two claims of two kinds

- **WHEN** `[acceptance]` requires provenance from `laptop`, then a verdict of the check `cli:test` from `box`
- **THEN** the acceptance policy answers the provenance asked of every artifact and the verdict asked of built artifacts alone, in that order

#### Scenario: A given channel's line

- **WHEN** `[channels.stable]` requires a rebuild from `box`
- **THEN** the policy for `stable` answers the gate's acceptance asked of every artifact, then the rebuild asked of built artifacts alone

#### Scenario: The one host's own given acceptance

- **WHEN** `[signers]` names one signer of kind `provenance` and none of `acceptance`, and the policy for `beta` is asked
- **THEN** it requires that signer's acceptance, asked of every artifact

#### Scenario: A layout's claims

- **WHEN** `[layout]` requires a verdict of `cli:test` from `box`
- **THEN** the layout policy answers that verdict asked of every artifact

#### Scenario: A key naming the artifacts

- **WHEN** an entry of `[acceptance]`'s `require` names a key besides `kind`, `check` and `signers`
- **THEN** the answer is unreadable, naming the file and the table, as for any other unknown key
