## MODIFIED Requirements

### Requirement: The suite knows the clauses of Parts A and B

The suite SHALL hold the numbers of every clause of Parts A and B, by
section, with the statement's revision they were read at. A clause the
statement adds or removes SHALL change this list in the commit that
adopts it.

#### Scenario: The clauses listed

- **WHEN** the suite's list of clauses is read
- **THEN** it holds every number from 1 to 143 exactly once, each under its section of Part A or Part B as agentplot/blueprints 90cf023 places it, 141, 142 and 143 after 13 in A.2, and names that revision as the one it was read at

## ADDED Requirements

### Requirement: The clauses the statement added through 143 wait named

Each of 141, 142 and 143 that no scenario cites SHALL be named in the
register in the commit that adopts the statement holding it: 141 with
the unit `every-claim-takes-the-catalogued-kind-that-carries-it`, and
142 and 143 with phase 2's keyless signing, each with its reason (rule
core.33, section 12).

#### Scenario: The register after the statement is adopted

- **WHEN** the coverage check runs after the commit that adopts agentplot/blueprints 90cf023
- **THEN** 141 is pending with its unit, 142 and 143 are pending with phase 2, each with its reason, and the check passes
