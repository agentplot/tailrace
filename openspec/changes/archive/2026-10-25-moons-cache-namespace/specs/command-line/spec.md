# Spec Delta

## MODIFIED Requirements

### Requirement: Each command calls the one core and renders its answer

`tailrace publish`, `tailrace verify`, `tailrace status` and `tailrace
sync` SHALL each turn their arguments into one question to the core and
render the core's answer; no command SHALL read a seam, a file of the
record or the history itself, or compute a version, a digest or a signer
of its own (56, core.31).

#### Scenario: The same answer through the command line

- **WHEN** a version is published over the fakes through the command line with `--json`, and the same question is put to the core directly over an identical binding
- **THEN** the printed JSON is the core's answer, value for value

## ADDED Requirements

### Requirement: Sync fills the checkout's cache from a landed revision

`tailrace sync [revision]` SHALL take the revision whose verdicts name
the entries, by default the newest revision of the branch the checkout
descends from, `--branch` (by default `main`) and `--json`; it SHALL be
bound by the host's bindings file as verify is, never making the store
it names, and SHALL answer each build identity as rendered, absent, or
the refusal or unreadable answer its render gave, exiting 0 when the
only entries not rendered are absent (54, 57).

#### Scenario: A sync with one entry gone

- **WHEN** `tailrace sync --json` runs over the fakes for a landed revision whose verdicts name two build identities, one of which the store holds no entry for
- **THEN** the printed JSON names one as rendered and the other as absent, and the exit is 0

#### Scenario: A sync on an unbound host

- **WHEN** the installed binary runs `tailrace sync` on a host with no bindings file
- **THEN** it answers absent naming the bindings file's path, exits 4, and writes nothing
