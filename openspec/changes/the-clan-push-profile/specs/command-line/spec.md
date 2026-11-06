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
