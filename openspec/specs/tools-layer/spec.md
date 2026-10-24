# tools-layer Specification

## Purpose
The gate's layer for tests that run against real tools rather than the
fakes, and the pinned tools that layer runs, so a profile proven against
the real thing is proven on every landing.

## Requirements

### Requirement: The gate runs the tools layer

The repository SHALL have a `check-tools` layer: the tests of the
profiles that need moon, sluice or OpenSSH, each run by a moon task of
its project tagged `check-tools`, one task per check (core.25). The
gate SHALL require the layer on `main` and on every bolt, beside
`check-fast`. A test that needs only git SHALL stay in `check-fast`.

#### Scenario: A landing to a bolt

- **WHEN** a unit is submitted to a bolt
- **THEN** the gate runs every task tagged `check-fast` and every task tagged `check-tools`

#### Scenario: The fast layer alone

- **WHEN** `moon run ':#check-fast'` runs
- **THEN** no test that needs moon or sluice to run a scratch repository runs

### Requirement: The tools are the tree's pins

moon, sluice and OpenSSH SHALL be the versions the tree pins, on every
host that runs the layer: moon by `.prototools`, sluice by its release
binary's version and hash in `devenv.nix`, OpenSSH by `devenv.lock`
(core.26).

#### Scenario: sluice in the shell

- **WHEN** `devenv shell -- sluice --version` runs on a host whose system sluice releases a binary for
- **THEN** it prints the version `devenv.nix` pins
