# conformance-suite Specification

## Purpose
The one conformance suite that holds every profile, the fakes among
them, to Parts A and B of the statement: scenarios that each cite the
clauses they exercise, grouped per seam and per part of the core, run
over whatever a caller binds, and reported alike for every profile (93,
core.33).

## Requirements

### Requirement: Every scenario cites the clauses it exercises

Every scenario of the suite SHALL carry the numbers of the clauses of
Parts A and B it exercises, as data the coverage check reads, and the
statement scenario it plays where it plays one. A scenario SHALL assert
what each clause it cites requires of the case it sets up; a clause it
does not assert is not cited (core.33).

#### Scenario: A scenario's citations are read

- **WHEN** the suite's scenarios are listed
- **THEN** each one names its group, its own name, the statement scenario it plays or none, and at least one clause number

#### Scenario: S1 names what it plays

- **WHEN** the publish group is listed
- **THEN** the scenario playing S1 says it plays S1 and cites, among others, 17, 19 and 20

### Requirement: Scenarios sit in groups that later units only add to

The suite SHALL hold its scenarios in groups: one per seam of Part B,
one for the binding, and one per part of the core (version, publish,
verify, status, surfaces, gate, ledger, cache, record, layouts). Each
group SHALL end at a marker, and a later unit SHALL add its scenarios
below its group's marker and change no scenario above it. A group with
nothing yet SHALL exist with its marker.

#### Scenario: The gate's group waits for its unit

- **WHEN** the gate's group is listed in bolt 1
- **THEN** it holds no scenario, and its marker is in place for bolt 4's gate unit to add below

#### Scenario: Every seam has a group

- **WHEN** the suite's groups are listed
- **THEN** there is one for each of the store, the build, the landing, the signer, the policy, a projection, a target and the keeper

### Requirement: A seam's group runs over any profile of that seam

A seam's group SHALL run over a rig that the caller supplies, made new
for each scenario, so that the fakes run it in the conformance crate
and a real profile runs it unchanged in its own tests. A scenario SHALL
reach the profile only through its seam's interface and the rig's setup
and observation.

#### Scenario: The store's group over the fake store

- **WHEN** the store's group runs over a rig binding the fake store
- **THEN** every scenario runs on a store of its own, and each passes

#### Scenario: One scenario's state is not another's

- **WHEN** one store scenario puts bytes and the next asks for the same digest
- **THEN** the next answers the digest not held

### Requirement: A failing profile is reported the same way, a fake included

The suite SHALL run every scenario of a group even after one fails and
SHALL report each failure naming the profile, the seam or part, the
scenario, the clauses it cites and why it failed. The report SHALL be
the same for a fake as for any profile, and a profile with any failure
fails the run (93).

#### Scenario: A store that holds bytes twice

- **WHEN** the store's group runs over a store profile, named for the test, that answers "not held already" for the same bytes put twice
- **THEN** the run fails with a report naming that profile, the store, the scenario for the same bytes put twice, clauses 1, 5 and 66, and what the store answered

#### Scenario: Every scenario is run

- **WHEN** two scenarios of a group fail
- **THEN** the report names both, and every other scenario of the group was run

### Requirement: A scenario that does not apply is named as not applicable

A scenario SHALL be not applicable to a profile only where its rig says
the profile does not do what the scenario stages, as a landing that
records no verdicts says so (C.6). The report SHALL name each such
scenario as not applicable, with the clauses it cites and the rig's
reason, and SHALL count it neither as passed nor as failed; a scenario
not applicable fails no run. A profile that cannot stage what it does
SHALL fail the scenario, never be reported not applicable.

#### Scenario: Not applicable in the report

- **WHEN** one scenario of a group of three says it does not apply to the rig and the other two pass
- **THEN** the run passes, and the report says two passed and one is not applicable, naming that scenario, its clauses and the reason

### Requirement: A source that cannot be reached is unreadable on every seam

Each seam's group SHALL hold its profile to answer unreadable, carrying
the source's own words unchanged, when the rig makes the source
unreachable, and never an empty answer, an absence or a refusal (58,
59).

#### Scenario: A landing that cannot be reached

