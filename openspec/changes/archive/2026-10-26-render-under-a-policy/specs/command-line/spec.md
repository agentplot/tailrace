## MODIFIED Requirements

### Requirement: Sync fills the checkout's cache from a landed revision
`tailrace sync [revision]` SHALL take the revision whose verdicts name
the entries and whose policy judges the layouts, by default the newest
revision of the branch the checkout descends from, `--branch` (by
default `main`) and `--json`; it SHALL be bound by the host's bindings
file as verify is, with the signers the file trusts, never making the
store it names, and SHALL answer,
as one answer of the core, each build identity as rendered, absent, or
the refusal or unreadable answer its render gave, and each layout as
refreshed, naming every pin held, refused with its claim found wanting,
or absent, or as what it still holds and when it was rendered beside
what could not be read. It SHALL exit 0 when the only entries and pins
not rendered or held are absent, and otherwise with the worst code
among them (51, 54, 57, 99).

#### Scenario: A sync with one entry gone
- **WHEN** `tailrace sync --json` runs over the fakes for a landed revision whose verdicts name two build identities, one of which the store holds no entry for
- **THEN** the printed JSON names one as rendered and the other as absent, and the exit is 0

#### Scenario: A sync on an unbound host
- **WHEN** the installed binary runs `tailrace sync` on a host with no bindings file
- **THEN** it answers absent naming the bindings file's path, exits 4, and writes nothing

#### Scenario: A sync with a version refused
- **WHEN** `tailrace sync --json` runs over the fakes in a checkout whose lockfile pins one version the policy admits, one it refuses and one the store does not name
- **THEN** the printed JSON is the core's answer value for value, naming the first held, the second refused with its version and the claim found wanting, and the third absent, and the exit is 1

#### Scenario: A sync whose layout cannot be refreshed
- **WHEN** `tailrace sync` runs over the fakes while the store's resolve is unreadable with the words `connection refused`, in a checkout whose layout was rendered before
- **THEN** it prints what the layout holds and when it was rendered, prints `connection refused` verbatim, and exits 3
