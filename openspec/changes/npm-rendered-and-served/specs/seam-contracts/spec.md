## MODIFIED Requirements

### Requirement: A projection's contract
A projection SHALL answer pack, given an output at a build identity and
the version where the bytes are an artifact's, with the ecosystem's own
bytes, their metadata only where a version was given, and whether the
same outputs and version, or the same build identity, pack to the same
bytes (84, 87, 100); render, given artifacts and their metadata or cache
entries by build identity, with a consumer layout and the versions or
build identities it holds (85); serve, given a layout, one request of the
ecosystem's own client with the base address it is served at, and a way
to ask the core for a version the layout lacks, with the response to
that request in the client's own protocol, or absent naming serving, for
every request, from a projection whose ecosystem's clients read files
(86, 115); pinned, given a checkout, with the ecosystem names and
versions its lockfile pins as the lockfile spells them, and, for a name
its manifest declares and no lockfile pins, the manifest's range as the
manifest spells it, or absent where the checkout holds no lockfile of
that ecosystem (137); and requires, given a held version's artifact,
with the family's names and ranges its manifest requires of that
ecosystem, as the manifest spells them, and whether one version is
within a range and which of two is newer by the ecosystem's own order,
or none from a projection whose ecosystem has no manifest (140, 23). A
projection SHALL never reach the store but through the store it is
handed, nor the policy or a version's claims at all: a version a served
layout lacks is asked of the core, which judges it, and the core, never
the projection, lists, judges and holds what a range or a requirement
names (core.2, core.1).

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
- **WHEN** a projection whose ecosystem has a manifest is asked what a held version requires
- **THEN** it answers the family's names and ranges as the version's manifest spells them, and reads nothing but that version's bytes through the store it is handed

#### Scenario: A request served
- **WHEN** a projection whose clients read no files is asked to serve a request for a version its layout holds
- **THEN** it answers the response its client reads, and asks the core for nothing

#### Scenario: A request for a version the layout lacks
- **WHEN** a projection is asked to serve a request for a version its layout lacks
- **THEN** it asks the core for that one version, by its ecosystem name and version, and answers the core's answer in its client's protocol: the version's response where the core rendered it, and otherwise a response carrying the refusal, the absence or the unreadable answer in the source's own words

#### Scenario: A projection whose clients read files
- **WHEN** a projection whose ecosystem's clients read files is asked to serve any request
- **THEN** the answer is absent naming serving, and the core is asked for nothing

#### Scenario: A request and a response written and read back
- **WHEN** a request and a response are written in their machine-readable form and read back
- **THEN** each is the same, a response's bytes included
