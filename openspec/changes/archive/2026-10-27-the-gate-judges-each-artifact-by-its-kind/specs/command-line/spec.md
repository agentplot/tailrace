## ADDED Requirements

### Requirement: Policy shows which artifacts a claim is asked of

`tailrace policy` SHALL show, beside each required claim the policy asks
of built artifacts alone, that it is asked of built artifacts alone, and
nothing beside a claim asked of every artifact, as the policy's answer
carries it (82, 134, core.31); `--json` SHALL print it as the core's
answer value for value.

#### Scenario: A verdict asked of built artifacts

- **WHEN** `tailrace policy` runs at a head whose `[acceptance]` requires provenance from `laptop` and a verdict of `cli:test` from `box`
- **THEN** it shows `provenance by laptop` as written and `verdict of cli:test by box` marked asked of built artifacts alone, and the exit is 0
