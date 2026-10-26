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
