## MODIFIED Requirements

### Requirement: One interface per seam

Each seam of Part B SHALL be exactly one interface holding that seam's
operations as the statement names them and no other: the store (66 to
72), the build (74, 75), the landing (77, 78), the signer (79, 80), the
policy (82, 131), a projection (84 to 86), a target (88 to 91) and the
keeper (120, 121). A profile implements one interface and nothing of
the core (core.2).

#### Scenario: Each seam's operations

- **WHEN** a reader lists the operations of each seam's interface
- **THEN** the store has put-bytes, put-artifact, attach, claims-of, name, resolve and read; the build has ensure and report; the landing has landed and verdicts-of; the signer has sign and verify; the policy has policy-for and channels; a projection has pack, render and serve; a target has desired, current, apply and report; the keeper has append and chain; and no interface has any other

#### Scenario: The seams name no tool

- **WHEN** the seams' code is searched for the names core.1 rules out
- **THEN** none appears, and the code imports nothing of any profile

### Requirement: The types that cross a seam

Every value an operation takes or gives SHALL be a type defined with the
seams, in the statement's words, and nowhere else: digest, revision,
branch, component, output, build identity, version, ecosystem and
ecosystem name, artifact metadata and index metadata (4, 6), cache
entry (52), claim and signed envelope (7, 8), verdict (78), signer,
audience and the policy's requirements (82), the channel a policy
declares with its component and binding (131), environment and
channel. The name of the one component of a repository that declares
none (123) SHALL be defined with the seams, so every profile and the
core name it alike. Every such type SHALL be readable by a program
(57). A cache entry SHALL be named by its build identity and its digest
alone, and SHALL carry no ecosystem, name or version (53).

#### Scenario: A value written and read back

- **WHEN** any value of these types is written in its machine-readable form and read back
- **THEN** it is the same value

#### Scenario: A cache entry is no artifact

- **WHEN** a cache entry is written in its machine-readable form
- **THEN** it names a build identity and a digest and nothing else, and reading it as artifact metadata is rejected

#### Scenario: A declared channel names its component

- **WHEN** a channel a policy declares is written in its machine-readable form
- **THEN** it names the channel, the one component it belongs to, and the channel it is bound to or that it is bound to none

### Requirement: The policy's contract

The policy SHALL answer policy-for, given a repository at a landed
revision and an audience of acceptance, a channel or a layout, with the
claims required, each with the signers that may make it and what each
may claim (82), or absent where no policy is declared. The policy for a
channel bound to another SHALL name the channel it is bound to, and the
policy for acceptance SHALL name no channel (83). The policy SHALL
answer channels, given a repository at a landed revision, with every
channel its policy declares, once each and ordered by name, each with
the one component it belongs to (125) and the channel it is bound to
where it is one, which belongs to the same component (37, 83); a
repository that declares one component SHALL have every channel in it,
and one that declares none SHALL have every channel in the one
component named `repository` (123, 131). A policy that declares no
channel SHALL answer an empty list, never an absence.

#### Scenario: Acceptance names no channel

- **WHEN** policy-for is asked for acceptance
- **THEN** its answer names no channel, and requirements for acceptance cannot be made naming one

#### Scenario: The channels and policy-for agree on a binding

- **WHEN** channels answers a channel bound to another, and policy-for is asked for that channel at the same revision
- **THEN** both name the same channel it is bound to
