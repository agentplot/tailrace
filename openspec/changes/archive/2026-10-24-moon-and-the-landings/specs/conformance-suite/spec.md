## ADDED Requirements

### Requirement: A verdict's signer by the landing's own name

The landing's group SHALL hold a landing that records verdicts to
answer each verdict's signer by the name the landing gives that signer,
the one name tailrace's policy names it by, and never by a second name
(104). For a landing that records no verdicts, the scenario SHALL be
not applicable. With this scenario, 104 SHALL leave the register
(core.33).

#### Scenario: Two signers, each by its own name

- **WHEN** a revision is landed with one verdict recorded by the signer `box` and another by the signer `forge`, and its verdicts are read
- **THEN** the verdict recorded by `box` names `box` and every signature its envelope holds is by `box`, the one recorded by `forge` names `forge` likewise, and no verdict names a signer that recorded none

#### Scenario: 104 no longer waits

- **WHEN** the coverage check runs
- **THEN** 104 is cited by the landing's group and is not in the register
