# moon-cache-projection Specification

## Purpose
Binds the projection seam to moon's cache (C.11): a task's output
archive packed by its task hash, and moon's outputs directory rendered
from cache entries, so moon replays a task another host ran.

## Requirements

### Requirement: pack answers the archive moon left for a task hash

The moon cache projection SHALL pack an output at a build identity with
no version as the bytes of the archive moon left for that task hash in
the checkout's moon outputs directory, unchanged, with no metadata
(84, 100), and SHALL say it cannot promise the same bytes again (87).
An archive moon did not leave SHALL be absent naming it.

#### Scenario: An archive packed

- **WHEN** moon has left an archive for a task hash and the projection packs an output at that hash with no version
- **THEN** the answer is the archive's bytes, unchanged, with no metadata, and says it cannot promise the same bytes again

#### Scenario: No archive for the hash

- **WHEN** the projection packs an output at a task hash moon left no archive for
- **THEN** the answer is absent naming the archive, and nothing is written

### Requirement: The moon cache packs no artifact

The moon cache projection SHALL answer absent naming artifacts for a
pack given a version, and for a render of artifacts, since a cache
entry is never an artifact (53).

#### Scenario: A pack given a version

- **WHEN** the projection is asked to pack an output at a version
- **THEN** the answer is absent naming artifacts, and nothing is read

### Requirement: render writes each entry where moon reads it

The projection SHALL render cache entries by reading each entry's bytes
through the store seam, checking they are the entry's digest, and
writing them as the archive of its task hash in the checkout's moon
outputs directory, following a link there; it SHALL answer the task
hashes the directory then holds of those given (85). An archive the
directory holds already SHALL be left as it is.

#### Scenario: Entries rendered

- **WHEN** the projection renders two cache entries the store holds
- **THEN** moon's outputs directory holds an archive for each task hash with the entry's bytes, and the answer holds those two build identities

#### Scenario: A directory that is a link

- **WHEN** the checkout's moon outputs directory is a link to a directory every checkout of the host shares, and entries are rendered
- **THEN** the archives are written in the shared directory and the link is unchanged

#### Scenario: Bytes that are not the entry's

- **WHEN** the store answers bytes for an entry whose digest is not the entry's
- **THEN** that entry is refused naming both digests, and no archive is written for it

#### Scenario: An archive moon already holds

- **WHEN** an entry is rendered for a task hash whose archive the directory holds already
- **THEN** the archive is not written over, and the answer holds that build identity

### Requirement: A build identity that is no task hash is refused

The projection SHALL refuse to render an entry whose build identity is
not a task hash, a non-empty string of lower-case hexadecimal digits,
naming it, and SHALL write nothing for it, so no verdict can place a
file outside moon's outputs directory.

#### Scenario: A build identity with a path in it

- **WHEN** the projection is asked to render an entry whose build identity is `../escape`
- **THEN** the render is refused naming that build identity, and no file is written

### Requirement: moon replays a rendered entry

A task whose archive the projection rendered SHALL replay under moon in
a checkout where its inputs hash the same, restoring the outputs the
archive holds, with no run of the task's command.

#### Scenario: A task that passed in one checkout replays in a fresh one

- **WHEN** a task ran in one checkout, its archive was packed and written to a store as a cache entry, and a fresh clone of the same revision renders that entry and runs the task
- **THEN** moon reports the task as cached, and its outputs hold what the first checkout's run left

### Requirement: The moon cache keeps the projection's contract

The moon cache projection SHALL pass the projection group's scenarios
of a projection that packs cache entries.

#### Scenario: The projection group over the moon cache

- **WHEN** the projection group's cache-entry scenarios run over the moon cache projection
- **THEN** each passes

### Requirement: A cache has no lockfile
pinned SHALL answer absent for every checkout, naming the lockfile and
the projection, since moon's cache has no lockfile (137); sync fills it
from the landing's verdicts and never from a pin.

#### Scenario: Pins of a checkout
- **WHEN** the moon cache projection is asked for a checkout's pins
- **THEN** the answer is absent naming the lockfile, never refused and never unreadable

### Requirement: A cache has no manifest
requires SHALL answer none, an empty list, for every artifact, and
satisfying SHALL answer none for any range and any versions, since
moon's cache has no manifest; neither reads anything (140).

#### Scenario: What an artifact requires
- **WHEN** the moon cache projection is asked what an artifact requires
- **THEN** the answer is none, an empty list, never absent, refused or unreadable

#### Scenario: Versions that satisfy a range
- **WHEN** the moon cache projection is asked which of some versions satisfy a range
- **THEN** the answer is none, an empty list, never absent, refused or unreadable
