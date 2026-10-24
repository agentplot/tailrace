# Spec Delta

## MODIFIED Requirements

### Requirement: The types that cross a seam

Every value an operation takes or gives SHALL be a type defined with the
seams, in the statement's words, and nowhere else: digest, revision,
branch, component, output, build identity, version, ecosystem and
ecosystem name, artifact metadata and index metadata (4, 6), cache
entry (52), claim and signed envelope (7, 8), verdict (78), signer,
audience and the policy's requirements (82), environment and channel.
Every such type SHALL be readable by a program (57). A cache entry
SHALL be named by its build identity and its digest alone, and SHALL
carry no ecosystem, name or version (53).

#### Scenario: A value written and read back

- **WHEN** any value of these types is written in its machine-readable form and read back
- **THEN** it is the same value

#### Scenario: A cache entry is no artifact

- **WHEN** a cache entry is written in its machine-readable form
- **THEN** it names a build identity and a digest and nothing else, and reading it as artifact metadata is rejected

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
