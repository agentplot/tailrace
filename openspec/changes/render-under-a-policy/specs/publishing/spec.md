## ADDED Requirements

### Requirement: The index carries the publisher's layout policy
Where the publishing repository declares a policy for layouts at the
landed revision, a publish SHALL sign it as a layout-policy claim about
the index, in the identity of the party running the publish and naming
that revision, and attach it to the index after the index is put and
before the version's name is moved (113, 8, core.8, core.9). Where none
is declared, it SHALL attach none. A publish whose index already
carries that claim SHALL attach nothing more.

#### Scenario: A publish under a policy for layouts
- **WHEN** a revision whose policy file declares a policy for layouts is published
- **THEN** the index carries one layout-policy claim by the publishing host naming the revision and the claims that policy requires, attached before the version's name was moved

#### Scenario: A publish with no policy for layouts
- **WHEN** a revision whose policy file declares no policy for layouts is published
- **THEN** the index carries no layout-policy claim, and the publish answers as before

#### Scenario: A repeat publish
- **WHEN** the same revision is published again and ends at the same index
- **THEN** the index still carries one layout-policy claim
