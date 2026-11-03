## ADDED Requirements

### Requirement: An image has no lockfile
pinned SHALL answer absent for every checkout, naming the lockfile and
the projection, since an image has no lockfile (137).

#### Scenario: Pins of a checkout
- **WHEN** the image projection is asked for a checkout's pins
- **THEN** the answer is absent naming the lockfile, never refused and never unreadable
