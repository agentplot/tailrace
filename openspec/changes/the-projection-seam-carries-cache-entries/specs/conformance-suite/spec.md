# Spec Delta

## MODIFIED Requirements

### Requirement: A projection's contract

The projection's group SHALL hold a projection to 84 to 87, and to the
parts of 48 and 50 a projection keeps: a layout rendered again from the
same artifacts is the same layout, and holds the versions it was given
and no other. It SHALL hold a projection whose rig says it packs cache
entries to their case of 84, 85 and 87, and one whose rig says it packs
none to answering absent for one; each case's scenarios are not
applicable to a projection of the other.

#### Scenario: Packed the same again

- **WHEN** one output is packed twice with one version
- **THEN** both answers carry the same bytes and metadata naming the ecosystem, name and version, or the projection says it cannot promise the same bytes

#### Scenario: A layout of the versions given

- **WHEN** a layout is rendered from two artifacts and rendered again from the same two
- **THEN** both renders hold those two versions and no other, and are the same layout

#### Scenario: Served where the clients read no files

- **WHEN** a projection whose clients read no files serves a rendered layout
- **THEN** it answers an address its clients read

#### Scenario: A cache entry packed with no version

- **WHEN** a projection that packs cache entries packs one output twice at one build identity with no version
- **THEN** neither answer carries artifact metadata, and both carry the same bytes, or the projection says it cannot promise the same bytes

#### Scenario: A layout of the build identities given

- **WHEN** a projection that packs cache entries renders a layout from two cache entries, each named by a build identity and the digest of the bytes packed at it, and renders it again from the same two
- **THEN** both renders hold those two build identities and no other, and are the same layout

#### Scenario: A projection that packs no cache entries

- **WHEN** a projection that packs no cache entries is asked to pack an output with no version and to render a cache entry
- **THEN** both answers are absent, never refused and never unreadable

#### Scenario: The fakes in both cases

- **WHEN** the projection group runs over the fake projection and over the fake projection scripted to pack no cache entries
- **THEN** every scenario passes or is not applicable to that case, and each cache-entry scenario passes in one of the two runs
