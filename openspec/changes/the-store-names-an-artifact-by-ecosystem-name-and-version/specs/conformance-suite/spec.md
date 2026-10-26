## ADDED Requirements

### Requirement: Store naming over the fakes
A store-naming group SHALL hold the core to 138, and to 137's range,
over the fakes, below its marker: what a publish names is found by a
consumer by ecosystem, name and version alone, and a range resolves by
the store's listing under the layout's policy. 138 SHALL leave the
register, and 137's range its parts, in the commit whose scenario first
cites them.

#### Scenario: A pin and a range found through what a publish wrote
- **WHEN** a component whose two outputs pack as `eco` `site` and `cli` is published at 1.0.0 and at 1.1.0, a third version of `cli` is in the store refused by the policy, and a checkout pinning `site` 1.0.0 and declaring `cli` by the range `1.` is synced
- **THEN** the layout holds `site` 1.0.0 and the newest admitted `cli` within the range, each from the index its publish answered, the refused version is answered naming its claim, and nothing in the checkout's ask or the answer names a component, a tag prefix or a repository

#### Scenario: The register after store naming
- **WHEN** the coverage check runs after this group's scenarios are added
- **THEN** 138 is cited and not in the register, and no part of 137 is pending

## MODIFIED Requirements

### Requirement: S1 over the fakes

The publish group SHALL play S1 over the fakes, an in-memory store and
a git history: a landed revision with two outputs and a landing
verdict naming their build is published.

#### Scenario: S1, a landed revision published

- **WHEN** a revision landed with a verdict naming its build is published, its two outputs read from the build and packed
- **THEN** both artifacts carry the version history derives, the index is put after both artifacts and their claims and names the revision, each artifact has one provenance in the host's identity naming the revision and build identity with the landing's verdict carried unchanged, each artifact's name by its ecosystem, name and version is made at the index before the version's name is, the answer names every digest and where each claim is attached, and nothing else moved: no other name, no keeper append and no target call

### Requirement: The store's contract

The store's group SHALL hold a store to 66 to 73 and to the Part A
clauses its operations keep: the same bytes put twice are held once
under their own digest (1, 5); an artifact keeps the metadata that says
what it is (4); attestations are held beside their digest, all of them,
none altered (8, 10); a name moves only from where it is expected to be
(70, 71); bytes read back are the bytes put (73); and a name's versions
are listed (138).

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

#### Scenario: A name's versions listed

- **WHEN** an index is put and named `artifact/eco/@acme/site@1.0.0` and `artifact/eco/@acme/site@1.1.0+b.1`, another `artifact/eco/@acme/site-cli@1.0.0`, and the versions of `eco` `@acme/site` and of a name never named are asked
- **THEN** the first answer is 1.0.0 and 1.1.0+b.1 and no other, each resolving to the index, and the second is absent, never an empty list
