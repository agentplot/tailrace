## ADDED Requirements

### Requirement: A required claim names the artifacts it is asked of

Each claim the policy's answer requires, for every audience, SHALL name
the artifacts it is asked of: every artifact a version names, or a
built artifact alone, leaving a rendered artifact unasked (82, 134), so
the core reads it from the policy and decides none of it (core.7). Its
machine-readable form SHALL read a required claim naming none as asked
of every artifact, and write one so asked as it wrote it before (57).

#### Scenario: A required claim written and read back

- **WHEN** a claim required of built artifacts alone, and one required of every artifact, are written in their machine-readable form and read back
- **THEN** each is the same value, the first naming built artifacts and the second naming nothing of the artifacts it is asked of

#### Scenario: A layout-policy claim written before

- **WHEN** a layout-policy claim's body whose required claims name nothing of the artifacts they are asked of is read
- **THEN** each required claim reads as asked of every artifact
