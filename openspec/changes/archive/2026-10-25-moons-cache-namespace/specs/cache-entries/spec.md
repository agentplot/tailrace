# Spec Delta

## Purpose

Holds a build's cache entries in the store under their build identity,
the lowest level of trust, so any host whose build identity matches
reads what another host built, and fills a checkout's cache from the
landing's verdicts; a cache entry is never an artifact (52 to 54, 100).

## ADDED Requirements

### Requirement: A cache entry is written under its build identity

Writing a cache entry SHALL pack the output at its build identity with
no version through a projection that packs cache entries, put the
bytes, and make the store's cache name for that build identity at their
digest from nothing (52). It SHALL put no manifest, attach no claim and
move no other name, so writing an entry offers nothing (53, 100).

#### Scenario: An entry written

- **WHEN** a host writes the cache entry of an output at a build identity
- **THEN** the store holds the bytes the projection packed and the cache name of that build identity names their digest, and the store was asked to put no manifest, attach nothing and move no other name

#### Scenario: No projection packs cache entries

- **WHEN** a host writes a cache entry and no bound projection packs cache entries
- **THEN** the answer is absent naming cache entries, and nothing is written

### Requirement: An entry held already is answered, never written over

A cache entry for a build identity the store already names SHALL be
answered as held, naming the digest the name is at, whether those are
the bytes packed now or bytes another host packed; the name SHALL NOT
move and nothing SHALL be retried (52, rule core.6).

#### Scenario: The same entry written twice

- **WHEN** one host writes the same cache entry twice
- **THEN** the second answer is held already at the same digest, and the store holds the bytes once

#### Scenario: Another host's bytes held first

- **WHEN** a second host writes a cache entry for a build identity whose cache name already names other bytes
- **THEN** the answer is held already naming the digest the name is at, and the name still names it

### Requirement: The same bytes are one store object

Bytes a cache entry names that the store already holds, as an
artifact's or as another entry's, SHALL be put once and answered held
already, and the artifact that holds them SHALL be unchanged (100).

#### Scenario: An entry whose bytes an artifact holds

- **WHEN** a cache entry is written whose bytes are already an artifact's layer
- **THEN** the put answers held already, the artifact reads back as before, and nothing names the artifact that did not before

### Requirement: Sync renders the entries the landing's verdicts name

Sync SHALL read, for a revision its branch has landed, the build
identity of every verdict that passed, resolve each one's cache name in
the store, and render the entries it finds through every bound
projection that packs cache entries, at the checkout it runs in; a
projection that packs none SHALL be passed over (52, C.3).

#### Scenario: A task that passed on one host replays on another

- **WHEN** one host wrote the cache entries of a landed revision's passing builds and a second host, sharing the store, syncs that revision
- **THEN** the second host's layout holds exactly those build identities, and its answer names each as rendered

#### Scenario: A verdict that failed names nothing to render

- **WHEN** a landed revision's verdicts include one that failed
- **THEN** sync does not ask the store for that verdict's build identity

#### Scenario: A revision that has not landed

- **WHEN** sync is asked for a revision its branch has not landed
- **THEN** it is refused naming the branch and the revision, and the store is asked nothing

### Requirement: Sync defaults to the newest landed revision the checkout descends from

Sync SHALL, when it is given no revision, render for the newest revision
of the branch that the checkout's head descends from, its common
ancestor with the branch.

#### Scenario: A checkout ahead of the branch

- **WHEN** a checkout has commits of its own on a revision the branch landed, and sync is given no revision
- **THEN** sync renders the entries of that landed revision's verdicts

### Requirement: A cache entry that is gone refuses nothing

A cache entry SHALL be removable at any time. Sync SHALL answer an entry
whose name the store no longer holds as absent, naming its build
identity, render every other entry, and refuse nothing; nothing
accepted or released SHALL change because an entry is gone (54, rule
core.30).

#### Scenario: One entry removed

- **WHEN** one of a revision's cache entries is removed from the store and the revision is synced
- **THEN** the answer names that build identity as absent and every other as rendered, and the sync is no refusal

#### Scenario: S14, a removed entry builds again

- **WHEN** a build's cache entry is removed, the next landing's verdicts name a new build identity for that output, and that landing is published
- **THEN** the publish offers those outputs, and every version, index and name the store held before is held as before

### Requirement: A cache entry is never an artifact

No channel, index or policy SHALL name a cache entry, and reading one
SHALL offer nothing and accept nothing: sync reads no policy and writes
nothing to the store, and a publish's index names no digest by way of a
cache name (53, rule core.30).

#### Scenario: Sync writes nothing to the store

- **WHEN** a revision is synced
- **THEN** the store was asked only to resolve names and read digests, and the policy was not asked

#### Scenario: A publish after a cache entry

- **WHEN** a revision whose outputs have cache entries is published
- **THEN** the index names only artifacts the publish packed with a version, and no cache name is read or moved by the publish
