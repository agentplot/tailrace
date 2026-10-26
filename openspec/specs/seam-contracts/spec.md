# seam-contracts Specification

## Purpose
The eight seams of Part B as the one interface each by which tailrace's
core reaches a store, a build, a landing, a signer, a policy, a
projection, a target and a keeper, with the values that cross them and
the answers they give.

## Requirements

### Requirement: One interface per seam

Each seam of Part B SHALL be exactly one interface holding that seam's
operations as the statement names them and no other: the store (66 to
72), the build (74, 75), the landing (77, 78), the signer (79, 80), the
policy (82, 131), a projection (84 to 86), a target (88 to 91) and the
keeper (120, 121). A profile implements one interface and nothing of
the core (core.2).

#### Scenario: Each seam's operations

- **WHEN** a reader lists the operations of each seam's interface
- **THEN** the store has put-bytes, put-artifact, attach, claims-of, name, resolve and read; the build has ensure and report; the landing has landed and verdicts-of; the signer has sign and verify; the policy has policy-for and channels; a projection has pack, render and serve; a target has desired, current, apply and report; the keeper has append and chain; and no interface has any other

#### Scenario: The seams name no tool

- **WHEN** the seams' code is searched for the names core.1 rules out
- **THEN** none appears, and the code imports nothing of any profile

### Requirement: The types that cross a seam

Every value an operation takes or gives SHALL be a type defined with the
seams, in the statement's words, and nowhere else: digest, revision,
branch, component, output, build identity, version, ecosystem and
ecosystem name, artifact metadata and index metadata (4, 6), cache
entry (52), claim and signed envelope (7, 8), verdict (78), signer,
audience and the policy's requirements (82), the channel a policy
declares with how it moves (131), a given channel held off with the
line that would switch it on (C.10), environment and channel. A channel
SHALL be its component and its name together, its component none where
the repository declares no component (125). The name of the one
component of a repository that declares none (123) SHALL be defined
with the seams, so every profile and the core name it alike. Every such
type SHALL be readable by a program (57). A cache entry SHALL be named
by its build identity and its digest alone, and SHALL carry no
ecosystem, name or version (53).

#### Scenario: A value written and read back

- **WHEN** any value of these types is written in its machine-readable form and read back
- **THEN** it is the same value

#### Scenario: A cache entry is no artifact

- **WHEN** a cache entry is written in its machine-readable form
- **THEN** it names a build identity and a digest and nothing else, and reading it as artifact metadata is rejected

#### Scenario: A declared channel names its component

- **WHEN** a channel a policy declares is written in its machine-readable form
- **THEN** it names the channel's name, its component or that the repository declares none, and how it moves, naming the channel it is bound to where it is bound and whether it takes prereleases where it moves on each publish

#### Scenario: A channel spelled

- **WHEN** the channel `beta` of no declared component and the channel `beta` of `lib` are spelled
- **THEN** they are `beta` and `lib/beta`, and the two are not equal

### Requirement: A digest is a function of bytes alone

A digest SHALL be computed from bytes alone, and the same bytes SHALL
give the same digest on every host and in every store (1). A text
that is not a digest SHALL be rejected when read as one, naming the
text.

#### Scenario: Same bytes, same digest

- **WHEN** the digest of two equal byte sequences is computed
- **THEN** the two digests are equal, and differ from the digest of any other byte sequence in the tests

#### Scenario: A text that is not a digest

- **WHEN** a text with no algorithm, or a digest of the wrong length or with characters outside its alphabet, is read as a digest
- **THEN** it is rejected, naming the text

### Requirement: An artifact says what its bytes are

Artifact metadata SHALL name the ecosystem whose form it takes, its
name in that ecosystem and its version (4). Index metadata SHALL name
one component and one version, and an index SHALL name every artifact
of that component at that version by digest (6, 110).

#### Scenario: Metadata with a part missing

- **WHEN** artifact metadata is made with an empty ecosystem, name or version
- **THEN** it is rejected, naming the part that is missing, and no metadata is made

#### Scenario: An index names its artifacts

- **WHEN** index metadata is made for a component, a version and a set of artifacts
- **THEN** it names each artifact's digest and metadata, and every artifact it names carries that version

### Requirement: A claim is about one digest by one signer

