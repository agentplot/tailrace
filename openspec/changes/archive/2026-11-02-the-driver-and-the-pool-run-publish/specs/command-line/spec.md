## ADDED Requirements

### Requirement: Push takes what the checkout left with --left

`tailrace push --left` SHALL push what the checkout it runs in left,
taking no revision and no `--branch`, and `--json`; it SHALL ask no
landing, SHALL answer each build identity as `tailrace push` does, and
SHALL exit 0 when the only build identities not written or held are
absent (54, 57, 139). A revision or `--branch` given beside `--left`
SHALL exit 2 naming the argument, with the core not asked.

#### Scenario: A check job's push

- **WHEN** `tailrace push --left --json` runs over the fakes in a checkout the fake build scripts as having left two passing build identities
- **THEN** the printed JSON names the checkout and both build identities as written with their digests, its JSON is the core's answer value for value, the landing was asked nothing, and the exit is 0

#### Scenario: A revision beside --left

- **WHEN** `tailrace push --left HEAD` runs
- **THEN** it exits 2 naming the revision it cannot take with `--left`, and the core is not asked

### Requirement: Publish fills the checkout it runs in

`tailrace publish` SHALL render the cache entries it reads from the
store into the working tree of the checkout it runs in, and SHALL answer
absent naming the working tree where the checkout has none, before
anything is read of the store (135).

#### Scenario: A bare repository

- **WHEN** `tailrace publish` runs in a repository with no working tree, for a landing with verdicts
- **THEN** it answers absent naming the working tree it would render into, and the store is asked nothing
