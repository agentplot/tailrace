# Spec Delta

## MODIFIED Requirements

### Requirement: Scriptable per test

Each fake SHALL be set up by the test that uses it, with what its seam
holds (the revisions a branch has landed and their verdicts, the
outputs a build leaves and their identities and outcomes, the policy
for each audience, each channel with its component, and how each
channel moves, the signers and their identities, a projection's
ecosystem, an environment's channel head and what it runs), and SHALL
share that state with every copy the test hands to the core.

#### Scenario: A landing scripted for one test

- **WHEN** a test scripts the fake landing to have landed one revision on a branch, with two verdicts
- **THEN** landed answers yes for that revision and no for any other, and verdicts-of answers the two verdicts as scripted

#### Scenario: State seen through a copy

- **WHEN** a test keeps one handle to a fake store and puts bytes through a copy of it
- **THEN** the test's handle sees the bytes held

#### Scenario: The fake policy's channels

- **WHEN** a test scripts the fake policy with policies for `beta` of `lib`, which it says moves on each publish, `stable` of `lib` bound to `beta`, and `edge` of no declared component at one revision only
- **THEN** channels at that revision answers `beta` of `lib` moving on each publish, `edge` moving by promotion alone and `stable` of `lib` bound to `beta`, and at another revision only the two of `lib`, with the call recorded and faultable like policy-for

#### Scenario: A scripted following channel that takes prereleases

- **WHEN** a test scripts the fake policy with `nightly` moving on each publish with prereleases
- **THEN** channels answers `nightly` moving on each publish with prereleases

#### Scenario: The fake keeper's two components

- **WHEN** a test appends a promotion to `beta` of `lib` on the fake keeper
- **THEN** the chain of `beta` of `lib` holds it, and the chain of `beta` of `web` and of `beta` of no declared component are absent