- **WHEN** a revision is landed with no verdict staged, the rig makes the landing unreachable, and the revision's verdicts are read
- **THEN** the answer is unreadable naming the landing, with its words unchanged, and not an empty list, over every landing, one that records verdicts or not

### Requirement: The store's contract

The store's group SHALL hold a store to 66 to 73 and to the Part A
clauses its operations keep: the same bytes put twice are held once
under their own digest (1, 5); an artifact keeps the metadata that says
what it is (4); attestations are held beside their digest, all of them,
none altered (8, 10); a name moves only from where it is expected to be
(70, 71); bytes read back are the bytes put (73).

#### Scenario: The same bytes put twice

- **WHEN** the same bytes are put twice
- **THEN** both answers name the digest of those bytes, the second says they were held already, and neither is an error

#### Scenario: An artifact read back

- **WHEN** an artifact's metadata and the digest of its bytes are put, and the artifact's digest is read
- **THEN** the store answers the same ecosystem, name, version and bytes' digest

#### Scenario: Two attestations on one digest

- **WHEN** two attestations are attached to one digest
- **THEN** claims-of answers both, each byte for byte as attached, and a digest the store does not hold is answered not held

#### Scenario: A name moved from where it is not

- **WHEN** a name is made from nothing at one digest, then moved with an expected digest it is not at
- **THEN** the move is refused naming the digest the name is at, and resolve still answers that digest

#### Scenario: A name that names nothing

- **WHEN** a name never made is resolved
- **THEN** the store answers that it names nothing, not refused and not unreadable

#### Scenario: Bytes read back

- **WHEN** bytes are put and read back by their digest
- **THEN** the bytes read have that digest, and a digest never put is answered not held

#### Scenario: A store that cannot be reached

- **WHEN** the rig makes the store unreachable and bytes are read
- **THEN** the answer is unreadable, carrying the store's own words unchanged, and never empty or not held

### Requirement: The build's contract

The build's group SHALL hold a build to 74 to 76: every output ensured
in one asking or the unmade ones named, and the same revision giving
the same build identity on every host.

#### Scenario: Outputs ensured and reported

- **WHEN** every declared output of a revision is ensured in one asking and each is reported
- **THEN** each report names a build identity, a pass and where the output's files are

#### Scenario: Outputs the build could not make

- **WHEN** two of three outputs cannot be made
- **THEN** ensure is refused naming exactly those two

#### Scenario: The same revision on a second host

- **WHEN** one revision's output is reported on the rig's host and on a second host the rig gives
- **THEN** both name the same build identity

### Requirement: The landing's contract

The landing's group SHALL hold a landing to 77 and 78. The rig SHALL
stage a landed revision from each verdict's check, build identity,
outcome and signer, and the landing SHALL make the envelope, never the
suite (12, core.8). The rig SHALL say whether its landing records
verdicts; for one that records none (C.6), verdicts as the landing
recorded them SHALL be not applicable.

#### Scenario: Landed and not landed

- **WHEN** the landing is asked of a revision its branch holds and of one it does not
- **THEN** it answers yes for the first and no for the second, both as answers, neither refused

#### Scenario: Verdicts as the landing recorded them

- **WHEN** a revision is landed with a passing and a failing verdict recorded, each naming its check, build identity, outcome and signer, and its verdicts are read
- **THEN** the answer holds each recorded verdict once, carrying the four as recorded, and each verdict carries an envelope of the in-toto payload type, with a payload, whose signature names the verdict's signer

#### Scenario: A revision with no verdict

- **WHEN** a revision is landed with no verdict recorded, and its verdicts are read
- **THEN** the answer is an empty set, neither absent nor unreadable, over every landing, one that records verdicts or not

#### Scenario: A landing that records no verdicts

- **WHEN** the landing's group runs over a rig that says its landing records no verdicts
- **THEN** the report names verdicts as the landing recorded them as not applicable, with the rig's reason, every other scenario of the group runs, and the run passes when they do

### Requirement: The signer's contract

The signer's group SHALL hold a signer to 79 to 81: a claim signed in
the identity at hand and about one digest (7), verify naming which
named signer made an attestation or that none did, and an expired
identity told as expired.

#### Scenario: Signed and verified

- **WHEN** a claim about one digest is signed and the envelope verified against signers naming the signing identity
- **THEN** verify names that signer, and the envelope holds one signature about that one digest

