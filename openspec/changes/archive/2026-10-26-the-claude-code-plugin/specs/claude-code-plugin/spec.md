# Spec Delta

## Purpose

tailrace's surface in a Claude Code session: a plugin kept outside the
core, as sluice keeps its plugins (section 9), whose hook keeps a session
from publishing a tailrace repository any way but `tailrace publish`, and
which answers nothing the one core does not (56, core.31).

## ADDED Requirements

### Requirement: An ecosystem's own publish command is refused in a tailrace repository

Before a Bash call in a repository whose HEAD holds
`.config/tailrace.toml`, the plugin's hook SHALL refuse a command that
runs an ecosystem's own publish command, the command a package manager
or image tool offers to put a package or image in a registry, and SHALL
give as its reason that the repository publishes only through
`tailrace publish`, for a landed revision.

#### Scenario: npm publish

- **WHEN** a session in a repository whose HEAD holds `.config/tailrace.toml` runs `npm publish`
- **THEN** the call is refused with a reason naming `tailrace publish`

#### Scenario: cargo publish

- **WHEN** the same session runs `cargo publish -p family-crate`
- **THEN** the call is refused with a reason naming `tailrace publish`

#### Scenario: The other ecosystems

- **WHEN** the same session runs `pnpm publish`, `yarn npm publish`, `bun publish`, `mvn clean deploy`, `./gradlew publish`, `uv publish`, `twine upload dist/*`, `poetry publish`, `dotnet nuget push x.nupkg`, `docker push acme/api:1` or `podman push acme/api:1`
- **THEN** each call is refused with a reason naming `tailrace publish`

#### Scenario: A publish inside a longer command

- **WHEN** the same session runs `cd packages/ui && NPM_CONFIG_PROVENANCE=true npm --workspace ui publish --dry-run`
- **THEN** the call is refused

### Requirement: Every other command goes through

The hook SHALL let through, with no output, every command that runs no
ecosystem's publish command, and every command in a repository whose
HEAD holds no `.config/tailrace.toml` or in a directory that is no git
repository.

#### Scenario: Building and packing

- **WHEN** a session in a tailrace repository runs `npm install`, `npm pack`, `cargo build`, `cargo package --no-verify`, `docker build .`, `git push origin unit/x` or `tailrace publish HEAD --json`
- **THEN** each call goes through and the hook prints nothing

#### Scenario: A publish only quoted

- **WHEN** a session in a tailrace repository runs `echo "npm publish"`
- **THEN** the call goes through

#### Scenario: Not a tailrace repository

- **WHEN** a session in a repository whose HEAD holds no `.config/tailrace.toml` runs `npm publish`
- **THEN** the call goes through and the hook prints nothing

### Requirement: The plugin answers nothing the core does not

The plugin SHALL compute no verdict, version, component or channel, and
SHALL read no store, no host record and no policy's contents (56,
core.31): its only reading of the repository is whether HEAD holds
`.config/tailrace.toml`, and its only answer is the refusal, which names
the command to run and nothing it derived.

#### Scenario: A refusal with no tailrace on the host

- **WHEN** the hook refuses `npm publish` in a tailrace repository on a host with no `tailrace` binary on `PATH`, in a package whose `package.json` names version `1.4.0`
- **THEN** the refusal is the same as on a host with the binary, and names no version, component or channel

### Requirement: The plugin is installed from the repository's marketplace

The repository's root SHALL hold a Claude Code marketplace named
`tailrace` listing one plugin, `tailrace`, whose source is
`plugins/claude-code`, so `/plugin marketplace add agentplot/tailrace`
then `/plugin install tailrace@tailrace` installs it; the marketplace and
the plugin's manifest SHALL pass the pinned Claude Code's `plugin
validate --strict` in the gate's fast layer (core.25, core.26).

#### Scenario: The manifests validate

- **WHEN** the gate's fast layer runs
- **THEN** the pinned Claude Code validates the marketplace and the plugin with `--strict`, with no user configuration or credential read

#### Scenario: A listed plugin with no manifest

- **WHEN** the marketplace lists a plugin whose source holds no `.claude-plugin/plugin.json`
- **THEN** the marketplace check fails naming the plugin
