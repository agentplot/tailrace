# Spec Delta

## MODIFIED Requirements

### Requirement: Each command calls the one core and renders its answer

`tailrace publish`, `tailrace verify`, `tailrace status`, `tailrace
sync`, `tailrace push` and `tailrace rebuild` SHALL each turn their
arguments into one question to the core and render the core's answer; no
command SHALL read a seam, a file of the record or the history itself,
or compute a version, a digest or a signer of its own (56, core.31).

#### Scenario: The same answer through the command line

- **WHEN** a version is published over the fakes through the command line with `--json`, and the same question is put to the core directly over an identical binding
- **THEN** the printed JSON is the core's answer, value for value

## ADDED Requirements

### Requirement: Push takes a landed revision

`tailrace push [revision]` SHALL take the landed revision whose
archives it pushes, by default the local branch's head, `--branch` (by
default `main`) and `--json`; it SHALL be bound by the host's bindings
file, needing no `[signing]`, and SHALL answer each build identity as
written, held already, absent, or the refusal or unreadable answer its
write gave, exiting 0 when the only build identities not written or
held are absent (54, 57).

#### Scenario: A push with one archive missing

- **WHEN** `tailrace push --json` runs over the fakes for a landed revision whose verdicts pass on two build identities, one of which has no archive on the host
- **THEN** the printed JSON names one as written with its digest and the other as absent, its JSON is the core's answer value for value, and the exit is 0

#### Scenario: A push on an unbound host

- **WHEN** the built binary runs `tailrace push` on a host with no bindings file
- **THEN** it answers absent naming the bindings file's path, exits 4, and writes nothing

### Requirement: Rebuild takes a landed revision

`tailrace rebuild [revision]` SHALL take the landed revision whose
policy file declares the components, by default the local branch's
head, `--branch` (by default `main`) and `--json`; it SHALL be bound by
the host's bindings file as verify is, never making the store it names,
and SHALL answer each entry it rebuilt as written or held, or the
refusal its write gave (57, 63).

#### Scenario: Rebuild after the record is deleted

- **WHEN** a version is published over the fakes, the host's record is deleted, and `tailrace rebuild --json` runs
- **THEN** it prints the version's entry as written, exits 0, and `tailrace status --json` then prints what it printed before the deletion