#### Scenario: No identity at hand

- **WHEN** a claim is signed with no identity at hand
- **THEN** the answer is absent naming the identity, and nothing is signed

#### Scenario: An envelope changed after signing

- **WHEN** an envelope's payload is changed and the envelope verified
- **THEN** verify is refused, saying none of the named signers made it

#### Scenario: A signer the policy does not name

- **WHEN** an envelope is verified against signers that do not include the one that signed it
- **THEN** verify is refused naming the signers it checked

#### Scenario: An expired identity

- **WHEN** an envelope signed by an identity the rig then expires is verified
- **THEN** verify names that signer and says it has expired, never valid and never absent

### Requirement: The policy's contract

The policy's group SHALL hold a policy to 82 and 83: for each audience,
the claims required with the signers that may make each and the kinds
and checks each may claim, binding no signer by output or ecosystem
(116); and, where the repository declares no policy for an audience,
an absence naming the audience. The revision beside it is verify's to
name, not the policy's (82).

#### Scenario: The claims an audience requires

- **WHEN** the policy for acceptance is asked at a revision whose policy requires a provenance from one signer and a verdict for one check from another
- **THEN** it answers both requirements, each with its signers and what each may claim

#### Scenario: Acceptance names no channel

- **WHEN** the policy for acceptance and for a channel bound to another are asked
- **THEN** acceptance answers no channel, and the bound channel's answer names the channel it is bound to

#### Scenario: No policy for an audience

- **WHEN** the policy for a layout is asked where none is declared
- **THEN** the answer is absent naming the audience, never refused

### Requirement: A projection's contract

The projection's group SHALL hold a projection to 84 to 87, and to the
parts of 48 and 50 a projection keeps: a layout rendered again from the
same artifacts is the same layout, and holds the versions it was given
and no other.

#### Scenario: Packed the same again

- **WHEN** one output is packed twice with one version
- **THEN** both answers carry the same bytes and metadata naming the ecosystem, name and version, or the projection says it cannot promise the same bytes

#### Scenario: A layout of the versions given

- **WHEN** a layout is rendered from two artifacts and rendered again from the same two
- **THEN** both renders hold those two versions and no other, and are the same layout

#### Scenario: Served where the clients read no files

- **WHEN** a projection whose clients read no files serves a rendered layout
- **THEN** it answers an address its clients read

### Requirement: A target's contract

The target's group SHALL hold a target to 88 to 91, and to the parts of
31, 40 and 43 a target keeps: what an environment should run is its
channel's head, an empty channel is an answer, and what it runs is
reported by digest with when it last converged.

#### Scenario: What an environment should run

- **WHEN** an environment's channel names a digest, and another's names nothing
- **THEN** the first answers that digest and the second answers empty, not an error

#### Scenario: An environment that cannot be read

- **WHEN** what an environment runs cannot be read
- **THEN** the answer is unreadable with the reason, never a digest

#### Scenario: Converged and reported

- **WHEN** a verified version is applied to an environment
- **THEN** the environment's current digest and its report name that version's digest and when it converged

### Requirement: The keeper's contract

The keeper's group SHALL hold a keeper to 120 to 122, and to 34, 35 and
105 as a keeper keeps them: promotions only appended, each after the
claim it names as predecessor, a stale predecessor refused naming the
head, a fork reported and never chosen.

#### Scenario: A chain appended in order

- **WHEN** a first promotion is appended with no predecessor and a second naming the first
- **THEN** the chain answers both, in that order, to two readers alike

#### Scenario: A stale predecessor

- **WHEN** a promotion names as its predecessor a claim that is no longer the head
- **THEN** the append is refused naming the claim at the head, and the chain is unchanged

#### Scenario: A fork

- **WHEN** the rig holds two promotions naming one predecessor and the chain is read
- **THEN** the chain is refused naming both claims, and neither is chosen

#### Scenario: A channel with no promotion

- **WHEN** the chain of a channel never appended to is read
- **THEN** the keeper answers that none is held, not an error

### Requirement: The binding

The binding's group SHALL hold the binding to 92, and SHALL hold the
suite itself to 93 by running a seam's group over a profile that breaks
its contract and finding it reported as failed.

