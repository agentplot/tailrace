# Spec Delta

## Purpose

The step after a landing that writes to the store the cache entries of
every build the landing's verdicts passed, from the archives the
landing's build left on the host, so any host whose build identity
matches replays them (52, C.3); it offers nothing and keeps nothing.

## ADDED Requirements

### Requirement: The push writes the entries of a landing's passing builds

Given a revision its branch has landed, the push SHALL read the
landing's verdicts and write the cache entry of each build identity a
passing verdict names, once, in the order the verdicts name them, from
the archive the landing's build left on this host (52, 78, C.3). A
build identity named only by verdicts that did not pass SHALL NOT be
written.

#### Scenario: A landing's archives pushed

- **WHEN** a revision is landed with verdicts that pass on two build identities and fail on a third, and the push is asked for it
- **THEN** the store's cache names of the two passing build identities name the bytes packed for them, and nothing is packed or written for the third

#### Scenario: One build identity named by two verdicts

- **WHEN** two passing verdicts name one build identity
- **THEN** its entry is written once and answered once

### Requirement: The push asks the landing first

The push SHALL ask the landing whether the branch landed the revision
before anything else is read, and SHALL refuse a revision it has not
landed, naming the branch and the revision, with nothing packed or
written (77).

#### Scenario: A revision not landed

- **WHEN** the push is asked for a revision its branch has not landed
- **THEN** it is refused naming the branch and the revision, and the store and every projection are asked nothing

### Requirement: Each build identity is answered on its own

The push SHALL answer each build identity it writes on its own: written
with the entry's digest, held already naming the digest the store's
name is at, absent where this host holds no archive for it, or the
refusal or unreadable answer its write gave; one build identity's answer
SHALL NOT stop the next. An absent archive SHALL refuse nothing (54).

#### Scenario: An archive not on this host

- **WHEN** one of a landing's passing build identities has no archive on the host and the push is asked
- **THEN** that build identity is answered absent naming where it looked, every other is written, and the push is no refusal

#### Scenario: A write that is refused

- **WHEN** the write of one build identity's entry is refused
- **THEN** that build identity's answer is the refusal with the source's words, and the next build identity is still written

### Requirement: A push repeated writes nothing

A push of a landing whose entries the store already names SHALL answer
each as held already at the digest the name is at, whether this host or
another wrote it, and SHALL move no name and retry nothing (52, rule
core.6).

#### Scenario: The same landing pushed twice

- **WHEN** a landing is pushed and then pushed again
- **THEN** the second answers every build identity held already at the digest the first wrote, and the store holds after it what it held before it

### Requirement: The push offers nothing and keeps nothing

The push SHALL put no manifest, attach no claim, sign nothing, ask no
policy and move no name but each entry's own (53, 100), and SHALL write
nothing to the host's record: which entries a landing became is answered
by its verdicts and the store (61).

#### Scenario: Nothing offered

- **WHEN** a landing is pushed
- **THEN** the store was asked to put bytes and make each entry's name and nothing else, and the signer and the policy were asked nothing

#### Scenario: Nothing kept

- **WHEN** a landing is pushed
- **THEN** every file of the host's record holds after the push what it held before it, and no file is added

### Requirement: A landing on one host replays on another

A landing whose checks moon ran on one host, pushed there, SHALL replay
on a second host that shares the store and syncs the landed revision:
moon on the second host SHALL answer every task the landing ran from the
cache (52, C.3).

#### Scenario: Every task the landing ran replays

- **WHEN** sluice lands a revision on one host whose checks run moon tasks with and without declared outputs, the push runs there, and a clone on a second host sharing the store runs `tailrace sync` for that revision and then moon over the same tasks
- **THEN** moon's run report on the second host names every task the landing ran as cached, and each declared output holds the bytes the first host's build wrote
