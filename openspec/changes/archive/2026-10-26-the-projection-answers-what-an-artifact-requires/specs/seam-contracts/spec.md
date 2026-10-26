## MODIFIED Requirements

### Requirement: One interface per seam
Each seam of Part B SHALL be exactly one interface holding that seam's
operations as the statement names them and no other: the store (66 to
72), the build (74, 75), the landing (77, 78), the signer (79, 80), the
policy (82), a projection (84 to 86, 137, 140), a target (88 to 91) and
the keeper (120, 121). A profile implements one interface and nothing of
the core (core.2).

#### Scenario: Each seam's operations
- **WHEN** a reader lists the operations of each seam's interface
- **THEN** the store has put-bytes, put-artifact, attach, claims-of, name, resolve and read; the build has ensure and report; the landing has landed and verdicts-of; the signer has sign and verify; the policy has policy-for; a projection has pack, render, serve, pinned, requires and satisfying; a target has desired, current, apply and report; the keeper has append and chain; and no interface has any other

#### Scenario: The seams name no tool
- **WHEN** the seams' code is searched for the names core.1 rules out
- **THEN** none appears, and the code imports nothing of any profile

### Requirement: The types that cross a seam

Every value an operation takes or gives SHALL be a type defined with the
seams, in the statement's words, and nowhere else: digest, revision,
branch, component, output, build identity, version, ecosystem and
ecosystem name, artifact metadata and index metadata (4, 6), cache
entry (52), claim and signed envelope (7, 8), verdict (78), signer,
audience and the policy's requirements (82), environment and channel,
and a manifest's range and the requirement that pairs an ecosystem name
with one (140). Every such type SHALL be readable by a program (57). A
cache entry SHALL be named by its build identity and its digest alone,
and SHALL carry no ecosystem, name or version (53). A range and a
requirement SHALL carry the text as the manifest spells it, unchanged.

#### Scenario: A value written and read back

- **WHEN** any value of these types is written in its machine-readable form and read back
- **THEN** it is the same value

#### Scenario: A cache entry is no artifact

- **WHEN** a cache entry is written in its machine-readable form
- **THEN** it names a build identity and a digest and nothing else, and reading it as artifact metadata is rejected

#### Scenario: A requirement keeps the manifest's spelling

- **WHEN** a requirement naming a scoped name and a range with an operator and a prerelease is written in its machine-readable form
- **THEN** it names that name and that range, character for character, and reads back as the same requirement

### Requirement: A projection's contract
A projection SHALL answer pack, given an output at a build identity and
the version where the bytes are an artifact's, with the ecosystem's own
bytes, their metadata only where a version was given, and whether the
same outputs and version, or the same build identity, pack to the same
bytes (84, 87, 100); render, given artifacts and their metadata or cache
entries by build identity, with a consumer layout and the versions or
build identities it holds (85); serve, given a layout, with where
its clients read it (86); pinned, given a checkout, with the
ecosystem names and versions its lockfile pins as the lockfile spells
them, or absent where the checkout holds no lockfile of that ecosystem
(137); requires, given an artifact of its ecosystem a layout holds, with
every name of that ecosystem and the range its manifest requires of it,
as the manifest spells them (140); and satisfying, given a range and
versions, with those versions that satisfy the range by the ecosystem's
own rules, newest first by the ecosystem's own ordering, never by
spelling or by the order given (23, 140). A projection whose ecosystem
has no manifest SHALL answer requires and satisfying with none, an empty
list, reading nothing, and never absent (140). The core SHALL learn what
a manifest requires and how versions order only through these two
operations, and SHALL never read a manifest or order versions itself
(140, core.1).

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

#### Scenario: A checkout with no lockfile
- **WHEN** pinned is asked of a checkout holding no lockfile of the projection's ecosystem
- **THEN** the answer is absent naming the lockfile, never refused and never an empty list of pins

#### Scenario: A pin written and read back
- **WHEN** a pin, an ecosystem name and a version, is written in its machine-readable form and read back
- **THEN** it is the same pin

#### Scenario: What a held version requires
- **WHEN** requires is asked of an artifact whose manifest requires two names of the ecosystem, each with a range
- **THEN** the answer is those two names, each with its range as the manifest spells it

#### Scenario: Versions that satisfy a range, newest first
- **WHEN** satisfying is given a range and versions listed in neither the ecosystem's order nor spelling order
- **THEN** the answer is the versions that satisfy the range, newest first by the ecosystem's ordering, and no other

#### Scenario: An ecosystem with no manifest
- **WHEN** a projection whose ecosystem has no manifest is asked requires of an artifact, or satisfying of a range
- **THEN** each answer is none, an empty list, never absent, refused or unreadable
