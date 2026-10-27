## ADDED Requirements

### Requirement: The gate told by a publish's answer over the fakes

The gate's group SHALL hold a scenario in which the core's publish
offers a version and answers, with nothing accepted and nothing asked
of the ledger, and the gate, told of that answer with the index it
names, accepts that index and moves each channel that follows each
publish; and a scenario in which the told index is not what the
version's name resolves to, answered unreadable with nothing written.
Each SHALL cite 95 and 130, and 130 SHALL leave the register in the
commit that cites it (core.33).

#### Scenario: The core starts nothing after an offer

- **WHEN** the suite publishes a version over the fakes and then tells the gate of the publish's answer
- **THEN** the fake keeper and the fake store record no acceptance, no promotion and no append before the gate is told, and after it the acceptance is attached to the index the publish's answer named and `beta`'s chain holds the promotion

#### Scenario: A told index the name does not resolve to

- **WHEN** the suite tells the gate of a publish's answer whose index differs from the one the version's name resolves to
- **THEN** the gate's answer is unreadable naming both digests, and the fake store and the fake keeper record no write

#### Scenario: 130 cited and not registered

- **WHEN** the coverage check runs
- **THEN** 130 is cited by the gate's group and is not in the register
