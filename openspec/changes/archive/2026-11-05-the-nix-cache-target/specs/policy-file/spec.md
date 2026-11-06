## MODIFIED Requirements

### Requirement: The environments' words are read

An environment SHALL be declared as `[environments.<name>]` where no
component is declared and `[environments.<component>.<name>]` where
components are. Its keys SHALL be C.10's words and no others: its
`channel`, `policy` (`auto` or `manual`, `auto` where unwritten),
`after`, a list of environment names of the same component, `target`,
and, where the environment is manual, `approvers`, the signers that may
say yes, read as a channel's `approvers` is (C.10, as ruled at
agentplot/blueprints 088430c). `target` SHALL be a table of exactly one
key, the profile of the environment's target, whose value is a table,
the target's own settings, which the profile SHALL carry as written and
read nothing of (C.10, 136). Any other `policy`, or a `channel` the file
does not declare, give or make, SHALL be unreadable naming the table. A
`target` naming no profile or two, or whose profile's value is not a
table, SHALL be unreadable naming the table and `target`. `approvers` on
an environment that is not manual SHALL be unreadable naming the table
and the key, since no one says yes to it. Any other key SHALL be
unreadable naming the table and the key, since a target's own settings
sit in the target's table and nowhere else (C.10).

#### Scenario: An environment naming its target

- **WHEN** `[environments.site]` names `channel = "stable"` and `target.nix_cache = { attic = "fleet:site" }`
- **THEN** the channels answer is read, and `site` makes no channel

#### Scenario: An environment with a target's own key

- **WHEN** `[environments.site]` names `channel = "stable"` and `bucket = "example-site"`
- **THEN** the channels are unreadable naming `[environments.site]` and `bucket`, and saying a target's settings sit in the target's table

#### Scenario: A target naming two profiles

- **WHEN** `[environments.site]` names `channel = "stable"` and a `target` table with the keys `nix_cache` and `bucket`
- **THEN** the channels and the environments are unreadable naming `[environments.site]` and `target`

#### Scenario: A target whose profile names no table

- **WHEN** `[environments.site]` names `target = { nix_cache = "fleet:site" }`
- **THEN** the channels and the environments are unreadable naming `[environments.site]` and `target`

#### Scenario: Approvers in a manual environment's table

- **WHEN** `[signers.ci]` is declared and `[environments.prod]` names `channel = "stable"`, `policy = "manual"` and `approvers = ["ci"]`
- **THEN** the channels answer lists `prod` bound to `stable`, and the policy for `prod` names `ci` as the one signer that may ask

#### Scenario: Approvers on an environment that moves on its own

- **WHEN** `[environments.site]` names `channel = "stable"` and `approvers = ["ci"]`, and no `policy`
- **THEN** the channels are unreadable naming `[environments.site]` and `approvers`

#### Scenario: An unknown policy word

- **WHEN** `[environments.prod]` names `policy = "sometimes"`
- **THEN** the channels are unreadable naming `[environments.prod]` and `sometimes`

## ADDED Requirements

### Requirement: The profile answers the environments the file declares

The policy file profile SHALL answer environments at a revision from the
file at that revision's tree and the table in force the channels are
read from, so the two never disagree (136, 65, rule core.22): each
environment, ordered by component and then name, the channel it follows,
the made one where it is manual or names `after`, whether it waits, its
`approvers` where manual, and its target's profile and settings, or
none.

#### Scenario: A cache environment in a component

- **WHEN** the file declares a component `swancloud-srv` and `[environments.swancloud-srv.cache]` names `channel = "beta"` and `target.nix_cache = { attic = "fleet:swancloud", closures = ["swancloud-srv"] }`
- **THEN** environments answers `cache` of `swancloud-srv`, following `beta` of `swancloud-srv`, not waiting, with the profile `nix_cache` and the settings `attic` and `closures` as written

#### Scenario: A manual environment follows its made channel

- **WHEN** `[environments.prod]` names `channel = "stable"`, `policy = "manual"` and `approvers = ["ci"]`
- **THEN** environments answers `prod` following the channel `prod`, waiting on `ci`

#### Scenario: An environment with no target

- **WHEN** `[environments.prod]` names `channel = "stable"` and no `target`
- **THEN** environments answers `prod` with no target

#### Scenario: No environment declared

- **WHEN** the file at a revision declares no environment
- **THEN** environments answers an empty list

### Requirement: The environments are unreadable as the channels are

A file that declares no environment, or no file, SHALL answer
environments with an empty list. A file whose environments' tables are
unreadable SHALL answer environments unreadable naming the table, as it
answers the channels.

#### Scenario: No policy file

- **WHEN** the revision's tree holds no `.config/timber.toml`
- **THEN** environments answers an empty list

#### Scenario: A broken environment table

- **WHEN** `[environments.prod]` names `policy = "sometimes"`
- **THEN** environments answers unreadable naming `[environments.prod]`, as the channels do
