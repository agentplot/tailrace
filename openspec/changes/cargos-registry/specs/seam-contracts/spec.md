## MODIFIED Requirements

### Requirement: One interface per seam
Each seam of Part B SHALL be exactly one interface holding that seam's
operations as the statement names them and no other: the store (66 to
72, and the listing of the names it holds, 138), the build (74, 75), the
landing (77, 78), the signer (79, 80), the policy (82), a projection (84
to 86, 137), a target (88 to 91) and the keeper (120, 121). A profile
implements one interface and nothing of the core (core.2).

#### Scenario: Each seam's operations
- **WHEN** a reader lists the operations of each seam's interface
- **THEN** the store has put-bytes, put-artifact, attach, claims-of, name, resolve, read and names; the build has ensure and report; the landing has landed and verdicts-of; the signer has sign and verify; the policy has policy-for; a projection has pack, render, serve and pinned; a target has desired, current, apply and report; the keeper has append and chain; and no interface has any other

#### Scenario: The seams name no tool
- **WHEN** the seams' code is searched for the names core.1 rules out
- **THEN** none appears, and the code imports nothing of any profile

### Requirement: The store's contract

The store SHALL answer put-bytes with the digest and whether it held
the bytes already (66, 5); put-artifact with the artifact's digest
(67); attach with the attestation held beside its digest (68, 8);
claims-of with every attestation as attached, or absent (69); name
with the name moved, or refused naming where it is (70); resolve with
a digest, or absent (71); read with what the digest holds, or
absent (72); and names, given a prefix, with every name it holds that
begins with it, in no order a reader relies on, none where it holds
none, and unreadable where it cannot list (138, 23, 58).

#### Scenario: A name moved only from where it is expected

- **WHEN** name is asked to move a name from a digest it is not at
- **THEN** the answer is refused, naming the digest the name is at, and the name does not move

#### Scenario: A name made from nothing

- **WHEN** name is asked to move a name that names nothing, from nothing
- **THEN** the name is made

#### Scenario: An artifact read back

- **WHEN** read is asked for the digest put-artifact answered
- **THEN** it answers the artifact's metadata and the digests it names

#### Scenario: The names under a prefix

- **WHEN** a store holding the names `artifact/cargo/fam-a@1.0.0`, `artifact/cargo/fam-a@1.1.0`, `artifact/cargo/fam-ab@1.0.0` and `fam-v1.0.0` is asked for the names under `artifact/cargo/fam-a@`
- **THEN** it answers the first two and no other, and asked under a prefix no name begins with, it answers none

### Requirement: A projection's contract
A projection SHALL answer pack, given an output at a build identity and,
where the bytes are an artifact's, the version, a way to ask whether the
component's paths hold a path of the repository, and a way to ask the
artifacts the store holds of a name in the projection's ecosystem, with
the ecosystem's own bytes, their metadata only where a version was
given, and whether the same outputs and version, or the same build
identity, pack to the same bytes (84, 87, 100, 110, 123, 138); render,
given artifacts and their metadata or cache entries by build identity,
with a consumer layout and the versions or build identities it holds
(85); serve, given a layout, one request of the ecosystem's own client
with the base address it is served at, and a way to ask the core for a
version the layout lacks, with the response to that request in the
client's own protocol, or absent naming serving, for every request, from
a projection whose ecosystem's clients read files (86, 115); and pinned,
given a checkout, with the ecosystem names and versions its lockfile
pins as the lockfile spells them, or absent where the checkout holds no
lockfile of that ecosystem (137). A projection SHALL never reach the
store but through the store it is handed, nor the policy or a version's
claims at all, nor match a component's paths or resolve a store's names
itself: a version a served layout lacks is asked of the core, which
judges it, and a component's paths and the store's artifacts of a name
are asked of the core through pack's input (core.2).

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