#### Scenario: One of each, and any number of projections and targets

- **WHEN** one store, build, landing, policy and keeper, two signers, two projections and two targets are bound
- **THEN** each projection is found by its ecosystem and each target by its environment

#### Scenario: A broken profile fails the suite

- **WHEN** the store's group runs over a store that holds other bytes under a digest
- **THEN** the run fails, reported as that profile's failure

### Requirement: S1 over the fakes

The publish group SHALL play S1 over the fakes, an in-memory store and
a git history: a landed revision with two outputs and a landing
verdict naming their build is published.

#### Scenario: S1, a landed revision published

- **WHEN** a revision landed with a verdict naming its build is published, its two outputs read from the build and packed
- **THEN** both artifacts carry the version history derives, the index is put after both artifacts and their claims, each artifact has one provenance in the host's identity naming the revision and build identity with the landing's verdict carried unchanged, the version's name is made at the index, the answer names every digest and where each claim is attached, and nothing else moved: no other name, no keeper append and no target call

### Requirement: S2 over the fakes

The publish group SHALL play S2: the same publisher publishes the same
revision again.

#### Scenario: S2, the same revision again

- **WHEN** the revision published in S1 is published again
- **THEN** the store records no put, attach or name move, the history and the host's record are unchanged, and the answer equals the first

### Requirement: Publish's refusals over the fakes

The publish group SHALL hold publish to 14 to 18, 102 and 111 over the
fakes: what has not landed, what did not pass, what no verdict names,
a publish that stops before its index, a store that names bytes
wrongly, and a repeat that packs differently.

#### Scenario: A revision not landed

- **WHEN** a revision the branch has not landed is published
- **THEN** the publish is refused naming the branch, and no seam but the landing is asked

#### Scenario: An output that did not pass

- **WHEN** one of two outputs is unmade, or its build reports a failure, or only a failing verdict names its build
- **THEN** the version is refused whole naming that output, no name is claimed, and nothing is offered

#### Scenario: A build no verdict names

- **WHEN** the landing carries verdicts and none names an output's build identity
- **THEN** the publish is refused naming the output and the identities the verdicts carry

#### Scenario: A publish that stops before its index

- **WHEN** the store refuses the index after the artifacts were put
- **THEN** the version's name names nothing, and no version exists

#### Scenario: A store that names bytes wrongly

- **WHEN** the store answers a digest other than the bytes' own
- **THEN** the publish is refused naming both digests

#### Scenario: A repeat that packs differently

- **WHEN** a revision is published again with a projection that now packs other bytes
- **THEN** the publish is refused naming the artifact and both digests, and the version's name stays with the first index

#### Scenario: Channels declared or not

- **WHEN** one revision is published in two worlds alike but that one's policy declares a channel
- **THEN** the two answers are equal, and neither world's keeper or target is asked

### Requirement: S5 over the fakes, as far as verify

The verify group SHALL play S5 to verify's answer: a version offered
with a verdict signed by a signer the policy does not name (record
0012).

#### Scenario: S5, a verdict from a signer the policy does not name

- **WHEN** a version whose carried verdict was signed by a signer the policy for acceptance does not name is verified
- **THEN** verify names the index by digest, lists that verdict with the signers it checked and as not counted, keeps it attached, moves no name and appends nothing, and the version reads back by digest as it was offered

### Requirement: S13 over the fakes, as far as verify

The verify group SHALL play S13 to verify's answer: a repository that
runs no landing gate publishes with no verdicts.

#### Scenario: S13, no landing gate

- **WHEN** a revision whose landing carries no verdicts is published, then verified under a policy that requires a verdict and under one that requires provenance alone
- **THEN** the publish offers on the build's pass alone, the first verify finds no verdict to count, and the second counts the provenance

### Requirement: Verify's answers over the fakes

The verify group SHALL hold verify to 9, 10, 58, 59, 65, 81, 116 and
117: the policy read at the version's landed revision, an expired
signer told and still counting, a signer beyond its reach not
counting, a changed attestation from no named signer, and a store that
cannot be read answered as unreadable.

#### Scenario: The policy read at the landed revision

