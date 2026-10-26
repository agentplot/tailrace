## MODIFIED Requirements

### Requirement: The policy's contract
The policy's group SHALL hold a policy to 82 and 83: for each audience,
the claims required with the signers that may make each and the kinds
and checks each may claim, binding no signer by output or ecosystem
(116); and, where the repository declares no policy for an audience,
an absence naming the audience, asked of a channel no policy declares,
since the policy file answers a layout with no table by the given
(C.10). The revision beside it is verify's to name, not the policy's
(82).

#### Scenario: The claims an audience requires
- **WHEN** the policy for acceptance is asked at a revision whose policy requires a provenance from one signer and a verdict for one check from another
- **THEN** it answers both requirements, each with its signers and what each may claim

#### Scenario: Acceptance names no channel
- **WHEN** the policy for acceptance and for a channel bound to another are asked
- **THEN** acceptance answers no channel, and the bound channel's answer names the channel it is bound to

#### Scenario: No policy for an audience
- **WHEN** the policy for the channel `nightly` is asked where no policy declares it
- **THEN** the answer is absent naming the audience, never refused

## ADDED Requirements

### Requirement: The projection's group holds pinned
The projection's group SHALL hold a projection to pinned, below its
marker: a checkout holding the ecosystem's lockfile answers what the rig
says it pins, as the lockfile spells it, and a checkout holding none
answers absent. The rig SHALL say whether its ecosystem has a lockfile,
and the lockfile scenario SHALL be not applicable to one that has none.
The scenarios cite 50 and 58 until the suite's statement list holds 137.

#### Scenario: Pinned as the lockfile spells it
- **WHEN** a projection whose ecosystem has a lockfile is asked for the pins of the rig's checkout holding one
- **THEN** it answers the names and versions the rig says that lockfile pins, and no other

#### Scenario: No lockfile, absent
- **WHEN** a projection is asked for the pins of a checkout holding no lockfile of its ecosystem
- **THEN** the answer is absent, never refused, unreadable or an empty list

#### Scenario: Every projection runs it
- **WHEN** the projection group runs over the fake projection, the file, image and moon cache profiles
- **THEN** the no-lockfile scenario passes for each, and the lockfile scenario passes over the fake set with pins and is not applicable to the three profiles

### Requirement: The layouts over the fakes
The layouts' group SHALL hold the core to 47, 50, 51, 61, 62, 63, 99 and
113 over the fakes, below its marker, playing S4, with a scenario for
each requirement of consumer layouts. 47, 51, 99 and 113 SHALL leave the
register in the commit whose scenario first cites them.

#### Scenario: S4 over the fakes
- **WHEN** a checkout whose fake projection pins two versions the store names under the consumer's policy is synced, and the store holds a third version of the name
- **THEN** the projection is asked to render exactly the two versions' artifacts at the checkout's layout place, and the store was asked nothing about the third

#### Scenario: The publisher's policy through the host's trust over the fakes
- **WHEN** the fake policy answers no policy for layouts and a pinned version's index carries layout-policy claims by a trusted and an untrusted signer of different content
- **THEN** the version is judged under the trusted signer's claim alone, and the answer names its revision

#### Scenario: The publisher's above the consumer's given over the fakes
- **WHEN** the fake policy answers a given policy for layouts and a pinned version's index carries a trusted signer's layout-policy claim
- **THEN** the version is judged under the publisher's policy, and where the index carries none, under the consumer's given

#### Scenario: A version refused over the fakes
- **WHEN** a pinned version's index lacks the acceptance claim the consumer's policy requires
- **THEN** the answer names the version and acceptance as the claim found wanting, and the render was not given it

#### Scenario: The store unreachable over the fakes
- **WHEN** a layout was rendered and the fake store's resolve is then unreadable for a second sync
- **THEN** the second answer names what the layout holds, when it was rendered and the store's own words, the projection was not asked to render, and nothing is named refused

#### Scenario: The register after the layouts
- **WHEN** the coverage check runs after this group's scenarios are added
- **THEN** 47, 51, 99 and 113 are cited and none of them is in the register
