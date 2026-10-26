## ADDED Requirements

### Requirement: A cache has no lockfile
pinned SHALL answer absent for every checkout, naming the lockfile and
the projection, since moon's cache has no lockfile (137); sync fills it
from the landing's verdicts and never from a pin.

#### Scenario: Pins of a checkout
- **WHEN** the moon cache projection is asked for a checkout's pins
- **THEN** the answer is absent naming the lockfile, never refused and never unreadable
