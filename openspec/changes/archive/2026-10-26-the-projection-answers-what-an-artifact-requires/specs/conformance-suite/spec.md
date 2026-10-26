## ADDED Requirements

### Requirement: The projection's group holds requires and satisfying
The projection's group SHALL hold a projection whose ecosystem has a
manifest, below its marker, to answering requires of the rig's held
artifact with what the rig says its manifest requires, as spelled, and
satisfying of the rig's range with the versions the rig says satisfy
it, newest first. The scenarios cite 23, 50 and 58 until the suite's
statement list holds 140.

#### Scenario: Required as the manifest spells it
- **WHEN** a projection whose ecosystem has a manifest is asked what the rig's held artifact requires
- **THEN** it answers the names and ranges the rig says that artifact's manifest requires, spelled as the manifest spells them, and no other

#### Scenario: Satisfied by the ecosystem's own ordering
- **WHEN** a projection whose ecosystem has a manifest is asked which of the rig's versions satisfy the rig's range, the versions listed in neither the ecosystem's order nor spelling order
- **THEN** it answers the versions the rig says satisfy the range, newest first, and no other

### Requirement: The projection's group holds an ecosystem with no manifest to none
The projection's group SHALL hold a projection whose ecosystem has no
manifest to answering none to requires and to satisfying. The rig SHALL
say whether its ecosystem has a manifest; the manifest scenarios SHALL
be not applicable to a projection whose ecosystem has none, and the
none scenario to one that has one.

#### Scenario: No manifest, none
- **WHEN** a projection whose ecosystem has no manifest is asked what the rig's held artifact requires and which versions satisfy a range
- **THEN** both answers are none, an empty list, never absent, refused or unreadable

#### Scenario: Every projection runs them
- **WHEN** the projection group runs over the fake projection, the file, image and moon cache profiles
- **THEN** the two manifest scenarios pass over the fake and are not applicable to the three profiles, and the none scenario passes over the three profiles and is not applicable to the fake
