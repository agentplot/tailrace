## MODIFIED Requirements

### Requirement: A layout holds the lockfile's versions and no other
For each bound projection whose pinned answers pins or ranges for the
checkout, sync SHALL render that ecosystem's layout from the pinned
versions the policy admits, the version held for each range, and the
versions the family requires of what is held, and no other, and SHALL
ask the store only for what the pins name, and for the versions of a
name only where a range is not met by what the layout holds (50, S4,
137). A projection whose pinned answers absent SHALL be passed over, and
no layout of it is rendered or recorded.

#### Scenario: S4, a lockfile's versions and no other
- **WHEN** a checkout whose lockfile pins two family versions that require nothing of the family is synced, the store holds a third version of the same name, and the policy admits all three
- **THEN** the layout holds exactly the two pinned versions, the store was asked nothing about the third, and the layout's place names no host

#### Scenario: No lockfile, no layout
- **WHEN** a checkout is synced and a bound projection answers pinned absent
- **THEN** no layout of that ecosystem is rendered, recorded or named in the answer as refused or unreadable

## ADDED Requirements

### Requirement: A range is held at the newest admitted version within it
For a range pinned answers, sync SHALL list the name's versions in the
store, keep those the projection answers satisfy the range, and judge
each under the layout's policy, newest first by the projection's
ordering, until one is admitted, and hold that one; none where the listing is empty or none
within it is admitted (137, 138, 140, 23). Each version judged SHALL be
answered as a pin is, held or refused naming the claim found wanting,
and a refused one SHALL count against the sync only where its range
holds no version (99).

#### Scenario: The newest admitted within a range
- **WHEN** a checkout's manifest declares `site` by a range within which the store names 1.0.0 and 1.1.0, which the policy admits, and 1.2.0, which it refuses, and names 2.0.0 outside it
- **THEN** the layout holds `site` 1.1.0 alone, sync's answer names the range holding 1.1.0 and 1.2.0 refused naming the claim found wanting, 2.0.0 was never judged, and the sync answers no refusal

#### Scenario: A range the store names nothing within
- **WHEN** a checkout's manifest declares a range of a name the store's listing answers with the empty list
- **THEN** the layout holds no version of it, and sync's answer names the range holding none

### Requirement: What a held version requires is held through every level
For every version a layout holds, sync SHALL ask the projection what it
requires of the family and hold each range so answered by the same rule,
level by level, until nothing held requires a range the layout does not
meet; a range SHALL be met where the layout holds a version of its name
the projection answers satisfies it (137, 140). Where the projection
answers requires absent, the version SHALL require nothing. Every version SHALL be
judged before anything is written, in a fixed order, so the same store
under the same policy holds the same versions (48). The core SHALL never
read a manifest itself (core.1).

#### Scenario: A pinned version's dependencies, two levels down
- **WHEN** a checkout's lockfile pins `app` 1.0.0, which requires `lib` by a range the store names 1.0.0 and 1.1.0 within, and `lib` 1.1.0 requires `base` by a range the store names 0.3.0 within
- **THEN** the layout holds `app` 1.0.0, `lib` 1.1.0 and `base` 0.3.0, and sync's answer names `lib`'s range as required by `app` 1.0.0 and `base`'s by `lib` 1.1.0

#### Scenario: A range met by what the layout holds
- **WHEN** a checkout's lockfile pins `app` 1.0.0 and `lib` 1.0.0, and `app` 1.0.0 requires `lib` by a range 1.0.0 is within
- **THEN** the layout holds `app` 1.0.0 and `lib` 1.0.0, and the store was asked for no listing of `lib`
