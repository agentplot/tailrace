## ADDED Requirements

### Requirement: The names under a prefix are read from the index

The layout store SHALL answer names, given a prefix, with the ref names
of `index.json`'s entries that begin with it, each once, and none where
no entry's does (138). A layout whose `index.json` cannot be read SHALL
answer unreadable, never none (58).

#### Scenario: Names from the index

- **WHEN** a layout holding the names `artifact/cargo/fam-a@1.0.0`, `artifact/cargo/fam-a@1.1.0` and `fam-v1.0.0` is asked for the names under `artifact/cargo/fam-a@`
- **THEN** it answers the first two and no other

#### Scenario: An index that cannot be read

- **WHEN** a layout's `index.json` is not JSON and the names under a prefix are asked
- **THEN** the answer is unreadable naming the file, not an empty list
