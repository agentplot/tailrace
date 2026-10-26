## ADDED Requirements

### Requirement: Each artifact is named by its ecosystem, its name and its version

A publish SHALL make, for each artifact of the version, the store name
`artifact/<ecosystem>/<name>@<version>` at the index's digest, made from
nothing and spelled from the artifact's metadata alone, naming no host,
store, component, prefix or repository (138, 3, 49, core.18).

#### Scenario: Two artifacts named

- **WHEN** a component whose two outputs pack as `eco` artifacts `site` and `cli` is published at version 1.3.0
- **THEN** the store's names `artifact/eco/site@1.3.0` and `artifact/eco/cli@1.3.0` each resolve to the index's digest, beside the version's tag at the same digest

#### Scenario: A consumer finds the artifact by name and version alone

- **WHEN** the name `artifact/eco/site@1.3.0` is resolved and the digest it names is read
- **THEN** the read answers an index naming one artifact of ecosystem `eco`, name `site` and version 1.3.0, and the claims attached to that index are read from it

#### Scenario: The same name for every publisher

- **WHEN** one revision is published by two hosts whose bindings name different stores
- **THEN** each store holds the artifact under the same name, which holds neither store's name nor the component's prefix

### Requirement: Every artifact name is read before any is made

A publish SHALL resolve every artifact name of the version before it
makes any. A name already at this index SHALL be left as it is. A name
at another digest SHALL refuse the version, naming the name, the digest
it is at and this index, and no name SHALL be made (70, 109, core.6).

#### Scenario: A name another index holds

- **WHEN** a store already names `artifact/eco/site@1.3.0` at another index and a component packing `site` at 1.3.0 is published
- **THEN** the publish is refused naming the name, the other index's digest and this index's, no artifact name is made, the version's tag is not made, and the store is asked to move no name

#### Scenario: A name already at this index

- **WHEN** a publish stopped after making one artifact's name, and the revision is published again
- **THEN** that name is not moved again, the other artifact's name is made from nothing, and the version's tag is made last

#### Scenario: A make refused by the store

- **WHEN** the fake store refuses the make of an artifact's name, naming the digest the name is at
- **THEN** the publish answers that refusal with the store's words, the make is not asked again, and the version's tag is not made

### Requirement: The index names its revision and its repository

A new index SHALL name the revision it was published from, and the
publishing repository by the owner and name of its landed branch's
remote's address, as the address is written (13, 138). Where the branch
names no remote, or its address gives no owner and name, the index SHALL
name no repository, and the publish SHALL go on.

#### Scenario: A repository with a remote

- **WHEN** a revision of a repository whose branch's remote is `https://example.com/acme/site.git` is published, on a host whose git rewrites that address to a local path
- **THEN** the index names the revision and the repository `acme/site`

#### Scenario: A repository with no remote

- **WHEN** a revision of a repository whose branch names no remote is published
- **THEN** the index names the revision and no repository, and the publish answers as it otherwise would

### Requirement: A declared component's version is named with its component

A publish of a declared component SHALL make the version's store name
`<component>:<tag>`, and a publish of a repository that declares none
the bare tag, as it did before; the gate's acceptance, a promotion, the
host's record and verify SHALL resolve a version by the same name (22,
125, C.1, core.4).

#### Scenario: A declared component published and accepted

- **WHEN** the component `lib` with prefix `lib-v` is published at 1.2.0, and the gate is asked to accept `lib` 1.2.0
- **THEN** the fake store's name `lib:lib-v1.2.0` resolves to the index, the store holds no name `lib-v1.2.0`, and the gate resolves `lib:lib-v1.2.0` to that index

#### Scenario: A version held under its bare tag

- **WHEN** the store holds a declared component's index under the bare tag `lib-v1.2.0`, as a publish before this change left it, and the revision is published again with projections that pack the same
- **THEN** `lib:lib-v1.2.0` is made from nothing at that index after the artifacts' names, no index is put, the bare tag still names the index, and the answer names the same index

## MODIFIED Requirements

### Requirement: The index is offered last, then the version is named

