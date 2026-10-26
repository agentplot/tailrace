## ADDED Requirements

### Requirement: An index holds its revision and repository in its bytes

put-artifact SHALL write an index's revision and its publishing
repository's owner and name as annotations of the OCI image index, each
only where the index metadata names it, and read SHALL answer them. An
index naming neither SHALL be written in the same bytes as before they
existed, so its digest does not change (1, 5, 13, 138).

#### Scenario: An index read back with where it came from

- **WHEN** an index naming a revision and the repository `acme/site` is put and its digest is read
- **THEN** the read answers that revision and that repository, beside its component, version and artifacts

#### Scenario: An index naming neither keeps its digest

- **WHEN** an index naming no revision and no repository is put
- **THEN** its image index carries no revision or repository annotation, and its digest is the one the same index had before indexes named them
