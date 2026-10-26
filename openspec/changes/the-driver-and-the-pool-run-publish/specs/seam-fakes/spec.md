## ADDED Requirements

### Requirement: The fake build answers what a checkout left

The fake build SHALL answer left, for a checkout a test names, with the
build identities the test scripted that checkout as holding, each with
its tasks, outcome and files, in the order scripted, and an empty list
for a checkout the test scripted nothing for; it SHALL record each
asking with the checkout it was given.

#### Scenario: A checkout scripted with two builds

- **WHEN** a test scripts a checkout as holding two build identities, one passed and one failed, and the core asks the fake build what that checkout left
- **THEN** the answer names both in the scripted order with their outcomes, and the fake's record holds one left naming the checkout

#### Scenario: A checkout scripted with nothing

- **WHEN** the core asks the fake build what a checkout no test scripted left
- **THEN** the answer is an empty list
