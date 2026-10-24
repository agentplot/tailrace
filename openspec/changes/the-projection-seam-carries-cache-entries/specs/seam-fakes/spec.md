# Spec Delta

## ADDED Requirements

### Requirement: The fake projection packs cache entries or none

The fake projection SHALL pack an output given a build identity and no
version as a cache entry, whose bytes name the build identity and the
output's files and no ecosystem, name or version, and SHALL render
cache entries as a layout holding their build identities (84, 85). A
test SHALL be able to script it to pack no cache entries, after which
both answer absent for one.

#### Scenario: A cache entry packed by the fake

- **WHEN** the fake projection packs one output twice at one build identity with no version, and once at another
- **THEN** the first two answers are the same bytes with no metadata, the third is other bytes, and none of them names an ecosystem, a name or a version

#### Scenario: The fake renders cache entries

- **WHEN** the fake projection renders two cache entries
- **THEN** the layout holds those two build identities and no version

#### Scenario: The fake scripted to pack none

- **WHEN** a test scripts the fake projection to pack no cache entries
- **THEN** a pack with no version and a render of cache entries each answer absent, and a pack with a version answers as before
