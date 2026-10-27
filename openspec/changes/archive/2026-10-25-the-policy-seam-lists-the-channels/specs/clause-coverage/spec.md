## MODIFIED Requirements

### Requirement: The suite knows the clauses of Parts A and B

The suite SHALL hold the numbers of every clause of Parts A and B, by
section, with the statement's revision they were read at. A clause the
statement adds or removes SHALL change this list in the commit that
adopts it.

#### Scenario: The clauses listed

- **WHEN** the suite's list of clauses is read
- **THEN** it holds every number from 1 to 131 exactly once, each under its section of Part A or Part B as agentplot/blueprints 673d334 places it, and names that revision as the one it was read at
