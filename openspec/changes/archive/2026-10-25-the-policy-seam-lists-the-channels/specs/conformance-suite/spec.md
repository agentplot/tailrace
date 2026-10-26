## MODIFIED Requirements

### Requirement: The policy's contract

The policy's group SHALL hold a policy to 82, 83 and 131: for each
audience, the claims required with the signers that may make each and
the kinds and checks each may claim, binding no signer by output or
ecosystem (116); where the repository declares no policy for an
audience, an absence naming the audience, the revision beside it being
verify's to name, not the policy's (82); and the channels the policy
declares, each with the one component it belongs to and the channel it
is bound to within that component, one component holding every channel
where the repository declares one or none, and an empty list where it
declares no channel (125, 37). The group's rig SHALL declare components
and channels at a landed revision as well as policies for audiences.

#### Scenario: The claims an audience requires

- **WHEN** the policy for acceptance is asked at a revision whose policy requires a provenance from one signer and a verdict for one check from another
- **THEN** it answers both requirements, each with its signers and what each may claim

#### Scenario: Acceptance names no channel

- **WHEN** the policy for acceptance and for a channel bound to another are asked
- **THEN** acceptance answers no channel, and the bound channel's answer names the channel it is bound to

#### Scenario: No policy for an audience

- **WHEN** the policy for a layout is asked where none is declared
- **THEN** the answer is absent naming the audience, never refused

#### Scenario: The channels a policy declares

- **WHEN** the channels are asked at a revision declaring the components `lib` and `web`, `beta` and `stable` of `lib` with `stable` bound to `beta`, and `edge` of `web`
- **THEN** the answer is those three channels by name, each with its component and `stable` with its binding, and the policy for `stable` names `beta` as the channel it is bound to

#### Scenario: One component has every channel

- **WHEN** the channels `beta` and `stable` are declared naming no component, at one revision declaring no component and at another declaring the one component `lib`
- **THEN** both belong to `repository` at the first and to `lib` at the second

#### Scenario: A policy declaring no channel

- **WHEN** the channels are asked at a revision whose policy declares acceptance and no channel
- **THEN** the answer is an empty list, not an absence

#### Scenario: The channels of a policy that cannot be reached

- **WHEN** the rig makes the policy unreachable and the channels are asked
- **THEN** the answer is unreadable naming the policy, with its words unchanged, and not an empty list
