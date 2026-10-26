## MODIFIED Requirements

### Requirement: Scriptable per test

Each fake SHALL be set up by the test that uses it, with what its seam
holds (the revisions a branch has landed and their verdicts, the
outputs a build leaves and their identities and outcomes, the policy
for each audience and the component each channel belongs to, the
signers and their identities, a projection's ecosystem, an
environment's channel head and what it runs), and SHALL share that
state with every copy the test hands to the core.

#### Scenario: A landing scripted for one test

- **WHEN** a test scripts the fake landing to have landed one revision on a branch, with two verdicts
- **THEN** landed answers yes for that revision and no for any other, and verdicts-of answers the two verdicts as scripted

#### Scenario: State seen through a copy

- **WHEN** a test keeps one handle to a fake store and puts bytes through a copy of it
- **THEN** the test's handle sees the bytes held

#### Scenario: The fake policy's channels

- **WHEN** a test scripts the fake policy with policies for the channels `beta` and `stable`, `stable` bound to `beta`, and puts `beta` in the component `lib` at one revision
- **THEN** channels at that revision answers `beta` of `lib` and `stable` of `repository` bound to `beta`, and at another revision both of `repository`, with the call recorded and faultable like policy-for
