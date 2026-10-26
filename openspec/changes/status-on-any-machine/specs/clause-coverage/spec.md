## MODIFIED Requirements

### Requirement: The suite knows the clauses of Parts A and B

The suite SHALL hold the numbers of every clause of Parts A and B, by
section, with the statement's revision they were read at. A clause the
statement adds or removes SHALL change this list in the commit that
adopts it.

#### Scenario: The clauses listed

- **WHEN** the suite's list of clauses is read
- **THEN** it holds every number from 1 to 132 exactly once, each under its section of Part A or Part B as agentplot/blueprints 157c828 places it, 132 after 60 in A.10, and names that revision as the one it was read at

## ADDED Requirements

### Requirement: The register names the part of a cited clause that waits

Beside the clauses no scenario cites, the register SHALL name each part
of a clause a scenario cites whose behaviour a later unit builds, with
the part in a few words, the unit of the plan or the phase that will
build it, and why it waits. The coverage check SHALL print each such
part as pending on every run, SHALL fail naming a part whose clause no
scenario cites, which belongs in the register whole, and SHALL fail
naming a part with no unit or no reason (core.33, section 12).

#### Scenario: A part pending on a pass

- **WHEN** the coverage check passes with 132 cited and its environments named as a part waiting on the first target
- **THEN** its output lists that part as pending with its unit and reason, and 132 as cited

#### Scenario: A part of a clause nothing cites

- **WHEN** the register names a part of a clause no scenario cites
- **THEN** the check fails naming the clause and saying it waits whole

#### Scenario: The parts this unit leaves

- **WHEN** the register is read as this unit leaves it
- **THEN** it names as parts 132's environments, with the four counts of each repository's row, for the first target, and 132's notes, for the notes unit; 36 leaves the clauses that wait, 98 waits on the first target, and 114 still waits on bolt 5's first served projection, whose served layout will cite it
