## ADDED Requirements

### Requirement: A pack is handed its component

A publish SHALL hand each output's pack, with the version, the component
the output belongs to, the component's declared outputs with the files
the build left for each, and a way to ask whether the component's paths
hold a path of the repository, matched as the version rule matches them
(84, 123). It SHALL do so only once every output of the component has
been built and its outcome read (core.12).

#### Scenario: The component and its outputs

- **WHEN** a component declaring the outputs `fam-a` and `fam-b` publishes
- **THEN** each output's pack is given the component and both outputs, each with the files its build left

#### Scenario: The component's paths

- **WHEN** a component declaring the paths `crates/fam-*/**` publishes an output, and its projection asks of `crates/fam-b/Cargo.toml` and of `crates/base-c/Cargo.toml`
- **THEN** the first is held by the component's paths and the second is not
