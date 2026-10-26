## ADDED Requirements

### Requirement: A version a served layout lacks is rendered on the request
When a served layout's client asks for a version the layout lacks, the
core SHALL resolve that one version as a pin is resolved, judge it under
the policy `tailrace sync` with no argument would use in the checkout the
layout belongs to, and, where the policy admits it, render the layout
holding what it held and that version, and record it; it SHALL answer
the one version held, refused naming the claim found wanting, absent, or
unreadable with the source's own words (115, 50, 47, 99, 51). A version
refused, absent or unreadable SHALL leave the layout and its record as
they were. Only the one version asked SHALL be read from the store.

#### Scenario: A miss the policy admits
- **WHEN** a layout holding `1.4.0` of a name is asked for `1.3.0`, which the store names and the policy admits
- **THEN** the layout holds `1.3.0` and `1.4.0`, the record names both, and the store was asked about `1.3.0` alone

#### Scenario: A miss the policy refuses
- **WHEN** a layout is asked for a version whose index lacks the claim the policy requires first
- **THEN** the answer names the version and that claim as wanting, and the layout and its record are unchanged

#### Scenario: A miss the store does not name
- **WHEN** a layout is asked for a version the store holds no name for
- **THEN** the answer is absent naming the name, never refused, and the layout is unchanged

#### Scenario: A miss while the store is unreachable
- **WHEN** a layout is asked for a version while the store cannot be reached
- **THEN** the answer is unreadable carrying the store's own words, and the layout is unchanged

### Requirement: One render of a layout at a time
A sync and a render on a miss SHALL each hold the layout's own lock, in
the checkout's git directory beside its record, from the first read of
the record to the last write, so two renders of one layout never
interleave and the record always names what the layout holds (48, 62).

#### Scenario: Two misses at once
- **WHEN** two requests for two versions a layout lacks are answered at once, and the policy admits both
- **THEN** the layout and its record hold both versions afterwards

#### Scenario: A sync during a miss
- **WHEN** a sync of a checkout runs while a miss renders into its layout
- **THEN** one waits for the other, and the record afterwards names exactly what the layout holds

### Requirement: A served layout's address is the core's, with the repository in the path
The address a served layout is reached at SHALL be
`http://127.0.0.1:7420/<owner>/<name>/<ecosystem>/`, the core's one name
and port, the owner and name the consuming repository's landing remote
gives it, and the ecosystem's name, the same on every host and in every
checkout of the repository, naming no directory and no store (114, 49,
core.18).

#### Scenario: Two remotes of one repository
- **WHEN** one checkout's landing remote is `https://example.com/agentplot/consumer.git` and another's `git@example.com:agentplot/consumer`
- **THEN** both checkouts' npm layout is served at `http://127.0.0.1:7420/agentplot/consumer/npm/`