The index SHALL be put only after every artifact it names is put and its
claims attached. The publisher's layout-policy claim, where one is
attached, SHALL be attached next; then each artifact's name SHALL be
made at the index's digest; and only then SHALL the version's name be
moved in the store from nothing to the index's digest. A version exists
only once its index is held, and a publish that stops before its index
has published nothing (6, 17, 113, 138, core.12, core.6).

#### Scenario: The order of puts

- **WHEN** a component with two outputs is published
- **THEN** the fake store records both artifacts and their attachments before the index, the index before either artifact's name, and both artifacts' names before the version's name

#### Scenario: A stopped publish

- **WHEN** a publish stops after one artifact is put
- **THEN** no index is held, the version's name and every artifact's name resolve to nothing, and the host's record holds no entry for the version

### Requirement: A second publish of a revision ends at the same index

A publish of a revision whose version's name already names an index,
or for a declared component whose bare tag does, as a publish before
version names carried their component left it, SHALL compare each packed artifact's digest with the digest the index
names for that ecosystem name. All the same, it SHALL attach the
publisher's layout policy where the index lacks it, make each artifact's
name the store lacks at that index, write nothing else, and answer the
same index (5, 18, 113, 138, S2). One different, the publish SHALL be
refused naming the artifact and both digests, nothing SHALL be offered
or named, and the name SHALL stay with the first index (18, 87).

#### Scenario: The same revision again

- **WHEN** a revision is published twice with projections that pack the same
- **THEN** the second answer names the same index and digests, and the store, the history and the host's record hold after it exactly what they held before it

#### Scenario: A version named before its artifacts were

- **WHEN** the store holds a version's index and its tag but no artifact's name and no layout-policy claim, as a publish before this change left it, and the revision, whose file writes a policy for layouts, is published again
- **THEN** the layout-policy claim is attached to that index, each artifact's name is then made from nothing at it, nothing is put, and the answer names the same index

#### Scenario: A projection that packs differently

- **WHEN** a revision is published twice and the fake projection packs different bytes the second time
- **THEN** the second publish is refused naming the artifact, the digest the index names and the digest packed now, the store records no put and no name, and the version's name still names the first index

### Requirement: A publish answers what a program reads

A publish SHALL end with an answer readable by a program that names, for
each component, the version, the tag, the revision, the index's digest,
each artifact's digest, ecosystem, name and store name, whether its
projection promises the same bytes again, and each claim with its kind,
its signer, its envelope's digest and the digest it is attached to (19,
57, 138).

#### Scenario: The answer names every digest

- **WHEN** a component with two outputs is published and its answer is read as JSON
- **THEN** it names the index's digest, the version, both artifacts' digests, ecosystem names and store names, and for each artifact the provenance claim's envelope digest and the artifact digest it is attached to, each equal to what the fake store holds

### Requirement: A publish offers and does nothing more

A publish SHALL never accept, promote, move a channel or reach a target,
and SHALL move no name in the store but its version's and its
artifacts' names (20, 138, core.10, I2).

#### Scenario: No channel moved

- **WHEN** a version of two artifacts is published with a target and a keeper bound
- **THEN** the fake keeper and the fake target record no call, and the fake store records three names made, the two artifacts' and the version's

### Requirement: The index carries the publisher's layout policy
Where the publishing repository's file writes a policy for layouts at
the landed revision, a publish SHALL sign it as a layout-policy claim about
the index, in the identity of the party running the publish and naming
that revision, and attach it to the index after the index is put and
before any artifact's name or the version's name is made (113, 8, 138,
core.8, core.9). Where the file writes none, or the revision holds no
policy file, it SHALL attach none. A publish whose index already carries
that claim SHALL attach nothing more.

#### Scenario: A publish under a policy for layouts
- **WHEN** a revision whose policy file declares a policy for layouts is published
- **THEN** the index carries one layout-policy claim by the publishing host naming the revision and the claims that policy requires, attached before the first artifact's name was made

#### Scenario: A publish with no layout line
- **WHEN** a revision whose policy file names a gate signer and no `[layout]` is published
- **THEN** the index carries no layout-policy claim, and the given is left for a consumer to lay beneath itself