- **WHEN** a version is verified after the policy file was edited in the checkout and in a later commit
- **THEN** the policy is asked once, at the version's revision, after the landing said it is landed

#### Scenario: An expired signer

- **WHEN** a version's provenance signer has expired since it signed
- **THEN** verify says the claim was made by that signer and that it has expired, and the claim counts

#### Scenario: A signer beyond its reach

- **WHEN** a signer the policy names for provenance alone signed a verdict
- **THEN** the verdict is listed, made by that signer, and not counted

#### Scenario: A changed attestation

- **WHEN** an attestation attached to an artifact was changed after signing
- **THEN** verify lists it as made by no named signer, not counted, and it stays attached

#### Scenario: Claims that cannot be read

- **WHEN** the store cannot be read for an artifact's claims
- **THEN** verify answers unreadable with the store's words unchanged, never an empty list

### Requirement: S15 over a two-component fixture

The version group SHALL play S15 over a repository declaring two
components, each with its own paths and tag prefix, and SHALL hold the
version to 21 to 24, 65, 123 and 124.

#### Scenario: S15, a landing that touches one component

- **WHEN** a landing touches only the first component's paths and is published
- **THEN** the first component's next version is derived from its own tags, claimed under its prefix, and its index offered and named, while the second gets no version, no tag, no index and no name

#### Scenario: A path both components name

- **WHEN** a landing touches a path both components name
- **THEN** both components take a version

#### Scenario: No component declared

- **WHEN** a repository declares no component
- **THEN** it publishes as one component over the whole repository with the rule's prefix

#### Scenario: The same revision, two publishers

- **WHEN** two clones of the shared history derive the version of one landed revision
- **THEN** both derive the same version, and the build was never told it

#### Scenario: A name another revision holds

- **WHEN** a version's name is already claimed in the shared history against another revision
- **THEN** the publish is refused naming both revisions, before anything is packed

#### Scenario: Versions ordered by their rules

- **WHEN** a component holds the tag 1.10.0 and a tag 1.9.0 made after it, and a feature lands
- **THEN** the next version is 1.11.0

#### Scenario: A checkout's edit is not read

- **WHEN** the policy file is edited in the checkout and not landed
- **THEN** the components read are the landed revision's

### Requirement: Status over the fakes

The status group SHALL hold status to 58, 60, 61 and 62: answered from
what the host keeps, with no seam asked and nothing written.

#### Scenario: Status after a publish

- **WHEN** status is asked after a publish
- **THEN** it answers the component, version, revision and index published, no fake records a call after the publish's last, and no file of the record changes

#### Scenario: The record read with a file reader

- **WHEN** an entry of the host's record is read as plain JSON from its file
- **THEN** it names the component, version, revision, branch, index and store, and nothing the store or git holds besides

#### Scenario: Nothing kept, and an entry that cannot be read

- **WHEN** status is asked with no record, and then with an entry that does not parse
- **THEN** the first is absent naming the directory, and the second unreadable naming the file

### Requirement: The surfaces answer from the one core

The surfaces group SHALL hold a surface to 56 and 57: publish, verify
and status asked through the surface answer, as a program reads them,
exactly what the core answers. It SHALL run over the command line.

#### Scenario: The command line answers what the core answers

- **WHEN** publish, verify and status are asked through the command line with JSON output over the fakes
- **THEN** each printed answer equals the core's answer to the same question, value for value, naming digests, versions and verdicts as the statement names them

### Requirement: A verdict's signer by the landing's own name

The landing's group SHALL hold a landing that records verdicts to
answer each verdict's signer by the name the landing gives that signer,
the one name tailrace's policy names it by, and never by a second name
(104). For a landing that records no verdicts, the scenario SHALL be
not applicable. With this scenario, 104 SHALL leave the register
(core.33).

#### Scenario: Two signers, each by its own name

- **WHEN** a revision is landed with one verdict recorded by the signer `box` and another by the signer `forge`, and its verdicts are read
- **THEN** the verdict recorded by `box` names `box` and every signature its envelope holds is by `box`, the one recorded by `forge` names `forge` likewise, and no verdict names a signer that recorded none

#### Scenario: 104 no longer waits

- **WHEN** the coverage check runs
- **THEN** 104 is cited by the landing's group and is not in the register
