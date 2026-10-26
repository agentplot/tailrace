## MODIFIED Requirements

### Requirement: The fake projection answers the pins a test sets
The fake projection SHALL answer pinned, for a checkout, the pins and
ranges a test set for that checkout, in the order set, and absent naming
the lockfile for a checkout given none; it SHALL render artifacts as a
layout holding their metadata, recording what it was given, and write
nothing; and given a range and its candidates, it SHALL hold, of the
candidates whose version begins with the range's text, the newest by the
version's dot-separated numeric parts, or none (137, 85).

#### Scenario: Pins set by the test
- **WHEN** a test sets two pins and a range for one checkout and asks the fake projection for that checkout's pins and another's
- **THEN** the first answer is the two pins and the range in the order set, and the second is absent naming the lockfile

#### Scenario: The fake renders artifacts
- **WHEN** the fake projection renders two artifacts at a place
- **THEN** the layout holds those two artifacts' metadata and no other, and the test reads back the place and the artifacts it was given

#### Scenario: The fake holds the newest within a range
- **WHEN** the fake projection renders the range `1.` with candidates at 1.2.0, 1.10.0 and 2.0.0
- **THEN** the layout holds 1.10.0 of that name and no other version of it
