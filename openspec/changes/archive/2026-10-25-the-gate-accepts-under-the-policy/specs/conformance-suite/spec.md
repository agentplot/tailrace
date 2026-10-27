# Spec Delta

## MODIFIED Requirements

### Requirement: Scenarios sit in groups that later units only add to

The suite SHALL hold its scenarios in groups: one per seam of Part B,
one for the binding, and one per part of the core (version, publish,
verify, status, surfaces, gate, ledger, cache, record, layouts). Each
group SHALL end at a marker, and a later unit SHALL add its scenarios
below its group's marker and change no scenario above it. A group with
nothing yet SHALL exist with its marker.

#### Scenario: The gate's group waits for its unit

- **WHEN** the gate's group is listed once bolt 4's gate unit has landed
- **THEN** its marker is in place and below it are that unit's scenarios, among them those playing S3, S5 and S9, and they cite 25 to 30, 94 to 96, 107 and 108, none of which the register still names

#### Scenario: Every seam has a group

- **WHEN** the suite's groups are listed
- **THEN** there is one for each of the store, the build, the landing, the signer, the policy, a projection, a target and the keeper
