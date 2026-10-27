## ADDED Requirements

### Requirement: The gate told by a publish judges the index the publish named

When a publish's answer tells the gate of a version, the telling SHALL
carry the index digest that answer named (95). The gate SHALL resolve
the version's name as it does for any version (core.4) and, where the
name resolves to another digest than the one it was told, SHALL answer
unreadable naming the version's name, the digest it resolves to and the
digest the publish named, before any claim is read or anything is
written. A version the gate is asked about by anyone else carries no
told digest and is judged by what its name resolves to.

#### Scenario: Told the digest the name resolves to

- **WHEN** a publish's answer names index `D` for 0.2.0, and 0.2.0's name resolves to `D`
- **THEN** the gate judges `D`, and its answer names `D`

#### Scenario: Told a digest the name no longer resolves to

- **WHEN** a publish's answer names index `D` for 0.2.0, and 0.2.0's name resolves to `E`
- **THEN** the gate answers unreadable naming 0.2.0's name, `E` and `D`; no claim is read, nothing is attached, and no channel moves
