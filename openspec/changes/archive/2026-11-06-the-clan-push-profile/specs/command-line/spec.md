## ADDED Requirements

### Requirement: Apply's answer carries the report and the moves after it

`timber apply`'s answer SHALL carry, beside what the core's apply
answers, the report claim's digest or the answer given in its place.
Where the host's own steps tell its gate, it SHALL carry each channel
that comes after the environment with the gate's answer for it; where
a job tells the gate, the job's words as written. Each part SHALL be in
the core's one form in the JSON answer (57).

#### Scenario: srv applied on a deployer that is its gate

- **WHEN** `timber apply srv --json` converges `srv`, and one channel comes after `srv`
- **THEN** the JSON names the report claim's digest and that channel's move, and the exit is 0

### Requirement: Apply's exit is the worst of its parts

The exit code SHALL be the worst of the apply's answer, the report's,
and each channel's, a channel held for reports that disagree counting
as refused (58).

#### Scenario: A report the target answers unread

- **WHEN** the update converges but the target's report is unread
- **THEN** the answer carries the unread report in place of the claim, no channel is asked to move, and the exit is the unreadable exit

## MODIFIED Requirements

### Requirement: Apply converges one environment

`timber apply <environment>` SHALL ask the core's apply for that
environment at the governed branch's landed head, taking `--branch` as
promote does and `--component <name>` where the repository declares
components, and SHALL render the core's answer and nothing of its own,
with `--json` printing it in its machine-readable form (56, 57, rule
core.31). Absent, refused and unreadable SHALL be three exits (58).

#### Scenario: An environment applied

- **WHEN** `timber apply cache --component swancloud-srv` runs, and the cache environment's channel has its head at `0.4.0` with every claim its policy requires
- **THEN** it prints `cache` of `swancloud-srv`, its channel, `0.4.0` and its index, the target's report and the report claim's digest, and exits 0

#### Scenario: A claim found wanting

- **WHEN** `timber apply prod` runs and the head of the channel `prod` follows lacks a claim its policy requires
- **THEN** it prints the refusal naming that claim, exits as a refusal, and no target was asked to apply

#### Scenario: Apply makes no store

- **WHEN** `timber apply` runs on a host with no bindings file and no given layout
- **THEN** the given layout is not made
