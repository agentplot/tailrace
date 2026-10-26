## ADDED Requirements

### Requirement: No component's tag enters a reserved space

The policy file SHALL be refused when a component's prefix, or the
rule's prefix where no component is declared, begins `artifact/`,
`cache/` or `channel/`, naming the component, the prefix and the space,
before any version is derived, so no version's name is ever an
artifact's, a cache entry's or a channel's name (138).

#### Scenario: A prefix in the artifact space

- **WHEN** a policy file declares a component whose prefix is `artifact/web-v` and a revision is published
- **THEN** the publish is refused naming the component, the prefix and `artifact/`, and no version is derived, claimed or offered

#### Scenario: The rule's prefix in the cache space

- **WHEN** a policy file declares no component and its rule's prefix is `cache/`
- **THEN** reading the policy file is refused naming the prefix and `cache/`

#### Scenario: A prefix that only begins like one

- **WHEN** a component's prefix is `artifact-v`
- **THEN** the policy file is read and the component's tags carry that prefix

### Requirement: A component's name names one repository

The policy file SHALL be refused, naming the component and what its
name would enter, before any version is derived, when a declared
component is named `artifact`, `cache` or `repository`, or its name
holds a `/`; and SHALL be refused naming both when two declared
components' names are equal once lowercased, so every declared
component has a repository of its own on a registry, apart from the
store's other spaces and from the one component of a repository that
declares none (125, C.1).

#### Scenario: A component named for a reserved space

- **WHEN** a policy file declares a component named `cache`
- **THEN** reading the policy file is refused naming the component and `cache/`, and nothing is derived

#### Scenario: A component named as the undeclared one is

- **WHEN** a policy file declares a component named `repository`
- **THEN** reading the policy file is refused naming the component

#### Scenario: A name holding a slash

- **WHEN** a policy file declares a component named `web/app`
- **THEN** reading the policy file is refused naming the component

#### Scenario: Two names differing only by case

- **WHEN** a policy file declares components named `Lib` and `lib`
- **THEN** reading the policy file is refused naming both components
