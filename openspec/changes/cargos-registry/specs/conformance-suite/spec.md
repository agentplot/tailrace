## MODIFIED Requirements

### Requirement: A projection's contract

The projection's group SHALL hold a projection to 84 to 87, and to the
parts of 48 and 50 a projection keeps: a layout rendered again from the
same artifacts is the same layout, and holds the versions it was given
and no other. It SHALL hold a projection whose rig says it packs cache
entries to their case of 84, 85 and 87, and one whose rig says it packs
none to answering absent for one; each case's scenarios are not
applicable to a projection of the other. The rig SHALL name its
ecosystem's request for a version's bytes, since a request's form is
the ecosystem's.

#### Scenario: Packed the same again

- **WHEN** one output is packed twice with one version
- **THEN** both answers carry the same bytes and metadata naming the ecosystem, name and version, or the projection says it cannot promise the same bytes

#### Scenario: A layout of the versions given

- **WHEN** a layout is rendered from two artifacts and rendered again from the same two
- **THEN** both renders hold those two versions and no other, and are the same layout

#### Scenario: Served where the clients read no files

- **WHEN** a projection whose clients read no files renders a layout and is asked to serve the rig's request for a version it holds
- **THEN** it answers a response that answers that request, and asks the core nothing

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

## ADDED Requirements

### Requirement: The projection's group holds serve
The projection's group SHALL hold a projection to serve, below its
marker, over the rig's own request for a version: a version the layout
holds is answered and asks the core nothing (86, 46); a version it lacks
is asked of the core once, and answered once the core renders it (115,
50); a refusal the core answers is carried in the response in its own
words (99, 59); and one layout served under one base from two places
answers the same and names neither place (49). Each scenario SHALL be
not applicable to a projection whose clients read files, which SHALL
answer absent naming serving.

#### Scenario: A version held, served
- **WHEN** a projection that serves renders two of the rig's artifacts and is asked to serve the rig's request for one
- **THEN** the response answers it, and the core was asked nothing

#### Scenario: A version lacked, asked of the core
- **WHEN** a projection that serves renders one of two artifacts and is asked for the other, the core rendering it on the ask
- **THEN** the core was asked once for that version's ecosystem name and version, and the response answers it

#### Scenario: A refusal in the response
- **WHEN** a projection that serves is asked for a version it lacks and the core answers it refused with the words `acceptance wanting`
- **THEN** the response is no answer of the version and carries `acceptance wanting` verbatim

#### Scenario: One base, two places
- **WHEN** one set of artifacts is rendered at two places and the same request is served from each under one base
- **THEN** the two responses are the same, and neither names either place

#### Scenario: Every projection runs serve
- **WHEN** the projection group runs over the fake projection, the Cargo projection, and the file, image and moon cache profiles
- **THEN** the serve scenarios pass over the fake and Cargo, and are not applicable to the three profiles, each of which answers absent naming serving

### Requirement: The Cargo projection runs the projection group
The Cargo projection SHALL run the projection group whole: its render,
serve, requires, satisfying and unreachable scenarios over crates a test
makes, and its pack and pinned scenarios with the cargo the tree pins,
in `check-tools` (93, core.33).

#### Scenario: The group over Cargo
- **WHEN** the conformance tests of the Cargo projection run
- **THEN** every scenario of the projection group passes or is not applicable with its reason, and none is left out

### Requirement: The layouts' group holds a range and the closure
The layouts' group SHALL hold the core, below its marker, over the
fakes, to holding the newest admitted version within a range by the
store's listing, and to holding what each held version requires through
every level (137, 138, 140). 137's range and closure part SHALL leave
the register in the commit whose scenarios first cite it.

#### Scenario: The newest admitted within a range over the fakes
- **WHEN** the fake projection answers a range of `site` within which the store lists 1.0.0 and 1.1.0, admitted, and 1.2.0, refused, and lists 2.0.0 outside it
- **THEN** the projection is asked to render `site` 1.1.0 alone, the answer names 1.2.0 refused with its claim, and 2.0.0 was never judged

#### Scenario: A pinned version's dependencies, two levels down, over the fakes
- **WHEN** the fake projection pins `app` 1.0.0, which requires `lib`, whose newest admitted version within the range requires `base`
- **THEN** the projection is asked to render all three, and the answer names each range with the version that required it

### Requirement: The layouts' group holds a render on a miss
The layouts' group SHALL hold the core to a render on a miss, below its
marker, over the fakes: a version the policy admits is added to the
layout and the record and only it, and what it requires that the layout
does not meet, is read; a refused one is answered naming the claim and
changes nothing; and the served layout's address is the core's name and
port with the repository in the path (115, 50, 47, 99, 114, 49). 46,
49, 114 and 115 SHALL leave the register in the commit whose scenario
first cites them, and the parts of 114 and 115 npm's unit builds, sync
starting the host's process and the process staying while a layout is
served, SHALL be named among the register's parts as npm, rendered and
served's.

#### Scenario: A miss over the fakes
- **WHEN** a layout rendered holding one version is asked for a second the policy admits
- **THEN** the projection is asked to render both versions' artifacts, the record names both, and the fake store was asked about the second alone

#### Scenario: A refused miss over the fakes
- **WHEN** a layout is asked for a version whose index lacks the claim the policy requires
- **THEN** the answer names that claim as wanting, and the projection was asked to render nothing

#### Scenario: The served address over the fakes
- **WHEN** the served address is asked for two remotes of one repository
- **THEN** both answer one address at port 7420 with the owner and name in the path

#### Scenario: The register after serving
- **WHEN** the coverage check runs after these scenarios are added
- **THEN** 46, 49, 114 and 115 are cited and none of them is in the register, and the parts of 114 and 115 left to npm's unit are reported pending
