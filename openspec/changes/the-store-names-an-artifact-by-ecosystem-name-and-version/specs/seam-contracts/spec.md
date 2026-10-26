## MODIFIED Requirements

### Requirement: An artifact says what its bytes are

Artifact metadata SHALL name the ecosystem whose form it takes, its
name in that ecosystem and its version (4). Index metadata SHALL name
one component and one version, and an index SHALL name every artifact
of that component at that version by digest (6, 110). Index metadata
SHALL also name the revision it was published from and the publishing
repository by its remote's owner and name, each where it has one, and
index metadata naming neither SHALL still be read (13, 138).

#### Scenario: Metadata with a part missing

- **WHEN** artifact metadata is made with an empty ecosystem, name or version
- **THEN** it is rejected, naming the part that is missing, and no metadata is made

#### Scenario: An index names its artifacts

- **WHEN** index metadata is made for a component, a version and a set of artifacts
- **THEN** it names each artifact's digest and metadata, and every artifact it names carries that version

#### Scenario: An index names where it came from

- **WHEN** index metadata naming a revision and the repository `acme/site` is written in its machine-readable form and read back
- **THEN** it is the same value, naming that revision and that owner and name

#### Scenario: An index written before it named where it came from

- **WHEN** index metadata in the machine-readable form that names no revision and no repository is read
- **THEN** it is read, naming neither, and written again it gives the same form
