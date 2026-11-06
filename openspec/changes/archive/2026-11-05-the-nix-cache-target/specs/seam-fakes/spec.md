## ADDED Requirements

### Requirement: The fake policy answers the environments a test sets

The fake policy SHALL answer environments, at a revision, the
environments a test set for that revision, ordered by component and then
name, each with its channel, whether it waits, its approvers and its
target's profile and settings, and an empty list at a revision given
none; the call SHALL be recorded and faultable as policy-for and
channels are (136).

#### Scenario: Environments set by the test

- **WHEN** a test sets `cache` of `web`, following `beta` of `web` with the target `nix_cache` and settings, at one revision, and asks environments at that revision and another
- **THEN** the first answer is `cache` of `web` as set, and the second an empty list

#### Scenario: Environments made to fail

- **WHEN** a test scripts environments to answer unreadable
- **THEN** environments answers unreadable carrying the scripted words, and the call is recorded

### Requirement: The fake target answers the head it is handed

The fake target SHALL answer desired with the head the core hands it,
as a target that is pushed to does (88), and SHALL hold, per
environment, what it runs, or that it cannot be read, as a test sets.
Every call SHALL be recorded.

#### Scenario: The head handed

- **WHEN** the fake target's desired is asked with a head answering `i2`, and with one answering the channel empty
- **THEN** the first answers `i2` and the second absent

### Requirement: The fake target holds no marker where a test says so

Where a test says the fake target holds no marker, current and report
SHALL answer unreadable naming why after any apply, never the digest
applied (119).

#### Scenario: A fake that holds no marker

- **WHEN** a test says the fake target holds no marker, and a version is applied to an environment
- **THEN** current and report answer unreadable naming why, and the apply is recorded

### Requirement: The fake target reads only the settings a test lets it

The fake target SHALL read only the settings keys a test lets it read,
answering desired, current and apply unreadable naming the first other
key of an environment's settings, with nothing applied (45).

#### Scenario: A settings key the fake does not read

- **WHEN** a test lets the fake read `cache`, and an environment's settings hold `cache` and `token`
- **THEN** apply answers unreadable naming `token`, and nothing is applied
