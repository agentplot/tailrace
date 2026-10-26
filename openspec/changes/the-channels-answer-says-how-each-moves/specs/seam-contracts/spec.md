# Spec Delta

## MODIFIED Requirements

### Requirement: The types that cross a seam

Every value an operation takes or gives SHALL be a type defined with the
seams, in the statement's words, and nowhere else: digest, revision,
branch, component, output, build identity, version, ecosystem and
ecosystem name, artifact metadata and index metadata (4, 6), cache
entry (52), claim and signed envelope (7, 8), verdict (78), signer,
audience and the policy's requirements (82), the channel a policy
declares with how it moves (131), environment and channel. A channel
SHALL be its component and its name together, its component none where
the repository declares no component (125). The name of the one
component of a repository that declares none (123) SHALL be defined
with the seams, so every profile and the core name it alike. Every such
type SHALL be readable by a program (57). A cache entry SHALL be named
by its build identity and its digest alone, and SHALL carry no
ecosystem, name or version (53).

#### Scenario: A value written and read back

- **WHEN** any value of these types is written in its machine-readable form and read back
- **THEN** it is the same value

#### Scenario: A cache entry is no artifact

- **WHEN** a cache entry is written in its machine-readable form
- **THEN** it names a build identity and a digest and nothing else, and reading it as artifact metadata is rejected

#### Scenario: A declared channel names its component

- **WHEN** a channel a policy declares is written in its machine-readable form
- **THEN** it names the channel's name, its component or that the repository declares none, and how it moves, naming the channel it is bound to where it is bound and whether it takes prereleases where it moves on each publish

#### Scenario: A channel spelled

- **WHEN** the channel `beta` of no declared component and the channel `beta` of `lib` are spelled
- **THEN** they are `beta` and `lib/beta`, and the two are not equal

### Requirement: The policy's contract

The policy SHALL answer policy-for, given a repository at a landed
revision and an audience of acceptance, a channel with its component,
or a layout, with the claims required, each with the signers that may
make it and what each may claim (82), or absent where no policy is
declared. The policy for a channel SHALL name the signers that may sign
its promotion (32) and the signers that may ask for it, none meaning any
asker the gate verifies (129, C.10), and the policy for a channel bound to another SHALL
name the channel it is bound to; the policy for acceptance SHALL name no
channel (83). The policy SHALL answer channels, given a repository at a
landed revision, with every channel its policy declares or gives, once
each, ordered by name and then by component, each with the one
component it belongs to (125) and how it moves: on each accepted
publish, with releases alone or prereleases too, by a promotion alone,
or bound to a channel of the same component (37, 83, 131, C.10); a
repository that declares one component SHALL
have every channel in it, and one that declares none SHALL have every
channel in that one, its channels spelled by name alone (123).

#### Scenario: Acceptance names no channel

- **WHEN** policy-for is asked for acceptance
- **THEN** its answer names no channel, and requirements for acceptance cannot be made naming one

#### Scenario: The channels and policy-for agree on a binding

- **WHEN** channels answers a channel bound to another, and policy-for is asked for that channel at the same revision
- **THEN** both name the same channel it is bound to

#### Scenario: A channel's promoters

- **WHEN** policy-for is asked for a channel
- **THEN** it names the signers whose promotion of that channel counts, each with what it may claim

#### Scenario: A channel's approvers

- **WHEN** policy-for is asked for a channel whose policy names who may ask for it, and for one whose policy names none
- **THEN** the first names those signers, and the second names none, which is any asker the gate verifies

### Requirement: The keeper's contract

The keeper SHALL answer append, given a channel with its component, a
promotion claim and the digest of the promotion it names as its
predecessor, with the claim held at the head of that channel's chain,
or refused naming the claim at the head (120); and chain, given a
channel with its component, with every promotion claim of it in order
from the first, or absent when none is held (121). Two components'
channels of one name SHALL be two chains (125). A chain that holds two
claims naming one predecessor SHALL be refused as a fork, naming both,
never answered by choosing one (105, 122). A chain answered from the
copy the host last read, because the source holding the ledger did not
answer, SHALL say when that copy was read and carry the source's words
of why it did not answer, a channel the copy holds no claim of answered
as a chain of no claims saying so, never as an absence; a chain read
from the source itself SHALL say so. An append SHALL never be held by
such a copy alone (60 as amended at agentplot/blueprints b64d367, 61,
C.13, C.14).

#### Scenario: An append whose predecessor is not the head

- **WHEN** append is given a predecessor that is not the head of the channel's chain
- **THEN** it is refused naming the claim at the head, and the chain is unchanged

#### Scenario: A chain read as last read

- **WHEN** the source holding a chain the host has read cannot be reached, and the chain is asked
- **THEN** the answer is the claims as last read, the time of that read and the source's words, and an append is unreadable

#### Scenario: An empty chain read as last read

- **WHEN** the host last read a channel's chain holding no claim, and the source holding it then cannot be reached
- **THEN** the answer is a chain of no claims, the time of that read and the source's words, and not absent

#### Scenario: Two components' chains of one name

- **WHEN** a promotion is appended to `beta` of `lib` and the chain of `beta` of `web` is asked
- **THEN** the chain of `beta` of `web` is absent

### Requirement: A channel's name in the store

A channel's name in the store SHALL be `channel/` and the channel as it
is spelled: `channel/<channel>` where the repository declares no
component, and `channel/<component>/<channel>` where it declares
components; the same for the core and every ledger profile that walks
from it, and never the name of a version or a cache entry (3, 125,
C.14).

#### Scenario: Stable's name

- **WHEN** the store name of the channel `stable` of a repository that declares no component is asked
- **THEN** it is `channel/stable`

#### Scenario: A component's stable

- **WHEN** the store name of the channel `stable` of the component `lib` is asked
- **THEN** it is `channel/lib/stable`