A claim SHALL be about exactly one digest, SHALL name its kind and the
revision it was made for, and SHALL name the build identity where it
has one (7, 13). A signed envelope SHALL have a digest of its own by
which a promotion names its predecessor (105, 120). An envelope holding
a claim of tailrace's form SHALL hold exactly one claim and the
signature of exactly one signer, and one holding any other number of
signatures SHALL be read as no claim of tailrace's form (7). An
envelope carried from another party SHALL hold every signature it was
made with, each with its signer and its key id as made, and the payload
and its type byte for byte (12, C.5).

#### Scenario: A claim with no build identity

- **WHEN** a claim is made with a revision and no build identity
- **THEN** it is a claim, and reading it back answers no build identity rather than an empty one

#### Scenario: An envelope's digest

- **WHEN** the same signed envelope is held in two places
- **THEN** both answer the same envelope digest

#### Scenario: A claim carried inside another

- **WHEN** a claim holds another party's signed envelope as found
- **THEN** reading the claim back answers that envelope byte for byte as it was made (12)

#### Scenario: Another party's envelope with two signatures

- **WHEN** a claim carries an envelope signed twice, each signature with its own key id
- **THEN** reading the claim back answers both signatures with their key ids, in the order made

#### Scenario: Tailrace's form with two signatures

- **WHEN** an envelope holding a claim of tailrace's form carries two signatures
- **THEN** it is read as no claim of tailrace's form

### Requirement: Four answers, and no other

Every seam operation SHALL answer what was asked, or absent, refused or
unreadable, and nothing else: no operation answers an error of another
kind, an empty value for a failed read, or unknown (58, core.19).
Absent, refused and unreadable SHALL each name the seam and the
profile that answered.

#### Scenario: Absent is not refused

- **WHEN** a store is asked to read a digest it does not hold
- **THEN** the answer is absent, naming the digest, and never refused or unreadable

#### Scenario: A failed read is not empty

- **WHEN** a store cannot be reached while it is asked for the claims of a digest
- **THEN** the answer is unreadable, naming what could not be read, and never an empty list of claims

### Requirement: A refusal carries the source's own words

A refusal SHALL name what Part B says that refusal names, and an
unreadable answer SHALL name what could not be read; each SHALL carry
the refusing or failing source's own words verbatim, where it gave
any, beside tailrace's own (59, core.19).

#### Scenario: The source's words kept

- **WHEN** a store refuses with words of its own
- **THEN** the refusal carries those words unchanged, byte for byte, beside the reason tailrace gives

#### Scenario: What each refusal names

- **WHEN** a name is refused because it is at another digest (70), an append because its predecessor is not the head (120), an ensure because outputs could not be made (74), or a verify because no signer the policy names made the attestation (80)
- **THEN** the refusal names, in turn, the digest the name is at or that it is at nothing, the claim at the head, each output not made, and the signers it was checked against

### Requirement: The store's contract

The store SHALL answer put-bytes with the digest and whether it held
the bytes already (66, 5); put-artifact with the artifact's digest
(67); attach with the attestation held beside its digest (68, 8);
claims-of with every attestation as attached, or absent (69); name
with the name moved, or refused naming where it is (70); resolve with
a digest, or absent (71); and read with what the digest holds, or
absent (72).

#### Scenario: A name moved only from where it is expected

- **WHEN** name is asked to move a name from a digest it is not at
- **THEN** the answer is refused, naming the digest the name is at, and the name does not move

#### Scenario: A name made from nothing

- **WHEN** name is asked to move a name that names nothing, from nothing
- **THEN** the name is made

#### Scenario: An artifact read back

- **WHEN** read is asked for the digest put-artifact answered
- **THEN** it answers the artifact's metadata and the digests it names

### Requirement: The build's contract

The build SHALL answer ensure, given a revision and every output
declared in one asking, with those outputs present, or refused naming
each output it could not make (74); and report with the build identity
an output came from, the build's outcome, and where its files are (75).

#### Scenario: Outputs that could not be made

- **WHEN** ensure is asked for three outputs and the build can make two
- **THEN** the answer is refused, naming the one it could not make

### Requirement: The landing's contract

The landing SHALL answer landed with whether a branch has landed a
revision (77), and verdicts-of with every verdict recorded for a
revision, each naming its check, its build identity, its outcome and
its signer, with its signed envelope as the landing recorded it, and
where the landing found it, in the landing's own words (12, 78).

#### Scenario: A revision with no verdicts

- **WHEN** verdicts-of is asked for a revision the landing recorded no verdict for
- **THEN** it answers an empty set of verdicts, which is not absent and not unreadable

#### Scenario: Where a verdict was found

- **WHEN** verdicts-of answers a verdict
- **THEN** the verdict says where the landing found it

### Requirement: The signer's contract

