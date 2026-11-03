## MODIFIED Requirements

### Requirement: The build's contract

The build SHALL answer ensure, given a revision and every output
declared in one asking, with those outputs present, or refused naming
each output it could not make (74); report with the build identity
an output came from, the build's outcome, and where its files are (75);
and left, given a checkout, with the build identities whose outputs the
checkout's own build state holds, each with its tasks, its outcome and
where its files are, read from the build's own records and from nothing
of tailrace's (139).

#### Scenario: Outputs that could not be made

- **WHEN** ensure is asked for three outputs and the build can make two
- **THEN** the answer is refused, naming the one it could not make

#### Scenario: What a checkout left

- **WHEN** left is asked of a checkout whose build state holds the outputs of two build identities
- **THEN** the answer names both, each with its tasks, its outcome and where its files are, and names no build identity the checkout's build state does not hold

#### Scenario: A checkout that ran nothing

- **WHEN** left is asked of a checkout whose build has recorded no run
- **THEN** the answer is an empty list, which is neither absent nor unreadable

## ADDED Requirements

### Requirement: An output whose cache entry the store lacks is a refusal of its own

The refusal a publish gives for an output whose build identity's cache
entry the store does not hold SHALL name the output and the build
identity, and SHALL round-trip through the machine-readable form as
every other refusal does (57, 58).

#### Scenario: The refusal round-trips

- **WHEN** the refusal naming an output and a build identity whose entry the store lacks is written as JSON and read back
- **THEN** it reads as the same refusal, naming the same output and build identity
