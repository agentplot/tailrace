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

### Requirement: A name's versions are the index's names

The layout store SHALL answer the listing of an ecosystem and a name
from the ref names of its `index.json` that read back as that
ecosystem's and that name's store names, answering each one's version,
and the empty list where none does (138).

#### Scenario: Versions listed from the index's names

- **WHEN** a layout store names `artifact/npm/@acme/site@1.0.0`, `artifact/npm/@acme/site@1.1.0+b.1`, `artifact/npm/@acme/site-cli@1.0.0` and `site-v1.0.0`, and the versions of `npm` `@acme/site` are asked
- **THEN** the answer is 1.0.0 and 1.1.0+b.1 and no other

#### Scenario: A name the layout does not hold

- **WHEN** the versions of a name the layout store names at no version are asked
- **THEN** the answer is the empty list, and nothing is written
