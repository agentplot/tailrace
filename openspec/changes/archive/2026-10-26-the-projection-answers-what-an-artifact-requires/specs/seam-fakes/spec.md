## ADDED Requirements

### Requirement: The fake projection answers the requirements a test sets
The fake projection SHALL answer requires, for an artifact, the
requirements a test set for that artifact's digest, in the order set,
and absent naming the manifest of that artifact's name and version for
an artifact given none; it reads no bytes (140). requires SHALL be
recorded and can be faulted, as every operation of a fake is.

#### Scenario: Requirements set by the test
- **WHEN** a test sets two requirements for one artifact and asks the fake projection what that artifact and another require
- **THEN** the first answer is the two requirements in the order set, and the second is absent naming the other artifact's manifest

#### Scenario: requires faulted and recorded
- **WHEN** a test faults requires once and asks it
- **THEN** it answers the fault, and the fake's calls record requires with the artifact it was given

### Requirement: The fake projection orders versions by Semantic Versioning
The fake projection SHALL answer satisfying with the versions given
that satisfy the range by Semantic Versioning, newest first by its
precedence (23, 140). A range it cannot read SHALL be refused carrying
the range parser's own words (59), and a version it cannot read SHALL
satisfy no range. satisfying SHALL be recorded and can be faulted.

#### Scenario: Newest first, not by spelling
- **WHEN** the fake projection is asked which of `1.9.0`, `2.0.0`, `1.10.0`, `1.2.0` and `1.1.9` satisfy `^1.2`
- **THEN** it answers `1.10.0`, `1.9.0`, `1.2.0`, in that order

#### Scenario: A range it cannot read
- **WHEN** the fake projection is asked which versions satisfy a range that is not a Semantic Versioning range
- **THEN** the answer is refused carrying the parser's words, never absent and never an empty list

#### Scenario: satisfying faulted and recorded
- **WHEN** a test faults satisfying once and asks it
- **THEN** it answers the fault, and the fake's calls record satisfying with the range and versions it was given
