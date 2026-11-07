## ADDED Requirements

### Requirement: The fake policy lists a channel that comes after environments

A test SHALL be able to script the fake policy with a bound channel
coming after the environments it names, and with a policy for that
channel requiring a report of each. The fake policy SHALL answer both
as scripted, recording each call and faultable like every other (B.5).

#### Scenario: second scripted after first

- **WHEN** a test scripts `second` bound to `first`'s channel and coming after `first`, with a policy requiring a report of `first` by `deployer`
- **THEN** channels answers `second` coming after `first`, and policy-for `second` answers that requirement, both calls recorded
