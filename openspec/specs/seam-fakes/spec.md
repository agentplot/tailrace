# seam-fakes Specification

## Purpose
One in-memory fake for each seam, keeping the contract its seam states
and scriptable per test, so timber's core and its conformance suite
run with no registry, task runner, merge queue, key or target on the
host.

## Requirements

### Requirement: One fake per seam

There SHALL be exactly one fake for each of the eight seams, each
implementing that seam's interface and no other (core.2), holding its
state in memory, and needing nothing outside the process: no network,
no file outside a test's own, no key and no tool.

#### Scenario: The core bound to fakes alone

- **WHEN** a test binds every seam to its fake and asks each fake every one of its operations
- **THEN** each answers with no network, no task runner, no merge queue and no key reached

### Requirement: A fake keeps its seam's contract

A fake's answers SHALL keep what Part B says of its seam, so that a
fake passes the conformance suite as a profile must (93): bytes held by
digest and never altered (73), the same bytes put twice held once (66,
5), a name moved only from where it is expected (70), a chain never
reordered (122).

#### Scenario: The same bytes put twice

- **WHEN** the fake store is given the same bytes twice
- **THEN** the second answer names the same digest and says the bytes were held already, and the store holds them once

#### Scenario: A name moved from where it is not

- **WHEN** the fake store is asked to move a name from a digest it is not at
- **THEN** it refuses naming the digest the name is at, and the name stays

#### Scenario: Attestations travel with their digest

- **WHEN** an attestation is attached to a digest the fake store holds
- **THEN** claims-of that digest answers it as attached, and claims-of a digest the store does not hold answers absent

#### Scenario: The keeper refuses a stale predecessor

- **WHEN** two promotions are appended to one channel naming the same predecessor
- **THEN** the first is held at the head and the second is refused naming it

### Requirement: Scriptable per test

