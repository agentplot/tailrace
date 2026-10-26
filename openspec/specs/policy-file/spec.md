# policy-file Specification

## Purpose
The policy seam over `.config/tailrace.toml` at a landed revision: the
signers the repository's authors name, what each may claim, and the
claims acceptance, a layout or a channel requires (C.10).

## Requirements

### Requirement: The policy is read from the file at the revision asked

The profile SHALL read `.config/tailrace.toml` from the tree of the
revision it is asked about, in the repository's history, and never from
a working tree (65, core.22).

#### Scenario: A checkout's edit is not read

- **WHEN** the working tree's policy file names a signer the file at the revision does not
- **THEN** the acceptance policy at that revision names only the signers of the committed file

### Requirement: Signers are named with their verifier and their reach

Each `[signers.<name>]` table SHALL name one signer: exactly one key
other than `kinds` and `checks` whose string value is the text the
signer's profile verifies by, `kinds` the claim kinds it may make, and
`checks` the checks it may claim a verdict of, by the landing's names
(82, 116). A signer is bound by no output and no ecosystem.

#### Scenario: A host key listed

- **WHEN** the file holds `[signers.laptop]` with `ssh = "ssh-ed25519 AAAA… host@laptop"` and `kinds = ["provenance"]`
- **THEN** a policy requiring provenance from `laptop` names a signer whose id is `laptop`, verified by `ssh-ed25519 AAAA… host@laptop`, who may claim provenance and no check

#### Scenario: A signer with two verifiers

- **WHEN** a `[signers.<name>]` table holds two keys besides `kinds` and `checks`
- **THEN** the answer is unreadable, naming the file, the revision and the table

### Requirement: Each audience lists the claims it requires, in order

`[acceptance]`, `[layout]` and `[channels.<name>]` SHALL each hold
`require`, a list of claims in the order the verdict reads them, each
with its `kind`, its `check` where it is a verdict, and the `signers` by
their names in `[signers]`; a channel's table MAY name `bound_to` (82,
83, 37). The acceptance and layout tables SHALL name no channel.

#### Scenario: Acceptance requires two claims

- **WHEN** `[acceptance]` requires provenance from `laptop`, then a verdict of the check `cli:test` from `box`
- **THEN** the acceptance policy answers those two claims in that order, each with its signers as `[signers]` names them

#### Scenario: A bound channel

- **WHEN** `[channels.stable]` names `bound_to = "beta"`
- **THEN** the policy for the channel `stable` names `beta` as the channel it is bound to

#### Scenario: Acceptance naming a channel

- **WHEN** `[acceptance]` names `bound_to`
- **THEN** the answer is unreadable, naming the file and the table

#### Scenario: A signer no table names

- **WHEN** a required claim names a signer `[signers]` does not hold
- **THEN** the answer is unreadable, naming the file, the revision and the signer

### Requirement: An audience with no table is absent

Where the revision holds no policy file, or its file holds no table for
the audience asked, the answer SHALL be absent naming the audience (82,
58). A file that cannot be read or parsed SHALL be unreadable naming the
file and the revision, with git's or the parser's own words (59).

#### Scenario: No layout policy

- **WHEN** the file holds `[acceptance]` and no `[layout]`
- **THEN** the policy for a layout is absent, naming the audience

#### Scenario: A malformed file

- **WHEN** the file at the revision is not well formed
- **THEN** the answer is unreadable, naming the file and the revision, with the parser's words verbatim

### Requirement: The profile passes the policy group

The profile SHALL pass the conformance suite's policy group, played
against a scratch repository whose commits hold the policy files the
group asks for (93, core.33).

#### Scenario: The group over the file

- **WHEN** the policy group runs against the profile over a scratch repository
- **THEN** every scenario of the group passes

### Requirement: A channel's table names its component

A `[channels.<name>]` table SHALL name the component the channel belongs
to with `component`, by a name a `[[component]]` table declares, and
MAY leave it out where the repository declares one component, or none:
the channel then belongs to the one declared, or to the one named
`repository` (123, 125, 131). A table that names no component where
several are declared, or names one the file does not declare, SHALL be
unreadable naming the file, the revision, the table and the components
the file declares. The profile reads only the `name` of each
`[[component]]` table.

#### Scenario: One component has every channel

- **WHEN** the file declares no `[[component]]`, and `[channels.beta]` and `[channels.stable]` name no component
- **THEN** both channels belong to the component `repository`

#### Scenario: The one declared component

- **WHEN** the file declares one `[[component]]` named `lib`, and `[channels.beta]` names no component
- **THEN** `beta` belongs to `lib`

#### Scenario: A channel naming no component among several

- **WHEN** the file declares `lib` and `web`, and `[channels.beta]` names no component
- **THEN** the channels are unreadable, naming the file, the revision, `[channels.beta]`, `lib` and `web`, and so is the policy for `beta`

#### Scenario: A channel naming a component not declared

- **WHEN** `[channels.beta]` names `component = "app"` and the file declares no component `app`
- **THEN** the channels are unreadable, naming `[channels.beta]` and `app`

### Requirement: A channel is bound within its component

A `bound_to` SHALL name a channel the file declares, of the same
component; one that names another component's channel, or a channel
the file does not declare, SHALL be unreadable naming the table and the
channel it names (37, 125). A channel's component and binding SHALL be
read the same way whether the channels are listed or one channel's
policy is asked, so the two never disagree (83).

#### Scenario: A binding across components

- **WHEN** `[channels.stable]` of `lib` names `bound_to = "edge"`, and `[channels.edge]` belongs to `web`
- **THEN** the channels are unreadable, naming `[channels.stable]` and `edge`, and so is the policy for `stable`

#### Scenario: A binding to a channel not declared

- **WHEN** `[channels.stable]` names `bound_to = "beta"` and the file holds no `[channels.beta]`
- **THEN** the channels are unreadable, naming `[channels.stable]` and `beta`

### Requirement: The profile lists the channels the file declares

The profile SHALL answer channels with one entry for each
`[channels.<name>]` table of the file at the revision asked, ordered by
name, each with its component and its `bound_to` where it names one
(131). Where the revision holds no policy file, or a file with no
`[channels]`, the answer SHALL be an empty list. A revision the
repository does not hold, or a file that cannot be read, SHALL answer
as policy-for answers it (58, 59).

#### Scenario: Two components' channels

- **WHEN** the file declares `lib` and `web`, `[channels.beta]` and `[channels.stable]` of `lib` with `stable` bound to `beta`, and `[channels.edge]` of `web`
- **THEN** the channels answer `beta` of `lib`, `edge` of `web` and `stable` of `lib` bound to `beta`, in that order

#### Scenario: No channel declared

- **WHEN** the file at the revision holds `[acceptance]` and no `[channels]`, or the revision holds no policy file
- **THEN** the channels answer an empty list

#### Scenario: A checkout's channel is not read

- **WHEN** the working tree's policy file declares a channel the file at the revision does not
- **THEN** the channels at that revision do not list it (65, core.22)
