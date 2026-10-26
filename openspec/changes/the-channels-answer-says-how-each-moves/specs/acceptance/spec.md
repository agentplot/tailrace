# Spec Delta

## ADDED Requirements

### Requirement: The channels that follow each publish come from the policy seam

The gate SHALL read, at the governed branch's landed head, which
channels follow each publish from the policy seam's channels answer
alone: each channel of the component published that the answer says
moves on each accepted publish (95, 125, 131). The gate SHALL read no
key of the policy file itself, and SHALL find the signer it accepts as
through the policy for acceptance and for each channel listed (core.1,
core.2, core.7).

#### Scenario: beta follows each publish with no line written

- **WHEN** the policy at the branch's head writes no channel in a repository of one component, and the gate accepts a version a publish told it of
- **THEN** `beta` follows that publish, and `stable` does not

#### Scenario: Two components' beta

- **WHEN** the repository declares `lib` and `web` and the gate accepts a version of `lib` a publish told it of
- **THEN** `beta` of `lib` follows that publish and `beta` of `web` does not

#### Scenario: A channels answer that cannot be read

- **WHEN** the policy's channels answer is unreadable at the branch's head
- **THEN** the gate's answer is unreadable with the policy's words, and nothing is accepted or moved

### Requirement: The gate moves a following channel only as its promoter

The gate SHALL move a channel that follows each publish only where the
signer it accepts as is among the promoters the channel's policy names;
otherwise that channel's answer SHALL be refused naming the promotion
and its promoters, and the acceptance SHALL stand (32, 9, core.7).

#### Scenario: A following channel the gate may not promote

- **WHEN** a hand-declared channel moves on each publish and its policy names `release`, not the gate's signer, as its promoter
- **THEN** the version is accepted, the channel does not move, and its answer is refused naming the promotion and `release`

## REMOVED Requirements

### Requirement: A channel that follows each publish is declared in the policy file

**Reason**: The gate read `[channels.<name>]` tables of the policy file
itself; 131 has it learn a channel only through the policy seam, and
C.10 gives every component `beta`, which follows each publish with no
line written.

**Migration**: The channels that follow each publish are those the
policy seam's channels answer says move on each publish (this change's
`The channels that follow each publish come from the policy seam`); the
policy file's reading of `on_each_publish`, and the refusal of a
following channel that is bound, move to the `policy-file` and
`channel-movement` capabilities.