The signer SHALL answer sign with the claim signed in the identity of
the party running tailrace, or absent when no identity is at hand
(79); and verify with which of the signers given made an attestation,
or refused when none did (80). A signature by an identity that has
expired SHALL be answered as made by that signer and expired, never as
current and never as absent (81).

#### Scenario: An expired signer

- **WHEN** verify is asked of an attestation whose signer's identity has expired
- **THEN** it answers that signer, marked expired

### Requirement: The policy's contract

The policy SHALL answer policy-for, given a repository at a landed
revision and an audience of acceptance, a channel with its component,
or a layout, with the claims required, each with the signers that may
make it and what each may claim (82), or absent where no policy is
declared. The policy for a channel SHALL name the signers that may sign
its promotion (32) and the signers that may ask for it, none meaning any
asker the gate verifies (129, C.10), and the policy for a channel bound to another SHALL
name the channel it is bound to; the policy for acceptance SHALL name no
channel (83). The policy SHALL answer channels, given a repository at a
landed revision, with every channel its policy declares or gives, once
each, ordered by name and then by component, each with the one
component it belongs to (125) and how it moves: on each accepted
publish, with releases alone or prereleases too, by a promotion alone,
or bound to a channel of the same component (37, 83, 131, C.10); a
repository that declares one component SHALL
have every channel in it, and one that declares none SHALL have every
channel in that one, its channels spelled by name alone (123). The
same channels answer SHALL carry, apart from the channels in force,
every channel the profile gives but holds until the repository switches
it on, and the repository has not switched on, marked held, once each,
ordered by name and then by component, each with its component, how it
would move once switched on, and the line that would switch it on, in
the profile's own words; an empty list where it holds none back; and
never a channel in force (131 as amended at agentplot/blueprints
1ae1cc5, C.10). The policy seam SHALL have no operation for the held
givens apart from channels (136).

#### Scenario: Acceptance names no channel

- **WHEN** policy-for is asked for acceptance
- **THEN** its answer names no channel, and requirements for acceptance cannot be made naming one

#### Scenario: The channels and policy-for agree on a binding

- **WHEN** channels answers a channel bound to another, and policy-for is asked for that channel at the same revision
- **THEN** both name the same channel it is bound to

#### Scenario: A channel's promoters

- **WHEN** policy-for is asked for a channel
- **THEN** it names the signers whose promotion of that channel counts, each with what it may claim

#### Scenario: A channel's approvers

- **WHEN** policy-for is asked for a channel whose policy names who may ask for it, and for one whose policy names none
- **THEN** the first names those signers, and the second names none, which is any asker the gate verifies

#### Scenario: A given held off

- **WHEN** the channels are asked at a revision whose policy holds a given the repository has not switched on
- **THEN** the one answer carries the channels in force and, apart from them, the held given marked held, naming its component, how it would move and the line that would switch it on, and no channel is both in force and held

#### Scenario: A channels answer written and read back

- **WHEN** a channels answer holding a channel in force and a given held off is written in its machine-readable form and read back
- **THEN** it is the same value, the held given naming the channel, its component or that the repository declares none, how it would move and its line, apart from the channels in force

### Requirement: A projection's contract

A projection SHALL answer pack, given an output at a build identity and
the version where the bytes are an artifact's, with the ecosystem's own
bytes, their metadata only where a version was given, and whether the
same outputs and version, or the same build identity, pack to the same
bytes (84, 87, 100); render, given artifacts and their metadata or cache
entries by build identity, with a consumer layout and the versions or
build identities it holds (85); and serve, given a layout, with where
its clients read it (86).

#### Scenario: A projection that cannot promise the same bytes

- **WHEN** pack answers bytes it cannot promise to make again
- **THEN** its answer says so

#### Scenario: An output packed as an artifact

- **WHEN** pack is given an output, a build identity and a version
- **THEN** its answer carries metadata naming the ecosystem, the name and that version

#### Scenario: An output packed as a cache entry

- **WHEN** a projection that packs cache entries is given an output and a build identity and no version
- **THEN** its answer carries the bytes and no artifact metadata, so nothing about it makes an offer

#### Scenario: A layout of cache entries

- **WHEN** a projection that packs cache entries renders a layout from cache entries
- **THEN** the answer names the build identities the layout holds, and no version

#### Scenario: A projection that packs no cache entries

- **WHEN** a projection that packs no cache entries is asked to pack an output with no version, or to render cache entries
- **THEN** each answer is absent naming cache entries, never refused and never unreadable

