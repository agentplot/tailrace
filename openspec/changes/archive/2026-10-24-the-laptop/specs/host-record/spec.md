# host-record Specification

## MODIFIED Requirements

### Requirement: The host keeps which versions it published from which revisions

After a publish moves a version's name to its index, the host SHALL
keep one entry for that version naming its component, version, tag,
revision, branch, the index's digest and the store it was offered to,
the store as the host's bindings name it, and nothing the store or git
holds besides: no artifact, no claim and no credential (61, 64, 118,
core.20, core.21).

#### Scenario: An entry after a publish

- **WHEN** a version is published
- **THEN** the host's record holds one entry for that component and version naming the revision, the branch, the index's digest and the store, and no envelope, artifact digest or credential

#### Scenario: The store as the bindings name it

- **WHEN** the built binary publishes a version on a host whose bindings file names `[store] layout` with a path beginning `~/`, and status is asked with `--json`
- **THEN** the version's entry names the store as `layout` and that path with `~/` expanded

#### Scenario: Nothing kept of a refused publish

- **WHEN** a publish is refused or stops before its index
- **THEN** the host's record holds no entry for that version
