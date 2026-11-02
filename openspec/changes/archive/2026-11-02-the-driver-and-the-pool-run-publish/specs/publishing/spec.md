## MODIFIED Requirements

### Requirement: Every output is ensured in one asking and its outcome read

A publish SHALL ask the build seam to ensure every output the component
declares in one asking, then read each output's build identity, outcome
and files, before anything is packed. Where the landing carries verdicts
for the revision, the publish SHALL first fill the checkout's build
cache from the store, as the next requirement says, so the asking
replays what the landing's checks built. An output the build could not
make SHALL refuse the version whole, naming each such output (74, 75,
16).

#### Scenario: One asking for every output

- **WHEN** a component declaring two outputs is published
- **THEN** the fake build records one ensure naming both outputs, then a report of each, before any pack

#### Scenario: An output the build could not make

- **WHEN** the fake build cannot make one of a component's two outputs
- **THEN** the publish is refused naming that output, and nothing is packed or put

#### Scenario: The cache filled before the asking

- **WHEN** a landing with verdicts is published and the store holds the cache entries its passing verdicts name
- **THEN** every cache-entry render into the checkout precedes the one ensure

## ADDED Requirements

### Requirement: Where the landing carries verdicts, the outputs come from the store

Where the landing carries verdicts, a publish SHALL resolve the cache
entry of each build identity a passing verdict names, once, and render
each the store holds into the checkout through every bound projection
that renders cache entries, before the build is asked (52, 135). It
SHALL ask the store only to resolve and read for this, write nothing to
it, and a render's refusal or unreadable answer SHALL be the publish's
whole answer.

#### Scenario: Entries rendered from the store

- **WHEN** a landing whose verdicts pass on two build identities is published and the store holds both entries
- **THEN** both are resolved and rendered into the checkout, and the store records no put and no name for them

#### Scenario: A failed verdict's build identity

- **WHEN** a landing's verdicts fail on one build identity and pass on another
- **THEN** only the passing build identity's entry is resolved

#### Scenario: A landing with no verdicts fills nothing

- **WHEN** the landing carries no verdicts for the revision
- **THEN** no cache name is resolved and nothing is rendered before the build is asked (C.6)

### Requirement: An output whose entry the store lacks refuses the version

Where the landing carries verdicts, an output whose reported build
identity's cache entry was not rendered from the store SHALL refuse the
version whole, naming the output and its build identity, before the
version's name is claimed and with nothing packed or offered (15, 16,
58, I3). A publish SHALL never offer bytes a build made on its own host
in place of the landing's.

#### Scenario: One output's entry missing from the store

- **WHEN** a component's two outputs come from build identities the landing's verdicts pass, and the store holds the entry of only one
- **THEN** the publish is refused naming the other output and its build identity, no tag is claimed, and the store records no put

#### Scenario: Every entry held

- **WHEN** the store holds the entry of every output's build identity
- **THEN** the publish offers the version as before, and each artifact's bytes are packed from what the build replayed

#### Scenario: A plain branch builds on the publishing host

- **WHEN** the landing carries no verdicts and the store holds no cache entry
- **THEN** the publish offers what the build left on the host, and is not refused for an entry missing (135, S13)
