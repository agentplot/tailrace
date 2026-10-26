## ADDED Requirements

### Requirement: A cache has no manifest
requires SHALL answer none, an empty list, for every artifact, and
satisfying SHALL answer none for any range and any versions, since
moon's cache has no manifest; neither reads anything (140).

#### Scenario: What an artifact requires
- **WHEN** the moon cache projection is asked what an artifact requires
- **THEN** the answer is none, an empty list, never absent, refused or unreadable

#### Scenario: Versions that satisfy a range
- **WHEN** the moon cache projection is asked which of some versions satisfy a range
- **THEN** the answer is none, an empty list, never absent, refused or unreadable
