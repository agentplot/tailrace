## ADDED Requirements

### Requirement: A range with no pin resolves to the newest version the policy admits

For a name the projection's pinned answers by a range, sync SHALL list
the versions the store names it at, locate and judge each under the
layout's policy as it judges a pin, before anything is written, and
have the projection render the newest admitted version within the
range, by the ecosystem's own rules (137, 138, 23, 47). The layout's
record SHALL keep the version held with its range (61).

#### Scenario: A first install's range
- **WHEN** a checkout declares `eco` `site` by the range `1.` with no pin, the store names `site` at 1.2.0, 1.4.0 and 2.0.0, and the policy admits all three
- **THEN** the layout holds 1.4.0, the answer names the range and 1.4.0 with its index's and artifact's digests, and the record keeps 1.4.0 with its range

### Requirement: A range's refusals and absence are answered

Every listed version of a range's name the policy refuses SHALL be
answered naming the claim found wanting, and kept in the record (99,
61). A name the store names at no version SHALL be answered absent
naming the name and the range. Where render holds no version within the
range, the range SHALL be answered absent naming the name, the range and
how many listed versions the policy refused.

#### Scenario: The newest within the range refused
- **WHEN** the store names `site` at 1.2.0 and 1.4.0, the range is `1.`, and 1.4.0's index lacks the acceptance claim the policy requires
- **THEN** the layout holds 1.2.0, and the answer names 1.4.0 refused naming acceptance as the claim found wanting

#### Scenario: Nothing within the range admitted
- **WHEN** every version the store names `site` at within the range is refused by the policy
- **THEN** the range is answered absent naming `site`, the range and the number refused, each refused version is answered naming its claim, and the layout holds no version of `site`

#### Scenario: A range of a name the store never named
- **WHEN** a checkout declares by a range a name the store names at no version
- **THEN** the range is answered absent naming the name and the range, never refused, and every other pin is held all the same

#### Scenario: A projection that holds what it was not given
- **WHEN** the fake projection, asked to render a range, answers that it holds a version that was no candidate
- **THEN** the layout is answered unreadable naming that version, and is kept as it was

## MODIFIED Requirements

### Requirement: A layout holds the lockfile's versions and no other
For each bound projection whose pinned answers pins or ranges for the
checkout, sync SHALL render that ecosystem's layout from the pinned
versions the policy admits, and for each range the newest version within
it the policy admits, and no other, and SHALL ask the store about a
pinned name only for what its pin names (50, S4). A projection whose
pinned answers absent SHALL be passed over, and no layout of it is
rendered or recorded.

#### Scenario: S4, a lockfile's versions and no other
- **WHEN** a checkout whose lockfile pins two family versions is synced, the store holds a third version of the same name, and the policy admits all three
- **THEN** the layout holds exactly the two pinned versions, the store was asked nothing about the third, and the layout's place names no host

#### Scenario: No lockfile, no layout
- **WHEN** a checkout is synced and a bound projection answers pinned absent
- **THEN** no layout of that ecosystem is rendered, recorded or named in the answer as refused or unreadable
