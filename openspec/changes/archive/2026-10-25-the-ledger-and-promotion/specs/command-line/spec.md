## MODIFIED Requirements

### Requirement: Each command calls the one core and renders its answer

`tailrace publish`, `tailrace verify`, `tailrace status`, `tailrace
sync`, `tailrace push`, `tailrace rebuild` and `tailrace promote` SHALL
each turn their arguments into one question to the core and render the
core's answer; no command SHALL read a seam, a file of the record or the
history itself, or compute a version, a digest, a signer or a channel of
its own (56, core.31).

#### Scenario: The same answer through the command line

- **WHEN** a version is published over the fakes through the command line with `--json`, and the same question is put to the core directly over an identical binding
- **THEN** the printed JSON is the core's answer, value for value

## ADDED Requirements

### Requirement: Promote takes a version and a channel

`tailrace promote <version> --to <channel>` SHALL take the version,
bare of its component's prefix, the channel, `--from <version>` where
the asker expects the channel at a version, `--component` (needed only
where more than one is declared), `--branch` (by default `main`) and
`--json`; `tailrace promote --back --to <channel>` SHALL take no version
and promote the version the channel's head moved from. It SHALL be
bound by the host's bindings file, needing `[signing]` and `[ledger]`,
and never making the store it names. It SHALL print the channel, the
version and index moved to and from, the promotion claim's digest and
its predecessor's, and how the store followed, with the store's own
answer beside where it was not written (33, 57).

#### Scenario: A promotion as JSON

- **WHEN** `tailrace promote 0.2.0 --to stable --json` runs over the fakes for an accepted version
- **THEN** the printed JSON is the core's answer value for value, naming the claim's digest, and the exit is 0

#### Scenario: A stale start

- **WHEN** `tailrace promote 0.3.0 --to stable --from 0.1.0` runs while `stable` is at 0.2.0
- **THEN** it exits 1 naming 0.2.0 as where `stable` is

#### Scenario: A version and --back together

- **WHEN** `tailrace promote 0.2.0 --back --to stable` is run
- **THEN** it exits 2 naming the arguments it cannot take together, and the core is not asked
