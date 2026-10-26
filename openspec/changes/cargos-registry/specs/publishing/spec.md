## ADDED Requirements

### Requirement: A pack is handed its component's paths and the store's artifacts of a name

A publish SHALL hand each output's pack, with the version, a way to ask
whether the component's paths hold a path of the repository, matched as
the version rule matches them (123), and a way to ask the artifacts the
store holds of a name in the output's ecosystem, each found by its
metadata in the index its version's name resolves to (138, 4). A name
the store holds no version of SHALL answer none; a version whose index
names no one artifact of that name SHALL answer unreadable naming the
name and the index.

#### Scenario: The component's paths

- **WHEN** a component declaring the paths `crates/fam-*/**` publishes an output, and its projection asks of `crates/fam-b/Cargo.toml` and of `crates/base-c/Cargo.toml`
- **THEN** the first is held by the component's paths and the second is not

#### Scenario: The store's artifacts of a name

- **WHEN** the store names `artifact/cargo/base-c@0.3.1` and `artifact/cargo/base-c@0.4.0`, each at an index naming one Cargo artifact `base-c` of that version, and a projection asks for `base-c`
- **THEN** it is answered both artifacts, each with its digest, and asked for a name the store holds none of, it is answered none
