## ADDED Requirements

### Requirement: A file has no lockfile
pinned SHALL answer absent for every checkout, naming the lockfile and
the projection, since a file has no lockfile (137).

#### Scenario: Pins of a checkout
- **WHEN** the file projection is asked for a checkout's pins
- **THEN** the answer is absent naming the lockfile, never refused and never unreadable
