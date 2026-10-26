## ADDED Requirements

### Requirement: A name's versions are read from the index

The layout store SHALL answer the versions of a name in an ecosystem
with what follows `artifact/<ecosystem>/<name>@` in the ref names of
`index.json`'s entries that begin with it, each once, and none where no
entry's does (138). A layout whose `index.json` cannot be read SHALL
answer unreadable, never none (58).

#### Scenario: Versions from the index

- **WHEN** a layout holding the names `artifact/cargo/fam-a@1.0.0`, `artifact/cargo/fam-a@1.1.0`, `artifact/cargo/fam-ab@1.0.0` and `fam-v1.0.0` is asked for the versions of `fam-a` in `cargo`
- **THEN** it answers `1.0.0` and `1.1.0` and no other

#### Scenario: An index that cannot be read

- **WHEN** a layout's `index.json` is not JSON and a name's versions are asked
- **THEN** the answer is unreadable naming the file, not an empty list
