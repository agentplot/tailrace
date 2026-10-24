# Spec Delta

## MODIFIED Requirements

### Requirement: The gate runs the tools layer

The repository SHALL have a `check-tools` layer: the tests of the
profiles that need moon, sluice, OpenSSH or a registry, each run by a
moon task of its project tagged `check-tools`, one task per check
(core.25). The gate SHALL require the layer on `main` and on every
bolt, beside `check-fast`. A test that needs only git SHALL stay in
`check-fast`.

#### Scenario: A landing to a bolt

- **WHEN** a unit is submitted to a bolt
- **THEN** the gate runs every task tagged `check-fast` and every task tagged `check-tools`

#### Scenario: The fast layer alone

- **WHEN** `moon run ':#check-fast'` runs
- **THEN** no test that needs moon or sluice to run a scratch repository, or a registry, runs

#### Scenario: The registry store in the layer

- **WHEN** `moon run ':#check-tools'` runs on a host whose `devenv.local.nix` names an ECR repository
- **THEN** the registry store's tests run against a zot and a reference distribution each started for the test, and against that ECR repository, as two tasks

### Requirement: The tools are the tree's pins

moon, sluice, OpenSSH, zot, the reference distribution and ECR's
credential helper SHALL be the versions the tree pins, on every host
that runs the layer: moon by `.prototools`, sluice by its release
binary's version and hash in `devenv.nix`, and the others by
`devenv.lock` (core.26). The ECR repository the layer pushes to SHALL
be the host's, named in its `devenv.local.nix`.

#### Scenario: sluice in the shell

- **WHEN** `devenv shell -- sluice --version` runs on a host whose system sluice releases a binary for
- **THEN** it prints the version `devenv.nix` pins

#### Scenario: zot in the shell

- **WHEN** the registry store's tests start a zot
- **THEN** they run the binary `devenv.nix` names by its store path, never one found on `PATH`
