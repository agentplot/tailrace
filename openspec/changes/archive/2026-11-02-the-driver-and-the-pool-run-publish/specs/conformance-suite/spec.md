## MODIFIED Requirements

### Requirement: The build's contract

The build's group SHALL hold a build to 74 to 76: every output ensured
in one asking or the unmade ones named, the same revision giving the
same build identity on every host, and what a checkout's run left
answered with each build identity, its tasks, its outcome and its files,
or an empty list for a checkout that ran nothing.

#### Scenario: Outputs ensured and reported

- **WHEN** every declared output of a revision is ensured in one asking and each is reported
- **THEN** each report names a build identity, a pass and where the output's files are

#### Scenario: Outputs the build could not make

- **WHEN** two of three outputs cannot be made
- **THEN** ensure is refused naming exactly those two

#### Scenario: The same revision on a second host

- **WHEN** one revision's output is reported on the rig's host and on a second host the rig gives
- **THEN** both name the same build identity

#### Scenario: What an ensure left

- **WHEN** a revision's outputs are ensured in a checkout the rig gives and left is asked of that checkout
- **THEN** left names the build identity each report answered, with a pass

#### Scenario: A checkout that ran nothing

- **WHEN** left is asked of a checkout the rig gives in which nothing has run
- **THEN** it answers an empty list

## ADDED Requirements

### Requirement: Publish from the store over the fakes

The publish group SHALL hold the core, over the fakes, to taking a
landing's outputs from the store, below its marker, citing 15, 16, 52
and 58: the passing verdicts' entries rendered before the build is
asked, an output whose entry the store lacks refusing the version
naming the output and its build identity with nothing claimed or put,
and a landing with no verdicts filling nothing.

#### Scenario: S1 with the outputs read from the cache

- **WHEN** a world's store holds the entries of every build identity its landing's passing verdicts name, and the landed revision is published
- **THEN** each entry was rendered before the one ensure, the version is offered, and the answer names every digest (S1)

#### Scenario: An entry missing from the store

- **WHEN** a world's store lacks the entry of one output's build identity and the landed revision is published
- **THEN** the publish is refused naming that output and build identity, no tag is claimed, and the store records no put

### Requirement: The push before a landing over the fakes

The cache's group SHALL hold the push of what a checkout left, over the
fakes, below the push's scenarios, citing 52, 53 and 54: a checkout's
passing builds written with the landing asked nothing, a failed one not
written, a push repeated writing nothing, and a world that holds none of
the outputs publishing what another world pushed before its landing.

#### Scenario: Pushed before the landing, published elsewhere

- **WHEN** one world's build scripts a checkout as having left the passing build identities a later landing's verdicts name, that world pushes what the checkout left, and a second world sharing only its store, whose build holds nothing, publishes the landed revision
- **THEN** the second world's projection renders exactly those build identities, the version is offered, and the first world's landing fake was asked nothing

#### Scenario: A failed build identity left

- **WHEN** a checkout is scripted as having left a failed build identity beside a passing one, and what it left is pushed
- **THEN** only the passing one's entry is written
