## ADDED Requirements

### Requirement: An artifact's name is a tag as every name is

The registry SHALL hold the name `artifact/<ecosystem>/<name>@<version>`
as a tag in the one repository the binding declares, escaped by the
same rule as every other name, a channel's among them, and refuse one
whose tag would pass the tag's length, naming it (70, 71, 138, C.1).

#### Scenario: A scoped package's name

- **WHEN** the name `artifact/npm/@acme/site@1.3.0` is made at an index and resolved, and so is `artifact/npm/acme/site@1.3.0`
- **THEN** each resolves to its own digest, each is a valid tag in the binding's repository, and no other repository is made or written

#### Scenario: A name too long for a tag

- **WHEN** an artifact's name escapes to a tag past 128 characters
- **THEN** making it is refused naming the name and its length, and no tag is written
