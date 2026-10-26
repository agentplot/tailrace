## MODIFIED Requirements

### Requirement: The gate runs the tools layer

The repository SHALL have a `check-tools` layer: the tests of the
profiles that need moon, sluice, OpenSSH, npm or a registry the test
starts itself, each run by a moon task of its project tagged
`check-tools`, one task per check (core.25). The gate SHALL require the
layer on `main` and on every bolt, beside `check-fast`. A test that
needs only git SHALL stay in `check-fast`. A check that needs a
credential a person signs in for, as the run against ECR does, SHALL run
outside the gate, under a tag of its own, by hand with a live session or
by a job that holds an identity of its own (core.38).

#### Scenario: A landing to a bolt

- **WHEN** a unit is submitted to a bolt
- **THEN** the gate runs every task tagged `check-fast` and every task tagged `check-tools`, and no task tagged `check-ecr`

#### Scenario: The fast layer alone

- **WHEN** `moon run ':#check-fast'` runs
- **THEN** no test that needs moon or sluice to run a scratch repository, npm, or a registry, runs

#### Scenario: The registry store in the layer

- **WHEN** `moon run ':#check-tools'` runs
- **THEN** the registry store's tests run against a zot and a reference distribution each started for the test, and nothing reaches ECR

#### Scenario: npm in the layer

- **WHEN** `moon run ':#check-tools'` runs
- **THEN** the npm projection's pack scenarios and its `npm install` against a served layout, a first install with no lockfile among them, run with the npm the tree pins, under a home and an npm configuration of the test's own, and no registry but the served layout is asked for a family package

#### Scenario: The ECR check outside the gate

- **WHEN** `moon run ':#check-ecr'` runs on a host whose `devenv.local.nix` names an ECR repository, with a live session for it
- **THEN** the registry store's group runs against that ECR repository, in a task of its own

### Requirement: The tools are the tree's pins

moon, sluice, OpenSSH, zot, the reference distribution, ECR's credential
helper, and Node.js with its npm SHALL be the versions the tree pins, on
every host that runs the layer: moon by `.prototools`, sluice and zot
each by its release binary's version and per-system hash in
`devenv.nix`, and the others by `devenv.lock` (core.26). The ECR
repository the ECR check pushes to SHALL be the host's, named in its
`devenv.local.nix`.

#### Scenario: sluice in the shell

- **WHEN** `devenv shell -- sluice --version` runs on a host whose system sluice releases a binary for
- **THEN** it prints the version `devenv.nix` pins

#### Scenario: zot in the shell

- **WHEN** the registry store's tests start a zot
- **THEN** they run the binary `devenv.nix` names by its store path, never one found on `PATH`

#### Scenario: npm in the shell

- **WHEN** `devenv shell -- npm --version` runs
- **THEN** it prints the version of the npm `devenv.lock`'s Node.js carries
