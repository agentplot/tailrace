# clause-coverage Specification

## Purpose
Keeps the suite honest about Parts A and B: every clause is cited by a
scenario or named in a register with the unit that will cite it and
why, checked in the gate on every run, with what waits reported in the
open (section 12, core.33, record 0012).

## Requirements

### Requirement: The suite knows the clauses of Parts A and B

The suite SHALL hold the numbers of every clause of Parts A and B, by
section, with the statement's revision they were read at. A clause the
statement adds or removes SHALL change this list in the commit that
adopts it.

#### Scenario: The clauses listed

- **WHEN** the suite's list of clauses is read
- **THEN** it holds every number from 1 to 132 exactly once, each under its section of Part A or Part B as agentplot/blueprints 157c828 places it, 132 after 60 in A.10, and names that revision as the one it was read at

### Requirement: A clause cited by nothing fails the suite

The coverage check SHALL fail naming every clause of Parts A and B that
no scenario cites and the register does not name.

#### Scenario: A clause neither cited nor registered

- **WHEN** the coverage check runs over scenarios and a register that between them leave clause 7 out
- **THEN** it fails naming clause 7

### Requirement: The register names each clause that waits, with its unit and why

The register SHALL name each clause no scenario cites yet, with the
unit of the plan that will cite it, or the phase of the plan while that
phase has no units carved, and the reason it waits. An entry naming
neither a unit nor a phase, or no reason, SHALL fail the check. The
unit or phase is a note for a reader, not a reference the check
resolves (core.33, section 12).

#### Scenario: An entry without its reason

- **WHEN** the register names a clause with a unit and no reason
- **THEN** the check fails naming the clause and the missing reason

#### Scenario: What the register holds in bolt 1

- **WHEN** the register is read as bolt 1 leaves it
- **THEN** it names 25 to 30, 32, 33, 36 to 39, 41, 42, 44 to 47, 49, 51 to 55, 63, 64, 94 to 101, 103, 104, 106 to 109, 112 to 115, 118, 119 and 125, each with its unit and reason, and no other clause; 42, 44, 45, 119, 55 and 112 name phase 2, whose units are not yet carved

### Requirement: A registered clause is reported as pending on every run

The coverage check SHALL print each registered clause as pending, with
its unit and reason, on every run whether it passes or fails, and
SHALL never count a pending clause as covered. Its output SHALL be
shown in the gate's log on a pass as on a failure.

#### Scenario: Pending on a pass

- **WHEN** the coverage check passes
- **THEN** its output lists every registered clause as pending with its unit, and the number of clauses cited and pending

### Requirement: A clause both cited and registered fails the suite

The coverage check SHALL fail naming every clause that a scenario cites
and the register still names, with the scenarios that cite it, so an
entry leaves the register in the commit that first cites its clause.

#### Scenario: A clause cited while still registered

- **WHEN** a scenario citing clause 25 is added and the register still names 25
- **THEN** the check fails naming clause 25 and that scenario

### Requirement: A citation names a clause of Parts A and B

The coverage check SHALL fail naming every scenario that cites a number
that is no clause of Parts A and B, every scenario that cites nothing,
and every register entry naming a number that is no clause.

#### Scenario: A citation of no clause

- **WHEN** a scenario cites clause 126
- **THEN** the check fails naming the scenario and 126

#### Scenario: A scenario that cites nothing

- **WHEN** a scenario carries no clause
- **THEN** the check fails naming the scenario

### Requirement: Coverage is a check of the gate

The coverage check SHALL be a check-fast task of the conformance
crate's own moon project, separate from its tests, and SHALL exit
non-zero on any finding (core.25).

#### Scenario: The gate runs coverage

- **WHEN** the check-fast tasks run
- **THEN** the conformance project's coverage task runs, and a finding fails the gate

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
