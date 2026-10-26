## MODIFIED Requirements

### Requirement: One interface per seam
Each seam of Part B SHALL be exactly one interface holding that seam's
operations as the statement names them and no other: the store (66 to
72), the build (74, 75), the landing (77, 78), the signer (79, 80), the
policy (82), a projection (84 to 86, 137), a target (88 to 91) and the
keeper (120, 121). A profile implements one interface and nothing of
the core (core.2).

#### Scenario: Each seam's operations
- **WHEN** a reader lists the operations of each seam's interface
- **THEN** the store has put-bytes, put-artifact, attach, claims-of, name, resolve and read; the build has ensure and report; the landing has landed and verdicts-of; the signer has sign and verify; the policy has policy-for; a projection has pack, render, serve and pinned; a target has desired, current, apply and report; the keeper has append and chain; and no interface has any other

#### Scenario: The seams name no tool
- **WHEN** the seams' code is searched for the names core.1 rules out
- **THEN** none appears, and the code imports nothing of any profile

### Requirement: A projection's contract
A projection SHALL answer pack, given an output at a build identity and
the version where the bytes are an artifact's, with the ecosystem's own
bytes, their metadata only where a version was given, and whether the
same outputs and version, or the same build identity, pack to the same
bytes (84, 87, 100); render, given artifacts and their metadata or cache
entries by build identity, with a consumer layout and the versions or
build identities it holds (85); serve, given a layout, with where
its clients read it (86); and pinned, given a checkout, with the
ecosystem names and versions its lockfile pins as the lockfile spells
them, or absent where the checkout holds no lockfile of that ecosystem
(137).

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

## ADDED Requirements

### Requirement: The publisher's layout policy is a kind of claim
The claim kinds SHALL include the publisher's layout policy: a claim
about a version's index whose body is the claims a layout must require,
each with the signers that may make it and what each may claim, as the
publishing repository declared them at the revision the claim names
(113). It SHALL be written under its own predicate type as every other
kind is.

#### Scenario: A layout-policy claim written and read back
- **WHEN** a layout-policy claim is made about an index, signed and read back from its envelope
- **THEN** it is a claim of the layout-policy kind about that index, naming the revision and no build identity, with the required claims as made
