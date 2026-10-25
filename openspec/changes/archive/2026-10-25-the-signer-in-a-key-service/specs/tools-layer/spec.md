# Spec Delta

## MODIFIED Requirements

### Requirement: The gate runs the tools layer

The repository SHALL have a `check-tools` layer: the tests of the
profiles that need moon, sluice, OpenSSH or a registry the test starts
itself, each run by a moon task of its project tagged `check-tools`,
one task per check (core.25). The gate SHALL require the layer on
`main` and on every bolt, beside `check-fast`. A test that needs only
git SHALL stay in `check-fast`. A check that needs a credential a
person signs in for, as the runs against ECR and against a KMS key do,
SHALL run outside the gate, under a tag of its own, by hand with a live
session or by a job that holds an identity of its own (core.38).

#### Scenario: A landing to a bolt

- **WHEN** a unit is submitted to a bolt
- **THEN** the gate runs every task tagged `check-fast` and every task tagged `check-tools`, and no task tagged `check-ecr` or `check-kms`

#### Scenario: The fast layer alone

- **WHEN** `moon run ':#check-fast'` runs
- **THEN** no test that needs moon or sluice to run a scratch repository, or a registry, runs, and nothing reaches AWS

#### Scenario: The registry store in the layer

- **WHEN** `moon run ':#check-tools'` runs
- **THEN** the registry store's tests run against a zot and a reference distribution each started for the test, and nothing reaches ECR or KMS

#### Scenario: The ECR check outside the gate

- **WHEN** `moon run ':#check-ecr'` runs on a host whose `devenv.local.nix` names an ECR repository, with a live session for it
- **THEN** the registry store's group runs against that ECR repository, in a task of its own

#### Scenario: The KMS check outside the gate

- **WHEN** `moon run ':#check-kms'` runs on a host whose `devenv.local.nix` names a KMS key and a second key, with a live session for their account
- **THEN** the key service signer's group runs against those keys, in a task of its own

#### Scenario: The KMS check on a host that names no key

- **WHEN** `moon run ':#check-kms'` runs on a host whose `devenv.local.nix` names no KMS key
- **THEN** it fails, naming the setting to set, and passes nothing it did not run

### Requirement: The tools are the tree's pins

moon, sluice, OpenSSH, zot, the reference distribution and ECR's
credential helper SHALL be the versions the tree pins, on every host
that runs the layer: moon by `.prototools`, sluice and zot each by its
release binary's version and per-system hash in `devenv.nix`, and the
others by `devenv.lock` (core.26). The ECR repository the ECR check
pushes to, and the KMS keys the KMS check signs with, SHALL be the
host's, named in its `devenv.local.nix`.

#### Scenario: sluice in the shell

- **WHEN** `devenv shell -- sluice --version` runs on a host whose system sluice releases a binary for
- **THEN** it prints the version `devenv.nix` pins

#### Scenario: zot in the shell

- **WHEN** the registry store's tests start a zot
- **THEN** they run the binary `devenv.nix` names by its store path, never one found on `PATH`

#### Scenario: The KMS keys are the host's

- **WHEN** the KMS check runs
- **THEN** it signs with the key `env.TAILRACE_TOOLS_KMS_KEY` names and uses the key `env.TAILRACE_TOOLS_KMS_STRANGER` names as the stranger, both from the host's `devenv.local.nix`, and with no other key
