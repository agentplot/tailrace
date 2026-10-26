# Spec Delta

## MODIFIED Requirements

### Requirement: Each command calls the one core and renders its answer

`tailrace publish`, `tailrace verify`, `tailrace status`, `tailrace
sync`, `tailrace push`, `tailrace rebuild`, `tailrace promote` and
`tailrace policy` SHALL each turn their arguments into one question to
the core and render the core's answer; no command SHALL read a seam, a
file of the record or the history itself, or compute a version, a
digest, a signer or a channel of its own (56, core.31).

#### Scenario: The same answer through the command line

- **WHEN** a version is published over the fakes through the command line with `--json`, and the same question is put to the core directly over an identical binding
- **THEN** the printed JSON is the core's answer, value for value

## ADDED Requirements

### Requirement: Policy shows the policy in force

`tailrace policy` SHALL take `--branch` (by default `main`) and
`--json`, and SHALL print, at the branch's landed head, the policy for
acceptance and for a layout and every channel the channels answer
lists, the givens beside the repository's own, each with its component,
how it moves, whether a following channel takes prereleases, the claims
its policy requires, its promoters and who may ask for it (C.10, 131,
82); and beneath that table, each given held off, with how it would
move and the line that would switch it on, as the policy's held answer
gives them, so a reader sees what one line would give them (C.10, as
amended at agentplot/blueprints b1267d7). Each part SHALL carry its own
answer, so an absent layout, a channel whose policy cannot be read, or
an unreadable held answer leaves the rest shown (58). It
SHALL be bound as status is, asking the landing and the policy alone,
and SHALL write nothing (60, core.32). It SHALL exit 0 whenever an
answer is made, and with the answer's own code where the branch's head
cannot be resolved or has not landed.

#### Scenario: The givens beside a line

- **WHEN** `tailrace policy --json` runs in a repository whose landed head holds a policy file with `[acceptance]` and `[channels.stable]` requiring a rebuild
- **THEN** the printed JSON is the core's answer value for value, listing `beta` moving on each publish with releases alone and `stable` bound to `beta` requiring the gate's acceptance and then the rebuild, and the exit is 0

#### Scenario: The givens held off, beneath the table

- **WHEN** `tailrace policy` runs at a head whose file writes no channel and declares no component
- **THEN** beneath `beta` and `stable` it shows `alpha`, each publish with prereleases too, switched on by `[channels.alpha]`, and `lts`, bound to `stable`, switched on by `[channels.lts]`, and with `--json` the same held answer value for value with the core's

#### Scenario: A given switched on leaves the held part

- **WHEN** `tailrace policy` runs at a head whose file holds `[channels.alpha]`
- **THEN** `alpha` is shown among the channels in force and not beneath them, and `lts` alone is shown held off

#### Scenario: alpha and its approvers shown

- **WHEN** `tailrace policy` runs at a head whose file holds `[channels.alpha]` and `[channels.stable]` naming `approvers = ["ci"]`
- **THEN** it shows `alpha` moving on each publish with prereleases, and `stable` with `ci` as who may ask

#### Scenario: No layout policy

- **WHEN** `tailrace policy` runs at a head whose file holds no `[layout]`
- **THEN** the layout is shown absent naming the audience, the channels are shown, and the exit is 0

#### Scenario: A head not landed

- **WHEN** the landing answers that the branch has not landed its head
- **THEN** it exits 1 naming the branch and the revision, and the policy is not asked
