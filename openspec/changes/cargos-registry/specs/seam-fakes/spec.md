## ADDED Requirements

### Requirement: The fake store lists a name's versions

The fake store SHALL answer the versions of a name in an ecosystem with
the version of every pointer `artifact/<ecosystem>/<name>@<version>` it
holds, and none where it holds none (138).

#### Scenario: The fake's versions of a name

- **WHEN** the fake store holds the names `artifact/cargo/fam-a@1.0.0` and `artifact/cargo/fam-b@1.0.0` and is asked for the versions of `fam-a` in `cargo`
- **THEN** it answers `1.0.0` alone, and asked for `fam-c`, it answers none
