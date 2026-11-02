## MODIFIED Requirements

### Requirement: The fake projection answers the pins a test sets
The fake projection SHALL answer pinned, for a checkout, the pins and
ranges a test set for that checkout, each an ecosystem name with a
version or with a range, in the order set, and absent naming the
lockfile for a checkout given none; it SHALL render artifacts as a
layout holding their metadata, recording what it was given, and write
nothing (137, 85).

#### Scenario: Pins set by the test
- **WHEN** a test sets two pins for one checkout and asks the fake projection for that checkout's pins and another's
- **THEN** the first answer is the two pins in the order set, and the second is absent naming the lockfile

#### Scenario: A pin and a range set by the test
- **WHEN** a test sets a checkout's pins to `app` 1.0.0 and a range `^1` of `lib`
- **THEN** pinned of that checkout answers both, as set and in the order set

#### Scenario: The fake renders artifacts
- **WHEN** the fake projection renders two artifacts at a place
- **THEN** the layout holds those two artifacts' metadata and no other, and the test reads back the place and the artifacts it was given
