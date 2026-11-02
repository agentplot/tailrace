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

### Requirement: A version a served layout lacks is rendered on the request
When a served layout's client asks for a version the layout lacks, the
core SHALL resolve that one version as a pin is resolved, judge it under
the policy `tailrace sync` with no argument would use in the checkout the
layout belongs to, found as a sync finds it: the consuming repository's
own where its file writes one, else the publisher's layout policy the
version's index carries, else the given (113, C.10); and, where the
policy admits it, hold that version and the family versions it
requires, by the loop a sync's render runs through every level, render
the layout holding what it held and those, and record them; it SHALL
answer the one version held, refused naming the claim found wanting,
absent, or unreadable with the source's own words (115, 50, 47, 99, 51,
137). A version refused, absent or unreadable SHALL leave the layout and
its record as they were. Only the one version asked, and the names it
requires that the layout does not meet, SHALL be read from the store.

#### Scenario: A miss the policy admits
- **WHEN** a layout holding `1.4.0` of a name is asked for `1.3.0`, which the store names and the policy admits
- **THEN** the layout holds `1.3.0` and `1.4.0`, the record names both, and the store was asked about `1.3.0` alone

#### Scenario: A miss that requires a name the layout lacks
- **WHEN** a layout holding `fam-a` `1.4.0` is asked for `fam-a` `1.3.0`, which the policy admits and which requires `fam-b` at `^2.0.0`, of which the store names `2.0.0` and `2.2.0`, both admitted, and the layout holds none
- **THEN** the layout and its record hold `fam-a` `1.3.0` and `1.4.0` and `fam-b` `2.2.0`, and the store was asked about `fam-a` `1.3.0` and the versions of `fam-b` alone

#### Scenario: A miss the policy refuses
- **WHEN** a layout is asked for a version whose index lacks the claim the policy requires first
- **THEN** the answer names the version and that claim as wanting, and the layout and its record are unchanged

#### Scenario: A miss the store does not name
- **WHEN** a layout is asked for a version the store holds no name for
- **THEN** the answer is absent naming the name, never refused, and the layout is unchanged

#### Scenario: A miss while the store is unreachable
- **WHEN** a layout is asked for a version while the store cannot be reached
- **THEN** the answer is unreadable carrying the store's own words, and the layout is unchanged

### Requirement: One render of a layout at a time
A sync and a render on a miss SHALL each hold the layout's own lock, in
the checkout's git directory beside its record, from the first read of
the record to the last write, so two renders of one layout never
interleave and the record always names what the layout holds (48, 62).

#### Scenario: Two misses at once
- **WHEN** two requests for two versions a layout lacks are answered at once, and the policy admits both
- **THEN** the layout and its record hold both versions afterwards

#### Scenario: A sync during a miss
- **WHEN** a sync of a checkout runs while a miss renders into its layout
- **THEN** one waits for the other, and the record afterwards names exactly what the layout holds

### Requirement: A served layout's address is the core's, with the repository in the path
The address a served layout is reached at SHALL be
`http://127.0.0.1:7420/<owner>/<name>/<ecosystem>/`, the core's one name
and port, the owner and name the consuming repository's landing remote
gives it, and the ecosystem's name, the same on every host and in every
checkout of the repository, naming no directory and no store (114, 49,
core.18).

#### Scenario: Two remotes of one repository
- **WHEN** one checkout's landing remote is `https://example.com/agentplot/consumer.git` and another's `git@example.com:agentplot/consumer`
- **THEN** both checkouts' Cargo layout is served at `http://127.0.0.1:7420/agentplot/consumer/cargo/`