### Requirement: A target's contract

A target SHALL answer desired with the digest an environment's channel
names, or absent when the channel is empty (88); current with the
digest the environment runs, or unreadable with the reason (89); apply,
given a version the reconciler has verified, with the environment
converged or what stopped it (90); and report with what it runs and
when it last converged (91).

#### Scenario: An environment that cannot be read

- **WHEN** current is asked of an environment whose state cannot be read
- **THEN** the answer is unreadable with the reason, never a digest

### Requirement: The keeper's contract

The keeper SHALL answer append, given a channel with its component, a
promotion claim and the digest of the promotion it names as its
predecessor, with the claim held at the head of that channel's chain,
or refused naming the claim at the head (120); and chain, given a
channel with its component, with every promotion claim of it in order
from the first, or absent when none is held (121). Two components'
channels of one name SHALL be two chains (125). A chain that holds two
claims naming one predecessor SHALL be refused as a fork, naming both,
never answered by choosing one (105, 122). A chain answered from the
copy the host last read, because the source holding the ledger did not
answer, SHALL say when that copy was read and carry the source's words
of why it did not answer, a channel the copy holds no claim of answered
as a chain of no claims saying so, never as an absence; a chain read
from the source itself SHALL say so. An append SHALL never be held by
such a copy alone (60 as amended at agentplot/blueprints b64d367, 61,
C.13, C.14).

#### Scenario: An append whose predecessor is not the head

- **WHEN** append is given a predecessor that is not the head of the channel's chain
- **THEN** it is refused naming the claim at the head, and the chain is unchanged

#### Scenario: A chain read as last read

- **WHEN** the source holding a chain the host has read cannot be reached, and the chain is asked
- **THEN** the answer is the claims as last read, the time of that read and the source's words, and an append is unreadable

#### Scenario: An empty chain read as last read

- **WHEN** the host last read a channel's chain holding no claim, and the source holding it then cannot be reached
- **THEN** the answer is a chain of no claims, the time of that read and the source's words, and not absent

#### Scenario: Two components' chains of one name

- **WHEN** a promotion is appended to `beta` of `lib` and the chain of `beta` of `web` is asked
- **THEN** the chain of `beta` of `web` is absent

### Requirement: The core is handed one binding

The core SHALL be handed the seams as one binding holding exactly one
store, one build, one landing, one policy and one keeper, the signers
bound for the host, and any number of projections by ecosystem and
targets by environment (92), and SHALL reach no seam except through it
(core.2).

#### Scenario: A binding with two projections

- **WHEN** a binding is made with two projections and one target
- **THEN** each projection is found by its ecosystem and the target by its environment, and the binding holds one store, one landing and one keeper

### Requirement: The promotion claim's body

A promotion claim's body SHALL be one type defined with the ledger seam,
readable by a program and read back the same (57): the channel, the
component, the version and index it moves to, the version and index it
moves from or none for the channel's first, the digest of the promotion
it follows or none, who asked, the revision the channel's policy was
read at and the digests of the claims that satisfied it, and when, in
seconds since the Unix epoch (33, 39, 105). Reading a claim's body SHALL
answer none for a claim that is not a promotion of tailrace's form.

#### Scenario: A promotion's body read back

- **WHEN** a promotion claim is made with a body and its envelope is read back
- **THEN** the body read is the body made, value for value, and an acceptance claim's envelope reads as no promotion

### Requirement: A channel's name in the store

A channel's name in the store SHALL be `channel/` and the channel as it
is spelled: `channel/<channel>` where the repository declares no
component, and `channel/<component>/<channel>` where it declares
components; the same for the core and every ledger profile that walks
from it, and never the name of a version or a cache entry (3, 125,
C.14).

#### Scenario: Stable's name

- **WHEN** the store name of the channel `stable` of a repository that declares no component is asked
- **THEN** it is `channel/stable`

#### Scenario: A component's stable

- **WHEN** the store name of the channel `stable` of the component `lib` is asked
- **THEN** it is `channel/lib/stable`

### Requirement: The refusals a promotion gives

A promotion's refusal SHALL name, as Part A says each names: the first
claim found wanting, by its kind, its check where it is a verdict, and
the signers that may make it (58); the version and index a channel is
at, or that it is empty, where a promotion expected it elsewhere (35);
and the channel a bound channel is bound to, where the version was
never a subject of its chain (37).

#### Scenario: Each refusal read back

- **WHEN** each of the three refusals is written in its machine-readable form and read back
- **THEN** it is the same refusal, naming what it names
