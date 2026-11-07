## ADDED Requirements

### Requirement: Apply converges one environment

`timber apply <environment>` SHALL ask the core's apply for that
environment at the governed branch's landed head, taking `--branch` as
promote does and `--component <name>` where the repository declares
components, and SHALL render the core's answer and nothing of its own,
with `--json` printing it in its machine-readable form (56, 57, rule
core.31). Absent, refused and unreadable SHALL be three exits (58).

#### Scenario: An environment applied

- **WHEN** `timber apply cache --component swancloud-srv` runs, and the cache environment's channel has its head at `0.4.0` with every claim its policy requires
- **THEN** it prints `cache` of `swancloud-srv`, its channel, `0.4.0` and its index, and the target's report or its unread reason, and exits 0

#### Scenario: A claim found wanting

- **WHEN** `timber apply prod` runs and the head of the channel `prod` follows lacks a claim its policy requires
- **THEN** it prints the refusal naming that claim, exits as a refusal, and no target was asked to apply

#### Scenario: Apply makes no store

- **WHEN** `timber apply` runs on a host with no bindings file and no given layout
- **THEN** the given layout is not made

### Requirement: Apply binds the repository's targets

`timber apply` SHALL be bound with the store opened and never made and
every target profile built, the repository's environments picking among
them (92), and SHALL keep no credential (rule core.21).

#### Scenario: The cache target bound

- **WHEN** `timber apply` runs in a repository whose environment names `target.nix_cache`
- **THEN** the Nix cache target is reached, with no credential read or written by timber
