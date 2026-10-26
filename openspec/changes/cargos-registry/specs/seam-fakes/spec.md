## ADDED Requirements

### Requirement: The fake store lists the names it holds

The fake store SHALL answer names, given a prefix, with every name it
holds that begins with it, and none where it holds none (138).

#### Scenario: The fake's names under a prefix

- **WHEN** the fake store holds the names `artifact/cargo/fam-a@1.0.0` and `artifact/cargo/fam-b@1.0.0` and is asked for the names under `artifact/cargo/fam-a@`
- **THEN** it answers the first and not the second