Each fake SHALL be set up by the test that uses it, with what its seam
holds (the revisions a branch has landed and their verdicts, the
outputs a build leaves and their identities and outcomes, the policy
for each audience, each channel with its component, and how each
channel moves, the signers and their identities, a projection's
ecosystem, an environment's channel head and what it runs), and SHALL
share that state with every copy the test hands to the core.

#### Scenario: A landing scripted for one test

- **WHEN** a test scripts the fake landing to have landed one revision on a branch, with two verdicts
- **THEN** landed answers yes for that revision and no for any other, and verdicts-of answers the two verdicts as scripted

#### Scenario: State seen through a copy

- **WHEN** a test keeps one handle to a fake store and puts bytes through a copy of it
- **THEN** the test's handle sees the bytes held

#### Scenario: The fake policy's channels

- **WHEN** a test scripts the fake policy with policies for `beta` of `lib`, which it says moves on each publish, `stable` of `lib` bound to `beta`, and `edge` of no declared component at one revision only
- **THEN** channels at that revision answers `beta` of `lib` moving on each publish, `edge` moving by promotion alone and `stable` of `lib` bound to `beta`, and at another revision only the two of `lib`, with the call recorded and faultable like policy-for

#### Scenario: A scripted following channel that takes prereleases

- **WHEN** a test scripts the fake policy with `nightly` moving on each publish with prereleases
- **THEN** channels answers `nightly` moving on each publish with prereleases

#### Scenario: A scripted given held off

- **WHEN** a test scripts the fake policy with `canary` held off, moving on each publish, with the line `[channels.canary]`, and scripts no other
- **THEN** the channels answer carries `canary` held, with how it would move and its line, apart from the channels in force, which do not include it, and a fake scripted with none held carries an empty list held, the one call recorded and faultable as before

#### Scenario: The fake keeper's two components

- **WHEN** a test appends a promotion to `beta` of `lib` on the fake keeper
- **THEN** the chain of `beta` of `lib` holds it, and the chain of `beta` of `web` and of `beta` of no declared component are absent

### Requirement: Scripted refusals and unreadable answers

A test SHALL be able to make any operation of any fake answer refused
or unreadable, with words of the test's choosing as the source's own,
once or until the test stops it, and the fake SHALL answer those words
verbatim (58, 59).

#### Scenario: A store that cannot be reached

- **WHEN** a test makes the fake store's claims-of unreadable with the words `connection refused`
- **THEN** claims-of answers unreadable carrying `connection refused` verbatim, and never an empty list

#### Scenario: A fault scripted once

- **WHEN** a test makes the next put-bytes refused, and put-bytes is asked twice
- **THEN** the first is refused with the scripted words and the second holds the bytes

### Requirement: Scripted faults the core must catch

The fakes SHALL let a test script what the core is bound to detect: a
store holding other bytes under a digest (2), a build leaving an output
unmade or failed (74, 16), a signer whose identity has expired (81) or
an attestation whose bytes were changed after signing, a projection
whose packing is not the same twice (87), and a keeper holding a fork
(105).

#### Scenario: Other bytes under a digest

- **WHEN** a test makes the fake store hold other bytes under a digest it holds
- **THEN** read of that digest answers those other bytes, so the core can refuse naming both

#### Scenario: An attestation changed after signing

- **WHEN** an envelope the fake signer made has its claim changed, and is verified against that signer
- **THEN** verify refuses it

#### Scenario: A fork held

- **WHEN** a test makes the fake keeper hold two claims naming one predecessor
- **THEN** chain refuses naming the fork and both claims

### Requirement: Every call recorded

Each fake SHALL record every operation it is asked, in order, with what
it was given and what it answered, readable by the test, so a test can
check what the core asked, in what order, and that nothing was written.

#### Scenario: Nothing written the second time

- **WHEN** the same bytes and artifact are put twice into the fake store
- **THEN** the record shows both asks, and the fake store's contents after the second are what they were after the first

#### Scenario: The order of asks

- **WHEN** two artifacts and then an index are put into the fake store
- **THEN** the fake store's record shows the index put after both artifacts

### Requirement: The fake projection packs cache entries or none

The fake projection SHALL pack an output given a build identity and no
version as a cache entry, whose bytes name the build identity and the
output's files and no ecosystem, name or version, and SHALL render
cache entries as a layout holding their build identities (84, 85). A
test SHALL be able to script it to pack no cache entries, after which
both answer absent for one.

#### Scenario: A cache entry packed by the fake

- **WHEN** the fake projection packs one output twice at one build identity with no version, and once at another
- **THEN** the first two answers are the same bytes with no metadata, the third is other bytes, and none of them names an ecosystem, a name or a version

#### Scenario: The fake renders cache entries

- **WHEN** the fake projection renders two cache entries
- **THEN** the layout holds those two build identities and no version

#### Scenario: The fake scripted to pack none

- **WHEN** a test scripts the fake projection to pack no cache entries
- **THEN** a pack with no version and a render of cache entries each answer absent, and a pack with a version answers as before

### Requirement: The fake projection answers the pins a test sets
The fake projection SHALL answer pinned, for a checkout, the pins and
ranges a test set for that checkout, each an ecosystem name with a
version or with a range, in the order set, and absent naming the
lockfile for a checkout given none; it SHALL render artifacts as a
layout holding their metadata, recording what it was given, and write
nothing (137, 85).

#### Scenario: Pins set by the test
- **WHEN** a test sets two pins for one checkout and asks the fake projection for that checkout's pins and another's
- **THEN** the first answer is the two pins in the order set, and the second is absent naming the lockfile

#### Scenario: A pin and a range set by the test
- **WHEN** a test sets a checkout's pins to `app` 1.0.0 and a range `^1` of `lib`
- **THEN** pinned of that checkout answers both, as set and in the order set

#### Scenario: The fake renders artifacts
- **WHEN** the fake projection renders two artifacts at a place
- **THEN** the layout holds those two artifacts' metadata and no other, and the test reads back the place and the artifacts it was given

### Requirement: The fake projection answers the requirements a test sets
The fake projection SHALL answer requires, for an artifact, the
requirements a test set for that artifact's digest, in the order set,
and absent naming the manifest of that artifact's name and version for
an artifact given none; it reads no bytes (140). requires SHALL be
recorded and can be faulted, as every operation of a fake is.

#### Scenario: Requirements set by the test
- **WHEN** a test sets two requirements for one artifact and asks the fake projection what that artifact and another require
- **THEN** the first answer is the two requirements in the order set, and the second is absent naming the other artifact's manifest

#### Scenario: requires faulted and recorded
- **WHEN** a test faults requires once and asks it
- **THEN** it answers the fault, and the fake's calls record requires with the artifact it was given

### Requirement: The fake projection orders versions by Semantic Versioning
The fake projection SHALL answer satisfying with the versions given
that satisfy the range by Semantic Versioning, newest first by its
precedence (23, 140). A range it cannot read SHALL be refused carrying
the range parser's own words (59), and a version it cannot read SHALL
satisfy no range. satisfying SHALL be recorded and can be faulted.

#### Scenario: Newest first, not by spelling
- **WHEN** the fake projection is asked which of `1.9.0`, `2.0.0`, `1.10.0`, `1.2.0` and `1.1.9` satisfy `^1.2`
- **THEN** it answers `1.10.0`, `1.9.0`, `1.2.0`, in that order

#### Scenario: A range it cannot read
- **WHEN** the fake projection is asked which versions satisfy a range that is not a Semantic Versioning range
- **THEN** the answer is refused carrying the parser's words, never absent and never an empty list

#### Scenario: satisfying faulted and recorded
- **WHEN** a test faults satisfying once and asks it
- **THEN** it answers the fault, and the fake's calls record satisfying with the range and versions it was given

### Requirement: The fake keeper answers as last read

The fake keeper SHALL keep, per channel, the chain as it last answered
it, and once made unreachable SHALL answer a channel it has answered
with that chain, the time it answered and the words it was made
unreachable with, and a channel it never answered unreadable with those
words; append SHALL then be unreadable (C.13, C.14). Every chain it
answers while reachable SHALL say it was read from the keeper itself.

#### Scenario: Unreachable after a read

- **WHEN** the fake keeper answers `stable`'s chain, is made unreachable with the words `ref unreadable`, and is asked again
- **THEN** it answers the same claims, the time of the first answer and `ref unreadable`, and an append is unreadable with those words

### Requirement: The fake landing answers as last fetched

The fake landing SHALL keep, as its copy, what it held when it last
answered, and once made unreachable SHALL answer every question from
that copy, saying so, with the time of that answer and the words it was
made unreachable with; made unreachable before it answered anything, it
SHALL answer unreadable with those words (77, C.5, C.6). While
reachable, every answer SHALL say it was read from the landing itself.

#### Scenario: Unreachable after a read

- **WHEN** the fake landing answers a landed revision, is made unreachable with the words `remote hung up`, and is asked again
- **THEN** it answers landed from the copy, with the time of the first answer and `remote hung up`

#### Scenario: A landing after the last answer

- **WHEN** the fake landing answers of `main`, lands another revision on it, is made unreachable, and is asked of that revision
- **THEN** it answers not landed from the copy, with the time of its last answer

#### Scenario: Unreachable before any answer

- **WHEN** the fake landing is made unreachable before it answered anything, and landed and verdicts-of are asked
- **THEN** both answer unreadable with the words it was made unreachable with

### Requirement: The fake projection serves its layouts
The fake projection SHALL serve a layout it rendered: a request whose
path names an ecosystem name and version the layout holds is answered
`200` with that artifact's metadata as its body; one naming a version the
layout lacks asks the core once and answers `200` where the core's answer
is held, and otherwise a status of `404`, `403` or `502` for absent,
refused or unreadable, with the answer's words as its body. It SHALL
record every request and every version it asked of the core, and a test
SHALL be able to script it to serve none, after which every request is
absent naming serving (86, 115).

#### Scenario: A version held and a version lacked
- **WHEN** the fake projection renders one version, then serves a request for it and a request for a second version the core answers refused
- **THEN** the first answer is `200` and asked the core nothing, the second is `403` carrying the refusal's words, and the test reads back both requests and the one version asked

#### Scenario: The fake scripted to serve none
- **WHEN** a test scripts the fake projection to serve none and asks it to serve a request
- **THEN** the answer is absent naming serving, and the core is asked nothing

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

### Requirement: The fake policy answers the environments a test sets

The fake policy SHALL answer environments, at a revision, the
environments a test set for that revision, ordered by component and then
name, each with its channel, whether it waits, its approvers and its
target's profile and settings, and an empty list at a revision given
none; the call SHALL be recorded and faultable as policy-for and
channels are (136).

#### Scenario: Environments set by the test

- **WHEN** a test sets `cache` of `web`, following `beta` of `web` with the target `nix_cache` and settings, at one revision, and asks environments at that revision and another
- **THEN** the first answer is `cache` of `web` as set, and the second an empty list

#### Scenario: Environments made to fail

- **WHEN** a test scripts environments to answer unreadable
- **THEN** environments answers unreadable carrying the scripted words, and the call is recorded

### Requirement: The fake target answers the head it is handed

The fake target SHALL answer desired with the head the core hands it,
as a target that is pushed to does (88), and SHALL hold, per
environment, what it runs, or that it cannot be read, as a test sets.
Every call SHALL be recorded.

#### Scenario: The head handed

- **WHEN** the fake target's desired is asked with a head answering `i2`, and with one answering the channel empty
- **THEN** the first answers `i2` and the second absent

### Requirement: The fake target holds no marker where a test says so

Where a test says the fake target holds no marker, current and report
SHALL answer unreadable naming why after any apply, never the digest
applied (119).

#### Scenario: A fake that holds no marker

- **WHEN** a test says the fake target holds no marker, and a version is applied to an environment
- **THEN** current and report answer unreadable naming why, and the apply is recorded

### Requirement: The fake target reads only the settings a test lets it

The fake target SHALL read only the settings keys a test lets it read,
answering desired, current and apply unreadable naming the first other
key of an environment's settings, with nothing applied (45).

#### Scenario: A settings key the fake does not read

- **WHEN** a test lets the fake read `cache`, and an environment's settings hold `cache` and `token`
- **THEN** apply answers unreadable naming `token`, and nothing is applied
